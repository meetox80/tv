#![allow(non_snake_case)]

use std::backtrace::Backtrace;
use std::error::Error;
use std::fmt;
use std::fmt::Write as _;
use std::io::IsTerminal;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;

static BOOT: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy)]
pub enum Subsystem {
    Core,
    Server,
    Cloudflared,
    Auth,
    Lastfm,
    Spotify,
    Serial,
    Metadata,
    Voting,
    Extensions,
    News,
}

impl fmt::Display for Subsystem {
    fn fmt(&self, F: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Subsystem::Core => write!(F, "core"),
            Subsystem::Server => write!(F, "server"),
            Subsystem::Cloudflared => write!(F, "cloudflared"),
            Subsystem::Auth => write!(F, "auth"),
            Subsystem::Lastfm => write!(F, "lastfm"),
            Subsystem::Spotify => write!(F, "spotify"),
            Subsystem::Serial => write!(F, "serial"),
            Subsystem::Metadata => write!(F, "metadata"),
            Subsystem::Voting => write!(F, "voting"),
            Subsystem::Extensions => write!(F, "extensions"),
            Subsystem::News => write!(F, "news"),
        }
    }
}

#[derive(Clone, Copy)]
pub enum Level {
    Info,
    Warn,
    Error,
    Debug,
}

impl fmt::Display for Level {
    fn fmt(&self, F: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Level::Info => write!(F, "info"),
            Level::Warn => write!(F, "warn"),
            Level::Error => write!(F, "error"),
            Level::Debug => write!(F, "debug"),
        }
    }
}

fn _Uptime() -> f64 {
    let Now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    let Boot = BOOT.load(Ordering::Relaxed);
    if Boot == 0 {
        BOOT.store(Now, Ordering::Relaxed);
        return 0.0;
    }

    (Now - Boot) as f64 / 1000.0
}

struct Paint(bool);

impl Paint {
    fn On(&self, Code: &str, Text: &str) -> String {
        if self.0 { format!("\x1b[{}m{}\x1b[0m", Code, Text) } else { Text.to_string() }
    }
}

fn _Paint() -> Paint {
    Paint(std::io::stderr().is_terminal() && std::env::var_os("NO_COLOR").is_none())
}

pub fn _Emit(Sys: Subsystem, Lvl: Level, Msg: &str) {
    let P = _Paint();
    let Tag = format!("{}/{}", Sys, Lvl);
    let Tag = match Lvl {
        Level::Error => P.On("31", &Tag),
        Level::Warn => P.On("33", &Tag),
        Level::Debug => P.On("2", &Tag),
        Level::Info => Tag,
    };
    eprintln!("{} {}: {}", P.On("2", &format!("[{:>10.3}]", _Uptime())), Tag, Msg);
}

fn _Frames() -> Vec<(String, String)> {
    let Trace = Backtrace::force_capture().to_string();
    let mut Frames: Vec<(String, String)> = Vec::new();
    let mut Lines = Trace.lines().map(str::trim).peekable();

    while let Some(Line) = Lines.next() {
        let Some((Index, Symbol)) = Line.split_once(": ") else { continue };
        if !Index.chars().all(|C| C.is_ascii_digit()) { continue }
        let Site = match Lines.peek() {
            Some(Next) if Next.starts_with("at ") => Lines.next().unwrap()[3..].to_string(),
            _ => String::new(),
        };
        let Site = Site.rsplit_once("src-tauri/").map(|(_, Rest)| Rest.to_string()).unwrap_or(Site);
        let Site = Site.strip_prefix("./").map(str::to_string).unwrap_or(Site);
        if Site.is_empty() || Site.starts_with('/') || Symbol.contains("::log::_") { continue }
        let Symbol = Symbol.split("::{closure").next().unwrap_or(Symbol).to_string();
        if Frames.last().is_some_and(|(Last, _)| *Last == Symbol) { continue }
        Frames.push((Symbol, Site));
    }

    Frames.truncate(8);
    Frames
}

pub fn _Fail(Sys: Subsystem, Msg: &str, Site: &str, Cause: Option<&dyn Error>) {
    let P = _Paint();
    let mut Rows: Vec<(&str, String)> = vec![("at", Site.to_string())];

    let mut Next = Cause;
    while let Some(E) = Next {
        Rows.push((if Rows.len() == 1 { "cause" } else { "from" }, E.to_string()));
        Next = E.source();
    }

    let Frames = _Frames();
    let Bar = P.On("31", "│");
    let mut Out = String::new();

    let _ = writeln!(Out);
    let _ = writeln!(
        Out,
        "{} {} {}  {}",
        P.On("2", &format!("[{:>10.3}]", _Uptime())),
        P.On("1;97;41", " error "),
        P.On("31", &Sys.to_string()),
        P.On("1", Msg),
    );
    for (Key, Value) in &Rows {
        let _ = writeln!(Out, "             {} {}  {}", Bar, P.On("2", &format!("{:<5}", Key)), Value);
    }
    if !Frames.is_empty() {
        let _ = writeln!(Out, "             {} {}", Bar, P.On("2", "stack"));
        for (Symbol, Location) in &Frames {
            let _ = writeln!(Out, "             {}   {}  {}", Bar, Symbol, P.On("2", Location));
        }
    }

    eprint!("{}", Out);
}

pub fn _InstallPanicHook() {
    std::panic::set_hook(Box::new(|Info| {
        let Payload = Info.payload();
        let Msg = Payload.downcast_ref::<&str>().map(|S| S.to_string())
            .or_else(|| Payload.downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "panic".to_string());
        let Site = Info.location().map(|L| format!("{}:{}", L.file(), L.line())).unwrap_or_default();
        let Thread = std::thread::current().name().unwrap_or("unnamed").to_string();
        _Fail(Subsystem::Core, &format!("panicked on thread {}: {}", Thread, Msg), &Site, None);
    }));
}

#[macro_export]
macro_rules! log {
    ($sys:expr, $lvl:expr, $($arg:tt)*) => {
        $crate::log::_Emit($sys, $lvl, &format!($($arg)*))
    };
}

#[macro_export]
macro_rules! fail {
    ($sys:expr, $err:expr, $($arg:tt)*) => {
        $crate::log::_Fail($sys, &format!($($arg)*), concat!(file!(), ":", line!()), Some(&$err as &dyn std::error::Error))
    };
}
