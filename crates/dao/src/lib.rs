use anyhow::Result;
use clap::Args;
use once_cell::sync::OnceCell;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;
use std::time::Duration;
use tracing::{debug, info, instrument, warn};

mod migration;

pub mod models;
pub mod settings;
pub mod tokens;
pub mod users;

#[derive(Args)]
pub struct DBArgs {
    /// Database driver
    #[clap(long("db-driver"), env("DATABASE_DRIVER"), default_value("sqlite"))]
    pub driver: String,
    /// Database filepath, only for sqlite
    #[clap(
        long("db-filepath"),
        env("DATABASE_FILEPATH"),
        default_value("rtland.db")
    )]
    pub filepath: String,
    /// Database host
    #[clap(long("db-host"), env("DATABASE_HOST"), default_value("127.0.0.1"))]
    pub host: String,
    /// Database port
    #[clap(long("db-port"), env("DATABASE_PORT"), default_value("5432"))]
    pub port: u16,
    /// Database user
    #[clap(long("db-user"), env("DATABASE_USER"), default_value("fuxiaohei"))]
    pub user: String,
    /// Database password
    #[clap(
        long("db-password"),
        env("DATABASE_PASSWORD"),
        default_value("fuxiaohei")
    )]
    pub password: String,
    /// Database name
    #[clap(
        long("db-database"),
        env("DATABASE_DATABASE"),
        default_value("runtime-land-db")
    )]
    pub database: String,
    /// Database connection pool size
    #[clap(long("db-pool-size"), env("DB_POOL_SIZE"), default_value("10"))]
    pub pool_size: u32,
}

impl DBArgs {
    fn url_internal(&self, pwd: &str) -> String {
        if self.driver == "sqlite" {
            // rwc means read-write-create
            return format!("{}://{}?mode=rwc", self.driver, self.filepath);
        }
        format!(
            "{}://{}:{}@{}:{}/{}",
            self.driver, self.user, pwd, self.host, self.port, self.database
        )
    }
    fn url(&self) -> String {
        self.url_internal(&self.password)
    }
    pub fn url_safe(&self) -> String {
        self.url_internal("******")
    }
}

impl std::fmt::Debug for DBArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.driver == "sqlite" {
            return f
                .debug_struct("DBArgs")
                .field("filepath", &self.filepath)
                .finish();
        }
        f.debug_struct("DBArgs")
            .field("host", &self.host)
            .field("port", &self.port)
            .field("user", &self.user)
            .field("password", &"******")
            .field("database", &self.database)
            .field("pool_size", &self.pool_size)
            .finish()
    }
}

/// DB connection pool
pub static DB: OnceCell<DatabaseConnection> = OnceCell::new();

/// connect connects to the database.
#[instrument("[DB]", skip_all)]
pub async fn connect(args: &DBArgs) -> Result<()> {
    debug!("Connecting: {}", args.url_safe());

    let mut opt = ConnectOptions::new(args.url());
    opt.max_connections(args.pool_size)
        .min_connections(3)
        .connect_timeout(Duration::from_secs(10))
        .acquire_timeout(Duration::from_secs(10))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .sqlx_logging_level(tracing::log::LevelFilter::Info);

    let db = Database::connect(opt).await?;

    // run migrations
    migration::Migrator::up(&db, None).await?;

    DB.set(db).unwrap();
    info!("Init success: {}", args.url_safe());

    // initialize settings
    init_defaults().await?;

    Ok(())
}

/// init_defaults initializes default settings.
async fn init_defaults() -> Result<()> {
    if settings::is_installed().await? {
        info!("dashboard is already installed, initializing defaults settings");
        return Ok(());
    }
    warn!("dashboard is not installed");
    Ok(())
}
