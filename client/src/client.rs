use std::ops::{Deref, DerefMut};
use std::str::FromStr;
use mirror::*;
use serde_json::{Value, from_value};
use tetris_model::connection::*;

pub struct Client<T: for<'a> Reflect<'a>, C: Connection> {
    value: T,
    connection: C,
}

impl<T: for<'a> Reflect<'a>, C: Connection> Deref for Client<T, C> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T: for<'a> Reflect<'a>, C: Connection> DerefMut for Client<T, C> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<T: for<'a> Reflect<'a>, C: Connection> Client<T, C> {
    pub fn new(mut connection: C) -> ::std::io::Result<Self> {
        loop {
            if let Some(message) = connection.message() {
                let value: Value = Value::from_str(message.as_str()).unwrap();
                let value: T = from_value(value).unwrap();

                return Ok(Self {
                    value,
                    connection,
                })
            }

            // oh boy...
        }
    }

    pub fn alive(&self) -> bool {
        self.connection.alive()
    }

    pub fn update(&mut self) {
        if self.connection.alive() {
            for message in Messages::new(&mut self.connection) {
                self.value.command_str(message.as_str()).expect("Invalid message received");
            }
        }
    }

    pub fn command(&mut self, cmd: &str) {
        self.connection.send(cmd);
    }
}
