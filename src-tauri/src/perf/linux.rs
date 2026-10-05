#![allow(non_snake_case)]

use std::collections::HashMap;
use std::fs;

use super::Usage;

struct Row {
    Parent: u32,
    Ticks: u64,
}

fn _Conf(Name: i32) -> u64 {
    let Value = unsafe { libc::sysconf(Name) };
    if Value > 0 { Value as u64 } else { 0 }
}

fn _Row(Pid: u32) -> Option<Row> {
    let Stat = fs::read_to_string(format!("/proc/{}/stat", Pid)).ok()?;
    let Tail = Stat.get(Stat.rfind(')')? + 2..)?;
    let Parts: Vec<&str> = Tail.split_whitespace().collect();

    let Parent: u32 = Parts.get(1)?.parse().ok()?;
    let User: u64 = Parts.get(11)?.parse().ok()?;
    let Sys: u64 = Parts.get(12)?.parse().ok()?;

    Some(Row { Parent, Ticks: User + Sys })
}

fn _Shared(Pid: u32) -> Option<u64> {
    let Rollup = fs::read_to_string(format!("/proc/{}/smaps_rollup", Pid)).ok()?;

    Rollup
        .lines()
        .find(|L| L.starts_with("Pss:"))
        .and_then(|L| L.split_whitespace().nth(1))
        .and_then(|V| V.parse::<u64>().ok())
        .map(|Kb| Kb * 1024)
}

fn _Resident(Pid: u32, Page: u64) -> u64 {
    fs::read_to_string(format!("/proc/{}/statm", Pid))
        .ok()
        .and_then(|S| S.split_whitespace().nth(1)?.parse::<u64>().ok())
        .map(|Pages| Pages.saturating_mul(Page))
        .unwrap_or(0)
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
    let Tick = _Conf(libc::_SC_CLK_TCK);
    let Page = _Conf(libc::_SC_PAGESIZE);
    if Tick == 0 || Page == 0 {
        return None;
    }

    let Root = std::process::id();
    let mut Rows: HashMap<u32, Row> = HashMap::new();

    for Entry in fs::read_dir("/proc").ok()?.flatten() {
        let Name = Entry.file_name();

        let Pid: u32 = match Name.to_str().and_then(|N| N.parse().ok()) {
            Some(P) => P,
            None => continue,
        };

        if let Some(Found) = _Row(Pid) {
            Rows.insert(Pid, Found);
        }
    }

    Rows.get(&Root)?;

    let Ours: Vec<u32> = Rows
        .keys()
        .copied()
        .filter(|Pid| *Pid == Root || _Descends(&Rows, *Pid, Root))
        .collect();

    let mut Ticks: u64 = 0;
    let mut Bytes: u64 = 0;

    for Pid in Ours {
        if let Some(Row) = Rows.get(&Pid) {
            Ticks += Row.Ticks;
        }

        let Used = _Shared(Pid).unwrap_or_else(|| _Resident(Pid, Page));
        Bytes = Bytes.saturating_add(Used);
    }

    Some(Usage {
        CpuNanos: Ticks as u128 * 1_000_000_000 / Tick as u128,
        RssBytes: Bytes,
    })
}

