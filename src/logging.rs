use clap::Parser;
use tracing_subscriber::{fmt::time::OffsetTime, EnvFilter};

/// init initializes the tracing subscriber.
/// It supports RUN_LOG env var to set the log level.
pub fn init(verbose: bool) {
    if std::env::var("RUST_LOG").ok().is_none() {
        if verbose {
            std::env::set_var("RUST_LOG", "rtland=debug")
        } else {
            std::env::set_var("RUST_LOG", "rtland=info")
        }
    }

    let timer = OffsetTime::new(
        time::UtcOffset::from_hms(8, 0, 0).unwrap(),
        time::format_description::parse(
            "[month]-[day] [hour]:[minute]:[second].[subsecond digits:2]",
        )
        .unwrap(),
    );

    tracing_subscriber::fmt()
        .with_timer(timer)
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .init();
}

#[derive(Parser, Debug)]
pub struct TraceArgs {
    /// Generate verbose output
    #[clap(short, long, global = true, conflicts_with = "quiet")]
    pub verbose: bool,
    /// Do not print progress messages.
    #[clap(short, long, global = true, conflicts_with = "verbose")]
    pub quiet: bool,
}