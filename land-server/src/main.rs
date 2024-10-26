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
    /// Verbose mode.
    #[clap(flatten)]
    output: land_helpers::logging::TraceArgs,
    /// Address to listen on.
    #[clap(long, default_value("0.0.0.0:9840"))]
    address: String,
    /// Template directory
    #[clap(long)]
    tpldir: Option<String>,
    // Database connection args.
    //#[clap(flatten)]
    //dbargs: land_dao::DBArgs,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    if args.version {
        land_helpers::version::println(env!("CARGO_PKG_NAME"), args.output.verbose);
        return Ok(());
    }

    Ok(())
}
