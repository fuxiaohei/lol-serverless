use super::ServerError;
use super::{utils::redirect, HtmlMinified};
use crate::routers::auth::SESSION_COOKIE_NAME;
use crate::routers::utils::hx_redirect;
use crate::templates::Engine;
use axum::http::StatusCode;
use axum::Form;
use axum::{
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use axum_extra::extract::CookieJar;
use gravatar::{Gravatar, Rating};
use land_dao::settings;
use land_tplvars::BreadCrumbKey;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// index returns the install page.
pub async fn index(engine: Engine) -> Result<impl IntoResponse, ServerError> {
    Ok(HtmlMinified(
        "install.hbs",
        engine,
        land_tplvars::new_empty("Install", BreadCrumbKey::None, None),
    ))
}

/// InstallFlag is the flag for install status that can be used in request object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallFlag {
    pub flag: bool,
}

/// middle is a middleware to check if the app is installed.
pub async fn middle(mut request: Request, next: Next) -> Result<Response, ServerError> {
    let path = request.uri().path();

    // skip static assets
    if path.starts_with("/static/") {
        // debug!("auth skip path: {}", path);
        return Ok(next.run(request).await);
    }

    // check if is installed
    let is_installed = land_dao::settings::is_installed().await?;

    // if is installed, redirect to home page
    if is_installed {
        // if path is /install, redirect to home page
        if path == "/install" {
            warn!(path = path, "app is installed, redirect to home page");
            return Ok(redirect("/").into_response());
        }
        let flag = InstallFlag { flag: true };
        request.extensions_mut().insert(flag);
        return Ok(next.run(request).await);
    }

    // if not installed, redirect to install page
    if path != "/install" {
        return Ok(redirect("/install").into_response());
    }
    let flag = InstallFlag { flag: false };
    request.extensions_mut().insert(flag);
    Ok(next.run(request).await)
}

/// InstallForm is the form from install page
#[derive(Debug, Deserialize)]
pub struct InstallForm {
    pub password: String,
    #[serde(rename = "password-confirm")]
    pub password_confirm: String,
    pub email: String,
}

/// handle handles the install form
pub async fn handle(
    jar: CookieJar,
    Form(install_form): Form<InstallForm>,
) -> Result<impl IntoResponse, ServerError> {
    // check if password and password_confirm are equal
    if install_form.password != install_form.password_confirm {
        warn!("password and password_confirm are not equal");
        return Err(ServerError::status_code(
            StatusCode::BAD_REQUEST,
            "Password and password confirm are not equal",
        ));
    }
    debug!("install form: {:?}", install_form);

    // mark the app is installed
    settings::set_installed().await?;
    info!("app set installed");

    // init default settings values
    settings::init_defaults().await?;

    // use email name as username
    let user_name = install_form
        .email
        .split('@')
        .next()
        .unwrap_or("first-user")
        .to_string();

    let avatar = Gravatar::new(&install_form.email)
        .set_rating(Some(Rating::Pg))
        .image_url()
        .to_string();

    // create admin user
    let user = land_dao::users::create(
        user_name.clone(),
        user_name,
        install_form.email,
        avatar,
        Some(install_form.password),
        Some(land_dao::users::UserRole::Admin),
        land_dao::users::OAuthParams::default(),
    )
    .await?;

    debug!("install user created: {:?}", user);

    // create current session
    let session = land_dao::tokens::create_session(user.id, 3600 * 24).await?;
    let mut session_cookie = Cookie::new(SESSION_COOKIE_NAME, session.value.clone());
    session_cookie.set_max_age(Some(time::Duration::days(1)));
    session_cookie.set_path("/");
    session_cookie.set_same_site(Some(SameSite::Strict));
    session_cookie.set_http_only(true);
    debug!(
        "install session created: {:?}, {:?}",
        session, session_cookie
    );

    // redirect to home page
    let resp = hx_redirect("/")?;
    Ok((jar.add(session_cookie), resp).into_response())
}
