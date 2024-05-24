pub mod ClientMsg;
pub mod ConstFile;
pub mod Kafka;
pub mod RedisPool;
pub mod client;
pub mod grpc_errors;
pub mod data {
    include!("./proto-gen/data.rs");
}
pub mod services {
    include!("./proto-gen/services.rs");
}

pub mod push {
    include!("./proto-gen/push.rs");
}
