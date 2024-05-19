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
