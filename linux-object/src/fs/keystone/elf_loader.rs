use alloc::sync::Arc;
use spin::Mutex;
use xmas_elf::{
    program::{Flags, ProgramHeader, SegmentData, Type},
    sections::SectionData,
    symbol_table::{DynEntry64, Entry},
    ElfFile,
};
use kernel_hal::PhysAddr;
use zircon_object::util::elf_loader::FlagsExt;
use zircon_object::vm::{PAGE_SIZE, pages, VmAddressRegion, VmObject};
use crate::error::{LxError, LxResult};
use crate::fs::keystone::{MemoryRegion};

pub trait EnclaveVmar {
    /// Create `VMObject` from all LOAD segments of `elf` and map them to this VMAR.
    /// Return the first `VMObject`.
    fn load_elf_to_epm(&self, elf: &ElfFile, epm: Arc<Mutex<MemoryRegion>>) -> LxResult<PhysAddr>;
}

impl EnclaveVmar for VmAddressRegion {
    fn load_elf_to_epm(&self, elf: &ElfFile, epm: Arc<Mutex<MemoryRegion>>) -> LxResult<PhysAddr> {
        let mut first_vmo = None;
        let mut first_paddr = None;
        for ph in elf.program_iter() {
            if ph.get_type().unwrap() != Type::Load {
                continue;
            }
            todo!(完成对于物理地址的逻辑);
            let (vmo, paddr) = make_vmo(elf, ph, epm.clone())?;
            let offset = ph.virtual_addr() as usize / PAGE_SIZE * PAGE_SIZE;
            let flags = ph.flags().to_mmu_flags();
            trace!("ph:{:#x?}, offset:{:#x?}, flags:{:#x?}", ph, offset, flags);
            //映射vmo物理内存块到 VMAR
            self.map_at(offset, vmo.clone(), 0, vmo.len(), flags)?;
            debug!("Map [{:x}, {:x})", offset, offset + vmo.len());
            if first_paddr.is_none() {
                first_paddr = Some(paddr);
            }
            first_vmo.get_or_insert(vmo);
        }
        Ok(first_paddr.unwrap())
    }
}

fn make_vmo(elf: &ElfFile, ph: ProgramHeader, epm: Arc<Mutex<MemoryRegion>>) -> LxResult<(Arc<VmObject>, PhysAddr)> {
    assert_eq!(ph.get_type().unwrap(), Type::Load);
    let page_offset = ph.virtual_addr() as usize % PAGE_SIZE;
    // (VirtAddr余数 + MemSiz)的pages
    let pages = pages(ph.mem_size() as usize + page_offset);
    trace!(
        "VmObject new pages: {:#x}, virtual_addr: {:#x}",
        pages,
        page_offset
    );
    let frames = epm.lock().alloc(pages).unwrap();
    let vmo = VmObject::new_with_frames(pages, frames);
    let data = match ph.get_data(elf).unwrap() {
        SegmentData::Undefined(data) => data,
        _ => return Err(LxError::EINVAL),
    };
    //调用 VMObjectTrait.write, 分配物理内存，后写入程序数据
    vmo.write(page_offset, data)?;
    Ok((vmo, frames[0].paddr.clone()))
}