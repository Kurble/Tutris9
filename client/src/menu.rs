use super::*;
use crate::connection::make_connection;
use crate::matchmaking::*;
use crate::controls::*;
use crate::buttons::*;

use std::collections::HashMap;

use quicksilver::{
    Result,
    geom::{Transform, Rectangle},
    graphics::{Background::Img, Color, Font, FontStyle, Image, View},
    lifecycle::Window,
    combinators::Future,
    saving::{save, load},
};

pub struct Menu {
    font: Font,
    logo: Image,
    buttons: Buttons,
    pattern: Image,
    pattern_timer: f32,
    controls: ControlMap,
    control_buttons: HashMap<BindPoint, usize>,
    await_remap: Option<BindPoint>,
    matchmaking: Option<Box<Matchmaking>>,
    current_status: String,
}

impl Menu {
    pub fn new() -> Box<Future<Item=Box<Scene>, Error=quicksilver::Error>> {
        let font = Font::load("font.ttf");
        let pattern = Image::load("pattern.png");
        let logo = Image::load("logo.png");

        Box::new(font.join(pattern.join(logo)).map(|(font, (pattern, logo))| {
            let button_style = FontStyle::new(48.0, Color::WHITE);

            let controls = load("tutris9", "_controls").unwrap_or(ControlMap::default());

            let mut buttons = Buttons::new();
            buttons.push(Button::new(
                vec![
                    Rectangle::new(Vector::new(280.0, 160.0), Vector::new(80.0, 80.0))
                ],
                vec![
                    Rectangle::new(Vector::new(280.0, 80.0), Vector::new(160.0, 160.0))
                ],
                Color { r: 1.0, g: 0.9, b: 0.2, a: 1.0 }, 0,
                Some(font.render("Stats", &button_style).unwrap())));
            buttons.push(Button::new(
                vec![
                    Rectangle::new(Vector::new(320.0, 240.0), Vector::new(80.0, 40.0)),
                    Rectangle::new(Vector::new(280.0, 280.0), Vector::new(80.0, 40.0)),
                ],
                vec![
                    Rectangle::new(Vector::new(320.0, 200.0), Vector::new(160.0, 80.0)),
                    Rectangle::new(Vector::new(240.0, 280.0), Vector::new(160.0, 80.0)),
                ],
                Color { r: 0.2, g: 1.0, b: 0.1, a: 1.0 }, 0,
                Some(font.render("Controls", &button_style).unwrap())));
            buttons.push(Button::new(
                vec![
                    Rectangle::new(Vector::new(240.0, 200.0), Vector::new(40.0, 40.0)),
                    Rectangle::new(Vector::new(200.0, 240.0), Vector::new(120.0, 40.0)),
                ],
                vec![
                    Rectangle::new(Vector::new(200.0, 160.0), Vector::new(80.0, 80.0)),
                    Rectangle::new(Vector::new(120.0, 240.0), Vector::new(240.0, 80.0)),
                ],
                Color { r: 1.0, g: 0.1, b: 0.9, a: 1.0 }, 0,
                Some(font.render("Play", &button_style).unwrap())));
            buttons.push(Button::new(
                vec![
                    Rectangle::new(Vector::new(120.0, 240.0), Vector::new(40.0, 40.0)),
                    Rectangle::new(Vector::new(40.0, 280.0), Vector::new(120.0, 40.0)),
                ],
                vec![
                    Rectangle::new(Vector::new(120.0, 200.0), Vector::new(80.0, 80.0)),
                    Rectangle::new(Vector::new(-40.0, 280.0), Vector::new(240.0, 80.0)),
                ],
                Color { r: 1.0, g: 0.45, b: 0.25, a: 1.0 }, 1,
                Some(font.render("Cancel", &button_style).unwrap())));
            buttons.push(Button::new(
                vec![
                    Rectangle::new(Vector::new(120.0, 240.0), Vector::new(40.0, 40.0)),
                    Rectangle::new(Vector::new(40.0, 280.0), Vector::new(120.0, 40.0)),
                ],
                vec![
                    Rectangle::new(Vector::new(120.0, 200.0), Vector::new(80.0, 80.0)),
                    Rectangle::new(Vector::new(-40.0, 280.0), Vector::new(240.0, 80.0)),
                ],
                Color { r: 1.0, g: 0.45, b: 0.25, a: 1.0 }, 2,
                Some(font.render("Back", &button_style).unwrap())));
            buttons.push(Button::new(
                vec![
                    Rectangle::new(Vector::new(120.0, 240.0), Vector::new(40.0, 40.0)),
                    Rectangle::new(Vector::new(40.0, 280.0), Vector::new(120.0, 40.0)),
                ],
                vec![
                    Rectangle::new(Vector::new(120.0, 200.0), Vector::new(80.0, 80.0)),
                    Rectangle::new(Vector::new(-40.0, 280.0), Vector::new(240.0, 80.0)),
                ],
                Color { r: 1.0, g: 0.45, b: 0.25, a: 1.0 }, 3,
                Some(font.render("Back", &button_style).unwrap())));
            buttons.push(Button::new(
                vec![
                    Rectangle::new(Vector::new(200.0, 280.0), Vector::new(240.0, 40.0)),
                ],
                vec![
                    Rectangle::new(Vector::new(180.0, 280.0), Vector::new(280.0, 40.0)),
                ],
                Color { r: 0.1, g: 0.1, b: 0.8, a: 1.0 }, 1,
                Some(font.render("", &button_style).unwrap())));

            let mut control_buttons = HashMap::new();
            for bp in [BindPoint::Left, BindPoint::Right, BindPoint::RotateCW,
                BindPoint::RotateCCW, BindPoint::SoftDrop, BindPoint::HardDrop,
                BindPoint::Hold].iter() {
                let y = 130.0 + control_buttons.len() as f32 * 30.0;
                let text = format!("{}: {}", bp, controls.binding(bp));

                control_buttons.insert(*bp, buttons.push(Button::new(
                    vec![
                        Rectangle::new(Vector::new(400.0, y), Vector::new(240.0, 25.0)),
                    ],
                    vec![
                        Rectangle::new(Vector::new(160.0, y), Vector::new(480.0, 25.0)),
                    ],
                    Color { r: 0.1, g: 0.1, b: 0.1, a: 1.0 }, 3,
                    Some(font.render(text.as_str(), &button_style).unwrap()))));
            }

            Box::new(Self {
                font,
                logo,
                buttons,
                pattern,
                pattern_timer: 0.0,
                controls,
                control_buttons,
                await_remap: None,
                matchmaking: None,
                current_status: "".to_string(),
            }) as Box<Scene>
        }))
    }
}

