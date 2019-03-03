use std::ops::{Deref, DerefMut};
use std::thread::sleep;
use std::time::Duration;
use std::sync::mpsc::Receiver;
use serde::*;
use mirror::*;
use tetris_model::connection::*;

pub struct SharedServer<T: for<'a> Reflect<'a> + Serialize, C: Connection> {
    value: T,
    listener: Receiver<C>,
    connections: Vec<C>,
}

impl<T: for<'a> Reflect<'a> + Serialize, C: Connection> Deref for SharedServer<T, C> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T: for<'a> Reflect<'a> + Serialize, C: Connection> DerefMut for SharedServer<T, C> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<T: for<'a> Reflect<'a> + Serialize, C: Connection> SharedServer<T, C> {
    pub fn new(value: T, listener: Receiver<C>) -> Self {
        Self {
            value,
            listener,
            connections: Vec::new(),
        }
    }

    pub fn update(&mut self) {
        for mut connection in self.listener.try_iter() {
            connection.send(serde_json::to_string(&self.value).unwrap().as_str());

            self.connections.push(connection);
        }

        for connection in self.connections.iter_mut() {
            let mut kill = false;
            for message in Messages::new(connection) {
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

        sleep(Duration::from_millis(20));
    }

    pub fn command(&mut self, cmd: &str) {
        self.value
            .command_str(cmd)
            .expect(format!("unable to execute broadcast command: {}", cmd).as_str());

        for connection in self.connections.iter_mut() {
            connection.send(cmd);
        }
    }

    pub fn connections(&self) -> usize {
        self.connections.len()
    }
}

