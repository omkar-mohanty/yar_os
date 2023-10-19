#![no_std]
#![no_main]
 #![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;
use lazy_static::lazy_static;
use titan_os::{serial_print, exit_qemu};
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

lazy_static! {
    static ref TEST_IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            idt.double_fault
                .set_handler_fn(test_double_fault_handler)
                .set_stack_index(titan_os::gdt::DOUBLE_FAULT_INDEX);
        }

        idt
    };
}

extern "x86-interrupt" fn test_double_fault_handler(_interrupt_frame: InterruptStackFrame, _ec:u64) -> ! {
    serial_print!("[OK]\n");
    exit_qemu(titan_os::QemuExitStatus::Success);
    loop {
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    serial_print!("stack overflow::stack_overflow...\t");
    titan_os::gdt::init();
    init();
    stack_overflow();
    panic!("Double fault handler not called");
    loop {
        
    }
}

fn init() {
    TEST_IDT.load();
}

#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow();
    volatile::Volatile::new(0).read();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    titan_os::test_panic_handler(info)
}
