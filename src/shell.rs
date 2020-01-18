use crate::{
    print,
    println,
    draw_char,
    fill_buffer,
    bmp,
    cap_bmp,
    rect,
    vga_apply,
    color,
    vga_buffer::Bitmap,
    vga_buffer::Color,
    vga_buffer::ScreenChar
};
use alloc::{vec::Vec,vec};
use lazy_static::lazy_static;
use spin::Mutex;
use core::fmt;

lazy_static! {
    pub static ref LUSH: Mutex<LunaShell> = Mutex::new(LunaShell {
        input: Vec::new(),
        commands: Vec::new(),
        handlers: Vec::new(),
    });
}

lazy_static! {
    pub static ref LURE: Mutex<LunaRenderer> = Mutex::new(LunaRenderer {
        input: Vec::new(),
        lastInputLength: 65535,
        lines: Vec::new(),
        color: Color::LightGray,
        enabled: true,
    });
}


lazy_static! {
    pub static ref LULI: Mutex<LunaListeners> = Mutex::new(LunaListeners {
        key_listeners: Vec::new(),
    });
}

pub struct LunaListeners {
    pub key_listeners: Vec<fn(char)>
}

pub struct LunaShell {
    pub input: Vec<char>,
    pub commands: Vec<Vec<char>>,
    pub handlers: Vec<fn(Vec<char>)>
}

impl LunaShell {
    pub fn keyboard_event(&mut self, key: char) {
        if key == '\u{0000}' {
            LURE.lock().draw();
            return;
        }

        let key_listeners = LULI.lock().key_listeners.to_vec();

        if key_listeners.len() > 0 {
            key_listeners[key_listeners.len() - 1](key);
            LURE.lock().draw();
            return;
        }

        if key == '\n' {
            self.finish_line();
            return;
        }

        if key == '\u{0008}' {
            if self.input.len() > 0 {
                self.input.swap_remove(self.input.len() - 1);
                LURE.lock().input = self.input.to_vec();
            }
            LURE.lock().draw();
            return;
        }

        self.input.push(key);
        LURE.lock().input = self.input.to_vec();
        LURE.lock().draw();
    }

    fn finish_line(&mut self) {
        let input: Vec<char> = self.input.to_vec();
        let mut command: Vec<char> = Vec::new();

        print!("\n");
        color!(Color::Blue);
        print!(">");
        color!(Color::LightGray);
        for i in &input {
            print!("{}", i);
        }
        color!(Color::DarkGray);
        print!("\n");

        let mut cmd_idx = 0;

        if input.len() > 0 {
            for cmd in &self.commands {
                let mut accepted = true;

                let mut cmd_char_idx = 0;

                for i in &input {
                    if input.len() < cmd.len() {
                        accepted = false;
                        break;
                    }

                    if cmd_char_idx >= cmd.len() {
                        if *i == ' ' {
                            break;
                        }
                        accepted = false;
                        break;
                    }

                    if &cmd[cmd_char_idx] != i {
                        accepted = false;
                    }

                    cmd_char_idx = cmd_char_idx + 1;
                }

                if accepted {
                    command = cmd.to_vec();
                    break;
                } else {
                    cmd_idx = cmd_idx + 1;
                }
            }
        }

        if command.len() == 0 {
            color!(Color::LightRed);
            print!("Unknown Command.")
        } else {
            // clone args to a mutable vector.
            let mut argsstr = self.input.to_vec();

            // remove "command " from string.
            if argsstr.len() > command.len() {
                for _ in 0..(command.len() + 1) {
                    argsstr.remove(0);
                }
            } else {
                argsstr = Vec::new();
            }

            self.handlers[cmd_idx](argsstr);
        }

        core::mem::drop(command);
        core::mem::drop(input);
        
        self.input = Vec::new();
        LURE.lock().input = Vec::new();
        
        print!("\n");
        LURE.lock().draw();
    }
}

pub struct LunaLine {
    pub chars: Vec<ScreenChar>,
    pub bitmap: Bitmap
}

impl LunaLine {
    pub fn new() -> LunaLine {
        LunaLine {
            chars: Vec::new(),
            bitmap: Bitmap::new(0,0,vec![])
        }
    }
}

pub struct LunaRenderer {
    pub input: Vec<char>,
    pub lastInputLength: u16,
    pub lines: Vec<LunaLine>,
    pub color: Color,
    pub enabled: bool
}


