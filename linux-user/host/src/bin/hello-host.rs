#![no_std]
#![no_main]

#[macro_use]
extern crate linux_user;
extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use keystone_rust_sdk::{EdgeCallHandler, Params, ElfData, Enclave};

#[no_mangle]
fn main(args: Vec<String>) -> i32 {
    println!("Hello world from host!");
    assert_eq!(args.len(), 3);
    let mut params = Params::new();
    let mut ret: usize = 0;
    let elf_data = ElfData::new(args.get(1).unwrap().as_str(), args.get(2).unwrap().as_str());
    let mut enclave = Enclave::new(&elf_data, params, 0).unwrap();
    let mut handler = EdgeCallHandler::init_internals(enclave.get_shared_buffer() as usize, enclave.get_shared_buffer_size());
    handler.register_call(8, move |_: *mut u8| {
        println!("ocall from enclave!");
    });
    enclave.register_ocall_handler(handler);
    println!("Prepared to run...");
    enclave.run(&mut ret);
    println!("Enclave task completed!");
    0
}
