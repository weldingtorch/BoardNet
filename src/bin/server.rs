// Server (task/client distributor)

use std::{thread::{self, JoinHandle}, sync::mpsc::{channel, Receiver}, time::Duration};

mod netlib;
use netlib::{Error, TcpStream, start_listener, send_u64, recieve_u64, send_data, recieve_data};
mod filelib;
use filelib::{BufReader, BufWriter,SaveData, FileError, get_hash_of, load_save_data};


#[derive(Debug)]
enum ServerError {
    FileError(FileError),
    NetError(Error),
    ProtocolError(String),
}

impl From<FileError> for ServerError {
    fn from(err: FileError) -> Self {
        ServerError::FileError(err)
    }
}

impl From<Error> for ServerError {
    fn from(err: Error) -> Self {
        ServerError::NetError(err)
    }
}

//fn serve_session(stream: TcpStream,  save_data: SaveData) -> Result<(), ServerError> {
fn serve_session(stream: TcpStream) -> Result<(), ServerError> {
    let mut reader = BufReader::new(&stream);
    let mut writer = BufWriter::new(&stream);
    
    let serving = true;
    println!("Serving {}", stream.peer_addr().unwrap());

    while serving {
        let rx = recieve_u64(&mut reader)?;

        println!("got cmd from a [{:?}]: {:?}", stream.peer_addr().unwrap() , rx); //dbg
        
        match rx {
            //1 => send_data(&mut writer, &get_bytes_of(&save_data.task_path)?)?,
            2 => send_u64(&mut writer, get_hash_of("./target/debug/client.exe")?)?,
            //3 => send_data(&mut writer, &get_bytes_of(&save_data.client_path)?)?,
            //4 => println!("stdout:{:?}", String::from_utf8(recieve_data(&mut reader)?.to_ascii_lowercase()).unwrap()),
            //5 => println!("stderr:{:?}", String::from_utf8(recieve_data(&mut reader)?.to_ascii_lowercase()).unwrap()),
            _ => {
                Err(ServerError::ProtocolError(
                    "Client violated protocol".to_owned(),
                ))?
            }
        };
    }

    Ok(())
}

fn thread_collector(rx: Receiver<JoinHandle<Result<(), ServerError>>>) -> Result<(), ServerError>{
    let mut handles = vec![];
    while !handles.is_empty() {
        if let Ok(handle) = rx.recv_timeout(Duration::from_millis(500)) {
            handles.push(handle);
        }

        for i in handles.len()..0 {
            if handles[i-1].is_finished() {
                handles.swap_remove(i-1).join().unwrap()?; // TODO thread communication on completion to get latest savedata
            }
        }
    }
    Ok(())
}

fn main() -> Result<(), ServerError> {
    //let save_data = load_save_data()?;
    let listener = start_listener("127.0.0.1:1337")?;

    let (tx, rx) = channel();
    let collector = thread::spawn( || {
        thread_collector(rx).unwrap();
    });

    for stream in listener.incoming(){
        //let thread_save_data = save_data.clone();
        tx.send(
            thread::spawn(|| {
                serve_session(stream?)
                //serve_session(stream?, thread_save_data)
            })
        ).unwrap();
    }

    collector.join().unwrap();   

    Ok(())
}