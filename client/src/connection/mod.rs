use mirror::Remote;

#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(not(target_arch = "wasm32"))]
pub fn make_connection(uri: &str) -> impl Remote {
    self::native::WsConnection::new(websocket::sync::client::ClientBuilder::new(uri)
        .unwrap()
        .connect_insecure()
        .unwrap())
}

#[cfg(target_arch = "wasm32")]
mod wasm;
#[cfg(target_arch = "wasm32")]
pub fn make_connection(uri: &str) -> impl Remote {
    self::wasm::WsConnection::new(uri)
}
