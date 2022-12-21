#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // do nothing
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
    loop{}
    // instead of return, here should be a exit syscall
}