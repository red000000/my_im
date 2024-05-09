use hello_world::greeter_client::GreeterClient;
use hello_world::HelloRequest;
use tonic::transport::Endpoint;
use tower::balance::p2c;
use tower::discover::ServiceList;
use tower::limit::ConcurrencyLimit;
use tower::load::{CompleteOnResponse, PendingRequests};

pub mod hello_world {
    include!("../proto-compiled/services.rs");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let endpoints = vec![
        Endpoint::from_static("http://[::1]:50051"),
        Endpoint::from_static("http://[::1]:50053"),
    ];

    let services = endpoints
        .into_iter()
        .map(|endpoint| {
            let channel = endpoint.connect_lazy();
            let loaded_channel = PendingRequests::new(channel, CompleteOnResponse::default()); // 确保实现了 Load
            ConcurrencyLimit::new(loaded_channel, 1) // ConcurrencyLimit 现在支持 Load
        })
        .collect::<Vec<_>>();

    let service_list = ServiceList::new(services);
    let balance = p2c::Balance::new(service_list);
    let mut client = GreeterClient::new(balance);

    loop {
        let request = tonic::Request::new(HelloRequest {
            name: "Tonic".into(),
        });

        let response = client.say_hello(request).await?;
        println!("{:?}", response);
    }
}
