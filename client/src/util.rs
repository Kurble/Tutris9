use std::time::Duration;

#[cfg(target_arch="wasm32")]
use stdweb::web::window;

pub fn add_seconds(duration: &mut Duration, seconds: f64) {
    let secs = Duration::from_secs(seconds as u64);
    let nanos = Duration::from_nanos((seconds.fract() * 1_000_000_000.0) as u64);
    *duration += secs + nanos;
}

#[cfg(target_arch="wasm32")]
pub fn get_host() -> String {
    window().location().unwrap().host().unwrap()
}

#[cfg(not(target_arch="wasm32"))]
pub fn get_host() -> String {
    String::from("127.0.0.1:3000")
}