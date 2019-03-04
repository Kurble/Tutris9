use std::ops::{Deref, DerefMut};
use std::str::FromStr;
use std::marker::PhantomData;
use mirror::*;
use serde_json::{Value, from_value};
use tetris_model::connection::*;
use futures::{Future, Poll, Async};

pub struct Client<T: for<'a> Reflect<'a>, C: Connection> {
    value: T,
    connection: C,
}

pub struct ConnectClient<T: for<'a> Reflect<'a>, C: Connection> {
    connection: Option<C>,
    ph: PhantomData<T>,
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

impl<T: for<'a> Reflect<'a>, C: Connection> Future for ConnectClient<T, C> {
    type Item = Client<T, C>;
    type Error = quicksilver::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Some(message) = self.connection.as_mut().unwrap().message() {
            let value: Value = Value::from_str(message.as_str()).unwrap();
            let value: T = from_value(value).unwrap();

            Ok(Async::Ready(Client {
                value,
                connection: self.connection.take().unwrap(),
            }))
        } else if self.connection.as_ref().unwrap().alive() {
            Ok(Async::NotReady)
        } else {
            Err(quicksilver::Error::IOError(::std::io::ErrorKind::ConnectionAborted.into()))
        }
    }
}

impl<T: for<'a> Reflect<'a>, C: Connection> Client<T, C> {
    pub fn new(connection: C) -> ConnectClient<T, C> {
        ConnectClient { connection: Some(connection), ph: PhantomData }
    }

    pub fn alive(&self) -> bool {
        self.connection.alive()
    }

    pub fn update(&mut self) {
        for message in Messages::new(&mut self.connection) {
            self.value.command_str(message.as_str()).expect("Invalid message received");
        }
    }

    pub fn command(&mut self, cmd: &str) {
        self.connection.send(cmd);
    }
}
