use alloc::sync::Arc;
use alloc::vec::Vec;
use core::slice::{from_raw_parts, from_raw_parts_mut};
use spin::Mutex;
use kernel_hal::mem::PhysFrame;
use kernel_hal::MMUFlags;
use kernel_hal::vm::{GenericPageTable, Page, PageSize, PagingError, PagingResult};
use zircon_object::vm::{PAGE_SIZE, PhysAddr, VirtAddr};
use crate::fs::keystone::MemoryRegion;


pub const PTE_V: usize = 0x001;
pub const PTE_R: usize = 0x002;
pub const PTE_W: usize = 0x004;
pub const PTE_X: usize = 0x008;
pub const PTE_U: usize = 0x010;
pub const PTE_G: usize = 0x020;
pub const PTE_A: usize = 0x040;
pub const PTE_D: usize = 0x080;

fn log2(mut x: usize) -> usize {
    let mut count = 0;
    while x > 1 {
        x /= 2;
        count += 1;
    }
    count
}

impl MemoryRegion {
    pub fn new(min_pages: usize) -> Self {
        let order = log2(min_pages) + 1;
        let count = 1 << order;
        let mut frames = PhysFrame::new_contiguous(count, order);
        frames.iter_mut().for_each(|frame| {frame.allocated = false; } );
        warn!("new memory region from {:x} to {:x}", frames[0].paddr, frames[frames.len()-1].paddr);
        Self {
            // root_page_table: epm_vaddr,
            // ptr: epm_vaddr,
            size: count * PAGE_SIZE,
            order,
            pa: frames[0].paddr().into(),
            frames
        }
    }

    pub fn alloc(&mut self, pages: usize) -> Option<Vec<PhysFrame>> {
        let mut ptr = 0;
        while ptr < self.frames.len() - pages {
            let mut contiguous = true;
            for index in ptr..(ptr + pages) {
                if self.frames[index].allocated {
                    ptr = index + 1;
                    contiguous = false;
                    break;
                }
            }
            if contiguous {
                let mut frames = Vec::new();
                self.frames[ptr..(ptr + pages)].iter_mut().for_each(|frame| {
                    frames.push(frame.clone());
                    frame.allocated = true;
                });
                return Some(frames);
            }
        }
        None
    }

    pub fn free_paddr(&self) -> Option<PhysAddr> {
        for frame in &self.frames {
            if !frame.allocated {
                return Some(frame.paddr);
            }
        }
        None
    }
}

pub struct EnclavePageTable {
    root: PhysAddr,
    epm: Arc<Mutex<MemoryRegion>>
}

impl EnclavePageTable {
    pub fn new(epm: Arc<Mutex<MemoryRegion>>) -> Self {
        let paddr = epm.lock().alloc(1).unwrap()[0].paddr;
        kernel_hal::mem::pmem_zero(paddr, PAGE_SIZE);
        Self {
            root: paddr,
            epm
        }
    }
    fn get_flag(flags: MMUFlags) -> usize {
        let mut pte_flag = PTE_V | PTE_A | PTE_D;
        if flags.contains(MMUFlags::WRITE) {
            pte_flag |= PTE_W;
        }
        if flags.contains(MMUFlags::READ) {
            pte_flag |= PTE_R;
        }
        if flags.contains(MMUFlags::EXECUTE) {
            pte_flag |= PTE_X;
        }
        if flags.contains(MMUFlags::USER) {
            pte_flag |= PTE_U;
        }
        pte_flag
    }

    fn get_mmu_flag(flags: usize) -> MMUFlags {
        let mut mmu_flags = MMUFlags::empty();
        if flags & PTE_W != 0 {
            mmu_flags |= MMUFlags::WRITE;
        }
        if flags & PTE_R != 0 {
            mmu_flags |= MMUFlags::READ;
        }
        if flags & PTE_X != 0 {
            mmu_flags |= MMUFlags::EXECUTE;
        }
        if flags & PTE_U != 0 {
            mmu_flags |= MMUFlags::USER;
        }
        mmu_flags
    }

    fn next_paddr(ppn: PhysAddr, vaddr: VirtAddr, level: usize) -> PhysAddr {
        ppn + (((vaddr >> (30 - 9 * level)) & ((1 << 9) - 1)) << 3)
    }
}

impl GenericPageTable for EnclavePageTable {
    fn table_phys(&self) -> PhysAddr {
        self.root
    }

