use std::io::{BufRead, BufReader};
use std::sync::Arc;

use tokio::sync::Mutex;
use warp::{Buf, Filter};
use env_logger;

use logsnarf_rs::handler::Handler;

#[tokio::main]
async fn main() {
    env_logger::init();

    let handler = Arc::new(Mutex::new(Handler::new()));

    let ingress = warp::post()
        .and(warp::path("ingress"))
        .and(warp::body::content_length_limit(16 * 1024 * 1024))
        .and(warp::body::bytes())
        .map(move |bytes| {
            let h = handler.clone();
            async move {
                let mut handler = h.lock().await;
                handle_body(&mut handler, bytes);
            };
            warp::reply()
        });

    warp::serve(ingress)
        .run(([127, 0, 0, 1], 3030))
        .await;
}


fn handle_body(handler: &mut Handler, mut body: impl Buf) {
    // while body.has_remaining() {
    //     let reader = BufReader::new(body.bytes());
    //     for line in reader.lines() {
    //         if let Ok(l) = line {
    //             handler.handle(l);
    //         }
    //     }
    //     let cnt = body.bytes().len();
    //     body.advance(cnt);
    // }
}

