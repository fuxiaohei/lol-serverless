use clap::Parser;

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
    output: land_helpers::logging::TraceArgs,
    /// Address to listen on.
    #[clap(long, default_value("0.0.0.0:8640"))]
    address: String,
    /// Template directory
    #[clap(long)]
    tpldir: Option<String>,
    /// Database connection args.
    #[clap(flatten)]
    dbargs: land_dao::DBArgs,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    if args.version {
        land_helpers::version::println(env!("CARGO_PKG_NAME"), args.output.verbose);
        return Ok(());
    }

    // Initialize logging
    land_helpers::logging::init(args.output.verbose);

    // Connect to database
    land_dao::connect(&args.dbargs)
        .await
        .expect("Failed to connect to database");

    // init storage operator
    land_modules::storage::init_defaults().await?;
    land_modules::storage::load_global().await?;

    // start http server
    server::start(args.address.parse()?, "./assets", args.tpldir.clone())
        .await
        .expect("Failed to start server");

    Ok(())
}
