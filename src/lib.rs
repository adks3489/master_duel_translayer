use anyhow::{anyhow, Context, Result};
use std::io::Write;
use sysinfo::{PidExt, ProcessExt, System, SystemExt};
use winapi::{
    shared::windef::HWND,
    um::{
        wingdi::{BITMAPFILEHEADER, BITMAPINFO},
        winnt::HANDLE,
    },
};
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
    }
    Ok(())
}

fn write_bmp(
    path: &str,
    bmf_header: &BITMAPFILEHEADER,
    bmi: &BITMAPINFO,
    data: &Vec<u8>,
) -> Result<(), anyhow::Error> {
    let mut file = std::fs::File::create(path)?;
    file.write(&bmf_header.bfType.to_le_bytes())?;
    file.write(&bmf_header.bfSize.to_le_bytes())?;
    file.write(&bmf_header.bfReserved1.to_le_bytes())?;
    file.write(&bmf_header.bfReserved2.to_le_bytes())?;
    file.write(&bmf_header.bfOffBits.to_le_bytes())?;
    file.write(&bmi.bmiHeader.biSize.to_le_bytes())?;
    file.write(&bmi.bmiHeader.biWidth.to_le_bytes())?;
    file.write(&bmi.bmiHeader.biHeight.to_le_bytes())?;
    file.write(&bmi.bmiHeader.biPlanes.to_le_bytes())?;
    file.write(&bmi.bmiHeader.biBitCount.to_le_bytes())?;
    file.write(&bmi.bmiHeader.biCompression.to_le_bytes())?;
    file.write(&bmi.bmiHeader.biSizeImage.to_le_bytes())?;
    file.write(&bmi.bmiHeader.biXPelsPerMeter.to_le_bytes())?;
    file.write(&bmi.bmiHeader.biYPelsPerMeter.to_le_bytes())?;
    file.write(&bmi.bmiHeader.biClrUsed.to_le_bytes())?;
    file.write(&bmi.bmiHeader.biClrImportant.to_le_bytes())?;
    file.write(&data)?;
    Ok(())
}
