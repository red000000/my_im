use crate::client_push_msg::ClientPushMsg;
use crate::const_file::message_type::HEARTBEAT_MESSAGE;
use crate::const_file::{DEFAULT_SLEEP_TIME, SERVER_WEB_SOCKET_URL};

use super::const_file::SERVICE_SERVER_URLS;
use super::data::GetServicesMsg;
use super::grpc_errors::GrpcErrors;
use super::services::greeter_client::GreeterClient as ServicesClient;
use futures_util::{SinkExt, StreamExt, TryStreamExt};
use log::{info, warn};
use reqwest::Client as ReqClient;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::Duration;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tonic::transport::{Channel, Endpoint};
use tower::balance::p2c;
use tower::discover::ServiceList;
use tower::load::{CompleteOnResponse, PeakEwma};

type ClientSink = futures_util::stream::SplitSink<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    tokio_tungstenite::tungstenite::Message,
>;
type ClientSinkMut<'a> = futures_util::stream::SplitSink<
    &'a mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    tokio_tungstenite::tungstenite::Message,
>;
type ClientStream = futures_util::stream::SplitStream<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
>;
type ClientStreamMut<'a> = futures_util::stream::SplitStream<
    &'a mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
>;

pub struct Client {
    alive: bool,
    heart_beat_client: reqwest::Client,
    sink: Option<Arc<Mutex<ClientSink>>>,
    stream: Option<Arc<Mutex<ClientStream>>>,
}
pub struct ClientTools;
pub enum HandleMode {
    Normal,
    Unknown,
}
impl Client {
    ///must use ws func beacuse ws default None
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let heart_beat_client = reqwest::ClientBuilder::new().no_proxy().build()?;
        Ok(Self {
            alive: false,
            heart_beat_client: heart_beat_client,
            sink: None,
            stream: None,
        })
    }
    //TODO:need complete it
    pub async fn heart_beat(
        &mut self,
        push_urls: Arc<Mutex<Vec<String>>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let heart_beat_msg = format!("{},{}", HEARTBEAT_MESSAGE, "");
        let mut interval =
            tokio::time::interval(tokio::time::Duration::from_secs(DEFAULT_SLEEP_TIME));
        let sink = Arc::clone(self.sink.as_ref().unwrap());
        loop {
            interval.tick().await;
            println!("heart_beat run!");
            let mut all_failed = true;

            let urls = push_urls.try_lock()?;
            for url in urls.iter() {
                match sink
                    .try_lock()?
                    .send(Message::Text(heart_beat_msg.clone()))
                    .await
                {
                    Ok(_) => {
                        // 如果有一个 URL 发送成功，则将标记设置为 false
                        all_failed = false;
                        break;
                    }
                    Err(err) => {
                        // 发送失败，继续下一个 URL
                        println!("Failed to send to {}: {:?}", url, err);
                    }
                }
            }
            if all_failed {
                // 如果所有 URL 都发送失败，则退出循环
                return Err("All URLs failed to send".into());
            }
        }
    }
    pub async fn push(
        &mut self,
        push_urls: Arc<Mutex<Vec<String>>>,
        msg: ClientPushMsg,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let urls = push_urls.try_lock()?;
        let mut all_failed = true; // 标记是否所有 URL 都发送失败
        let string_msg = format!("{}", msg.msg);
        let sink = Arc::clone(self.sink.as_ref().unwrap());
        // 遍历发送每个 URL
        for url in urls.iter() {
            match sink
                .try_lock()?
                .send(Message::Text(string_msg.clone()))
                .await
            {
                Ok(_) => {
                    // 如果有一个 URL 发送成功，则将标记设置为 false
                    all_failed = false;
                    break;
                }
                Err(err) => {
                    // 发送失败，继续下一个 URL
                    println!("Failed to send to {}: {:?}", url, err);
                }
            }
        }
        // 检查是否所有 URL 都发送失败
        if all_failed {
            // 如果所有 URL 都发送失败，则返回错误
            Err("push error".into())
        } else {
            Ok(())
        }
    }
    //TODO:recevice server msg
    async fn recevice(&self, mut client_stream: ClientStream) {
        while let Ok(Some(message)) = client_stream.try_next().await {
            match message {
                Message::Text(text) => {
                    info!("Received: {}", text);
                }
                Message::Close(_) => {
                    info!("Connection closed");
                    break;
                }
                _ => {
                    warn!("Received unexpected message");
                }
            }
        }
    }
    pub fn ws(&mut self, ws: WebSocketStream<MaybeTlsStream<TcpStream>>) -> &mut Self {
        let (sink, stream) = ws.split();
        self.sink = Some(Arc::new(Mutex::new(sink)));
        self.stream = Some(Arc::new(Mutex::new(stream)));
        self
    }
}
impl ClientTools {
    pub fn get_hanlde_mode(msg: &Message) -> HandleMode {
        let msg_str = msg.to_string();
        let msg_vec = msg_str.split(",").collect::<Vec<&str>>();
        match msg_vec[0] {
            "normal" => HandleMode::Normal,
            _ => HandleMode::Unknown,
        }
    }
    pub async fn connect_async_urls(
        urls: Vec<String>,
    ) -> Result<Option<WebSocketStream<MaybeTlsStream<TcpStream>>>, Box<dyn std::error::Error>>
    {
        let mut all_failed = true;
        let mut web_socket = None;
        for url in urls {
            match tokio_tungstenite::connect_async(url).await {
                Ok((ws, _)) => {
                    // 如果有一个 URL 发送成功，则将标记设置为 false
                    all_failed = false;
                    web_socket = Some(ws);
                }
                Err(err) => {
                    info!("{}", err);
                }
            }
        }
        if all_failed {
            // 如果所有 URL 都发送失败，则返回错误
            Err("push error".into())
        } else {
            Ok(web_socket)
        }
    }
    pub async fn update_service_urls(
        urls: Vec<&'static str>,
        push_urls: Arc<Mutex<Vec<String>>>,
    ) -> Result<(), GrpcErrors> {
        let service_list = ClientTools::get_service_list(urls).await;
        let balance = p2c::Balance::new(service_list);
        let mut client_sink = ServicesClient::new(balance);

        let mut interval = tokio::time::interval(Duration::from_secs(DEFAULT_SLEEP_TIME));
        loop {
            interval.tick().await;
            push_urls.try_lock()?.clear();
            let mut stream = client_sink
                .get_services(GetServicesMsg::default())
                .await?
                .into_inner();
            while let Ok(Some(service_msg)) = stream.try_next().await {
                if service_msg.service_group == "push_server_urls" && service_msg.live_type == "1" {
                    info!("push_server_urls: {:?}", service_msg.url);
                    push_urls.try_lock()?.push(service_msg.url);
                }
            }
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
}
#[tokio::test]
async fn test_heart_beat() {
    std::env::set_var("RUST_LOG", "info,tower_http=debug");
    pretty_env_logger::init();
    let push_urls = Arc::new(Mutex::new(Vec::new()));
    let push_urls_in_update = push_urls.clone();
    let push_urls_in_heart = push_urls.clone();

    let req_client = ReqClient::builder().no_proxy().build().unwrap();
    let msg = ClientPushMsg {
        msg: "hello".to_string(),
    };
    let mut connect_urls = Vec::new();
    for default_url in SERVICE_SERVER_URLS {
        connect_urls.push(format!("{}{}", default_url, SERVER_WEB_SOCKET_URL));
    }

    let ws = ClientTools::connect_async_urls(connect_urls)
        .await
        .unwrap()
        .unwrap();

    let mut client = Client::new().unwrap();
    client.ws(ws);
    client.push(push_urls, msg).await.unwrap();
    let client = Arc::new(Mutex::new(client));
    tokio::spawn(async move {
        if let Err(e) =
            ClientTools::update_service_urls(SERVICE_SERVER_URLS.to_vec(), push_urls_in_update)
                .await
        {
            println!("update service urls error! {}", e);
            panic!()
        }
    });

    tokio::spawn(async move {
        if let Err(e) = client
            .try_lock()
            .unwrap()
            .heart_beat(push_urls_in_heart)
            .await
        {
            println!("heart beat error! {}", e);
            panic!()
        }
    });

    loop {}
}
