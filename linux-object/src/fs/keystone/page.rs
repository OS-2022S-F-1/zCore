use kernel_hal::addr::{page_count};
use zircon_object::vm::{PAGE_SIZE, VmObject};
use super::{Epm, Utm};

fn log2(mut x: usize) -> usize {
    let mut count = 0;
    while x > 1 {
        x /= 2;
        count += 1;
    }
    count
}

impl Epm {
    pub fn new(min_pages: usize) -> Self {
        let order = unsafe { log2(min_pages) } as usize + 1;
        let count = 1 << order;
        let (vmo, device_phys_addr) = VmObject::new_contiguous(count, order).unwrap();
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
        let min_pages = page_count(untrusted_size);
        let order = unsafe { log2(min_pages) } as usize + 1;
        let count = 1 << order;
        let (vmo, device_phys_addr) = VmObject::new_contiguous(count, order).unwrap();
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

