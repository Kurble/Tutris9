use super::*;
use super::client::Client;
use tetris_model::instance::*;
use std::time::{Instant, Duration};
use rand::thread_rng;
use rand::seq::SliceRandom;

use quicksilver::{
    Future,
    Result,
    geom::{Rectangle, Transform, Vector},
    graphics::{Background, Background::Img, Background::Blended, Color, Image, View},
    input::{Key, ButtonState},
    lifecycle::{Window},
};

pub struct Game {
    client: Client<InstanceState>,
    player_id: usize,
    player_key: String,

    state: ActiveState,
    last_line_drop: Instant,

    own_blocks: Image,
    other_blocks: Image,
    own_bg: Image,
    other_bg: Image,
    ko: Image,
    //ui: Image,

    mapping: [usize; 8],
}

impl Game {
    pub fn new(mut client: Client<InstanceState>, player_id: usize, player_key: String) -> Self {
        client.command(format!("call:login:\"{}\"", player_key).as_str());

        let mut mapping = [0; 8];
        let mut mapping_i = (0..9).filter(|&i| i != player_id);
        for i in mapping.iter_mut() {
            *i = mapping_i.next().unwrap();
        }
        mapping.shuffle(&mut thread_rng());

        Self {
            client,
            player_id,
            player_key,

            state: ActiveState {
                x: 2,
                y: -4,
                rotation: 0,
            },
            last_line_drop: Instant::now(),

            own_blocks: Image::load("own_blocks.png").wait().expect("unable to load own_blocks.png"),
            other_blocks: Image::load("help_blocks.png").wait().expect("unable to load other_blocks.png"),
            own_bg: Image::load("own_bg.png").wait().expect("unable to load own_bg.png"),
            other_bg: Image::load("other_bg.png").wait().expect("unable to load other_bg.png"),
            ko: Image::load("ko.png").wait().expect("error"),
            //ui: Image::load("ui.png").wait().expect("unable to load ui.png"),

            mapping,
        }
    }

    fn drop(&mut self) {
        // send drop command
        self.client.command(format!("call:drop:\"{}\" {}",
                                    self.player_key,
                                    serde_json::to_string(&self.state).unwrap()).as_str());

        // update the field in advance
        for y in 0..4 {
            for x in 0..4 {
                let shape = self.client.games[self.player_id].current as usize;
                let rotation = self.state.rotation as usize;
                let col = tetris_model::shapes::SHAPES[shape][rotation][x+y*4];
                if col != 0 {
                    if self.state.y + y as i32 >= 0 {
                        let index = ((self.state.y + y as i32) * 10 + self.state.x + x as i32) as usize;
                        self.client.games[self.player_id].field[index] = col;
                    }
                }
            }
        }

        // update the current tetrimino in advance
        self.state = ActiveState::new();
        self.client.games[self.player_id].current = self.client.games[self.player_id].next[0];
    }

    fn draw_state<F: Fn(&Image)->Background>(&self,
                                             window: &mut Window,
                                             blocks: &[Image],
                                             state: ActiveState,
                                             make_bg: F) {
        let tetrimino = self.client.games[self.player_id].current as usize;
        for y in 0..4 {
            for x in 0..4 {
                let block =
                    tetris_model::shapes::SHAPES[tetrimino][state.rotation as usize][x+y*4];

                match block {
                    0 => (),
                    c => {
                        let pos = Vector::new(240.0, 20.0);
                        let x = state.x + x as i32;
                        let y = state.y + y as i32 - 1;
                        let rect = Rectangle::new(Vector::new(x as f32 * 16.0, y as f32 * 16.0) + pos,
                                                  Vector::new(16.0, 16.0));

                        window.draw(&rect, make_bg(&blocks[c as usize]));
                    }
                }
            }
        }
    }

    fn draw_game(&self, window: &mut Window, blocks: &[Image], field: &[u8], pos: Vector) {
        let w = blocks[0].area().width();
        let h = blocks[1].area().height();
        for (i, &val) in field.iter().enumerate() {
            let x = i%10;
            let y = i/10;
            match val {
                0 => (),
                b => {
                    let rect = Rectangle::new(Vector::new(w * x as f32, h * y as f32) + pos,
                                              Vector::new(w, h));
                    window.draw(&rect, Img(&blocks[b as usize % 8]));
                },
            }
        }
    }
}

impl Scene for Game {
    fn update(&mut self, _: &mut Window) -> Result<()> {
        self.client.update();

        if self.client.started &&
            !self.client.games[self.player_id].ko &&
            self.client.games[self.player_id].current != 8 {
            while self.last_line_drop.elapsed() >= Duration::from_millis(self.client.speed) {
                self.last_line_drop += Duration::from_millis(self.client.speed);
                let before = self.state;
                self.state = self.client.games[self.player_id].slide_down(self.state);

                if before.y == self.state.y {
                    self.drop();
                }
            }
        } else {
            self.last_line_drop = Instant::now();
        }

        Ok(())
    }

