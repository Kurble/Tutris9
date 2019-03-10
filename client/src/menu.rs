use super::*;
use crate::connection::make_connection;

use quicksilver::{
    Result,
    geom::{Transform, Rectangle},
    graphics::{Background::Img, Background::Col, Color, Font, FontStyle, Image, View},
    input::{ButtonState, MouseButton},
    lifecycle::Window,
    combinators::Future,
};

struct Button {
    rectangles: Vec<Rectangle>,
    rectangles_hover: Vec<Rectangle>,
    color: Color,
    hover: f32,
    click: bool,
    clicked: bool,
    enabled: bool,
    text: Image,
}

pub struct Menu {
    logo: Image,
    buttons: Vec<Button>,
    pattern: Image,
    pattern_timer: f32,
}

impl Menu {
    pub fn new() -> Box<Future<Item=Box<Scene>, Error=quicksilver::Error>> {
        let font = Font::load("font.ttf");
        let pattern = Image::load("pattern.png");
        let logo = Image::load("logo.png");

        Box::new(font.join(pattern.join(logo)).map(|(font, (pattern, logo))| {
            let button_style = FontStyle::new(48.0, Color::WHITE);

            let mut buttons = Vec::new();
            buttons.push(Button {
                rectangles: vec![
                    Rectangle::new(Vector::new(280.0, 160.0), Vector::new(80.0, 80.0))
                ],
                rectangles_hover: vec![
                    Rectangle::new(Vector::new(280.0, 80.0), Vector::new(160.0, 160.0))
                ],
                color: Color { r: 1.0, g: 0.9, b: 0.2, a: 1.0 },
                hover: 0.0,
                click: false,
                clicked: false,
                enabled: true,
                text: font.render("Stats", &button_style).unwrap(),
            });
            buttons.push(Button {
                rectangles: vec![
                    Rectangle::new(Vector::new(320.0, 240.0), Vector::new(80.0, 40.0)),
                    Rectangle::new(Vector::new(280.0, 280.0), Vector::new(80.0, 40.0)),
                ],
                rectangles_hover: vec![
                    Rectangle::new(Vector::new(320.0, 200.0), Vector::new(160.0, 80.0)),
                    Rectangle::new(Vector::new(240.0, 280.0), Vector::new(160.0, 80.0)),
                ],
                color: Color { r: 0.2, g: 1.0, b: 0.1, a: 1.0 },
                hover: 0.0,
                click: false,
                clicked: false,
                enabled: true,
                text: font.render("Controls", &button_style).unwrap(),
            });
            buttons.push(Button {
                rectangles: vec![
                    Rectangle::new(Vector::new(240.0, 200.0), Vector::new(40.0, 40.0)),
                    Rectangle::new(Vector::new(200.0, 240.0), Vector::new(120.0, 40.0)),
                ],
                rectangles_hover: vec![
                    Rectangle::new(Vector::new(200.0, 160.0), Vector::new(80.0, 80.0)),
                    Rectangle::new(Vector::new(120.0, 240.0), Vector::new(240.0, 80.0)),
                ],
                color: Color { r: 1.0, g: 0.1, b: 0.9, a: 1.0 },
                hover: 0.0,
                click: false,
                clicked: false,
                enabled: true,
                text: font.render("Play", &button_style).unwrap(),
            });
            buttons.push(Button {
                rectangles: vec![
                    Rectangle::new(Vector::new(120.0, 240.0), Vector::new(40.0, 40.0)),
                    Rectangle::new(Vector::new(40.0, 280.0), Vector::new(120.0, 40.0)),
                ],
                rectangles_hover: vec![
                    Rectangle::new(Vector::new(200.0, 200.0), Vector::new(80.0, 80.0)),
                    Rectangle::new(Vector::new(40.0, 280.0), Vector::new(240.0, 80.0)),
                ],
                color: Color { r: 1.0, g: 0.7, b: 0.5, a: 1.0 },
                hover: 0.0,
                click: false,
                clicked: false,
                enabled: false,
                text: font.render("Cancel", &button_style).unwrap(),
            });

            Box::new(Self {
                logo,
                buttons,
                pattern,
                pattern_timer: 0.0,
            }) as Box<Scene>
        }))
    }
}

