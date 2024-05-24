use crate::data::{GetServicesMsg, Msg};
use crate::grpc_errors::GrpcErrors;
use crate::services::greeter_client::GreeterClient as ServicesClient;
use crate::ClientMsg::{ClientMsg, ClientMsgType};
use crate::ConstFile::client_message_type::{HEARTBEAT_MESSAGE, RESULT_MESSAGE};
use crate::ConstFile::client_result_type::{FAILED, SUCCESS};
use crate::ConstFile::service_group::WS_SERVER_GROUP;
use crate::ConstFile::times::CLIENT_DEFAULT_SLEEP_TIME;
use crate::ConstFile::{SERVER_WEB_SOCKET_URL, SERVICE_SERVER_URLS, WS_MODE};
use futures_util::{SinkExt, StreamExt, TryStreamExt};
use log::{error, info, warn};

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
use uuid::Uuid;

type ClientSink = futures_util::stream::SplitSink<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    tokio_tungstenite::tungstenite::Message,
>;

type ClientStream = futures_util::stream::SplitStream<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
>;
type BalancedServiceList =
    p2c::Balance<ServiceList<Vec<PeakEwma<Channel>>>, http::Request<tonic::body::BoxBody>>;
pub struct Client {
    //TODO:the uuid need a sign up button! there is test
    uuid: Uuid,
    alive: Arc<Mutex<bool>>,
    sink: Option<Arc<Mutex<ClientSink>>>,
    stream: Option<Arc<Mutex<ClientStream>>>,
    push_urls: Arc<Mutex<Vec<String>>>,
}
pub struct ClientTools;
pub enum HandleMode {
    Result,
    Unknown,
}
impl Client {
    ///must use ws func beacuse ws default None
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            uuid: Uuid::new_v4(),
            alive: Arc::new(Mutex::new(false)),
            sink: None,
            stream: None,
            push_urls: Arc::new(Mutex::new(Vec::new())),
        })
    }

    pub async fn run(&mut self) {
        let push_urls_in_update = Arc::clone(&self.push_urls);
        tokio::spawn(async move {
            if let Err(e) =
                ClientTools::update_service_urls(SERVICE_SERVER_URLS.to_vec(), push_urls_in_update)
                    .await
            {
                error!("update service urls error! {}", e);
                panic!()
            }
        });
        let push_urls_in_connect = Arc::clone(&self.push_urls);
        let alive_in_connect = Arc::clone(&self.alive);
        let ws = ClientTools::connect_async_urls(push_urls_in_connect, alive_in_connect)
            .await
            .expect("connect service servers error!")
            .expect("get option ws error!");
        self.ws(ws);
        let sink_in_heart_beat = Arc::clone(self.sink.as_ref().expect("get inited sink error!"));
        let alive_in_heart_beat = Arc::clone(&self.alive);
        let stream_in_recevice = Arc::clone(self.stream.as_ref().unwrap());
        tokio::spawn(async move {
            if let Err(e) = ClientTools::heart_beat(sink_in_heart_beat).await {
                error!("heart beat error! {}", e);
                *alive_in_heart_beat
                    .try_lock()
                    .expect("get client alive error!") = false;
                panic!()
            }
        });
        tokio::spawn(async move {
            if let Err(e) = ClientTools::recevice(stream_in_recevice).await {
                error!("{}", e);
            }
        });
        loop {
            tokio::time::sleep(Duration::from_secs(CLIENT_DEFAULT_SLEEP_TIME)).await;
        }
    }

    fn ws(&mut self, ws: WebSocketStream<MaybeTlsStream<TcpStream>>) -> &mut Self {
        let (sink, stream) = ws.split();
        self.sink = Some(Arc::new(Mutex::new(sink)));
        self.stream = Some(Arc::new(Mutex::new(stream)));
        self
    }
}
impl ClientTools {
    pub async fn connect_async_urls(
        push_urls: Arc<Mutex<Vec<String>>>,
        alive: Arc<Mutex<bool>>,
    ) -> Result<Option<WebSocketStream<MaybeTlsStream<TcpStream>>>, Box<dyn std::error::Error>>
    {
        let mut all_failed = true;
        let mut web_socket = None;

        tokio::time::sleep(Duration::from_secs(CLIENT_DEFAULT_SLEEP_TIME)).await; //等待urls更新
        let urls = push_urls.try_lock()?;
        for url in urls.iter() {
            match tokio_tungstenite::connect_async(format!(
                "{}{}{}",
                WS_MODE, url, SERVER_WEB_SOCKET_URL
            ))
            .await
            {
                Ok((ws, _)) => {
                    // 如果有一个 URL 发送成功，则将标记设置为 false
                    all_failed = false;
                    *alive.try_lock()? = true;
                    web_socket = Some(ws);
                    break;
                }
                Err(err) => {
                    info!("{}", err);
                }
            }
        }

        if all_failed {
            // 如果所有 URL 都发送失败，则返回错误
            Err("connect server error".into())
        } else {
            Ok(web_socket)
        }
    }
    async fn update_service_urls(
        service_urls: Vec<&'static str>,
        ws_urls: Arc<Mutex<Vec<String>>>,
    ) -> Result<(), GrpcErrors> {
        let balance_service_list = ClientTools::get_balanced_service_list(service_urls).await;
        let mut service_client = ServicesClient::new(balance_service_list);

        loop {
            ws_urls.try_lock()?.clear();
            let mut stream = service_client
                .get_services(GetServicesMsg::default())
                .await?
                .into_inner();
            while let Ok(Some(service_msg)) = stream.try_next().await {
                //只获取ws服务器urls
                if service_msg.service_group == WS_SERVER_GROUP && service_msg.live_type == "1" {
                    info!("ws_server_url: {}", service_msg.url);
                    ws_urls.try_lock()?.push(service_msg.url);
                }
            }
            tokio::time::sleep(Duration::from_secs(CLIENT_DEFAULT_SLEEP_TIME)).await;
        }
    }
    /// send heart beat by ws
    async fn heart_beat(sink: Arc<Mutex<ClientSink>>) -> Result<(), Box<dyn std::error::Error>> {
        let heart_beat_msg = format!("{}", HEARTBEAT_MESSAGE);
        loop {
            sink.try_lock()?
                .send(Message::Text(heart_beat_msg.clone()))
                .await?;
            info!("heart beat running");
            tokio::time::sleep(Duration::from_secs(CLIENT_DEFAULT_SLEEP_TIME)).await;
        }
    }
    async fn push(
        sink: Arc<Mutex<ClientSink>>,
        msg: ClientMsg,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let string_msg = format!(
            "{},{},{},{}",
            msg.msg_type.to_string(),
            msg.msg_client_uuid,
            msg.msg_data,
            msg.msg_key,
        );
        sink.try_lock()?
            .send(Message::Text(string_msg.clone()))
            .await?;
        Ok(())
    }
    async fn recevice(stream: Arc<Mutex<ClientStream>>) -> Result<(), Box<dyn std::error::Error>> {
        info!("recevice running");

        while let Ok(Some(message)) = stream.try_lock()?.try_next().await {
            match message {
                Message::Text(text) => {
                    info!("Received: {}", text);
                    match ClientTools::get_handle_mode(&text) {
                        HandleMode::Result => {
                            //TODO:result msg handle
                            info!("{},{}", "result message:", text);
                            let mut text_vec = text.split(',').collect::<Vec<&str>>();
                            text_vec.remove(0);
                            let result = text_vec.join(",");
                            match result.as_str() {
                                SUCCESS => {
                                    info!("success to push msg!")
                                }
                                FAILED => {
                                    warn!("failed to push msg! please push again");
                                }
                                _ => {
                                    warn!("unknown result!")
                                }
                            }
                        }
                        HandleMode::Unknown => {
                            //TODO:unknown msg handle
                            warn!("{},{}", "unknown message:", text);
                        }
                    }
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
        Ok(())
    }
    fn get_handle_mode(msg: &String) -> HandleMode {
        let msg_str = msg.to_string();
        let msg_vec = msg_str.split(",").collect::<Vec<&str>>();
        match msg_vec[0] {
            RESULT_MESSAGE => HandleMode::Result,
            _ => HandleMode::Unknown,
        }
    }
    ///from urls get service list
    pub async fn get_balanced_service_list(urls: Vec<&'static str>) -> BalancedServiceList {
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
        p2c::Balance::new(service_list)
    }
}
#[tokio::test]
async fn test_heart_beat() {
    std::env::set_var("RUST_LOG", "info,tower_http=debug");
    pretty_env_logger::init();

    let mut client = Client::new().unwrap();
    let msg = ClientMsg {
        msg_client_uuid: client.uuid,
        msg_type: ClientMsgType::Text,
        msg_data: String::from("test"),
        msg_key: String::from("hui-hua-1"),
    };

    let push_urls_clone = Arc::clone(&client.push_urls);
    let push_urls_clone_clone = Arc::clone(&client.push_urls);
    let alive_clone = Arc::clone(&client.alive);
    tokio::spawn(async move {
        if let Err(e) =
            ClientTools::update_service_urls(SERVICE_SERVER_URLS.to_vec(), push_urls_clone).await
        {
            error!("update service urls error! {}", e);
            panic!()
        }
    });
    let ws = ClientTools::connect_async_urls(push_urls_clone_clone, alive_clone)
        .await
        .unwrap()
        .unwrap();
    client.ws(ws);
    let sink_clone = Arc::clone(&client.sink.unwrap());
    let stream_clone = Arc::clone(&client.stream.unwrap());
    tokio::spawn(async move {
        if let Err(e) = ClientTools::recevice(stream_clone).await {
            error!("recevice error! {}", e);
            panic!()
        }
    });

    tokio::time::sleep(Duration::from_secs(1)).await;
    ClientTools::push(sink_clone, msg).await.unwrap();
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
        info!("loop!");
    }
}
