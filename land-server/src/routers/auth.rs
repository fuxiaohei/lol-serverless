use axum::{extract::{OriginalUri, Request}, http::StatusCode, middleware::Next, response::{IntoResponse, Response}};
use axum_extra::extract::CookieJar;
use tracing::{debug, warn};
use crate::routers::{install::InstallFlag, utils::redirect};
use super::ServerError;

/// SESSION_COOKIE_NAME is the session cookie name
pub static SESSION_COOKIE_NAME: &str = "_runtime_land_session";

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
