
use std::io;
use std::io::Read;
use std::fs;
use std::ffi::OsString;

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

    pub fn new_from_file(filename : OsString) -> Result<File, (OsString, io::Error)> {
        let print_filename = filename.to_string_lossy().into_owned();
        let file = File {
            data : read_file(filename)?,
            filename : Some(print_filename),
            modified : false,
            cursor_pos : 0,
            top_line : 0,
        };
        Ok(file)
    }
    
}

fn read_file(filename : OsString) -> Result<Vec<u8>, (OsString, io::Error)> {
    let mut file = match fs::File::open(&filename) {
        Ok(f) => f,
        Err(e) => return Err((filename, e))
    };
    let mut data = Vec::new();
    match file.read_to_end(&mut data) {
        Ok(_) => (),
        Err(e) => return Err((filename, e))
    }
    Ok(data)
}
