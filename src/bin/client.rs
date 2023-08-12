// Client task manager/executer 

use std::io::Read;
use std::ops::Deref;
use std::str::Bytes;
use std::thread::{self, JoinHandle};
use std::process::{Command, Output};
use std::net::Shutdown;
use std::fs::File;

mod netlib;
use netlib::{Error, TcpStream, connect_to, recieve_u64, recieve_data, send_data};
mod filelib;
use filelib::{Write, BufReader, BufWriter, SaveData, FileError, FileType, get_hash_of, load_save_data};

use crate::filelib::save_save_data;

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

fn update(reader:&mut BufReader<&TcpStream>, writer:&mut BufWriter<&TcpStream>, save_data:&mut SaveData) -> Result<(), ClientError> {
    let (code, path) = (2, &save_data.client_path);

    let l_hash = get_hash_of(save_data)?;
    send_data(writer, &[code])?;
    let r_hash = recieve_u64(reader)?;
    if l_hash == r_hash {
        return Ok(());
    }

    send_data(writer, &[code + 1])?;
    let new = recieve_data(reader)?;
    println!("{}", String::from_utf8(new.to_ascii_lowercase()).unwrap()); //dbg
    let mut file = match File::options().write(true).open(dbg!(&save_data.task_path)) { //save_data.get_path(ft)
        Ok(f) => f,
        Err(_e) => File::create(&save_data.task_path)?,
    };
    file.write_all(&new)?;
    file.flush()?;
    Ok(())
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

fn main() -> Result<(), ClientError> {
    let mut save_data = load_save_data()?;

    let stream = connect_to("127.0.0.1:1337")?;
    let mut reader = BufReader::new(&stream);
    let mut writer = BufWriter::new(&stream);
    
    //update(&mut reader, &mut writer, &mut save_data)?;

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
    save_save_data(&save_data)?;
    
    println!("Everything is done!");
    Ok(())
}