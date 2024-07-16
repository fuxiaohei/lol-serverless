use anyhow::Result;
use clap::Parser;
use land_common::{logging, version};

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
    output: logging::TraceArgs,
    /// Token that authenticate to land-server
    #[clap(long, env = "LAND_SERVER_TOKEN", default_value = "")]
    token: String,
    /// Address to listen on.
    #[clap(long, default_value("0.0.0.0:9940"))]
    address: String,
    /// Data directory
    #[clap(long, env = "LAND_DATA_DIR", default_value = "./data")]
    dir: String,
    /// The url of cloud server
    #[clap(long = "url",env = "LAND_SERVER_URL", value_parser = validate_url,default_value("http://127.0.0.1:9840"))]
    pub server_url: String,
    /// Hostname
    #[clap(long = "hostname")]
    pub hostname: Option<String>,
    /// IP
    #[clap(long = "ip")]
    pub ip: Option<String>,
}

fn validate_url(url: &str) -> Result<String, String> {
    let _: url::Url = url.parse().map_err(|_| "invalid url".to_string())?;
    Ok(url.to_string())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    if args.version {
        version::print(env!("CARGO_PKG_NAME"), args.output.verbose);
        return Ok(());
    }

    // Initialize tracing
    logging::init(args.output.verbose);

    // Start server
    let opts = land_wasm_server::Opts {
        addr: args.address.parse().unwrap(),
        dir: args.dir,
        enable_wasmtime_aot: true,
        endpoint_name: args.hostname,
        ..Default::default()
    };
    land_wasm_server::start(opts).await?;

    Ok(())
}