impl LunaRenderer {
    pub fn draw(&mut self) {
        if !self.enabled { return; }
        let mut lIx = 0;
        let mut cIx = 0;

        if self.input.len() as u16 <= self.lastInputLength {
            fill_buffer!(Color::Black);

            while self.lines.len() > 21 {
                self.lines.remove(0);
            }

            for i in 0..self.lines.len() {
                let line = &self.lines[i]; 
                if line.bitmap.width > 0 {
                    bmp!(0, 8*lIx, &line.bitmap);
                } else {
                    for chr in line.chars.to_vec() {
                        draw_char!(8*cIx, 8*lIx, chr);
                        cIx = cIx + 1;
                    }
                    self.lines[i].bitmap = cap_bmp!(0, 8*lIx, 319, 8);
                }
                
                lIx = lIx + 1;
                cIx = 0;
            }
        }

        draw_char!(8*cIx, 8*24 - 4, ScreenChar::new('>', Color::LightBlue));

        for chr in self.input.to_vec() {
            cIx = cIx + 1;
            draw_char!(8*cIx, 8*24 - 4, ScreenChar::new(chr, Color::LightGray));
        }

        self.lastInputLength = self.input.len() as u16;

        vga_apply!();
    }

    fn push_bmp_line(&mut self, bmp: Bitmap) {
        let lureLine = LunaLine {
            bitmap: bmp,
            chars: vec![]
        };
        self.lines.push(lureLine);
    }

    fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                b'\n' => {
                    self.lines.push(LunaLine::new());
                },
                0x20..=0x7e => {
                    if self.lines.len() == 0 { self.lines.push(LunaLine::new()); }
                    let idx = self.lines.len() - 1;
                    self.lines[idx].chars.push(ScreenChar::new(byte as char, self.color));
                },
                _ => {},
            }
        }
    }
}

impl fmt::Write for LunaRenderer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use x86_64::instructions::interrupts;
    use core::fmt::Write;

    interrupts::without_interrupts(|| {
        LURE.lock().write_fmt(args).unwrap();
    });
}

#[doc(hidden)]
pub fn _draw() {
    LURE.lock().draw();
}

#[doc(hidden)]
pub fn _color(color: Color) {
    LURE.lock().color = color;
}

#[doc(hidden)]
pub fn _lure_set_enable(enabled: bool) {
    LURE.lock().enabled = enabled;
}

#[doc(hidden)]
pub fn _lure_push_bmp(bmp: Bitmap) {
    LURE.lock().push_bmp_line(bmp);
}

#[doc(hidden)]
pub fn _lushkey_handler(key: char) {
    LUSH.lock().keyboard_event(key);
}

#[doc(hidden)]
pub fn _lush_pop_listener() {
    LULI.lock().key_listeners.pop();
}

#[doc(hidden)]
pub fn _lush_push_listener(key_listener: fn(char)) {
    LULI.lock().key_listeners.push(key_listener);
}

#[macro_export]
macro_rules! lure_enabled {
    ($enabled:expr) => ($crate::shell::_lure_set_enable($enabled));
}

#[macro_export]
macro_rules! lure_bmp {
    ($bitmap:expr) => ($crate::shell::_lure_push_bmp($bitmap));
}

#[macro_export]
macro_rules! lush_keypop {
    () => ($crate::shell::_lush_pop_listener());
}

#[macro_export]
macro_rules! lush_keypush {
    ($listener:expr) => ($crate::shell::_lush_push_listener($listener));
}

#[macro_export]
macro_rules! LUSHKeyHandler {
    ($key:expr) => ($crate::shell::_lushkey_handler($key));
}

#[doc(hidden)]
pub fn _lushadd_command(key: Vec<char>, handler: fn(Vec<char>)) {
    LUSH.lock().commands.push(key);
    LUSH.lock().handlers.push(handler);
}

#[macro_export]
macro_rules! LUSHAddCommand {
    ($key:expr, $handler:expr) => ($crate::shell::_lushadd_command($key, $handler));
}

#[macro_export]
macro_rules! draw {
    () => ($crate::shell::_draw());
}

#[macro_export]
macro_rules! color {
    ($color:expr) => ($crate::shell::_color($color));
}

/// Like the `print!` macro in the standard library, but prints to the VGA text buffer.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::shell::_print(format_args!($($arg)*)));
}

/// Like the `println!` macro in the standard library, but prints to the VGA text buffer.
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}