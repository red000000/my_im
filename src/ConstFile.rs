pub const SERVICE_SERVER_URLS: [&str; 1] = ["http://[::1]:50051"];
pub const PUSH_SERVER_URLS: [&str; 1] = ["http://[::1]:50053"];

/// get message slice from redis data
/// now default data struct in redis-data-struct.md
pub const CHUNK_SIZE_BY_REDIS_DATA: usize = 4;
pub const DEFAULT_REDIS_URL: &str = "redis://127.0.0.1/";
pub mod times {
    pub const CLIENT_DEFAULT_SLEEP_TIME: u64 = 5;
    pub const SERVER_DEFAULT_SLEEP_TIME: u64 = 5;
    pub const KAFKA_DEFAULT_WAITING_TIME: u64 = 5;
}

///live type define
/// "1" is true,"0" is false
mod live_type {
    pub const __LIVE_TYPE_TRUE__: &str = "1";
    pub const __LIVE_TYPE_FALSE__: &str = "0";
}

///NO Domain Name！
pub const HTTP_MODE: &str = "http://";
pub const WS_MODE: &str = "ws://";
pub const SERVER_HEARBEAT_URL: &str = "/heartbeat";
pub const SERVER_WEB_SOCKET_URL: &str = "/ws";
pub mod client_mseeage_subscript {
    pub const CLIENT_UUID_SUBSCRIPT: usize = 0;
    pub const MSG_DATA_SUBSCRIPT: usize = 1;
    pub const KAFKA_KEY_SUBSCRIPT: usize = 2;
}
pub mod client_message_type {
    pub const TEXT_MESSAGE: &str = "text";
    pub const RESULT_MESSAGE: &str = "result";
    pub const HEARTBEAT_MESSAGE: &str = "heartbeat";
}
pub mod client_result_type {
    pub const SUCCESS: &str = "success";
    pub const FAILED: &str = "failed";
}

pub mod kafka {
    ///use kafka init
    pub const KAFKA_CLUSER_IP: &str = "192.168.33.144";
    pub const KAFKA_CLUSER_PORTS: [&str; 3] = ["9094", "9095", "9096"];
    pub const SECURITY_PROTOCOL: &str = "PLAINTEXT";
    pub const GROUP_ID: &str = "test-group";
    pub const ACKS: &str = "all";
    pub const AUTO_OFFSET_RESET: &str = "earliest";
    //add something
}
pub mod service_group {
    pub const GENERAL_SERVICE_GROUP: &str = "service_server_urls";
    pub const WS_SERVER_GROUP: &str = "ws_server_urls";
    pub const PUSH_SERVER_URLS: &str = "push_server_urls";
}
