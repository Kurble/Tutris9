use crate::instance::InstanceContainer;
use crate::game::run_game_server;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Receiver;
use std::time::Instant;
use rand::random;
use mirror::*;

struct Match {
    users: Vec<String>,
    wait_time: usize,
}

pub fn run_matchmaking_server<R>(listener: Receiver<R>,
                                 container: Arc<Mutex<InstanceContainer<R>>>) -> Result<(), Error>
    where
        R: Remote + Send + 'static
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

    let mut server = PrivateServer::new(factory, listener);
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

            current_match.users.retain(|client_key| {
                server.clients().find(|c| c.player_key.as_str() == client_key).is_some()
            });

            // try to fill the current match
            for client in server.clients() {
                if client.matched == false && current_match.users.len() < 9 {
                    let key = format!("{:x}-{:x}", random::<u64>(), random::<u64>());

                    client.command("matched/set:true")?;
                    client.command(format!("player_key/set:\"{}\"", key).as_str())?;

                    current_match.users.push(key);
                    current_match.wait_time = 10;
                }

                client.command(format!("wait_time/set:{}", current_match.wait_time).as_str())?;
                client.command(format!("players_found/set:{}", current_match.users.len()).as_str())?;
            }

            if current_match.wait_time == 0 {
                if current_match.users.len() < 2 {
                    current_match.wait_time = 91;
                } else {
                    let users = current_match.users.clone();

                    // make sure everyone has the correct player id
                    for client in server.clients() {
                        if let Some(id) = users.iter().enumerate()
                            .find(|(_, u)| u.as_str() == client.player_key)
                            .map(|(i, _)| i) {
                            client.command(format!("player_id/set:{}", id).as_str())?;
                        }
                    }

                    // create a new instance server to host the match
                    let c = container.clone();
                    let slot = container
                        .lock()
                        .map(move |mut i| {
                            let users = users;
                            i.create(move |listener, _| {
                                run_game_server(listener, users).expect("Game server failed");
                            }, c)
                        })
                        .expect("Failed to create game instance");

                    // report the existence of the new host to the users that should connect to it.
                    let commands = [
                        format!("instance_address/set:\"{}\"", slot),
                        format!("done/set:true"),
                    ];
                    for client in server.clients() {
                        if client.matched {
                            for command in commands.iter() {
                                client.command(command.as_str()).ok();
                            }
                            client.kick();
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