#[cfg(unix)]
pub fn with_stderr_silenced<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    use libc::{O_WRONLY, STDERR_FILENO, close, dup, dup2, open};
    unsafe {
        let saved_stderr: i32 = dup(STDERR_FILENO);
        if saved_stderr == -1 {
            return f();
        };
        let dev_null: i32 = open(b"/dev/null\0".as_ptr() as *const i8, O_WRONLY, 0);
        if dev_null == -1 {
            close(saved_stderr);
            return f();
        };
        dup2(dev_null, STDERR_FILENO);
        let result: R = f();
        dup2(saved_stderr, STDERR_FILENO);
        close(saved_stderr);
        close(dev_null);
        result
    }
}

#[cfg(windows)]
pub fn with_stderr_silenced<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr::null_mut;
    use winapi::um::fileapi::{CreateFileW, OPEN_EXISTING};
    use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
    use winapi::um::processenv::{GetStdHandle, SetStdHandle};
    use winapi::um::winbase::STD_ERROR_HANDLE;
    use winapi::um::winnt::{FILE_SHARE_READ, FILE_SHARE_WRITE, GENERIC_WRITE};

    unsafe {
        let old_handle: *mut winapi::ctypes::c_void = GetStdHandle(STD_ERROR_HANDLE);

        let nul_wide: Vec<u16> = OsStr::new("NUL").encode_wide().chain(Some(0)).collect();
        let new_handle: *mut winapi::ctypes::c_void = CreateFileW(
            nul_wide.as_ptr(),
            GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            null_mut(),
            OPEN_EXISTING,
            0,
            null_mut(),
        );
        if new_handle == INVALID_HANDLE_VALUE {
            return f();
        };

        SetStdHandle(STD_ERROR_HANDLE, new_handle);
        let result: R = f();
        SetStdHandle(STD_ERROR_HANDLE, old_handle);
        CloseHandle(new_handle);
        result
    }
}

#[cfg(not(any(unix, windows)))]
pub fn with_stderr_silenced<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    f()
}
