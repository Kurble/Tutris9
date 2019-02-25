extern crate quicksilver;
extern crate mirror;
extern crate tetris_model;
extern crate serde;
extern crate serde_json;
extern crate rand;

mod client;
mod game;
mod matchmaking;

use quicksilver::{
    Result,
    geom::Vector,
    lifecycle::{Settings, State, Window, Event, run},
};

pub trait Scene {
    fn update(&mut self, window: &mut Window) -> Result<()>;
    fn event(&mut self, event: &Event, window: &mut Window) -> Result<()>;
    fn draw(&mut self, window: &mut Window) -> Result<()>;
    fn advance(&mut self) -> Option<Box<Scene>>;
}

struct DrawScene {
    current: Box<Scene>,
}

impl State for DrawScene {
    fn new() -> Result<Self> {
        Ok(DrawScene {
            current: Box::new(
                matchmaking::Matchmaking::new(
                    client::Client::new("127.0.0.1:1337").unwrap()
                )
            )
        })
    }

    fn update(&mut self, window: &mut Window) -> Result<()> {
        self.current.update(window)?;

        if let Some(next) = self.current.advance() {
            self.current = next;
        }

        Ok(())
    }

    fn event(&mut self, event: &Event, window: &mut Window) -> Result<()> {
        self.current.event(event, window)
    }

    fn draw(&mut self, window: &mut Window) -> Result<()> {
        self.current.draw(window)
    }
}

fn main() {
    run::<DrawScene>("Tetris 99 clone",
                     Vector::new(640, 360),
                     Settings {
                         draw_rate: 16.6666667,
                         update_rate: 16.6666667,
                         vsync: true,
                         ..Settings::default()
                     });
}
