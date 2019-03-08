mod game;
mod menu;
mod matchmaking;
mod util;
mod connection;

use quicksilver::{
    Result,
    geom::Vector,
    graphics::Color,
    lifecycle::{Settings, State, Window, Event, run},
    combinators::Future,
};
use futures::Async;
use std::mem::replace;

pub trait Scene {
    fn update(&mut self, window: &mut Window) -> Result<()>;
    fn event(&mut self, event: &Event, window: &mut Window) -> Result<()>;
    fn draw(&mut self, window: &mut Window) -> Result<()>;
    fn advance(&mut self) -> Option<Box<Future<Item=Box<Scene>, Error=quicksilver::Error>>>;
}

enum DrawScene {
    None,
    NotFullscreen(Box<Future<Item=Box<Scene>, Error=quicksilver::Error>>),
    Loading(Box<Future<Item=Box<Scene>, Error=quicksilver::Error>>),
    Loaded(Box<Scene>),
}

impl State for DrawScene {
    fn new() -> Result<Self> {
        if cfg!(debug_assertions) {
            Ok(DrawScene::Loading(menu::Menu::new()))
        } else {
            Ok(DrawScene::NotFullscreen(menu::Menu::new()))
        }
    }

    fn update(&mut self, window: &mut Window) -> Result<()> {
        let next = match replace(self, DrawScene::None) {
            DrawScene::NotFullscreen(mut future) => {
                window.set_fullscreen(true);
                if let Ok(Async::Ready(scene)) = future.poll() {
                    DrawScene::Loaded(scene)
                } else {
                    DrawScene::Loading(future)
                }
            },

            DrawScene::Loading(mut future) => {
                if let Ok(Async::Ready(scene)) = future.poll() {
                    DrawScene::Loaded(scene)
                } else {
                    DrawScene::Loading(future)
                }
            },

            other => other,
        };

        replace(self, next);

        let next = if let &mut DrawScene::Loaded(ref mut scene) = self {
            scene.update(window)?;
            scene.advance()
        } else {
            None
        };

        if let Some(next) = next {
            replace(self, DrawScene::Loading(next));
        }

        Ok(())
    }

    fn event(&mut self, event: &Event, window: &mut Window) -> Result<()> {
        if let &mut DrawScene::Loaded(ref mut scene) = self {
            scene.event(event, window)?;
        }
        Ok(())
    }

    fn draw(&mut self, window: &mut Window) -> Result<()> {
        if let &mut DrawScene::Loaded(ref mut scene) = self {
            scene.draw(window)?;
        } else {
            window.clear(Color::BLACK)?;
        }
        Ok(())
    }
}

fn main() {
    run::<DrawScene>("Tutris 9",
                     Vector::new(640, 360),
                     Settings {
                         draw_rate: 16.6666667,
                         update_rate: 16.6666667,
                         vsync: true,
                         ..Settings::default()
                     });
}
