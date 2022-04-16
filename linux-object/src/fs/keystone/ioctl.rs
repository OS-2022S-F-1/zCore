use alloc::sync::Arc;
use kernel_hal::user::{Error, UserInOutPtr};
use zircon_object::task::vmar;
use zircon_object::vm::VmAddressRegion;
use crate::error::{LxError, LxResult};
use crate::fs::keystone::Utm;
use super::enclave_manager::*;
use super::Enclave;
use super::sbi::*;

pub const IOC_MAGIC: usize = 0xa4 << 8;
pub const CREATE_ENCLAVE: usize = IOC_MAGIC & 0x00;
pub const DESTROY_ENCLAVE: usize = IOC_MAGIC & 0x01;
pub const RUN_ENCLAVE: usize = IOC_MAGIC & 0x04;
pub const RESUME_ENCLAVE: usize = IOC_MAGIC & 0x05;
pub const FINALIZE_ENCLAVE: usize = IOC_MAGIC & 0x06;
pub const UTM_INIT: usize = IOC_MAGIC & 0x07;


struct Cmd(pub usize);

impl From<usize> for Cmd {
    fn from(x: usize) -> Self {
        Cmd(x)
    }
}

impl Cmd {
    fn get_field(&self, lo: usize, hi: usize) -> usize {
        return (self.0 & (1 << (hi + 1) - 1)) >> lo
    }

    pub fn ioc_size(&self) -> usize {
        return get_field(16, 29)
    }

    pub fn ioc_type(&self) -> usize {
        return get_field(8, 15)
    }

    pub fn match_field(&self) -> usize {
        self.0 & 0xFFFF
    }
}

pub struct RuntimeParams {
    runtime_entry: usize,
    user_entry: usize,
    untrusted_ptr: usize,
    untrusted_size: usize
}

pub struct CreateParams {
     eid: usize,
    //Min pages required
     min_pages: usize,
    // virtual addresses
     runtime_vaddr: usize,
     user_vaddr: usize,
     pt_ptr: usize,
     utm_free_ptr: usize,
    //Used for hash
     epm_paddr: usize,
     utm_paddr: usize,
     runtime_paddr: usize,
     user_paddr: usize,
     free_paddr: usize,

     epm_size: usize,
     utm_size: usize,
    // Runtime Parameters
    params: RuntimeParams
}

pub struct RunParams {
    eid: usize,
    error: usize,
    value: usize
}

pub fn ioctl(cmd: Cmd, mut enclave_id: UserInOutPtr<usize>, mut base: UserInOutPtr<u8>, vmar: Arc<VmAddressRegion>) -> LxResult<usize>
{
    if let Ok(mut data) = base.read_array(cmd.ioc_size()) {
        let ret = match cmd.match_field() {
            CREATE_ENCLAVE => { create_enclave(enclave_id, data.as_slice() as &mut CreateParams, vmar) }
            DESTROY_ENCLAVE => { destroy_enclave(enclave_id, data.as_slice() as &CreateParams) }
            RUN_ENCLAVE => { run_enclave(data.as_slice() as &mut RunParams) }
            RESUME_ENCLAVE => { resume_enclave(data.as_slice() as &mut RunParams) }
            FINALIZE_ENCLAVE => { finalize_enclave(data.as_slice() as &CreateParams) }
            UTM_INIT => { utm_init_ioctl(data.as_slice() as &mut CreateParams, vmar) }
            _ => { Err(LxError::ENOSYS) }
        };
        if let Ok(_) = base.write_array(data.as_slice()) {
            ret
        } else {
            Err(LxError::EFAULT)
        }
    } else {
        Err(LxError::EFAULT)
    }
}

fn create_enclave(mut enclave_id: UserInOutPtr<usize>, data: &mut CreateParams, vmar: Arc<VmAddressRegion>) -> LxResult<usize> {
    let enclave = Arc::new(Enclave::new(data.min_pages, vmar));
    data.pt_ptr = enclave.epm.root_page_table;
    data.epm_size = enclave.epm.size;
    data.eid = alloc(enclave)?;
    enclave_id.write(data.eid);
    Ok(0)
}

fn finalize_enclave(data: &CreateParams) -> LxResult<usize> {
    if let Some(enclave) = get_enclave_by_id(data.eid) {
        enclave.is_init = false;
        let sbi_create = SbiCreate {
            epm_region: SbiRegion {
                paddr: enclave.epm.pa,
                size: enclave.epm.size
            },
            utm_region: SbiRegion {
                paddr: enclave.utm.pa,
                size: enclave.utm.size
            },
            runtime_paddr: data.runtime_paddr,
            user_paddr: data.user_paddr,
            free_paddr: data.free_paddr,
            runtime_params: RuntimeParams {
                runtime_entry: 0,
                user_entry: 0,
                untrusted_ptr: 0,
                untrusted_size: 0
            }
        };
        let ret = sbi_sm_create_enclave(sbi_create as *const SbiCreate as usize);
        if ret.error != 0 {
            remove_by_id(data.eid);
            Err(LxError::EINVAL)
        } else {
            enclave.eid = ret.value as usize;
            Ok(0)
        }
    } else {
        error!("Invalid enclave id");
        Err(LxError::EINVAL)
    }
}

fn destroy_enclave(mut enclave_id: UserInOutPtr<usize>, data: &CreateParams) -> LxResult<usize> {
    if let Some(enclave) = get_enclave_by_id(data.eid) {
        remove_by_id(data.eid);
        if enclave.eid >= 0 {
            let ret = sbi_sm_destroy_enclave(enclave.eid);
            enclave_id.write(0);
            if ret.error >= 0 {
                error!("cannot destroy enclave: SBI failed with error code {}", ret.error);
                Err(LxError::EINVAL)
            } else {
                Ok(0)
            }
        } else {
            warn!("destroy_enclave: skipping");
        }
    } else {
        Err(LxError::EINVAL)
    }
}

fn run_enclave(data: &mut RunParams) -> LxResult<usize> {
    if let Some(enclave) = get_enclave_by_id(data.eid) {
            if enclave.eid >= 0 {
                let ret = sbi_sm_run_enclave(enclave.eid);
                data.error = ret.error as usize;
                data.value = ret.value as usize;
                Ok(0)
            } else {
                error!("real enclave does not exist");
                Err(LxError::EINVAL)
            }
    } else {
        Err(LxError::EINVAL)
    }
}

fn resume_enclave(data: &mut RunParams) -> LxResult<usize> {
    if let Some(enclave) = get_enclave_by_id(data.eid) {
        if enclave.eid >= 0 {
            let ret = sbi_sm_resume_enclave(enclave.eid);
            data.error = ret.error as usize;
            data.value = ret.value as usize;
            Ok(0)
        } else {
            error!("real enclave does not exist");
            Err(LxError::EINVAL)
        }
    } else {
        Err(LxError::EINVAL)
    }
}

fn utm_init_ioctl(data: &CreateParams, vmar: Arc<VmAddressRegion>) -> LxResult<usize> {
    if let Some(enclave) = get_enclave_by_id(data.eid) {
        enclave.utm = Some(Utm::new(data.params.untrusted_size, vmar));
        Ok(0)
    } else {
        Err(LxError::EINVAL)
    }
}


