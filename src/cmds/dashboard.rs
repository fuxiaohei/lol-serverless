use anyhow::Result;
use clap::Args;
use tracing::debug;

/// Command Dashboard start dashboard server.
#[derive(Args, Debug)]
pub struct Dashboard {
    /// Address to listen on.
    #[clap(long, default_value("0.0.0.0:4040"))]
    address: String,
    /// Template directory
    #[clap(long)]
    tpldir: Option<String>,
}

impl Dashboard {
    pub async fn run(&self) -> Result<()> {
        debug!("start dashboard flag: {:?}", self);

        Ok(())
    }
}
