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
    params.set_free_mem_size(1024 * 1024);
    params.set_untrusted_mem(0xffffffff80000000, 1024 * 1024);
    println!("Set size successfully!");
    let elf_data = ElfData::new(args.get(1).unwrap().as_str(), args.get(2).unwrap().as_str());
    println!("Get elf data successfully!");
    let mut enclave = Enclave::new(&elf_data, params, 0).unwrap();

    // let mut handler = EdgeCallHandler::init_internals(enclave.get_shared_buffer() as usize, enclave.get_shared_buffer_size());
    // enclave.register_ocall_handler(handler);
    // enclave.run(&mut ret);
    0
}
