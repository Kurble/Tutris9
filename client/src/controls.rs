use std::collections::HashMap;
use std::fmt::Display;
use std::fmt::Formatter;
use std::ops::Index;

use serde::*;
use quicksilver::lifecycle::Window;
use quicksilver::lifecycle::Event;
use quicksilver::input::Key;
use quicksilver::input::GamepadButton;
use quicksilver::input::GamepadAxis;

#[derive(Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum BindPoint {
    Left,
    Right,
    SoftDrop,
    HardDrop,
    RotateCW,
    RotateCCW,
    Hold,
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Binding {
    KeyboardKey(u32),
    GamepadKey(u32),
    GamepadAxis(u32, bool),
}

#[derive(Clone, Copy, Serialize, Deserialize)]
struct ControlState {
    binding: Binding,
    pressed: bool,
    fired: bool,
    repeat: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ControlMap {
    controls: HashMap<BindPoint, ControlState>,
    repeat: f32,
}

impl Display for BindPoint {
    fn fmt(&self, f: &mut Formatter) -> ::std::fmt::Result {
        match self {
            &BindPoint::Left => write!(f, "Left"),
            &BindPoint::Right => write!(f, "Right"),
            &BindPoint::RotateCW => write!(f, "Rotate CW"),
            &BindPoint::RotateCCW => write!(f, "Rotate CCW"),
            &BindPoint::HardDrop => write!(f, "Hard Drop"),
            &BindPoint::SoftDrop => write!(f, "Soft Drop"),
            &BindPoint::Hold => write!(f, "Hold"),
        }
    }
}

impl Display for Binding {
    fn fmt(&self, f: &mut Formatter) -> ::std::fmt::Result {
        match self {
            &Binding::KeyboardKey(key) => {
                let key = unsafe {
                    std::mem::transmute::<u8, Key>(key as u8)
                };
                write!(f, "{:?}", key)
            },
            &Binding::GamepadAxis(axis, positive) => {
                let axis = unsafe {
                    std::mem::transmute::<u8, GamepadAxis>(axis as u8)
                };
                match (axis, positive) {
                    (GamepadAxis::LeftStickX, false) => write!(f, "L. Analog ←"),
                    (GamepadAxis::LeftStickX, true) => write!(f, "L. Analog →"),
                    (GamepadAxis::LeftStickY, false) => write!(f, "L. Analog ↑"),
                    (GamepadAxis::LeftStickY, true) => write!(f, "L. Analog ↓"),
                    (GamepadAxis::RightStickX, false) => write!(f, "R. Analog ←"),
                    (GamepadAxis::RightStickX, true) => write!(f, "R. Analog →"),
                    (GamepadAxis::RightStickY, false) => write!(f, "R. Analog ↑"),
                    (GamepadAxis::RightStickY, true) => write!(f, "R. Analog ↓"),
                }
            },
            &Binding::GamepadKey(key) => {
                let key = unsafe {
                    std::mem::transmute::<u32, GamepadButton>(key)
                };
                write!(f, "{:?}", key)
            },
        }
    }
}

impl ControlState {
    fn keyboard(key: u32) -> Self {
        Self {
            binding: Binding::KeyboardKey(key),
            pressed: false,
            fired: false,
            repeat: 0.0,
        }
    }
}

impl Default for ControlMap {
    fn default() -> Self {
        let mut controls = HashMap::new();
        controls.insert(BindPoint::Left,      ControlState::keyboard(Key::Left as u32));
        controls.insert(BindPoint::Right,     ControlState::keyboard(Key::Right as u32));
        controls.insert(BindPoint::SoftDrop,  ControlState::keyboard(Key::Down as u32));
        controls.insert(BindPoint::HardDrop,  ControlState::keyboard(Key::Up as u32));
        controls.insert(BindPoint::RotateCW,  ControlState::keyboard(Key::D as u32));
        controls.insert(BindPoint::RotateCCW, ControlState::keyboard(Key::A as u32));
        controls.insert(BindPoint::Hold,      ControlState::keyboard(Key::Space as u32));
        Self {
            controls,
            repeat: 0.07,
        }
    }
}

impl Index<BindPoint> for ControlMap {
    type Output = bool;

    fn index(&self, i: BindPoint) -> &bool {
        self.controls.get(&i).map(|s| &s.fired).unwrap()
    }
}

impl ControlMap {
    pub fn update(&mut self, window: &mut Window) {
        for (_, state) in self.controls.iter_mut() {
            let was_pressed = state.pressed;

            state.fired = false;
            state.repeat += window.update_rate() as f32 / 1000.0;

            match &state.binding {
                &Binding::KeyboardKey(key) => {
                    let key = unsafe {
                        std::mem::transmute::<u8, Key>(key as u8)
                    };
                    state.pressed = window.keyboard()[key].is_down();
                },
                &Binding::GamepadAxis(axis, positive) => {
                    let axis = unsafe {
                        std::mem::transmute::<u8, GamepadAxis>(axis as u8)
                    };
                    state.pressed = window.gamepads()
                        .first()
                        .map(|gamepad| {
                            if gamepad[axis] > 0.5 {
                                positive
                            } else if gamepad[axis] < -0.5 {
                                !positive
                            } else {
                                false
                            }
                        })
                        .unwrap_or(false);
                },
                &Binding::GamepadKey(key) => {
                    let key = unsafe {
                        std::mem::transmute::<u32, GamepadButton>(key)
                    };
                    state.pressed = window.gamepads()
                        .first()
                        .map(|gamepad| gamepad[key].is_down())
                        .unwrap_or(false);
                },
            }

            if !was_pressed && state.pressed {
                state.fired = true;
                state.repeat = -self.repeat * 2.0;
            }

            if state.pressed && state.repeat > self.repeat {
                state.repeat -= self.repeat;
                state.fired = true;
            }
        }
    }

    pub fn binding(&self, point: &BindPoint) -> Binding {
        self.controls.get(point).unwrap().binding
    }

    pub fn remap(&mut self, point: BindPoint, bind: Binding) {
        let point = self.controls.get_mut(&point).unwrap();
        point.binding = bind;
        point.pressed = false;
        point.fired = false;
        point.repeat = 0.0;
    }

    pub fn event_to_binding(event: Event) -> Option<Binding> {
        match event {
            Event::Key(key, _) => Some(Binding::KeyboardKey(key as u32)),
            Event::GamepadAxis(_, axis, amount) => if amount > 0.5 {
                Some(Binding::GamepadAxis(axis as u32, true))
            } else if amount < -0.5 {
                Some(Binding::GamepadAxis(axis as u32, false))
            } else {
                None
            },
            Event::GamepadButton(_, key, _) => Some(Binding::GamepadKey(key as u32)),
            _ => None,
        }
    }
}
