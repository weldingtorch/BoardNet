// Client task manager/executer 

use std::thread::{self, JoinHandle};
use std::process::{Command, Output, ExitCode};
use std::net::Shutdown;
use std::fs::File;

mod netlib;
use netlib::{Error, TcpStream, connect_to, send_u64, recieve_u64, recieve_data, send_data};
mod filelib;
use filelib::{Write, BufReader, BufWriter, SaveData, FileError, FileType, get_hash_of, load_save_data, save_save_data};

const CLIENT_PATH: &str = "./target/debug/client.exe";
const NEW_CLIENT_PATH: &str = "./target/debug/new_client.exe";

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
    let l_hash = get_hash_of(CLIENT_PATH)?;
    send_u64(writer, 2)?;  // Ask for remote client hash
    let r_hash = recieve_u64(reader)?;
    if l_hash == r_hash {
        return Ok(false);  // Client is up to date
    }

    let file = File::options().create(true).truncate(true).write(true).open(dbg!(NEW_CLIENT_PATH))?;
    let mut file = BufWriter::new(file);
    
    send_u64(writer, 3)?;  // Ask for remote client file
    recieve_data(reader, &mut file)?;
    
    if get_hash_of(NEW_CLIENT_PATH)? != r_hash {  // Bad practice, should rewrite get_hash_of
        panic!("Not implemented error for hash mismatch, failed to update");
    }
    
    Ok(true)
}

fn run_task() -> JoinHandle<Result<Output, ClientError>> {
    // TODO: execute shell file
    thread::spawn(|| {
        let output = Command::new("cmd")  // sh for linux
            .args(["/C", "python ./task/main.py"])
            .output()?;
        println!("Task executed with {}", output.status);
        Ok(output)
    })
}

fn main() -> ExitCode {
    println!("I am client!");
    //let mut save_data = load_save_data()?;  // used to store masterIp, publicKey, privateKey, ...

    // next line panics if no server
    let stream = connect_to("127.0.0.1:1337").unwrap();  // will change to bruteforcing master ip
    let mut reader = BufReader::new(&stream);
    let mut writer = BufWriter::new(&stream);
    
    match update(&mut reader, &mut writer) {
        Ok(is_ready) => {
            if is_ready {
                println!("Asking shell to update client");
                return ExitCode::from(2);  // Try to update
            } else {
                println!("Client is up to date!");
            }
        },
        Err(e) => {
            println!("Failed to download update"); // not enough permissions, network or io error 
            println!("{:?}", e);
            return ExitCode::from(1);
        }
    };

    // Template client loop
    let recv_buf: Vec<u8> = vec![];
    let mut recv_writer = BufWriter::new(recv_buf.clone());
    loop {
        recieve_data(&mut reader, &mut recv_writer).unwrap();
        println!("{:?}", recv_buf);
    };
    
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