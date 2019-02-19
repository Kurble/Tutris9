use std::ops::Deref;
use std::net::*;
use std::io::{Read, Write};
use std::iter::repeat;
use serde::*;
use mirror::*;
use tetris_model::connection::Connection;

pub struct InstanceServer<T: for<'a> Reflect<'a> + Serialize> {
    value: T,
    listener: TcpListener,
    connections: Vec<Connection>,
}

impl<T: for<'a> Reflect<'a> + Serialize> Deref for InstanceServer<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T: for<'a> Reflect<'a> + Serialize> InstanceServer<T> {
    pub fn new(value: T, server: &str) -> ::std::io::Result<Self> {
        let mut listener = TcpListener::bind(server)?;

        Ok(Self {
            value,
            listener,
            connections: Vec::new(),
        })
    }

    pub fn update(&mut self) {
        if let Ok((mut stream, _)) = self.listener.accept() {
            let mut connection = Connection::new(stream).unwrap();

            connection.send(serde_json::to_string_pretty(&self.value).unwrap().as_str());

            self.connections.push(connection);
        }

        for connection in self.connections.iter_mut() {
            let mut kill = false;
            for message in connection.messages() {
                if self.value.command_str(message.as_str()).is_err() {
                    kill = true;
                } else {
                    // idk
                }
            }

            if kill {
                connection.close();
            }
        }

        self.connections.retain(|connection| connection.alive());
    }

    pub fn command(&self, cmd: &str) {
        // todo
    }
}

