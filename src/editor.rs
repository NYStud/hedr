
use super::screen::*;
use super::term::*;

const KEY_HELP_SPACING : i32 = 16;

pub struct Editor {
    screen : Screen,
    quit : bool,
}

impl Editor {
    pub fn new() -> Editor {
        Editor {
            screen : Screen::new(),
            quit : false,
        }
    }

    pub fn run(&mut self) {
        let mut orig_term = setup_term(0);
        self.screen.init();
        self.screen.redraw_needed = true;
        show_cursor(false);
        clear_screen();

        while ! self.quit {
            if self.screen.redraw_needed {
                self.draw_main_screen();
            }
            self.process_input();
        }
       
        reset_color();
        clear_screen();
        show_cursor(true);
        restore_term(0, &mut orig_term);
    }

    fn show_msg<S>(&mut self, msg : S) where S: Into<String> {
        self.screen.show_msg(msg);
    }

    fn draw_header(&mut self) {
        self.screen.move_cursor(1, 1);
        set_color(Color::FGBlack, Color::BGGray);
        print!(" hedx v0.0.1");
        clear_eol();
    }

    fn draw_key_help(&mut self, x : i32, y : i32, key : &str, help : &str) {
        self.screen.move_cursor(x, y);
        set_color(Color::FGBlack, Color::BGGray);
        print!("{}", key);
        reset_color();
        print!(" {}", help);
        for _ in help.len()..(KEY_HELP_SPACING as usize) {
            print!(" ");
        }
    }
    
    fn draw_footer(&mut self) {
        reset_color();
        let w = KEY_HELP_SPACING;
        let h = self.screen.h;
        
        self.draw_key_help(1 + 0*w, h-1, "^G",  "Get Help");
        self.draw_key_help(1 + 0*w, h-0, "^X",  "Exit");

        self.draw_key_help(1 + 1*w, h-1, "^O",  "Write File");
        self.draw_key_help(1 + 1*w, h-0, "^R",  "Read File");
        
        self.screen.move_cursor(1, self.screen.h-2);
        if let Some(ref msg) = self.screen.msg {
            set_color(Color::FGBlack, Color::BGGray);
            print!(" {}", msg);
        }
        clear_eol();
    }
    
    fn draw_main_screen(&mut self) {
        self.draw_header();
        self.draw_footer();
        flush();
        self.screen.redraw_needed = false;
    }

    fn process_input(&mut self) {
        match read_key() {
            Ok(key) => {
                self.screen.msg_was_set = false;
                if key == ctrl_key!('x') {
                    self.quit = true;
                    return;
                } else if key == ctrl_key!('g') {
                    self.show_msg("Help is not available just yet");
                } else if key == ctrl_key!('o') {
                    self.show_msg("Writing files not implemented");
                } else if key == ctrl_key!('r') {
                    self.show_msg("Reading files not implemented");
                }
                
                if ! self.screen.msg_was_set {
                    self.screen.clear_msg();
                }
            },
            Err(e) => {
                self.show_msg(format!("ERROR: {:?}\r\n", e));
            }
        };
        
    }
}
