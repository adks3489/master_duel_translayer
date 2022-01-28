use anyhow::{anyhow, Result};
use std::{ffi::c_void, mem::size_of};
use winapi::{
    ctypes::c_int,
    shared::{
        minwindef::{DWORD, FALSE, LPARAM, LPVOID, WORD},
        ntdef::NULL,
        windef::{DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, HGDIOBJ, HWND, LPRECT, RECT},
    },
    um::{
        dwmapi::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS},
        processthreadsapi::OpenProcess,
        wingdi::{
            BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits,
            SelectObject, BITMAPFILEHEADER, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
            HGDI_ERROR, RGBQUAD, SRCCOPY,
        },
        winnt::{HANDLE, LONG},
        winuser::{
            EnumChildWindows, EnumWindows, GetDC, GetForegroundWindow, GetWindowThreadProcessId,
            SetLastErrorEx, SetProcessDpiAwarenessContext,
        },
    },
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

pub fn capture_screen(hwnd: HWND) -> Result<Option<(BITMAPFILEHEADER, BITMAPINFO, Vec<u8>)>> {
    unsafe {
        let active_hwnd = GetForegroundWindow();
        if active_hwnd != hwnd {
            return Ok(None);
        }
        SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        let mut client_rect = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        DwmGetWindowAttribute(
            hwnd,
            DWMWA_EXTENDED_FRAME_BOUNDS,
            &mut client_rect as LPRECT as LPVOID,
            size_of::<RECT>() as DWORD,
        );
        dbg!(
            client_rect.top,
            client_rect.bottom,
            client_rect.left,
            client_rect.right
        );
        let hdc = GetDC(NULL as HWND);
        if hdc.is_null() {
            return Err(anyhow!("GetDC failed"));
        }
        let hdc_mem = CreateCompatibleDC(hdc);
        if hdc_mem.is_null() {
            return Err(anyhow!("CreateCompatibleDC failed"));
        }
        let width = client_rect.right - client_rect.left;
        let height = client_rect.bottom - client_rect.top;
        dbg!(width, height);
        let bmp_target = CreateCompatibleBitmap(hdc, width, height);
        if bmp_target.is_null() {
            return Err(anyhow!("CreateCompatibleBitmap failed"));
        }
        let res = SelectObject(hdc_mem, bmp_target as HGDIOBJ);
        if res.is_null() || res == HGDI_ERROR {
            return Err(anyhow!("SelectObject failed"));
        }
        if BitBlt(
            hdc_mem,
            0,
            0,
            width,
            height,
            hdc,
            client_rect.left,
            client_rect.top,
            SRCCOPY,
        ) == FALSE
        {
            return Err(anyhow!("BitBlt failed"));
        }

        const PIXEL_WIDTH: usize = 4;
        const BITMAPINFOHEADER_SIZE: u32 = 40;
        const BITMAPFILEHEADER_SIZE: u32 = 14;
        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: BITMAPINFOHEADER_SIZE,
                biWidth: width as LONG,
                biHeight: height as LONG,
                biPlanes: 1,
                biBitCount: 8 * PIXEL_WIDTH as WORD,
                biCompression: BI_RGB,
                biSizeImage: (width * height * PIXEL_WIDTH as c_int) as DWORD,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [RGBQUAD {
                rgbBlue: 0,
                rgbGreen: 0,
                rgbRed: 0,
                rgbReserved: 0,
            }],
        };
        let size: usize = (width * height) as usize * PIXEL_WIDTH;
        let mut data: Vec<u8> = Vec::with_capacity(size);
        data.set_len(size);

        GetDIBits(
            hdc,
            bmp_target,
            0,
            height as DWORD,
            &mut data[0] as *mut u8 as *mut c_void,
            &mut bmi as *mut BITMAPINFO,
            DIB_RGB_COLORS,
        );
        let bmp_size = ((width * bmi.bmiHeader.biBitCount as i32 + 31) / 32) * 4 * height;
        let bmf_header = BITMAPFILEHEADER {
            bfType: 0x4D42,
            bfSize: bmp_size as u32 + BITMAPFILEHEADER_SIZE + BITMAPINFOHEADER_SIZE,
            bfReserved1: 0,
            bfReserved2: 0,
            bfOffBits: BITMAPFILEHEADER_SIZE + BITMAPINFOHEADER_SIZE,
        };

        DeleteDC(hdc);
        DeleteObject(bmp_target as HGDIOBJ);

        DeleteDC(hdc);
        DeleteDC(hdc_mem);

        Ok(Some((bmf_header, bmi, data)))
    }
}
