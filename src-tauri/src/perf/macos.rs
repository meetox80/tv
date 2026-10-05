#![allow(non_snake_case)]

use std::collections::HashMap;
use std::mem;

use super::Usage;

struct Row {
    Parent: u32,
    Nanos: u128,
    Rss: u64,
}

fn _Pids() -> Option<Vec<u32>> {
    let Bytes = unsafe { libc::proc_listpids(libc::PROC_ALL_PIDS, 0, std::ptr::null_mut(), 0) };
    if Bytes <= 0 {
        return None;
    }

    let Room = Bytes as usize / mem::size_of::<i32>() + 64;
    let mut Buffer: Vec<i32> = vec![0; Room];

    let Written = unsafe {
        libc::proc_listpids(
            libc::PROC_ALL_PIDS,
            0,
            Buffer.as_mut_ptr() as *mut libc::c_void,
            (Buffer.len() * mem::size_of::<i32>()) as i32,
        )
    };

    if Written <= 0 {
        return None;
    }

    let Count = Written as usize / mem::size_of::<i32>();
    Some(Buffer[..Count].iter().filter(|P| **P > 0).map(|P| *P as u32).collect())
}

fn _Parent(Pid: u32) -> Option<u32> {
    let mut Info: libc::proc_bsdinfo = unsafe { mem::zeroed() };
    let Size = mem::size_of::<libc::proc_bsdinfo>() as i32;

    let Read = unsafe {
        libc::proc_pidinfo(
            Pid as i32,
            libc::PROC_PIDTBSDINFO,
            0,
            &mut Info as *mut _ as *mut libc::c_void,
            Size,
        )
    };

    if Read != Size {
        return None;
    }

    Some(Info.pbi_ppid)
}

fn _Task(Pid: u32) -> Option<(u128, u64)> {
    let mut Info: libc::proc_taskinfo = unsafe { mem::zeroed() };
    let Size = mem::size_of::<libc::proc_taskinfo>() as i32;

    let Read = unsafe {
        libc::proc_pidinfo(
            Pid as i32,
            libc::PROC_PIDTASKINFO,
            0,
            &mut Info as *mut _ as *mut libc::c_void,
            Size,
        )
    };

    if Read != Size {
        return None;
    }

    Some((
        Info.pti_total_user as u128 + Info.pti_total_system as u128,
        Info.pti_resident_size,
    ))
}

fn _Descends(Rows: &HashMap<u32, Row>, Pid: u32, Root: u32) -> bool {
    let mut At = Pid;

    for _ in 0..64 {
        let Parent = match Rows.get(&At) {
            Some(R) => R.Parent,
            None => return false,
        };

        if Parent == Root {
            return true;
        }

        if Parent <= 1 {
            return false;
        }

        At = Parent;
    }

    false
}

pub fn Sample() -> Option<Usage> {
    let Root = std::process::id();
    let mut Rows: HashMap<u32, Row> = HashMap::new();

    for Pid in _Pids()? {
        let Parent = match _Parent(Pid) {
            Some(P) => P,
            None => continue,
        };

        let (Nanos, Rss) = match _Task(Pid) {
            Some(T) => T,
            None => continue,
        };

        Rows.insert(Pid, Row { Parent, Nanos, Rss });
    }

    Rows.get(&Root)?;

    let mut Nanos: u128 = 0;
    let mut Rss: u64 = 0;

    for (Pid, Row) in &Rows {
        if *Pid == Root || _Descends(&Rows, *Pid, Root) {
            Nanos += Row.Nanos;
            Rss = Rss.saturating_add(Row.Rss);
        }
    }

    Some(Usage { CpuNanos: Nanos, RssBytes: Rss })
}
