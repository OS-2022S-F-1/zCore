use std::env;
use keystone_rust_sdk::{EdgeCallHandler, Params, Enclave};

fn main() {
    let mut args = env::args();
    assert_eq!(args.len(), 3);
    let mut params = Params::new();
    let mut enclave = Enclave::new();

    params.set_free_mem_size(1024 * 1024);
    params.set_untrusted_mem(0xffffffff80000000, 1024 * 1024);

    enclave.init(args.nth(1).unwrap(), args.nth(2), params, 0);

    let mut handler = EdgeCallHandler::init_internals(enclave.get_shared_buffer(), enclave.get_shared_buffer_size());
    enclave.register_ocall_dispatch(handler.incoming_call_dispatch);

    enclave.run();
}
