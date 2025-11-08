use {napi::bindgen_prelude::*, solana_hash::Hash, solana_pubkey::Pubkey, std::str::FromStr};

pub(crate) fn convert_pubkey(address: &[u8]) -> Pubkey {
    Pubkey::try_from(address).unwrap()
}

// Guards and restores the thread's floating-point environment.
// On Linux x86_64 this captures both x87 and SSE (MXCSR) state via libc's fenv APIs.
// On other architectures it is a no-op.
#[cfg(target_arch = "x86_64")]
pub(crate) struct FpuEnvGuard {
    saved: libc::fenv_t,
}

#[cfg(target_arch = "x86_64")]
impl FpuEnvGuard {
    pub fn new() -> Self {
        unsafe {
            let mut env = std::mem::MaybeUninit::<libc::fenv_t>::uninit();
            // Best effort: ignore return codes; restoring on Drop is still safe.
            let _ = libc::fegetenv(env.as_mut_ptr());
            Self {
                saved: env.assume_init(),
            }
        }
    }
}

#[cfg(target_arch = "x86_64")]
impl Drop for FpuEnvGuard {
    fn drop(&mut self) {
        unsafe {
            // Restore the saved environment and clear any pending exceptions.
            let _ = libc::fesetenv(&mut self.saved as *mut libc::fenv_t);
            let _ = libc::feclearexcept(libc::FE_ALL_EXCEPT);
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
