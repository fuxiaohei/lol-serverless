use anyhow::Result;
use axum::{routing::any, Router};
use land_service::memenvs;
use land_utils::get_hostname;
use metrics_exporter_prometheus::PrometheusBuilder;
use std::{net::SocketAddr, sync::OnceLock, time::Duration};
use tokio::net::TcpListener;
use tower_http::timeout::TimeoutLayer;
use tracing::{debug, info};

mod handle;
mod middle;

/// Opts for the worker server
pub struct Opts {
    pub addr: SocketAddr,
    pub dir: String,
    pub default_wasm: Option<String>,
    pub endpoint_name: Option<String>,
    pub enable_wasmtime_aot: bool,
    pub enable_metrics: bool,
    pub metrics_addr: Option<String>,
}

impl Default for Opts {
    fn default() -> Self {
        Self {
            addr: "127.0.0.1:8840".parse().unwrap(),
            dir: "./data".to_string(),
            default_wasm: None,
            endpoint_name: Some("localhost".to_string()),
            enable_wasmtime_aot: false,
            enable_metrics: false,
            metrics_addr: None,
        }
    }
}

static DEFAULT_WASM: OnceLock<String> = OnceLock::new();
static ENDPOINT_NAME: OnceLock<String> = OnceLock::new();
static ENABLE_WASMTIME_AOT: OnceLock<bool> = OnceLock::new();
static ENABLE_METRICS: OnceLock<bool> = OnceLock::new();

async fn init_opts(opts: &Opts) -> Result<()> {
    let hostname = if let Some(endpoint) = &opts.endpoint_name {
        endpoint.clone()
    } else {
        get_hostname()?
    };

    debug!("Endpoint: {}", hostname);
    debug!("Wasm dir: {}", opts.dir);
    debug!("Default wasm: {:?}", opts.default_wasm);
    debug!("Enable Wasmtime AOT: {}", opts.enable_wasmtime_aot);
    debug!("Enable Metrics: {}", opts.enable_metrics);

    // create directory
    std::fs::create_dir_all(&opts.dir).unwrap();

    DEFAULT_WASM
        .set(opts.default_wasm.clone().unwrap_or_default())
        .unwrap();
    ENDPOINT_NAME.set(hostname).unwrap();
    ENABLE_WASMTIME_AOT.set(opts.enable_wasmtime_aot).unwrap();
    ENABLE_METRICS.set(opts.enable_metrics).unwrap();
    land_wasm_host::FILE_DIR.set(opts.dir.clone()).unwrap();

    // init clients to handle fetch hostcall
    land_wasm_host::init_clients();

    // init wasmtime engine in memory.
    // some instances are live in one wasmtime engine.
    land_wasm_host::init_engines()?;

    // load envs to memory from local files
    memenvs::init_envs(format!("{}/envs", opts.dir)).await?;

    if opts.enable_metrics {
        let addr: SocketAddr = opts
            .metrics_addr
            .clone()
            .unwrap_or_else(|| "127.0.0.1:9000".to_string())
            .parse()
            .unwrap();
        PrometheusBuilder::new()
            .with_http_listener(addr)
            .install()?;
        info!("Metrics server started at {}", addr);
    }

    Ok(())
}

async fn load_default_wasm() -> Result<()> {
    let default_wasm = DEFAULT_WASM.get().unwrap();
    if default_wasm.is_empty() {
        debug!("No default wasm");
        return Ok(());
    }
    let aot_enable = ENABLE_WASMTIME_AOT.get().unwrap();
    let _ = land_wasm_host::Worker::new_in_pool(default_wasm, *aot_enable).await?;
    Ok(())
}

/// start worker server
pub async fn start(opts: Opts) -> Result<()> {
    init_opts(&opts).await?;

    // load default wasm
    load_default_wasm().await?;

    let app = Router::new()
        .route("/", any(handle::run))
        .route("/*path", any(handle::run))
        .layer(TimeoutLayer::new(Duration::from_secs(10)))
        .route_layer(axum::middleware::from_fn(middle::worker_info));
    let make_service = app.into_make_service_with_connect_info::<SocketAddr>();
    info!("Starting worker server on: {}", opts.addr);
    let listener = TcpListener::bind(opts.addr).await?;
    axum::serve(listener, make_service).await?;
    Ok(())
}
