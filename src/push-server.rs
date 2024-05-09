use std::net::SocketAddr;
use std::sync::Arc;

use my_im::const_file::DEFAULT_REDIS_URL;
use my_im::redis_pool::RedisPool;
use reqwest::Client;

use tokio::sync::Mutex;
use tonic::{transport::Server, Request, Response, Status};

use my_im::grpc_errors::GrpcErrors;
use my_im::push::greeter_server::{Greeter, GreeterServer};
use my_im::push::{HelloReply, HelloRequest, Msg, PushResult};

#[derive(Debug)]
pub struct MyGreeter {
    client: Client,
    pool_clone: Arc<Mutex<RedisPool>>,
}
impl MyGreeter {
    pub async fn new() -> Result<Self, GrpcErrors> {
        let client = Client::builder().no_proxy().build()?;
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
}
#[tonic::async_trait]
impl Greeter for MyGreeter {
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
    ///consumer in there
    async fn push_msg(&self, request: Request<Msg>) -> Result<Response<PushResult>, Status> {
        let msg = request.into_inner().msg;
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
    let mut greeter = MyGreeter::new().await?;
    greeter.client(greeter_client).pool_clone(pool_clone);
    Server::builder()
        .add_service(GreeterServer::new(greeter))
        .serve(bind_addr)
        .await?;
    Ok(())
}
