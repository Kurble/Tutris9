use super::*;
use client::Client;
use tetris_model::instance::*;

use quicksilver::{
    Future,
    Result,
    geom::{Circle, Line, Rectangle, Transform, Triangle, Vector},
    graphics::{Background::Img, Color, Image, View},
    lifecycle::{Settings, State, Window, run},
};

pub struct Game {
    client: Client<InstanceState>,
    player_id: usize,
    player_key: String,

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

            own_blocks: Image::load("own_blocks.png").wait().expect("unable to load own_blocks.png"),
            other_blocks: Image::load("other_blocks.png").wait().expect("unable to load other_blocks.png"),
            own_bg: Image::load("own_bg.png").wait().expect("unable to load own_bg.png"),
            other_bg: Image::load("other_bg.png").wait().expect("unable to load other_bg.png"),
            ui: Image::load("ui.png").wait().expect("unable to load ui.png"),
        }
    }
}

impl Scene for Game {
    fn update(&mut self, window: &mut Window) -> Result<()> {
        self.client.update();

        Ok(())
    }

    fn event(&mut self, event: &Event, window: &mut Window) -> Result<()> {
        Ok(())
    }

    fn draw(&mut self, window: &mut Window) -> Result<()> {
        let view = Rectangle::new(Vector::ZERO, Vector::new(640.0, 360.0));

        window.clear(Color { r: 0.1, g: 0.1, b: 0.4, a: 1.0 })?;
        window.set_view(View::new(view));

        window.draw_ex(&view, Img(&self.ui), Transform::IDENTITY, -2);

        let own_bg = Rectangle::new(Vector::new(240.0, 20.0), Vector::new(160.0, 320.0));

        let other_bg = Rectangle::new(Vector::ZERO, Vector::new(20.0, 40.0));

        window.draw_ex(&own_bg, Img(&self.own_bg), Transform::IDENTITY, -1);

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