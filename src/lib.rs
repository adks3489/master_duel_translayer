use anyhow::{anyhow, Context, Result};
use sysinfo::{PidExt, ProcessExt, System, SystemExt};
use winapi::{shared::windef::HWND, um::winnt::HANDLE};
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
        handle: win_util::open_process(0x400, pid).with_context(|| format!("Unable to get handle of {}", PROCESS_NAME))?,
        hwnd: win_util::get_by_pid(pid).with_context(|| format!("Unable to get hwnd of {}", PROCESS_NAME))?,
    })
}
