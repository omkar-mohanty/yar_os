#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(titan_os::test_runner)]

use titan_os::println;
use core::panic::PanicInfo;


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
    loop {}
}
