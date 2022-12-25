#![no_std]
#![no_main]

#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

mod vga_buffer;
mod serial;

use core::panic::PanicInfo;
use core::arch::asm;

static HELLO: &[u8] = b"Hello World!";

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn()
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        // execute each test
        test.run();
    }

    // exit qemu
    exit_qemu(QemuExitCode::Success)

    // qemu will exit with (value<<1)|1, but any exit code other than 0 will be considered failure by cargo test
    // therefore, we need to define our own success exit code in cargo.toml
}


#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}

#[test_case]
fn cpuid_test() {
    let mut eax: u64;
    let mut edx: u64;
    serial_print!("\x1b[34m");
    unsafe {
        asm!(
            "cpuid",
            inout("eax") 0 as u64 => eax // eax同时作为输入和输出，初始化为0，再把eax结果保存到rust的eax
        )
    }
    serial_println!("\n[cpuid] maximum basic function: {:#X}", eax);
    unsafe {
        asm!(
        "cpuid",
        inout("eax") 0x80000000 as u64 => eax
        )
    }
    // 应该是 0x8000000A
    serial_println!("[cpuid] maximum extended function: {:#X}", eax);
    unsafe {
        asm!(
        "cpuid",
        inout("eax") 1 as u64 => eax
        )
    }
    serial_println!("[cpuid] family, model, stepping: {:#X}", eax);

    // 测试long mode
    unsafe {
        asm!(
        "cpuid",
        in("eax") 0x80000001 as u64,
        out("edx") edx
        )
    }
    if 0x20000000 & edx == 0x20000000 {
        serial_println!("[cpuid] long mode supported");
    } else {
        serial_println!("[cpuid] long mode not supported!, {:#X}", edx)
    }
    serial_print!("\x1b[0m");
}

#[cfg(test)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // do nothing
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", _info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // do nothing
    println!("{}", _info);
    loop {}
}

// when write a value to the I/O port specified by iobase, it cause QEMU exit with (value << 1) | 1
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
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
    // panic!("some panic message");

    #[cfg(test)]
    test_main();

    loop {}

    // instead of return, here should be a exit syscall
}