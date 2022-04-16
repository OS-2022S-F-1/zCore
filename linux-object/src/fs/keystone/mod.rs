mod ioctl;
mod enclave;
mod page;
mod enclave_manager;
mod sbi;

use alloc::sync::Arc;
use kernel_hal::{PhysAddr, VirtAddr};
use rcore_fs::vfs::PollStatus;
use zircon_object::impl_kobject;
use zircon_object::vm::{VmAddressRegion, VmObject};
use crate::error::LxResult;
use crate::fs::keystone::ioctl::ioctl;
use crate::fs::OpenFlags;
use super::FileLike;

pub struct Keystone;

struct Epm {
    root_page_table: usize,
    ptr: VirtAddr,
    size: usize,
    order: usize,
    pa: PhysAddr,
    vmar: Arc<VmAddressRegion>
}

struct Utm {
    root_page_table: usize,
    ptr: VirtAddr,
    size: usize,
    order: usize,
    pa: PhysAddr,
    vmar: Arc<VmAddressRegion>
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

    fn ioctl(&self, request: usize, vmar: Arc<VmAddressRegion>, arg1: usize, arg2: usize, arg3: usize) -> LxResult<usize> {
        ioctl(request.into(),arg1.into(), arg2.into(), vmar)
    }

    fn get_vmo(&self, offset: usize, len: usize) -> LxResult<Arc<VmObject>> {
        todo!()
    }
}

impl Drop for Keystone {
    fn drop(&mut self) {

    }
}