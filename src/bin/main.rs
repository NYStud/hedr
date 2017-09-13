extern crate hedr;

use std::io;
use std::env;
use std::os::unix::ffi::OsStrExt;
use std::error::Error;

use hedr::editor::Editor;
use hedr::file::File;

fn print_help() {
    print!(r"hedr [options] [FILE]...

options:
 -V  --version      show version information and exit
 -h  --help         show this help and exit
 -v                 view mode (read-only)
 FILE               file to edit or view
");
}

fn print_version() {
    print!(r"hedr, a tiny hex editor
Version 0.1
Copyright (C) 2017 Ricardo R. Massaro
Source code: https://github.com/ricardo-massaro/hedr
");
}

fn parse_cmdline(editor : &mut Editor) -> bool  {
    let mut args = env::args_os();
    
    let progname = args.next();
    if progname.is_none() {
        return true;
    }
    let progname = match progname.unwrap().into_string() {
        Ok(s) => s,
        Err(_) => {
            print!("Error: can't convert program name to UTF-8\n");
            return false;
        }
    };
    
    while let Some(arg) = args.next() {
        if arg.len() > 0 && arg.as_os_str().as_bytes()[0] == b'-' {
            match arg.into_string() {
                Ok(s) => match s.as_str() {
                    "-h" | "--help" => {
                        print_help();
                        return false;
                    }

                    "-V" | "--version" => {
                        print_version();
                        return false;
                    }
                    
                    "-v" => editor.read_only = true,

                    _ => {
                        print!("{}: unknown option: '{}'\n", progname, s);
                        return false;
                    }
                },
                Err(arg) => {
                    print!("{}: can't convert argument to UTF-8: {:?}\n", progname, arg);
                    return false;
                }
            }
        } else {
            match File::new_from_file(arg) {
                Ok(file) => editor.add_file(file),
                Err((filename, e)) => {
                    print!("{}: error reading file {:?}: {}\n", progname, filename, e.description());
                    return false;
                }
            }
        }
    }
    true
}

fn main() {
    let stdin = io::stdin();
    let mut editor = Editor::new(stdin.lock());

    if ! parse_cmdline(&mut editor) {
        return;
    }
    if let Err(e) = editor.run() {
        println!("ERROR: {}", e);
    }
}
