#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(titan_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::{boxed::Box, vec::Vec};
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use titan_os::{
    allocator::{self, HEAP_SIZE},
    memory,
};

entry_point!(main);

fn main(boot_info: &'static BootInfo) -> ! {
    titan_os::init();
    let (mut mapper, frame_allocator) = unsafe { memory::init(&boot_info) };
    allocator::init_heap(&mut mapper, frame_allocator).expect("Initialization failed");
    test_main();
    loop {}
}

#[test_case]
fn simple_allocation() {
    let heap_value_1 = Box::new(41);
    let heap_value_2 = Box::new(13);
    assert_eq!(*heap_value_1, 41);
    assert_eq!(*heap_value_2, 13);
}

#[test_case]
fn large_vec() {
    let n = 1000;
    let mut vec = Vec::new();
    for i in 0..n {
        vec.push(i);
    }
    assert_eq!(vec.iter().sum::<u64>(), (n - 1) * n / 2);
}

#[test_case]
fn many_boxes() {
    for i in 0..HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
}

#[cfg(not(bump_allocator))]
#[test_case]
fn many_boxes_long_lived() {
    let long = Box::new(1);

    for i in 0..HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }

    assert_eq!(*long, 1);
}

#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
    titan_os::test_panic_handler(panic_info)
}
