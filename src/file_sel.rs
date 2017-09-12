
use std::fs;
use std::io;
use std::path::PathBuf;
use std::cmp::Ordering;
use std::ffi::{OsString, OsStr};

use super::screen::*;
use super::term::*;
use super::editor;
use super::editor::Editor;

#[derive(Eq)]
struct FileInfo {
    pub name : OsString,
    pub is_dir : bool,
}

impl Ord for FileInfo {

    fn cmp(&self, other: &FileInfo) -> Ordering {
        if self.is_dir && ! other.is_dir {
            Ordering::Less
        } else if ! self.is_dir && other.is_dir {
            Ordering::Greater
        } else {
            self.name.cmp(&other.name)
        }
    }
    
}

impl PartialOrd for FileInfo {

    fn partial_cmp(&self, other : &FileInfo) -> Option<Ordering> {
        Some(self.cmp(other))
    }
    
}

impl PartialEq for FileInfo {

    fn eq(&self, other : &FileInfo) -> bool {
        self.name.eq(&other.name)
    }
    
}


pub struct FileSel<'a, 'b : 'a> {
    editor : &'a mut Editor<'b>,
    quit : bool,
    selected_filename : Option<OsString>,
    cur_dir : Option<PathBuf>,
    files : Vec<FileInfo>,
    sel_index : usize,
    top_index : usize,
}

impl<'a, 'b> FileSel<'a, 'b> {

    pub fn new(editor : &'a mut Editor<'b>) -> FileSel<'a, 'b> {
        FileSel {
            editor : editor,
            quit : false,
            selected_filename : None,
            cur_dir : None,
            files : vec![],
            sel_index : 0,
            top_index : 0,
        }
    }
    
    fn draw_header(&mut self) {
        self.editor.screen.move_cursor(1, 1);
        set_color(Color::FGBlack, Color::BGGray);
        print!(" Select File");
        clear_eol();
        self.editor.screen.move_cursor(self.editor.screen.w - 11, 1);
        print!(" hedx v0.1");
        clear_eol();
        
        self.editor.screen.move_cursor(1, editor::HEADER_LINES);
        reset_color();
        clear_eol();
    }
    
    fn draw_footer(&mut self) {
        reset_color();
        let w = editor::SHORTCUT_SPACING;
        let h = self.editor.screen.h;

        // key shortcuts
        self.editor.void_key_help(1 + 0*w, h-1);
        self.editor.draw_key_help(1 + 0*w, h-0, "^C", "Cancel");

        self.editor.void_key_help(1 + 1*w, h-1);
        self.editor.void_key_help(1 + 1*w, h-0);

        // message
        self.editor.screen.move_cursor(1, self.editor.screen.h - editor::FOOTER_LINES + 1);
        if let Some(ref msg) = self.editor.screen.msg {
            set_color(Color::FGBlack, Color::BGGray);
            print!(" {}", msg);
        }
        clear_eol();
    }
    
    fn draw_main_screen(&mut self) {
        self.draw_header();
        self.draw_footer();

        let mut max_filename_len = 0;
        for fi in &self.files {
            let count = fi.name.to_string_lossy().chars().count();
            if max_filename_len < count {
                max_filename_len = count;
            }
        }
        if self.editor.screen.w < 20 {
            return;
        }
        if max_filename_len > (self.editor.screen.w - 20) as usize {
            max_filename_len = (self.editor.screen.w - 20) as usize;
        }
        
        let mut line = editor::HEADER_LINES + 1;
        let mut file_index = self.top_index;
        reset_color();
        while file_index < self.files.len() && line <= self.editor.screen.h - editor::FOOTER_LINES {
            self.editor.screen.move_cursor(1, line);
            if file_index == self.sel_index {
                set_color(Color::FGBlack, Color::BGGray);
            }
            
            let fi = &self.files[file_index];
            let len = fi.name.to_string_lossy().chars().count();
            for (i, c) in fi.name.to_string_lossy().char_indices() {
                if len > max_filename_len && i > max_filename_len-3 {
                    print!("...");
                    break;
                }
                print!("{}", c);
            }
            for _ in len..max_filename_len {
                print!(" ");
            }
            if fi.is_dir {
                print!("              (dir)");
            } else {
                print!(" {:12} bytes", 0);
            }

            reset_color();
            clear_eol();

            file_index += 1;
            line += 1;
        }

        while file_index < self.files.len() && line <= self.editor.screen.h - editor::FOOTER_LINES {
            self.editor.screen.move_cursor(1, line);
            clear_eol();
        }
        
        flush_screen();
    }

