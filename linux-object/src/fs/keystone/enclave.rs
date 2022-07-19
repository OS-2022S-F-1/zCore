use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::Mutex;
use zircon_object::vm::VmAddressRegion;
use crate::fs::keystone::{EnclaveParams, MemoryRegion};
use crate::fs::keystone::page::EnclavePageTable;
use super::{Enclave};


impl EnclaveParams {
    pub fn empty() -> Self {
        Self {
            pt_ptr: 0,
            utm_free_ptr: 0,
            runtime_paddr: 0,
            user_paddr: 0,
            free_paddr: 0
        }
    }
}

impl Enclave {
    pub fn new(min_pages: usize) -> Self {
        let epm = Arc::new(Mutex::new(MemoryRegion::new(min_pages)));
        Enclave {
            eid: -1,
            close_on_pexit: 1,
            utm: Arc::new(Mutex::new(MemoryRegion {
                // root_page_table: epm_vaddr,
                // ptr: epm_vaddr,
                size: 0,
                order: 0,
                pa: 0,
                frames: Vec::new()
            })),
            epm: epm.clone(),
            vmar:VmAddressRegion::new_root_with_pt(Arc::new(Mutex::new(EnclavePageTable::new(epm.clone())))),
            params: EnclaveParams::empty(),
            is_init: true
        }
    }
}

impl Drop for Enclave {
    fn drop(&mut self) {

    }
}

