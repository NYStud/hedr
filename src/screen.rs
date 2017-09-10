
use std::io;
use std::io::Write;
use super::term::get_win_size;

#[derive(PartialEq)]
pub enum Color {
    None,
    
    FGDefault = 39,
    FGBlack   = 30,
    FGRed     = 31,
    FGGreen   = 32,
    FGYellow  = 33,
    FGBlue    = 34,
    FGMagenta = 35,
    FGCyan    = 36,
    FGGray    = 37,
    
    BGDefault = 49,
    BGBlack   = 40,
    BGRed     = 41,
    BGGreen   = 42,
    BGYellow  = 43,
    BGBlue    = 44,
    BGMagenta = 45,
    BGCyan    = 46,
    BGGray    = 47,
}

pub struct Screen {
    w : i32,
    h : i32,
}

impl Screen {
    pub fn new() -> Screen {
        Screen {
            w : 0,
            h : 0,
        }
    }

    pub fn init(&mut self) -> bool {
        if let Some((w,h)) = get_win_size(0) {
            self.w = w;
            self.h = h;
            true
        } else {
            false
        }
    }

    pub fn move_cursor(&self, x : i32, y : i32) {
        let mut x = x;
        let mut y = y;
        if x < 1 { x = 1; }
        if y < 1 { y = 1; }
        if x > self.w { x = self.w; }
        if y > self.h { y = self.h; }
        print!("\x1b[{};{}H", y, x);
    }
}

pub fn set_color(c1 : Color, c2 : Color) {
    if c1 != Color::None {
        print!("\x1b[{}m", c1 as i32);
    }
    if c2 != Color::None {
        print!("\x1b[{}m", c2 as i32);
    }
}

pub fn reset_color() {
    print!("\x1b[0m");
}

pub fn show_cursor(show : bool) {
    if show {
        print!("\x1b[?25h");
    } else {
        print!("\x1b[?25l");
    }
}

pub fn clear_eol() {
    print!("\x1b[K");
}

pub fn clear_screen() {
    print!("\x1b[2J");
    print!("\x1b[H");
}

pub fn flush()
{
    let stdout = io::stdout();
    stdout.lock().flush().unwrap();
}