impl Drop for Menu {
    fn drop(&mut self) {
        save("tutris9", "_controls", &self.controls).ok();
    }
}

impl Scene for Menu {
    fn update(&mut self, window: &mut Window) -> Result<()> {
        if self.await_remap.is_none() {
            self.buttons.update(window);
        }

        if let Some(status) = self.matchmaking.as_ref().map(|mm| mm.status()) {
            if status.as_str() != self.current_status.as_str() {
                let button_style = FontStyle::new(48.0, Color::WHITE);
                self.current_status = status;
                self.buttons[6].set_text(Some(self.font.render(self.current_status.as_str(),
                                                               &button_style).unwrap()));
            }
        }

        // process the stats button
        if self.buttons[0].clicked() {
            self.buttons.set_menu(2);
        }

        // process the controls button
        if self.buttons[1].clicked() {
            self.buttons.set_menu(3);
        }

        // process the play button
        if self.buttons[2].clicked() {
            self.buttons.set_menu(1);

            let address = format!("{}//{}/instance/0", util::get_protocol(), util::get_host());
            let client = mirror::Client::new(make_connection(address.as_str()));
            self.matchmaking = Some(Box::new(MatchmakingImpl::new(client, self.controls.clone())));
        }

        // process the matchmaking cancel button
        if self.buttons[3].clicked() {
            self.matchmaking = None;
            self.buttons.set_menu(0);
        }

        // process the back buttons (4 and 5)
        for i in 4..=5 {
            if self.buttons[i].clicked() {
                self.buttons.set_menu(0);
            }
        }

        for (bp, btn) in self.control_buttons.iter() {
            if self.buttons[*btn].clicked() {
                let button_style = FontStyle::new(48.0, Color::WHITE);
                let text = format!("{}: ...", bp);

                self.buttons[*btn].set_text(Some(self.font.render(text.as_str(),
                                                                  &button_style).unwrap()));
                self.await_remap = Some(*bp);
            }
        }

        Ok(())
    }

    fn event(&mut self, event: &Event, window: &mut Window) -> Result<()> {
        if let Some(point) = self.await_remap {
            if let Some(binding) = ControlMap::event_to_binding(event.clone()) {
                self.controls.remap(point, binding);

                let button_style = FontStyle::new(48.0, Color::WHITE);
                let text = format!("{}: {}", point, binding);
                self.buttons[*self.control_buttons.get(&point).unwrap()]
                    .set_text(Some(self.font.render(text.as_str(), &button_style).unwrap()));

                self.await_remap = None;
            }
        } else {
            self.buttons.event(*event, window);
        }

        Ok(())
    }

    fn draw(&mut self, window: &mut Window) -> Result<()> {
        window.clear(Color::BLACK)?;

        self.pattern_timer += window.draw_rate() as f32 * 0.000015;

        let view = Rectangle::new(Vector::new(0.0, 0.0), Vector::new(640.0, 360.0));
        window.set_view(View::new(view));

        util::draw_pattern(self.pattern_timer, &self.pattern, view, window);

        // logo
        window.draw_ex(&Rectangle::new(Vector::new(224.0, 40.0), Vector::new(192.0, 72.0)),
                       Img(&self.logo), Transform::IDENTITY, 1);

        // buttons
        self.buttons.draw(window);

        Ok(())
    }

    fn advance(&mut self) -> Option<Box<Future<Item=Box<Scene>, Error=quicksilver::Error>>> {
        if self.matchmaking.as_mut().map(|mm| {
            mm.update();
            mm.is_ok()
        }).unwrap_or(false) {
            Some(self.matchmaking.take().map(|mut mm| mm.take()).unwrap())
        } else {
            None
        }
    }
}
