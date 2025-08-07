/// Return the current Unix timestamp (seconds since epoch) at the time of program execution
#[inline]
pub fn sol_get_unix_timestamp() -> u64 {
    #[cfg(target_os = "solana")]
    unsafe {
        crate::syscalls::sol_get_unix_timestamp()
    }

    #[cfg(not(target_os = "solana"))]
    {
        crate::program_stubs::sol_get_unix_timestamp()
    }
}
