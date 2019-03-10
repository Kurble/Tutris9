use std::time::Duration;

#[cfg(target_arch="wasm32")]
use stdweb::web::window;

#[cfg(not(target_arch="wasm32"))]
use std::env::args;

pub fn add_seconds(duration: &mut Duration, seconds: f64) {
    let secs = Duration::from_secs(seconds as u64);
    let nanos = Duration::from_nanos((seconds.fract() * 1_000_000_000.0) as u64);
    *duration += secs + nanos;
}

#[cfg(target_arch="wasm32")]
pub fn get_protocol() -> String {
    let proto = window().location().unwrap().protocol().unwrap();
    if proto == "http:" {
        String::from("ws:")
    } else {
        String::from("wss:")
    }
}

#[cfg(not(target_arch="wasm32"))]
pub fn get_protocol() -> String {
    args()
        .skip(1)
        .next()
        .and_then(|arg| arg.split("//").next().map(|s| s.to_string()))
        .unwrap_or("ws:".to_string())
}

#[cfg(target_arch="wasm32")]
pub fn get_host() -> String {
    window().location().unwrap().host().unwrap()
}

#[cfg(not(target_arch="wasm32"))]
pub fn get_host() -> String {
    args()
        .skip(1)
        .next()
        .and_then(|arg| arg.split("//").skip(1).next().map(|s| s.to_string()))
        .unwrap_or("localhost".to_string())
}