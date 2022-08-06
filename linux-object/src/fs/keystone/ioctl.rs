use alloc::sync::Arc;
use core::mem::size_of;
use spin::Mutex;
use xmas_elf::ElfFile;
use kernel_hal::addr::page_count;
use kernel_hal::{MMUFlags, PAGE_SIZE};
use kernel_hal::user::{UserInOutPtr, UserInPtr};
use zircon_object::vm::VmObject;
use crate::error::{LxError, LxResult};
use crate::fs::keystone::elf_loader::EnclaveVmar;
use crate::fs::keystone::MemoryRegion;
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

pub const DEFAULT_STACK_SIZE: usize = 1024 * 16;
pub const DEFAULT_STACK_START: usize = 0x0000000040000000;
pub const DEFAULT_STACK_PAGES: usize = DEFAULT_STACK_SIZE / PAGE_SIZE;

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

    // pub fn ioc_type(&self) -> usize {
    //     self.get_field(8, 15)
    // }

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
    // Used for load elf and write page table
    runtime_vaddr: usize,
    runtime_size: usize,
    user_vaddr: usize,
    user_size: usize,

    utm_free_ptr: usize,
    // Runtime Parameters
    params: RuntimeParams
}

pub struct RunParams {
    eid: usize,
    error: usize,
    value: usize
}

pub fn ioctl(cmd: Cmd, base: usize) -> LxResult<usize> {
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

fn create_enclave(params: &mut CreateParams) -> LxResult<usize> {
    let mut enclave = Enclave::new(params.min_pages);
    let runtime_ptr: UserInPtr<u8> = params.runtime_vaddr.into();
    if let Ok(data) = runtime_ptr.read_array(params.runtime_size) {
        let elf = ElfFile::new(data.as_slice()).map_err(|_| LxError::EFAULT)?;
        // let size = elf.load_segment_size();
        // let image_vmar = enclave.vmar.allocate(Some(0), size, VmarFlags::CAN_MAP_RXW, PAGE_SIZE)?;
        // let mut base = image_vmar.addr();
        let runtime_paddr = enclave.vmar.load_elf_to_epm(&elf, enclave.epm.clone(), false)?;
        enclave.params.runtime_paddr = runtime_paddr.into();
    } else {
        return Err(LxError::EFAULT);
    }
    let user_ptr: UserInPtr<u8> = params.user_vaddr.into();
    if let Ok(data) = user_ptr.read_array(params.user_size) {
        let elf = ElfFile::new(data.as_slice()).map_err(|_| LxError::EFAULT)?;
        // let size = elf.load_segment_size();
        // let image_vmar = enclave.vmar.allocate(Some(0), size, VmarFlags::CAN_MAP_RXW, PAGE_SIZE)?;
        // let mut base = image_vmar.addr();
        let user_paddr = enclave.vmar.load_elf_to_epm(&elf, enclave.epm.clone(), true)?;
        enclave.params.user_paddr = user_paddr.into();
    } else {
        return Err(LxError::EFAULT);
    }
    let mut epm = enclave.epm.lock();
    // Init user stack
    let user_stack_frames = epm.alloc(DEFAULT_STACK_PAGES + 8).unwrap();
    enclave.params.pt_ptr = enclave.vmar.table_phys().into();
    enclave.params.free_paddr = epm.free_paddr().unwrap();
    drop(epm);
    // Fill physical address
    let user_stack_vmo = VmObject::new_with_frames(DEFAULT_STACK_PAGES + 8, user_stack_frames);
    enclave.vmar.map_at(DEFAULT_STACK_START - DEFAULT_STACK_SIZE, user_stack_vmo.clone(), 0, user_stack_vmo.len(),
                     MMUFlags::USER | MMUFlags::READ | MMUFlags::WRITE)?;

    params.eid = alloc(enclave).unwrap();
    Ok(0)
}

fn finalize_enclave(data: &CreateParams) -> LxResult<usize> {
    modify_enclave_by_id(data.eid, |enclave| {
        enclave.is_init = false;
        let epm = enclave.epm.lock();
        let utm = enclave.utm.lock();
        let sbi_create = SbiCreate {
            epm_region: SbiRegion {
                paddr: epm.pa,
                size: epm.size
            },
            utm_region: SbiRegion {
                paddr: utm.pa,
                size: utm.size
            },
            runtime_paddr: enclave.params.runtime_paddr,
            user_paddr: enclave.params.user_paddr,
            free_paddr: enclave.params.free_paddr,
            runtime_params: RuntimeParams {
                runtime_entry: data.params.runtime_entry,
                user_entry: data.params.user_entry,
                untrusted_ptr: data.params.untrusted_ptr,
                untrusted_size: data.params.untrusted_size
            }
        };
        drop(epm);
        drop(utm);
        let ret: Sbiret = sbi_sm_create_enclave(&sbi_create as *const SbiCreate as usize).into();
        if ret.error == 0 {
            enclave.eid = ret.value as isize;
            Ok(0)
        } else {
            Err(LxError::EINVAL)
        }
    }).map_err(|_| {
        error!("Invalid enclave id");
        remove_by_id(data.eid);
        LxError::EINVAL
    })
}

fn destroy_enclave(data: &CreateParams) -> LxResult<usize> {
    if let Ok(sbi_eid) = get_enclave_sbi_eid(data.eid) {
        if sbi_eid >= 0 {
            let ret: Sbiret = sbi_sm_destroy_enclave(sbi_eid as usize).into();
            if ret.error > 0 {
                error!("cannot destroy enclave: SBI failed with error code {}", ret.error);
                Err(LxError::EINVAL)
            } else {
                remove_by_id(data.eid);
                warn!("kernel remove enclave completed!");
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

fn utm_init_ioctl(data: &mut CreateParams) -> LxResult<usize> {
    modify_enclave_by_id(data.eid, |enclave| {
        enclave.utm = Arc::new(Mutex::new(MemoryRegion::new(page_count(data.params.untrusted_size))));
        // let utm = enclave.utm.lock();
        // data.utm_free_ptr = utm.pa;
        Ok(0)
    })
}


