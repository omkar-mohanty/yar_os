#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(titan_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use titan_os::{
    allocator, memory, println,
    task::{executor::Executor, keyboard::print_keypresses, Task},
    BOOT_INFO,
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
    BOOT_INFO.init_once(|| boot_info);
    titan_os::init();
    let (mut mapper, frame_allocator) = unsafe { memory::init(&boot_info) };
    allocator::init_heap(&mut mapper, frame_allocator).expect("Heap initialization failed");
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
