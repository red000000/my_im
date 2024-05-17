use super::const_file::*;
use super::data::ServiceMsg;
use super::grpc_errors::GrpcErrors;
use bb8::Pool;
use bb8_redis::redis::AsyncCommands;
use bb8_redis::RedisConnectionManager;
use std::boxed::Box;
#[derive(Debug)]
pub struct RedisPool {
    pub pool: Pool<RedisConnectionManager>,
}

impl RedisPool {
    pub async fn new(redis_url: &'static str) -> Result<Self, Box<dyn std::error::Error>> {
        // 创建 Redis 连接池管理器
        let manager = RedisConnectionManager::new(redis_url)?;
        // 使用管理器创建连接池
        let pool = Pool::builder().build(manager).await?;

        Ok(RedisPool { pool })
    }
    ///get service msgs from redis
    pub async fn get_service_msgs_from_redis(&self) -> Result<Vec<ServiceMsg>, GrpcErrors> {
        let mut datas = Vec::new();
        let mut services = Vec::new();
        let mut conn = self.pool.get().await?;

        // 获取所有服务组的键（例如 从service_server_urls中获取push_server_urls）
        let service_groups: Vec<String> = conn.smembers(GENERAL_SERVICE_GROUP).await?;

        for service_group_key in service_groups {
            let pattern = format!("{}:*", service_group_key);
            // 获取每个服务组内的所有服务
            let service_groups: Vec<String> = conn.keys(pattern).await?;
            for service_group in service_groups {
                let service_group_keys: Vec<String> = conn.hkeys(&service_group).await?;
                for key in service_group_keys {
                    let data: String = conn.hget(&service_group, key).await?;
                    datas.push(data);
                }
            }
        }
        for chunks in datas.chunks(CHUNK_SIZE_BY_REDIS_DATA) {
            if let [id, service_group, live_type, url] = chunks {
                let service = ServiceMsg {
                    id: id.to_string(),
                    service_group: service_group.to_string(),
                    live_type: live_type.to_string(),
                    url: url.to_string(),
                };
                services.push(service);
            } else {
                return Err(GrpcErrors::GetDataFromRedisFailed(
                    "Chunk pattern mismatch,the data is not right!!",
                ));
            }
        }
        Ok(services)
    }
    pub async fn set_live_type_by_service_group_and_id(
        &self,
        flag: bool,
        service_group_and_id: impl ToString,
    ) -> Result<(), GrpcErrors> {
        self.pool
            .get()
            .await?
            .hset(service_group_and_id.to_string(), "live_type", flag)
            .await?;
        Ok(())
    }
}
