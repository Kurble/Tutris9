use super::*;

use tetris_model::connection::*;

use quicksilver::{
    Result,
    graphics::{Background::Img, Color, Font, FontStyle, Image},
    input::{Key, ButtonState},
    lifecycle::Window,
    combinators::Future,
};

pub struct Menu {
    play: Image,
    exit: Image,
    connect: bool,
}

impl Menu {
    pub fn new() -> Box<Future<Item=Box<Scene>, Error=quicksilver::Error>> {
        let font = Font::load("font.ttf");

        Box::new(font.map(|font| {
            let button_style = FontStyle::new(48.0, Color::WHITE);

            let play = font.render("Enter game (space)", &button_style).unwrap();
            let exit = font.render("Exit (f4)", &button_style).unwrap();

            Box::new(Self {
                play,
                exit,
                connect: false,
            }) as Box<Scene>
        }))
    }
}

impl Scene for Menu {
    fn update(&mut self, _: &mut Window) -> Result<()> {
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

    fn advance(&mut self) -> Option<Box<Future<Item=Box<Scene>, Error=quicksilver::Error>>> {
        if self.connect {
            self.connect = false;

            let address = format!("ws://{}/instance/0", util::get_host());
            let client = client::Client::new(make_connection(address.as_str()));
            Some(matchmaking::Matchmaking::new(client))
        } else {
            None
        }
    }
}
