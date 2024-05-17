pub struct ClientPushMsg {
    pub msg: String,
}
impl ClientPushMsg {
    pub fn new() -> Self {
        Self { msg: String::new() }
    }
    pub fn msg(&mut self, msg: &str) -> &mut Self {
        self.msg = msg.to_string();
        self
    }
}
