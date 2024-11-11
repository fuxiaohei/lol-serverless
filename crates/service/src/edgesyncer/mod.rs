use std::sync::{Once, OnceLock};

pub mod heartbeat;
pub mod tasks;

static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
static CLIENT_ONCE: Once = Once::new();
