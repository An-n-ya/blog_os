#![no_std]
#![no_main]

mod vga_buffer;

use core::panic::PanicInfo;

static HELLO: &[u8] = b"Hello World!";

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // do nothing
    println!("{}", _info);
    loop {}
}

// 1. without this no_mangle attribute,
// the Compiler would generate some wired cryptic name like: _ZN3blog_os4_start7hb173fedf945531caE
// to give every function an unique name
// with this no_mangle attribute, we tell the compiler do not change this function name
// we need this name to be the entry point
// 2. we need `extern C`, because we want the compiler to use the C calling convention
// 3. the return type ! means do not allow return
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // let vga_buffer = 0xb8000 as *mut u8;
    // for (i, &byte) in HELLO.iter().enumerate() {
    //     unsafe {
    //         *vga_buffer.offset(i as isize * 2) = byte;
    //         *vga_buffer.offset(i as isize * 2 + 1) = 0xb; // set color to cyan
    //     }
    // }
    use core::fmt::Write;
    vga_buffer::WRITER.lock().write_str("Hello world!").unwrap();
    write!(vga_buffer::WRITER.lock(), ", some numbers: {} {}\n", 42, 3.14).unwrap();
    println!("Hello Again{}", "!");
    panic!("some panic message");
    loop {}

    // instead of return, here should be a exit syscall
}