    fn map(&mut self, page: Page, paddr: PhysAddr, flags: MMUFlags) -> PagingResult {
        let mut ppn = self.root;
        for i in 0..3 {
            let next_paddr = EnclavePageTable::next_paddr(ppn, page.vaddr, i);
            let mut pte: usize = 0;
            unsafe { kernel_hal::mem::pmem_read(next_paddr, from_raw_parts_mut(&mut pte as *mut usize as *mut u8, 8)); }
            if pte & PTE_V == 0 {
                ppn = self.epm.lock().alloc(1).unwrap()[0].paddr;
                kernel_hal::mem::pmem_zero(ppn, PAGE_SIZE);
                pte = EnclavePageTable::get_flag(flags) | (ppn << 10);
                unsafe { kernel_hal::mem::pmem_write(next_paddr, from_raw_parts(&pte as *const usize as *const u8, 8)); }
            } else {
                ppn = pte >> 10;
            }
        }
        let next_paddr = ppn + (page.vaddr & ((1 << 12) - 1));
        let pte = EnclavePageTable::get_flag(flags) | (paddr << 10);
        unsafe { kernel_hal::mem::pmem_write(next_paddr, from_raw_parts(&pte as *const usize as *const u8, 8)); }
        Ok(())
    }

    fn unmap(&mut self, vaddr: VirtAddr) -> PagingResult<(PhysAddr, PageSize)> {
        self.unmap_cont(vaddr, PAGE_SIZE)?;
        Ok((0, PageSize::Size4K))
    }

    fn update(
        &mut self,
        vaddr: VirtAddr,
        _paddr: Option<PhysAddr>,
        flags: Option<MMUFlags>,
    ) -> PagingResult<PageSize> {
        if let Some(flags) = flags {
            let mut ppn = self.root;
            for i in 0..3 {
                let next_paddr = EnclavePageTable::next_paddr(ppn, vaddr, i);
                let mut pte: usize = 0;
                unsafe { kernel_hal::mem::pmem_read(next_paddr, from_raw_parts_mut(&mut pte as *mut usize as *mut u8, 8)); }
                if pte & PTE_V == 0 {
                    return Ok(PageSize::Size4K);
                } else {
                    ppn = pte >> 10;
                }
            }
            let next_paddr = ppn + vaddr & ((1 << 12) - 1);
            let pte = if let Some(paddr) = _paddr {EnclavePageTable::get_flag(flags) | (paddr << 10) } else {
                let mut pte: usize = 0;
                unsafe { kernel_hal::mem::pmem_read(next_paddr, from_raw_parts_mut(&mut pte as *mut usize as *mut u8, 8)); }
                pte | EnclavePageTable::get_flag(flags)
            };
            unsafe {kernel_hal::mem::pmem_write(next_paddr, from_raw_parts(&pte as *const usize as *const u8, 8)); }
        }
        Ok(PageSize::Size4K)
    }

    fn query(&self, vaddr: VirtAddr) -> PagingResult<(PhysAddr, MMUFlags, PageSize)> {
        let mut ppn = self.root;
        for i in 0..3 {
            let next_paddr = EnclavePageTable::next_paddr(ppn, vaddr, i);
            let mut pte: usize = 0;
            unsafe { kernel_hal::mem::pmem_read(next_paddr, from_raw_parts_mut(&mut pte as *mut usize as *mut u8, 8)); }
            if pte & PTE_V == 0 {
                return Err(PagingError::NotMapped);
            } else {
                ppn = pte >> 10;
            }
        }
        let next_paddr = ppn + vaddr & ((1 << 12) - 1);
        let mut pte: usize = 0;
        unsafe { kernel_hal::mem::pmem_read(next_paddr, from_raw_parts_mut(&mut pte as *mut usize as *mut u8, 8)); }
        Ok((
            next_paddr,
            EnclavePageTable::get_mmu_flag(pte & ((1 << 10) - 1)),
            PageSize::Size4K,
        ))
    }

    fn unmap_cont(&mut self, vaddr: VirtAddr, size: usize) -> PagingResult {
        if size == 0 {
            return Ok(());
        }
        for page_index in 0..size / PAGE_SIZE {
            let mut ppn = self.root;
            for i in 0..3 {
                let next_paddr = EnclavePageTable::next_paddr(ppn, vaddr + PAGE_SIZE * page_index, i);
                let mut pte: usize = 0;
                unsafe {kernel_hal::mem::pmem_read(next_paddr, from_raw_parts_mut(&mut pte as *mut usize as *mut u8, 8)); }
                if pte & PTE_V == 0 {
                    return Ok(());
                } else {
                    ppn = pte >> 10;
                }
            }
            let next_paddr = ppn + vaddr & ((1 << 12) - 1);
            let pte: usize = 0;
            unsafe {kernel_hal::mem::pmem_write(next_paddr, from_raw_parts(&pte as *const usize as *const u8, 8)); }
        }
        Ok(())
    }
}

