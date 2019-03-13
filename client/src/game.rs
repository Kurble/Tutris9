use super::*;
use crate::util::*;
use crate::persistent::*;
use crate::controls::*;
use crate::buttons::*;
use mirror::{Remote, Client};
use tetris_model::instance::*;
use std::time::Duration;
use rand::thread_rng;
use rand::seq::SliceRandom;

use quicksilver::{
    Future,
    Result,
    geom::{Rectangle, Transform, Vector},
    graphics::{Background, Background::Img, Background::Col, Background::Blended, Color, Image, View, Font, FontStyle},
    lifecycle::{Window},
    saving::save,
};

pub struct Game<R: Remote> {
    client: Client<InstanceState, R>,
    player_id: usize,
    player_key: String,
    data: Persistent,
    buttons: Buttons,

    state: ActiveState,
    last_line_drop: Duration,
    return_to_menu: bool,
    game_over_duration: Option<Duration>,

    font: Font,
    position_style: FontStyle,
    position_header: Image,
    position: Option<(Image, usize)>,

    result_style: FontStyle,
    result: Option<Image>,

    message: Image,
    own_blocks: Image,
    other_blocks: Image,
    own_bg: Image,
    other_bg: Image,
    ko: Image,
    bomb: Image,
    bomb_small: Image,
    pattern: Image,

    pattern_timer: f32,

    mapping: [usize; 8],
}

