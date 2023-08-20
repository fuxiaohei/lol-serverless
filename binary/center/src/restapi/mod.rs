use axum::extract::DefaultBodyLimit;
use axum::response::IntoResponse;
use axum::{
    body::Body,
    http::{Request, Response, StatusCode},
    middleware,
    routing::{any, delete, get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use tracing::warn;

mod admin;
mod auth;
mod deployment;
mod params;
mod project;
mod register;
mod ws;

fn auth_router() -> Router {
    Router::new()
        .route("/v1/token/oauth", post(auth::create_oauth_token))
        .route("/v1/token/verify/:token", post(auth::verify_token))
        .route("/v1/signup", post(register::register))
        .route("/v1/login", post(register::login_by_email))
}

fn api_router() -> Router {
    Router::new()
        .route("/v1/home", get(default_handler))
        .route("/v1/token/deployment", post(auth::create_for_deployment))
        .route("/v1/token/deployment", get(auth::list_for_deployment))
        .route("/v1/token/deployment/:uuid", delete(auth::remove_token))
        .route("/v1/project", post(project::create_handler))
        .route("/v1/project/:name", get(project::query_handler))
        .route("/v1/project/:name/overview", get(project::overview_handler))
        .route("/v1/project/:name", delete(project::remove_handler))
        .route("/v1/project/:name/rename", post(project::rename_handler))
        .route("/v1/projects", get(project::list_handler))
        .route("/v1/deployment", post(deployment::create_handler))
        .route(
            "/v1/deployment/:uuid/publish",
            post(deployment::publish_handler),
        )
        .route(
            "/v1/deployment/:uuid/disable",
            post(deployment::disable_handler),
        )
        .route(
            "/v1/deployment/:uuid/enable",
            post(deployment::enable_handler),
        )
        .route("/v1/settings/regions", get(admin::list_regions))
        .route(
            "/v1/settings/region_tokens",
            get(admin::list_tokens_for_region),
        )
        .route(
            "/v1/settings/region_tokens",
            post(admin::create_token_for_region),
        )
        .route("/v1/settings/domains", get(admin::list_settings_domains))
        .route("/v1/settings/domains", post(admin::update_settings_domain))
        .route("/v1/settings/storage", get(admin::list_settings_storage))
        .route("/v1/settings/storage", post(admin::update_settings_storage))
        .route("/v1/settings/stats", get(admin::stats_handler))
        .route_layer(middleware::from_fn(auth::middleware))
}

/// default_handler is the default handler for all requests.
async fn default_handler(_req: Request<Body>) -> Response<Body> {
    let builder = Response::builder().status(404);
    builder.body(Body::from("Route Not Matched")).unwrap()
}

pub fn router() -> Router {
    let cors = CorsLayer::new()
        .allow_headers(Any)
        .allow_methods(Any)
        .allow_origin(Any);

    Router::new()
        .merge(auth_router())
        .merge(api_router())
        .route("/v1/region/ws", get(ws::ws_handler))
        .route("/", any(default_handler))
        .route("/*path", any(default_handler))
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024))
        .layer(cors)
}

// Make our own error that wraps `anyhow::Error`.
pub struct AppError(anyhow::Error, StatusCode);

#[derive(serde::Serialize)]
struct AppErrorJson {
    pub message: String,
}

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        warn!("app error: {:?}", self.0);
        (
            self.1,
            axum::Json::from(AppErrorJson {
                message: self.0.to_string(),
            }),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into(), StatusCode::INTERNAL_SERVER_ERROR)
    }
}
