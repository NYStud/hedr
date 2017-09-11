
use super::screen::*;
use super::term::*;
use super::file::File;
use super::file_sel::FileSel;
use std::error::Error;
use std::io;
use std::ffi::{OsString, OsStr};

pub const SHORTCUT_SPACING : i32 = 16;
pub const HEADER_LINES : i32 = 2;
pub const FOOTER_LINES : i32 = 3;
pub const BORDER_LINES : i32 = HEADER_LINES + FOOTER_LINES;

#[derive(Copy, Clone, PartialEq)]
enum EditorMode {
    Default,
    ReadFilename,
    ReadString,
    ReadYesNo,
}

pub struct Editor {
    pub screen : Screen,
    pub quit : bool,
    files : Vec<File>,
    cur_file : usize,
    mode : EditorMode,
}

impl Editor {
    pub fn new() -> Editor {
        Editor {
            screen : Screen::new(),
            quit : false,
            files : Vec::new(),
            cur_file : 0,
            mode : EditorMode::Default,
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        let term_fd = 0;
        let mut orig_term = setup_term(term_fd)?;
        self.screen.init();
        show_cursor(false);
        clear_screen();

        if self.files.len() == 0 {
            self.add_file(File::new());
        }
        
        self.screen.redraw_needed = true;
        while ! self.quit {
            if self.screen.redraw_needed {
                self.draw_main_screen();
            }
            //if let Err(_) = self.process_input() {
            //    self.quit = true;
            //}
            self.process_input();
        }
       
        reset_color();
        clear_screen();
        show_cursor(true);
        flush_screen();
        restore_term(term_fd, &mut orig_term)?;
        Ok(())
    }

    pub fn read_key(&mut self) -> u32 {
        let stdin = io::stdin();
        let mut reader = stdin.lock();
        match read_key(&mut reader) {
            Ok(key) => key,
            Err(_) => {
                self.quit = true;
                0
            }
        }
    }
    
    pub fn cur_file_mut(&mut self) -> Option<&mut File> {
        self.files.get_mut(self.cur_file)
    }

    pub fn cur_file(&self) -> Option<&File> {
        self.files.get(self.cur_file)
    }
    
    pub fn add_file(&mut self, file : File) {
        if self.files.len() == 1 && self.files[0].data.len() == 0 {
            // remove initial empty file
            self.files.pop();
        }
        self.files.push(file);
        self.cur_file = self.files.len() - 1;
        //self.files[self.cur_file].top_line = 1;
    }

    pub fn remove_cur_file(&mut self) {
        if self.cur_file < self.files.len() {
            self.files.remove(self.cur_file);
            if self.cur_file > 0 {
                if self.cur_file > self.files.len() {
                    self.cur_file -= 1
                } else {
                    self.cur_file = 0;
                }
            }
        }
    }

    pub fn go_to_next_file(&mut self) {
        if self.cur_file+1 < self.files.len() {
            self.cur_file += 1;
        } else if self.files.len() > 0 {
            self.cur_file = (self.cur_file + 1) % self.files.len();
        }
    }

    pub fn go_to_prev_file(&mut self) {
        if self.cur_file > 0 {
            self.cur_file -= 1;
        } else if self.files.len() > 0 {
            self.cur_file = (self.cur_file + self.files.len() - 1) % self.files.len();
        }
    }
    
    pub fn show_msg<S>(&mut self, msg : S) where S: Into<String> {
        self.screen.show_msg(msg);
    }

    pub fn clear_msg(&mut self) {
        self.screen.clear_msg();
    }

    fn draw_header(&mut self) {
        self.screen.move_cursor(1, 1);
        set_color(Color::FGBlack, Color::BGGray);
        print!(" ");
        if let Some(file) = self.cur_file() {
            match file.filename {
                Some(ref filename) => print!("{}", filename),
                None => print!("NO FILE"),
            }
        }
        clear_eol();
        self.screen.move_cursor(self.screen.w - 11, 1);
        print!(" hedx v0.1");
        clear_eol();

        self.screen.move_cursor(1, HEADER_LINES);
        reset_color();
        clear_eol();
    }

    pub fn draw_key_help(&mut self, x : i32, y : i32, key : &str, help : &str) {
        self.screen.move_cursor(x, y);
        set_color(Color::FGBlack, Color::BGGray);
        print!("{}", key);
        reset_color();
        print!(" {}", help);
        for _ in help.len()..(SHORTCUT_SPACING as usize) {
            print!(" ");
        }
    }
    
    pub fn void_key_help(&mut self, x : i32, y : i32) {
        self.screen.move_cursor(x, y);
        reset_color();
        clear_eol();
    }
    
    fn draw_footer(&mut self) {
        reset_color();
        let w = SHORTCUT_SPACING;
        let h = self.screen.h;

        // key shortcuts
        match self.mode {
            EditorMode::Default => {
                self.draw_key_help(1 + 0*w, h-1, "^G", "Get Help");
                self.draw_key_help(1 + 0*w, h-0, "^X", "Exit");
                
                self.draw_key_help(1 + 1*w, h-1, "^O", "Write File");
                self.draw_key_help(1 + 1*w, h-0, "^R", "Read File");

                self.void_key_help(1 + 2*w, h-1);
                self.void_key_help(1 + 2*w, h-0);
            },

            EditorMode::ReadFilename => {
                self.draw_key_help(1 + 0*w, h-1, "^T", "To Files");
                self.draw_key_help(1 + 0*w, h-0, "^C", "Cancel");

                self.void_key_help(1 + 1*w, h-1);
                self.void_key_help(1 + 1*w, h-0);
            },

            EditorMode::ReadString => {
                self.void_key_help(1 + 0*w, h-1);
                self.draw_key_help(1 + 0*w, h-0, "^C",  "Cancel");

                self.void_key_help(1 + 1*w, h-1);
                self.void_key_help(1 + 1*w, h-0);
            },
            
            EditorMode::ReadYesNo => {
                self.draw_key_help(1 + 0*w, h-1, " Y", "Yes");
                self.draw_key_help(1 + 0*w, h-0, " N", "No");

                self.void_key_help(1 + 1*w, h-1);
                self.draw_key_help(1 + 1*w, h-0, "^C",  "Cancel");

                self.void_key_help(1 + 2*w, h-1);
                self.void_key_help(1 + 2*w, h-0);
            }
        }

        // message
        self.screen.move_cursor(1, self.screen.h - FOOTER_LINES + 1);
        if let Some(ref msg) = self.screen.msg {
            set_color(Color::FGBlack, Color::BGGray);
            print!(" {}", msg);
        }
        clear_eol();
    }
    
    fn draw_main_screen(&mut self) {
        self.draw_header();
        self.draw_footer();

        let mut line = HEADER_LINES + 1;

        if let Some(file) = self.cur_file() {
            let mut off = 16 * file.top_line;
            while off < file.data.len() && line <= self.screen.h - FOOTER_LINES {
                self.screen.move_cursor(1, line);
                print!("{:08x} | ", off);

                let line_len = if file.data.len() - off < 16 { file.data.len() - off } else { 16 };
                for i in 0..line_len {
                    if i == 8 { print!(" "); }
                    if file.cursor_pos == off+i {
                        self.screen.move_cursor((11 + 3*i + if i>=8 { 1 } else { 0 }) as i32, line);
                        set_color(Color::FGBlack, Color::BGGray);
                        print!(" ");
                    }
                    print!("{:02x} ", file.data[off+i]);
                    if file.cursor_pos == off+i {
                        reset_color();
                    }
                }
                for i in line_len..16 {
                    if i == 8 { print!(" "); }
                    print!("   ");
                }
                print!("| ");
                for i in 0..line_len {
                    let b = file.data[off+i];
                    if file.cursor_pos == off+i {
                        set_color(Color::FGBlack, Color::BGGray);
                    }
                    print!("{}", if b >= 32 && b < 127 { b } else { b'.' } as char);
                    if file.cursor_pos == off+i {
                        reset_color();
                    }
                }
                clear_eol();

                line += 1;
                off += 16;
            }
        }
        
        reset_color();
        for i in line .. self.screen.h - FOOTER_LINES + 1 {
            self.screen.move_cursor(1, i);
            //print!("{}", i);
            clear_eol();
        }
        
        flush_screen();
        self.screen.redraw_needed = false;
    }

    fn process_input(&mut self) {
        let key = self.read_key();
        self.screen.msg_was_set = false;
        if key == ctrl_key!('x') {
            self.quit = true;
            return;
        } else if key == ctrl_key!('l') {
            clear_screen();
            self.screen.redraw_needed = true;
        } else if key == ctrl_key!('g') {
            self.show_msg("Help is not available just yet");
        } else if key == ctrl_key!('o') {
            self.show_msg("Writing files not implemented");
        } else if key == ctrl_key!('r') {
            self.prompt_read_file();
        } else if key == ctrl_key!('a') || key == KEY_HOME {
            self.move_cursor_home();
        } else if key == ctrl_key!('e') || key == KEY_END {
            self.move_cursor_end();
        } else if key == KEY_ARROW_UP {
            self.move_cursor_up();
        } else if key == KEY_ARROW_DOWN {
            self.move_cursor_down();
        } else if key == KEY_ARROW_LEFT {
            self.move_cursor_left();
        } else if key == KEY_ARROW_RIGHT {
            self.move_cursor_right();
        } else if key == KEY_CTRL_HOME {
            self.move_cursor_start_of_file();
        } else if key == KEY_CTRL_END {
            self.move_cursor_end_of_file();
        } else if key == KEY_PAGE_UP {
            self.move_cursor_page_up();
        } else if key == KEY_PAGE_DOWN {
            self.move_cursor_page_down();
        } else if key == alt_key!(',') {
            self.go_to_prev_file();
        } else if key == alt_key!('.') {
            self.go_to_next_file();
        }
        
        if ! self.screen.msg_was_set {
            self.clear_msg();
        }
    }

    fn ensure_cursor_visible(&mut self, visible_len_after : usize) {
        let n_page_lines = (self.screen.h - BORDER_LINES) as usize;
        if let Some(file) = self.cur_file_mut() {
            let last_line = file.data.len() / 16 + if file.data.len() % 16 != 0 { 1 } else { 0 };

            if ! (file.cursor_pos / 16 >= file.top_line
                  && (file.cursor_pos+visible_len_after) / 16 >= file.top_line
                  && file.cursor_pos / 16 < file.top_line + n_page_lines
                  && (file.cursor_pos+visible_len_after) / 16 < file.top_line + n_page_lines) {
                if file.cursor_pos / 16 < n_page_lines/2 {
                    file.top_line = 0;
                } else {
                    file.top_line = file.cursor_pos / 16 - n_page_lines/2;
                    if file.top_line + n_page_lines > last_line {
                        file.top_line = last_line - n_page_lines;
                    }
                }
            }
        }
    }
    
    fn move_cursor_start_of_file(&mut self) {
        if let Some(file) = self.cur_file_mut() {
            file.cursor_pos = 0;
            file.top_line = 0;
        }
        self.screen.redraw_needed = true;
    }

    fn move_cursor_end_of_file(&mut self) {
        if let Some(file) = self.cur_file_mut() {
            if file.data.len() > 0 {
                file.cursor_pos = file.data.len() - 1;
            } else {
                file.cursor_pos = 0;
            }
        }
        self.ensure_cursor_visible(1);
        self.screen.redraw_needed = true;
    }
    
    fn move_cursor_home(&mut self) {
        if let Some(file) = self.cur_file_mut() {
            file.cursor_pos = file.cursor_pos / 16 * 16;
        }
        self.screen.redraw_needed = true;
    }

    fn move_cursor_end(&mut self) {
        if let Some(file) = self.cur_file_mut() {
            file.cursor_pos = file.cursor_pos / 16 * 16 + 15;
            if file.data.len() > 0 && file.cursor_pos >= file.data.len() {
                file.cursor_pos = file.data.len()-1;
            }
        }
        self.screen.redraw_needed = true;
    }
    
    fn move_cursor_page_up(&mut self) {
        let n_page_lines = (self.screen.h - BORDER_LINES) as usize;
        if let Some(file) = self.cur_file_mut() {
            if file.cursor_pos >= 16*n_page_lines {
                file.cursor_pos -= 16*n_page_lines;
                if file.top_line > n_page_lines {
                    file.top_line -= n_page_lines;
                } else {
                    file.top_line = 0;
                }
            } else {
                file.cursor_pos = 0;
            }
        }
        self.ensure_cursor_visible(0);
        self.screen.redraw_needed = true;
    }

    fn move_cursor_page_down(&mut self) {
        let n_page_lines = (self.screen.h - BORDER_LINES) as usize;
        if let Some(file) = self.cur_file_mut() {
            let last_line = file.data.len() / 16 + if file.data.len() % 16 != 0 { 1 } else { 0 };
            if file.cursor_pos + 16*n_page_lines < file.data.len() {
                file.cursor_pos += 16*n_page_lines;
                file.top_line += n_page_lines;
                if file.top_line + n_page_lines > last_line {
                    if last_line > n_page_lines {
                        file.top_line = last_line - n_page_lines;
                    } else {
                        file.top_line = 0;
                    }
                }
            } else if file.data.len() > 0 {
                file.cursor_pos = file.data.len() - 1;
            } else {
                file.cursor_pos = 0;
            }
        }
        self.screen.redraw_needed = true;
    }
    
    fn move_cursor_up(&mut self) {
        if let Some(file) = self.cur_file_mut() {
            if file.cursor_pos >= 16 {
                file.cursor_pos -= 16;
            }
        }
        self.ensure_cursor_visible(0);
        self.screen.redraw_needed = true;
    }

    fn move_cursor_down(&mut self) {
        if let Some(file) = self.cur_file_mut() {
            if file.cursor_pos+16 < file.data.len() {
                file.cursor_pos += 16;
            }
        }
        self.ensure_cursor_visible(0);
        self.screen.redraw_needed = true;
    }
    
    fn move_cursor_left(&mut self) {
        if let Some(file) = self.cur_file_mut() {
            if file.cursor_pos > 0 {
                file.cursor_pos -= 1;
            }
        }
        self.ensure_cursor_visible(0);
        self.screen.redraw_needed = true;
    }
    
    fn move_cursor_right(&mut self) {
        if let Some(file) = self.cur_file_mut() {
            if file.cursor_pos+1 < file.data.len() {
                file.cursor_pos += 1;
            } else if file.data.len() > 0 {
                file.cursor_pos = file.data.len()-1;
            } else {
                file.cursor_pos = 0;
            }
        }
        self.ensure_cursor_visible(0);
        self.screen.redraw_needed = true;
    }
    
    pub fn prompt_read_file(&mut self) -> bool {
        let filename = match self.prompt_get_filename("Read file") {
            Some(filename) => filename,
            None => return false,
        };

        match File::new_from_file(filename) {
            Ok(file) => {
                self.add_file(file);
                true
            },
            Err(e) => {
                self.show_msg(format!("Error reading file: {}", e.description()));
                false
            },
        }
    }

    pub fn prompt_get_filename(&mut self, prompt : &str) -> Option<OsString> {
        let old_mode = self.mode;
        self.mode = EditorMode::ReadFilename;
        let ret = self.prompt_get_text(prompt);
        self.mode = old_mode;
        ret
    }

    pub fn prompt_get_string(&mut self, prompt : &str) -> Option<String> {
        let old_mode = self.mode;
        self.mode = EditorMode::ReadString;
        let ret = self.prompt_get_text(prompt);
        self.mode = old_mode;
        if let Some(os_str) = ret {
            match os_str.into_string() {
                Ok(string) => Some(string),
                Err(_) => None,
            }
        } else {
            None
        }
    }

    pub fn prompt_get_yes_no(&mut self, prompt : &str) -> Option<bool> {
        let old_mode = self.mode;
        self.mode = EditorMode::ReadYesNo;
        
        let mut answer : Option<bool> = None;

        self.clear_msg();
        self.screen.redraw_needed = true;
        while ! self.quit {
            show_cursor(false);
            if self.screen.redraw_needed {
                self.draw_main_screen();
            }
            reset_color();
            set_color(Color::FGBlack, Color::BGGray);
            self.screen.move_cursor(1, self.screen.h - FOOTER_LINES + 1);
            print!(" {}", prompt);
            clear_eol();
            show_cursor(true);
            flush_screen();

            let key = self.read_key();
            if key == ctrl_key!('c') {
                break;
            }
            if key == b'Y' as u32 || key == b'y' as u32 {
                answer = Some(true);
                break;
            }
            if key == b'Y' as u32 || key == b'y' as u32 {
                answer = Some(false);
                break;
            }
        }

        show_cursor(false);
        self.screen.redraw_needed = true;
        self.mode = old_mode;
        answer
    }

    fn prompt_get_text(&mut self, prompt : &str) -> Option<OsString> {
        let mut filename = Vec::<char>::new();
        let mut text : Option<OsString> = None;
        let mut cursor_pos = 0_usize;

        self.clear_msg();
        self.screen.redraw_needed = true;
        while ! self.quit {
            show_cursor(false);
            if self.screen.redraw_needed {
                self.draw_main_screen();
            }
            reset_color();
            set_color(Color::FGBlack, Color::BGGray);
            self.screen.move_cursor(1, self.screen.h - FOOTER_LINES + 1);
            print!(" {}: ", prompt);
            for c in &filename {
                print!("{}", c);
            }
            clear_eol();
            self.screen.move_cursor(((prompt.len() + 4 + cursor_pos)&0xffff_ffff) as i32, self.screen.h - FOOTER_LINES + 1);
            show_cursor(true);
            flush_screen();

            let key = self.read_key();
            if key == ctrl_key!('c') {
                break;
            }
            if key == 13 {
                let string : String = filename.into_iter().collect();
                text = Some(string.into());
                break;
            }

            if key >= 32 && key < 127 {
                if cursor_pos >= filename.len() {
                    filename.push(key as u8 as char);
                } else {
                    filename.insert(cursor_pos, key as u8 as char);
                }
                cursor_pos += 1;
                continue;
            }
            if key == 8 || key == 127 {
                if cursor_pos > 0 {
                    cursor_pos -= 1;
                    filename.remove(cursor_pos);
                }
                continue;
            }
            if key == ctrl_key!('a') || key == KEY_HOME {
                cursor_pos = 0;
                continue;
            }
            if key == ctrl_key!('e') || key == KEY_END {
                cursor_pos = filename.len();
                continue;
            }
            if key == ctrl_key!('t') && self.mode == EditorMode::ReadFilename {
                let mut fs = FileSel::new(self);
                show_cursor(false);
                text = fs.select_file(OsStr::new("."));
                show_cursor(true);
                if text.is_some() {
                    break;
                }
            }
            if key == KEY_ARROW_LEFT {
                if cursor_pos > 0 {
                    cursor_pos -= 1;
                }
                continue;
            }
            if key == KEY_ARROW_RIGHT {
                if cursor_pos < filename.len() {
                    cursor_pos += 1;
                }
                continue;
            }
        }

        show_cursor(false);
        self.screen.redraw_needed = true;
        text
    }
    
}
