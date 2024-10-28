use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version)]
#[clap(disable_version_flag = true)] // handled manually
#[clap(
    name = env!("CARGO_PKG_NAME"),
    about = concat!(env!("CARGO_PKG_NAME")," ",env!("CARGO_PKG_VERSION")),
)]
struct Args {
    /// Print version info and exit.
    #[clap(short = 'V', long)]
    version: bool,
    #[clap(flatten)]
    output: land_helpers::logging::TraceArgs,
    /// Token that authenticate to land-server
    #[clap(long, env = "LAND_SERVER_TOKEN", default_value = "")]
    token: String,
    /// Address to listen on.
    #[clap(long, default_value("0.0.0.0:8642"))]
    address: String,
    /// Data directory
    #[clap(long, env = "LAND_DATA_DIR", default_value = "./data")]
    dir: String,
    /// The url of cloud server
    #[clap(long = "url",env = "LAND_SERVER_URL", value_parser = validate_url,default_value("http://127.0.0.1:9840"))]
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

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    if args.version {
        land_helpers::version::println(env!("CARGO_PKG_NAME"), args.output.verbose);
        return Ok(());
    }

    // Initialize tracing
    land_helpers::logging::init(args.output.verbose);

    /*
    // Initialize agent role
    agent::init_ip(args.ip).await?;
    agent::init_sync(
        args.server_url.clone(),
        args.token.clone(),
        args.dir.clone(),
    )
    .await;
    agent::init_task(
        args.server_url.clone(),
        args.token.clone(),
        args.dir.clone(),
        args.service_name.clone(),
    )
    .await;*/

    // Start server
    let opts = land_worker_server::Opts {
        addr: args.address.parse().unwrap(),
        dir: args.dir,
        default_wasm: None,
        enable_wasmtime_aot: true,
        endpoint_name: args.hostname,
        enable_metrics: true,
        metrics_addr: Some(args.metrics_addr),
    };
    land_worker_server::start(opts).await?;

    Ok(())
}