    fn event(&mut self, event: &Event, _: &mut Window) -> Result<()> {
        if self.client.started && !self.client.games[self.player_id].ko {
            match event {
                Event::Key(Key::Left, ButtonState::Pressed) => {
                    self.state = self.client.games[self.player_id].slide_left(self.state);
                },
                Event::Key(Key::Right, ButtonState::Pressed) => {
                    self.state = self.client.games[self.player_id].slide_right(self.state);
                },
                Event::Key(Key::Down, ButtonState::Pressed) => {
                    self.state = self.client.games[self.player_id].slide_down(self.state);
                },
                Event::Key(Key::Up, ButtonState::Pressed) => {
                    self.state = self.client.games[self.player_id].hard_drop(self.state);
                    self.drop();
                },
                Event::Key(Key::A, ButtonState::Pressed) => {
                    self.state = self.client.games[self.player_id].rotate_left(self.state);
                },
                Event::Key(Key::D, ButtonState::Pressed) => {
                    self.state = self.client.games[self.player_id].rotate_right(self.state);
                },
                _ => (),
            }
        }

        Ok(())
    }

    fn draw(&mut self, window: &mut Window) -> Result<()> {
        // clear the window
        window.clear(Color { r: 0.1, g: 0.2, b: 0.4, a: 1.0 })?;

        // make sure we're rendering in 16:9 with the right pixel scale
        let view = Rectangle::new(Vector::ZERO, Vector::new(640.0, 360.0));
        window.set_view(View::new(view));

        // render the ui background
        //window.draw_ex(&view, Img(&self.ui), Transform::IDENTITY, -2);

        // render a background for the main game
        let bg = Rectangle::new(Vector::new(240.0, 20.0), Vector::new(160.0, 320.0));
        window.draw_ex(&bg, Img(&self.own_bg), Transform::IDENTITY, -1);

        let blocks: Vec<_> = (0..8)
            .map(|i| {
                self.own_blocks.subimage(Rectangle::new(Vector::new(i as f32 * 16.0, 0.0),
                                                        Vector::new(16.0, 16.0)))
            })
            .collect();

        // render the blocks for the main game
        self.draw_game(window,
                       blocks.as_slice(),
                       &self.client.games[self.player_id].field[10..],
                       Vector::new(240.0, 20.0));

        // render the falling tetrimino
        if !self.client.games[self.player_id].ko {
            self.draw_state(window,
                            blocks.as_slice(),
                            self.state,
                            |img| Img(img));
            self.draw_state(window,
                            blocks.as_slice(),
                            self.client.games[self.player_id].hard_drop(self.state),
                            |img| Blended(img, Color::WHITE.with_alpha(0.3)));
        }

        let blocks: Vec<_> = (0..8)
            .map(|i| {
                self.other_blocks.subimage(Rectangle::new(Vector::new(i as f32 * 8.0, 0.0),
                                                          Vector::new(8.0, 8.0)))
            })
            .collect();

        // render the next tetriminoes
        for i in 0..6 {
            let id = self.client.games[self.player_id].next[i] as usize;
            let pos = Vector::new(408.0, 24.0 + 32.0 * i as f32);
            for y in 0..4 {
                for x in 0..4 {
                    let color = tetris_model::shapes::SHAPES[id][0][x+y*4] as usize;
                    if color > 0 {
                        let rect = Rectangle::new(Vector::new(8.0 * x as f32, 8.0 * y as f32) + pos,
                                                  Vector::new(8.0, 8.0));
                        window.draw(&rect, Img(&blocks[color]));
                    }
                }
            }
        }

        // render other games
        for y in 0..2 {
            for x in 0..4 {
                let i = self.mapping[x + y * 4];
                if i < self.client.games.len() {
                    let bg = if x < 2 {
                        Rectangle::new(Vector::new(20.0 + x as f32 * 90.0,
                                                   20.0 + y as f32 * 165.0),
                                       Vector::new(80.0, 160.0))
                    } else {
                        Rectangle::new(Vector::new(450.0 + (x-2) as f32 * 90.0,
                                                   20.0 + y as f32 * 165.0),
                                       Vector::new(80.0, 160.0))
                    };

                    window.draw_ex(&bg, Img(&self.other_bg), Transform::IDENTITY, -1);

                    self.draw_game(window, blocks.as_slice(), &self.client.games[i].field[10..],
                                   bg.pos);

                    if self.client.games[i].ko {
                        window.draw_ex(&Rectangle::new(Vector::new(16.0, 56.0) + bg.pos,
                                                       Vector::new(48.0, 48.0)),
                                       Img(&self.ko), Transform::IDENTITY, 1);
                    }
                }
            }
        }

        Ok(())
    }

    fn advance(&mut self) -> Option<Box<Scene>> {
        //


        None
    }
}