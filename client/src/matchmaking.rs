use super::*;
use crate::game::Game;
use crate::connection::make_connection;
use crate::controls::*;
use mirror::{Remote, Client};
use tetris_model::matchmaking::MatchmakingState;
use quicksilver::Future;

pub trait Matchmaking {
    fn update(&mut self);

    fn is_ok(&self) -> bool;

    fn status(&self) -> String;

    fn take(&mut self) -> Box<Future<Item=Box<Scene>, Error=quicksilver::Error>>;
}

pub enum MatchmakingImpl<R: Remote> {
    Connecting(ControlMap, Box<Future<Item=Client<MatchmakingState, R>, Error=mirror::Error>>),

    Waiting(ControlMap, Client<MatchmakingState, R>),

    Ok(Box<Future<Item=Box<Scene>, Error=quicksilver::Error>>),

    Error(mirror::Error),

    Poisoned,
}

impl<R: Remote + 'static> MatchmakingImpl<R> {
    pub fn new<F>(client: F, controls: ControlMap) -> Self where
        F: 'static + Future<Item=Client<MatchmakingState, R>, Error=mirror::Error>
    {
        MatchmakingImpl::Connecting(controls, Box::new(client))
    }
}

impl<R: Remote + 'static> Matchmaking for MatchmakingImpl<R> {
    fn update(&mut self) {
        let next = match replace(self, MatchmakingImpl::Poisoned) {
            MatchmakingImpl::Connecting(controls, mut future) => {
                match future.poll() {
                    Ok(Async::NotReady) => MatchmakingImpl::Connecting(controls, future),
                    Ok(Async::Ready(o)) => MatchmakingImpl::Waiting(controls, o),
                    Err(e) => MatchmakingImpl::Error(e),
                }
            },
            MatchmakingImpl::Waiting(controls, mut client) => {
                client.update();
                if client.done {
                    let address = format!("{}//{}/instance/{}", util::get_protocol(), util::get_host(),
                                          client.instance_address);
                    let game_client = Client::new(make_connection(address.as_str()));

                    MatchmakingImpl::Ok(Game::new(game_client,
                                                  client.player_id,
                                                  client.player_key.clone(),
                                                  controls))
                } else if !client.alive() {
                    MatchmakingImpl::Error(mirror::Error::ConnectionDropped)
                } else {
                    MatchmakingImpl::Waiting(controls, client)
                }
            },
            other => other,
        };
        replace(self, next);
    }

    fn is_ok(&self) -> bool {
        match self {
            &MatchmakingImpl::Ok(_) => true,
            &_ => false,
        }
    }

    fn status(&self) -> String {
        match self {
            &MatchmakingImpl::Connecting(_, _) => "Connecting...".to_string(),
            &MatchmakingImpl::Waiting(_, ref client) => format!("Matching... {}", client.wait_time),
            &MatchmakingImpl::Ok(_) => "Done!".to_string(),
            &MatchmakingImpl::Error(ref e) => format!("Error: {:?}", e),
            &MatchmakingImpl::Poisoned => panic!(),
        }
    }

    fn take(&mut self) -> Box<Future<Item=Box<Scene>, Error=quicksilver::Error>> {
        match replace(self, MatchmakingImpl::Poisoned) {
            MatchmakingImpl::Ok(result) => result,
            _ => panic!(),
        }
    }
}
