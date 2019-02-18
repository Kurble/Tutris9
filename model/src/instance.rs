use serde::*;
use mirror::*;

#[derive(Deserialize, Reflect)]
pub struct InstanceState {
    pub games: Vec<PlayerState>,
}

#[derive(Deserialize, Reflect)]
pub struct PlayerState {
    pub field: Vec<u8>,
    pub score: usize,
    pub current: Tetromino,
    pub next: Vec<Tetromino>,
    pub ko: bool,
    pub target: usize,
}

#[derive(Deserialize, Reflect)]
pub struct Tetromino {
    pub color: u8,
    pub style: u8,
}
