// Client updater

use std::env::current_dir;
//use std::fs::rename;
//use std::io::Error;
//use std::process::Command;

/*
fn update() -> Result<(), Error> {
    println!("Trying to update client");
    rename("./new_client.exe", "./client.exe")?;
    Ok(())
}
*/
fn main() {
    println!("Started shell");
    println!("cwd: {:?}", current_dir().unwrap());
    loop {
        let mut code: i8 = -1;
        println!("Running app");
        code = 1; // 0 - ok; -1 - err; 1 - try update; 
        println!("App finished");
        //if let Err(error) = code {
        //    0 => Ok(()),
        //    1 => update(),
        //    _ => Err(())
        //} else {
        //    print()
        //};
    }
}

