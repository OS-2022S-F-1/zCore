use alloc::sync::Arc;
use zircon_object::task::Process;
use super::{Enclave, Epm, Utm};

const ENCLAVE_IDR_MIN: usize = 0x1000;
const ENCLAVE_IDR_MAX: usize = 0xffff;

impl Enclave {
    pub fn new(min_pages: usize) -> Self {
        Enclave {
            eid: -1,
            close_on_pexit: 1,
            utm: None,
            epm: Some(Epm::new(min_pages)),
            is_init: true
        }
    }
}

impl Drop for Enclave {
    fn drop(&mut self) {

    }
}

