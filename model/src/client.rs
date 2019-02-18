use std::ops::Deref;
use std::net::*;
use std::io::Write;
use serde::*;
use mirror::*;

pub struct Client<T: for<'a> Reflect<'a>> {
    value: T,
    connection: TcpStream,
    buffer: Vec<u8>,
}

impl<T: for<'a> Reflect<'a>> Deref for Client<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T: for<'a> Reflect<'a>> Client<T> {
    pub fn new(value: T, server: &str, query: &str) -> ::std::io::Result<Self> {
        let mut stream = TcpStream::connect(server)?;
        stream.write_all(query.as_bytes())?;

        // todo: check if access is granted

        Ok(Self {
            value,
            connection: stream,
            buffer: Vec::new(),
        })
    }

    pub fn update(&mut self) {
        //
    }

    pub fn command(&self, cmd: &str) {
        // todo
    }
}
