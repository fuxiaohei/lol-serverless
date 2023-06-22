use clap::Parser;
use land_core::trace;
use land_core::version;

mod deploy;
mod embed;
mod flags;
mod server;

/// cli command line
#[derive(Parser)]
#[clap(name = "land-cli", version = version::get())]
enum Cli {
    /// Init creates a new project
    Init(flags::Init),
    /// Build compiles the project
    Build(flags::Build),
    /// Serve runs the project
    Serve(flags::Serve),
    /// Deploy to cloud server
    Deploy(flags::Deploy),
}

#[tokio::main]
async fn main() {
    trace::init();

    let args = Cli::parse();
    match args {
        Cli::Init(cmd) => cmd.run().await,
        Cli::Build(cmd) => cmd.run().await,
        Cli::Serve(cmd) => cmd.run().await,
        Cli::Deploy(cmd) => cmd.run().await,
    }
}