    fn process_input(&mut self) {
        let key = self.editor.read_key();
        if key == ctrl_key!('c') {
            self.quit = true;
            return;
        }
        if key == 13 {
            self.confirm_selection();
            return;
        }

        self.editor.screen.msg_was_set = false;
        if key == KEY_ARROW_UP {
            self.move_sel_up();
        } else if key == KEY_ARROW_DOWN {
            self.move_sel_down();
        }
        if ! self.editor.screen.msg_was_set {
            self.editor.clear_msg();
        }
    }

    fn confirm_selection(&mut self) {
        if self.sel_index >= self.files.len() {
            return;
        }
        let file = self.files.remove(self.sel_index);
        if file.is_dir {
            if let Err(e) = self.change_dir(&file.name) {
                self.editor.show_msg(format!("Error listing directory: {}", e));
            }
            reset_color();
            clear_screen();
            self.editor.screen.redraw_needed = true;
        } else {
            self.selected_filename = if let Some(ref mut dir) = self.cur_dir.take() {
                dir.push(file.name.clone());
                match dir.canonicalize() {
                    Ok(dir) => Some(dir.as_os_str().to_os_string()),
                    Err(_) => Some(dir.as_os_str().to_os_string())
                }
            } else {
                Some(file.name.clone())
            };
            self.quit = true;
        }
    }

    fn ensure_sel_visible(&mut self) {
        let n_page_lines = (self.editor.screen.h - editor::BORDER_LINES) as usize;
        if self.sel_index < self.top_index || self.sel_index >= self.top_index + n_page_lines {
            if self.sel_index >= n_page_lines/2 {
                self.top_index = self.sel_index - n_page_lines/2;
            } else {
                self.top_index = 0;
            }
            self.editor.screen.redraw_needed = true;
        }
    }
    
    fn move_sel_up(&mut self) {
        if self.sel_index > 0 {
            self.sel_index -= 1;
            self.ensure_sel_visible();
        }
        self.editor.screen.redraw_needed = true;
    }

    fn move_sel_down(&mut self) {
        if self.sel_index+1 < self.files.len() {
            self.sel_index += 1;
            self.ensure_sel_visible();
        }
        self.editor.screen.redraw_needed = true;
    }
    
    fn change_dir(&mut self, dir : &OsStr) -> io::Result<()> {
        let mut path = PathBuf::new();
        if let Some(ref root) = self.cur_dir {
            path.push(root);
        }
        path.push(dir);

        let mut files = vec![];
        files.push(FileInfo {
            name : OsStr::new("..").to_os_string(),
            is_dir : true,
        });
        
        let list = fs::read_dir(&path)?;
        for file in list {
            let file = file?;
            let fi = FileInfo {
                name : file.file_name(),
                is_dir : file.path().is_dir(),
            };
            files.push(fi);
        }

        files.sort();
        self.files = files;
        self.sel_index = 0;
        self.top_index = 0;
        self.cur_dir = Some(path);
        Ok(())
    }
    
    pub fn select_file(&mut self, root_dir : &OsStr) -> Option<OsString> {

        reset_color();
        clear_screen();
        flush_screen();

        if let Err(e) = self.change_dir(root_dir) {
            self.editor.show_msg(format!("Error reading directory: {}", e));
        } else {
            self.editor.clear_msg();
        }
        self.editor.screen.redraw_needed = true;
        while ! self.quit && ! self.editor.quit {
            if self.editor.screen.redraw_needed {
                self.draw_main_screen();
            }
            self.process_input();
        }
        reset_color();
        clear_screen();
        flush_screen();
        self.editor.screen.redraw_needed = true;
        
        self.selected_filename.take()
    }
    
}
