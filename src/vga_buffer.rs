use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;
use alloc::{vec::Vec};

use font8x8::{BASIC_FONTS, UnicodeFonts};

#[cfg(test)]
use crate::{serial_print, serial_println};

lazy_static! {
    /// A global `Writer` instance that can be used for printing to the VGA text buffer.
    ///
    /// Used by the `print!` and `println!` macros.
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        vgabuffer: unsafe { &mut *(0xa0000 as *mut Buffer) },
        buffer: [[Color::Black as u8; BUFFER_WIDTH]; BUFFER_HEIGHT],
    });
}

/// The standard color palette in VGA text mode.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

pub struct Bitmap {
    pub data: Vec<u8>,
    pub height: usize,
    pub width: usize,
}

impl Bitmap {
    /// Create a new `ColorCode` with the given foreground and background colors.
    pub fn new(width: usize, height: usize, data: Vec<u8>) -> Bitmap {
        Bitmap {
            data: data,
            height: height,
            width: width,
        }
    }
}

/// A screen character in the VGA text buffer, consisting of an ASCII character and a `ColorCode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct ScreenChar {
    character: char,
    color: Color,
}

impl ScreenChar {
    /// Create a new `ColorCode` with the given foreground and background colors.
    pub fn new(character: char, color: Color) -> ScreenChar {
        ScreenChar {
            character: character,
            color: color
        }
    }
}

/// The height of the vga buffer.
const BUFFER_HEIGHT: usize = 200;
/// The width of the vga buffer.
const BUFFER_WIDTH: usize = 320;

/// The height of the vga buffer.
const CHAR_WIDTH: usize = 8;
/// The width of the vga buffer.
const CHAR_HEIGHT: usize = 8;

/// A structure representing the VGA text buffer.
#[repr(transparent)]
struct Buffer {
    pixels: [[u8; BUFFER_WIDTH]; BUFFER_HEIGHT],
}


/// A writer type that allows writing ASCII bytes and strings to an underlying `Buffer`.
///
/// Wraps lines at `BUFFER_WIDTH`. Supports newline characters and implements the
/// `core::fmt::Write` trait.
pub struct Writer {
    vgabuffer: &'static mut Buffer,
    buffer: [[u8; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

impl Writer {
    pub fn write_pixel(&mut self, x: usize, y: usize, color: Color) {
        self.buffer[y][x] = color as u8;
    }

    pub fn write_raw_pixel(&mut self, x: usize, y: usize, color: u8) {
        self.buffer[y][x] = color;
    }

    pub fn capture_bmp(&mut self, x: usize, y: usize, width: usize, height: usize) -> Bitmap {
        let mut capture = Bitmap::new(width, height, Vec::new());
        for _y in y..(y+height) {
            for _x in x..(x+width) {
                capture.data.push(self.buffer[_y][_x] as u8);
            }
        }
        return capture;
    }

    pub fn draw_rect(&mut self, x: usize, y: usize, width: usize, height: usize, color: Color) {
        let c = color as u8;
        for _y in y..(y+height) {
            for _x in x..(x+width) {
                self.buffer[_y][_x] = c;
            }
        }
    }

    pub fn fill_buffer(&mut self, color: Color) {
        self.buffer = [[color as u8; BUFFER_WIDTH]; BUFFER_HEIGHT];
    }

    pub fn draw_bmp(&mut self, x: usize, y: usize, bmp: &Bitmap) {
        for _y in 0..(bmp.height) {
            for _x in 0..(bmp.width) {
                if bmp.data[(_y * bmp.width + _x)] > 0 {
                    self.buffer[y + _y][x + _x] = bmp.data[(_y * bmp.width + _x)] as u8;
                }
            }
        }
    }

    pub fn draw_char(&mut self, x: usize, y: usize, screen_char: ScreenChar) {
        if let Some(glyph) = BASIC_FONTS.get(screen_char.character) {
            let mut _x = 0;
            let mut _y = 0;
            for g in &glyph {
                for bit in 0..8 {
                    match *g & 1 << bit {
                        0 => self.write_pixel(x + _x, y + _y, Color::Black),
                        _ => self.write_pixel(x + _x, y + _y, screen_char.color),
                    }
                    _x = _x + 1;
                }
                _x = 0;
                _y = _y + 1;
            }
        }
    }

    pub fn apply(&mut self) {
        self.vgabuffer.pixels = self.buffer.clone();
    } 

    /// Writes the given ASCII string to the buffer.
    ///
    /// Wraps lines at `BUFFER_WIDTH`. Supports the `\n` newline character. Does **not**
    /// support strings with non-ASCII characters, since they can't be printed in the VGA text
    /// mode.
    fn write_string(&mut self, s: &str) {

    }

    /// Shifts all lines one line up and clears the last row.
    fn new_line(&mut self) {

    }

    /// Clears a row by overwriting it with blank characters.
    fn clear_row(&mut self, row: usize) {

    }
}

#[macro_export]
macro_rules! pixel {
    ($x:expr, $y:expr, $color:expr) => ($crate::vga_buffer::_pixel($x, $y, $color));
}

#[macro_export]
macro_rules! raw_pixel {
    ($x:expr, $y:expr, $color:expr) => ($crate::vga_buffer::_raw_pixel($x, $y, $color));
}

#[macro_export]
macro_rules! rect {
    ($x:expr, $y:expr, $w:expr, $h:expr, $color:expr) => ($crate::vga_buffer::_rect($x, $y, $w, $h, $color));
}

#[macro_export]
macro_rules! cap_bmp {
    ($x:expr, $y:expr, $w:expr, $h:expr) => ($crate::vga_buffer::_cap_bmp($x, $y, $w, $h));
}


#[macro_export]
macro_rules! bmp {
    ($x:expr, $y:expr, $bmp:expr) => ($crate::vga_buffer::_bmp($x, $y, $bmp));
}

#[macro_export]
macro_rules! draw_char {
    ($x:expr, $y:expr, $chr:expr) => ($crate::vga_buffer::_char($x, $y, $chr));
}

#[macro_export]
macro_rules! fill_buffer {
    ($color:expr) => ($crate::vga_buffer::_fill_buffer($color));
}

#[macro_export]
macro_rules! vga_apply {
    () => ($crate::vga_buffer::_apply());
}

pub fn _pixel(x: usize, y: usize, color: Color) {
    WRITER.lock().write_pixel(x, y, color);
}

pub fn _raw_pixel(x: usize, y: usize, color: u8) {
    WRITER.lock().write_raw_pixel(x, y, color);
}

pub fn _rect(x: usize, y: usize, width: usize, height: usize, color: Color) {
    WRITER.lock().draw_rect(x, y, width, height, color);
}

pub fn _cap_bmp(x: usize, y: usize, width: usize, height: usize) -> Bitmap {
    return WRITER.lock().capture_bmp(x, y, width, height);
}

pub fn _bmp(x: usize, y: usize, bmp: &Bitmap) {
    WRITER.lock().draw_bmp(x, y, &bmp);
}

pub fn _char(x: usize, y: usize, chr: ScreenChar) {
    WRITER.lock().draw_char(x, y, chr);
}

pub fn _fill_buffer(color: Color) {
    WRITER.lock().fill_buffer(color);
}

pub fn _apply() {
    WRITER.lock().apply();
}