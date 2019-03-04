use super::*;
use super::client::*;
use super::game::Game;
use tetris_model::matchmaking::MatchmakingState;
use tetris_model::connection::*;

use quicksilver::{
    Result,
    Future,
    graphics::{Background::Img, Color, Font, FontStyle},
    lifecycle::Window,
};

pub struct Matchmaking<C: Connection> {
    pub client: Client<MatchmakingState, C>,
    font: Font,
    timer_style: FontStyle,
}

impl<C: Connection + 'static> Matchmaking<C> {
    pub fn new(client: ConnectClient<MatchmakingState, C>) -> Box<Future<Item=Box<Scene>, Error=quicksilver::Error>> {
        let font = Font::load("font.ttf");

        Box::new(client.join(font).map(move |(client, font)| {
            Box::new(Self {
                client,
                font,
                timer_style: FontStyle::new(48.0, Color::WHITE),
            }) as Box<Scene>
        }))
    }
}

impl<C: Connection> Scene for Matchmaking<C> {
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
            let address = format!("ws://{}/instance/{}", util::get_host(),
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
