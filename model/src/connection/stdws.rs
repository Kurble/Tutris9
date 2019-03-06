use super::*;
use std::rc::Rc;
use std::cell::RefCell;
use stdweb::web::WebSocket;
use stdweb::web::event::{SocketCloseEvent, SocketErrorEvent, SocketMessageEvent };
use stdweb::traits::*;

struct Inner {
    socket: WebSocket,
    messages: Vec<String>,
    alive: bool,
}

pub struct WsConnection {
    inner: Rc<RefCell<Inner>>,
}

impl WsConnection {
    pub fn new(uri: &str) -> Self {
        let ws = WebSocket::new(uri).unwrap();
        let inner = Rc::new(RefCell::new(Inner {
            socket: ws,
            messages: Vec::new(),
            alive: true,
        }));

        let i = inner.clone();
        inner.borrow().socket.add_event_listener(move |_: SocketErrorEvent| {
            i.borrow_mut().alive = false;
        });

        let i = inner.clone();
        inner.borrow().socket.add_event_listener(move |_: SocketCloseEvent| {
            i.borrow_mut().alive = false;
        });

        let i = inner.clone();
        inner.borrow().socket.add_event_listener(move |event: SocketMessageEvent| {
            let text = event.data().into_text().unwrap();
            i.borrow_mut().messages.push(text);
        });

        WsConnection { inner }
    }
}

impl Connection for WsConnection {
    fn close(&mut self) {
        self.inner.borrow().socket.close();
    }

    fn alive(&self) -> bool {
        self.inner.borrow().alive
    }

    fn send(&mut self, message: &str) {
        let ok = self.inner.borrow().socket.send_text(message).is_ok();
        self.inner.borrow_mut().alive &= ok;
    }

    fn message(&mut self) -> Option<String> {
        let m = &mut self.inner.borrow_mut().messages;
        if m.len() > 0 {
            Some(m.remove(0))
        } else {
            None
        }
    }
}
