use crate::routers;
use anyhow::Result;
use std::net::SocketAddr;
use tokio::{net::TcpListener, signal};
use tracing::info;

/// start starts the server.
pub async fn start(addr: SocketAddr, assets_dir: &str, tpl_dir: Option<String>) -> Result<()> {
    let app = routers::new(assets_dir, tpl_dir).await?;
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
