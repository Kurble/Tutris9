extern crate serde;
extern crate serde_json;
extern crate mirror;
extern crate byteorder;
extern crate rand;

#[cfg(target_arch = "wasm32")]
#[macro_use]
extern crate stdweb;

pub mod connection;

pub mod shapes;

pub mod instance;
pub mod matchmaking;


