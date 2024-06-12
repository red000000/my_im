use std::net::SocketAddr;
use std::sync::Arc;

use log::{info, warn};
use my_im::ConstFile::kafka::SECURITY_PROTOCOL;
use my_im::ConstFile::DEFAULT_REDIS_URL;
use my_im::Kafka::ProducerTools;
use my_im::RedisPool::RedisPool;
use reqwest::Client;

use tokio::sync::Mutex;
use tonic::{transport::Server, Request, Response, Status};

use my_im::data::{HeartBeatReply, HeartBeatRequest, Msg, PushResult};
use my_im::grpc_errors::GrpcErrors;
use my_im::push::greeter_server::{Greeter, GreeterServer};

use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use warp::Filter;

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
        //:fix the warnning about kafka don't have bootstrap server
        let kafka_producer = ProducerTools::new()?;
        Ok(Self {
            heart_beat_client,
            kafka_producer,
            pool_clone,
        })
    }
    pub async fn send_msg(&self, msg: Msg) -> Result<(), GrpcErrors> {
        info!("{},{}", msg.kafka_topic, msg.kafka_key);
        let topic = msg.kafka_topic.as_str();
        let key = msg.kafka_key.as_str();
        let payload = format!("{},{},{}", msg.time, msg.client_uuid, msg.msg);
        let record = FutureRecord::to(topic).payload(&payload).key(key);
        self.kafka_producer
            .send(record, std::time::Duration::from_secs(3))
            .await?;

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
    ///producer send msg
    async fn push_msg(&self, request: Request<Msg>) -> Result<Response<PushResult>, Status> {
        info!("push msg!");
        let msg = request.into_inner();
        self.send_msg(msg).await?;
        info!("push msg success~");
        Ok(Response::new(PushResult { result: true }))
    }
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("RUST_LOG", "info");
    pretty_env_logger::try_init_timed()?;
    let bind_addr = "[::1]:50053".parse()?;
    let greeter_client = Client::builder().no_proxy().build()?;
    let pool_clone = Arc::new(Mutex::new(RedisPool::new(DEFAULT_REDIS_URL).await?));
    // heart beat and grpc server in one thread to avoid the problem of grpc server stop but http server alive!
    let future1 = http_heart_beat();
    let future2 = grpc_server_run(bind_addr, greeter_client, pool_clone);
    match tokio::try_join!(future1, future2) {
        Ok(((), ())) => Ok(()),
        Err(e) => {
            println!("{},{}", e, "The services server is stop!");
            panic!();
        }
    }
}
async fn http_heart_beat() -> Result<(), Box<dyn std::error::Error>> {
    let log = warp::log("heart_beat_server");
    let heart_beat_router = warp::path("heartbeat")
        .and(warp::path::end())
        .then(|| async move { warp::reply() })
        .with(log);
    warp::serve(heart_beat_router)
        .run(([127, 0, 0, 1], 50053))
        .await;
    Ok(())
}
async fn grpc_server_run(
    bind_addr: SocketAddr,
    greeter_client: Client,
    pool_clone: Arc<Mutex<RedisPool>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut push_inner = PushInner::new().await?;
    push_inner
        .heart_beat_client(greeter_client)
        .pool_clone(pool_clone);
    Server::builder()
        .add_service(GreeterServer::new(push_inner))
        .serve(bind_addr)
        .await?;
    Ok(())
}
