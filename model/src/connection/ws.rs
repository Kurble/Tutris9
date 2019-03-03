use super::*;

use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, channel, TryRecvError};
use std::thread::spawn;

use websocket::message::{Message, OwnedMessage};
use websocket::sync::Client;
use websocket::sync::stream::{Stream, Splittable};
use websocket::sync::sender::Writer;

pub struct WsConnection<S: Stream+Splittable+Send> {
    alive: bool,
    writer: Arc<Mutex<Writer<<S as Splittable>::Writer>>>,
    reader: Receiver<String>,
}

impl<S: Stream+Splittable+Send+'static> WsConnection<S> where
    <S as Splittable>::Reader: Send,
    <S as Splittable>::Writer: Send,
{
    pub fn new(client: Client<S>) -> Self {
        let (mut reader, writer) = client.split().unwrap();

        let writer = Arc::new(Mutex::new(writer));

        let (tx, rx) = channel();

        let result = Self {
            alive: true,
            writer: writer.clone(),
            reader: rx,
        };

        spawn(move || {
            for message in reader.incoming_messages() {
                match message {
                    Ok(OwnedMessage::Ping(data)) => {
                        if writer.lock()
                            .unwrap()
                            .send_message(&Message::pong(data))
                            .is_err() {
                            break;
                        }
                    },
                    Ok(OwnedMessage::Text(data)) => {
                        if tx.send(data).is_err() {
                            break;
                        }
                    },
                    _ => {
                        break;
                    }
                }
            }
        });

        result
    }
}

impl<S: Stream+Splittable+Send> Connection for WsConnection<S> {
    fn close(&mut self) {
        self.alive = false;
    }

    fn alive(&self) -> bool {
        self.alive
    }

    fn send(&mut self, message: &str) {
        self.alive &= self.writer
            .lock()
            .unwrap()
            .send_message(&Message::text(message.to_string()))
            .is_ok();
    }

    fn message(&mut self) -> Option<String> {
        match self.reader.try_recv() {
            Ok(message) => {
                Some(message)
            },
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                self.alive = false;
                None
            },
        }
    }
}