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
        .set("bootstrap.servers", "localhost:9094,localhost:9095,localhost:9096")
        .set("auto.offset.reset", "latest")
        .create()
        .expect("Consumer creation failed");

    // 订阅主题
    consumer
        .subscribe(&["test-topic"])
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
