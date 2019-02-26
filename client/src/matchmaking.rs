use super::*;
use super::client::*;
use super::game::Game;
use tetris_model::matchmaking::MatchmakingState;

use quicksilver::{
    Result,
    Future,
    graphics::{Background::Img, Color, Font, FontStyle},
    lifecycle::Window,
};

pub struct Matchmaking {
    pub client: Client<MatchmakingState>,
    font: Font,
    timer_style: FontStyle,
}

impl Matchmaking {
    pub fn new(client: Client<MatchmakingState>) -> Self {
        Self {
            client,

            font: Font::load("font.ttf").wait().expect("unable to load font"),
            timer_style: FontStyle::new(48.0, Color::WHITE),
        }
    }
}

impl Scene for Matchmaking {
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

    fn advance(&mut self) -> Option<Box<Scene>> {
        if !self.client.alive() {
            if self.client.done {
                self.client.done = false;
                Client::new(self.client.instance_address.as_str()).ok()
                    .map(|client| Box::new(Game::new(client,
                                                     self.client.player_id,
                                                     self.client.player_key.clone())) as Box<_>)
            } else {
                Some(Box::new(super::menu::Menu::new()))
            }
        } else {
            None
        }
    }
}
