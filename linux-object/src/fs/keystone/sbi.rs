use super::ioctl::RuntimeParams;

const KEYSTONE_SBI_EXT_ID: usize = 0x08424b45;
const SBI_SM_CREATE_ENCLAVE: usize = 2001;
const SBI_SM_DESTROY_ENCLAVE: usize = 2002;
const SBI_SM_RUN_ENCLAVE: usize = 2003;
const SBI_SM_RESUME_ENCLAVE: usize = 2005;

pub struct Sbiret {
    pub error: i32,
    pub value: i32
}

pub struct SbiRegion {
    pub paddr: usize,
    pub size: usize
}

pub struct SbiCreate {
    // Memory regions for the enclave
    pub epm_region: SbiRegion,
    pub utm_region: SbiRegion,
    // physical addresses
    pub runtime_paddr: usize,
    pub user_paddr: usize,
    pub free_paddr: usize,
    // Parameters
    pub runtime_params: RuntimeParams
}

impl From<u64> for Sbiret {
    fn from(x: u64) -> Self {
        Sbiret {
            error: (x & ((1 << 32) - 1)) as i32,
            value: (x >> 32) as i32
        }
    }
}

#[inline(always)]
fn sbi_call(eid: usize, fid: usize, arg0: usize, arg1: usize, arg2: usize) -> u64 {
    let ret: u64;
    unsafe {
        core::arch::asm!("ecall",
        in("a0") arg0,
        in("a1") arg1,
        in("a2") arg2,
        in("a6") fid,
        in("a7") eid,
        lateout("a0") ret,
        );
    };
    ret
}

pub fn sbi_sm_create_enclave(args: usize) -> u64 {
    sbi_call(KEYSTONE_SBI_EXT_ID,
    SBI_SM_CREATE_ENCLAVE,
    args, 0, 0)
}

pub fn sbi_sm_run_enclave(eid: usize) -> u64 {
    sbi_call(KEYSTONE_SBI_EXT_ID,
    SBI_SM_RUN_ENCLAVE,
    eid, 0, 0)
}

pub fn sbi_sm_destroy_enclave(eid: usize) -> u64 {
    sbi_call(KEYSTONE_SBI_EXT_ID,
    SBI_SM_DESTROY_ENCLAVE,
    eid, 0, 0)
}

pub fn sbi_sm_resume_enclave(eid: usize) -> u64 {
    sbi_call(KEYSTONE_SBI_EXT_ID,
    SBI_SM_RESUME_ENCLAVE,
    eid, 0, 0)
}
