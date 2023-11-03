extern crate fxhash;

pub use fxhash::hash64;
use std::collections::hash_map::{DefaultHasher, HashMap};
//use std::default;
pub use std::io::{Read, Write, BufRead, BufReader, BufWriter, Error};
use std::fs::File;
use std::fmt::Display;
use std::num::ParseIntError;


#[derive(Debug)]
pub enum FileError {
    IO(Error),
    ParseInt(ParseIntError),
    UnknownTaskType(String),
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

#[derive(Debug, Clone)]
pub struct Task {
    pub id: u32,
    pub shell: String,
    pub attachment: Option<Attachment>,
    
}

impl Task {
    pub fn as_bytes(&self) -> Box<[u8]> {
        let id_bytes = self.id.to_be_bytes();
        let shell_bytes = self.shell.as_bytes();
        let attachment_bytes = self.attachment.as_ref().unwrap().as_bytes(); // NOTE: total garbage, use serde
        [id_bytes.as_slice(), shell_bytes, &attachment_bytes].concat().into_boxed_slice()
    }
}

#[derive(Debug, Clone)]
pub struct Attachment {
    attachment_type: AttachmentType,
    filename: String,
    file: String,
}

impl Attachment {
    fn as_bytes(&self) -> Box<[u8]> {
        let filename_bytes = self.filename.as_bytes();
        let file_bytes = self.file.as_bytes();
        let attachment_type_bytes = (self.attachment_type as u8).to_be_bytes();
        [filename_bytes, file_bytes, attachment_type_bytes.as_slice()].concat().into_boxed_slice()
    }
}

#[derive(Default, Debug, Clone, Copy)]
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
pub fn get_bytes_of(path: &str) -> Result<(BufReader<File>, u64), FileError>{
    let file = File::open(path)?;
    let length = file.metadata()?.len();
    Ok((BufReader::new(file), length))
}

fn get_unbuffered_bytes_of(path: &str) -> Result<Box<[u8]>, FileError> {
    let mut reader = get_bytes_of(path)?.0;
    let mut data = vec![];
    reader.read_to_end(&mut data)?;
    Ok(data.into_boxed_slice())
}

pub fn get_hash_of(path: &str/*, cached_data: &mut CachedData*/) -> Result<u64, FileError> {
    //if cached_data.client_hash != 0u64 {
    //    cached_data.client_hash
    //} else {
        let client_hash = hash64(&get_unbuffered_bytes_of(path)?);
        //cached_data.client_hash = client_hash;
        //client_hash 
    //}
    Ok(client_hash)
}
