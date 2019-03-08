use serde::*;
use mirror::*;
use std::iter::{repeat, repeat_with};
use std::time::{Instant, Duration};
use std::mem::replace;
use rand::{SeedableRng, Rng, random, thread_rng};
use rand::rngs::StdRng;
use rand::seq::IteratorRandom;

pub struct ServerState {
    pub players: Vec<String>,
    pub awaiting: Vec<String>,
    pub deadline: Instant,
}

#[ReflectFn(
    Fn(name="server_update", args="0"),
    Fn(name="login", args="1"),
    Fn(name="drop", args="2"),
    Fn(name="target", args="2"),
    Fn(name="hold", args="1"),
)]
#[derive(Serialize, Deserialize, Reflect)]
pub struct InstanceState {
    pub state: Hidden<ServerState>,
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
    Fn(name="gen_garbage", args="0"),
)]
#[derive(Serialize, Deserialize, Reflect)]
pub struct PlayerState {
    pub random: Hidden<StdRng>,
    pub field: Vec<u8>,
    pub score: usize,
    pub hold: u8,
    pub held: bool,
    pub current: u8,
    pub next: Vec<u8>,
    pub ko: bool,
    pub target: usize,
    pub moves: usize,
    pub combo: usize,
    pub garbage: Vec<(u8, u8)>,
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
}

