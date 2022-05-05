use alloc::sync::Arc;
use core::intrinsics::{log2f64};
use kernel_hal::{PhysAddr, VirtAddr};
use kernel_hal::addr::{page_count};
use zircon_object::task::Process;
use zircon_object::vm::{PAGE_SIZE, VmObject};
use super::{Epm, Utm};

impl Epm {
    pub fn new(min_pages: usize) -> Self {
        let order = unsafe { log2f64(min_pages as f64) } as usize + 1;
        let count = 1 << order;
        let (vmo, device_phys_addr) = VmObject::new_contiguous(count, order)?;
        Epm {
            // root_page_table: epm_vaddr,
            // ptr: epm_vaddr,
            size: count * PAGE_SIZE,
            order,
            pa: device_phys_addr,
            vmo
        }
    }
}

impl Utm {
    pub fn new(untrusted_size: usize) -> Self {
        let min_page = page_count(untrusted_size);
        let order = unsafe { log2f64(min_pages as f64) } as usize + 1;
        let count = 1 << order;
        let (vmo, device_phys_addr) = VmObject::new_contiguous(count, order)?;
        Utm {
            // root_page_table: epm_vaddr,
            // ptr: epm_vaddr,
            size: count * PAGE_SIZE,
            order,
            pa: device_phys_addr,
            vmo
        }
    }
}

