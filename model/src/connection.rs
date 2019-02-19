use std::net::TcpStream;
use std::io::{Read, Write, ErrorKind};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::iter::repeat;

pub struct Connection {
    stream: TcpStream,
    buffer: Vec<u8>,
    buffer_written: usize,
    alive: bool,
}

pub struct Messages<'a>(&'a mut Connection);

impl Connection {
    pub fn new(stream: TcpStream) -> ::std::io::Result<Self> {
        stream.set_nonblocking(true)?;
        stream.set_nodelay(true)?;
        Ok(Self {
            stream,
            buffer: Vec::new(),
            buffer_written: 0,
            alive: true
        })
    }

    pub fn messages(&mut self) -> Messages {
        Messages(self)
    }

    pub fn close(&mut self) {
        self.alive = false;
    }

    pub fn alive(&self) -> bool {
        self.alive
    }

    pub fn send(&mut self, message: &str) -> ::std::io::Result<()> {
        let len = message.as_bytes().len() as u16 + 2;
        self.stream.write_u16::<BigEndian>(len)?;
        self.stream.write_all(message.as_bytes())?;
        self.stream.flush()?;
        Ok(())
    }

    fn parse_message(&mut self) -> ::std::io::Result<Option<String>> {
        if self.buffer_written >= 2 {
            let mut slice: &[u8] = self.buffer.as_slice();
            let len = slice.read_u16::<BigEndian>()? as usize;

            if self.buffer_written >= len {
                let mut message_bytes = Vec::new();
                message_bytes.resize(len - 2, 0);
                slice.read_exact(message_bytes.as_mut_slice())?;

                let message = String::from_utf8(message_bytes).unwrap();

                println!("Receive message: {}", message);

                self.buffer_written -= len;
                self.buffer.drain(..len).count();

                Ok(Some(message))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn read_message(&mut self) -> ::std::io::Result<Option<String>> {
        if let Some(msg) = self.parse_message()? {
            Ok(Some(msg))
        } else {
            // make sure there is enough room to read messages
            if self.buffer.len() - self.buffer_written < 1024 {
                self.buffer.extend(repeat(0).take(1024));
            }

            // read some data from the stream
            let slice = self.buffer.as_mut_slice();
            let written = self.stream.read(&mut slice[self.buffer_written..]);

            match written {
                Ok(0) => {
                    println!("Eof received");
                    self.alive = false;
                },

                Ok(written) => { self.buffer_written += written; },

                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    // nothing received. just wait!
                }

                Err(_) => {
                    println!("Error received");
                    // client died.
                    self.alive = false;
                },
            }

            if self.buffer_written > 2 {
                 self.parse_message()
            } else {
                Ok(None)
            }
        }
    }
}

impl<'a> Iterator for Messages<'a> {
    type Item = String;
    fn next(&mut self) -> Option<String> {
        let result = self.0.read_message();
        if let Ok(msg) = result {
            msg
        } else {
            self.0.alive = false;
            None
        }
    }
}