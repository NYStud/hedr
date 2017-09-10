
use super::screen::*;
use super::term::*;

pub struct Editor {
    screen : Screen,
}

impl Editor {
    pub fn new() -> Editor {
        Editor {
            screen : Screen::new(),
        }
    }

    pub fn run(&mut self) {
        let mut orig_term = setup_term(0);
        self.screen.init();
        show_cursor(false);
        clear_screen();

        self.test_loop();
       
        reset_color();
        clear_screen();
        show_cursor(true);
        restore_term(0, &mut orig_term);
    }

    fn test_loop(&self) {
        print!("Press some keys, ^X to exit...\r\n");
        flush();
    
        loop {
            match read_key() {
                Ok(key) => {
                    if key == ctrl_key!('x') {
                        break
                    };
                    self.screen.move_cursor(1, 3);
                    if key >= 32 && key < 127 {
                        reset_color();
                    } else {
                        set_color(Color::FGBlack, Color::BGGray);
                    }
                    print!("{}", key);
                    clear_eol();
                    flush();
                },
                Err(e) => {
                    print!("ERROR: {:?}\r\n", e);
                    break;
                }
            };
        };
    }
}
