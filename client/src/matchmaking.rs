use super::*;
use crate::game::Game;
use crate::connection::make_connection;
use mirror::{Remote, Client};
use tetris_model::matchmaking::MatchmakingState;

use quicksilver::{
    Result,
    Future,
    graphics::{Background::Img, Color, Font, FontStyle},
    lifecycle::Window,
};

pub struct Matchmaking<R: Remote> {
    pub client: Client<MatchmakingState, R>,
    font: Font,
    timer_style: FontStyle,
}

impl<R: Remote + 'static> Matchmaking<R> {
    pub fn new<F>(client: F) -> Box<Future<Item=Box<Scene>, Error=quicksilver::Error>> where
        F: 'static + Future<Item=Client<MatchmakingState, R>, Error=mirror::Error>
    {
        let font = Font::load("font.ttf");
        let client = client.map_err(|_| quicksilver::Error::IOError(::std::io::ErrorKind::ConnectionRefused.into()));

        Box::new(client.join(font).map(move |(client, font)| {
            Box::new(Self {
                client,
                font,
                timer_style: FontStyle::new(48.0, Color::WHITE),
            }) as Box<Scene>
        }))
    }
}

impl<R: Remote> Scene for Matchmaking<R> {
    fn update(&mut self, _: &mut Window) -> Result<()> {
        self.client.update();
        Ok(())
    }

    fn event(&mut self, _event: &Event, _: &mut Window) -> Result<()> {
        Ok(())
    }

    fn draw(&mut self, window: &mut Window) -> Result<()> {
        window.clear(Color::BLACK)?;

        let text = format!("Finding match.. {}", self.client.wait_time);
        let timer = self.font.render(text.as_str(), &self.timer_style)?;
        let mut area = timer.area();

        area.pos.x = 128.0;
        area.pos.y = 128.0;

        window.draw(&area, Img(&timer));

        Ok(())
    }

    fn advance(&mut self) -> Option<Box<Future<Item=Box<Scene>, Error=quicksilver::Error>>> {
        if self.client.done {
            self.client.done = false;
            let address = format!("{}//{}/instance/{}", util::get_protocol(), util::get_host(),
                                  self.client.instance_address);
            let client = Client::new(make_connection(address.as_str()));

            Some(Game::new(client,
                           self.client.player_id,
                           self.client.player_key.clone()))
        } else if !self.client.alive() {
            Some(super::menu::Menu::new())
        } else {
            None
        }
    }
}
