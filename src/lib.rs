#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(asm_const)]

extern crate alloc;

pub mod allocator;
pub mod drivers;
pub mod gdt;
pub mod interrupts;
pub mod memory;
pub mod serial;
pub mod task;
pub mod vga_buffer;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;

pub trait Testable {
    fn run(&self);
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[OK]");
    }
}

pub fn init() {
    gdt::init();
    interrupts::init_idt();
    unsafe {
        interrupts::PICS.lock().initialize();
    }
    x86_64::instructions::interrupts::enable();
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }

    exit_qemu(QemuExitStatus::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error : {}\n", info);
    exit_qemu(QemuExitStatus::Failure);
    loop {}
}

#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum QemuExitStatus {
    Failure = 0x11,
    Success = 0x10,
}

pub fn exit_qemu(exit_code: QemuExitStatus) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[test_case]
fn trivial_test() {
    assert_eq!(1, 1);
}

#[cfg(test)]
entry_point!(kernel_main);

#[cfg(test)]
fn kernel_main(_boot_info: &'static BootInfo) -> ! {
    init();
    test_main();
    serial_println!("Finished tests");
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}
