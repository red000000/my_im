#[tokio::main]
async fn main() {
    use rdkafka::config::ClientConfig;
    use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
    use rdkafka::message::Message;
    std::env::set_var("RUST_LOG", "info");
    pretty_env_logger::try_init_timed().unwrap();
    // 创建 Kafka 消费者配置
    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", "test-group")
        .set(
            "bootstrap.servers",
            "192.168.33.144:9094,192.168.33.144:9095,192.168.33.144:9096",
        )
        //.set("security.protocol", "PLAINTEXT")
        .set("auto.offset.reset", "earliest")
        //.set("debug", "broker,topic,msg")
        .create()
        .expect("Consumer creation failed");

    // 订阅主题
    consumer
        .subscribe(&["text"])
        .expect("Can't subscribe to specified topics");
    println!("ok");
    // 消费消息
    loop {
        match consumer.recv().await {
            Ok(message) => {
                let payload = match message.payload_view::<str>() {
                    Some(Ok(s)) => s,
                    Some(Err(e)) => {
                        println!("Error while deserializing message payload: {:?}", e);
                        ""
                    }
                    None => "",
                };
                log::info!("Received message: {}", payload);
                consumer
                    .commit_message(&message, CommitMode::Async)
                    .unwrap();
            }
            Err(e) => println!("Kafka error: {}", e),
        }
    }
}
