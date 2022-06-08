use core::mem::size_of;
use kernel_hal::user::{UserInOutPtr};
use crate::error::{LxError, LxResult};
use crate::fs::keystone::Utm;
use super::enclave_manager::*;
use super::Enclave;
use super::sbi::*;

pub const IOC_MAGIC: usize = 0xa4 << 8;
pub const CREATE_ENCLAVE: usize = IOC_MAGIC | 0x00;
pub const DESTROY_ENCLAVE: usize = IOC_MAGIC | 0x01;
pub const RUN_ENCLAVE: usize = IOC_MAGIC | 0x04;
pub const RESUME_ENCLAVE: usize = IOC_MAGIC | 0x05;
pub const FINALIZE_ENCLAVE: usize = IOC_MAGIC | 0x06;
pub const UTM_INIT: usize = IOC_MAGIC | 0x07;


pub struct Cmd(pub usize);

impl From<usize> for Cmd {
    fn from(x: usize) -> Self {
        Cmd(x)
    }
}

impl Cmd {
    fn get_field(&self, lo: usize, hi: usize) -> usize {
        (self.0 & ((1 << hi + 1) - 1)) >> lo
    }

    pub fn ioc_size(&self) -> usize {
        self.get_field(16, 29)
    }

    pub fn ioc_type(&self) -> usize {
        self.get_field(8, 15)
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

pub fn ioctl(cmd: Cmd, base: usize) -> LxResult<usize> {
    info!("Call keystone ioctl!");
    match cmd.match_field() {
        CREATE_ENCLAVE | DESTROY_ENCLAVE | FINALIZE_ENCLAVE | UTM_INIT => {
            if cmd.ioc_size() >= size_of::<CreateParams>() {
                let mut ptr: UserInOutPtr<CreateParams> = base.into();
                if let Ok(mut data) = ptr.read() {
                    let ret = match cmd.match_field() {
                        CREATE_ENCLAVE => { create_enclave(&mut data) },
                        DESTROY_ENCLAVE => { destroy_enclave(&data) },
                        FINALIZE_ENCLAVE => { finalize_enclave(&data) },
                        UTM_INIT => { utm_init_ioctl(&mut data) },
                        _ => { Err(LxError::ENOSYS) }
                    };
                    if let Ok(_) = ptr.write(data) {
                        return ret;
                    }
                }
            }
            Err(LxError::EFAULT)
        },
        RUN_ENCLAVE | RESUME_ENCLAVE => {
            if cmd.ioc_size() >= size_of::<RunParams>() {
                let mut ptr: UserInOutPtr<RunParams> = base.into();
                if let Ok(mut data) = ptr.read() {
                    let ret = match cmd.match_field() {
                        RUN_ENCLAVE => { run_enclave(&mut data) },
                        RESUME_ENCLAVE => { resume_enclave(&mut data) },
                        _ => { Err(LxError::ENOSYS) }
                    };
                    if let Ok(_) = ptr.write(data) {
                        return ret;
                    }
                }
            }
            Err(LxError::EFAULT)
        }
        _ => { Err(LxError::ENOSYS) }
    }
}

fn create_enclave(data: &mut CreateParams) -> LxResult<usize> {
    let enclave = Enclave::new(data.min_pages);
    data.pt_ptr = enclave.epm.pa;
    data.epm_size = enclave.epm.size;
    data.eid = alloc(enclave).unwrap();
    Ok(0)
}

fn finalize_enclave(data: &CreateParams) -> LxResult<usize> {
    if let Ok(_) = modify_enclave_by_id(data.eid, |enclave| {
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
        let ret: Sbiret = sbi_sm_create_enclave(&sbi_create as *const SbiCreate as usize).into();
        if ret.error == 0 {
            enclave.eid = ret.value as isize;
            Ok(0)
        } else {
            Err(LxError::EINVAL)
        }
    }) {
        Ok(0)
    } else {
        error!("Invalid enclave id");
        remove_by_id(data.eid);
        Err(LxError::EINVAL)
    }
}

fn destroy_enclave(data: &CreateParams) -> LxResult<usize> {
    if let Ok(sbi_eid) = get_enclave_sbi_eid(data.eid) {
        remove_by_id(data.eid);
        if sbi_eid >= 0 {
            let ret: Sbiret = sbi_sm_destroy_enclave(sbi_eid as usize).into();
            if ret.error > 0 {
                error!("cannot destroy enclave: SBI failed with error code {}", ret.error);
                Err(LxError::EINVAL)
            } else {
                Ok(0)
            }
        } else {
            warn!("destroy_enclave: skipping");
            Ok(0)
        }
    } else {
        Err(LxError::EINVAL)
    }
}

fn run_enclave(data: &mut RunParams) -> LxResult<usize> {
    if let Ok(sbi_eid) = get_enclave_sbi_eid(data.eid) {
        if sbi_eid >= 0 {
            let ret: Sbiret = sbi_sm_run_enclave(sbi_eid as usize).into();
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
    if let Ok(sbi_eid) = get_enclave_sbi_eid(data.eid) {
        if sbi_eid >= 0 {
            let ret: Sbiret = sbi_sm_resume_enclave(sbi_eid as usize).into();
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

fn utm_init_ioctl(data: &CreateParams) -> LxResult<usize> {
    modify_enclave_by_id(data.eid, |enclave| {
        enclave.utm = Utm::new(data.params.untrusted_size);
        Ok(0)
    })
}


