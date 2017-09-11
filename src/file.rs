
use std::io;
use std::io::Read;
use std::fs;
use std::ffi::{OsString, OsStr};

pub struct File {
    pub data : Vec<u8>,
    pub filename : Option<String>,
    pub modified : bool,
    pub cursor_pos : usize,
    pub top_line : usize,
}

impl File {

    pub fn new() -> File {
        File {
            data : vec![],
            filename : None,
            modified : false,
            cursor_pos : 0,
            top_line : 0,
        }
    }

    pub fn new_from_file(filename : OsString) -> io::Result<File> {
        let file = File {
            data : read_file(&filename)?,
            filename : Some(filename.to_string_lossy().into_owned()),
            modified : false,
            cursor_pos : 0,
            top_line : 0,
        };
        Ok(file)
    }
    
}

fn read_file(filename : &OsStr) -> io::Result<Vec<u8>> {
    let mut file = fs::File::open(filename)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    Ok(data)
}
