mod game;
mod matchmaking;
mod instance;

use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, SyncSender, sync_channel, TryRecvError};
use std::str::FromStr;
use std::fs::metadata;

use actix::*;
use actix_web::server::HttpServer;
use actix_web::{ws, http, App, Error, HttpRequest, HttpResponse};
use actix_web::fs::NamedFile;

use clap::App as ClapApp;
use clap::Arg;

use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

#[derive(Message)]
struct WsMessage(pub String);

#[derive(Message)]
struct WsClose;

struct WsServerState {
    instances: Arc<Mutex<instance::InstanceContainer<WsConnection>>>,
}

struct Ws { id: usize, addr: String, tx: Option<SyncSender<String>> }

struct WsConnection {
    rx: Receiver<String>,
    addr: Addr<Ws>,
    alive: bool,
}

impl Actor for Ws {
    type Context = ws::WebsocketContext<Self, WsServerState>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let (tx, rx) = sync_channel(8);
        let addr = ctx.address();
        if ctx.state().instances.lock().unwrap().submit(self.id, WsConnection {
            rx,
            addr,
            alive: true,
        }).is_err() {
            println!("Unable to forward {} to instance {}", self.addr, self.id);
            ctx.stop();
        }

        self.tx = Some(tx);
    }
}

impl Handler<WsMessage> for Ws {
    type Result = ();

    fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

impl Handler<WsClose> for Ws {
    type Result = ();

    fn handle(&mut self, _: WsClose, ctx: &mut Self::Context) {
        ctx.close(None);
        ctx.stop();
    }
}

impl StreamHandler<ws::Message, ws::ProtocolError> for Ws {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        match msg {
            ws::Message::Ping(msg) => {
                ctx.pong(&msg);
            },
            ws::Message::Pong(_) => {
                //
            },
            ws::Message::Text(text) => {
                self.tx.as_ref().unwrap().send(text).unwrap();
            },
            ws::Message::Binary(_) |
            ws::Message::Close(_) => {
                ctx.stop();
            },
        }
    }

    fn error(&mut self, _err: ws::ProtocolError, _ctx: &mut Self::Context) -> Running {
        println!("Client {} disconnected unexpectedly", self.addr);
        Running::Stop
    }
}

impl mirror::Remote for WsConnection {
    fn close(&mut self) {
        if self.alive {
            self.addr.do_send(WsClose);
        }
        self.alive = false;
    }

    fn alive(&self) -> bool {
        self.alive
    }

    fn send(&mut self, message: &str) -> Result<(), mirror::Error> {
        Ok(self.addr.do_send(WsMessage(message.to_string())))
    }

    fn recv(&mut self) -> Option<String> {
        match self.rx.try_recv() {
            Ok(result) => Some(result),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                self.alive = false;
                None
            },
        }
    }
}

fn instance_route(req: &HttpRequest<WsServerState>) -> Result<HttpResponse, Error> {
    if let Ok(id) = req.path().split_at("/instance/".len()).1.parse::<usize>() {
        let addr = req.connection_info().remote().unwrap_or("<unknown>").to_string();
        return ws::start(req, Ws { id, addr, tx: None });
    } else {
        Ok(HttpResponse::BadRequest().finish())
    }
}

fn main() {
    let matches = ClapApp::new("tutris-server")
        .arg(Arg::with_name("bind-to")
            .short("b")
            .long("bind-to")
            .help("The address to bind the sever to")
            .default_value("127.0.0.1:3000")
            .takes_value(true)
            .required(true))
        .get_matches();

    let bind = matches.value_of("bind-to").unwrap_or("127.0.0.1:3000".into());

    println!("Tutris-9 server starting..");
    println!("Server will listen on {}", bind);

    let sys = actix::System::new("Tutris 9");

    let instances = Arc::new(Mutex::new(instance::InstanceContainer::new()));

    let use_ssl = metadata("key.pem").is_ok() && metadata("cert.pem").is_ok();

    instances.lock().unwrap().create(move |listener, container| {
        matchmaking::run_matchmaking_server(listener, container).expect("matchmaker failed");
    }, instances.clone());

    let server = HttpServer::new(move || {
        App::with_state(WsServerState { instances: instances.clone() })
            .handler("/static/", |req: &HttpRequest<WsServerState>| -> Result<NamedFile, Error> {
                let path = req.path().trim_start_matches("/");
                let file = if path.ends_with(".wasm") {
                    let mime = FromStr::from_str("application/wasm").unwrap();
                    NamedFile::open(path)
                        .map_err(|io| Error::from(io))
                        .map(|f| f.set_content_type(mime))
                } else {
                    NamedFile::open(path)
                        .map_err(|io| Error::from(io))
                };

                file
            })
            .resource("/", |r| r.method(http::Method::GET).f(|_| {
                HttpResponse::Found()
                    .header("LOCATION", "/static/index.html")
                    .finish()
            }))
            .resource("/instance/{id}", |r| r.f(instance_route))
    });

    if use_ssl {
        let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
        builder.set_private_key_file("key.pem", SslFiletype::PEM).unwrap();
        builder.set_certificate_chain_file("cert.pem").unwrap();
        server.bind_ssl(bind, builder).unwrap().start();
        println!("Server started using SSL");
    } else {
        server.bind(bind).unwrap().start();
        println!("SSL key and certificiate not found while looking for cert.pem and key.pem.");
        println!("Server started without SSL");
    }

    let _ = sys.run();
}
