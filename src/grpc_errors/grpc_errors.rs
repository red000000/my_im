use super::grpc_errors_status::GrpcErrorTemplate;
use tonic::Status;
// 定义一个包含具体错误状态码的枚举

pub enum GrpcErrors {
    ConnectServerFailed(&'static str),
    GetDataFromRedisFailed(&'static str),

    Io(std::io::Error),
    Json(serde_json::Error),
    ParseInt(std::num::ParseIntError),
    RedisError(bb8_redis::redis::RedisError),
    RunRedisError(bb8::RunError<bb8_redis::redis::RedisError>),
    TryLockError(tokio::sync::TryLockError),
    ReqwestError(reqwest::Error),
    ReqwestToStrError(reqwest::header::ToStrError),
    DynError(Box<dyn std::error::Error>),
    TonicTransportError(tonic::transport::Error),
    StatusError(Status),
    // 添加更多错误类型
}
impl From<std::io::Error> for GrpcErrors {
    fn from(err: std::io::Error) -> Self {
        GrpcErrors::Io(err)
    }
}

impl From<serde_json::Error> for GrpcErrors {
    fn from(err: serde_json::Error) -> Self {
        GrpcErrors::Json(err)
    }
}

impl From<std::num::ParseIntError> for GrpcErrors {
    fn from(err: std::num::ParseIntError) -> Self {
        GrpcErrors::ParseInt(err)
    }
}
impl From<bb8_redis::redis::RedisError> for GrpcErrors {
    fn from(err: bb8_redis::redis::RedisError) -> Self {
        GrpcErrors::RedisError(err)
    }
}
impl From<bb8::RunError<bb8_redis::redis::RedisError>> for GrpcErrors {
    fn from(err: bb8::RunError<bb8_redis::redis::RedisError>) -> Self {
        GrpcErrors::RunRedisError(err)
    }
}
impl From<tokio::sync::TryLockError> for GrpcErrors {
    fn from(err: tokio::sync::TryLockError) -> Self {
        GrpcErrors::TryLockError(err)
    }
}
impl From<reqwest::Error> for GrpcErrors {
    fn from(err: reqwest::Error) -> Self {
        GrpcErrors::ReqwestError(err)
    }
}
impl From<reqwest::header::ToStrError> for GrpcErrors {
    fn from(err: reqwest::header::ToStrError) -> Self {
        GrpcErrors::ReqwestToStrError(err)
    }
}
impl From<Box<dyn std::error::Error>> for GrpcErrors {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        GrpcErrors::DynError(err)
    }
}
impl From<tonic::transport::Error> for GrpcErrors {
    fn from(err: tonic::transport::Error) -> Self {
        GrpcErrors::TonicTransportError(err)
    }
}
impl From<Status> for GrpcErrors {
    fn from(error: Status) -> Self {
        // Convert tonic::Status to an appropriate GrpcErrors variant
        GrpcErrors::StatusError(error)
    }
}
impl From<GrpcErrors> for Status {
    fn from(err: GrpcErrors) -> Self {
        match err {
            GrpcErrors::ConnectServerFailed(e) => {
                GrpcErrorTemplate::CONNECT_SERVER_FAILED.get_status(e)
            }
            GrpcErrors::GetDataFromRedisFailed(e) => {
                GrpcErrorTemplate::GET_DATA_FROM_REDIS_FAILED.get_status(e)
            }
            GrpcErrors::Io(e) => GrpcErrorTemplate::IO_READ_ERROR.get_status(e),
            GrpcErrors::Json(e) => GrpcErrorTemplate::JSON_PARSE_ERROR.get_status(e),
            GrpcErrors::ParseInt(e) => GrpcErrorTemplate::PARSE_INT_ERROR.get_status(e),
            GrpcErrors::RedisError(e) => GrpcErrorTemplate::REDIS_ERROR.get_status(e),
            GrpcErrors::RunRedisError(e) => GrpcErrorTemplate::RUN_REDIS_ERROR.get_status(e),
            GrpcErrors::TryLockError(e) => GrpcErrorTemplate::TRY_LOCK_ERROR.get_status(e),
            GrpcErrors::ReqwestError(e) => GrpcErrorTemplate::REQWEST_ERROR.get_status(e),
            GrpcErrors::ReqwestToStrError(e) => {
                GrpcErrorTemplate::REQWEST_TO_STR_ERROR.get_status(e)
            }
            GrpcErrors::DynError(e) => GrpcErrorTemplate::DYN_ERROR.get_status(e),
            GrpcErrors::TonicTransportError(e) => {
                GrpcErrorTemplate::TONIC_TRANSPORT_ERROR.get_status(e)
            }
            GrpcErrors::StatusError(e) => GrpcErrorTemplate::STATUS_ERROR.get_status(e),
        }
    }
}
impl std::fmt::Display for GrpcErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GrpcErrors::ConnectServerFailed(msg) => write!(f, "连接服务器失败: {}", msg),
            GrpcErrors::GetDataFromRedisFailed(msg) => write!(f, "从Redis获取数据失败: {}", msg),
            GrpcErrors::Io(err) => write!(f, "IO错误: {}", err),
            GrpcErrors::Json(err) => write!(f, "JSON解析错误: {}", err),
            GrpcErrors::ParseInt(err) => write!(f, "解析整数错误: {}", err),
            GrpcErrors::RedisError(err) => write!(f, "Redis错误: {}", err),
            GrpcErrors::RunRedisError(err) => write!(f, "运行Redis错误: {}", err),
            GrpcErrors::TryLockError(err) => write!(f, "尝试锁定错误: {}", err),
            GrpcErrors::ReqwestError(err) => write!(f, "Reqwest错误: {}", err),
            GrpcErrors::ReqwestToStrError(err) => write!(f, "Reqwest ToStr 错误: {}", err),
            GrpcErrors::DynError(err) => write!(f, "动态错误: {}", err),
            GrpcErrors::TonicTransportError(err) => write!(f, "Tonic Transport 错误: {}", err),
            GrpcErrors::StatusError(err) => write!(f, "Status 错误: {}", err),
            // 添加更多错误类型
        }
    }
}
