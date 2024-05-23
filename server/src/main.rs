// Server (distributes tasks and client updates)

extern crate cluster;
#[macro_use] extern crate rocket;

mod web;
mod db;

use std::collections::HashMap;
use std::fs::File;
use std::io::{prelude::*, BufReader, BufWriter, Error, ErrorKind};
use std::net::{TcpStream};
use std::sync::{Arc, RwLock, mpsc::{channel, Receiver, Sender}};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use cluster::ioutils::{start_listener, send_u64, send_data, recieve_data, send_data_buffered, get_bytes_of, get_hash_of};
use cluster::filelib::{Task, TaskOutput, FileError};
use cluster::netfaces::{ClientState, ClientMessage};
use web::start_web_server;

use ciborium::{ser, de, from_reader};
use queues::{Buffer, IsQueue};


#[cfg(debug_assertions)]
const CLIENT_PATH: &str = "../target/debug/client.exe";
#[cfg(not(debug_assertions))]
const CLIENT_PATH: &str = "./client";

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


fn greet_client(stream: &mut TcpStream) ->Result<bool, ServerError> {
    stream.write_all(b"master")?;
    
    let mut buf = [0u8; 6];
    stream.read_exact(&mut buf)?;
    
    match &buf {
        b"search" => Ok(false),
        b"normal" => Ok(true),
        _ => Err(ServerError::ProtocolError("Wrong greeting".to_owned())),
    }
}

fn update_client(mut reader: &mut BufReader<&TcpStream>, writer: &mut BufWriter<&TcpStream>) -> Result<ClientState, ServerError> {
    // First client message
    match from_reader(&mut reader).unwrap() {
        ClientMessage::SkipUpdate => return Ok(ClientState::Ready),
        ClientMessage::DoUpdate => {
            send_u64(writer, get_hash_of(CLIENT_PATH)?)?; // Send client hash // NOTE: can be cached
        }
        _ => Err(ServerError::ProtocolError("Wrong update scheme".to_owned()))?
    };
    
    // Second client message
    match from_reader(&mut reader).unwrap() {
        ClientMessage::HashMatched => Ok(ClientState::Ready),  // Client is up to date
        ClientMessage::HashMismatched => {
            let mut payload = get_bytes_of(CLIENT_PATH)?;
            send_data_buffered(writer, &mut payload.0, payload.1)?;
            
            Ok(ClientState::Updating) // Client disconnects
        },
        _ => Err(ServerError::ProtocolError("Wrong hash comparison".to_owned()))?
    }
}


fn thread_collector(rx: Receiver<Option<JoinHandle<Result<(), ServerError>>>>){
    let mut handles = vec![];
    let mut running = true;

    while running {
        // NOTE: magic number
        if let Ok(msg) = rx.recv_timeout(Duration::from_millis(500)) {
            if let Some(handle) = msg {
                handles.push(handle);
            } else {
                running = false;
                // check handles for the last time before exiting scope and dropping handles
                // manage live threads in manager somehow
            }
        }

        for i in (0..handles.len()).rev() {
            if handles[i].is_finished() {
                if let Err(e) = handles.swap_remove(i).join().unwrap() {
                    println!("Client thread finished with error: {:?}", e);
                } // TODO: check behaviour on receivig SIG_INT from OS
            }
        }
    }
}


#[derive(Debug)]
pub struct Worker {
    id: u8,
    is_free: bool,
}

impl Worker {
    fn new(id: u8) -> Worker {
        Worker {id, is_free: true}
    }
}

pub enum ManagerEvent {
    NewTask(Task),
    NewWorker(Sender<Task>), // worker_tx
    WorkerMessage(u8, TaskOutput), // worker_id, output
    Stop,
}

