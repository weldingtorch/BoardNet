// Client launcher (updates and runs client)

use std::fs::rename;
use std::io::Error;
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
    println!("Started launcher");
    //println!("cwd: {:?}", current_dir().unwrap());
    loop {
        println!("Starting client");
        
        let mut client_child = Command::new(CLIENT_PATH).spawn().expect("Failed to start client");
        let status = client_child.wait().unwrap();
        
        println!("Client exited");
        println!("Cleint status: {}", status);
        
        if let Some(code) = status.code() {
            match code {
                1 => sleep(Duration::from_secs(5)),  // If errored restart with timeout (change to 5m)
                2 => update().unwrap(),  // Try to update
                _ => (),  // restart     // If updater can't rename a file but client was able to 
            };                           // overwrite old new_client, that's a problem
        } else {
            println!("Oh no! Client process was killed! (Got signal)");  // So drammatic
        };
    }
}

