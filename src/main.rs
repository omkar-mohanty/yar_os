#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(titan_os::test_runner)]

use core::panic::PanicInfo;
use titan_os::{println, QemuExitStatus, exit_qemu};

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use titan_os::test_panic_handler;
    test_panic_handler(info)
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World {}", "!");
    #[cfg(test)]
    exit_qemu(QemuExitStatus::Success);
    loop {}
}
