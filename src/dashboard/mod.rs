use anyhow::Result;
use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use axum_template::engine::Engine;
use std::net::SocketAddr;
use tokio::{net::TcpListener, signal};
use tower_http::services::ServeDir;
use tracing::info;

mod examples;
mod routers;
mod templates;
mod tplvars;

/// handle_notfound returns a not found response.
async fn handle_notfound() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Page not found")
}

/// start_server starts the server.
pub async fn start_server(
    addr: SocketAddr,
    assets_dir: &str,
    tpl_dir: Option<String>,
) -> Result<()> {
    // extract templates
    let hbs = templates::new_handlebar(assets_dir, tpl_dir.clone())?;
    // set static assets directory
    let static_assets_dir = format!("{}/static", tpl_dir.unwrap_or(assets_dir.to_string()));

    let app = Router::new()
        .route("/", get(routers::index::index))
        .route(
            "/install",
            get(routers::install::page).post(routers::install::handle),
        )
        .route("/installed", get(routers::install::installed))
        .route(
            "/sign-in",
            get(routers::auth::sign_in).post(routers::auth::handle_sign_in),
        )
        .route("/sign-out", get(routers::auth::sign_out))
        .route("/projects", get(routers::projects::index))
        .route("/projects/:name", get(routers::projects::single))
        .route(
            "/projects/:name/settings",
            get(routers::projects::settings).post(routers::projects::handle_settings),
        )
        .route("/projects/:name/envs", post(routers::projects::handle_envs))
        .route("/new", get(routers::projects::new))
        .route("/new/:name", get(routers::projects::handle_new))
        .route("/settings", get(routers::settings::index))
        .route("/settings/tokens", post(routers::settings::handle_token))
        .route("/admin", get(routers::admin::index))
        .route("/admin/general", get(routers::admin::general))
        .route("/admin/workers", get(routers::admin::workers))
        .route("/admin/workers/tokens", post(routers::admin::handle_token))
        .route("/admin/domains", post(routers::admin::update_domains))
        .route("/admin/storage", post(routers::admin::update_storage))
        .route("/_worker_api/ping", post(routers::worker_api::ping))
        .nest_service("/static", ServeDir::new(static_assets_dir))
        .fallback(handle_notfound)
        .route_layer(axum::middleware::from_fn(routers::auth::middle))
        .route_layer(axum::middleware::from_fn(routers::install::middle))
        .route_layer(axum::middleware::from_fn(routers::logger))
        .with_state(Engine::from(hbs));

    info!("Listening on {}", addr);
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Ctrl-C received, shutting down");
        },
        _ = terminate => {
            info!("SIGTERM received, shutting down");
        },
    }
}
