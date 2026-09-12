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

/// Returns `true` when standard output is a terminal that renders ANSI colors.
///
/// Mirrors `console`: `NO_COLOR` disables colors, and so does a missing or `dumb` `TERM`.
#[cfg(unix)]
pub fn stdout_is_color_terminal() -> bool {
    use std::io::IsTerminal;

    std::io::stdout().is_terminal()
        && std::env::var("NO_COLOR").is_err()
        && std::env::var("TERM").is_ok_and(|term| term != "dumb")
}

/// Returns `true` when standard output is a terminal that renders ANSI colors.
///
/// A Windows console only renders them once virtual terminal processing is switched on, which is
/// attempted here. A terminal that is not a console, such as the MSYS2 or Cygwin pseudo terminal,
/// renders them unless `TERM` is `dumb`.
#[cfg(windows)]
pub fn stdout_is_color_terminal() -> bool {
    use std::{io::IsTerminal, os::windows::io::AsRawHandle};

    const ENABLE_VIRTUAL_TERMINAL_PROCESSING: u32 = 0x0004;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetConsoleMode(console: *mut std::ffi::c_void, mode: *mut u32) -> i32;
        fn SetConsoleMode(console: *mut std::ffi::c_void, mode: u32) -> i32;
    }

    let stdout = std::io::stdout();
    if !stdout.is_terminal() || std::env::var("NO_COLOR").is_ok() {
        return false;
    }

    let handle = stdout.as_raw_handle();
    let mut mode = 0u32;
    if unsafe { GetConsoleMode(handle, &mut mode) } == 0 {
        return std::env::var("TERM").map_or(true, |term| term != "dumb");
    }

    mode & ENABLE_VIRTUAL_TERMINAL_PROCESSING != 0
        || unsafe { SetConsoleMode(handle, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING) } != 0
}

/// Returns `false` where colors cannot be detected.
#[cfg(not(any(unix, windows)))]
pub fn stdout_is_color_terminal() -> bool {
    false
}

/// Returns the number of columns of the terminal standard output is connected to.
#[cfg(unix)]
pub fn stdout_columns() -> Option<usize> {
    use std::{
        ffi::{c_int, c_ulong},
        io::IsTerminal,
        os::fd::AsRawFd,
    };

    #[repr(C)]
    struct WindowSize {
        rows: u16,
        columns: u16,
        width: u16,
        height: u16,
    }

    #[cfg(any(target_os = "linux", target_os = "android"))]
    const TIOCGWINSZ: c_ulong = if cfg!(any(
        target_arch = "mips",
        target_arch = "mips64",
        target_arch = "powerpc",
        target_arch = "powerpc64",
        target_arch = "sparc64"
    )) {
        0x4008_7468
    } else {
        0x5413
    };
    #[cfg(not(any(target_os = "linux", target_os = "android")))]
    const TIOCGWINSZ: c_ulong = 0x4008_7468;

    unsafe extern "C" {
        fn ioctl(descriptor: c_int, request: c_ulong, ...) -> c_int;
    }

    let stdout = std::io::stdout();
    if !stdout.is_terminal() {
        return None;
    }

    let mut size = WindowSize {
        rows: 0,
        columns: 0,
        width: 0,
        height: 0,
    };
    if unsafe { ioctl(stdout.as_raw_fd(), TIOCGWINSZ, &mut size) } != 0 {
        return None;
    }

    (size.rows > 0 && size.columns > 0).then_some(usize::from(size.columns))
}

/// Returns the number of columns of the console window standard output is connected to.
#[cfg(windows)]
pub fn stdout_columns() -> Option<usize> {
    use std::os::windows::io::AsRawHandle;

    #[repr(C)]
    struct Coordinate {
        x: i16,
        y: i16,
    }

    #[repr(C)]
    struct Rectangle {
        left: i16,
        top: i16,
        right: i16,
        bottom: i16,
    }

    #[repr(C)]
    struct ScreenBufferInfo {
        size: Coordinate,
        cursor_position: Coordinate,
        attributes: u16,
        window: Rectangle,
        maximum_window_size: Coordinate,
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetConsoleScreenBufferInfo(
            console: *mut std::ffi::c_void,
            info: *mut ScreenBufferInfo,
        ) -> i32;
    }

    let origin = || Coordinate { x: 0, y: 0 };
    let mut info = ScreenBufferInfo {
        size: origin(),
        cursor_position: origin(),
        attributes: 0,
        window: Rectangle {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        },
        maximum_window_size: origin(),
    };
    if unsafe { GetConsoleScreenBufferInfo(std::io::stdout().as_raw_handle(), &mut info) } == 0 {
        return None;
    }

    usize::try_from(i32::from(info.window.right) - i32::from(info.window.left) + 1).ok()
}

/// Returns `None` where the terminal size cannot be queried.
#[cfg(not(any(unix, windows)))]
pub fn stdout_columns() -> Option<usize> {
    None
}
