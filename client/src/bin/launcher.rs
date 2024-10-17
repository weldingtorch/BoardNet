// Client launcher (updates and runs client)

use std::fs::rename;
use std::io::Error;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;


const ERROR_EXITCODE: u8 = 1;
const UPDATE_EXITCODE: u8 = 2;

#[cfg(debug_assertions)]
const CLIENT_PATH: &str = "../target/debug/client.exe";
#[cfg(debug_assertions)]
const NEW_CLIENT_PATH: &str = "../target/debug/new_client.exe";
#[cfg(not(debug_assertions))]
const CLIENT_PATH: &str = "./client";
#[cfg(not(debug_assertions))]
const NEW_CLIENT_PATH: &str = "./new_client";


fn update() -> Result<(), Error> {
    println!("Trying to update client");
    rename(NEW_CLIENT_PATH, CLIENT_PATH)?;
    Command::new("chmod").args(["u+x", CLIENT_PATH]).status()?;
    Ok(())
}

fn main() {
    println!("Started launcher");
    
    loop {
        println!("Starting client");
        
        let mut client_child = Command::new(CLIENT_PATH).spawn().expect("Failed to start client");
        let status = client_child.wait().unwrap();
        
        println!("Client exited");
        println!("Cleint status: {}", status);
        
        if let Some(code) = status.code() {
            match code as u8 {  // Match only 8 last bits as Unix already truncates exit status
                0 => break,
                ERROR_EXITCODE => sleep(Duration::from_secs(5)),  // If errored restart with timeout (change to 5m)
                UPDATE_EXITCODE => update().unwrap(),  // Try to update
                _ => (),  // restart     // If updater can't rename a file but client was able to 
            };                           // overwrite old new_client, that's a problem
        } else {
            println!("Oh no! Client process was killed! (Got signal)");  // So drammatic
        };
    }
}

