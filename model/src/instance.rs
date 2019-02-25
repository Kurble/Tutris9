use serde::*;
use mirror::*;
use std::iter::{repeat, repeat_with};
use std::time::{Instant, Duration};
use std::mem::replace;
use rand::{SeedableRng, Rng, random};
use rand::rngs::StdRng;

pub struct ServerState {
    pub players: Vec<String>,
    pub awaiting: Vec<String>,
    pub deadline: Instant,
    pub broadcast_commands: Vec<String>,
}

#[ReflectFn(
    Fn(name="login", args="1"),
    Fn(name="drop", args="2"),
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

#[ReflectFn(
    Fn(name="clear", args="1"),
    Fn(name="compact", args="0"),
    Fn(name="attack", args="2"),
)]
#[derive(Serialize, Deserialize, Reflect)]
pub struct PlayerState {
    pub random: Hidden<StdRng>,
    pub field: Vec<u8>,
    pub score: usize,
    pub hold: u8,
    pub current: u8,
    pub next: Vec<u8>,
    pub ko: bool,
    pub target: usize,
    pub moves: usize,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
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

    pub fn drop(&mut self, player: String, state: ActiveState) {
        if let Some(id) = self.context.as_ref().unwrap().player_index(player.as_str()) {
            let context = self.context.as_mut().unwrap();

            context.broadcast(format!("games/{}/moves/set:{}", id, self.games[id].moves + 1));

            let mut ko = false;

            // place the tetrimino
            for y in 0..4 {
                for x in 0..4 {
                    let shape = self.games[id].current as usize;
                    let rotation = state.rotation as usize;
                    let col = super::shapes::SHAPES[shape][rotation][x+y*4];
                    if col != 0 && state.y + y as i32 >= 0 {
                        let index = (state.y + y as i32) * 10 + state.x + x as i32;

                        self.games[id].field[index as usize] = col;

                        context.broadcast(format!("games/{}/field/{}/set:{}",
                                                  id,
                                                  index,
                                                  col));
                    }
                }
            }

            // check for cleared lines
            let mut have_clear = false;
            for y in 0..4 {
                let y = state.y + y;
                if y >= 0 && y < 21 {
                    let line = (y * 10) as usize;
                    let clear = self.games[id].field[line..line + 10]
                        .iter()
                        .fold(true, |clear, &b| clear && b > 0);
                    if clear {
                        context.broadcast(format!("games/{}/call:clear:{}", id, y));
                        have_clear = true;
                    }
                }
            }
            if have_clear {
                context.broadcast(format!("games/{}/call:compact:", id));
            }

            // check for k.o.
            if self.games[id].field[..10].iter().find(|&&x| x > 0).is_some() {
                context.player_ko(player.as_str());
            }

            // move on to the next piece
            let next = self.games[id].random.as_mut().unwrap().gen::<u8>() % 7;
            context.broadcast(format!("games/{}/current/set:{}",
                                      id,
                                      self.games[id].next[0]));
            context.broadcast(format!("games/{}/next/remove:0", id));
            context.broadcast(format!("games/{}/next/push:{}", id, next));
        }
    }
}

impl PlayerState {
    pub fn new() -> Self {
        let mut rng = StdRng::from_seed(random());
        let current = rng.gen::<u8>() % 7;
        let next = repeat_with(|| rng.gen::<u8>() % 7).take(32).collect();

        Self {
            random: Hidden::new(rng),

            field: repeat(0).take(10*21).collect(),
            score: 0,
            current,
            hold: 8,
            next,
            ko: false,
            target: 0,
            moves: 0,
        }
    }

    fn clear(&mut self, line: usize) {
        for i in 0..10 {
            self.field[line*10+i] = 0;
        }
    }

    fn compact(&mut self) {
        for _ in 0..4 {
            for y in 0..20 {
                let y = 20 - y;
                if self.field[(y * 10)..].iter().take(10).find(|&&x| x > 0).is_none() {
                    for x in 0..10 {
                        self.field[y*10 + x] = self.field[(y-1)*10 + x];
                        self.field[(y-1)*10 + x] = 0;
                    }
                }
            }
        }
    }

    fn attack(&mut self, column: usize, count: usize) {
        for _ in 0..count {
            self.field.extend((0..10).map(|x| if x == column { 0 } else { 8 }));
            self.field.drain(..10).find(|&x| x > 0).is_some();
        }
    }

    fn collision(&self, state: ActiveState) -> bool {
        let grid = &super::shapes::SHAPES[self.current as usize][state.rotation as usize];
        let field = &self.field[0..];

        for y in 0..4 {
            for x in 0..4 {
                if grid[x+y*4] != 0 {
                    let x = state.x + x as i32;
                    let y = state.y + y as i32;
                    if x < 0 || x > 9 || y > 20 || (y >= 0 && field[(x+y*10) as usize] != 0) {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Calculates a new x position after sliding left once, for the current tetrimino.
    pub fn slide_left(&self, state: ActiveState) -> ActiveState {
        let next = ActiveState {
            x: state.x - 1,
            y: state.y,
            rotation: state.rotation,
        };

        if self.collision(next) { state } else { next }
    }

    /// Calculates a new x position after sliding right once, for the current tetrimino.
    pub fn slide_right(&self, state: ActiveState) -> ActiveState {
        let next = ActiveState {
            x: state.x + 1,
            y: state.y,
            rotation: state.rotation,
        };

        if self.collision(next) { state } else { next }
    }

    /// Calculates a new y position after sliding down once, for the current tetrimino.
    pub fn slide_down(&self, state: ActiveState) -> ActiveState {
        let next = ActiveState {
            x: state.x,
            y: state.y + 1,
            rotation: state.rotation,
        };

        if self.collision(next) { state } else { next }
    }

    /// Calculates a new state after rotating right once, for the current tetrimino.
    pub fn rotate_left(&self, state: ActiveState) -> ActiveState {
        let next = ActiveState {
            x: state.x,
            y: state.y,
            rotation: if state.rotation == 0 { 3 } else { state.rotation-1 },
        };

        let current = self.current as usize;
        let rotation = next.rotation as usize;

        if self.collision(next) {
            for kick in super::shapes::KICK_LEFT[current][rotation].iter() {
                let kicked = ActiveState {
                    x: next.x + kick.0,
                    y: next.y - kick.1,
                    rotation: next.rotation
                };
                if !self.collision(kicked) {
                    return kicked;
                }
            }
            state
        } else {
            next
        }
    }

    /// Calculates a new state after rotating right once, for the current tetrimino.
    pub fn rotate_right(&self, state: ActiveState) -> ActiveState {
        let next = ActiveState {
            x: state.x,
            y: state.y,
            rotation: if state.rotation == 3 { 0 } else { state.rotation+1 },
        };

        let current = self.current as usize;
        let rotation = next.rotation as usize;

        if self.collision(next) {
            for kick in super::shapes::KICK_RIGHT[current][rotation].iter() {
                let kicked = ActiveState {
                    x: next.x + kick.0,
                    y: next.y - kick.1,
                    rotation: next.rotation
                };
                if !self.collision(kicked) {
                    return kicked;
                }
            }
            state
        } else {
            next
        }
    }

    /// Calculates the state after performing a hard drop
    pub fn hard_drop(&self, mut state: ActiveState) -> ActiveState {
        let mut next = self.slide_down(state);
        while next.y != state.y {
            state = next;
            next = self.slide_down(state);
        }
        next
    }
}

impl ActiveState {
    pub fn new() -> Self {
        Self {
            x: 2,
            y: -4,
            rotation: 0,
        }
    }
}
