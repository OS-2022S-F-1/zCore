use alloc::vec::Vec;
use hashbrown::HashMap;
use spin::RwLock;
use crate::error::{LxError, LxResult};
use crate::fs::keystone::Enclave;

const ENCLAVE_IDR_MIN: usize = 0x1000;
const ENCLAVE_IDR_MAX: usize = 0xffff;

struct EnclaveManagerInner {
    avail: Vec<usize>,
    enclave_map: HashMap<usize, Enclave>
}

pub struct EnclaveManager {
    inner: RwLock<EnclaveManagerInner>
}

impl EnclaveManager {
    pub fn new() -> Self {
        let mut avail: Vec<usize> = Vec::new();
        (ENCLAVE_IDR_MIN..ENCLAVE_IDR_MAX).for_each(|id| { avail.push(id); } );
        EnclaveManager {
            inner: RwLock::new(EnclaveManagerInner {
                avail,
                enclave_map: HashMap::new()
            })
        }
    }
}

impl EnclaveManagerInner {
    pub fn get_enclave_sbi_eid(&self, id: usize) -> LxResult<isize> {
        if let Some(enclave) = self.enclave_map.get(&id) {
            Ok(enclave.eid)
        } else {
            Err(LxError::EINVAL)
        }
    }

    pub fn modify_enclave_by_id<F, T>(&mut self, id: usize, mut f: F) -> LxResult<T>
        where
            F: FnMut(&mut Enclave) -> LxResult<T>, {
        if let Some(enclave) = self.enclave_map.get_mut(&id) {
            f(enclave)
        } else {
            Err(LxError::EINVAL)
        }
    }

    pub fn alloc(&mut self, enclave: Enclave) -> Option<usize> {
        if let Some(id) = self.avail.pop() {
            self.enclave_map.insert(id, enclave);
            Some(id)
        } else {
            None
        }
    }

    pub fn remove(&mut self, id: usize) {
        self.enclave_map.remove(&id).unwrap();
    }
}

lazy_static::lazy_static! {
    pub static ref ENCLAVE_MANAGER: EnclaveManager = EnclaveManager::new();
}

pub fn get_enclave_sbi_eid(id: usize) -> LxResult<isize> {
    ENCLAVE_MANAGER.inner.read().get_enclave_sbi_eid(id)
}

pub fn modify_enclave_by_id<F, T>(id: usize, f: F) -> LxResult<T>
    where
        F: FnMut(&mut Enclave) -> LxResult<T>, {
    ENCLAVE_MANAGER.inner.write().modify_enclave_by_id(id, f)
}

pub fn alloc(enclave: Enclave) -> Option<usize> {
    ENCLAVE_MANAGER.inner.write().alloc(enclave)
}

pub fn remove_by_id(id: usize) {
    ENCLAVE_MANAGER.inner.write().remove(id)
}

