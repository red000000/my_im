use uuid::Uuid;

use crate::ConstFile::client_message_type::{HEARTBEAT_MESSAGE, TEXT_MESSAGE};

pub enum ClientMsgType {
    HeartBeat,
    Text,
}
pub struct ClientMsg {
    //type need frist
    pub msg_type: ClientMsgType,
    pub msg_client_uuid: Uuid,
    pub msg_data: String,
    //TODO:Change type to uuid to ensure uniqueness, need two uuid to be a new id,there is the conversations id
    pub msg_key: String,
}
impl Clone for ClientMsgType {
    fn clone(&self) -> Self {
        match self {
            ClientMsgType::HeartBeat => ClientMsgType::HeartBeat,
            ClientMsgType::Text => ClientMsgType::Text,
        }
    }
}
impl ToString for ClientMsgType {
    fn to_string(&self) -> String {
        match self {
            ClientMsgType::HeartBeat => HEARTBEAT_MESSAGE.to_string(),
            ClientMsgType::Text => TEXT_MESSAGE.to_string(),
        }
    }
}

impl ClientMsg {
    /// default type is text
    pub fn new() -> Self {
        Self {
            msg_client_uuid: Uuid::nil(),
            msg_type: ClientMsgType::Text,
            msg_data: "".to_string(),
            msg_key: "".to_string(),
        }
    }
    pub fn msg_client_uuid(&mut self, msg_client_uuid: Uuid) -> &mut Self {
        self.msg_client_uuid = msg_client_uuid;
        self
    }
    pub fn msg_type(&mut self, msg_type: ClientMsgType) -> &mut Self {
        self.msg_type = msg_type;
        self
    }
    pub fn msg_data(&mut self, msg_data: String) -> &mut Self {
        self.msg_data = msg_data;
        self
    }
}
impl Clone for ClientMsg {
    fn clone(&self) -> Self {
        Self {
            msg_client_uuid: self.msg_client_uuid,
            msg_type: self.msg_type.clone(),
            msg_data: self.msg_data.clone(),
            msg_key: self.msg_key.clone(),
        }
    }
}
