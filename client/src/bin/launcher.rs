// Client launcher (updates and runs client)

use std::fs::rename;
use std::io::{Error, stdout, Write};
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;


const CLIENT_PATH: &str = "../target/debug/client.exe";
const NEW_CLIENT_PATH: &str = "../target/debug/new_client.exe";

fn update() -> Result<(), Error> {
    println!("Trying to update client");
    rename(NEW_CLIENT_PATH, CLIENT_PATH)?;
    Ok(())
}

fn main() {
    println!("Started shell");
    //println!("cwd: {:?}", current_dir().unwrap());
    loop {
        println!("Starting client");
        let client_process = Command::new(CLIENT_PATH).output().expect("Failed to start client");
        println!("Client exited");
        stdout().write_all(&client_process.stdout).expect("Failed to print stdout of client");
        let status = client_process.status;
        println!("Cleint status: {}", status);
        if let Some(code) = status.code() {
            match code {
                1 => sleep(Duration::from_secs(5)),  // If errored restart with timeout (change to 5m)
                2 => update().unwrap(),  // Try to update
                _ => (),  // restart     // If updater can't rename a file but client was able to 
            };                           // overwrite old new_client, that's a problem
        } else {
            println!("Oh no! Client process was killed! (Got signal)");
        };
    }
}

