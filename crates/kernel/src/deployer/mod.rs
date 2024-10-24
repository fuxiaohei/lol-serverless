mod review;
mod waiting;

/// init_background initializes the background tasks.
pub async fn init_background() {
    waiting::init_background().await;
    review::init_background().await;
}