impl<R: Remote + 'static> Game<R> {
    pub fn new<F>(client: F, player_id: usize, player_key: String, data: Persistent)
        -> Box<Future<Item=Box<Scene>, Error=quicksilver::Error>>
        where
            F: 'static + Future<Item=Client<InstanceState, R>, Error=mirror::Error>
    {
        let client = client.map_err(|_| quicksilver::Error::IOError(::std::io::ErrorKind::ConnectionRefused.into()));
        let font = Font::load("font.ttf");
        let own_blocks = Image::load("own_blocks.png");
        let other_blocks = Image::load("other_blocks.png");
        let own_bg = Image::load("own_bg.png");
        let other_bg = Image::load("other_bg.png");
        let ko = Image::load("ko.png");
        let bomb = Image::load("bomb.png");
        let bomb_small = Image::load("bomb_small.png");
        let pattern = Image::load("pattern.png");

        Box::new(client.join(font.join(own_blocks.join(other_blocks.join(own_bg.join(other_bg.join(ko.join(bomb.join(bomb_small.join(pattern)))))))))
            .map(move |(mut client, (font, (own_blocks, (other_blocks, (own_bg, (other_bg, (ko, (bomb, (bomb_small, pattern)))))))))| {
                client.command(format!("call:login:\"{}\"", player_key).as_str()).unwrap();

                let mut mapping = [0; 8];
                let mut mapping_i = (0..9).filter(|&i| i != player_id);
                for i in mapping.iter_mut() {
                    *i = mapping_i.next().unwrap();
                }
                mapping.shuffle(&mut thread_rng());

                let position_style = FontStyle::new(32.0, Color::WHITE);
                let result_style = FontStyle::new(160.0, Color::WHITE);
                let position_header = font.render("Place: ", &position_style).unwrap();
                let message = font.render("Get Ready!", &result_style).unwrap();
                let mut buttons = Buttons::new();
                buttons.push(Button::new(vec![util::rect(40.0, 280.0, 150.0, 40.0)],
                                         vec![util::rect(20.0, 280.0, 190.0, 60.0)],
                                         Color { r: 0.1, g: 0.1, b:  0.8, a: 1.0 }, 1,
                                         font.render("Return", &position_style).ok()));

                Box::new(Self {
                    client,
                    player_id,
                    player_key,
                    data,
                    buttons,

                    state: ActiveState {
                        x: 2,
                        y: -4,
                        rotation: 0,
                    },
                    last_line_drop: Duration::from_secs(0),
                    return_to_menu: false,
                    game_over_duration: None,
                    font,
                    position_style,
                    result_style,
                    position: None,
                    position_header,
                    result: None,
                    message,
                    own_blocks,
                    other_blocks,
                    own_bg,
                    other_bg,
                    ko,
                    bomb,
                    bomb_small,
                    pattern,

                    pattern_timer: 0.0,
                    mapping,
                }) as Box<Scene>
            }))
    }

    fn drop_current(&mut self) {
        // send drop command
        self.client.command(format!("call:drop:\"{}\" {}",
                                    self.player_key,
                                    serde_json::to_string(&self.state).unwrap()).as_str()).unwrap();

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

        // update statistics
        self.data.statistics.bricks += 1;
        match self.client.games[self.player_id].current {
            0 => self.data.statistics.i_blocks += 1,
            1 => self.data.statistics.o_blocks += 1,
            2 => self.data.statistics.t_blocks += 1,
            3 => self.data.statistics.j_blocks += 1,
            4 => self.data.statistics.l_blocks += 1,
            5 => self.data.statistics.s_blocks += 1,
            6 => self.data.statistics.z_blocks += 1,
            _ => (),
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
                        let pos = Vector::new(230.0, 0.0);
                        let x = state.x + x as i32;
                        let y = state.y + y as i32 - 1;
                        let rect = Rectangle::new(Vector::new(x as f32 * 18.0, y as f32 * 18.0) + pos,
                                                  Vector::new(18.0, 18.0));

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

impl<R: Remote> Drop for Game<R> {
    fn drop(&mut self) {
        // append the server side session statistics
        self.data.statistics.lines_cleared += self.client.games[self.player_id].lines_cleared;
        self.data.statistics.garbage_sent += self.client.games[self.player_id].garbage_sent;
        self.data.statistics.garbage_received += self.client.games[self.player_id].garbage_received;

        // update statics that relate a number of games
        if self.client.games[self.player_id].ko || self.client.done {
            match self.client.games_ko.iter()
                .enumerate()
                .find(|(_, e)| **e == self.player_id)
                .map(|(i, _)| self.client.games.len() - i)
                .unwrap_or(1) {
                1 => self.data.statistics.firsts += 1,
                2 => self.data.statistics.seconds += 1,
                3 => self.data.statistics.thirds += 1,
                _ => (),
            }
            self.data.statistics.games_played += 1;
        }

        save("tutris9", "data", &self.data).ok();
    }
}

impl<R: Remote + 'static> Scene for Game<R> {
    fn update(&mut self, window: &mut Window) -> Result<()> {
        self.client.update();
        self.data.controls.update(window);
        self.buttons.update(window);

        if self.client.in_game(self.player_id) {
            if self.data.controls[BindPoint::Left] {
                self.state = self.client.games[self.player_id].slide_left(self.state);
            }
            if self.data.controls[BindPoint::Right] {
                self.state = self.client.games[self.player_id].slide_right(self.state);
            }
            if self.data.controls[BindPoint::SoftDrop] {
                self.state = self.client.games[self.player_id].slide_down(self.state);
            }
            if self.data.controls[BindPoint::HardDrop] {
                self.state = self.client.games[self.player_id].hard_drop(self.state);
                self.data.statistics.hard_drops += 1;
                self.drop_current();
            }
            if self.data.controls[BindPoint::RotateCCW] {
                self.state = self.client.games[self.player_id].rotate_left(self.state);
                self.data.statistics.rotations += 1;
            }
            if self.data.controls[BindPoint::RotateCW] {
                self.state = self.client.games[self.player_id].rotate_right(self.state);
                self.data.statistics.rotations += 1;
            }
            if self.data.controls[BindPoint::Hold] {
                if !self.client.games[self.player_id].held {
                    self.client.games[self.player_id].held = true;
                    self.data.statistics.holds += 1;
                    self.state = ActiveState::new();
                    self.client
                        .command(format!("call:hold:\"{}\"", self.player_key).as_str())
                        .unwrap();
                }
            }
        } else if self.buttons[0].clicked() {
            self.return_to_menu = true;
        }

        add_seconds(&mut self.last_line_drop, window.update_rate() / 1000.0);
        self.game_over_duration.as_mut().map(|go| add_seconds(go, window.update_rate() / 1000.0));

        if self.client.in_game(self.player_id) {
            while self.last_line_drop >= Duration::from_millis(self.client.speed) {
                self.last_line_drop -= Duration::from_millis(self.client.speed);
                let before = self.state;
                self.state = self.client.games[self.player_id].slide_down(self.state);

                if before.y == self.state.y {
                    self.drop_current();
                }
            }
        } else {
            self.last_line_drop = Duration::from_secs(0);
            if self.client.started && self.game_over_duration.is_none() {
                self.game_over_duration = Some(Duration::from_secs(0));
                self.message = self.font.render("Game Over!", &self.result_style).unwrap();
            }

            let limit = Duration::from_secs(3);
            if self.game_over_duration.as_ref().map(|&t| t > limit).unwrap_or(false) {
                self.buttons.set_menu(1);
            }
        }

        Ok(())
    }

    fn event(&mut self, event: &Event, window: &mut Window) -> Result<()> {
        if !self.client.in_game(self.player_id) {
            self.buttons.event(*event, window);
        }

        Ok(())
    }

    fn draw(&mut self, window: &mut Window) -> Result<()> {
        // clear the window
        window.clear(Color { r: 0.1, g: 0.2, b: 0.4, a: 1.0 })?;

        // make sure we're rendering in 16:9 with the right pixel scale
        let view = Rectangle::new(Vector::ZERO, Vector::new(640.0, 360.0));
        window.set_view(View::new(view));

        // return tiling pattern on the background
        self.pattern_timer += window.draw_rate() as f32 * 0.000015;
        util::draw_pattern(self.pattern_timer, &self.pattern, view, window);
        window.draw_ex(&Rectangle::new(Vector::new(-320.0, -120.0), Vector::new(640.0, 240.0)),
                       Col(Color::BLACK),
                       Transform::translate(Vector::new(280.0, 40.0)) * Transform::rotate(60.0),
                       -2);
        window.draw_ex(&Rectangle::new(Vector::new(-240.0, -80.0), Vector::new(480.0, 160.0)),
                       Col(Color::BLACK),
                       Transform::translate(Vector::new(480.0, 360.0)) * Transform::rotate(-30.0),
                       -2);

        // draw buttons
        self.buttons.draw(window);

        // render a background for the main game
        let bg = Rectangle::new(Vector::new(230.0, 0.0), Vector::new(180.0, 360.0));
        window.draw_ex(&bg, Img(&self.own_bg), Transform::IDENTITY, -1);

        let blocks: Vec<_> = (0..8)
            .map(|i| {
                self.own_blocks.subimage(Rectangle::new(Vector::new(i as f32 * 32.0, 0.0),
                                                        Vector::new(32.0, 32.0)))
            })
            .collect();

        // render the blocks for the main game
        self.draw_game(window, blocks.as_slice(), &self.client.games[self.player_id].field[10..],
                       Vector::new(18.0, 18.0), Vector::new(230.0, 0.0));

        // render positions left
        let position = self.client.games.len() - self.client.games_ko.len();
        if self.position.as_ref().map(|(_, pos)| *pos != position).unwrap_or(true) {
            let text = format!("{}/{}", position, self.client.games.len());
            self.position = Some((self.font.render(text.as_str(),
                                                   &self.position_style).unwrap(), position));
        }
        if let Some((image, _)) = self.position.as_ref() {
            let size = image.area().size;
            let hsize = self.position_header.area().size;
            window.draw(&Rectangle::new(Vector::new(520.0 - hsize.x * 0.5, 290.0), hsize), Img(&self.position_header));
            window.draw(&Rectangle::new(Vector::new(520.0 - size.x * 0.5, 310.0), size), Img(image));
        }

        // render the result
        if !self.client.started || self.client.done || self.client.games[self.player_id].ko {
            if self.client.started && self.result.is_none() {
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

            let tick = 0.75;

            if self.game_over_duration
                .map(|d| d.as_secs() as f64 + d.subsec_nanos() as f64 / 1_000_000_000.0)
                .map(|d| d < tick * 3.0 && (d / tick).fract() < 0.5)
                .unwrap_or(true) {
                let size = self.message.area().size;
                window.draw_ex(&Rectangle::new(Vector::new(320.0 - size.x * 0.25, 200.0),
                                               size * 0.5),
                               Img(&self.message), Transform::IDENTITY, 1);
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
                let pos = Vector::new(420.0, 144.0 + 32.0 * i as f32);
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
                        Rectangle::new(Vector::new(40.0 + x as f32 * 50.0,
                                                   40.0 + y as f32 * 90.0),
                                       Vector::new(40.0, 80.0))
                    } else {
                        Rectangle::new(Vector::new(510.0 + (x-2) as f32 * 50.0,
                                                   40.0 + y as f32 * 90.0),
                                       Vector::new(40.0, 80.0))
                    };

                    window.draw_ex(&bg, Img(&self.other_bg), Transform::IDENTITY, -1);

                    self.draw_game(window, blocks.as_slice(), &self.client.games[i].field[10..],
                                   Vector::new(4.0, 4.0), bg.pos);

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
                        window.draw_ex(&Rectangle::new(Vector::new(-4.0, 16.0) + bg.pos,
                                                       Vector::new(48.0, 48.0)),
                                       Img(&self.ko), Transform::IDENTITY, 1);
                    }
                }
            }
        }

        Ok(())
    }

    fn advance(&mut self) -> Option<Box<Future<Item=Box<Scene>, Error=quicksilver::Error>>> {
        if self.return_to_menu {
            Some(super::menu::Menu::new())
        } else {
            None
        }
    }
}