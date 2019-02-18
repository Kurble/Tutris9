use serde::*;
use mirror::*;

#[derive(Deserialize, Reflect)]
pub struct MatchmakingState {
    pub instance_address: String,
    pub player_id: usize,
    pub players_found: usize,
    pub wait_time: usize,
    pub done: bool,
}