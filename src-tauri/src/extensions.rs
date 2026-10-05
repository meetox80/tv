#![allow(non_snake_case)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use rquickjs::{CatchResultExt, CaughtError, Context, Ctx, Function, Module, Object, Persistent, Runtime, Value};
use serde_json::{json, Value as Json};

use crate::db::Database;
use crate::log::{Subsystem::Extensions, Level::*};

const MANIFEST: &str = "extension.json";
const ENABLED_KEY: &str = "extensions_enabled";
const MEMORY_LIMIT: usize = 64 * 1024 * 1024;
const STACK_LIMIT: usize = 1024 * 1024;
const START_BUDGET: Duration = Duration::from_secs(5);
const STOP_BUDGET: Duration = Duration::from_secs(2);
const SOURCE_LIMIT: u64 = 4 * 1024 * 1024;

#[derive(Clone, serde::Deserialize)]
pub struct Manifest {
    #[serde(rename = "Id")]
    pub Id: String,
    #[serde(rename = "Name")]
    pub Name: String,
    #[serde(rename = "Version")]
    pub Version: String,
    #[serde(rename = "Description", default)]
    pub Description: Option<String>,
    #[serde(rename = "Panel", default)]
    pub Panel: Option<String>,
    #[serde(rename = "Server", default)]
    pub Server: Option<String>,
}

type Teardown = Option<(Persistent<Function<'static>>, Persistent<Object<'static>>)>;

struct Worker {
    Stop: Arc<AtomicBool>,
    Deadline: Arc<Mutex<Option<Instant>>>,
    Commands: Sender<()>,
}

impl Worker {
    fn Shutdown(self) {
        *self.Deadline.lock().unwrap() = Some(Instant::now() + STOP_BUDGET);
        if self.Commands.send(()).is_err() {
            self.Stop.store(true, Ordering::Relaxed);
        }
    }
}

struct Entry {
    Manifest: Manifest,
    Dir: PathBuf,
    Problem: Arc<Mutex<Option<String>>>,
    Worker: Option<Worker>,
}

pub struct Registry {
    Root: PathBuf,
    Entries: Mutex<Vec<Entry>>,
}

fn _ValidId(Id: &str) -> bool {
    (1..=40).contains(&Id.len()) && Id.bytes().all(|B| B.is_ascii_lowercase() || B.is_ascii_digit() || B == b'-')
}

fn _Inside(Dir: &Path, Relative: &str) -> Option<PathBuf> {
    let Base = Dir.canonicalize().ok()?;
    let Full = Base.join(Relative).canonicalize().ok()?;
    (Full.starts_with(&Base) && Full.is_file()).then_some(Full)
}

fn _Read(Dir: &Path, Relative: &str) -> Result<String, String> {
    let Path = _Inside(Dir, Relative).ok_or_else(|| format!("nie znaleziono pliku {}", Relative))?;
    let Size = std::fs::metadata(&Path).map_err(|E| E.to_string())?.len();
    if Size > SOURCE_LIMIT {
        return Err(format!("plik {} jest za duży", Relative));
    }
    std::fs::read_to_string(&Path).map_err(|E| format!("nie można odczytać {}: {}", Relative, E))
}

fn _Enabled(Db: &Database) -> BTreeSet<String> {
    Db.Get(ENABLED_KEY)
        .and_then(|Raw| serde_json::from_str::<Vec<String>>(&Raw).ok())
        .unwrap_or_default()
        .into_iter()
        .collect()
}

fn _SaveEnabled(Db: &Database, Set: &BTreeSet<String>) {
    match serde_json::to_string(Set) {
        Ok(Raw) => Db.Set(ENABLED_KEY, &Raw),
        Err(E) => { fail!(Extensions, E, "could not encode enabled extensions"); }
    }
}

fn _Describe(Caught: CaughtError<'_>) -> String {
    match Caught {
        CaughtError::Exception(E) => {
            let Message = E.message().unwrap_or_else(|| "nieznany błąd".to_string());
            match E.stack().filter(|S| !S.trim().is_empty()) {
                Some(Stack) => format!("{}\n{}", Message, Stack.trim_end()),
                None => Message,
            }
        }
        CaughtError::Value(V) => format!("{:?}", V),
        CaughtError::Error(E) => {
            if matches!(E, rquickjs::Error::Exception) {
                "przerwano wykonywanie (przekroczony czas lub pamięć)".to_string()
            } else {
                E.to_string()
            }
        }
    }
}

