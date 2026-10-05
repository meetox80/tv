#![allow(non_snake_case, non_camel_case_types)]

use std::collections::HashMap;
use std::mem;

use super::Usage;

type HANDLE = isize;

const INVALID_HANDLE: HANDLE = -1;
const TH32CS_SNAPPROCESS: u32 = 0x0000_0002;
const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;
const PROCESS_VM_READ: u32 = 0x0010;

#[repr(C)]
struct PROCESSENTRY32W {
    dwSize: u32,
    cntUsage: u32,
    th32ProcessID: u32,
    th32DefaultHeapID: usize,
    th32ModuleID: u32,
    cntThreads: u32,
    th32ParentProcessID: u32,
    pcPriClassBase: i32,
    dwFlags: u32,
    szExeFile: [u16; 260],
}

#[repr(C)]
struct PROCESS_MEMORY_COUNTERS_EX {
    cb: u32,
    PageFaultCount: u32,
    PeakWorkingSetSize: usize,
    WorkingSetSize: usize,
    QuotaPeakPagedPoolUsage: usize,
    QuotaPagedPoolUsage: usize,
    QuotaPeakNonPagedPoolUsage: usize,
    QuotaNonPagedPoolUsage: usize,
    PagefileUsage: usize,
    PeakPagefileUsage: usize,
    PrivateUsage: usize,
}

#[link(name = "kernel32")]
extern "system" {
    fn CreateToolhelp32Snapshot(Flags: u32, Pid: u32) -> HANDLE;
    fn Process32FirstW(Snapshot: HANDLE, Entry: *mut PROCESSENTRY32W) -> i32;
    fn Process32NextW(Snapshot: HANDLE, Entry: *mut PROCESSENTRY32W) -> i32;
    fn OpenProcess(Access: u32, Inherit: i32, Pid: u32) -> HANDLE;
    fn CloseHandle(Object: HANDLE) -> i32;
    fn GetProcessTimes(
        Process: HANDLE,
        Creation: *mut u64,
        Exit: *mut u64,
        Kernel: *mut u64,
        User: *mut u64,
    ) -> i32;
    fn K32GetProcessMemoryInfo(
        Process: HANDLE,
        Counters: *mut PROCESS_MEMORY_COUNTERS_EX,
        Size: u32,
    ) -> i32;
}

fn _Parents() -> Option<HashMap<u32, u32>> {
    let Snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if Snapshot == INVALID_HANDLE {
        return None;
    }

    let mut Entry: PROCESSENTRY32W = unsafe { mem::zeroed() };
    Entry.dwSize = mem::size_of::<PROCESSENTRY32W>() as u32;

    let mut Tree: HashMap<u32, u32> = HashMap::new();
    let mut More = unsafe { Process32FirstW(Snapshot, &mut Entry) };

    while More != 0 {
        Tree.insert(Entry.th32ProcessID, Entry.th32ParentProcessID);
        More = unsafe { Process32NextW(Snapshot, &mut Entry) };
    }

    unsafe { CloseHandle(Snapshot) };
    Some(Tree)
}

fn _Usage(Pid: u32) -> Option<(u128, u64)> {
    let Process = unsafe {
        OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_VM_READ, 0, Pid)
    };

    if Process == 0 {
        return None;
    }

    let mut Creation: u64 = 0;
    let mut Exit: u64 = 0;
    let mut Kernel: u64 = 0;
    let mut User: u64 = 0;

    let Timed = unsafe {
        GetProcessTimes(Process, &mut Creation, &mut Exit, &mut Kernel, &mut User)
    };

    let mut Counters: PROCESS_MEMORY_COUNTERS_EX = unsafe { mem::zeroed() };
    Counters.cb = mem::size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32;

    let Measured = unsafe {
        K32GetProcessMemoryInfo(Process, &mut Counters, Counters.cb)
    };

    unsafe { CloseHandle(Process) };

    if Timed == 0 || Measured == 0 {
        return None;
    }

    Some((
        (Kernel as u128 + User as u128) * 100,
        Counters.PrivateUsage as u64,
    ))
}

fn _Descends(Tree: &HashMap<u32, u32>, Pid: u32, Root: u32) -> bool {
    let mut At = Pid;

    for _ in 0..64 {
        let Parent = match Tree.get(&At) {
            Some(P) => *P,
            None => return false,
        };

        if Parent == Root {
            return true;
        }

        if Parent == 0 || Parent == At {
            return false;
        }

        At = Parent;
    }

    false
}

pub fn Sample() -> Option<Usage> {
    let Root = std::process::id();
    let Tree = _Parents()?;

    let mut Nanos: u128 = 0;
    let mut Rss: u64 = 0;

    for Pid in Tree.keys() {
        if *Pid != Root && !_Descends(&Tree, *Pid, Root) {
            continue;
        }

        if let Some((Cpu, Memory)) = _Usage(*Pid) {
            Nanos += Cpu;
            Rss = Rss.saturating_add(Memory);
        }
    }

    Some(Usage { CpuNanos: Nanos, RssBytes: Rss })
}
