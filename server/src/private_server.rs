use std::ops::Deref;
use std::thread::sleep;
use std::time::Duration;
use std::sync::mpsc::Receiver;
use serde::*;
use mirror::*;
use tetris_model::connection::*;

pub struct User<T: for<'a> Reflect<'a> + Serialize, C: Connection> {
    value: T,
    connection: C,
}

pub struct PrivateServer<T: for<'a> Reflect<'a> + Serialize, C: Connection> {
    factory: Box<Fn() -> T>,
    listener: Receiver<C>,
    connections: Vec<User<T, C>>,
}

impl<T: for<'a> Reflect<'a> + Serialize, C: Connection> Deref for User<T, C> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T: for<'a> Reflect<'a> + Serialize, C: Connection> PrivateServer<T, C> {
    pub fn new<F: 'static + Fn()->T>(factory: F, listener: Receiver<C>) -> Self {
        Self {
            factory: Box::new(factory),
            listener,
            connections: Vec::new(),
        }
    }

    pub fn update(&mut self) {
        for mut connection in self.listener.try_iter() {
            let value = (self.factory)();

            // send over the base value to the use as part of the protocol
            connection.send(serde_json::to_string(&value).unwrap().as_str());

            self.connections.push(User { value, connection });
        }

        for user in self.connections.iter_mut() {
            let mut kill = false;
            for message in Messages::new(&mut user.connection) {
                if user.value.command_str(message.as_str()).is_err() {
                    kill = true;
                } else {
                    // idk
                }
            }

            if kill {
                user.connection.close();
            }
        }

        self.connections.retain(|user| user.connection.alive());

        sleep(Duration::from_millis(100));
    }

    pub fn users(&mut self) -> impl Iterator<Item = &mut User<T, C>> {
        self.connections.iter_mut()
    }
}

impl<T: for<'a> Reflect<'a> + Serialize, C: Connection> User<T, C> {
    pub fn command(&mut self, command: &str) {
        if let Ok(_) = self.value.command_str(command) {
            self.connection.send(command);
        } else {
            println!("Unable to execute private command: {}", command);
            self.kick();
        }
    }

    pub fn kick(&mut self) {
        self.connection.close();
    }
}
