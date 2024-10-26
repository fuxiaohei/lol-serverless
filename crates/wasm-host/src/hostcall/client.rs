use super::host::land::http::types::RedirectPolicy;
use reqwest::{redirect, Client};
use std::sync::Once;
use std::sync::OnceLock;

pub static CLIENT_INIT_ONCE: Once = Once::new();
pub static REDIRECT_FOLLOW_POOL: OnceLock<Client> = OnceLock::new();
pub static REDIRECT_ERROR_POOL: OnceLock<Client> = OnceLock::new();
pub static REDIRECT_MANUAL_POOL: OnceLock<Client> = OnceLock::new();

/// init_clients is used to init http clients
pub fn init_clients() {
    CLIENT_INIT_ONCE.call_once(|| {
        REDIRECT_ERROR_POOL
            .set(
                reqwest::Client::builder()
                    .redirect(RedirectPolicy::Error.try_into().unwrap())
                    .build()
                    .unwrap(),
            )
            .unwrap();
        REDIRECT_FOLLOW_POOL
            .set(
                reqwest::Client::builder()
                    .redirect(RedirectPolicy::Follow.try_into().unwrap())
                    .build()
                    .unwrap(),
            )
            .unwrap();
        REDIRECT_MANUAL_POOL
            .set(
                reqwest::Client::builder()
                    .redirect(RedirectPolicy::Manual.try_into().unwrap())
                    .build()
                    .unwrap(),
            )
            .unwrap();
    });
}

/// get_client is used to get http client by redirect policy
pub fn get_client(r: RedirectPolicy) -> Client {
    match r {
        RedirectPolicy::Follow => REDIRECT_FOLLOW_POOL.get().unwrap().clone(),
        RedirectPolicy::Error => REDIRECT_ERROR_POOL.get().unwrap().clone(),
        RedirectPolicy::Manual => REDIRECT_MANUAL_POOL.get().unwrap().clone(),
    }
}

impl TryFrom<RedirectPolicy> for redirect::Policy {
    type Error = anyhow::Error;
    fn try_from(value: RedirectPolicy) -> Result<Self, Self::Error> {
        match value {
            RedirectPolicy::Follow => Ok(redirect::Policy::default()),
            RedirectPolicy::Error => Ok(redirect::Policy::custom(|attempt| {
                attempt.error(anyhow::anyhow!("redirect policy is error"))
            })),
            RedirectPolicy::Manual => Ok(redirect::Policy::none()),
        }
    }
}
