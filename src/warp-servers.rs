#[tokio::main]
async fn main() {}
#[cfg(test)]
mod test {

    use futures_util::{SinkExt, StreamExt, TryStreamExt};
    use warp::{filters::ws::Message, Filter};
    #[tokio::test]
    async fn server_test1() {
        std::env::set_var("RUST_LOG", "info");
        pretty_env_logger::try_init_timed().unwrap();
        let log = warp::log("Server1");

        let heart_router = warp::path("heartbeat")
            .and(warp::path::end())
            .then(|| async { warp::reply() });
        let router = warp::path("test1")
            .and(warp::path::end())
            .and(warp::ws())
            .then(|ws: warp::filters::ws::Ws| async {
                ws.on_upgrade(|web_socket| async {
                    let (mut ws_sink, mut ws_stream) = web_socket.split();
                    while let Ok(Some(msg)) = ws_stream.try_next().await {
                        if let Ok(msg) = msg.to_str() {
                            let return_msg = format!("This is test_1 server get {}", msg);
                            ws_sink.send(Message::text(return_msg)).await.unwrap();
                        }
                    }
                })
            })
            .or(heart_router)
            .with(log);
        warp::serve(router).run(([127, 0, 0, 1], 3030)).await
    }
    #[tokio::test]
    async fn server_test2() {
        std::env::set_var("RUST_LOG", "info");
        pretty_env_logger::try_init_timed().unwrap();
        let log = warp::log("Server2");

        let heart_router = warp::path("heartbeat")
            .and(warp::path::end())
            .then(|| async { warp::reply() });
        let router = warp::path("test2")
            .and(warp::path::end())
            .and(warp::ws())
            .then(|ws: warp::filters::ws::Ws| async {
                ws.on_upgrade(|web_socket| async {
                    let (mut ws_sink, mut ws_stream) = web_socket.split();
                    while let Ok(Some(msg)) = ws_stream.try_next().await {
                        if let Ok(msg) = msg.to_str() {
                            let return_msg = format!("This is test_2 server get {}", msg);
                            ws_sink.send(Message::text(return_msg)).await.unwrap();
                        }
                    }
                })
            })
            .or(heart_router)
            .with(log);
        warp::serve(router).run(([127, 0, 0, 1], 3031)).await
    }
}
