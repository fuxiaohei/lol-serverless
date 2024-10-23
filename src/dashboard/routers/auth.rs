use super::{install::InstallFlag, HtmlMinified, ServerError};
use crate::dashboard::{
    routers::{error_html, hx_redirect, redirect},
    templates::Engine,
    tplvars::{AuthUser, BreadCrumbKey, Empty, Page, Vars},
};
use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Form,
};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    CookieJar,
};
use land_dao::{tokens, users};
use serde::Deserialize;
use tracing::{debug, warn};

/// SESSION_COOKIE_NAME is the session cookie name
pub static SESSION_COOKIE_NAME: &str = "__rt_land_session";

/// sign_in shows sign-in page
pub async fn sign_in(engine: Engine) -> Result<impl IntoResponse, ServerError> {
    Ok(HtmlMinified(
        "sign-in.hbs",
        engine,
        Vars::new(
            Page::new("Sign-in", BreadCrumbKey::None, None),
            Empty::default(),
        ),
    ))
}

#[derive(Debug, Deserialize)]
pub struct SignInForm {
    pub email: String,
    pub password: String,
}

/// handle_sign_in handles the sign-in logic
/// if success, redirect to homepage
pub async fn handle_sign_in(
    jar: CookieJar,
    Form(f): Form<SignInForm>,
) -> Result<impl IntoResponse, ServerError> {
    let user = users::get_by_email(&f.email, Some(users::UserStatus::Active)).await?;
    if user.is_none() {
        warn!("user is not found: {}", f.email);
        return Ok(error_html("User is not found or inactive").into_response());
    }
    let user = user.unwrap();
    if !users::verify_password(&user, &f.password) {
        warn!("password is invalid: {}", f.email);
        return Ok(error_html("Password is invalid").into_response());
    }

    // create new session
    let session = tokens::create_session(user.id, 3600 * 24).await?;
    let mut session_cookie = Cookie::new(super::auth::SESSION_COOKIE_NAME, session.value.clone());
    session_cookie.set_max_age(Some(time::Duration::days(1)));
    session_cookie.set_path("/");
    session_cookie.set_same_site(Some(SameSite::Strict));
    session_cookie.set_http_only(true);
    debug!(
        "sign-in session created: {:?}, {:?}",
        session, session_cookie
    );

    // redirect to home page
    let resp = hx_redirect("/")?;
    Ok((jar.add(session_cookie), resp).into_response())
}

/// sign_out handles the sign-out logic
pub async fn sign_out(jar: CookieJar) -> Result<impl IntoResponse, ServerError> {
    let session_value = jar
        .get(SESSION_COOKIE_NAME)
        .map(|c| c.value())
        .unwrap_or_default();
    if session_value.is_empty() {
        warn!("session is empty when sign-out");
        return Ok(redirect("/sign-in").into_response());
    }

    let token = tokens::get_by_value(session_value, Some(tokens::Usage::Session)).await?;
    if token.is_none() {
        warn!("session token not found when sign-out");
        return Ok((
            jar.remove(Cookie::from(SESSION_COOKIE_NAME)),
            redirect("/sign-in"),
        )
            .into_response());
    }
    let token = token.unwrap();
    tokens::set_expired(token.id, &token.name).await?;
    warn!("session is expired when sign-out");
    Ok((
        jar.remove(Cookie::from(SESSION_COOKIE_NAME)),
        redirect("/sign-in"),
    )
        .into_response())
}

/// middle is a middleware to check if the user is authenticated
pub async fn middle(mut request: Request, next: Next) -> Result<Response, ServerError> {
    // check install flag
    // if not installed, skip auth
    let flag = request.extensions().get::<InstallFlag>().cloned();
    if let Some(flag) = flag {
        if !flag.flag {
            return Ok(next.run(request).await);
        }
    }

    let uri = request.uri().clone();
    let path = uri.path();

    // skip static assets
    if path.starts_with("/static/") {
        // debug!("auth skip path: {}", path);
        return Ok(next.run(request).await);
    }

    // get session value
    let headers = request.headers();
    let jar = CookieJar::from_headers(headers);
    let session_value = jar
        .get(SESSION_COOKIE_NAME)
        .map(|c| c.value())
        .unwrap_or_default();

    // if path is /sign-*, it need validate session
    // if success, /sign-in redirects to homepage, /sign-out continues
    if path.starts_with("/sign") {
        // if session is exist, validate session in sign-in page
        if path.starts_with("/sign-in") && !session_value.is_empty() {
            debug!(path = path, "Session is exist when sign-in");
            let user = users::verify_session(session_value).await;
            if user.is_ok() {
                // session is ok, redirect to homepage
                return Ok(redirect("/").into_response());
            }
            debug!(path = path, "Session is invalid when sign-in, {:?}", user);
        }
        return Ok(next.run(request).await);
    }

    // session_value is empty, redirect to sign-in page
    if session_value.is_empty() {
        warn!(path = path, "session is empty");
        if request.method() != "GET" {
            // skip redirect for GET method
            return Err(ServerError::status_code(
                StatusCode::UNAUTHORIZED,
                "session is empty",
            ));
        }
        // no clerk session, redirect to sign-in page
        return Ok(redirect("/sign-in").into_response());
    }

    // after validation, it gets session user from session_id and set to request extensions
    let user = users::verify_session(session_value).await;
    if user.is_err() {
        warn!(path = path, "Session is invalid: {}", session_value);
        // session is invalid, redirect to sign-out page to remove session
        return Ok(redirect("/sign-out").into_response());
    }

    let user = user.unwrap();
    let session_user = AuthUser::new(&user);
    request.extensions_mut().insert(session_user);
    Ok(next.run(request).await)
}
