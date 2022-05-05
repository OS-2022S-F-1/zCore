mod ioctl;
mod enclave;
mod page;
mod enclave_manager;
mod sbi;

use alloc::sync::Arc;
use lazy_static::lazy_static;
use kernel_hal::{PhysAddr, VirtAddr};
use rcore_fs::vfs::PollStatus;
use zcore_drivers::prelude::CapabilityType::Key;
use zircon_object::impl_kobject;
use zircon_object::vm::{VmAddressRegion, VmObject};
use crate::error::{LxError, LxResult};
use crate::fs::keystone::enclave_manager::get_enclave_by_id;
use crate::fs::keystone::ioctl::ioctl;
use crate::fs::OpenFlags;
use super::FileLike;

struct Keystone;

lazy_static! {
    pub static ref KEYSTONE: Arc<Keystone> = {
        Arc::new(Keystone {})
    };
}

struct Epm {
    // root_page_table: usize,
    // ptr: VirtAddr,
    pub size: usize,
    pub order: usize,
    pub pa: PhysAddr,
    pub vmo: Arc<VmObject>
}

struct Utm {
    // root_page_table: usize,
    // ptr: VirtAddr,
    pub size: usize,
    pub order: usize,
    pub pa: PhysAddr,
    pub vmo: Arc<VmObject>
}

struct Enclave {
    eid: usize,
    close_on_pexit: i32,
    utm: Option<Utm>, // untrusted share page
    epm: Option<Epm>, // enclave private memory
    is_init: bool
}

impl_kobject!(Keystone);

impl FileLike for Keystone {
    fn flags(&self) -> OpenFlags {
        OpenFlags::RDWR
    }

    fn set_flags(&self, f: OpenFlags) -> LxResult {
        Ok(())
    }

    fn dup(&self) -> Arc<dyn FileLike> {
        Arc::new(Self {
        })
    }

    async fn read(&self, buf: &mut [u8]) -> LxResult<usize> {
        Ok(0)
    }

    fn write(&self, buf: &[u8]) -> LxResult<usize> {
        Ok(0)
    }

    async fn read_at(&self, offset: u64, buf: &mut [u8]) -> LxResult<usize> {
        Ok(0)
    }

    fn write_at(&self, offset: u64, buf: &[u8]) -> LxResult<usize> {
        Ok(0)
    }

    fn poll(&self) -> LxResult<PollStatus> {
        Ok(PollStatus{
            read: false,
            write: false,
            error: false
        })
    }

    async fn async_poll(&self) -> LxResult<PollStatus> {
        Ok(PollStatus{
            read: false,
            write: false,
            error: false
        })
    }

    fn ioctl(&self, request: usize, arg1: usize, arg2: usize, arg3: usize) -> LxResult<usize> {
        ioctl(request.into(),arg1.into())
    }

    fn get_vmo(&self, offset: usize, len: usize) -> LxResult<Arc<VmObject>> {
        let enclave_id = len >> 48;
        let len = len & 0xffffffffff;
        if let Some(enclave) = get_enclave_by_id(enclave_id) {
            let vmo = if enclave.is_init {
                enclave.epm?.vmo
            } else {
                enclave.utm?.vmo
            };
            if let Ok(child) = vmo.create_child(false, offset, len) {
                Ok(child)
            } else {
                LxError::EINVAL
            }
        } else {
            LxError::EINVAL
        }
    }
}

impl Drop for Keystone {
    fn drop(&mut self) {

    }
}
