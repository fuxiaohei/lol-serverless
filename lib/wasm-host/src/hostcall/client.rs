use super::host_service::land::http::types::RedirectPolicy;
use once_cell::sync::OnceCell;
use reqwest::Client;
use std::sync::Once;

pub static CLIENT_INIT_ONCE: Once = Once::new();
pub static REDIRECT_FOLLOW_POOL: OnceCell<Client> = OnceCell::new();
pub static REDIRECT_ERROR_POOL: OnceCell<Client> = OnceCell::new();
pub static REDIRECT_MANUAL_POOL: OnceCell<Client> = OnceCell::new();

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
