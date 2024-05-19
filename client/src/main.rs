// Client (executes tasks)

mod savedat;
extern crate cluster;

use std::fs::File;
use std::io::{prelude::*, BufReader, BufWriter, Error};
use std::net::{TcpStream, Shutdown};
use std::process::{Command, Output, ExitCode};

use savedat::{SaveData, load_save_data, save_save_data};
use cluster::ioutils::{connect_to, discover_master_ip, recieve_u64, recieve_data, send_data, recieve_data_buffered, get_hash_of};
use cluster::filelib::{FileError, Task, TaskOutput, AttachmentType};
use cluster::netfaces::{ClientState, ClientMessage};

use ciborium::{ser, de};


#[cfg(debug_assertions)]
const CLIENT_PATH: &str = "../target/debug/client.exe";
#[cfg(debug_assertions)]
const NEW_CLIENT_PATH: &str = "../target/debug/new_client.exe";

#[cfg(not(debug_assertions))]
const CLIENT_PATH: &str = "./client.exe";
#[cfg(not(debug_assertions))]
const NEW_CLIENT_PATH: &str = "./new_client.exe";

const ERROR_EXITCODE: u8 = 1;
const UPDATE_EXITCODE: u8 = 2;


#[derive(Debug)]
enum ClientError {
    FileError(FileError),
    NetError(Error),
    CBORError(ser::Error<Error>),
    UpdateError(String),
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

impl From<ser::Error<std::io::Error>> for ClientError {
    fn from(err: ser::Error<std::io::Error>) -> Self {
        ClientError::CBORError(err)
    }
}

fn greet_client() {
    // TODO: this
    todo!();
}

fn update(reader: &mut BufReader<&TcpStream>, mut writer: &mut BufWriter<&TcpStream>) -> Result<ClientState, ClientError> {
    let l_hash = get_hash_of(CLIENT_PATH)?;
    
    ser::into_writer(&ClientMessage::DoUpdate, &mut writer)?;  // Ask for latest client hash
    writer.flush()?;
    let r_hash = recieve_u64(reader)?;
    
    if l_hash == r_hash {
        ser::into_writer(&ClientMessage::HashMatched, &mut writer)?;
        writer.flush()?;
        return Ok(ClientState::Ready);  // Client is up to date
    }

    let file = File::options().create(true).truncate(true).write(true).open(NEW_CLIENT_PATH)?;
    let mut file = BufWriter::new(file);
    
    ser::into_writer(&ClientMessage::HashMismatched, &mut writer)?;  // Ask for latest client file
    writer.flush()?;

    recieve_data_buffered(reader, &mut file)?;
    
    if get_hash_of(NEW_CLIENT_PATH)? != r_hash {  // TODO: Bad practice, should rewrite get_hash_of
        return Err(ClientError::UpdateError("Failed to verify new client: hash mismatch.".to_owned()));
    }
    
    Ok(ClientState::Updating)
}

fn run_task(cwd: String) -> Result<Output, ClientError> {
    println!("\nTask execution.\n");
    
    let output = Command::new("cmd")  // sh -c for unix
        .args(["/c", &format!("task.bat")])
        .current_dir(cwd)
        //.spawn()?.stdout?.read;
        .output()?;
    
    //let output = child.wait_with_output()?;
    
    println!("End of task execution.\n");
    println!("Task executed with {}", output.status);
    Ok(output)
}

fn main() -> ExitCode {
    println!("Client started");

    let mut save_data = load_save_data().unwrap();  // used to store masterIp, clientId

    let mut stream = match connect_to((save_data.master_ip, 1337)) {
        Ok(s) => s,
        Err(e) => {
            println!("{:?}", e);
            save_data.set_master_ip(SaveData::default().master_ip);

            let addr = discover_master_ip().unwrap();
            save_data.set_master_ip(addr);
            save_save_data(&save_data).unwrap();
            
            connect_to((addr, 1337)).unwrap()
        }
    };

    let mut test_buf = [0u8; 6];
    stream.read_exact(&mut test_buf).unwrap();
    
    if &test_buf != b"master" {
        save_data.set_master_ip(SaveData::default().master_ip);
        save_save_data(&save_data).unwrap();
        panic!("Wrong greeting. Master ip has been reset. Rebooting.");
    } 

    stream.write_all(b"normal").unwrap();
    

    let mut stream_reader = BufReader::new(&stream);
    let mut stream_writer = BufWriter::new(&stream);

    match update(&mut stream_reader, &mut stream_writer) {
        Ok(state) => {
            match state {
                ClientState::Ready => {
                    println!("Client is up to date");
                }, 
                ClientState::Updating => {
                    println!("Asking shell to update client");
                    return ExitCode::from(UPDATE_EXITCODE);  // Tell launcher to update client
                },
            }
        },
        Err(e) => {
            println!("Failed to download update"); // Not enough permissions, network or io error 
            println!("{:?}", e);
            return ExitCode::from(ERROR_EXITCODE);  // Tell launcher about error
        }
    }
    
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
        let output = run_task(task_cwd.clone()).unwrap();  // TODO: handle result (send it to server?)
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