fn _Host<'js>(Ctx: &Ctx<'js>, Id: &str, Manifest: &Manifest) -> rquickjs::Result<Object<'js>> {
    let Host = Object::new(Ctx.clone())?;
    Host.set("id", Id)?;
    Host.set("version", Manifest.Version.as_str())?;
    let Tag = Id.to_string();
    Host.set("log", Function::new(Ctx.clone(), move |Message: String| {
        log!(Extensions, Info, "[{}] {}", Tag, Message);
    })?)?;
    Ok(Host)
}

fn _Settle<'js>(Result: Value<'js>) -> rquickjs::Result<()> {
    match Result.into_promise() {
        Some(Promise) => Promise.finish::<()>(),
        None => Ok(()),
    }
}

fn _Run(Id: String, Manifest: Manifest, Source: String, Problem: Arc<Mutex<Option<String>>>, Stop: Arc<AtomicBool>, Deadline: Arc<Mutex<Option<Instant>>>, Commands: Receiver<()>) {
    let Fail = |Message: String| {
        log!(Extensions, Error, "[{}] {}", Id, Message);
        *Problem.lock().unwrap() = Some(Message);
    };

    let Runtime = match Runtime::new() {
        Ok(R) => R,
        Err(E) => return Fail(format!("nie można uruchomić silnika: {}", E)),
    };
    Runtime.set_memory_limit(MEMORY_LIMIT);
    Runtime.set_max_stack_size(STACK_LIMIT);

    let Watch = (Arc::clone(&Stop), Arc::clone(&Deadline));
    Runtime.set_interrupt_handler(Some(Box::new(move || {
        Watch.0.load(Ordering::Relaxed) || Watch.1.lock().unwrap().is_some_and(|At| Instant::now() >= At)
    })));

    let Context = match Context::full(&Runtime) {
        Ok(C) => C,
        Err(E) => return Fail(format!("nie można utworzyć kontekstu: {}", E)),
    };

    *Deadline.lock().unwrap() = Some(Instant::now() + START_BUDGET);
    let Started: Result<Teardown, String> = Context.with(|Ctx| {
        let Host = _Host(&Ctx, &Id, &Manifest).catch(&Ctx).map_err(_Describe)?;
        let (Loaded, Evaluated) = Module::declare(Ctx.clone(), format!("{}/{}", Id, Manifest.Server.as_deref().unwrap_or("server.js")), Source)
            .and_then(|Declared| Declared.eval())
            .catch(&Ctx)
            .map_err(_Describe)?;
        Evaluated.finish::<()>().catch(&Ctx).map_err(_Describe)?;

        if let Ok(Activate) = Loaded.get::<_, Function>("activate") {
            let Result: Value = Activate.call((Host.clone(),)).catch(&Ctx).map_err(_Describe)?;
            _Settle(Result).catch(&Ctx).map_err(_Describe)?;
        }

        Ok(Loaded.get::<_, Function>("deactivate").ok().map(|Deactivate| {
            (Persistent::save(&Ctx, Deactivate), Persistent::save(&Ctx, Host))
        }))
    });
    *Deadline.lock().unwrap() = None;

    let Teardown = match Started {
        Ok(T) => T,
        Err(E) => return Fail(E),
    };
    log!(Extensions, Info, "[{}] server part running", Id);

    let _ = Commands.recv();

    if let Some((Deactivate, Host)) = Teardown {
        let Outcome: Result<(), String> = Context.with(|Ctx| {
            let Deactivate = Deactivate.restore(&Ctx).catch(&Ctx).map_err(_Describe)?;
            let Host = Host.restore(&Ctx).catch(&Ctx).map_err(_Describe)?;
            let Result: Value = Deactivate.call((Host,)).catch(&Ctx).map_err(_Describe)?;
            _Settle(Result).catch(&Ctx).map_err(_Describe)
        });
        if let Err(E) = Outcome {
            log!(Extensions, Warn, "[{}] deactivate failed: {}", Id, E);
        }
    }

    log!(Extensions, Info, "[{}] server part stopped", Id);
}

