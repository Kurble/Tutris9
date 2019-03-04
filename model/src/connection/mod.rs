pub trait Connection {
    fn close(&mut self);
    fn alive(&self) -> bool;
    fn send(&mut self, message: &str);
    fn message(&mut self) -> Option<String>;
}

pub struct Messages<'a, C: 'a + Connection>(&'a mut C);

impl<'a, C: 'a + Connection> Messages<'a, C> {
    pub fn new(c: &'a mut C) -> Self {
        Messages(c)
    }
}

impl<'a, C: Connection> Iterator for Messages<'a, C> {
    type Item = String;
    fn next(&mut self) -> Option<String> {
        self.0.message()
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod ws;
#[cfg(not(target_arch = "wasm32"))]
pub fn make_connection(uri: &str) -> impl Connection {
    self::ws::WsConnection::new(websocket::sync::client::ClientBuilder::new(uri)
        .unwrap()
        .connect_insecure()
        .unwrap())
}

#[cfg(target_arch = "wasm32")]
mod stdws;
#[cfg(target_arch = "wasm32")]
pub fn make_connection(uri: &str) -> impl Connection {
    self::stdws::WsConnection::new(uri)
}
