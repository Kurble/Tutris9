use serde::*;
use mirror::*;
use std::iter::{repeat, repeat_with};
use rand::random;

pub struct ServerPrivate {
    pub players: Vec<String>,
    pub awaiting: Vec<String>,
    pub timer: usize,
}

#[ReflectFn(
    Fn(name="login", args="1"),
    Fn(name="drop", args="3"),
)]
#[derive(Serialize, Deserialize, Reflect)]
pub struct InstanceState {
    pub context: Hidden<ServerPrivate>,
    pub games: Vec<PlayerState>,
    pub started: bool,
    pub done: bool,
}

#[derive(Serialize, Deserialize, Reflect)]
pub struct PlayerState {
    pub field: Vec<u8>,
    pub score: usize,
    pub hold: u8,
    pub current: u8,
    pub next: Vec<u8>,
    pub ko: bool,
    pub target: usize,
    pub seed: usize,
}

impl InstanceState {
    pub fn login(&mut self, player: String) {
        self.context.as_mut().unwrap().awaiting.retain(|key| key.as_str() != player.as_str());
    }

    pub fn drop(&mut self, player: String, x: usize, y: usize) {

    }
}

impl PlayerState {
    pub fn new() -> Self {
        Self {
            field: repeat(0).take(10*20).collect(),
            score: 0,
            current: random::<u8>() % 7,
            hold: 8,
            next: repeat_with(|| random::<u8>() % 7).take(32).collect(),
            ko: false,
            target: 0,
            seed: random(),
        }
    }

    //
    pub fn drop(&mut self, x: usize, y: usize) -> usize {
        0
    }
}
