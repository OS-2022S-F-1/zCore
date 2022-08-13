#![no_std]
#![no_main]

extern crate linux_user;
extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use keystone_rust_sdk::{ocall, eapp_ret};

#[no_mangle]
fn main(_args: Vec<String>) -> i32 {
    let data = [0u8; 32];
    let mut ret  = [0u8; 32];
    ocall(8, data.as_ptr(), 32, ret.as_mut_ptr(), 32);
    eapp_ret(0);
}

