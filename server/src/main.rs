extern crate tetris_model;
extern crate mirror;
extern crate serde;
extern crate serde_json;
extern crate clap;

mod instance_server;
mod user_server;

use std::time::{Instant, Duration};
use std::process::Command;
use clap::{App, Arg};
use std::iter::repeat;
use std::mem::replace;

struct Match {
    users: Vec<String>,
    wait_time: usize,
}

fn run_matchmaking_server() -> ::std::io::Result<()> {
    println!("Starting matchmaking server.. ");

    let factory = || tetris_model::matchmaking::MatchmakingState {
        done: false,
        matched: false,
        player_key: String::new(),
        player_id: 0,
        players_found: 0,
        instance_address: String::new(),
        wait_time: 91,
    };

    let mut server = user_server::UserServer::new(factory, "0.0.0.0:1337")?;
    let mut last_match = Instant::now();

    let mut current_match = Match {
        users: Vec::new(),
        wait_time: 4,
    };

    println!("Startup complete, entering main loop");

    loop {
        server.update();

        let check = Instant::now();
        if check.duration_since(last_match).as_secs() >= 1 {
            last_match = check;

            current_match.wait_time -= 1;

            // try to fill the current match
            for user in server.users() {
                if user.matched == false && current_match.users.len() < 101 {
                    let key = format!("player_key_afcb8f7acaf7f6_{}", current_match.users.len());

                    user.command("matched/set:true");
                    user.command(format!("player_id/set:{}", current_match.users.len()).as_str());
                    user.command(format!("player_key/set:\"{}\"", key).as_str());

                    current_match.users.push(key);
                    current_match.wait_time = 10;
                }

                user.command(format!("wait_time/set:{}", current_match.wait_time).as_str());
                user.command(format!("players_found/set:{}", current_match.users.len()).as_str());
            }

            if current_match.wait_time == 0 {
                if current_match.users.len() < 2 {
                    println!("Not enough users to start instance.. resetting wait time");
                    current_match.wait_time = 91;
                } else {
                    let port = 1338;
                    let instance_address = format!("127.0.0.1:{}", port);

                    let mut args = Vec::new();
                    args.push(format!("--instance=0.0.0.0:{}", port));
                    for user in current_match.users {
                        args.push(format!("--user={}", user));
                    }

                    Command::new("tetris_server")
                        .args(args.iter())
                        .spawn()
                        .expect("failed to start instance. server is broken.");

                    let commands = [
                        format!("instance_address/set:\"{}\"", instance_address),
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

                    current_match = Match {
                        users: Vec::new(),
                        wait_time: 91,
                    };
                }
            }
        }
    }

    Ok(())
}

fn run_instance_server(address: String, users: Vec<String>) -> std::io::Result<()> {
    println!("instance started on {}", address);

    let instance = tetris_model::instance::InstanceState::new(users);

    let mut server = instance_server::InstanceServer::new(instance, address.as_str()).unwrap();
    loop {
        server.update();
        server.server_update();

        let commands = replace(&mut server.context.as_mut().unwrap().broadcast_commands,
                               Vec::new());

        for command in commands {
            server.command(command.as_str());
        }

        if server.done {
            break;
        }
    }

    println!("instance terminating on {}", address);
    Ok(())
}

fn main() {
    let matches = App::new("Tetris 99 clone server")
        .version("1.0")
        .author("Bram Buurlage. <brambuurlage@gmail.com>")
        .arg(Arg::with_name("instance")
            .short("i")
            .long("instance")
            .value_name("INSTANCE")
            .help("Runs an instance server on the specified address")
            .takes_value(true))
        .arg(Arg::with_name("user")
            .short("u")
            .long("user")
            .multiple(true)
            .takes_value(true)
            .number_of_values(1)
            .requires("instance")
            .help("Adds a user the instance should expect"))
        .get_matches();

    if matches.is_present("instance") {
        run_instance_server(matches.value_of("instance").unwrap().into(),
                            matches.values_of("user")
                                .unwrap()
                                .map(|u| String::from(u))
                                .collect())
            .expect("Error when running instance server.. exiting");
    } else {
        run_matchmaking_server()
            .expect("Error when running matchmaking server.. restarting");
    }
}
