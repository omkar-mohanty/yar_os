#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(titan_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use titan_os::{exit_qemu, println, test_panic_handler, QemuExitStatus};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    exit_qemu(QemuExitStatus::Success);
    loop {}
}

#[test_case]
fn test_println() {
    println!("test_println output");
}

#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}
