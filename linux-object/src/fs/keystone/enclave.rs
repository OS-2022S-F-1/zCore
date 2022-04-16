use alloc::sync::Arc;
use core::ptr::{null, null_mut};
use zircon_object::task::Process;
use zircon_object::vm::VmAddressRegion;
use super::{Enclave, Epm, Utm};

const ENCLAVE_IDR_MIN: usize = 0x1000;
const ENCLAVE_IDR_MAX: usize = 0xffff;
//
// unsigned long calculate_required_pages(
// unsigned long eapp_sz,
// unsigned long eapp_stack_sz,
// unsigned long rt_sz,
// unsigned long rt_stack_sz)
// {
// unsigned long req_pages = 0;
//
// req_pages += PAGE_UP(eapp_sz)/PAGE_SIZE;
// req_pages += PAGE_UP(eapp_stack_sz)/PAGE_SIZE;
// req_pages += PAGE_UP(rt_sz)/PAGE_SIZE;
// req_pages += PAGE_UP(rt_stack_sz)/PAGE_SIZE;
//
// // FIXME: calculate the required number of pages for the page table.
// // For now, we must allocate at least 1 (top) + 2 (enclave) + 2 (runtime) pages for pg tables
// req_pages += 15;
// return req_pages;
// }
//
// /* Smart destroy, handles partial initialization of epm and utm etc */
// int destroy_enclave(struct enclave* enclave)
// {
// struct epm* epm;
// struct utm* utm;
// if (enclave == NULL)
// return -ENOSYS;
//
// epm = enclave->epm;
// utm = enclave->utm;
//
// if (epm)
// {
// epm_destroy(epm);
// kfree(epm);
// }
// if (utm)
// {
// utm_destroy(utm);
// kfree(utm);
// }
// kfree(enclave);
// return 0;
// }
//

impl Enclave {
    pub fn new(min_pages: usize, vmar: Arc<VmAddressRegion>) -> Self {
        Enclave {
            eid: -1,
            close_on_pexit: 1,
            utm: None,
            epm: Some(Epm::new(min_pages, vmar)),
            is_init: true
        }
    }
}

impl Drop for Enclave {
    fn drop(&mut self) {
        // int keystone_destroy_enclave(struct file *filep, unsigned long arg)
        // {
        //     int ret;
        //     struct keystone_ioctl_create_enclave *enclp = (struct keystone_ioctl_create_enclave *) arg;
        //     unsigned long ueid = enclp->eid;
        //
        //     ret = __keystone_destroy_enclave(ueid);
        //     if (!ret) {
        //         filep->private_data = NULL;
        //     }
        //     return ret;
        // }
        //
        // int __keystone_destroy_enclave(unsigned int ueid)
        // {
        //     struct sbiret ret;
        //     struct enclave *enclave;
        //     enclave = get_enclave_by_id(ueid);
        //
        //     if (!enclave) {
        //         keystone_err("invalid enclave id\n");
        //         return -EINVAL;
        //     }
        //
        //     if (enclave->eid >= 0) {
        //     ret = sbi_sm_destroy_enclave(enclave->eid);
        //     if (ret.error) {
        //         keystone_err("fatal: cannot destroy enclave: SBI failed with error code %ld\n", ret.error);
        //         return -EINVAL;
        //     }
        // } else {
        //     keystone_warn("keystone_destroy_enclave: skipping (enclave does not exist)\n");
        // }
        //
        //
        //     destroy_enclave(enclave);
        //     enclave_idr_remove(ueid);
        //
        //     return 0;
        // }
    }
}
//
// unsigned int enclave_idr_alloc(struct enclave* enclave)
// {
// unsigned int ueid;
//
// mutex_lock(&idr_enclave_lock);
// ueid = idr_alloc(&idr_enclave, enclave, ENCLAVE_IDR_MIN, ENCLAVE_IDR_MAX, GFP_KERNEL);
// mutex_unlock(&idr_enclave_lock);
//
// if (ueid < ENCLAVE_IDR_MIN || ueid >= ENCLAVE_IDR_MAX) {
// keystone_err("failed to allocate UID\n");
// return 0;
// }
//
// return ueid;
// }
//
// struct enclave* enclave_idr_remove(unsigned int ueid)
// {
// struct enclave* enclave;
// mutex_lock(&idr_enclave_lock);
// enclave = idr_remove(&idr_enclave, ueid);
// mutex_unlock(&idr_enclave_lock);
// return enclave;
// }
//
// struct enclave* get_enclave_by_id(unsigned int ueid)
// {
// struct enclave* enclave;
// mutex_lock(&idr_enclave_lock);
// enclave = idr_find(&idr_enclave, ueid);
// mutex_unlock(&idr_enclave_lock);
// return enclave;
// }
