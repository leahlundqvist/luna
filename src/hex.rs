use crate::{println, print};
use alloc::{vec::Vec};

pub fn char_to_hex(c: char) -> u64 {
    match c {
        '0' => 0,
        '1' => 1,
        '2' => 2,
        '3' => 3,
        '4' => 4,
        '5' => 5,
        '6' => 6,
        '7' => 7,
        '8' => 8,
        '9' => 9,
        'a' => 10,
        'b' => 11,
        'c' => 12,
        'd' => 13,
        'e' => 14,
        'f' => 15,
        'A' => 10,
        'B' => 11,
        'C' => 12,
        'D' => 13,
        'E' => 14,
        'F' => 15,
        _ => 0
    }
}

pub fn char_hex_vec_to_int(chars: Vec<char>) -> u64 {
    let mut value = 0;

    for i in 0..chars.len() {
        let mut pow = 16 as u64;

        if i == chars.len() - 1 {
            pow = 1;
        } else {
            for j in (0..(chars.len() - i - 2)) {
                pow = pow * 16;
            }
        }
        
        value = value + (char_to_hex(chars[i]) * pow);
    }

    return value;
}
