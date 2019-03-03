use super::*;

use tetris_model::connection::*;

use quicksilver::{
    Result,
    Future,
    graphics::{Background::Img, Color, Font, FontStyle, Image},
    input::{Key, ButtonState},
    lifecycle::Window,
};

pub struct Menu {
    play: Image,
    exit: Image,
    connect: bool,
}

impl Menu {
    pub fn new() -> Self {
        let font = Font::load("font.ttf").wait().expect("unable to load font");
        let button_style = FontStyle::new(48.0, Color::WHITE);

        let play = font.render("Enter game (space)", &button_style).unwrap();
        let exit = font.render("Exit (f4)", &button_style).unwrap();

        Self {
            play,
            exit,
            connect: false,
        }
    }
}

impl Scene for Menu {
    fn update(&mut self, _: &mut Window) -> Result<()> {
        //
        Ok(())
    }

    fn event(&mut self, event: &Event, _: &mut Window) -> Result<()> {
        match event {
            Event::Key(Key::Space, ButtonState::Pressed) => {
                self.connect = true;
            },
            Event::Key(Key::F4, ButtonState::Pressed) => {
                panic!("this is quit");
            },

            _ => (),
        }

        Ok(())
    }

    fn draw(&mut self, window: &mut Window) -> Result<()> {
        window.clear(Color::BLACK)?;

        let mut area = self.play.area();
        area.pos.x = 128.0;
        area.pos.y = 128.0;
        window.draw(&area, Img(&self.play));

        let mut area = self.exit.area();
        area.pos.x = 128.0;
        area.pos.y = 200.0;
        window.draw(&area, Img(&self.exit));

        Ok(())
    }

    fn advance(&mut self) -> Option<Box<Scene>> {
        if self.connect {
            self.connect = false;
            client::Client::new(make_connection("ws://127.0.0.1:3000/instance/0"))
                .ok()
                .map(|client| Box::new(matchmaking::Matchmaking::new(client)) as Box<_>)
        } else {
            None
        }
    }
}
