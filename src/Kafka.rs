use std::time::Duration;

use super::ConstFile::kafka::*;
use log::{info, warn};
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::message::Message;
use rdkafka::producer::{FutureProducer, FutureRecord};
pub struct KafkaTools;
impl KafkaTools {
    pub fn get_bootstrap_servers(ip: &str, ports: Vec<&str>) -> String {
        let mut urls = Vec::new();
        for port in ports {
            let url = format!("{}:{}", ip, port);
            urls.push(url);
        }
        urls.join(",")
    }
}
pub struct ProducerTools;
impl ProducerTools {
    pub fn new() -> Result<FutureProducer, Box<dyn std::error::Error>> {
        let bootstrap_servers =
            KafkaTools::get_bootstrap_servers(KAFKA_CLUSER_IP, KAFKA_CLUSER_PORTS.to_vec());
        // 创建 Kafka 生产者配置
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", bootstrap_servers)
            .set("acks", ACKS)
            .create()?;
        Ok(producer)
    }
}

pub struct ConsumerTools;
impl ConsumerTools {
    pub fn new() -> Result<StreamConsumer, Box<dyn std::error::Error>> {
        let bootstrap_servers =
            KafkaTools::get_bootstrap_servers(KAFKA_CLUSER_IP, KAFKA_CLUSER_PORTS.to_vec());
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", bootstrap_servers)
            .set("group.id", GROUP_ID)
            .set("security.protocol", SECURITY_PROTOCOL)
            .set("auto.offset.reset", AUTO_OFFSET_RESET)
            .create()?;
        Ok(consumer)
    }
}
#[tokio::test]
async fn test() {
    let producer = ProducerTools::new().unwrap();
    let record1 = FutureRecord::to("test-topic").payload("hello").key("key");
    let record2 = FutureRecord::to("test-topic")
        .payload("hello!my name is bob!")
        .key("key");
    let result = producer
        .send(record1, Duration::from_secs(5))
        .await
        .unwrap();
    println!("{:?}", result);
    let result = producer
        .send(record2, Duration::from_secs(5))
        .await
        .unwrap();
    println!("{:?}", result);
}
#[tokio::test]
async fn test_consumer() {
    std::env::set_var("RUST_LOG", "info");
    pretty_env_logger::init();
    let consumer = ConsumerTools::new().unwrap();
    consumer.subscribe(&["text2"]).unwrap();
    loop {
        match consumer.recv().await {
            Ok(message) => {
                let msg = message.payload_view::<str>().unwrap().unwrap();
                info!("{}", msg);
                consumer
                    .commit_message(&message, CommitMode::Async)
                    .unwrap();
            }
            Err(e) => {
                warn!("{:?}", e);
            }
        }
    }
}
