extern crate hedr;

use hedr::editor::Editor;

fn main() {
    let mut editor = Editor::new();
    if let Err(e) = editor.run() {
        println!("ERROR: {}", e);
    }
}
