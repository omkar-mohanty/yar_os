#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(titan_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use titan_os::println;

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
    titan_os::init();
    println!("Hello World {}", "!");
    #[cfg(test)]
    test_main();
    loop {}
}
