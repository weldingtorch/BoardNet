// Server (task/client distributor)

use std::{thread::{self, JoinHandle}, sync::mpsc::{channel, Receiver, Sender}, time::Duration, fs::File};

mod netlib;
use netlib::{Error, TcpStream, start_listener, send_u64, recieve_u64, send_data, recieve_data};
mod filelib;
use filelib::{BufReader, BufWriter,SaveData, FileError, get_hash_of, load_save_data};


const CLIENT_PATH: &str = "./target/debug/client.exe";

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

    // one-time update sequence
    let rx = recieve_u64(&mut reader)?;
    
    match rx {
        //1 => send_data(&mut writer, &get_bytes_of(&save_data.task_path)?)?,
        2 => send_u64(&mut writer, get_hash_of(CLIENT_PATH)?)?,  // can be cached
        //3 => send_data(&mut writer, &get_bytes_of(&save_data.client_path)?)?,
        //4 => println!("stdout:{:?}", String::from_utf8(recieve_data(&mut reader)?.to_ascii_lowercase()).unwrap()),
        //5 => println!("stderr:{:?}", String::from_utf8(recieve_data(&mut reader)?.to_ascii_lowercase()).unwrap()),
        _ => {
            Err(ServerError::ProtocolError(
                "Client violated protocol".to_owned(),
            ))?
        }
    };

    // task loop
    while serving {
        
        //let task = rx.recv() // pipe from task manager
        //tell client to recieve task
        //send task
        //tell manager everything is ok or pass him error
        //await task completion  
        
        
        

        //println!("got cmd from a [{:?}]: {:?}", stream.peer_addr().unwrap() , rx); //dbg
        
        
    }

    Ok(())
}

fn thread_collector(rx: Receiver<Option<JoinHandle<Result<(), ServerError>>>>){
    let mut handles = vec![];
    let mut running = true;

    while running {
        if let Ok(msg) = rx.recv_timeout(Duration::from_millis(500)) {
            if let Some(handle) = msg {
                handles.push(handle);
            } else {
                running = false;
                // check handles for the last time before exiting scope and dropping handles
                // manage live threads in manager somehow
            }
        }

        for i in handles.len()..0 {
            if handles[i-1].is_finished() {
                if let Err(e) = handles.swap_remove(i-1).join().unwrap() {
                    println!("Client thread finished with error: {:?}", e);
                } // TODO: check behaviour on receivig SIG_INT from OS
            }
        }
    }
}

//                                 thread_recv    thread_send
fn task_manager(main_rx: Receiver<(Receiver<u64>, Sender<u64>)>, web_rx: Receiver<u64>) {

}

fn web_server(man_tx: Sender<u64>) {

}


fn main() -> Result<(), ServerError> {
    //let save_data = load_save_data()?;
    let listener = start_listener("127.0.0.1:1337")?;

    let (man_tx, man_rx) = channel();
    let (web_tx, web_rx) = channel();
    
    let manager = thread::spawn( || {
        task_manager(man_rx, web_rx);
    });
    
    let web_server = thread::spawn( || {
        web_server(web_tx);
    });
    
    let (col_tx, col_rx) = channel();
    let collector = thread::spawn( || {
        thread_collector(col_rx);
    });

    

    for stream in listener.incoming(){
        //let thread_save_data = save_data.clone();
        col_tx.send(
            Some(
                thread::spawn(|| {
                    serve_session(stream?)
                    //serve_session(stream?, thread_save_data)
                })
            )
        ).unwrap();
    }

    col_tx.send(None).unwrap();
    collector.join().unwrap();   

    Ok(())
}