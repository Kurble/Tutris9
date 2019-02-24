use std::ops::Deref;
use std::net::*;
use serde::*;
use mirror::*;
use tetris_model::connection::Connection;

pub struct User<T: for<'a> Reflect<'a> + Serialize> {
    value: T,
    connection: Connection,
}

pub struct UserServer<T: for<'a> Reflect<'a> + Serialize> {
    factory: Box<Fn() -> T>,
    listener: TcpListener,
    connections: Vec<User<T>>,
}

impl<T: for<'a> Reflect<'a> + Serialize> Deref for User<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T: for<'a> Reflect<'a> + Serialize> UserServer<T> {
    pub fn new<F: 'static + Fn()->T>(factory: F, server: &str) -> ::std::io::Result<Self> {
        let listener = TcpListener::bind(server)?;

        listener.set_nonblocking(true)?;

        Ok(Self {
            factory: Box::new(factory),
            listener,
            connections: Vec::new(),
        })
    }

    pub fn update(&mut self) {
        if let Ok((stream, address)) = self.listener.accept() {
            let value = (self.factory)();
            let mut connection = Connection::new(stream).unwrap();

            println!("User connected on {}", address);

            // send over the base value to the use as part of the protocol
            connection.send(serde_json::to_string_pretty(&value).unwrap().as_str());

            self.connections.push(User { value, connection });
        }

        for user in self.connections.iter_mut() {
            let mut kill = false;
            for message in user.connection.messages() {
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
    }

    pub fn users(&mut self) -> impl Iterator<Item = &mut User<T>> {
        self.connections.iter_mut()
    }
}

impl<T: for<'a> Reflect<'a> + Serialize> User<T> {
    pub fn command(&mut self, command: &str) {
        self.value.command_str(command).unwrap();
        self.connection.send(command);
    }

    pub fn kick(&mut self) {
        self.connection.close();
    }
}
