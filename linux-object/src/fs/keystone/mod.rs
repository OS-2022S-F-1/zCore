mod ioctl;
mod enclave;
mod page;
mod enclave_manager;
mod sbi;
mod elf_loader;

use alloc::boxed::Box;
use async_trait::async_trait;
use alloc::sync::Arc;
use alloc::vec::Vec;
use lazy_static::lazy_static;
use kernel_hal::{MMUFlags, PAGE_SIZE, PhysAddr};
use rcore_fs::vfs::PollStatus;
use spin::Mutex;
use kernel_hal::mem::PhysFrame;

use zircon_object::{impl_kobject};
use zircon_object::object::{KObjectBase, KoID, Signal};
use zircon_object::vm::{VmAddressRegion, VmObject};

use crate::error::{LxError, LxResult};
use crate::fs::keystone::enclave_manager::modify_enclave_by_id;
use crate::fs::keystone::ioctl::ioctl;
use crate::fs::OpenFlags;
use super::FileLike;

/// Abstract fd for keystone driver
pub struct Keystone {
    base: KObjectBase,
    // path: String,
}

lazy_static! {
    /// global fd
    pub static ref KEYSTONE: Arc<Keystone> = {
        Arc::new(Keystone {
            base: KObjectBase::new(),
            // path: "/mnt/keystone".into()
        })
    };
}

pub struct MemoryRegion {
    // root_page_table: usize,
    // ptr: VirtAddr,
    pub size: usize,
    pub order: usize,
    pub pa: PhysAddr,
    pub frames: Vec<PhysFrame>
}

pub struct EnclaveParams {
    pt_ptr: usize,
    runtime_paddr: usize,
    user_paddr: usize,
    free_paddr: usize
}

pub struct Enclave {
    eid: isize,
    // close_on_pexit: i32,
    epm: Arc<Mutex<MemoryRegion>>, // enclave private memory
    utm: Arc<Mutex<MemoryRegion>>, // untrusted share page
    vmar: Arc<VmAddressRegion>,
    params: EnclaveParams,
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
            // path: "/mnt/keystone".into()
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

    fn get_vmo(&self, _offset: usize, _len: usize) -> LxResult<Arc<VmObject>> {
        Err(LxError::EINVAL)
    }
}

impl Keystone {
    pub fn mmap(
        &self,
        addr: Option<usize>,
        len: usize,
        _offset: usize,
        user: bool
    ) -> LxResult<Arc<VmObject>> {
        let enclave_id = len >> 48;
        let align_len = (len & ((1 << 48) - 1)) / PAGE_SIZE;
        // let offset = offset / PAGE_SIZE;
        modify_enclave_by_id(enclave_id, |enclave| {
            let mut memory = enclave.utm.lock();
            let alloc_frames = memory.alloc(align_len).unwrap();
            // let alloc_frames: Vec<PhysFrame> = Vec::from(&memory.frames[offset..(offset + align_len)]);
            drop(memory);
            warn!("Begin to keystone map from {:x}...", alloc_frames[0].paddr);
            let vmo = VmObject::new_with_frames(align_len, alloc_frames);
            let mmu_flags = if user { MMUFlags::USER | MMUFlags::READ | MMUFlags::WRITE } else { MMUFlags::READ | MMUFlags::WRITE };
            enclave.vmar.map(addr, vmo.clone(), 0, vmo.len(), mmu_flags)?;
            warn!("Keystone map success!");
            Ok(vmo)
        })
    }
}

impl Drop for Keystone {
    fn drop(&mut self) {

    }
}
