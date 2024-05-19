use std::collections::HashMap;
use std::fs::File;
use std::io::{prelude::*, BufReader, BufWriter, Error};
use std::net::{Ipv4Addr, AddrParseError};
use std::str::FromStr;


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




#[derive(Debug, Clone)]
pub struct SaveData {
    pub master_ip: Ipv4Addr,
}
 
impl SaveData {
    pub fn set_master_ip(&mut self, new_master_ip: Ipv4Addr) {
        self.master_ip = new_master_ip;
    }
}

impl Default for SaveData {
    fn default() -> Self {
        Self {
            master_ip: Ipv4Addr::UNSPECIFIED,  
        }
    }
}

impl TryFrom<HashMap<String, String>> for SaveData {
    type Error = AddrParseError;

    fn try_from(hm: HashMap<String, String>) -> Result<Self, AddrParseError> {
        let mut savedata = SaveData::default();
        
        for (k, v) in hm.iter() {
            match k.as_str() {
                "master_ip" => savedata.master_ip = Ipv4Addr::from_str(v)?,
                _ => continue,
            };
        }
        
        Ok(savedata)
    }
}

impl From<&SaveData> for String {
    fn from (sd: &SaveData) -> Self {
        let mut str = String::new();
        str.push_str(&format!("master_ip = {}\n", sd.master_ip));
        str
    }
}

pub fn load_save_data() -> Result<SaveData, Error> {
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
    
    match SaveData::try_from(raw_save_data) {
        Ok(data) => Ok(data),
        Err(e)=> Err(Error::new(std::io::ErrorKind::InvalidData, e))
    }
}

pub fn save_save_data(save_data: &SaveData) -> Result<(), Error> {
    let file = match File::options().write(true).open("./save.dat") {
        Ok(f) => f,
        Err(_e) => File::create("./save.dat")?,
    };
    let mut writer = BufWriter::new(file);
    writer.write_all(String::from(save_data).as_bytes())?;
    writer.flush()?;
    Ok(())
}
