use std::sync::mpsc::Receiver;
use std::time::Duration;
use std::thread::sleep;
use mirror::*;

pub fn run_game_server<R>(listener: Receiver<R>,
                          users: Vec<String>) -> Result<(), Error> where
    R: Remote
{
    let instance = tetris_model::instance::InstanceState::new(users);

    let mut server = SharedServer::new(instance, listener);

    println!("Game instance started");

    loop {
        server.update();
        server.local_command("call:server_update:")?;

        if server.done || (server.started && server.clients() == 0) {
            ::std::thread::sleep(Duration::from_secs(1));
            break;
        }

        sleep(Duration::from_millis(15));
    }

    println!("Game instance terminating");

    Ok(())
}