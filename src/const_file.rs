pub const SERVICE_SERVER_URLS: [&str; 1] = ["http://[::1]:50051"];
pub const PUSH_SERVER_URLS: [&str; 1] = ["http://[::1]:50053"];
/// get message slice from redis data
/// now default data struct in redis-data-struct.md
pub const CHUNK_SIZE_BY_REDIS_DATA: usize = 4;
pub const DEFAULT_REDIS_URL: &str = "redis://127.0.0.1/";
pub const GENERAL_SERVICE_GROUP: &str = "service_server_urls";

pub const DEFAULT_SLEEP_TIME: u64 = 5;
///live type define
/// "1" is true,"0" is false
mod live_type {
    pub const __LIVE_TYPE_TRUE__: &str = "1";
    pub const __LIVE_TYPE_FALSE__: &str = "0";
}

///NO Domain Name！
pub const SERVER_HEARBEAT_URL: &str = "/heartbeat";
pub const SERVER_WEB_SOCKET_URL: &str = "/ws";

pub mod message_type {
    pub const NORMAL_MESSAGE: &str = "normal";
    pub const HEARTBEAT_MESSAGE: &str = "heartbeat";
}
