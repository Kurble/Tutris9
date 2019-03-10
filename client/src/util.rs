use std::time::Duration;
use std::cmp::Ordering;

use quicksilver::{
    geom::{Rectangle, Transform, Vector},
    graphics::{Background::Img, Image},
    lifecycle::Window,
};

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

pub fn draw_pattern(timer: f32, pattern: &Image, view: Rectangle, window: &mut Window) {
    let size = 256.0;
    let transform = Transform::rotate(30.0);
    let inverse = transform.inverse();

    let points: Vec<_> = [
        (0.0,         0.0),
        (view.size.x, 0.0),
        (0.0,         view.size.y),
        (view.size.x, view.size.y)
    ].iter().map(|&(x, y)| inverse * Vector::new(x, y)).collect();

    let c = |a: &f32, b: &f32| a.partial_cmp(b).unwrap_or(Ordering::Equal);
    let x_min = points.iter().map(|p| p.x).min_by(c).unwrap();
    let mut y = points.iter().map(|p| p.y).min_by(c).unwrap();
    let x_max = points.iter().map(|p| p.x).max_by(c).unwrap() + size * 0.5;
    let y_max = points.iter().map(|p| p.y).max_by(c).unwrap() + size * 0.5;

    let tile = Rectangle::new(Vector::ZERO, Vector::new(size, size));

    while y < y_max {
        let mut x = x_min - size * 1.5 + timer.fract() * size;
        while x < x_max {
            window.draw_ex(&tile,
                           Img(pattern),
                           transform  * Transform::translate(Vector::new(x, y)),
                           -3);

            x += size;
        }
        y += size;
    }
}
