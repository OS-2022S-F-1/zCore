use super::{Enclave, Epm, Utm};


impl Enclave {
    pub fn new(min_pages: usize) -> Self {
        let epm = Epm::new(min_pages);
        Enclave {
            eid: -1,
            close_on_pexit: 1,
            utm: Utm {
                // root_page_table: epm_vaddr,
                // ptr: epm_vaddr,
                size: 0,
                order: 0,
                pa: 0,
                vmo: epm.vmo.clone()
            },
            epm,
            is_init: true
        }
    }
}

impl Drop for Enclave {
    fn drop(&mut self) {

    }
}

