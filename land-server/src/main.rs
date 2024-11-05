use anyhow::Result;
use clap::Parser;
use land_dao::DBArgs;
use land_utils::{logger, version};

mod routers;
mod server;
mod templates;

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
    /// Verbose mode.
    #[clap(flatten)]
    output: logger::TraceArgs,
    /// Address to listen on.
    #[clap(long, default_value("0.0.0.0:8844"))]
    address: String,
    /// Template directory
    #[clap(long)]
    tpldir: Option<String>,
    /// Database connection args.
    #[clap(flatten)]
    dbargs: DBArgs,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    if args.version {
        version::println(env!("CARGO_PKG_NAME"), args.output.verbose);
        return Ok(());
    }

    // initialize logger
    logger::init(args.output.verbose);

    // connect to database
    land_dao::connect(&args.dbargs)
        .await
        .expect("Failed to connect to database");

    // start http server
    server::start(args.address.parse()?, "./assets", args.tpldir.clone())
        .await
        .expect("Failed to start server");

    Ok(())
}
