use land_sdk::http::{Body, Error, Request, Response};
use land_sdk::{http_main, ExecutionCtx};

#[http_main]
pub fn handle_request(req: Request, mut ctx: ExecutionCtx) -> Result<Response, Error> {
    // read uri and method from request
    let url = req.uri().clone();
    let method = req.method().to_string().to_uppercase();

    ctx.sleep(1500);

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
        .header("X-Request-Url", url.to_string())
        .header("X-Request-Method", method)
        .body(Body::from("Hello Runtime.land!!"))
        .unwrap())
}
