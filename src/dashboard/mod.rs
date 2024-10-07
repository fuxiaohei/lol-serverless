use anyhow::Result;
use axum::{http::StatusCode, response::IntoResponse, routing::get, Router};
use axum_template::engine::Engine;
use std::net::SocketAddr;
use tokio::{net::TcpListener, signal};
use tower_http::services::ServeDir;
use tracing::info;

mod routers;
mod templates;

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
        .route("/", get(routers::index))
        .nest_service("/static", ServeDir::new(static_assets_dir))
        .fallback(handle_notfound)
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
