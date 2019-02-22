use serde::*;
use mirror::*;
use std::iter::{repeat, repeat_with};
use std::time::{Instant, Duration};
use std::mem::replace;
use rand::random;

pub struct ServerState {
    pub players: Vec<String>,
    pub awaiting: Vec<String>,
    pub deadline: Instant,
    pub broadcast_commands: Vec<String>,
}

#[ReflectFn(
    Fn(name="login", args="1"),
    Fn(name="drop", args="3"),
)]
#[derive(Serialize, Deserialize, Reflect)]
pub struct InstanceState {
    pub context: Hidden<ServerState>,
    pub games: Vec<PlayerState>,
    pub games_ko: Vec<usize>,
    pub status: String,
    pub started: bool,
    pub done: bool,
    pub speed: u64,
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
    pub moves: usize,
}

#[derive(Clone, Copy)]
pub struct ActiveState {
    pub x: i32,
    pub y: i32,
    pub rotation: i32,
}

impl ServerState {
    pub fn player_index(&self, player_key: &str) -> Option<usize> {
        self.players
            .iter()
            .enumerate()
            .find(|(_, p)| p.as_str() == player_key)
            .map(|(i, _)| i)
    }

    pub fn player_ko(&mut self, player_key: &str) {
        let index = self.player_index(player_key).unwrap();
        self.broadcast(format!("games/{}/ko/set:true", index));
        self.broadcast(format!("games_ko/push:{}", index));
    }

    pub fn broadcast(&mut self, command: String) {
        self.broadcast_commands.push(command);
    }
}

impl InstanceState {
    pub fn new(players: Vec<String>) -> Self {
        Self {
            context: Hidden::new(ServerState {
                players: players.clone(),
                awaiting: players.clone(),
                deadline: Instant::now() + Duration::from_secs(10),
                broadcast_commands: Vec::new(),
            }),
            games: players
                .iter()
                .map(|_| PlayerState::new())
                .collect(),
            games_ko: Vec::new(),
            status: String::from("Waiting for players.."),
            started: false,
            done: false,
            speed: 750,
        }
    }

    pub fn login(&mut self, player: String) {
        if self.started == false {
            let context = self.context.as_mut().unwrap();
            context.awaiting.retain(|key| key.as_str() != player.as_str());
            let count = context.awaiting.len();
            context.broadcast(format!("status/set:\"Waiting for players.. ({})\"", count));

            // update the timer so that we don't have to wait too long
            let option1 = Instant::now() + Duration::from_secs(5);
            let option2 = context.deadline + Duration::from_secs(1);
            if option2 < option1 {
                context.deadline = option2;
            } else {
                context.deadline = option1;
            }
        }
    }

    pub fn server_update(&mut self) {
        let context = self.context.as_mut().unwrap();

        if Instant::now() > context.deadline && self.started == false {
            context.broadcast(String::from("started/set:true"));
            context.broadcast(String::from("status/set:\"In game\""));
            let missed = replace(&mut context.awaiting, Vec::new());

            for player in missed {
                context.player_ko(player.as_str());
            }
        }
    }

    pub fn drop(&mut self, player: String, x: usize, y: usize) {
        if let Some(id) = self.context.as_ref().unwrap().player_index(player.as_str()) {
            //self.context.as_mut().unwrap().broadcast("");
            self.games[id].drop(x, y);
        }
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
            moves: 0,
        }
    }

    /// Calculates a new x position after sliding left once, for the current tetrimino.
    pub fn slide_left(&self, state: ActiveState) -> ActiveState {
        ActiveState {
            x: state.x - 1,
            y: state.y,
            rotation: state.rotation,
        }
    }

    /// Calculates a new x position after sliding right once, for the current tetrimino.
    pub fn slide_right(&self, state: ActiveState) -> ActiveState {
        ActiveState {
            x: state.x + 1,
            y: state.y,
            rotation: state.rotation,
        }
    }

    /// Calculates a new y position after sliding down once, for the current tetrimino.
    pub fn slide_down(&self, state: ActiveState) -> ActiveState {
        ActiveState {
            x: state.x,
            y: state.y + 1,
            rotation: state.rotation,
        }
    }

    /// Calculates a new state after rotating right once, for the current tetrimino.
    pub fn rotate_left(&self, state: ActiveState) -> ActiveState {
        ActiveState {
            x: state.x,
            y: state.y,
            rotation: if state.rotation == 0 { 3 } else { state.rotation-1 },
        }
    }

    /// Calculates a new state after rotating right once, for the current tetrimino.
    pub fn rotate_right(&self, state: ActiveState) -> ActiveState {
        ActiveState {
            x: state.x,
            y: state.y,
            rotation: if state.rotation == 3 { 0 } else { state.rotation+1 },
        }
    }

    pub fn drop(&mut self, _x: usize, _y: usize) -> usize {
        0
    }
}
