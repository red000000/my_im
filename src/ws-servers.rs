#[tokio::main]
async fn main() {}

#[cfg(test)]
mod test {
    use my_im::{
        client::ClientTools,
        data::Msg,
        push::greeter_client::GreeterClient as PushClient,
        ConstFile::{
            client_message_type::{HEARTBEAT_MESSAGE, RESULT_MESSAGE, TEXT_MESSAGE},
            client_mseeage_subscript::*,
            client_result_type::{FAILED, SUCCESS},
            PUSH_SERVER_URLS,
        },
    };
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use tonic::Request;

    use futures_util::{SinkExt, StreamExt, TryStreamExt};
    use log::{info, warn};
    use warp::ws::Message;
    use warp::Filter;

    #[tokio::test]
    pub async fn server_test1() {
        std::env::set_var("RUST_LOG", "info");
        pretty_env_logger::try_init_timed().unwrap();

        let bind_addr: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 3030);
        let log = warp::log("Server1");
        let heart_beat_router = warp::path("heartbeat")
            .and(warp::path::end())
            .then(|| async { warp::reply() });
        let router = warp::path("ws")
            .and(warp::path::end())
            .and(warp::ws())
            .then(|ws: warp::filters::ws::Ws| async move {
                ws.on_upgrade(|web_socket| async move {
                    let mut push_client = PushClient::new(
                        ClientTools::get_balanced_service_list(PUSH_SERVER_URLS.to_vec()).await,
                    );
                    let (mut ws_sink, mut ws_stream) = web_socket.split();
                    while let Ok(Some(msg)) = ws_stream.try_next().await {
                        let mut msg_vec = msg
                            .to_str()
                            .expect("msg is not string")
                            .split(',')
                            .collect::<Vec<&str>>();
                        match msg_vec[0] {
                            HEARTBEAT_MESSAGE => {
                                info!("heartbeat");
                            }
                            TEXT_MESSAGE => {
                                info!("{:?}", msg);
                                msg_vec.remove(0);
                                let dt = chrono::Local::now();
                                info!("{:?}", msg_vec);
                                if let Ok(_) = push_client
                                    .push_msg(Request::new(Msg {
                                        time: dt.to_string(),
                                        client_uuid: msg_vec[CLIENT_UUID_SUBSCRIPT].to_string(),
                                        msg: msg_vec[MSG_DATA_SUBSCRIPT].to_string(),
                                        kafka_topic: "text2".to_string(),
                                        kafka_key: msg_vec[KAFKA_KEY_SUBSCRIPT].to_string(),
                                    }))
                                    .await
                                {
                                    info!("success!!");
                                    ws_sink
                                        .send(Message::text(format!(
                                            "{},{}",
                                            RESULT_MESSAGE, SUCCESS
                                        )))
                                        .await
                                        .expect("send result success error!");
                                } else {
                                    info!("failed!!");
                                    ws_sink
                                        .send(Message::text(format!(
                                            "{},{}",
                                            RESULT_MESSAGE, FAILED
                                        )))
                                        .await
                                        .expect("send result failed error!");
                                    warn!(
                                "send msg to push server error! the msg from client uuid:{} ",
                                msg_vec[1]
                            );
                                }
                            }
                            _ => {
                                ws_sink
                                    .send(Message::text(format!("{},{}", RESULT_MESSAGE, FAILED)))
                                    .await
                                    .expect("send result ok error!");
                                info!("unknown");
                            }
                        }
                    }
                })
            })
            .or(heart_beat_router)
            .with(log);
        warp::serve(router).run(bind_addr).await
    }
    #[tokio::test]
    pub async fn server_test2() {
        std::env::set_var("RUST_LOG", "info");
        pretty_env_logger::try_init_timed().unwrap();

        let bind_addr: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 3031);
        let log = warp::log("Server1");
        let heart_beat_router = warp::path("heartbeat")
            .and(warp::path::end())
            .then(|| async { warp::reply() });
        let router = warp::path("ws")
            .and(warp::path::end())
            .and(warp::ws())
            .then(|ws: warp::filters::ws::Ws| async move {
                ws.on_upgrade(|web_socket| async move {
                    let mut push_client = PushClient::new(
                        ClientTools::get_balanced_service_list(PUSH_SERVER_URLS.to_vec()).await,
                    );
                    let (mut ws_sink, mut ws_stream) = web_socket.split();
                    while let Ok(Some(msg)) = ws_stream.try_next().await {
                        let mut msg_vec = msg
                            .to_str()
                            .expect("msg is not string")
                            .split(',')
                            .collect::<Vec<&str>>();
                        match msg_vec[0] {
                            HEARTBEAT_MESSAGE => {
                                info!("heartbeat");
                            }
                            TEXT_MESSAGE => {
                                info!("{:?}", msg);
                                msg_vec.remove(0);
                                let dt = chrono::Local::now();
                                info!("{:?}", msg_vec);
                                if let Ok(_) = push_client
                                    .push_msg(Request::new(Msg {
                                        time: dt.to_string(),
                                        client_uuid: msg_vec[CLIENT_UUID_SUBSCRIPT].to_string(),
                                        msg: msg_vec[MSG_DATA_SUBSCRIPT].to_string(),
                                        kafka_topic: "text2".to_string(),
                                        kafka_key: msg_vec[KAFKA_KEY_SUBSCRIPT].to_string(),
                                    }))
                                    .await
                                {
                                    info!("success!!");
                                    ws_sink
                                        .send(Message::text(format!(
                                            "{},{}",
                                            RESULT_MESSAGE, SUCCESS
                                        )))
                                        .await
                                        .expect("send result success error!");
                                } else {
                                    info!("failed!!");
                                    ws_sink
                                        .send(Message::text(format!(
                                            "{},{}",
                                            RESULT_MESSAGE, FAILED
                                        )))
                                        .await
                                        .expect("send result failed error!");
                                    warn!(
                                "send msg to push server error! the msg from client uuid:{} ",
                                msg_vec[1]
                            );
                                }
                            }
                            _ => {
                                ws_sink
                                    .send(Message::text(format!("{},{}", RESULT_MESSAGE, FAILED)))
                                    .await
                                    .expect("send result ok error!");
                                info!("unknown");
                            }
                        }
                    }
                })
            })
            .or(heart_beat_router)
            .with(log);
        warp::serve(router).run(bind_addr).await
    }
    #[tokio::test]
    async fn test_push_server() {
        let blanced = ClientTools::get_balanced_service_list(PUSH_SERVER_URLS.to_vec()).await;
        let mut client = PushClient::new(blanced);
        let res = client
            .push_msg(Request::new(Msg {
                time: "test time".to_string(),
                client_uuid: "test-uuid".to_string(),
                msg: "test msg".to_string(),
                kafka_topic: "text".to_string(),
                kafka_key: "test-key".to_string(),
            }))
            .await
            .unwrap();
        println!("{:?}", res);
    }
}
