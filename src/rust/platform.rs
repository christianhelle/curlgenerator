//! Platform queries answered through the operating system's own libraries.
//!
//! Everything here only depends on `std`, so the Windows code paths can be type checked on any
//! machine with `rustc --target x86_64-pc-windows-gnu --crate-type lib --emit=metadata`.

use std::ffi::OsString;

/// Returns the host name of the current machine.
#[cfg(unix)]
pub fn machine_name() -> Option<OsString> {
    use std::{ffi::c_char, os::unix::ffi::OsStringExt};

    unsafe extern "C" {
        fn gethostname(name: *mut c_char, length: usize) -> i32;
    }

    let mut buffer = vec![0u8; 256];
    if unsafe { gethostname(buffer.as_mut_ptr().cast(), buffer.len()) } != 0 {
        return None;
    }

    let end = buffer
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(buffer.len());
    buffer.truncate(end);

    Some(OsString::from_vec(buffer))
}

/// Returns the DNS host name of the current machine.
#[cfg(windows)]
pub fn machine_name() -> Option<OsString> {
    use std::os::windows::ffi::OsStringExt;

    const COMPUTER_NAME_PHYSICAL_DNS_HOSTNAME: i32 = 5;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetComputerNameExW(name_type: i32, buffer: *mut u16, size: *mut u32) -> i32;
    }

    let mut size = 0u32;
    unsafe {
        GetComputerNameExW(
            COMPUTER_NAME_PHYSICAL_DNS_HOSTNAME,
            std::ptr::null_mut(),
            &mut size,
        )
    };

    let mut buffer = vec![0u16; size as usize];
    let succeeded = unsafe {
        GetComputerNameExW(
            COMPUTER_NAME_PHYSICAL_DNS_HOSTNAME,
            buffer.as_mut_ptr(),
            &mut size,
        )
    } != 0;
    if !succeeded {
        return None;
    }
    buffer.truncate(size as usize);

    Some(OsString::from_wide(&buffer))
}

/// Returns `None` where the machine name cannot be queried.
#[cfg(not(any(unix, windows)))]
pub fn machine_name() -> Option<OsString> {
    None
}
