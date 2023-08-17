// Client task manager/executer 

use std::io::Read;
use std::ops::Deref;
use std::str::Bytes;
use std::thread::{self, JoinHandle};
use std::process::{Command, Output, ExitCode};
use std::net::Shutdown;
use std::fs::File;

mod netlib;
use netlib::{Error, TcpStream, connect_to, recieve_u64, recieve_data, send_data};
mod filelib;
use filelib::{Write, BufReader, BufWriter, SaveData, FileError, FileType, get_hash_of, load_save_data, save_save_data};

#[derive(Debug)]
enum ClientError {
    FileError(FileError),
    NetError(Error),
}

impl From<FileError> for ClientError {
    fn from(err: FileError) -> Self {
        ClientError::FileError(err)
    }
}

impl From<Error> for ClientError {
    fn from(err: Error) -> Self {
        ClientError::NetError(err)
    }
}

fn update(reader:&mut BufReader<&TcpStream>, writer:&mut BufWriter<&TcpStream>) -> Result<bool, ClientError> {
    let l_hash = get_hash_of("./target/debug/client.exe")?;
    send_data(writer, &[2])?;
    let r_hash = recieve_u64(reader)?;
    if l_hash == r_hash {
        return Ok(false);
    }

    send_data(writer, &[3])?;
    let new = recieve_data(reader)?;
    println!("{}", String::from_utf8(new.to_ascii_lowercase()).unwrap()); //dbg
    let mut file = File::options().create(true).truncate(true).write(true).open(dbg!("./target/debug/new_client.exe"))?;
    file.write_all(&new)?;
    file.flush()?;
    Ok(true)
}

fn run_task() -> JoinHandle<Result<Output, ClientError>> {
    // TODO: add different ways of execution based on TaskType
    // like one-time/loop job or py/exe/shell
    thread::spawn(|| {
        let output = Command::new("cmd")
            .args(["/C", "python ./task/main.py"])
            .output()?;
        println!("Task executed with {}", output.status);
        Ok(output)
    })
}

fn main() -> ExitCode {
    println!("I am client!");
    //let mut save_data = load_save_data()?;  // used to store masterIp, publicKey, privateKey, ...

    // next line panics with exit code 101
    let stream = connect_to("127.0.0.1:1337").unwrap();  // will change to bruteforcing master ip
    let mut reader = BufReader::new(&stream);
    let mut writer = BufWriter::new(&stream);
    
    if let Ok(is_ready) = update(&mut reader, &mut writer) {
        if is_ready {
            println!("Asking shell to update client");
            return ExitCode::from(2);  // Try to update
        } else {
            println!("Client is up to date!");
        }
    } else {
        //return ExitCode::from(3) // ???? it would be nice to return err(), so that error is displayed in 
    }                            // stdout by eprintln! but on the other hand I still need to ask shell to update

    loop {
        if let Ok(recv) = recieve_data(&mut reader) {
            println!("{:?}", recv);
        } else {
            break;
        }
    }
    // let output = run_task().join().unwrap()?;
    // send_data(&mut writer, &[4])?;
    // send_data(&mut writer, &dbg!(output.stdout))?;
    // send_data(&mut writer, &[5])?;
    // send_data(&mut writer, &dbg!(output.stderr))?;

    stream.shutdown(Shutdown::Both).expect("Failed to close connection to remote");
    //save_save_data(&save_data)?;
    
    println!("Everything is done!");
    ExitCode::SUCCESS
}