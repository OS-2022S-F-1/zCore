#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]

use buddy_system_allocator::LockedHeap;

#[global_allocator]
static HEAP: LockedHeap = LockedHeap::empty();
use keystone_rust_sdk::{ocall, eapp_ret};

#[no_mangle]
fn main() -> ! {
    let data = [0u8; 32];
    let mut ret  = [0u8; 32];
    ocall(8, data.as_ptr(), 32, ret.as_mut_ptr(), 32);
    eapp_ret(0)
}

#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