impl Registry {
    pub fn Open(Root: PathBuf) -> Self {
        if let Err(E) = std::fs::create_dir_all(&Root) {
            fail!(Extensions, E, "could not create extensions folder {:?}", Root);
        }
        Self { Root, Entries: Mutex::new(Vec::new()) }
    }

    fn _Discover(&self) -> Vec<(Manifest, PathBuf)> {
        let Folders = match std::fs::read_dir(&self.Root) {
            Ok(F) => F,
            Err(E) => {
                fail!(Extensions, E, "could not read extensions folder {:?}", self.Root);
                return Vec::new();
            }
        };

        let mut Found: Vec<(Manifest, PathBuf)> = Vec::new();
        for Folder in Folders.flatten() {
            let Dir = Folder.path();
            if !Dir.is_dir() {
                continue;
            }
            let Raw = match std::fs::read_to_string(Dir.join(MANIFEST)) {
                Ok(R) => R,
                Err(_) => continue,
            };
            let Manifest: Manifest = match serde_json::from_str(&Raw) {
                Ok(M) => M,
                Err(E) => {
                    log!(Extensions, Warn, "invalid manifest in {:?}: {}", Dir, E);
                    continue;
                }
            };
            let Named = Dir.file_name().and_then(|N| N.to_str()) == Some(Manifest.Id.as_str());
            if !_ValidId(&Manifest.Id) || !Named {
                log!(Extensions, Warn, "skipping {:?}: id must match the folder and use a-z, 0-9 and -", Dir);
                continue;
            }
            Found.push((Manifest, Dir));
        }
        Found.sort_by_key(|A| A.0.Name.to_lowercase());
        Found
    }

    pub fn Load(&self, Db: &Database) {
        let Enabled = _Enabled(Db);
        let Found = self._Discover();
        let mut Entries = self.Entries.lock().unwrap();
        for Old in Entries.drain(..) {
            if let Some(W) = Old.Worker {
                W.Shutdown();
            }
        }

        for (Manifest, Dir) in Found {
            let mut Entry = Entry { Manifest, Dir, Problem: Arc::new(Mutex::new(None)), Worker: None };
            if Enabled.contains(&Entry.Manifest.Id) {
                Self::_Start(&mut Entry);
            }
            Entries.push(Entry);
        }

        log!(Extensions, Info, "{} extension(s) found, {} enabled", Entries.len(), Entries.iter().filter(|E| Enabled.contains(&E.Manifest.Id)).count());
    }

    fn _Start(Entry: &mut Entry) {
        *Entry.Problem.lock().unwrap() = None;
        let Some(Relative) = Entry.Manifest.Server.clone() else { return };

        let Source = match _Read(&Entry.Dir, &Relative) {
            Ok(S) => S,
            Err(E) => {
                log!(Extensions, Error, "[{}] {}", Entry.Manifest.Id, E);
                *Entry.Problem.lock().unwrap() = Some(E);
                return;
            }
        };

        let Stop = Arc::new(AtomicBool::new(false));
        let Deadline = Arc::new(Mutex::new(None));
        let (Commands, Inbox) = mpsc::channel();
        let (Id, Manifest, Problem) = (Entry.Manifest.Id.clone(), Entry.Manifest.clone(), Arc::clone(&Entry.Problem));
        let (StopFlag, Timer) = (Arc::clone(&Stop), Arc::clone(&Deadline));

        let Spawned = std::thread::Builder::new()
            .name(format!("ext-{}", Id))
            .spawn(move || _Run(Id, Manifest, Source, Problem, StopFlag, Timer, Inbox));

        match Spawned {
            Ok(_) => Entry.Worker = Some(Worker { Stop, Deadline, Commands }),
            Err(E) => {
                fail!(Extensions, E, "could not start thread for {}", Entry.Manifest.Id);
                *Entry.Problem.lock().unwrap() = Some(format!("nie można uruchomić wątku: {}", E));
            }
        }
    }

