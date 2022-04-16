use alloc::sync::Arc;
use alloc::vec::Vec;
use hashbrown::HashMap;
use crate::fs::keystone::Enclave;

const ENCLAVE_IDR_MIN: usize = 0x1000;
const ENCLAVE_IDR_MAX: usize = 0xffff;

struct EnclaveManagerInner {
    avail: Vec<usize>,
    enclave_map: HashMap<usize, Arc<Enclave>>
}

struct EnclaveManager {
    inner: Arc<EnclaveManagerInner>
}

impl EnclaveManager {
    pub fn new() -> Self {
        let mut avail: Vec<usize> = Vec::new();
        (ENCLAVE_IDR_MIN..ENCLAVE_IDR_MAX).for_each(|id| { avail.push(id); } );
        EnclaveManager {
            inner: Arc::new {
                avail,
                enclave_map: HashMap::new()
            }
        }
    }

    pub fn get_enclave_by_id(&self, id: usize) -> Option<Arc<Enclave>> {
        if let Some(enclave) = self.inner.enclave_map.get(&id) {
            Some(enclave.clone())
        } else {
            None
        }
    }

    pub fn alloc(&self, enclave: Arc<Enclave>) -> Option<usize> {
        if let Some(id) = self.inner.avail.pop() {
            self.enclave_map.insert(id, enclave);
            Some(id)
        } else {
            None
        }
    }

    pub fn remove(&self, id: usize) {
        self.inner.enclave_map.remove(&id);
    }
}

lazy_static::lazy_static! {
    pub static ref ENCLAVE_MANAGER: EnclaveManager = EnclaveManager::new();
}

pub fn get_enclave_by_id(id: usize) -> Option<Arc<Enclave>> {
    ENCLAVE_MANAGER.get_enclave_by_id(id)
}

pub fn alloc(enclave: Arc<Enclave>) -> Option<usize> {
    ENCLAVE_MANAGER.alloc(enclave)
}

pub fn remove_by_id(id: usize) {
    ENCLAVE_MANAGER.remove(id)
}

