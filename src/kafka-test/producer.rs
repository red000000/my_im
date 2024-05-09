#[tokio::main]
async fn main() {
    use rdkafka::config::ClientConfig;
    use rdkafka::producer::{FutureProducer, FutureRecord};

    // 创建 Kafka 生产者配置
    let producer: FutureProducer = ClientConfig::new()
        .set(
            "bootstrap.servers",
            "localhost:9094,localhost:9095,localhost:9096",
        )
        .set("acks", "1")
        .set("debug", "broker,topic,msg")
        .create()
        .expect("Producer creation error");

    // 主题和消息内容

    let topic = "test-topic";
    let payload = format!("CNMD SB ");

    // 发送消息
    let delivery_status = producer
        .send(
            FutureRecord::to(topic).payload(&payload).key("rust-key"),
            std::time::Duration::from_secs(3),
        )
        .await;

    // 检查消息是否成功发送
    match delivery_status {
        Ok(delivery) => println!("Sent: {:?}", delivery),
        Err((e, _)) => println!("Error: {:?}", e),
    }
}
