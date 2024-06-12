use log::{info, warn};
use my_im::Kafka::{ConsumerTools, ProducerTools};
use rdkafka::{
    consumer::{CommitMode, Consumer},
    producer::FutureRecord,
    Message,
};

#[tokio::main]
async fn main() {
    use rdkafka::config::ClientConfig;
    use rdkafka::producer::{FutureProducer, FutureRecord};

    // 创建 Kafka 生产者配置
    let producer: FutureProducer = ClientConfig::new()
        .set(
            "bootstrap.servers",
            "192.168.33.144:9094,192.168.33.144:9095,192.168.33.144:9096",
        )
        //.set("security.protocol", "PLAINTEXT")
        .create()
        .expect("Producer creation error");

    // 主题和消息内容

    let topic = "text2";
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
#[tokio::test]
async fn test_producer() {
    let producer = ProducerTools::new().unwrap();
    let payload = "new test";
    let delivery_status = producer
        .send(
            FutureRecord::to("text").payload(payload).key("rust-key"),
            std::time::Duration::from_secs(3),
        )
        .await;
    match delivery_status {
        Ok(delivery) => println!("Sent: {:?}", delivery),
        Err((e, _)) => println!("Error: {:?}", e),
    }
}
#[tokio::test]
async fn test_consumer() {
    std::env::set_var("RUST_LOG", "info");
    pretty_env_logger::init();
    let consumer = ConsumerTools::new().unwrap();
    consumer.subscribe(&["text"]).unwrap();

    loop {
        match consumer.recv().await {
            Ok(message) => {
                let payload = match message.payload_view::<str>() {
                    Some(Ok(s)) => s,
                    Some(Err(e)) => {
                        info!("Error while deserializing message payload: {:?}", e);
                        ""
                    }
                    None => "",
                };
                log::info!("Received message: {}", payload);
                consumer
                    .commit_message(&message, CommitMode::Async)
                    .unwrap();
            }
            Err(e) => warn!("Kafka error: {}", e),
        }
    }
}
