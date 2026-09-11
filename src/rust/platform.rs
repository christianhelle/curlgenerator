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

/// A calendar date and wall clock time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalTime {
    /// The year, for example `2026`.
    pub year: i32,
    /// The month, from `1` to `12`.
    pub month: u32,
    /// The day of the month, from `1` to `31`.
    pub day: u32,
    /// The hour, from `0` to `23`.
    pub hour: u32,
    /// The minute, from `0` to `59`.
    pub minute: u32,
    /// The second, from `0` to `60`.
    pub second: u32,
}

/// Returns the current time in the local time zone.
#[cfg(unix)]
pub fn local_time() -> Option<LocalTime> {
    use std::{
        ffi::{c_char, c_int, c_long},
        time::{SystemTime, UNIX_EPOCH},
    };

    /// The fields of `struct tm`, including the extensions glibc, musl and the BSDs append.
    #[repr(C)]
    struct Tm {
        second: c_int,
        minute: c_int,
        hour: c_int,
        day: c_int,
        month: c_int,
        year: c_int,
        weekday: c_int,
        year_day: c_int,
        daylight_saving: c_int,
        offset: c_long,
        zone: *const c_char,
    }

    unsafe extern "C" {
        fn tzset();
        fn localtime_r(time: *const c_long, result: *mut Tm) -> *mut Tm;
    }

    let now =
        c_long::try_from(SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs()).ok()?;
    let mut tm = Tm {
        second: 0,
        minute: 0,
        hour: 0,
        day: 0,
        month: 0,
        year: 0,
        weekday: 0,
        year_day: 0,
        daylight_saving: 0,
        offset: 0,
        zone: std::ptr::null(),
    };

    unsafe { tzset() };
    if unsafe { localtime_r(&now, &mut tm) }.is_null() {
        return None;
    }

    Some(LocalTime {
        year: tm.year + 1900,
        month: u32::try_from(tm.month + 1).ok()?,
        day: u32::try_from(tm.day).ok()?,
        hour: u32::try_from(tm.hour).ok()?,
        minute: u32::try_from(tm.minute).ok()?,
        second: u32::try_from(tm.second).ok()?,
    })
}

/// Returns the current time in the local time zone.
#[cfg(windows)]
pub fn local_time() -> Option<LocalTime> {
    #[repr(C)]
    struct SystemTime {
        year: u16,
        month: u16,
        weekday: u16,
        day: u16,
        hour: u16,
        minute: u16,
        second: u16,
        milliseconds: u16,
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetLocalTime(time: *mut SystemTime);
    }

    let mut time = SystemTime {
        year: 0,
        month: 0,
        weekday: 0,
        day: 0,
        hour: 0,
        minute: 0,
        second: 0,
        milliseconds: 0,
    };
    unsafe { GetLocalTime(&mut time) };

    Some(LocalTime {
        year: i32::from(time.year),
        month: u32::from(time.month),
        day: u32::from(time.day),
        hour: u32::from(time.hour),
        minute: u32::from(time.minute),
        second: u32::from(time.second),
    })
}

/// Returns `None` where the local time cannot be queried.
#[cfg(not(any(unix, windows)))]
pub fn local_time() -> Option<LocalTime> {
    None
}
