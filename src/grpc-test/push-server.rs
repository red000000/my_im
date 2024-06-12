use std::net::SocketAddr;
use std::sync::Arc;

use my_im::const_file::DEFAULT_REDIS_URL;
use my_im::redis_pool::RedisPool;
use reqwest::Client;

use tokio::sync::Mutex;
use tonic::{transport::Server, Request, Response, Status};

use my_im::data::{HeartBeatReply, HeartBeatRequest, Msg, PushResult};
use my_im::grpc_errors::GrpcErrors;
use my_im::push::greeter_server::{Greeter, GreeterServer};

use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};

/*type LoadBalancedPushClient = PushClient<
    p2c::Balance<ServiceList<Vec<PeakEwma<Channel>>>, http::Request<tonic::body::BoxBody>>,
>;*/
pub struct PushInner {
    heart_beat_client: Client,
    kafka_producer: FutureProducer,
    pool_clone: Arc<Mutex<RedisPool>>,
}
impl PushInner {
    pub async fn new() -> Result<Self, GrpcErrors> {
        let heart_beat_client = Client::builder().no_proxy().build()?;
        let pool_clone = Arc::new(Mutex::new(RedisPool::new(DEFAULT_REDIS_URL).await?));
        //FIX:get kafka urls by service servers
        let kafka_producer = ClientConfig::new().create()?;
        Ok(Self {
            heart_beat_client,
            kafka_producer,
            pool_clone,
        })
    }
    pub async fn send_msg(&self, msg: Msg) -> Result<(), GrpcErrors> {
        let topic = msg.topic.as_str();
        let key = msg.kafka_key.as_str();
        let payload = msg.msg.as_str();
        let record = FutureRecord::to(topic).key(key).payload(payload);
        self.kafka_producer
            .send(record, std::time::Duration::from_secs(3))
            .await
            .expect("send msg error!");

        Ok(())
    }
    pub fn heart_beat_client(&mut self, heart_beat_client: Client) -> &mut Self {
        self.heart_beat_client = heart_beat_client;
        self
    }
    pub fn pool_clone(&mut self, pool_clone: Arc<Mutex<RedisPool>>) -> &mut Self {
        self.pool_clone = pool_clone;
        self
    }
    pub fn kafka_producer(&mut self, kafka_producer: FutureProducer) -> &mut Self {
        self.kafka_producer = kafka_producer;
        self
    }
}
#[tonic::async_trait]
impl Greeter for PushInner {
    async fn heart_beat(
        &self,
        _: Request<HeartBeatRequest>,
    ) -> Result<Response<HeartBeatReply>, Status> {
        let result = Response::new(HeartBeatReply { result: true });
        Ok(result)
    }
    ///producer in there
    async fn push_msg(&self, request: Request<Msg>) -> Result<Response<PushResult>, Status> {
        let msg = request.into_inner();
        self.send_msg(msg).await?;
        Ok(Response::new(PushResult { result: true }))
    }
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bind_addr = "[::1]:50053".parse()?;
    let greeter_client = Client::builder().no_proxy().build()?;
    let pool_clone = Arc::new(Mutex::new(RedisPool::new(DEFAULT_REDIS_URL).await?));

    match server_run(bind_addr, greeter_client, pool_clone).await {
        Ok(()) => Ok(()),
        Err(e) => {
            println!("{},{}", e, "The services server is stop!");
            panic!();
        }
    }
}
async fn server_run(
    bind_addr: SocketAddr,
    greeter_client: Client,
    pool_clone: Arc<Mutex<RedisPool>>,
) -> Result<(), GrpcErrors> {
    let mut greeter = PushInner::new().await?;
    let kafka_producer = ClientConfig::new()
        .set(
            "bootstrap.servers",
            "localhost:9094,localhost:9095,localhost:9096",
        )
        .create()?;
    greeter
        .heart_beat_client(greeter_client)
        .pool_clone(pool_clone)
        .kafka_producer(kafka_producer);
    Server::builder()
        .add_service(GreeterServer::new(greeter))
        .serve(bind_addr)
        .await?;
    Ok(())
}
