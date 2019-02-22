use std::ops::Deref;
use std::net::*;
use std::str::FromStr;
use mirror::*;
use serde_json::{Value, from_value};
use tetris_model::connection::Connection;

pub struct Client<T: for<'a> Reflect<'a>> {
    value: T,
    connection: Connection,
}

impl<T: for<'a> Reflect<'a>> Deref for Client<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T: for<'a> Reflect<'a>> Client<T> {
    pub fn new(server: &str) -> ::std::io::Result<Self> {
        let mut connection = Connection::new(TcpStream::connect(server)?)?;

        loop {
            if let Some(message) = connection.messages().next() {
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
            for message in self.connection.messages() {
                self.value.command_str(message.as_str()).expect("Invalid message received");
            }
        }
    }

    pub fn command(&mut self, cmd: &str) {
        self.connection.send(cmd);
    }
}
