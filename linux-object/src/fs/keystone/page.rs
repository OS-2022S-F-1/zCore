use kernel_hal::addr::{page_count};
use kernel_hal::mem::PhysFrame;
use zircon_object::vm::{PAGE_SIZE};
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
        let order = log2(min_pages) + 1;
        let count = 1 << order;
        let frames = PhysFrame::new_contiguous(count, order);
        Epm {
            // root_page_table: epm_vaddr,
            // ptr: epm_vaddr,
            size: count * PAGE_SIZE,
            order,
            pa: frames[0].paddr().into(),
            frames
        }
    }
}

impl Utm {
    pub fn new(untrusted_size: usize) -> Self {
        let min_pages = page_count(untrusted_size);
        let order = log2(min_pages) + 1;
        let count = 1 << order;
        let frames = PhysFrame::new_contiguous(count, order);
        Utm {
            // root_page_table: epm_vaddr,
            // ptr: epm_vaddr,
            size: count * PAGE_SIZE,
            order,
            pa: frames[0].paddr().into(),
            frames
        }
    }
}