//              Event_recv
fn task_manager(rx: Receiver<ManagerEvent>, workers_lock: Arc<RwLock<HashMap<u8, Worker>>>) {
    let mut pending: Buffer<Task> = Buffer::new(64);
    let mut max_id = 0;
    let mut senders: HashMap<u8, Sender<Task>> = HashMap::new();
    
    // wait for next ManagerEvent
    for event in rx {
        match event {
            ManagerEvent::NewTask(task) => {
                // Push new task to queue
                println!("[mng] Recieved task (id = {})", &task.id);
                if pending.size() < pending.capacity() {
                    pending.add(task).unwrap();
                } else {
                    // TODO: Cancel task
                }
            }
            
            ManagerEvent::NewWorker(tx) => {
                // Construct new worker
                max_id += 1;
                let new_worker = Worker::new(max_id);
                
                // Send init task to worker with worker id
                let init_task = Task {
                    id: max_id.into(), 
                    shell: String::new(), 
                    attachment: None, 
                };
                tx.send(init_task).unwrap();

                // Add new worker to workers
                workers_lock.write().unwrap().insert(max_id, new_worker);
                senders.insert(max_id, tx);
            }

            ManagerEvent::WorkerMessage(id, output) => {
                // TODO: implement
                // add result for success and fail
                // success -> finish task; set worker free
                // fail -> retry task?
                let stdout = String::from_utf8(output.stdout).unwrap();
                let stderr = String::from_utf8(output.stderr).unwrap();
                
                println!("[mng] Completed task");
                println!("[mng] id:\t{}", output.task_id);
                println!("[mng] code:\t{:?}", output.code);
                println!("[mng] stdout:\t{}", stdout);
                println!("[mng] stderr:\t{}", stderr);
                
                workers_lock.write().unwrap().get_mut(&id).unwrap().is_free = true;
            }

            ManagerEvent::Stop => {
                todo!();
                // TODO: implement
                // break?
            }
        };

        
        // Assign a task to each worker
        let mut workers = workers_lock.write().unwrap();
        let mut dead_ids: Vec<u8> = vec![];

        for worker in workers.values_mut() {
            if pending.size() == 0 {
                // Task queue is empty
                break
            }

            let id = worker.id;

            if worker.is_free {
                let task = pending.remove().expect("Failed to pop task from queue");
                
                if let Err(e) = senders[&id].send(task) {
                    // Worker thread is dead
                    pending.add(e.0).unwrap(); // Push back task
                    dead_ids.push(id); // Add worker to removal list
                } else {
                    worker.is_free = false;
                    // NOTE: add Task to Worker struct?
                    // change task status in db?
                }
            }
        }


        for id in &dead_ids {
            senders.remove(id); // Remove worker tx from senders
            workers.remove(id); // Remove worker from workers
        }
        dead_ids.clear();
    }
}

fn offer_tasks(rx: Receiver<Task>, reader: &mut BufReader<&TcpStream>, writer: &mut BufWriter<&TcpStream>, mng_tx: Sender<ManagerEvent>) -> Result<(), Error>{
    
    let id: u8 = rx.recv().unwrap().id.try_into().unwrap(); // Init task that contains worker id (guarantied to fit in u8)

    for task in &rx{
        // TODO: send task back to manager on fail

        println!("[wt_{}] Recieved task (id = {})", id, task.id);

        let mut serialized_task = vec![];
        ser::into_writer(&task, &mut serialized_task).unwrap();  // TODO: handle result
        send_data(writer, &serialized_task)?;                     // FIXME: try to ser directly into writer w/o send_data

        if let Some(at) = &task.attachment {
            let file = File::open(format!("./tasks/{}/{}", task.id, at.filename))?;
            let mut file_reader = BufReader::new(file);

            send_data_buffered(writer, &mut file_reader, at.size)?;
        }

        println!("[wt_{}] Sent task (id = {}) to worker", id, task.id);

        let serialized = recieve_data(reader)?;                // FIXME: try to de directly from reader w/o recieve_data
        let output: TaskOutput = de::from_reader(serialized.as_slice()).expect("Failed to deserialize result");
        mng_tx.send(ManagerEvent::WorkerMessage(id, output)).expect("Manager died");
    }
    
    Ok(())
}


fn main() -> Result<(), ServerError> {

    //let save_data = load_save_data()?;
    let listener = start_listener("0.0.0.0:1337")?;

    let workers: Arc<RwLock<HashMap<u8, Worker>>> = Arc::new(RwLock::new(HashMap::new()));

    let (mng_tx, mng_rx) = channel();
    
    let manager = thread::spawn({
            let workers = workers.clone();
            move || task_manager(mng_rx, workers)
    });

    let web_server = thread::spawn({
        let mng_tx = mng_tx.clone();
        move || start_web_server(mng_tx, workers).unwrap()
    });
    
    // NOTE: Check if I still need it when working out graceful shutdown
    let (col_tx, col_rx) = channel();
    let collector = thread::spawn(
        move || thread_collector(col_rx)
    );

    
    for connection in listener.incoming() {
        // NOTE: Can create too many threads. Maybe try switching to thread pools or async?
        let (tx, rx) = channel::<Task>();
        let mng_tx = mng_tx.clone();

        let handle = thread::spawn(|| {
            let mut stream = connection?;
            
            match greet_client(&mut stream)? {
                false => return Ok(()), // It's a search, end serving early
                true => (),             // Go on
            };

            let mut reader = BufReader::new(&stream);
            let mut writer = BufWriter::new(&stream);

            println!("Serving {}", stream.peer_addr().unwrap());

            match update_client(&mut reader, &mut writer)? {
                ClientState::Ready => {
                    mng_tx.send(ManagerEvent::NewWorker(tx)).expect("Manager is dead");
                    
                    // TODO: handle error properly
                    if let Err(e) = offer_tasks(rx, &mut reader, &mut writer, mng_tx) {
                        match e.kind() {
                            ErrorKind::UnexpectedEof => {},
                            _ => {},
                        }
                    }
                },
                ClientState::Updating => (),
            }

            Ok(())
        });
        
        col_tx.send(Some(handle)).expect("Collector is dead");  // Tell thread collector about new worker
    }

    // NOTE: in what order should I join threads?
    col_tx.send(None).expect("Collector is dead");
    mng_tx.send(ManagerEvent::Stop).unwrap();
    // stop web server?
    collector.join().unwrap();
    manager.join().unwrap();
    web_server.join().unwrap();

    Ok(())
}