impl Button {
    fn rectangles<'a>(&'a self) -> impl Iterator<Item=Rectangle> + 'a {
        let v = self.hover.min(0.15) / 0.15;
        let u = 1.0 - v;
        self.rectangles
            .iter()
            .zip(self.rectangles_hover.iter())
            .map(move |(a, b)| Rectangle::new(a.pos * u + b.pos * v, a.size * u + b.size * v))
    }
}

impl Scene for Menu {
    fn update(&mut self, window: &mut Window) -> Result<()> {
        let mouse = window.mouse().pos();
        let mouse_inside = move |rect: &Rectangle| {
            mouse.x >= rect.pos.x &&
                mouse.y >= rect.pos.y &&
                mouse.x < rect.pos.x+rect.size.x &&
                mouse.y < rect.pos.y+rect.size.y
        };

        for button in self.buttons.iter_mut() {
            if button.enabled {
                if button.rectangles().find(mouse_inside).is_some() {
                    button.hover += window.update_rate() as f32 / 1000.0;
                    button.hover = button.hover.min(0.15);
                } else {
                    button.hover -= window.update_rate() as f32 / 1000.0;
                    button.hover = button.hover.max(0.0);
                }
            } else {
                button.hover = 0.0;
            }
        }
        Ok(())
    }

    fn event(&mut self, event: &Event, window: &mut Window) -> Result<()> {
        let mouse = window.mouse().pos();
        let mouse_inside = move |rect: &Rectangle| {
            mouse.x >= rect.pos.x &&
                mouse.y >= rect.pos.y &&
                mouse.x < rect.pos.x+rect.size.x &&
                mouse.y < rect.pos.y+rect.size.y
        };

        match event {
            Event::MouseButton(MouseButton::Left, ButtonState::Pressed) => {
                if let Some(ref mut button) = self.buttons.iter_mut()
                    .rev()
                    .find(|button| button.rectangles().find(mouse_inside).is_some()) {
                    button.click = button.enabled;
                }
            },
            Event::MouseButton(MouseButton::Left, ButtonState::Released) => {
                for button in self.buttons.iter_mut() {
                    if button.click && button.rectangles().find(mouse_inside).is_some() {
                        button.clicked = button.enabled;
                    }
                    button.click = false;
                }
            }

            _ => (),
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
        for button in self.buttons.iter().filter(|button| button.enabled) {
            let mut text_weight = 0.0;
            let mut text_pos = Vector::ZERO;
            for rect in button.rectangles() {
                window.draw_ex(&rect, Col(button.color), Transform::IDENTITY, -1);

                text_pos += (rect.pos + rect.size * 0.5) * rect.size.x * rect.size.y;
                text_weight += rect.size.x * rect.size.y;
            }
            text_pos *= 1.0 / text_weight;

            let text_size = button.text.area().size;
            text_pos.x -= text_size.x * 0.5;
            text_pos.y -= text_size.y * 0.5;
            window.draw_ex(&Rectangle::new(text_pos, text_size), Img(&button.text),
                           Transform::IDENTITY, 0);
        }

        Ok(())
    }

    fn advance(&mut self) -> Option<Box<Future<Item=Box<Scene>, Error=quicksilver::Error>>> {
        if self.buttons[2].clicked {
            self.buttons[2].clicked = false;

            let address = format!("{}//{}/instance/0", util::get_protocol(), util::get_host());
            let client = mirror::Client::new(make_connection(address.as_str()));
            Some(matchmaking::Matchmaking::new(client))
        } else {
            None
        }
    }
}
