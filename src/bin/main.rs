extern crate hedr;

use std::io;
use hedr::editor::Editor;

fn main() {
    let stdin = io::stdin();
    let mut editor = Editor::new(stdin.lock());
    if let Err(e) = editor.run() {
        println!("ERROR: {}", e);
    }
}
