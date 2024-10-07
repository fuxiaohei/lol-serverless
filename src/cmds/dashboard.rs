use crate::dashboard;
use anyhow::Result;
use clap::Args;
use land_dao::DBArgs;
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
    /// Database connection args.
    #[clap(flatten)]
    dbargs: DBArgs,
}

impl Dashboard {
    pub async fn run(&self) -> Result<()> {
        debug!("start dashboard flag: {:?}", self);

        // connect to database
        land_dao::connect(&self.dbargs)
            .await
            .expect("Failed to connect to database");

        // start http server
        dashboard::start_server(self.address.parse()?, "./assets", self.tpldir.clone())
            .await
            .expect("Failed to start server");
        Ok(())
    }
}
