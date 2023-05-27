use clap::Parser;
use tracing::{debug, debug_span, info, Instrument};

#[derive(Parser, Debug)]
#[clap(name = "moni-runtime", version = moni_lib::version::get())]
struct Cli {
    #[clap(long, env("MONI_HTTP_ADDR"), default_value("127.0.0.1:38889"))]
    pub http_addr: String,
}

#[tokio::main]
async fn main() {
    moni_lib::tracing::init();

    let args = Cli::parse();

    debug!("load args: {:?}", args);

    // init storage
    moni_lib::storage::init()
        .await
        .expect("init storage failed");
    info!("Init storage success");

    moni_runtime::server::start(args.http_addr.parse().unwrap())
        .instrument(debug_span!("[Http]"))
        .await
        .unwrap();
}
