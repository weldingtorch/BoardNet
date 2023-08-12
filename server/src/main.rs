// Server (task/client distributor)

use std::thread;

mod netlib;
use netlib::{Error, TcpStream, start_listener, recieve_data, send_u64, send_data};
mod filelib;
use filelib::{BufReader, BufWriter,SaveData, FileError, FileType, get_bytes_of, get_hash_of, load_save_data};


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

fn serve_session(stream: TcpStream,  save_data: SaveData) -> Result<(), ServerError> {
    let mut reader = BufReader::new(&stream);
    let mut writer = BufWriter::new(&stream);

    let serving = true;
    while serving {
        let rx = recieve_data(&mut reader)?;
        if rx.len() != 1 {
            Err(ServerError::ProtocolError(
                "Client violated protocol".to_owned(),
            ))?;
        }
        let opcode = rx[0];
        
        if 1 != 0 {
            println!("got cmd from a {:?}: {:?}", stream.peer_addr() , opcode); //dbg
        }
        // match opcode {
        //     0 => send_u64(&mut writer, get_hash_of(FileType::Task, &save_data)?)?,
        //     1 => send_data(&mut writer, &get_bytes_of(&save_data.task_path)?)?,
        //     2 => send_u64(&mut writer, get_hash_of(FileType::Client, &save_data)?)?,
        //     3 => send_data(&mut writer, &get_bytes_of(&save_data.client_path)?)?,
        //     4 => println!("stdout:{:?}", String::from_utf8(recieve_data(&mut reader)?.to_ascii_lowercase()).unwrap()),
        //     5 => println!("stderr:{:?}", String::from_utf8(recieve_data(&mut reader)?.to_ascii_lowercase()).unwrap()),
        //     _ => {
        //         Err(ServerError::ProtocolError(
        //             "Client violated protocol".to_owned(),
        //         ))?
        //     }
        //};
    }

    //let task_file = File::open("./task/main.py").expect("Failed to open task file")
    //let task_buffer = task_file.read(&mut buffer).unwrap();
    //stream.write(task_file);
    //println!("Succesfully established connection: {stream:?}");
    Ok(())
}

fn main() -> Result<(), ServerError> {
    let save_data = load_save_data()?;
    let mut handles = vec!();
    let listener = start_listener("127.0.0.1:1337")?;
    let running = true;
    

    while running {
        let connection = listener.accept();
        let thread_save_data = save_data.clone();
        handles.push(thread::spawn(|| {
            let (stream, addr) = connection?;
            println!("Serving {addr}");
            serve_session(stream, thread_save_data)
        }));
    }

    for handle in handles.into_iter() {
        if !handle.is_finished() {
            handle.join().unwrap().unwrap(); // TODO thread communication on completion to get latest savedata
        }
    }

    Ok(())
}