mod game;
mod menu;
mod matchmaking;
mod util;
mod connection;
mod controls;
mod buttons;
mod persistent;

use quicksilver::{
    Result,
    geom::{Transform, Vector},
    graphics::{Color, Background::Col},
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
    FadeOut(Box<Scene>, f32, Box<Future<Item=Box<Scene>, Error=quicksilver::Error>>),
    Loading(Box<Future<Item=Box<Scene>, Error=quicksilver::Error>>),
    FadeIn(Box<Scene>, f32),
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

            DrawScene::FadeOut(scene, progress, next) => {
                let progress = progress + window.update_rate() as f32 / 500.0;
                if progress > 1.0 {
                    DrawScene::Loading(next)
                } else {
                    DrawScene::FadeOut(scene, progress, next)
                }
            },

            DrawScene::Loading(mut future) => {
                if let Ok(Async::Ready(scene)) = future.poll() {
                    DrawScene::FadeIn(scene, 0.0)
                } else {
                    DrawScene::Loading(future)
                }
            },

            DrawScene::Loaded(mut scene) => {
                scene.update(window)?;
                if let Some(next) = scene.advance() {
                    DrawScene::FadeOut(scene, 0.0, next)
                } else {
                    DrawScene::Loaded(scene)
                }
            },

            DrawScene::FadeIn(scene, progress) => {
                let progress = progress + window.update_rate() as f32 / 500.0;
                if progress > 1.0 {
                    DrawScene::Loaded(scene)
                } else {
                    DrawScene::FadeIn(scene, progress)
                }
            },

            other => other,
        };

        replace(self, next);

        Ok(())
    }

    fn event(&mut self, event: &Event, window: &mut Window) -> Result<()> {
        if let &mut DrawScene::Loaded(ref mut scene) = self {
            scene.event(event, window)?;
        }
        Ok(())
    }

    fn draw(&mut self, window: &mut Window) -> Result<()> {
        match self {
            &mut DrawScene::FadeOut(ref mut scene, progress, _) => {
                scene.draw(window)?;
                let color = Color { a: progress, ..Color::BLACK };
                let trans = Transform::IDENTITY;
                window.draw_ex(&util::rect(0.0, 0.0, 640.0, 360.0), Col(color), trans, 100);
            },
            &mut DrawScene::Loaded(ref mut scene) => {
                scene.draw(window)?;
            },
            &mut DrawScene::FadeIn(ref mut scene, progress) => {
                scene.draw(window)?;
                let color = Color { a: 1.0 - progress, ..Color::BLACK };
                let trans = Transform::IDENTITY;
                window.draw_ex(&util::rect(0.0, 0.0, 640.0, 360.0), Col(color), trans, 100);
            },
            &mut _ => {
                window.clear(Color::BLACK)?;
            },
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
                         //scale: ImageScaleStrategy::Blur,
                         ..Settings::default()
                     });
}
