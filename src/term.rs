use std::io;
use std::io::Read;
use termios::*;

macro_rules! ctrl_key {
    ($x:expr) => (($x as u32) & 0x1f);
}

macro_rules! alt_key {
    ($x:expr) => ((($x as u32) & 0x1f) + KEY_ALT_MIN);
}

const KEY_ALT_MIN : u32 = 3000;

const KEY_ARROW_UP         : u32 = 1001;
const KEY_ARROW_DOWN       : u32 = 1002;
const KEY_ARROW_LEFT       : u32 = 1003;
const KEY_ARROW_RIGHT      : u32 = 1004;

const KEY_HOME             : u32 = 1005;
const KEY_END              : u32 = 1006;
const KEY_INS              : u32 = 1007;
const KEY_DEL              : u32 = 1008;
const KEY_PAGE_UP          : u32 = 1009;
const KEY_PAGE_DOWN        : u32 = 1010;

const KEY_CTRL_HOME        : u32 = 1011;
const KEY_CTRL_END         : u32 = 1012;
const KEY_CTRL_INS         : u32 = 1013;
const KEY_CTRL_DEL         : u32 = 1014;
const KEY_CTRL_PAGE_UP     : u32 = 1015;
const KEY_CTRL_PAGE_DOWN   : u32 = 1016;

const KEY_F1               : u32 = 1021;
const KEY_F2               : u32 = 1022;
const KEY_F3               : u32 = 1023;
const KEY_F4               : u32 = 1024;
const KEY_F5               : u32 = 1025;
const KEY_F6               : u32 = 1026;
const KEY_F7               : u32 = 1027;
const KEY_F8               : u32 = 1028;
const KEY_F9               : u32 = 1029;
const KEY_F10              : u32 = 1030;
const KEY_F11              : u32 = 1031;
const KEY_F12              : u32 = 1032;

const KEY_SHIFT_F1         : u32 = 1041;
const KEY_SHIFT_F2         : u32 = 1042;
const KEY_SHIFT_F3         : u32 = 1043;
const KEY_SHIFT_F4         : u32 = 1044;
const KEY_SHIFT_F5         : u32 = 1045;
const KEY_SHIFT_F6         : u32 = 1046;
const KEY_SHIFT_F7         : u32 = 1047;
const KEY_SHIFT_F8         : u32 = 1048;
//const KEY_SHIFT_F9         : u32 = 1049;
//const KEY_SHIFT_F10        : u32 = 1050;
//const KEY_SHIFT_F11        : u32 = 1051;
//const KEY_SHIFT_F12        : u32 = 1052;

pub fn setup_term(fd : i32) -> Termios {
    let orig = Termios::from_fd(fd).unwrap();
    let mut termios = orig.clone();

    termios.c_iflag &= !(IGNBRK | BRKINT | PARMRK | ISTRIP | INLCR | IGNCR | ICRNL | IXON);
    termios.c_oflag &= !OPOST;
    termios.c_lflag &= !(ECHO | ECHONL | ICANON | ECHOE | ECHOK | ISIG | IEXTEN);
    termios.c_cflag &= !(CSIZE | PARENB);
    termios.c_cflag |= CS8;
    termios.c_cc[VMIN] = 0;
    termios.c_cc[VTIME] = 1;
    
    tcsetattr(fd, TCSANOW, &mut termios).unwrap();

    orig
}

pub fn restore_term(fd : i32, termios : &mut Termios) {
    tcsetattr(fd, TCSANOW, termios).unwrap();
}

fn is_letter(b : u8) -> bool {
    b >= b'A' && b <= b'Z'
}

fn is_digit(b : u8) -> bool {
    b >= b'0' && b <= b'9'
}

