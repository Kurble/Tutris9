use crate::controls::ControlMap;

use serde::*;

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Persistent {
    pub nickname: String,
    pub controls: ControlMap,
    pub statistics: Statistics,
}

#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub struct Statistics {
    pub lines_cleared: usize,
    pub rotations: usize,
    pub garbage_sent: usize,
    pub garbage_received: usize,
    pub firsts: usize,
    pub seconds: usize,
    pub thirds: usize,
    pub games_played: usize,
    pub hard_drops: usize,
    pub holds: usize,
    pub bricks: usize,
    pub o_blocks: usize,
    pub i_blocks: usize,
    pub s_blocks: usize,
    pub z_blocks: usize,
    pub l_blocks: usize,
    pub j_blocks: usize,
    pub t_blocks: usize,
}
