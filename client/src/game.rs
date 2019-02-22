use super::*;
use client::Client;
use tetris_model::instance::*;
use std::time::{Instant, Duration};

use quicksilver::{
    Future,
    Result,
    geom::{Rectangle, Transform, Vector},
    graphics::{Background::Img, Color, Image, View},
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
    ui: Image,
}

impl Game {
    pub fn new(mut client: Client<InstanceState>, player_id: usize, player_key: String) -> Self {
        client.command(format!("call:login:\"{}\"", player_key).as_str());

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
            other_blocks: Image::load("other_blocks.png").wait().expect("unable to load other_blocks.png"),
            own_bg: Image::load("own_bg.png").wait().expect("unable to load own_bg.png"),
            other_bg: Image::load("other_bg.png").wait().expect("unable to load other_bg.png"),
            ui: Image::load("ui.png").wait().expect("unable to load ui.png"),
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
                    window.draw(&rect, Img(&blocks[b as usize]));
                },
            }
        }
    }
}

impl Scene for Game {
    fn update(&mut self, _: &mut Window) -> Result<()> {
        self.client.update();

        if self.client.started {
            while self.last_line_drop.elapsed() >= Duration::from_millis(self.client.speed) {
                self.last_line_drop += Duration::from_millis(self.client.speed);
                self.state = self.client.games[self.player_id].slide_down(self.state);
            }
        } else {
            self.last_line_drop = Instant::now();
        }

        Ok(())
    }

    fn event(&mut self, event: &Event, _: &mut Window) -> Result<()> {
        if self.client.started {
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
        window.clear(Color { r: 0.1, g: 0.1, b: 0.4, a: 1.0 })?;

        // make sure we're rendering in 16:9 with the right pixel scale
        let view = Rectangle::new(Vector::ZERO, Vector::new(640.0, 360.0));
        window.set_view(View::new(view));

        // render the ui background
        window.draw_ex(&view, Img(&self.ui), Transform::IDENTITY, -2);

        // render a background for the main game
        let bg = Rectangle::new(Vector::new(240.0, 20.0), Vector::new(160.0, 320.0));
        window.draw_ex(&bg, Img(&self.own_bg), Transform::IDENTITY, -1);

        // render the blocks for the main game
        let blocks: Vec<_> = (0..8)
            .map(|i| {
                self.own_blocks.subimage(Rectangle::new(Vector::new(i as f32 * 16.0, 0.0),
                                                        Vector::new(16.0, 16.0)))
            })
            .collect();
        self.draw_game(window,
                       blocks.as_slice(),
                       self.client.games[self.player_id].field.as_slice(),
                       Vector::new(240.0, 20.0));

        // render the falling tetrimino
        let tetrimino = self.client.games[self.player_id].current as usize;
        for y in 0..4 {
            for x in 0..4 {
                let block =
                    tetris_model::shapes::SHAPES[tetrimino][self.state.rotation as usize][x+y*4];

                match block {
                    0 => (),
                    c => {
                        let pos = Vector::new(240.0, 20.0);
                        let x = self.state.x + x as i32;
                        let y = self.state.y + y as i32;
                        let rect = Rectangle::new(Vector::new(x as f32 * 16.0, y as f32 * 16.0) + pos,
                                                  Vector::new(16.0, 16.0));

                        window.draw(&rect, Img(&blocks[c as usize]));
                    }
                }

            }
        }

        for y in 0..7 {
            for x in 0..14 {
                let bg = if x < 7 {
                    Rectangle::new(Vector::new(15.0 + x as f32 * 25.0,
                                               22.0 + y as f32 * 45.0),
                                   Vector::new(20.0, 40.0))
                } else {
                    Rectangle::new(Vector::new(450.0 + (x-7) as f32 * 25.0,
                                               22.0 + y as f32 * 45.0),
                                   Vector::new(20.0, 40.0))
                };

                window.draw_ex(&bg, Img(&self.other_bg), Transform::IDENTITY, -1);
            }
        }

        Ok(())
    }

    fn advance(&mut self) -> Option<Box<Scene>> {
        //


        None
    }
}