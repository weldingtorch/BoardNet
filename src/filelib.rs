// filelib (defines common structs)
// TODO: rename this module

use std::io::Error;
use std::fmt::Display;
use std::num::ParseIntError;

use serde::{Serialize, Deserialize};


#[derive(Debug)]
pub enum FileError {
    IO(Error),
    ParseInt(ParseIntError),
    //UnknownTaskType(String),
}

impl From<Error> for FileError {
    fn from(err: Error) -> Self {
        FileError::IO(err)
    }
}

impl From<ParseIntError> for FileError {
    fn from(err: ParseIntError) -> Self {
        FileError::ParseInt(err)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: u32,
    pub shell: String,
    pub attachment: Option<Attachment>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub size: u64,
    pub attachment_type: AttachmentType,
    pub retain_attachment: bool,
    pub filename: String,
    // NOTE: file itself is stored in fs under task id filename
    // ./attachments/{task.id} 
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub enum AttachmentType {
    #[default] Raw,
    TarArchive,
    // May add other comperssion types
}

impl Display for AttachmentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self{
            AttachmentType::Raw => "Raw",
            AttachmentType::TarArchive => "TarArchive",
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct TaskOutput {
    pub task_id: u32,
    pub code: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}


/*
#[derive(Debug, Clone, Copy)]
pub struct CachedData {
    pub client_hash: u64,
    pub public_key: u64,
    pub private_key: u64,
}

impl CachedData {
    fn set_client_hash(&mut self, new_client_hash: u64) {
        self.client_hash = new_client_hash;
    }
    fn set_public_key(&mut self, new_public_key: u64) {
        self.public_key = new_public_key;
    }
    fn set_private_key(&mut self, new_private_key: u64) {
        self.private_key = new_private_key;
    }
}

impl Default for CachedData {
    fn default() -> Self {
        Self {
            client_hash: Default::default(),
            public_key: Default::default(),
            private_key: Default::default(), 
        }
    }
}
*/



/*
#[derive(Debug, Clone)]
pub struct SaveData {
    pub task_path: String,
    pub task_type: TaskType,
    pub client_path: String,
}
 
impl SaveData {
    pub fn set_task_type(&mut self, new_task_type: TaskType) {
        self.task_type = new_task_type;
    }
    pub fn set_path(&mut self, ft: FileType, new_path: String) {
        match ft {
            FileType::Client => self.client_path = new_path,
            FileType::Task => self.task_path = new_path,
        };
    }
}

impl Default for SaveData {
    fn default() -> Self {
        Self {
            task_path: String::new(), 
            task_type: Default::default(), 
            client_path: String::from("./target/debug/client.exe"),  
        }
    }
}

impl From<HashMap<String, String>> for SaveData {
    fn from(hm: HashMap<String, String>) -> Self {
        let mut savedata = SaveData::default();
        
        for (k, v) in hm.iter() {
            match k.as_str() {
                "task_path" => savedata.task_path = v.clone(),
                "task_type" => savedata.task_type = match v.as_str() {
                    "py" => TaskType::Python,
                    "exe" => TaskType::Executable,
                    "tar" => TaskType::Archive,
                    e => Err(FileError::UnknownTaskType(format!(
                                   "Unknown task type \"{}\"",
                                   e
                                ))).unwrap(),
                },
                "client_path" => savedata.client_path = v.clone(),
                _ => continue,
            };
        }
        
        savedata
    }
}

impl From<&SaveData> for String {
    fn from (sd: &SaveData) -> Self {
        let mut str = String::new();
        str.push_str(&format!("task_path = {}\n", sd.task_path));
        str.push_str(&format!("task_type = {}\n", sd.task_type));
        str.push_str(&format!("client_path = {}\n", sd.client_path));
        str
    }
}

pub fn load_save_data() -> Result<SaveData, FileError> {
    let save_file =  match File::open("./save.dat") {
        Ok(file) => file,
        Err(_e) => {
            let def_save_data = SaveData::default();
            save_save_data(&def_save_data)?;
            return Ok(def_save_data);
        },
    };
    let reader = BufReader::new(save_file);
    let mut raw_save_data = HashMap::new();
    
    for rawline in reader.lines() {
        let line = rawline?;
        let key_value: Vec<&str> = line.split('=').collect();
        if key_value.len() != 2 {
            Err(Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid save.dat syntax",
            ))?
        }
        raw_save_data.insert(key_value[0].trim().to_owned(), key_value[1].trim().to_owned());
    }
    
    Ok(SaveData::from(raw_save_data))
}

pub fn save_save_data(save_data: &SaveData) -> Result<(), FileError> {
    let file = match File::options().write(true).open("./save.dat") {
        Ok(f) => f,
        Err(_e) => File::create("./save.dat")?,
    };
    let mut writer = BufWriter::new(file);
    writer.write_all(String::from(save_data).as_bytes())?;
    writer.flush()?;
    Ok(())
}
*/

