use anyhow::Result;
use clap::{CommandFactory, Parser};
use color_print::cprintln;
use std::process;

mod cmds;
mod dashboard;
mod logging;
mod version;

#[derive(Parser, Debug)]
enum SubCommands {
    Dashboard(cmds::dashboard::Dashboard),
    Worker(cmds::worker::Worker),
}

#[derive(Parser, Debug)]
#[clap(author, version)]
#[clap(disable_version_flag = true)] // handled manually
#[clap(
    name = env!("CARGO_PKG_NAME"),
    about = concat!(env!("CARGO_PKG_NAME")," ",env!("CARGO_PKG_VERSION")),
)]
struct CliArgs {
    /// Print version info and exit.
    #[clap(short = 'V', long)]
    version: bool,
    #[clap(flatten)]
    output: logging::TraceArgs,
    #[clap(subcommand)]
    cmd: Option<SubCommands>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = CliArgs::parse();
    if args.version {
        version::print(env!("CARGO_PKG_NAME"), args.output.verbose);
        return Ok(());
    }

    // Initialize tracing
    logging::init(args.output.verbose);

    // Run subcommand
    let res = match args.cmd {
        Some(SubCommands::Dashboard(n)) => n.run().await,
        Some(SubCommands::Worker(w)) => w.run().await,
        None => {
            CliArgs::command().print_long_help().unwrap();
            process::exit(2);
        }
    };
    if let Err(err) = res {
        cprintln!("<red>Something wrong:\n  {}</red>", err);
        process::exit(2);
    }
    Ok(())
}
