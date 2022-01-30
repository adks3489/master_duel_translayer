use crate::region::Region;
use anyhow::{anyhow, Context, Result};
use leptonica_sys::{pixReadMemBmp, Pix};
use scopeguard::defer;
use std::io::{BufWriter, Write};
use std::{ffi::CStr, ptr, str};
use sysinfo::{PidExt, ProcessExt, System, SystemExt};
use tesseract_sys::{
    TessBaseAPICreate, TessBaseAPIDelete, TessBaseAPIGetUTF8Text, TessBaseAPIInit3,
    TessBaseAPIRecognize, TessBaseAPISetImage2, TessBaseAPISetPageSegMode, TessBaseAPISetRectangle,
    TessBaseAPISetVariable, TessPageSegMode_PSM_RAW_LINE,
};
use winapi::{
    shared::windef::HWND,
    um::{
        wingdi::{BITMAPFILEHEADER, BITMAPINFO},
        winnt::HANDLE,
    },
};
mod region;
mod win_util;

pub struct Process {
    pub pid: u32,
    handle: HANDLE,
    hwnd: HWND,
}

const PROCESS_NAME: &str = "masterduel.exe";

pub fn attach_process() -> Result<Process> {
    let sys = System::new_all();
    let p = sys
        .processes_by_exact_name(PROCESS_NAME)
        .next()
        .ok_or(anyhow!("Cannot find opened Master Duel process."))?;
    let pid = p.pid().as_u32();
    Ok(Process {
        pid,
        handle: win_util::open_process(0x400, pid)
            .with_context(|| format!("Unable to get handle of {}", PROCESS_NAME))?,
        hwnd: win_util::get_by_pid(pid)
            .with_context(|| format!("Unable to get hwnd of {}", PROCESS_NAME))?,
    })
}

pub fn capture(process: &Process) -> Result<()> {
    let r = win_util::capture_screen(process.hwnd)?;
    if let Some((bmf_header, bmi, data)) = r {
        write_bmp("test.bmp", &bmf_header, &bmi, &data)?;
        let mut buf = Vec::with_capacity(54 + data.len());
        combine_bmp(&mut buf, &bmf_header, &bmi, &data)?;
        let pix = unsafe { pixReadMemBmp(buf.as_ptr(), buf.len().try_into().unwrap()) };
        let s = ocr(pix, Some(region::MAIN_MENU_DUEL));
        dbg!(s);
    }
    Ok(())
}

fn ocr(image: *mut Pix, region: Option<Region>) -> String {
    unsafe {
        let cube = TessBaseAPICreate();
        defer! {
            TessBaseAPIDelete(cube);
        }
        TessBaseAPIInit3(
            cube,
            ptr::null(),
            CStr::from_bytes_with_nul_unchecked(b"eng\0").as_ptr(),
        );
        TessBaseAPISetVariable(
            cube,
            CStr::from_bytes_with_nul_unchecked(b"tessedit_char_whitelist\0").as_ptr(),
            CStr::from_bytes_with_nul_unchecked(
                b" !#\"$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~\0",
            )
            .as_ptr(),
        );
        TessBaseAPISetPageSegMode(cube, TessPageSegMode_PSM_RAW_LINE);
        TessBaseAPISetImage2(cube, image);
        if let Some(r) = region {
            TessBaseAPISetRectangle(cube, r.left, r.top, r.width(), r.height());
        }
        TessBaseAPIRecognize(cube, ptr::null_mut());

        str::from_utf8(CStr::from_ptr(TessBaseAPIGetUTF8Text(cube)).to_bytes())
            .unwrap()
            .trim()
            .to_string()
    }
}

fn combine_bmp(
    buf: &mut impl Write,
    bmf_header: &BITMAPFILEHEADER,
    bmi: &BITMAPINFO,
    data: &Vec<u8>,
) -> Result<()> {
    buf.write(&bmf_header.bfType.to_le_bytes())?;
    buf.write(&bmf_header.bfSize.to_le_bytes())?;
    buf.write(&bmf_header.bfReserved1.to_le_bytes())?;
    buf.write(&bmf_header.bfReserved2.to_le_bytes())?;
    buf.write(&bmf_header.bfOffBits.to_le_bytes())?;
    buf.write(&bmi.bmiHeader.biSize.to_le_bytes())?;
    buf.write(&bmi.bmiHeader.biWidth.to_le_bytes())?;
    buf.write(&bmi.bmiHeader.biHeight.to_le_bytes())?;
    buf.write(&bmi.bmiHeader.biPlanes.to_le_bytes())?;
    buf.write(&bmi.bmiHeader.biBitCount.to_le_bytes())?;
    buf.write(&bmi.bmiHeader.biCompression.to_le_bytes())?;
    buf.write(&bmi.bmiHeader.biSizeImage.to_le_bytes())?;
    buf.write(&bmi.bmiHeader.biXPelsPerMeter.to_le_bytes())?;
    buf.write(&bmi.bmiHeader.biYPelsPerMeter.to_le_bytes())?;
    buf.write(&bmi.bmiHeader.biClrUsed.to_le_bytes())?;
    buf.write(&bmi.bmiHeader.biClrImportant.to_le_bytes())?;
    buf.write(&data)?;
    Ok(())
}

fn write_bmp(
    path: &str,
    bmf_header: &BITMAPFILEHEADER,
    bmi: &BITMAPINFO,
    data: &Vec<u8>,
) -> Result<()> {
    let file = std::fs::File::create(path)?;
    combine_bmp(&mut BufWriter::new(file), &bmf_header, &bmi, &data)
}

#[cfg(test)]
mod test {
    use crate::{ocr, region::Region};
    use leptonica_sys::{pixFreeData, pixRead};
    use std::ffi::CStr;

    fn ocr_from_file(path: &[u8]) -> String {
        unsafe {
            let image = pixRead(CStr::from_bytes_with_nul_unchecked(path).as_ptr());
            let s = ocr(image, None);
            pixFreeData(image);
            s
        }
    }
    fn ocr_from_file_with_region(path: &[u8], region: Region) -> String {
        unsafe {
            let image = pixRead(CStr::from_bytes_with_nul_unchecked(path).as_ptr());
            let s = ocr(image, Some(region));
            pixFreeData(image);
            s
        }
    }

    #[test]
    fn ocr_test() {
        assert_eq!("DUEL", ocr_from_file(b"tests/duel.bmp\0"));
        assert_eq!("Drytron Alpha Thuban", ocr_from_file(b"tests/deck.bmp\0"));
        assert_eq!(
            "Drytron Alpha",
            ocr_from_file_with_region(
                b"tests/deck.bmp\0",
                Region {
                    left: 0,
                    top: 0,
                    right: 190,
                    bottom: 33
                }
            )
        );
    }
}
