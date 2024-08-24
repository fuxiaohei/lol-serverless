use land_sdk::http::{Body, Error, Request, Response};
use land_sdk::{http_main, ExecutionCtx};

#[http_main]
pub fn handle_request(req: Request, mut ctx: ExecutionCtx) -> Result<Response, Error> {

    let seq_id = ctx.sleep(1500);
    // this function is called in host with tokio::spawn
    ctx.sleep_callback(seq_id, || {
        println!("sleep 1.5s done!");
    });

    // this function is called in guest.
    // std::thread::sleep will block main thread(wasm runtime is single-thread currently)
    ctx.wait_until(|| {
        println!("sleep 1s...");
        std::thread::sleep(std::time::Duration::from_secs(1));
        println!("sleep 1s done!");
    });

    ctx.wait_until(|| {
        println!("sleep 2s...");
        std::thread::sleep(std::time::Duration::from_secs(2));
        println!("sleep 2s done!");
    });

    // build response
    Ok(http::Response::builder()
        .status(200)
        .body(Body::from("Hello Runtime.land!!"))
        .unwrap())
}
