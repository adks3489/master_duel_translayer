use anyhow::Result;
use winapi::{
    shared::minwindef::{DWORD, LPARAM},
    shared::windef::HWND,
    um::processthreadsapi::OpenProcess,
    um::winnt::HANDLE,
    um::winuser::{EnumChildWindows, EnumWindows, GetWindowThreadProcessId, SetLastErrorEx},
};

unsafe extern "system" fn callback_enum_windows_until<T: FnMut(HWND) -> i32>(
    window: HWND,
    param: LPARAM,
) -> i32 {
    let func = &mut *(param as *mut T);

    func(window)
}
pub fn enum_by_until<T: FnMut(HWND) -> i32>(
    parent: Option<HWND>,
    mut cmp_func: T,
) -> std::io::Result<()> {
    let lparam = &mut cmp_func as *mut _ as LPARAM;

    let result: i32;

    //Necessary if we want to guarantee that we can correctly detect interrupt of enumeration.
    unsafe { SetLastErrorEx(0, 0) };
    if let Some(parent_window) = parent {
        result = unsafe {
            EnumChildWindows(
                parent_window,
                Some(callback_enum_windows_until::<T>),
                lparam,
            )
        };
    } else {
        result = unsafe { EnumWindows(Some(callback_enum_windows_until::<T>), lparam) };
    }

    //If cmp_func returns 0 then EnumWindows too.
    //But it is not an error case.
    if result == 0 {
        let error = std::io::Error::last_os_error();

        if let Some(errno) = error.raw_os_error() {
            if errno != 0 {
                return Err(std::io::Error::last_os_error());
            }
        }
    }

    Ok(())
}
pub fn get_thread_process_id(window: HWND) -> (u32, u32) {
    let mut process_pid: u32 = 0;
    let thread_pid = unsafe { GetWindowThreadProcessId(window, &mut process_pid) };

    (process_pid, thread_pid)
}
pub fn get_by_pid(pid: u32) -> Result<HWND> {
    let mut found_window: Option<HWND> = None;
    enum_by_until(None, |handle: HWND| {
        let (process_pid, _) = get_thread_process_id(handle);
        if process_pid == pid {
            found_window = Some(handle);
            return 0;
        }
        1
    })?;
    Ok(found_window.unwrap())
}

pub fn open_process(desired_address: DWORD, pid: u32) -> Result<HANDLE> {
    let handle = unsafe { OpenProcess(desired_address, 0, pid) };
    if handle.is_null() {
        Err(anyhow::anyhow!("Open process fail"))
    } else {
        Ok(handle)
    }
}
