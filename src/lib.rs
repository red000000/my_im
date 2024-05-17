pub mod client;
pub mod client_push_msg;
pub mod client_trait;
pub mod const_file;
pub mod grpc_errors;
pub mod redis_pool;
pub mod data {
    include!("./proto-gen/data.rs");
}
pub mod services {
    include!("./proto-gen/services.rs");
}
/*
pub mod push {
    include!("./proto-gen/push.rs");
}
*/
