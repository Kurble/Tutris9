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
    graphics::{Background, Background::Img, Background::Blended, Color, Image, View, Font, FontStyle},
    input::{Key, ButtonState},
    lifecycle::{Window},
};

pub struct Game {
    client: Client<InstanceState>,
    player_id: usize,
    player_key: String,

    state: ActiveState,
    last_line_drop: Instant,
    return_to_menu: bool,

    font: Font,
    position_style: FontStyle,
    position: Option<(Image, usize)>,
    result_style: FontStyle,
    result: Option<Image>,
    own_blocks: Image,
    other_blocks: Image,
    own_bg: Image,
    other_bg: Image,
    ko: Image,
    bomb: Image,
    bomb_small: Image,
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
            return_to_menu: false,

            font: Font::load("font.ttf").wait().unwrap(),
            position_style: FontStyle::new(32.0, Color::WHITE),
            result_style: FontStyle::new(160.0, Color::WHITE),
            position: None,
            result: None,
            own_blocks: Image::load("own_blocks.png").wait().unwrap(),
            other_blocks: Image::load("other_blocks.png").wait().unwrap(),
            own_bg: Image::load("own_bg.png").wait().unwrap(),
            other_bg: Image::load("other_bg.png").wait().unwrap(),
            ko: Image::load("ko.png").wait().unwrap(),
            bomb: Image::load("bomb.png").wait().unwrap(),
            bomb_small: Image::load("bomb_small.png").wait().unwrap(),
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

    fn draw_game(&self, window: &mut Window, blocks: &[Image], field: &[u8], size: Vector, pos: Vector) {
        let w = size.x;
        let h = size.y;
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

        if self.client.in_game(self.player_id) {
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
        if self.client.in_game(self.player_id) {
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
                Event::Key(Key::Space, ButtonState::Pressed) => {
                    if !self.client.games[self.player_id].held {
                        self.client.games[self.player_id].held = true;
                        self.state = ActiveState::new();
                        self.client.command(format!("call:hold:\"{}\"", self.player_key).as_str());
                    }
                },
                _ => (),
            }
        } else {
            match event {
                Event::Key(Key::Space, ButtonState::Pressed) => {
                    if self.client.done {
                        self.return_to_menu = true;
                    }
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
                self.own_blocks.subimage(Rectangle::new(Vector::new(i as f32 * 32.0, 0.0),
                                                        Vector::new(32.0, 32.0)))
            })
            .collect();

        // render the blocks for the main game
        self.draw_game(window, blocks.as_slice(), &self.client.games[self.player_id].field[10..],
                       Vector::new(16.0, 16.0), Vector::new(240.0, 20.0));

        // render positions left
        let position = self.client.games.len() - self.client.games_ko.len();
        if self.position.as_ref().map(|(_, pos)| *pos != position).unwrap_or(true) {
            let text = format!("{}/{}", position, self.client.games.len());
            self.position = Some((self.font.render(text.as_str(),
                                                   &self.position_style).unwrap(), position));
        }
        if let Some((image, _)) = self.position.as_ref() {
            let size = image.area().size;
            window.draw(&Rectangle::new(Vector::new(408.0, 340.0 - size.y), size), Img(image));
        }

        // render the result
        if self.client.done || self.client.games[self.player_id].ko {
            if self.result.is_none() {
                let final_position = self.client.games_ko.iter()
                    .enumerate()
                    .find(|(_, e)| **e == self.player_id)
                    .map(|(i, _)| self.client.games.len() - i)
                    .unwrap_or(1);
                let text = format!("{}", final_position);
                self.result = Some(self.font.render(text.as_str(), &self.result_style).unwrap());
            }

            if let Some(result) = self.result.as_ref() {
                let size = result.area().size;
                window.draw(&Rectangle::new(Vector::new(320.0 - size.x * 0.25, 80.0), size * 0.5),
                            Img(result));
            }
        }

        if self.client.in_game(self.player_id) {
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
        }

        let blocks: Vec<_> = (0..8)
            .map(|i| {
                self.other_blocks.subimage(Rectangle::new(Vector::new(i as f32 * 16.0, 0.0),
                                                          Vector::new(16.0, 16.0)))
            })
            .collect();

        if self.client.in_game(self.player_id) {
            // render the next tetriminoes
            for i in 0..6 {
                let id = self.client.games[self.player_id].next[i] as usize;
                let pos = Vector::new(408.0, 24.0 + 32.0 * i as f32);
                for y in 0..4 {
                    for x in 0..4 {
                        let color = tetris_model::shapes::SHAPES[id][0][x + y * 4] as usize;
                        if color > 0 {
                            let rect = Rectangle::new(Vector::new(8.0 * x as f32, 8.0 * y as f32) + pos,
                                                      Vector::new(8.0, 8.0));
                            window.draw(&rect, Img(&blocks[color]));
                        }
                    }
                }
            }

            // render hold tetrimino
            if self.client.games[self.player_id].hold < 8 {
                let id = self.client.games[self.player_id].hold as usize;
                let pos = Vector::new(204.0, 24.0);
                for y in 0..4 {
                    for x in 0..4 {
                        let color = tetris_model::shapes::SHAPES[id][0][x + y * 4] as usize;
                        if color > 0 {
                            let rect = Rectangle::new(Vector::new(8.0 * x as f32, 8.0 * y as f32) + pos,
                                                      Vector::new(8.0, 8.0));
                            window.draw(&rect, Img(&blocks[color]));
                        }
                    }
                }
            }

            // render waiting garbage
            for (i, (_, delay)) in self.client.games[self.player_id].garbage.iter().enumerate() {
                let rect = Rectangle::new(Vector::new(228.0, 332.0 - i as f32 * 12.0),
                                          Vector::new(8.0, 8.0));
                let bomb = Rectangle::new(Vector::new(16.0 * *delay as f32, 0.0),
                                          Vector::new(16.0, 16.0));
                window.draw(&rect, Img(&self.bomb.subimage(bomb)));
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
                                   Vector::new(8.0, 8.0), bg.pos);

                    // render waiting garbage
                    for (i, (_, delay)) in self.client.games[i].garbage.iter().enumerate() {
                        let rect = Rectangle::new(Vector::new(bg.pos.x-5.0,
                                                              bg.pos.y+bg.size.y-4.0-i as f32*5.0),
                                                  Vector::new(4.0, 4.0));
                        let bomb = Rectangle::new(Vector::new(8.0 * *delay as f32, 0.0),
                                                  Vector::new(8.0, 8.0));
                        window.draw(&rect, Img(&self.bomb_small.subimage(bomb)));
                    }

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
        if self.return_to_menu {
            Some(Box::new(super::menu::Menu::new()))
        } else {
            None
        }
    }
}