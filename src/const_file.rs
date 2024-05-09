pub const SERVICES_URLS: [&str; 2] = ["http://[::1]:50051", "http://[::1]:50053"];

/// get message slice from redis data
/// now default data struct in redis-data-struct.md
pub const CHUNK_SIZE_BY_REDIS_DATA: usize = 4;
pub const DEFAULT_REDIS_URL: &str = "redis://127.0.0.1/";
pub const GENERAL_SERVICE_GROUP: &str = "service_server_urls";

///live type define
/// "1" is true,"0" is false
pub const LIVE_TYPE_TRUE: &str = "1";
pub const LIVE_TYPE_FALSE: &str = "0";

///NO Domain Name！
pub const SERVER_HEARBEAT_URL: &str = "/heartbeat";
