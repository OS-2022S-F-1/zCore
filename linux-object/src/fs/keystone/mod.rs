mod ioctl;
mod enclave;
mod page;
mod enclave_manager;
mod sbi;

use alloc::boxed::Box;
use alloc::string::String;
use async_trait::async_trait;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::mem::align_of;
use lazy_static::lazy_static;
use kernel_hal::{PAGE_SIZE, PhysAddr};
use rcore_fs::vfs::PollStatus;
use kernel_hal::mem::PhysFrame;

use zircon_object::impl_kobject;
use zircon_object::object::{KObjectBase, KoID, Signal};
use zircon_object::vm::{VmObject};

use crate::error::{LxResult};
use crate::fs::keystone::enclave_manager::modify_enclave_by_id;
use crate::fs::keystone::ioctl::ioctl;
use crate::fs::OpenFlags;
use super::FileLike;

/// Abstract fd for keystone driver
pub struct Keystone {
    base: KObjectBase,
    path: String,
}

lazy_static! {
    /// global fd
    pub static ref KEYSTONE: Arc<Keystone> = {
        Arc::new(Keystone {
            base: KObjectBase::new(),
            path: "/mnt/keystone".into()
        })
    };
}

struct Epm {
    // root_page_table: usize,
    // ptr: VirtAddr,
    pub size: usize,
    pub order: usize,
    pub pa: PhysAddr,
    pub frames: Vec<PhysFrame>
}

struct Utm {
    // root_page_table: usize,
    // ptr: VirtAddr,
    pub size: usize,
    pub order: usize,
    pub pa: PhysAddr,
    pub frames: Vec<PhysFrame>
}

pub struct Enclave {
    eid: isize,
    close_on_pexit: i32,
    utm: Utm, // untrusted share page
    epm: Epm, // enclave private memory
    is_init: bool
}

impl_kobject!(Keystone);

#[async_trait]
impl FileLike for Keystone {
    fn flags(&self) -> OpenFlags {
        OpenFlags::RDWR
    }

    fn set_flags(&self, _f: OpenFlags) -> LxResult {
        Ok(())
    }

    fn dup(&self) -> Arc<dyn FileLike> {
        Arc::new(Keystone {
            base: KObjectBase::new(),
            path: "/mnt/keystone".into()
        })
    }

    async fn read(&self, _buf: &mut [u8]) -> LxResult<usize> {
        Ok(0)
    }

    fn write(&self, _buf: &[u8]) -> LxResult<usize> {
        Ok(0)
    }

    async fn read_at(&self, _offset: u64, _buf: &mut [u8]) -> LxResult<usize> {
        Ok(0)
    }

    fn write_at(&self, _offset: u64, _buf: &[u8]) -> LxResult<usize> {
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

    fn ioctl(&self, request: usize, arg1: usize, _arg2: usize, _arg3: usize) -> LxResult<usize> {
        ioctl(request.into(),arg1.into())
    }

    fn get_vmo(&self, offset: usize, len: usize) -> LxResult<Arc<VmObject>> {
        info!("Call keystone mmap!");
        let enclave_id = len >> 48;
        let align_len = len & ((1 << 48) - 1);
        let offset = offset / PAGE_SIZE;
        modify_enclave_by_id(enclave_id, |enclave| {
            let frames = if enclave.is_init {
                enclave.epm.frames.as_slice()
            } else {
                enclave.utm.frames.as_slice()
            };
            let mut alloc_frames: Vec<PhysFrame> = Vec::from(&frames[offset..(offset + align_len)]);
            Ok(VmObject::new_with_frames(align_len, alloc_frames))
        })
    }
}

impl Drop for Keystone {
    fn drop(&mut self) {

    }
}
