#![no_std]
#![cfg(not(feature = "std"))]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(luna::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::{boxed::Box, rc::Rc, vec, vec::Vec};
use luna::{
    println,
    print,
    raw_pixel,
    rect,
    bmp,
    draw_char,
    color,
    fill_buffer,
    cap_bmp,
    vga_apply,
    vga_buffer::Color,
    vga_buffer::Bitmap,
    vga_buffer::ScreenChar,
    hex::char_hex_vec_to_int,
    LUSHKeyHandler,
    LUSHAddCommand,
    lush_keypush,
    lush_keypop,
    lure_enabled,
    lure_bmp,
    shell::LunaRenderer,
    shell::LunaLine
};
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use lazy_format::lazy_format;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use luna::allocator;
    use luna::memory::{self, BootInfoFrameAllocator};
    use x86_64::VirtAddr;

    luna::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");
    
    fn memchk_handler(args: Vec<char>) {
        println!("args@{:p}", args.as_slice());
        println!("newV@{:p}", vec![args.len()].as_slice());      
        println!("boxV@{:p}", Box::into_raw(Box::new(args)));      
    }
    LUSHAddCommand!(vec!['m', 'e', 'm', 'c', 'h', 'k'], memchk_handler);

    fn set_handler(args: Vec<char>) {
        let set_value = Box::into_raw(Box::new(args));
        print!("address: {:p}", set_value); 
    }
    LUSHAddCommand!(vec!['s', 'e', 't'], set_handler);

    fn get_handler(args: Vec<char>) {
        let addr = char_hex_vec_to_int(args);
        let charvec = unsafe { &mut *(addr as *mut Vec<char>) };
        for i in charvec {
            print!("{}", i)
        }
    }
    LUSHAddCommand!(vec!['g', 'e', 't'], get_handler);

    fn hex_handler(args: Vec<char>) {
        print!("{}", char_hex_vec_to_int(args));
    }
    LUSHAddCommand!(vec!['h', 'e', 'x'], hex_handler);
    
    fn echo_handler(args: Vec<char>) {
        // print each character in the remaining string.
        for i in args {
            print!("{}", i);
        }
    }
    LUSHAddCommand!(vec!['e', 'c', 'h', 'o'], echo_handler);

    fn lusc_handler(args: Vec<char>) {
        // print each character in the remaining string.
        let mut cells = vec![0 as i32];
        let mut iters = 0;
        let mut i = 0;
        let mut j = 0;
        let mut jump = 0;
        let mut jumpToNextBrack = false;
        let mut done = false;
        let mut charsPrinted = 0;
        let mut numbers: Vec<u8> = Vec::new();

        while !done {
            if i >= args.len() {
                done = true;
                break;
            }
            if iters > 1000000 {
                color!(Color::LightRed);
                println!("max iterations reached.");
                color!(Color::LightGray);
                done = false;
                break;
            }
            if jumpToNextBrack {
                if args[i] == ']' {
                    jumpToNextBrack = false;
                }
            } else {
                if args[i] as u8 >= b'0' && args[i] as u8 <= b'9' {
                    numbers.push(args[i] as u8 - b'0');
                } else {
                    let mut opCount = 0;
                    for i in 0..numbers.len() {
                        let mut pow = 10 as i32;

                        if i == numbers.len() - 1 {
                            pow = 1;
                        } else {
                            for j in (0..(numbers.len() - i - 2)) {
                                pow = pow * 10;
                            }
                        }
                        
                        opCount = opCount + (numbers[i] as i32 * pow);
                    }
                    if numbers.len() == 0 {
                        opCount = 1;
                    }

                    numbers = Vec::new();

                    match args[i] {
                        '>' => {
                            for i in 0..opCount {
                                if j + 1 >= cells.len() { cells.push(0 as i32); }
                                j = j + 1;
                            }
                        },
                        '<' => {
                            for i in 0..opCount {
                                if j - 1 >= 0 { j = j - 1; }
                            }
                        },
                        '+' => {
                            cells[j] = cells[j] + opCount;
                        },
                        '-' => {
                            cells[j] = cells[j] - opCount;
                        },
                        '.' => {
                            for i in 0..opCount {
                                print!("{}", (cells[j] as u8) as char);
                                charsPrinted = charsPrinted + 1;
                            }
                        },
                        '[' => {
                            jump = i;
                            if cells[j] == 0 {
                                jumpToNextBrack = true;
                            }
                        },
                        ']' => {
                            if cells[j] != 0 {
                                i = jump;
                            }
                        },
                        _ => {}
                    }
                }
            }

            i = i + 1;
            iters = iters + 1;
        }

        if charsPrinted > 0 {
            print!("\n");
        }

        print!("[ ");
        for cell in cells {
            print!("{} ", cell);
        }
        print!("]");
    }
    LUSHAddCommand!(vec!['l', 'u', 's', 'c'], lusc_handler);


    fn keyown_key_handler(key: char) {
        if key == '\u{001b}' {
            rect!(0,0,319,199,Color::Black);
            lush_keypop!();
            lure_enabled!(true);
        }
        vga_apply!();
    }
    fn keyown_handler(args: Vec<char>) {
        rect!(0,0,319,199,Color::DarkGray);
        vga_apply!();
        lush_keypush!(keyown_key_handler);
        lure_enabled!(false);
    }
    LUSHAddCommand!(vec!['k', 'e', 'y', 'o', 'w', 'n'], keyown_handler);

    fn colors_handler(args: Vec<char>) {
        fill_buffer!(Color::Black);
        for y in 0..24 {
            for x in 0..24 {
                raw_pixel!(x * 2, y * 2, (24 * y + x) as u8);
            }
        }
        
        lure_bmp!(cap_bmp!(0,0,319,8));
        lure_bmp!(cap_bmp!(0,8,319,8));
        lure_bmp!(cap_bmp!(0,16,319,8));
        lure_bmp!(cap_bmp!(0,24,319,8));
        lure_bmp!(cap_bmp!(0,32,319,8));
        lure_bmp!(cap_bmp!(0,48,319,8));
    }
    LUSHAddCommand!(vec!['c', 'o', 'l', 'o', 'r', 's'], colors_handler);

    fn color_handler(args: Vec<char>) {
        fill_buffer!(Color::Black);

        let c = char_hex_vec_to_int(args) as u8;
        
        for y in 0..8 {
            for x in 0..319 {
                raw_pixel!(x, y, c);
            }
        }

        lure_bmp!(cap_bmp!(0,0,48,8));
    }
    LUSHAddCommand!(vec!['c', 'o', 'l', 'o', 'r'], color_handler);

    fn edit_handler(args: Vec<char>) {
    
    }
    LUSHAddCommand!(vec!['e', 'd', 'i', 't'], edit_handler);


    color!(Color::Pink);
    print!("Luna ");
    color!(Color::LightRed);
    println!("v0.2");
    color!(Color::Yellow);
    println!("rustc 1.41.0-nightly");

    color!(Color::LightBlue);
    LUSHKeyHandler!('\u{0000}');

    luna::hlt_loop();
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    luna::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    luna::test_panic_handler(info)
}
