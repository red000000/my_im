use futures_util::stream;
use futures_util::Stream;

use my_im::const_file::*;
use my_im::grpc_errors::GrpcErrors;
use my_im::redis_pool::RedisPool;
use my_im::services::greeter_server::{Greeter, GreeterServer};
use my_im::services::{GetServicesMsg, HelloReply, HelloRequest, ServiceMsg};

use reqwest::Client;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{transport::Server, Request, Response, Status};

#[derive(Debug)]
pub struct MyGreeter {
    client: Client,
    pool_clone: Arc<Mutex<RedisPool>>,
}
impl MyGreeter {
    pub async fn new() -> Result<Self, GrpcErrors> {
        let client = Client::new();
        let pool_clone = Arc::new(Mutex::new(RedisPool::new(DEFAULT_REDIS_URL).await?));
        Ok(Self { client, pool_clone })
    }
    pub fn client(&mut self, client: Client) -> &mut Self {
        self.client = client;
        self
    }
    pub fn pool_clone(&mut self, pool_clone: Arc<Mutex<RedisPool>>) -> &mut Self {
        self.pool_clone = pool_clone;
        self
    }
    //service_server_urls下有多个键值对，键为push_server_urls,值为哈希表
    pub async fn get_service_msgs(&self) -> Result<Vec<ServiceMsg>, GrpcErrors> {
        self.pool_clone
            .try_lock()?
            .get_service_msgs_from_redis()
            .await
    }
}
#[tonic::async_trait]
impl Greeter for MyGreeter {
    type GetServicesStream = Pin<Box<dyn Stream<Item = Result<ServiceMsg, Status>> + Send + Sync>>;
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        println!("Got a request: {:?}", request);

        let reply = HelloReply {
            message: format!("This is server1, Hello {}!", request.into_inner().name),
        };

        Ok(Response::new(reply))
    }

    async fn get_services(
        &self,
        _request: Request<GetServicesMsg>,
    ) -> Result<Response<Self::GetServicesStream>, Status> {
        let services = self.get_service_msgs().await?;
        // 创建一个多元素流，每个元素都用Ok封装
        let output_stream = stream::iter(services.into_iter().map(Ok));
        println!("{:?}", output_stream);
        // 将流装箱并返回
        Ok(Response::new(
            Box::pin(output_stream) as Self::GetServicesStream
        ))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bind_addr = "[::1]:50051".parse()?;
    let heart_beat_client = Client::builder().no_proxy().build()?;
    let greeter_client = Client::builder().no_proxy().build()?;
    let count = Arc::new(Mutex::new(0));
    let count_clone = Arc::clone(&count);
    let redis_pool = Arc::new(Mutex::new(RedisPool::new(DEFAULT_REDIS_URL).await?));
    let pool_clone_in_heart = Arc::clone(&redis_pool); // 克隆连接池
    let pool_clone_in_server = Arc::clone(&redis_pool); // 克隆连接池
    
    //心跳监听
    tokio::spawn(async move {
        if let Err(e) = start_heartbeat(heart_beat_client, count_clone, pool_clone_in_heart).await {
            println!("{},{}", e, "The heartbeat is stop!");
            panic!();
        }
    });

    // 在新的异步任务中运行服务
    tokio::spawn(async move {
        if let Err(e) = server_run(bind_addr, greeter_client, pool_clone_in_server).await {
            println!("{},{}", e, "The services server is stop!");
            panic!();
        }
    });
    loop {
        //打印hearbeat计数器
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        println!("hearbeat is running,count :{}", *count.try_lock()?);
    }
}

async fn start_heartbeat(
    client: Client,
    count: Arc<Mutex<u32>>,
    pool_clone: Arc<Mutex<RedisPool>>,
) -> Result<(), GrpcErrors> {
    use tokio::time::{self, Duration};
    let mut interval = time::interval(Duration::from_secs(10));
    loop {
        interval.tick().await;
        let services_msgs = pool_clone.try_lock()?.get_service_msgs_from_redis().await?;
        for service_msg in services_msgs {
            let heartbeat_url = format!("{}{}", service_msg.url, SERVER_HEARBEAT_URL);
            match client.get(heartbeat_url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        let service_and_id =
                            format!("{}:{}", service_msg.service_group, service_msg.id);
                        pool_clone
                            .try_lock()?
                            .set_live_type_by_service_group_and_id(true, service_and_id)
                            .await?;
                    }
                }
                Err(e) => {
                    let service_and_id =
                        format!("{}:{}", service_msg.service_group, service_msg.id);
                    pool_clone
                        .try_lock()?
                        .set_live_type_by_service_group_and_id(false, service_and_id)
                        .await?;
                    println!("{},{}", e, "heartbeat is failed!");
                }
            }
        }
        *count.try_lock()? += 1;
    }
}
async fn server_run(
    bind_addr: SocketAddr,
    greeter_client: Client,
    pool_clone: Arc<Mutex<RedisPool>>,
) -> Result<(), GrpcErrors> {
    let mut greeter = MyGreeter::new().await?;
    greeter.client(greeter_client).pool_clone(pool_clone);
    Server::builder()
        .add_service(GreeterServer::new(greeter))
        .serve(bind_addr)
        .await?;
    Ok(())
}

