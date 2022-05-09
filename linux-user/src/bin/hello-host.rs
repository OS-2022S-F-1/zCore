use std::env;
use keystone_rust_sdk::{EdgeCallHandler, Params, Enclave};

fn main() {
    // println!("Hello, world!");
    // let mut args = env::args();
    // assert_eq!(args.len(), 3);
    // let mut params = Params::new();
    // let mut enclave = Enclave::new();
    // let mut ret: usize = 0;
    // params.set_free_mem_size(1024 * 1024);
    // params.set_untrusted_mem(0xffffffff80000000, 1024 * 1024);
    //
    // enclave.init(&*args.nth(1).unwrap(), &*args.nth(2).unwrap(), params, 0);
    //
    // let mut handler = EdgeCallHandler::init_internals(enclave.get_shared_buffer() as usize, enclave.get_shared_buffer_size());
    // enclave.register_ocall_dispatch(handler.incoming_call_dispatch);
    // enclave.run(&mut ret);
}
