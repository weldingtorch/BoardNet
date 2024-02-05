// Client (executes tasks)

extern crate cluster;

use std::io::{prelude::*, BufReader, BufWriter, Error};
use std::thread;
use std::process::{Command, Output, ExitCode};
use std::net::Shutdown;
use std::fs::File;

use cluster::ioutils::{TcpStream, connect_to, send_u64, recieve_u64, recieve_data, send_data, recieve_data_buffered, get_hash_of};
use cluster::filelib::{FileError, Task, TaskOutput, AttachmentType};

use ciborium::{ser, de};


const CLIENT_PATH: &str = "../target/debug/client.exe";
const NEW_CLIENT_PATH: &str = "../target/debug/new_client.exe";

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


fn update(reader: &mut BufReader<&TcpStream>, writer: &mut BufWriter<&TcpStream>) -> Result<bool, ClientError> {
    let l_hash = get_hash_of(CLIENT_PATH)?;
    send_u64(writer, 2)?;  // Ask for remote client hash
    let r_hash = recieve_u64(reader)?;
    if l_hash == r_hash {
        return Ok(false);  // Client is up to date
    }

    let file = File::options().create(true).truncate(true).write(true).open(NEW_CLIENT_PATH)?;
    let mut file = BufWriter::new(file);
    
    send_u64(writer, 2)?;  // Ask for remote client file
    recieve_data_buffered(reader, &mut file)?;
    
    if get_hash_of(NEW_CLIENT_PATH)? != r_hash {  // Bad practice, should rewrite get_hash_of
        panic!("Not implemented error for hash mismatch, failed to update");
    }
    
    Ok(true)
}

fn run_task(cwd: String, timeout: u16) -> Result<Output, ClientError> {
    let handle = thread::spawn(move || {
        let output = Command::new("cmd")  // sh -c for unux
            .args(["/c", &format!("task.bat")]).current_dir(cwd)
            .output()?;
        // TODO: timeout execution using task specified timeout value 
        println!("Task executed with {}", output.status);
        Ok(output)
    });

    println!("Spawned exec thread");
    let output = handle.join().unwrap();
    output
}

fn main() -> ExitCode {
    println!("Client started");
    //let mut save_data = load_save_data()?;  // used to store masterIp, publicKey, privateKey, ...

    // next line panics if no server
    let stream = connect_to("127.0.0.1:1337").unwrap();  // TODO: change to bruteforcing master ip
    let mut stream_reader = BufReader::new(&stream);
    let mut stream_writer = BufWriter::new(&stream);
    
    match update(&mut stream_reader, &mut stream_writer) {
        Ok(is_ready) => {
            if is_ready {
                println!("Asking shell to update client");
                return ExitCode::from(2);  // Try to update
            } else {
                println!("Client is up to date");
            }
        },
        Err(e) => {
            println!("Failed to download update"); // Not enough permissions, network or io error 
            println!("{:?}", e);
            return ExitCode::from(1);
        }
    };

    send_u64(&mut stream_writer, 1).unwrap(); // Tell master we are ready for working 
    println!("Told master client is ready");
    
    loop {
        // Recieve new task
        println!("Waiting to recieve new task");
        
        let serialized_task = recieve_data(&mut stream_reader).unwrap();
        let task: Task = de::from_reader(serialized_task.as_slice()).unwrap();  // NOTE: try to catch this error
        
        println!("Recieved new task: {:?}", task);
        
        let task_cwd = format!("./{}", task.id);
        let shell_path = format!("{}/task.bat", &task_cwd);
        let mut clean_up = true;
        
        // Save shell to file
        std::fs::create_dir(&task_cwd).unwrap();
        let mut shell_file = File::create(&shell_path).unwrap(); //.sh for unix
        shell_file.write_all(task.shell.as_bytes()).unwrap();

        // Save attachment to file if there is any
        if let Some(attachment) = &task.attachment {
            clean_up = !attachment.retain_attachment;
            let path = format!("{}/{}", &task_cwd, &attachment.filename);
            let file = File::create(path).unwrap();
            let mut file_writer = BufWriter::new(file);
            
            recieve_data_buffered(&mut stream_reader, &mut file_writer).unwrap();

            match attachment.attachment_type {
                AttachmentType::Raw => (),
                AttachmentType::TarArchive => todo!("unpack"),
            }
        }

        // Execute new task
        let output = run_task(task_cwd.clone(), task.timeout).unwrap();  // TODO: handle result (send it to server?)
        let task_output = TaskOutput {
            task_id: task.id,
            code: output.status.code(),
            stdout: output.stdout, 
            stderr: output.stderr
        };
        
        // Send execution result
        let mut serialized_output = vec![];
        ser::into_writer(&task_output, &mut serialized_output).unwrap();
        send_data(&mut stream_writer, &serialized_output).unwrap();

        // Clean up fs if required 
        std::fs::remove_file(&shell_path).unwrap();

        if clean_up {
            std::fs::remove_dir_all(task_cwd).unwrap();
        }
    };

    stream.shutdown(Shutdown::Both).expect("Failed to close connection to remote");
    
    println!("Everything is done!");

}