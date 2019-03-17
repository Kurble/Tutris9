use std::ops::Index;
use std::ops::IndexMut;
use std::mem::replace;

use quicksilver::{
    geom::{Transform, Rectangle, Vector},
    graphics::{Background::Img, Background::Col, Color, Image},
    input::{ButtonState, MouseButton},
    lifecycle::{Window, Event},
};

pub struct Buttons {
    buttons: Vec<Button>,
    menu: usize,
}

pub struct Button {
    rectangles: Vec<Rectangle>,
    rectangles_hover: Vec<Rectangle>,
    color: Color,
    hover: f32,
    click: bool,
    clicked: bool,
    menu: usize,
    enabled: f32,
    text: Option<Image>,
}

impl Index<usize> for Buttons {
    type Output = Button;
    fn index(&self, i: usize) -> &Button {
        self.buttons.index(i)
    }
}

impl IndexMut<usize> for Buttons {
    fn index_mut(&mut self, i: usize) -> &mut Button {
        self.buttons.index_mut(i)
    }
}

impl Buttons {
    pub fn new() -> Self {
        Self {
            buttons: Vec::new(),
            menu: 0,
        }
    }

    pub fn push(&mut self, button: Button) -> usize {
        let result = self.buttons.len();
        self.buttons.push(button);
        result
    }

    pub fn set_menu(&mut self, menu: usize) {
        self.menu = menu;
    }

    pub fn update(&mut self, window: &mut Window) {
        let mouse = window.mouse().pos();
        let mouse_inside = move |rect: &Rectangle| {
            mouse.x >= rect.pos.x &&
                mouse.y >= rect.pos.y &&
                mouse.x < rect.pos.x+rect.size.x &&
                mouse.y < rect.pos.y+rect.size.y
        };

        for button in self.buttons.iter_mut() {
            if button.menu == self.menu {
                button.enabled += window.update_rate() as f32 / 1000.0;
                button.enabled = button.enabled.min(0.1);

                if button.rectangles().find(mouse_inside).is_some() {
                    button.hover += window.update_rate() as f32 / 1000.0;
                    button.hover = button.hover.min(0.15);
                } else {
                    button.hover -= window.update_rate() as f32 / 1000.0;
                    button.hover = button.hover.max(0.0);
                }
            } else {
                button.enabled -= window.update_rate() as f32 / 1000.0;
                button.enabled = button.enabled.max(0.0);
                button.hover = 0.0;
            }
        }
    }

    pub fn event(&mut self, event: Event, window: &mut Window) {
        let mouse = window.mouse().pos();
        let mouse_inside = move |rect: &Rectangle| {
            mouse.x >= rect.pos.x &&
                mouse.y >= rect.pos.y &&
                mouse.x < rect.pos.x + rect.size.x &&
                mouse.y < rect.pos.y + rect.size.y
        };

        let menu = self.menu;

        match event {
            Event::MouseButton(MouseButton::Left, ButtonState::Pressed) => {
                if let Some(ref mut button) = self.buttons.iter_mut()
                    .rev()
                    .filter(|button| button.menu == menu)
                    .find(|button| button.rectangles().find(mouse_inside).is_some()) {
                    button.click = button.enabled > 0.0;
                }
            },
            Event::MouseButton(MouseButton::Left, ButtonState::Released) => {
                for button in self.buttons.iter_mut()
                    .filter(|button| button.menu == menu) {
                    if button.click && button.rectangles().find(mouse_inside).is_some() {
                        button.clicked = button.enabled > 0.0;
                    }
                    button.click = false;
                }
            }

            _ => (),
        }
    }

    pub fn draw(&mut self, window: &mut Window) {
        for button in self.buttons.iter().filter(|button| button.enabled > 0.0) {
            let mut text_weight = 0.0;
            let mut text_pos = Vector::ZERO;
            for rect in button.rectangles() {
                window.draw_ex(&rect, Col(button.color), Transform::IDENTITY, -1);

                text_pos += (rect.pos + rect.size * 0.5) * rect.size.x * rect.size.y;
                text_weight += rect.size.x * rect.size.y;
            }
            text_pos *= 1.0 / text_weight;

            if button.enabled >= 0.1 {
                let text_size = button.text.as_ref().unwrap().area().size;
                text_pos.x -= text_size.x * 0.25;
                text_pos.y -= text_size.y * 0.25;
                window.draw_ex(&Rectangle::new(text_pos, text_size * 0.5), Img(button.text.as_ref().unwrap()),
                               Transform::IDENTITY, 0);
            }
        }
    }
}

impl Button {
    pub fn new(normal: Vec<Rectangle>, hover: Vec<Rectangle>, color: Color, menu: usize,
               text: Option<Image>) -> Self {
        Button {
            rectangles: normal,
            rectangles_hover: hover,
            color,
            menu,
            text,
            ..Button::default()
        }
    }

    pub fn clicked(&mut self) -> bool {
        replace(&mut self.clicked, false)
    }

    pub fn set_text(&mut self, text: Option<Image>) {
        self.text = text;
    }

    fn rectangles<'a>(&'a self) -> impl Iterator<Item=Rectangle> + 'a {
        let v = self.hover.min(0.15) / 0.15;
        let u = 1.0 - v;
        let e = self.enabled.min(0.1) / 0.1;
        self.rectangles
            .iter()
            .zip(self.rectangles_hover.iter())
            .map(move |(a, b)| Rectangle::new(a.pos * u + b.pos * v, a.size * u + b.size * v))
            .map(move |r| Rectangle::new(r.pos + r.size * 0.5 * (1.0-e), r.size * e))
    }
}

impl Default for Button {
    fn default() -> Button {
        Button {
            rectangles: Vec::new(),
            rectangles_hover: Vec::new(),
            color: Color::WHITE,
            hover: 0.0,
            click: false,
            clicked: false,
            menu: 0,
            enabled: 0.0,
            text: None
        }
    }
}