// src/client.rs
use futures_util::stream::StreamExt;
use my_im::const_file::SERVICES_URLS;
use my_im::grpc_errors::GrpcErrors;
use my_im::services::greeter_client::GreeterClient;
use my_im::services::HelloRequest;
use std::time::Duration;
use tonic::transport::{Channel, Endpoint};
use tower::balance::p2c;
use tower::discover::ServiceList;
use tower::load::peak_ewma::PeakEwma;
use tower::load::CompleteOnResponse;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service_list = get_service_list(SERVICES_URLS.to_vec()).await;
    let balance = p2c::Balance::new(service_list);
    let mut client = GreeterClient::new(balance);
    tokio::spawn(async move {
        let service_list = get_service_list(SERVICES_URLS.to_vec()).await;
        if let Err(e) = update_push_server(service_list).await {
            println!("{}", e);
        }
    });
    //接下来从服务端获取实际服务器地址，例如kafka节点，小功能接口等进行处理

    
    Ok(())
}
async fn update_push_server(
    service_list: ServiceList<Vec<PeakEwma<Channel>>>,
) -> Result<(), GrpcErrors> {
    let balance = p2c::Balance::new(service_list);
    let mut client = GreeterClient::new(balance);
    loop {
        let request = tonic::Request::new(HelloRequest {
            name: "World".into(),
        });
        let response = client.say_hello(request).await?;
        println!("Response: {:?}", response.into_inner().message);
    }
}
async fn get_service_list(urls: Vec<&'static str>) -> ServiceList<Vec<PeakEwma<Channel>>> {
    let channels = futures_util::stream::iter(
        urls.into_iter()
            .map(|endpoint| Endpoint::from_static(endpoint).connect_lazy()),
    )
    .map(|channel| {
        PeakEwma::new(
            channel,
            Duration::from_secs(10),
            0.1,
            CompleteOnResponse::default(),
        )
    })
    .collect::<Vec<_>>()
    .await;
    let service_list = ServiceList::new(channels);
    service_list
}