impl InstanceState {
    pub fn new(players: Vec<String>) -> Self {
        Self {
            state: Hidden::new(ServerState {
                players: players.clone(),
                awaiting: players.clone(),
                deadline: Instant::now() + Duration::from_secs(10),
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

    pub fn player_ko<C: Context>(&mut self, context: &mut C, player_key: &str) {
        let index = self.state.player_index(player_key).unwrap();
        context.command(self, format!("games/{}/ko/set:true", index).as_str()).unwrap();
        context.command(self, format!("games_ko/push:{}", index).as_str()).unwrap();
    }

    pub fn in_game(&self, player: usize) -> bool {
        self.started &&
            !self.done &&
            !self.games[player].ko &&
            self.games_ko.len() != self.games.len() - 1
    }

    fn server_update<C: Context>(&mut self, mut context: C) {
        if Instant::now() > self.state.deadline && self.started == false {
            context.command(self, "started/set:true").unwrap();
            context.command(self, "status/set:\"In game\"").unwrap();
            let missed = replace(&mut self.state.awaiting, Vec::new());

            for player in missed {
                if let Some(index) = self.state.player_index(player.as_str()) {
                    context.command(self, format!("games/{}/ko/set:true", index).as_str()).unwrap();
                    context.command(self, format!("games_ko/push:{}", index).as_str()).unwrap();
                }
            }
        }

        if self.started && !self.done && self.games.iter().filter(|g| !g.ko).count() < 2 {
            context.command(self, "done/set:true").unwrap();
        }

        if !self.done {
            for i in 0..self.games.len() {
                if self.games[i].target >= self.games.len() ||
                    self.games[self.games[i].target].ko || self.games[i].target == i {
                    let new_target = self.games
                        .iter()
                        .enumerate()
                        .filter(|&(j, g)| i != j && !g.ko)
                        .map(|(j, _)| j)
                        .choose(&mut thread_rng())
                        .unwrap();

                    context.command(self, format!("games/{}/target/set:{}", i, new_target)).unwrap();
                }
            }
        }
    }

    fn login<C: Context>(&mut self, mut context: C, player: String) {
        if self.started == false { ;
            self.state.awaiting.retain(|key| key.as_str() != player.as_str());
            let count = self.state.awaiting.len();
            context
                .command(self, format!("status/set:\"Waiting for players.. ({})\"", count))
                .unwrap();

            // update the timer so that we don't have to wait too long
            let option1 = Instant::now() + Duration::from_secs(5);
            let option2 = self.state.deadline + Duration::from_secs(1);
            if option2 < option1 {
                self.state.deadline = option2;
            } else {
                self.state.deadline = option1;
            }
        }
    }

    fn drop<C: Context>(&mut self, mut context: C, player: String, state: ActiveState) {
        if let Some(id) = self.state.player_index(player.as_str()) {
            if self.in_game(id) {
                context.command(self, format!("games/{}/moves/set:{}", id,
                                              self.games[id].moves + 1)).unwrap();

                if self.games[id].held {
                    context.command(self, format!("games/{}/held/set:false", id)).unwrap();
                }

                // place the tetrimino
                for y in 0..4 {
                    for x in 0..4 {
                        let shape = self.games[id].current as usize;
                        let rotation = state.rotation as usize;
                        let col = super::shapes::SHAPES[shape][rotation][x + y * 4];
                        if col != 0 && state.y + y as i32 >= 0 {
                            let index = (state.y + y as i32) * 10 + state.x + x as i32;

                            self.games[id].field[index as usize] = col;

                            context.command(self, format!("games/{}/field/{}/set:{}", id, index,
                                                          col)).unwrap();
                        }
                    }
                }

                // check for cleared lines
                let mut lines = 0;
                for y in 0..4 {
                    let y = state.y + y;
                    if y >= 0 && y < 21 {
                        let line = (y * 10) as usize;
                        let clear = self.games[id].field[line..line + 10]
                            .iter()
                            .fold(true, |clear, &b| clear && b > 0);
                        if clear {
                            context.command(self, format!("games/{}/call:clear:{}",id,y)).unwrap();
                            lines += 1;
                        }
                    }
                }
                if lines > 0 {
                    context.command(self, format!("games/{}/call:compact:", id)).unwrap();
                    context.command(self, format!("games/{}/combo/set:{}", id,
                                                  self.games[id].combo + 1)).unwrap();

                    let garbage = match lines {
                        2 => 1,
                        3 => 2,
                        4 => 4,
                        _ => 0,
                    };

                    if garbage > 0 {
                        let column = random::<u32>() % 10;
                        for i in 0..garbage {
                            if i < self.games[id].garbage.len() {
                                context
                                    .command(self, format!("games/{}/garbage/remove:0", id))
                                    .unwrap();
                            } else {
                                context
                                    .command(self, format!("games/{}/garbage/push:[{},3]",
                                                           self.games[id].target, column))
                                    .unwrap();
                            }
                        }
                    }

                } else if self.games[id].combo > 0 {
                    context.command(self, format!("games/{}/combo/set:{}", id, 0)).unwrap();
                }

                if self.games[id].garbage.len() > 0 {
                    context.command(self, format!("games/{}/call:gen_garbage:", id)).unwrap();
                }

                // check for k.o.
                if self.games[id].field[..10].iter().find(|&&x| x > 0).is_some() {
                    context.command(self, format!("games/{}/ko/set:true", id)).unwrap();
                    context.command(self, format!("games_ko/push:{}", id)).unwrap();
                }

                // move on to the next piece
                let next = self.games[id].random.as_mut().unwrap().gen::<u8>() % 7;
                context.command(self, format!("games/{}/current/set:{}", id,
                                              self.games[id].next[0])).unwrap();
                context.command(self, format!("games/{}/next/remove:0", id)).unwrap();
                context.command(self, format!("games/{}/next/push:{}", id, next)).unwrap();
            }
        }
    }

    fn target<C: Context>(&mut self, mut context: C, player: String, target: usize) {
        if let Some(id) = self.state.player_index(player.as_str()) {
            if self.in_game(id) {
                let mut actual_target = target;

                if actual_target >= self.games.len() || self.games[actual_target].ko ||
                    actual_target == id {
                    actual_target = self.games
                        .iter()
                        .enumerate()
                        .filter(|&(j, g)| id != j && !g.ko)
                        .map(|(j, _)| j)
                        .choose(&mut thread_rng())
                        .unwrap();
                }

                context
                    .command(self, format!("games/{}/target/set:{}", id, actual_target))
                    .unwrap();
            }
        }
    }

    fn hold<C: Context>(&mut self, mut context: C, player: String) {
        if let Some(id) = self.state.player_index(player.as_str()) {
            if self.in_game(id) && !self.games[id].held {
                context.command(self, format!("games/{}/held/set:true", id)).unwrap();

                let old = self.games[id].hold;
                let current = self.games[id].current;

                context.command(self, format!("games/{}/hold/set:{}", id, current)).unwrap();

                if old == 8 {
                    let next = self.games[id].random.as_mut().unwrap().gen::<u8>() % 7;
                    context.command(self, format!("games/{}/current/set:{}", id,
                                                  self.games[id].next[0])).unwrap();
                    context.command(self, format!("games/{}/next/remove:0", id)).unwrap();
                    context.command(self, format!("games/{}/next/push:{}", id, next)).unwrap();
                } else {
                    context.command(self, format!("games/{}/current/set:{}", id, old)).unwrap();
                }
            }
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
            held: false,
            next,
            ko: false,
            target: 10,
            moves: 0,
            combo: 0,
            garbage: Vec::new(),
        }
    }

    fn clear<C: Context>(&mut self, _: C, line: usize) {
        for i in 0..10 {
            self.field[line*10+i] = 0;
        }
    }

    fn compact<C: Context>(&mut self, _: C) {
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

    fn gen_garbage<C: Context>(&mut self, _: C) {
        for (column, delay) in self.garbage.iter_mut() {
            if *delay > 0 {
                *delay -= 1;
            }
            if *delay == 0 {
                self.field.extend((0..10).map(|x| if x == *column { 0 } else { 8 }));
                self.field.drain(..10).find(|&x| x > 0).is_some();
            }
        }

        self.garbage.retain(|(_, delay)| *delay > 0);
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
