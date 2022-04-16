use alloc::sync::Arc;
use core::intrinsics::{log2f64};
use kernel_hal::{PhysAddr, VirtAddr};
use kernel_hal::addr::{page_count};
use zircon_object::task::Process;
use zircon_object::vm::{PAGE_SIZE, VmAddressRegion, VmObject};
use super::{Epm, Utm};

impl Epm {
    pub fn new(min_pages: usize, vmar: Arc<VmAddressRegion>) -> Self {
        let order = unsafe { log2f64(min_pages as f64) } as usize + 1;
        let count = 1 << order;
        let (vmo, device_phys_addr) = VmObject::new_contiguous(count, order)?;
        let epm_vaddr = vmar.map(vmar_offset, vmo.clone(), 0, vmo.len(), prot.to_flags())?;
        Epm {
            root_page_table: epm_vaddr,
            ptr: epm_vaddr,
            size: count * PAGE_SIZE,
            order,
            pa: device_phys_addr,
            vmar
        }
    }
}

impl Utm {
    pub fn new(untrusted_size: usize, vmar: Arc<VmAddressRegion>) -> Self {
        let min_page = page_count(untrusted_size);
        let order = unsafe { log2f64(min_pages as f64) } as usize + 1;
        let count = 1 << order;
        let (vmo, device_phys_addr) = VmObject::new_contiguous(count, order)?;
        let epm_vaddr = vmar.map(vmar_offset, vmo.clone(), 0, vmo.len(), prot.to_flags())?;
        Utm {
            root_page_table: epm_vaddr,
            ptr: epm_vaddr,
            size: count * PAGE_SIZE,
            order,
            pa: device_phys_addr,
            vmar
        }
    }
}

impl Drop for Epm {
    fn drop(&mut self) {
        self.vmar.unmap(ptr, self.size)?;
    }
}

impl Drop for Utm {
    fn drop(&mut self) {
        self.vmar.unmap(ptr, self.size)?;
    }
}


//
// int utm_init(struct utm* utm, size_t untrusted_size)
// {
// unsigned long req_pages = 0;
// unsigned long order = 0;
// unsigned long count;
// req_pages += PAGE_UP(untrusted_size)/PAGE_SIZE;
// order = ilog2(req_pages - 1) + 1;
// count = 0x1 << order;
//
// utm->order = order;
//
// /* Currently, UTM does not utilize CMA.
//  * It is always allocated from the buddy allocator */
// utm->ptr = (void*) __get_free_pages(GFP_HIGHUSER, order);
// if (!utm->ptr) {
// keystone_err("failed to allocate UTM (size = %ld bytes)\n",(1<<order));
// return -ENOMEM;
// }
//
// utm->size = count * PAGE_SIZE;
// if (utm->size != untrusted_size) {
// /* Instead of failing, we just warn that the user has to fix the parameter. */
// keystone_warn("shared buffer size is not multiple of PAGE_SIZE\n");
// }
//
// return 0;
// }