    pub fn SetEnabled(&self, Db: &Database, Id: &str, On: bool) -> bool {
        let mut Entries = self.Entries.lock().unwrap();
        let Some(Entry) = Entries.iter_mut().find(|E| E.Manifest.Id == Id) else { return false };

        let mut Enabled = _Enabled(Db);
        if On { Enabled.insert(Id.to_string()); } else { Enabled.remove(Id); }
        _SaveEnabled(Db, &Enabled);

        if let Some(W) = Entry.Worker.take() {
            W.Shutdown();
        }
        if On {
            Self::_Start(Entry);
        } else {
            *Entry.Problem.lock().unwrap() = None;
        }

        log!(Extensions, Info, "[{}] {}", Id, if On { "enabled" } else { "disabled" });
        true
    }

    pub fn DisableAll(&self, Db: &Database) {
        _SaveEnabled(Db, &BTreeSet::new());
        let mut Entries = self.Entries.lock().unwrap();
        for Entry in Entries.iter_mut() {
            if let Some(W) = Entry.Worker.take() {
                W.Shutdown();
            }
            *Entry.Problem.lock().unwrap() = None;
        }
        log!(Extensions, Warn, "all extensions disabled");
    }

    pub fn List(&self, Db: &Database) -> Json {
        let Enabled = _Enabled(Db);
        let Entries = self.Entries.lock().unwrap();
        let Items: Vec<Json> = Entries.iter().map(|E| json!({
            "Id": E.Manifest.Id,
            "Name": E.Manifest.Name,
            "Version": E.Manifest.Version,
            "Description": E.Manifest.Description,
            "Enabled": Enabled.contains(&E.Manifest.Id),
            "Panel": E.Manifest.Panel.is_some(),
            "Server": E.Manifest.Server.is_some(),
            "Error": E.Problem.lock().unwrap().clone(),
        })).collect();
        json!({ "Folder": self.Root.display().to_string(), "Items": Items })
    }

    pub fn PanelSource(&self, Db: &Database, Id: &str) -> Option<String> {
        if !_Enabled(Db).contains(Id) {
            return None;
        }
        let Entries = self.Entries.lock().unwrap();
        let Entry = Entries.iter().find(|E| E.Manifest.Id == Id)?;
        let Relative = Entry.Manifest.Panel.as_deref()?;
        match _Read(&Entry.Dir, Relative) {
            Ok(Source) => Some(Source),
            Err(E) => {
                log!(Extensions, Warn, "[{}] {}", Id, E);
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _Manifest() -> Manifest {
        Manifest {
            Id: "probe".into(),
            Name: "Probe".into(),
            Version: "1.0.0".into(),
            Description: None,
            Panel: None,
            Server: Some("server.js".into()),
        }
    }

    fn _Launch(Source: &str) -> (Arc<Mutex<Option<String>>>, Sender<()>, std::thread::JoinHandle<()>) {
        let Problem = Arc::new(Mutex::new(None));
        let (Commands, Inbox) = mpsc::channel();
        let Stop = Arc::new(AtomicBool::new(false));
        let Deadline = Arc::new(Mutex::new(None));
        let (P, S) = (Arc::clone(&Problem), Source.to_string());
        let Thread = std::thread::spawn(move || _Run("probe".into(), _Manifest(), S, P, Stop, Deadline, Inbox));
        (Problem, Commands, Thread)
    }

    #[test]
    fn runs_activate_and_deactivate() {
        let (Problem, Commands, Thread) = _Launch("export function activate(Host) { Host.log('hello ' + Host.id) }\nexport function deactivate(Host) { Host.log('bye') }");
        std::thread::sleep(Duration::from_millis(200));
        Commands.send(()).unwrap();
        Thread.join().unwrap();
        assert_eq!(*Problem.lock().unwrap(), None);
    }

    #[test]
    fn reports_thrown_errors() {
        let (Problem, _Commands, Thread) = _Launch("export function activate() { throw new Error('boom') }");
        Thread.join().unwrap();
        assert!(Problem.lock().unwrap().as_deref().is_some_and(|P| P.starts_with("boom")));
    }

    #[test]
    fn interrupts_endless_loops() {
        let Started = Instant::now();
        let (Problem, _Commands, Thread) = _Launch("export function activate() { while (true) {} }");
        Thread.join().unwrap();
        assert!(Started.elapsed() < START_BUDGET + Duration::from_secs(2));
        assert!(Problem.lock().unwrap().is_some());
    }
}
