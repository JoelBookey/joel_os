#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(joel_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use joel_os::println;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("hello");

    joel_os::init();

    #[cfg(test)]
    test_main();
    joel_os::snake::run();
    joel_os::hlt_loop();
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    joel_os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    joel_os::test_panic_handler(info)
}
