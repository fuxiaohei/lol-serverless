use anyhow::Result;
use clap::Args;
use tracing::debug;

/// Worker command starts a worker node that connects to the land-server.
#[derive(Args, Debug)]
pub struct Worker {
    /// Token that authenticate to land-server
    #[clap(long, env = "LAND_SERVER_TOKEN", default_value = "")]
    token: String,
    /// Address to listen on.
    #[clap(long, default_value("0.0.0.0:4090"))]
    address: String,
    /// Data directory
    #[clap(long, env = "LAND_DATA_DIR", default_value = "./data")]
    dir: String,
    /// The url of cloud server
    #[clap(long = "url",env = "LAND_SERVER_URL", value_parser = validate_url,default_value("http://127.0.0.1:4040"))]
    pub server_url: String,
    /// The service name to generate traefik conf
    #[clap(
        long = "service-name",
        env = "LAND_SERVICE_NAME",
        default_value("land-worker@docker")
    )]
    pub service_name: String,
    /// Hostname
    #[clap(long = "hostname")]
    pub hostname: Option<String>,
    /// IP
    #[clap(long = "ip")]
    pub ip: Option<String>,
    /// Metrics listen address, default 0.0.0.0:9000
    #[clap(
        long = "metrics-addr",
        env = "LAND_METRICS_ADDR",
        default_value("0.0.0.0:9000")
    )]
    pub metrics_addr: String,
}

fn validate_url(url: &str) -> Result<String, String> {
    let _: url::Url = url.parse().map_err(|_| "invalid url".to_string())?;
    Ok(url.to_string())
}

impl Worker {
    pub async fn run(&self) -> Result<()> {
        debug!("start worker server flag: {:?}", self);

        // Start server
        let opts = crate::worker_server::Opts {
            addr: self.address.parse().unwrap(),
            dir: self.dir.clone(),
            default_wasm: None,
            enable_wasmtime_aot: true,
            endpoint_name: self.hostname.clone(),
            enable_metrics: true,
            metrics_addr: Some(self.metrics_addr.clone()),
        };
        crate::worker_server::start(opts).await?;
        Ok(())
    }
}
