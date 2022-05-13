#![no_std]
#![no_main]

#[macro_use]
extern crate linux_user;
extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

#[no_mangle]
fn main(args: Vec<String>) -> i32  {
    println!("Hello world from user!");
    0
}
