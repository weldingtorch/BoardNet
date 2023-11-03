// Server (task/client distributor)

use std::{thread::{self, JoinHandle}, sync::{mpsc::{channel, Receiver, Sender}, Arc, RwLock}, time::Duration, collections::HashMap, io::Write};
use queues::{Buffer, IsQueue};

mod netlib;
use netlib::{Error, TcpStream, start_listener, send_u64, recieve_u64, send_data, recieve_data};
mod filelib;
use filelib::{BufReader, BufWriter, Task, FileError,  get_bytes_of, get_hash_of};

use crate::netlib::write_to_buf;


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

fn serve_session(rx: Receiver<Task>, stream: TcpStream, man_tx: Sender<ManagerEvent>) -> Result<(), ServerError> {
    let mut reader = BufReader::new(&stream);
    let mut writer = BufWriter::new(&stream);
    
    let serving = true;
    println!("Serving {}", stream.peer_addr().unwrap());

    // client-chosen communication mode
    let mode = recieve_u64(&mut reader)?;
    
    match mode {
        1 => {writer.write(&rx.recv().expect("Manager is dead").as_bytes())?;}, // ready for new task
        2 => send_u64(&mut writer, get_hash_of(CLIENT_PATH)?)?,  // launch update sequence // NOTE: can be cached
        /* what I expect to happen inside 2
        3 => {
            let mut payload = get_bytes_of(CLIENT_PATH)?;
            write_to_buf(&mut payload.0, &mut writer, payload.1)?}
            , // wtf?
        */
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

        for i in (0..handles.len()).rev() {
            if handles[i-1].is_finished() {
                if let Err(e) = handles.swap_remove(i-1).join().unwrap() {
                    println!("Client thread finished with error: {:?}", e);
                } // TODO: check behaviour on receivig SIG_INT from OS
            }
        }
    }
}


struct Worker {
    id: u8,
    is_free: bool,
}

impl Worker {
    fn new(id: u8) -> Worker {
        Worker {id, is_free: true}
    }
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
                    // TODO: Cancel task or push back to rx?
                }
            }
            
            ManagerEvent::NewWorker(tx) => {
                // Construct new worker and add it to workers
                max_id += 1;
                let new_worker = Worker::new(max_id);
                workers.write().unwrap().push(new_worker);
                senders.insert(max_id, tx);
                // Add new worker to workers
            }

            ManagerEvent::WorkerMessage() => {
                // TODO: implement
                // success -> finish task
                // fail -> retry task?
            }

            ManagerEvent::Stop => {
                // TODO: implement
                // break?
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
    // TODO: call a function from web/mod.rs to start web server
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
    
    // NOTE: Check if I still need it when working out graceful shutdown
    let (col_tx, col_rx) = channel();
    let collector = thread::spawn( || {
        thread_collector(col_rx);
    });

    
    for stream in listener.incoming(){
        // NOTE: Can create too many threads. Maybe try switching to thread pools or async?
        let (tx, rx) = channel::<Task>();
        let c_man_tx = man_tx.clone();

        let handle = thread::spawn(|| {
            serve_session(rx, stream?, c_man_tx)
        });
        
        col_tx.send(Some(handle)).expect("Collector is dead 💀");

        man_tx.send(ManagerEvent::NewWorker(tx)).expect("Manager is dead 💀");
    }

    col_tx.send(None).unwrap();
    collector.join().unwrap();   

    Ok(())
}