extern crate termios;

#[macro_use]
mod term;

use std::io;
use std::io::Write;
use term::*;

fn main() {
    let mut orig_term = setup_term(0);

    let stdout = io::stdout();
    print!("Press some keys, ^Q to exit...\r\n");
    stdout.lock().flush().unwrap();

    loop {
        match read_key() {
            Ok(key) => {
                if key == ctrl_key!('q') {
                    break
                };
                print!("{}\r\n", key)
            },
            Err(e) => {
                print!("ERROR: {}\r\n", e);
                break;
            }
        };
    };

    restore_term(0, &mut orig_term);
}
