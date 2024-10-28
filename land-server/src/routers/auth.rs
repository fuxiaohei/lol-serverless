use super::{install::InstallFlag, HtmlMinified, ServerError};
use crate::{
    routers::{error_htmx, hx_redirect, redirect},
    templates::Engine,
};
use axum::{
    extract::{OriginalUri, Request},
    middleware::Next,
    response::{IntoResponse, Response},
    Form,
};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    CookieJar,
};
use http::StatusCode;
use land_tplvars::{BreadCrumbKey, Empty};
use serde::Deserialize;
use tracing::{debug, warn};

/// SESSION_COOKIE_NAME is the session cookie name
pub static SESSION_COOKIE_NAME: &str = "_runtime_land_session";

/// sign_in shows sign-in page
pub async fn sign_in(engine: Engine) -> Result<impl IntoResponse, ServerError> {
    Ok(HtmlMinified(
        "sign-in.hbs",
        engine,
        Empty::new_vars("Sign In", BreadCrumbKey::None, None),
    ))
}

/// SignInForm is the form from sign-in page
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
    let user =
        land_dao::users::get_by_email(&f.email, Some(land_dao::users::UserStatus::Active)).await?;
    if user.is_none() {
        warn!("user is not found: {}", f.email);
        return Ok(error_htmx("User is not found or inactive").into_response());
    }
    let user = user.unwrap();
    if !land_dao::users::verify_password(&user, &f.password) {
        warn!("password is invalid: {}", f.email);
        return Ok(error_htmx("Password is invalid").into_response());
    }

    // create new session
    let session = land_dao::tokens::create_session(user.id, 3600 * 24).await?;
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

    let token =
        land_dao::tokens::get_by_value(session_value, Some(land_dao::tokens::Usage::Session))
            .await?;
    if token.is_none() {
        warn!("session token not found when sign-out");
        return Ok((
            jar.remove(Cookie::from(SESSION_COOKIE_NAME)),
            redirect("/sign-in"),
        )
            .into_response());
    }
    let token = token.unwrap();
    land_dao::tokens::set_expired(token.id, &token.name).await?;
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
    let method = request.method().to_string();

    // skip static assets
    if path.starts_with("/static/") {
        // debug!("auth skip path: {}", path);
        return Ok(next.run(request).await);
    }

    // worker api check auth token, not session
    if path.starts_with("/_worker_api") {
        return worker_auth(request, next).await.map_err(|status_code| {
            ServerError::status_code(status_code, &status_code.to_string())
        });
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
            let user = land_dao::users::verify_session(session_value).await;
            if user.is_ok() {
                let user = user.unwrap();
                debug!(
                    path = path,
                    "session verified user: {:?}, last_login_at: {:?}",
                    user.name,
                    user.last_login_at
                );
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
    let user = land_dao::users::verify_session(session_value).await;
    if user.is_err() {
        warn!(path = path, "Session is invalid: {}", session_value);
        // session is invalid, redirect to sign-out page to remove session
        return Ok(redirect("/sign-out").into_response());
    }

    let user = user.unwrap();
    debug!(
        path = path,
        "session verified user: {:?}, last_login_at: {:?}", user.name, user.last_login_at
    );
    let session_user = land_tplvars::User::new(&user);

    // check admin path
    if path.starts_with("/admin") && !session_user.is_admin {
        warn!(path = path, "User is not admin: {}", session_user.email);
        if method != "GET" {
            return Err(ServerError::status_code(
                StatusCode::UNAUTHORIZED,
                "Restricted access",
            ));
        }
        return Ok(redirect("/").into_response());
    }

    request.extensions_mut().insert(session_user);
    Ok(next.run(request).await)
}

async fn worker_auth(request: Request, next: Next) -> Result<Response, StatusCode> {
    let path = if let Some(path) = request.extensions().get::<OriginalUri>() {
        // This will include nested routes
        path.0.path().to_owned()
    } else {
        request.uri().path().to_owned()
    };

    let auth_header = request.headers().get("Authorization");
    if auth_header.is_none() {
        warn!(path = path, "No authorization header");
        return Err(StatusCode::UNAUTHORIZED);
    }
    let auth_value = auth_header
        .unwrap()
        .to_str()
        .unwrap()
        .trim_start_matches("Bearer ");
    if auth_value.is_empty() {
        warn!(path = path, "Authorization header is empty");
        return Err(StatusCode::UNAUTHORIZED);
    }

    // get worker token
    let token =
        match land_dao::tokens::get_by_value(auth_value, Some(land_dao::tokens::Usage::Worker))
            .await
        {
            Ok(t) => t,
            Err(e) => {
                warn!(path = path, "Error getting token: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };
    if token.is_none() {
        warn!(path = path, "Token not found");
        return Err(StatusCode::UNAUTHORIZED);
    }
    let token = token.unwrap();
    if land_dao::tokens::is_expired(&token) {
        warn!(path = path, "Token is expired");
        return Err(StatusCode::UNAUTHORIZED);
    }
    if token.status != land_dao::tokens::Status::Active.to_string() {
        warn!(path = path, "Token is not active");
        return Err(StatusCode::UNAUTHORIZED);
    }
    // check if the token is used in the last 60 seconds
    if chrono::Utc::now().timestamp() - token.latest_used_at.and_utc().timestamp() > 60 {
        match land_dao::tokens::set_usage_at(token.id).await {
            Ok(_) => {}
            Err(e) => {
                warn!(path = path, "Error update usage at: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    }
    Ok(next.run(request).await)
}