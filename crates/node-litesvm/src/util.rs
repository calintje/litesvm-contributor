use {napi::bindgen_prelude::*, solana_hash::Hash, solana_pubkey::Pubkey, std::str::FromStr};

pub(crate) fn convert_pubkey(address: &[u8]) -> Pubkey {
    Pubkey::try_from(address).unwrap()
}

// Guards and restores the thread's floating-point environment.
// On x86_64 this captures both x87 control word and SSE (MXCSR) state without relying on libc fenv symbols.
// On other architectures it is a no-op.
#[cfg(target_arch = "x86_64")]
pub(crate) struct FpuEnvGuard {
    mxcsr: u32,
    x87_cw: u16,
}

#[cfg(target_arch = "x86_64")]
impl FpuEnvGuard {
    pub fn new() -> Self {
        unsafe {
            // Save SSE control/status register (MXCSR)
            let mut mxcsr: u32 = 0;
            core::arch::asm!("stmxcsr [{ptr}]", ptr = in(reg) &mut mxcsr, options(nostack, preserves_flags));
            // Save x87 control word via fnstcw
            let mut x87_cw: u16 = 0;
            core::arch::asm!("fnstcw [{cw}]", cw = in(reg) &mut x87_cw, options(nostack, preserves_flags));
            Self { mxcsr, x87_cw }
        }
    }
}

#[cfg(target_arch = "x86_64")]
impl Drop for FpuEnvGuard {
    fn drop(&mut self) {
        unsafe {
            // Hard-reset FP environment to defaults to avoid propagating any state into V8:
            // - MXCSR default mask: 0x1F80 (all exceptions masked, nearest rounding, flush modes off)
            // - x87 control word default: 0x037F (all exceptions masked, nearest rounding)
            let mxcsr_default: u32 = 0x1F80;
            core::arch::asm!("ldmxcsr [{ptr}]", ptr = in(reg) &mxcsr_default, options(nostack, preserves_flags));
            // Clear any pending x87 exceptions then set default control word
            core::arch::asm!("fnclex", options(nostack, preserves_flags));
            let x87_default: u16 = 0x037F;
            core::arch::asm!("fldcw [{cw}]", cw = in(reg) &x87_default, options(nostack, preserves_flags));
        }
    }
}

// No-op guard for non-x86_64 targets.
#[cfg(not(target_arch = "x86_64"))]
pub(crate) struct FpuEnvGuard;
#[cfg(not(target_arch = "x86_64"))]
impl FpuEnvGuard {
    pub fn new() -> Self {
        Self
    }
}

pub(crate) fn try_parse_hash(raw: &str) -> Result<Hash> {
    Hash::from_str(raw).map_err(|e| {
        Error::new(
            Status::GenericFailure,
            format!("Failed to parse blockhash: {e}"),
        )
    })
}

pub(crate) fn bigint_to_u64(val: &BigInt) -> Result<u64> {
    let res = val.get_u64();
    if res.0 {
        return Err(Error::new(
            Status::GenericFailure,
            format!("Cannot convert negative bigint to u64: {val:?}"),
        ));
    }
    if !res.2 {
        return Err(Error::new(
            Status::GenericFailure,
            format!("Bigint too large for u64: {val:?}"),
        ));
    }
    Ok(res.1)
}

pub(crate) fn bigint_to_u128(val: &BigInt) -> Result<u128> {
    let res = val.get_u128();
    if res.0 {
        return Err(Error::new(
            Status::GenericFailure,
            format!("Cannot convert negative bigint to u128: {val:?}"),
        ));
    }
    if !res.2 {
        return Err(Error::new(
            Status::GenericFailure,
            format!("Bigint too large for u128: {val:?}"),
        ));
    }
    Ok(res.1)
}

pub(crate) fn bigint_to_i64(val: &BigInt) -> Result<i64> {
    let res = val.get_i64();
    if !res.1 {
        return Err(Error::new(
            Status::GenericFailure,
            format!("Bigint too large for i64: {val:?}"),
        ));
    }
    Ok(res.0)
}

pub(crate) fn bigint_to_usize(val: &BigInt) -> Result<usize> {
    let res = val.get_u64();
    if res.0 {
        return Err(Error::new(
            Status::GenericFailure,
            format!("Cannot convert negative bigint to usize: {val:?}"),
        ));
    }
    if !res.2 {
        return Err(Error::new(
            Status::GenericFailure,
            format!("Bigint too large for usize: {val:?}"),
        ));
    }
    let val_u64 = res.1;
    usize::try_from(val_u64).map_err(|_| {
        Error::new(
            Status::GenericFailure,
            format!("Bigint too large for usize: {val_u64}"),
        )
    })
}
