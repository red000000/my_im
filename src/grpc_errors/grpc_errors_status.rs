use tonic::{Code, Status};
pub struct GrpcErrorTemplate {
    pub code: i32,
    pub message_template: &'static str,
}
impl GrpcErrorTemplate {
    pub fn get_status(&self, details: impl ToString) -> Status {
        Status::new(
            Code::from_i32(self.code),
            format!("{}: {}", self.message_template, details.to_string()),
        )
    }

    pub const CONNECT_SERVER_FAILED: GrpcErrorTemplate = GrpcErrorTemplate {
        code: 1,
        message_template: "Failed to connect to the server",
    };
    pub const GET_DATA_FROM_REDIS_FAILED: GrpcErrorTemplate = GrpcErrorTemplate {
        code: 2,
        message_template: "Failed to get data from Redis",
    };
    pub const JSON_PARSE_ERROR: GrpcErrorTemplate = GrpcErrorTemplate {
        code: 100,
        message_template: "Failed to parse JSON",
    };
    pub const IO_READ_ERROR: GrpcErrorTemplate = GrpcErrorTemplate {
        code: 101,
        message_template: "Failed to read from IO",
    };
    pub const PARSE_INT_ERROR: GrpcErrorTemplate = GrpcErrorTemplate {
        code: 102,
        message_template: "Failed to parse integer",
    };
    pub const REDIS_ERROR: GrpcErrorTemplate = GrpcErrorTemplate {
        code: 103,
        message_template: "Failed to connect redis and handle",
    };
    pub const RUN_REDIS_ERROR: GrpcErrorTemplate = GrpcErrorTemplate {
        code: 104,
        message_template: "Failed to run redis command",
    };
    pub const TRY_LOCK_ERROR: GrpcErrorTemplate = GrpcErrorTemplate {
        code: 105,
        message_template: "Failed to try lock",
    };
    pub const REQWEST_ERROR: GrpcErrorTemplate = GrpcErrorTemplate {
        code: 106,
        message_template: "Failed to send request to reqwest",
    };
    pub const REQWEST_TO_STR_ERROR: GrpcErrorTemplate = GrpcErrorTemplate {
        code: 107,
        message_template: "Failed to convert reqwest response to string",
    };
    pub const DYN_ERROR:GrpcErrorTemplate=GrpcErrorTemplate{
        code:108,
        message_template:"Failed to convert dyn error to string"
    };
    pub const TONIC_TRANSPORT_ERROR:GrpcErrorTemplate=GrpcErrorTemplate{
        code:109,
        message_template:"Failed to convert tonic transport error to string"
    };
    pub const STATUS_ERROR:GrpcErrorTemplate=GrpcErrorTemplate{
        code:110,
        message_template:"Failed to convert tonic status error to string"
    };
}
