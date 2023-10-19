#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
use core::panic::PanicInfo;

use titan_os::{exit_qemu, serial_println, QemuExitStatus};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    trivial_assert();
    exit_qemu(QemuExitStatus::Failure);
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[OK]");
    exit_qemu(QemuExitStatus::Success);
    loop {}
}

fn trivial_assert() {
    serial_println!("Should Panic");
    assert_eq!(0, 1);
}
