#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(titan_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::boxed::Box;
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use titan_os::{
    allocator,
    memory,
    println,
    task::{executor::Executor, keyboard::print_keypresses, Task},
};
use x86_64::{
    structures::paging::{Page, Translate},
    VirtAddr,
};

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

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    titan_os::init();
    let phys_memory_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_memory_offset) };
    let mut frame_allocator =
        unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("Heap initialization failed");
    #[cfg(test)]
    test_main();

    titan_os::drivers::init();
    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(print_keypresses()));
    executor.run();
}

async fn example_number() -> u32 {
    42
}

async fn example_task() {
    let number = example_number().await;
    println!("async number{}", number);
}
