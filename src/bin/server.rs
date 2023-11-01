// Server (task/client distributor)

use std::{thread::{self, JoinHandle}, sync::{mpsc::{channel, Receiver, Sender}, Arc, RwLock}, time::Duration, collections::HashMap};
use queues::{Buffer, IsQueue};

mod netlib;
use netlib::{Error, TcpStream, start_listener, send_u64, recieve_u64, send_data, recieve_data};
mod filelib;
use filelib::{BufReader, BufWriter, FileError, get_hash_of, Task};


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
fn serve_session(rx: Receiver<Task>, stream: TcpStream, man_tx: Sender<ManagerEvent>) -> Result<(), ServerError> {
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


struct Worker {
    is_free: bool,
    id: u8,
}


enum ManagerEvent {
    NewTask(Task),
    NewWorker(Sender<Task>), // thread_send
    WorkerMessage(),
    Stop,
}

//              Event_recv
fn task_manager(rx: Receiver<ManagerEvent>, workers: Arc<RwLock<Vec<Worker>>>) {
    //let running = true; // Arc<AtomicBool>
    let mut pending: Buffer<Task> = Buffer::new(64);
    let mut max_id = 0;
    let mut senders: HashMap<u8, Sender<Task>> = HashMap::new();
    
    for event in rx.recv() {
        
        match event {
            ManagerEvent::NewTask(task) => {
                // Push new task to queue
                if pending.size() < pending.capacity() {
                    pending.add(task).unwrap();
                } else {
                    // Cancel task or push back to rx?
                }
            }
            
            ManagerEvent::NewWorker(tx) => {
                // Construct new worker and add it to workers
                max_id += 1;
                let new_worker = Worker {
                    is_free: true,
                    id: max_id,
                };
                workers.write().unwrap().push(new_worker);
                senders.insert(max_id, tx);
                // Add new worker to workers
            }

            ManagerEvent::WorkerMessage() => {

            }

            ManagerEvent::Stop => {

            }
        };

        
        // Assign a task to each worker
        let mut w_workers = workers.write().unwrap();
        
        for i in w_workers.len()..0 {
            if pending.size() == 0 {
                // Task queue is empty
                break
            }

            if w_workers[i].is_free {
                let task = pending.remove().expect("Failed to pop task from queue");
                
                if let Err(e) = senders[&w_workers[i].id].send(task) {
                    pending.add(e.0).unwrap(); // push back task
                    w_workers.swap_remove(i);
                } // Exclude that worker from workers?
            }
        }
    }
    
    
}

fn web_server(workers: Arc<RwLock<Vec<Worker>>>, man_tx: Sender<ManagerEvent>) {
}


fn main() -> Result<(), ServerError> {
    //let save_data = load_save_data()?;
    let listener = start_listener("127.0.0.1:1337")?;

    let workers: Arc<RwLock<Vec<Worker>>> = Arc::new(RwLock::new(Vec::new()));

    let (man_tx, man_rx) = channel();
    
    let manager = {
        let workers = workers.clone();
        
        thread::spawn(|| {
            task_manager(man_rx, workers);
        })
    };

    //let (web_tx, web_rx) = channel();

    let web_server = {
        let c_man_tx = man_tx.clone();
        thread::spawn(move || {
            web_server(workers, c_man_tx);
        })
    };
    
    let (col_tx, col_rx) = channel();
    let collector = thread::spawn( || {
        thread_collector(col_rx);
    });

    
    for stream in listener.incoming(){
        //let thread_save_data = save_data.clone();
        let (tx, rx) = channel::<Task>();
        let c_man_tx = man_tx.clone();

        let handle = thread::spawn(|| {
            serve_session(rx, stream?, c_man_tx)
            //serve_session(stream?, thread_save_data)
        });
        
        col_tx.send(Some(handle)).expect("Collector is dead 💀");

        man_tx.send(ManagerEvent::NewWorker(tx)).expect("Manager is dead 💀");
    }

    col_tx.send(None).unwrap();
    collector.join().unwrap();   

    Ok(())
}