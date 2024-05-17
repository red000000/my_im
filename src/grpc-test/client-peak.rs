// src/client_sink.rs
use futures_util::stream::StreamExt;
use futures_util::{SinkExt, TryStreamExt};
use log::{info, warn};
use my_im::const_file::SERVICE_SERVER_URLS;
use my_im::data::{GetServicesMsg, Msg};
use my_im::grpc_errors::GrpcErrors;
use my_im::services::greeter_client::GreeterClient as ServicesClient;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;
use tonic::transport::{Channel, Endpoint};
use tower::balance::p2c;
use tower::discover::ServiceList;
use tower::load::peak_ewma::PeakEwma;
use tower::load::CompleteOnResponse;

type ClientSink = futures_util::stream::SplitSink<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    tokio_tungstenite::tungstenite::Message,
>;
type ClientStream = futures_util::stream::SplitStream<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("RUST_LOG", "info,tower_http=debug");
    let push_urls = Arc::new(Mutex::new(Vec::new()));
    let push_urls_in_update = push_urls.clone();
    let push_urls_in_push = push_urls.clone();

    let msg = Msg {
        msg: "hello".to_string(),
        topic: "test".to_string(),
        kafka_key: "client_sink-peak".to_string(),
    };

    let client_sink: ClientSink;
    let client_stream: ClientStream;
    let (ws, _) = tokio_tungstenite::connect_async("ws://127.0.0.1:3030/test1").await?;

    (client_sink, client_stream) = ws.split();
    tokio::spawn(async move {
        let service_list_in_update = get_service_list(SERVICE_SERVER_URLS.to_vec()).await;
        if let Err(e) = update_service_urls(service_list_in_update, push_urls_in_update).await {
            println!("update service urls error! {}", e);
            panic!()
        }
    });
    Ok(())
}