fn parse_key(s : &[u8]) -> ::std::io::Result<u32> {
    print!("key: <ESC>");
    for b in s {
        if *b >= 32u8 && *b < 127u8 {
            print!("{}", *b as char);
        } else if *b == 0x1b {
            print!("<ESC>");
        } else {
            print!("<\\x{:x}>", b);
        }
    }
    print!("\r\n");

    if s.len() == 0 {
        return Ok(0x1b);
    }
    
    if s.len() == 1 && s[0] >= 32 && s[0] < 127 {
        return Ok(alt_key!(s[0]));
    }

    if s.len() == 2 && s[0] == b'[' && is_letter(s[1]) {
        let key = match s[1] {
            b'A' => KEY_ARROW_UP,
            b'B' => KEY_ARROW_DOWN,
            b'C' => KEY_ARROW_RIGHT,
            b'D' => KEY_ARROW_LEFT,
            b'H' => KEY_HOME,
            b'F' => KEY_END,
            _ => 0xffff_ffff,
        };
        return Ok(key);
    }

    if s.len() == 3 && s[0] == b'[' && is_digit(s[1]) && s[2] == b'~' {
        let key = match s[1] {
            b'1' => KEY_HOME,
            b'2' => KEY_INS,
            b'3' => KEY_DEL,
            b'4' => KEY_END,
            b'5' => KEY_PAGE_UP,
            b'6' => KEY_PAGE_DOWN,
            b'7' => KEY_HOME,
            b'8' => KEY_END,
            _ => 0xffff_ffff,
        };
        return Ok(key);
    }

    if s.len() == 3 && s[0] == b'[' && is_digit(s[1]) && s[2] == b'^' {
        let key = match s[1] {
            b'1' => KEY_CTRL_HOME,
            b'2' => KEY_CTRL_INS,
            b'3' => KEY_CTRL_DEL,
            b'4' => KEY_CTRL_END,
            b'5' => KEY_CTRL_PAGE_UP,
            b'6' => KEY_CTRL_PAGE_DOWN,
            b'7' => KEY_CTRL_HOME,
            b'8' => KEY_CTRL_END,
            _ => 0xffff_ffff,
        };
        return Ok(key);
    }

    if s.len() == 5 && s[0] == b'[' && is_digit(s[1]) && s[2] == b';' && is_digit(s[3]) {
        if s[1] == b'1' && s[3] == b'5' && s[4] == b'H' { return Ok(KEY_CTRL_HOME); }
        if s[1] == b'1' && s[3] == b'5' && s[4] == b'F' { return Ok(KEY_CTRL_END); }
    }

    if s.len() == 2 && s[0] == b'O' {
        let key = match s[1] {
            b'F' => KEY_HOME,
            b'H' => KEY_END,
            b'P' => KEY_F1,
            b'Q' => KEY_F2,
            b'R' => KEY_F3,
            b'S' => KEY_F4,
            _ => 0xffff_ffff,
        };
        return Ok(key);
    }

    if s.len() == 3 && s[0] == b'[' && s[1] == b'[' && is_letter(s[2]) {
        let key = match s[1] {
            b'A' => KEY_F1,
            b'B' => KEY_F2,
            b'C' => KEY_F3,
            b'D' => KEY_F4,
            b'E' => KEY_F5,
            _ => 0xffff_ffff,
        };
        return Ok(key);
    }

    if s.len() == 4 && s[0] == b'[' && is_digit(s[1]) && is_digit(s[2]) && s[3] == b'~' {
        let key = match (s[1], s[2]) {
            (b'1', b'5') => KEY_F5,
            (b'1', b'7') => KEY_F6,
            (b'1', b'8') => KEY_F7,
            (b'1', b'9') => KEY_F8,
            (b'2', b'0') => KEY_F9,
            (b'2', b'1') => KEY_F10,
            (b'2', b'3') => KEY_F11,
            (b'2', b'4') => KEY_F12,

            (b'2', b'5') => KEY_SHIFT_F1,
            (b'2', b'6') => KEY_SHIFT_F2,
            (b'2', b'8') => KEY_SHIFT_F3,
            (b'2', b'9') => KEY_SHIFT_F4,
            (b'3', b'1') => KEY_SHIFT_F5,
            (b'3', b'2') => KEY_SHIFT_F6,
            (b'3', b'3') => KEY_SHIFT_F7,
            (b'3', b'4') => KEY_SHIFT_F8,

            (_, _) => 0xffff_ffff,
        };
        return Ok(key);
    }
    
    return Ok(0xffff_ffff)
}

pub fn read_key() -> ::std::io::Result<u32> {
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut buffer = [0u8;32];
    loop {
        match reader.read(&mut buffer[0..1]) {
            Ok(n) => {
                if n == 0 {
                    continue;
                }
                if buffer[0] != 0x1b {
                    return Ok(buffer[0] as u32);
                }
                break;
            },
            Err(e) => return Err(e)
        };
    };
    let mut n = 0_usize;
    let mut min = 2usize;
    loop {
        if n >= buffer.len() {
            return parse_key(&buffer[0..n]);
        }
        match reader.read(&mut buffer[n..n+1]) {
            Ok(len) => {
                if len == 0 {
                    return parse_key(&buffer[0..n]);
                }
                let b = buffer[n];
                n += 1;
                if min == 0 && (b == b'~' || b == b'^' || (b >= b'A' && b <= b'Z')) {
                    return parse_key(&buffer[0..n]);
                }
                if min == 0 && b == b';' {
                    min = 2;
                }
                if min > 0 {
                    min -= 1;
                }
            },
            Err(e) => return Err(e),
        }
    };
}
