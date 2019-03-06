use tetris_model::connection::*;

mod shared_server;
mod private_server;
mod instance;

use self::instance::InstanceContainer;

use std::time::{Instant, Duration};
use std::mem::replace;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, SyncSender, sync_channel, TryRecvError};
use std::str::FromStr;

use actix::*;
use actix_web::server::HttpServer;
use actix_web::{ws, http, App, Error, HttpRequest, HttpResponse};
use actix_web::fs::NamedFile;

use rand::random;

use clap::App as ClapApp;
use clap::Arg;

struct Match {
    users: Vec<String>,
    wait_time: usize,
}

fn run_matchmaking_server<C>(listener: Receiver<C>,
                             container: Arc<Mutex<InstanceContainer<C>>>) -> ::std::io::Result<()>
    where
        C: Connection + Send + 'static
{
    let factory = || tetris_model::matchmaking::MatchmakingState {
        done: false,
        matched: false,
        player_key: String::new(),
        player_id: 0,
        players_found: 0,
        instance_address: String::new(),
        wait_time: 91,
    };

    let mut server = private_server::PrivateServer::new(factory, listener);
    let mut last_match = Instant::now();

    let mut current_match = Match {
        users: Vec::new(),
        wait_time: 4,
    };

    loop {
        server.update();

        let check = Instant::now();
        if check.duration_since(last_match).as_secs() >= 1 {
            last_match = check;

            current_match.wait_time -= 1;

            current_match.users.retain(|user_key| {
                server.users().find(|user| user.player_key.as_str() == user_key).is_some()
            });

            // try to fill the current match
            for user in server.users() {
                if user.matched == false && current_match.users.len() < 9 {
                    let key = format!("{:x}-{:x}", random::<u64>(), random::<u64>());

                    user.command("matched/set:true");
                    user.command(format!("player_key/set:\"{}\"", key).as_str());

                    current_match.users.push(key);
                    current_match.wait_time = 10;
                }

                user.command(format!("wait_time/set:{}", current_match.wait_time).as_str());
                user.command(format!("players_found/set:{}", current_match.users.len()).as_str());
            }

            if current_match.wait_time == 0 {
                if current_match.users.len() < 2 {
                    current_match.wait_time = 91;
                } else {
                    let users = current_match.users.clone();

                    // make sure everyone has the correct player id
                    for user in server.users() {
                        if let Some(id) = users.iter().enumerate()
                            .find(|(_, u)| u.as_str() == user.player_key)
                            .map(|(i, _)| i) {
                            user.command(format!("player_id/set:{}", id).as_str());
                        }
                    }

                    // create a new instance server to host the match
                    let c = container.clone();
                    let slot = container
                        .lock()
                        .map(move |mut i| {
                            let users = users;
                            i.create(move |listener, _| {
                                run_instance_server(listener, users).expect("matchmaker failed");
                            }, c)
                        })
                        .expect("failed to create game instance");

                    // report the existence of the new host to the users that should connect to it.
                    let commands = [
                        format!("instance_address/set:\"{}\"", slot),
                        format!("done/set:true"),
                    ];
                    for user in server.users() {
                        if user.matched {
                            for command in commands.iter() {
                                user.command(command.as_str());
                            }
                            user.kick();
                        }
                    }

                    // reset matchmaking
                    current_match = Match {
                        users: Vec::new(),
                        wait_time: 91,
                    };
                }
            }
        }
    }
}

fn run_instance_server<C>(listener: Receiver<C>,
                          users: Vec<String>) -> std::io::Result<()> where
    C: Connection
{
    let instance = tetris_model::instance::InstanceState::new(users);

    let mut server = shared_server::SharedServer::new(instance, listener);

    println!("instance started");
    loop {
        server.update();
        server.server_update();

        let commands = replace(&mut server.context.as_mut().unwrap().broadcast_commands,
                               Vec::new());

        for command in commands {
            server.command(command.as_str());
        }

        if server.done || (server.started && server.connections() == 0) {
            ::std::thread::sleep(Duration::from_secs(1));
            break;
        }
    }
    println!("instance terminating");

    Ok(())
}

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

impl Connection for WsConnection {
    fn close(&mut self) {
        if self.alive {
            self.addr.do_send(WsClose);
        }
        self.alive = false;
    }

    fn alive(&self) -> bool {
        self.alive
    }

    fn send(&mut self, message: &str) {
        self.addr.do_send(WsMessage(message.to_string()));
    }

    fn message(&mut self) -> Option<String> {
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

    instances.lock().unwrap().create(|listener, container| {
        run_matchmaking_server(listener, container).expect("matchmaker failed");
    }, instances.clone());

    HttpServer::new(move || {
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
    }).bind(bind).unwrap().start();

    println!("Server systems started");

    let _ = sys.run();
}
