#![allow(non_snake_case)]

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::AppState;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod platform;

#[cfg(target_os = "macos")]
#[path = "macos.rs"]
mod platform;

#[cfg(target_os = "windows")]
#[path = "windows.rs"]
mod platform;

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
#[path = "other.rs"]
mod platform;

pub struct Usage {
    pub CpuNanos: u128,
    pub RssBytes: u64,
}

pub const HISTORY: usize = 60;

const SAMPLE_MS: u64 = 1000;
const FRESH_MS: u128 = 8000;

#[derive(Default)]
pub struct Frames {
    pub Fps: f64,
    pub Low: f64,
    pub AtMs: u128,
}

pub struct Meter {
    Cpu: VecDeque<f32>,
    Rss: VecDeque<u64>,
    Up: VecDeque<u64>,
    Down: VecDeque<u64>,
    Supported: bool,
}

impl Default for Meter {
    fn default() -> Self {
        Self {
            Cpu: VecDeque::with_capacity(HISTORY),
            Rss: VecDeque::with_capacity(HISTORY),
            Up: VecDeque::with_capacity(HISTORY),
            Down: VecDeque::with_capacity(HISTORY),
            Supported: true,
        }
    }
}

fn _Keep<T>(Series: &mut VecDeque<T>, Value: T) {
    if Series.len() == HISTORY {
        Series.pop_front();
    }
    Series.push_back(Value);
}

impl Meter {
    fn Push(&mut self, Cpu: f32, Rss: u64) {
        _Keep(&mut self.Cpu, Cpu.clamp(0.0, 100.0));
        _Keep(&mut self.Rss, Rss);
    }

    fn PushWire(&mut self, Up: u64, Down: u64) {
        _Keep(&mut self.Up, Up);
        _Keep(&mut self.Down, Down);
    }
}

pub type Shared = Mutex<Meter>;

fn _NowMs() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|D| D.as_millis())
        .unwrap_or(0)
}

fn _Cores() -> f64 {
    std::thread::available_parallelism()
        .map(|N| N.get() as f64)
        .unwrap_or(1.0)
}

pub fn Snapshot(State: &Arc<AppState>) -> serde_json::Value {
    let Voting = State.Voting.Activity.Json(&State.Db);
    let Meter = State.Meter.lock().unwrap();
    let Frames = State.Frames.lock().unwrap();

    let Fresh = Frames.AtMs > 0 && _NowMs().saturating_sub(Frames.AtMs) < FRESH_MS;
    let Cpu: Vec<f32> = Meter.Cpu.iter().map(|V| (V * 10.0).round() / 10.0).collect();

    serde_json::json!({
        "Supported": Meter.Supported,
        "Cpu": Cpu,
        "Rss": Meter.Rss.iter().collect::<Vec<_>>(),
        "Fps": if Fresh { Some((Frames.Fps * 10.0).round() / 10.0) } else { None },
        "Low": if Fresh { Some((Frames.Low * 10.0).round() / 10.0) } else { None },
        "History": HISTORY,
        "Network": {
            "Up": Meter.Up.iter().collect::<Vec<_>>(),
            "Down": Meter.Down.iter().collect::<Vec<_>>(),
            "Sent": State.Traffic.Sent(),
            "Received": State.Traffic.Received(),
            "Requests": State.Traffic.Requests(),
        },
        "Voting": Voting,
    })
}

fn _Rate(Bytes: u64, Wall: Duration) -> u64 {
    let Seconds = Wall.as_secs_f64();
    if Seconds <= 0.0 { 0 } else { (Bytes as f64 / Seconds).round() as u64 }
}

pub fn Spawn(State: Arc<AppState>) {
    std::thread::spawn(move || {
        let Cores = _Cores();
        let mut Previous: Option<(u128, Instant)> = None;
        let mut Wire = (State.Traffic.Sent(), State.Traffic.Received(), Instant::now());

        loop {
            let Usage = platform::Sample();
            let At = Instant::now();
            let (Sent, Received) = (State.Traffic.Sent(), State.Traffic.Received());
            let Wall = At.duration_since(Wire.2);
            let (Up, Down) = (_Rate(Sent.saturating_sub(Wire.0), Wall), _Rate(Received.saturating_sub(Wire.1), Wall));
            Wire = (Sent, Received, At);

            let mut Ticked = Previous.is_some();
            {
                let mut Meter = State.Meter.lock().unwrap();
                match Usage {
                    Some(Now) => {
                        if let Some((Cpu, When)) = Previous {
                            let Elapsed = At.duration_since(When).as_nanos();
                            let Burned = Now.CpuNanos.saturating_sub(Cpu);
                            let Share = if Elapsed == 0 { 0.0 } else { (Burned as f64 / Elapsed as f64) / Cores * 100.0 };
                            Meter.Push(Share as f32, Now.RssBytes);
                        }
                        Previous = Some((Now.CpuNanos, At));
                    }
                    None => {
                        Meter.Supported = false;
                        Ticked = true;
                    }
                }
                if Ticked {
                    Meter.PushWire(Up, Down);
                }
            }

            if Ticked {
                State.PerfTick.send_modify(|Count| *Count += 1);
            }

            std::thread::sleep(Duration::from_millis(SAMPLE_MS));
        }
    });
}

pub fn Report(State: &Arc<AppState>, Fps: f64, Low: f64) {
    let mut Held = State.Frames.lock().unwrap();
    Held.Fps = Fps.max(0.0);
    Held.Low = Low.max(0.0);
    Held.AtMs = _NowMs();
}
