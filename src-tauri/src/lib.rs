#![allow(non_snake_case)]

#[macro_use]
pub mod log;
mod code;
mod cloudflared;
pub mod cloudflared_dl;
mod extensions;
pub mod cover;
pub mod db;
mod bpm;
mod lastfm;
mod lessons;
mod metadata;
mod news;
mod perf;
mod preview;
mod serial;
mod slides;
mod server;
mod spotify;
mod voting;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock, atomic::{AtomicBool, AtomicU8, AtomicU16, AtomicU64}};
use std::time::Instant;
use std::time::Duration;
use tauri::{Emitter, Manager};
use log::{Subsystem, Level};

const _WINDOW_REVEAL_FALLBACK: Duration = Duration::from_millis(800);
pub const SESSION_TTL: Duration = Duration::from_secs(15 * 60);

pub static STARTED_AT: OnceLock<u64> = OnceLock::new();

const CACHE_BUDGET: i64 = 512 * 1024 * 1024;

pub fn ExtensionsPath() -> PathBuf {
    dirs::data_dir().unwrap_or_else(|| PathBuf::from(".")).join("zstio-tv").join("extensions")
}

pub fn DatabasePath() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("zstio-tv")
        .join("zstio-tv.db")
}

#[derive(Clone, PartialEq, serde::Serialize)]
pub struct NowPlaying {
    pub Title: String,
    pub Artist: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SetupStage {
    Pairing = 0,
    Token = 1,
    Verifying = 2,
    Ports = 3,
    Password = 4,
    Done = 5,
}

impl SetupStage {
    fn FromValue(Value: u8) -> Self {
        match Value {
            1 => Self::Token,
            2 => Self::Verifying,
            3 => Self::Ports,
            4 => Self::Password,
            5 => Self::Done,
            _ => Self::Pairing,
        }
    }

    pub fn AsText(self) -> &'static str {
        match self {
            Self::Pairing => "pairing",
            Self::Token => "token",
            Self::Verifying => "verifying",
            Self::Ports => "ports",
            Self::Password => "password",
            Self::Done => "done",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CoverLookup {
    Chain,
    Deezer,
}

impl CoverLookup {
    pub fn Parse(Text: &str) -> Option<Self> {
        match Text.trim() {
            "auto" => Some(CoverLookup::Chain),
            "deezer" => Some(CoverLookup::Deezer),
            _ => None,
        }
    }
}

pub struct Showing {
    pub Page: String,
    pub Pages: Vec<String>,
    pub Seconds: u32,
    pub Since: Instant,
}

#[derive(Clone)]
pub struct CoverArt {
    pub Bytes: Vec<u8>,
    pub Mime: String,
    pub Tag: String,
}

impl CoverArt {
    pub fn New(Bytes: Vec<u8>, Mime: String) -> Self {
        use sha2::Digest;
        let Tag = hex::encode(&sha2::Sha256::digest(&Bytes)[..8]);
        CoverArt { Bytes, Mime, Tag }
    }
}

pub struct AppState {
    pub PairingCode: Mutex<String>,
    pub ClientConnected: AtomicBool,
    pub CodeAccepted: AtomicBool,
    pub Db: db::Database,
    pub CloudflaredBin: Mutex<PathBuf>,
    pub SetupShutdown: tokio::sync::Notify,
    pub CurrentTrack: Mutex<Option<NowPlaying>>,
    pub CurrentCover: Mutex<Option<CoverArt>>,
    pub LastTrack: Mutex<Option<NowPlaying>>,
    pub LastfmError: Mutex<Option<String>>,
    pub SerialError: Mutex<Option<String>>,
    pub SerialOpen: AtomicBool,
    pub SerialEpoch: AtomicU64,
    pub BpmCache: Mutex<HashMap<String, Option<f64>>>,
    pub Meter: perf::Shared,
    pub PerfTick: tokio::sync::watch::Sender<u64>,
    pub Frames: Mutex<perf::Frames>,
    pub Windowed: Mutex<Option<(tauri::PhysicalSize<u32>, tauri::PhysicalPosition<i32>)>>,
    pub Sessions: Mutex<HashMap<String, Instant>>,
    pub Handle: OnceLock<tauri::AppHandle>,
    pub SetupPort: AtomicU16,
    pub SetupProgress: AtomicU8,
    pub Voting: voting::Runtime,
    pub Runtime: OnceLock<tokio::runtime::Handle>,
    pub Probe: String,
    pub Traffic: Arc<server::traffic::Totals>,
    pub Showing: Mutex<Option<Showing>>,
    pub Extensions: extensions::Registry,
}

impl AppState {
    pub fn SetTrack(&self, Track: Option<NowPlaying>) {
        *self.CurrentTrack.lock().unwrap() = Track.clone();
        let Some(Playing) = Track else { return };

        let Fresh = self.LastTrack.lock().unwrap().as_ref() != Some(&Playing);
        if Fresh {
            *self.LastTrack.lock().unwrap() = Some(Playing.clone());
            *self.CurrentCover.lock().unwrap() = None;
            self.Db.SaveLastPlayed(&Playing.Title, &Playing.Artist);
        }

        if let Some(Handle) = self.Handle.get() {
            Handle.emit("track-changed", &Playing).ok();
        }
    }

    pub fn SetLastfmError(&self, Message: Option<String>) {
        *self.LastfmError.lock().unwrap() = Message;
    }

    pub fn SetSerialError(&self, Message: Option<String>) {
        *self.SerialError.lock().unwrap() = Message;
    }

    pub fn BumpSerial(&self) {
        self.SerialEpoch.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn SetCover(&self, Decoded: Option<(Vec<u8>, String)>) {
        let Next = Decoded.map(|(Bytes, Mime)| CoverArt::New(Bytes, Mime));
        if let Some(Art) = &Next {
            self.Db.SaveLastCover(&Art.Bytes, &Art.Mime);
        }

        *self.CurrentCover.lock().unwrap() = Next;
        let Url = self.CoverUrl();

        if let Some(Handle) = self.Handle.get() {
            Handle.emit("cover-changed", Url).ok();
        }
    }

    pub fn LookupCover(self: &Arc<Self>, Title: String, Artist: String, Lookup: CoverLookup) {
        let State = Arc::clone(self);
        let Work = async move {
            let Url = match Lookup {
                CoverLookup::Chain => metadata::FindCoverUrl(&State.Db, &Title, &Artist).await,
                CoverLookup::Deezer => metadata::DeezerCoverUrl(&Title, &Artist).await,
            };

            let Some(Url) = Url else {
                log!(Subsystem::Metadata, Level::Info, "no cover found for {} - {}", Artist, Title);
                return;
            };

            let Some(Art) = metadata::Download(&Url).await else {
                log!(Subsystem::Metadata, Level::Warn, "cover download failed: {}", Url);
                return;
            };

            let Still = State.CurrentTrack.lock().unwrap()
                .as_ref()
                .is_some_and(|C| C.Title == Title && C.Artist == Artist);
            let Empty = State.CurrentCover.lock().unwrap().is_none();

            if Still && Empty {
                log!(Subsystem::Metadata, Level::Info, "cover filled for {} - {}", Artist, Title);
                State.SetCover(Some(Art));
            }
        };

        match self.Runtime.get() {
            Some(Rt) => { Rt.spawn(Work); }
            None => log!(Subsystem::Metadata, Level::Warn, "cover lookup skipped, runtime not ready"),
        }
    }

    pub fn NotifyVoting(self: &Arc<Self>) {
        if !self.Voting.ClaimBoardUpdate() {
            return;
        }

        let State = Arc::clone(self);
        let Emit = async move {
            tokio::time::sleep(Duration::from_millis(400)).await;
            State.Voting.ReleaseBoardUpdate();
            if let Some(Handle) = State.Handle.get() {
                Handle.emit("voting-changed", voting::TvBoard(&State.Db)).ok();
            }
        };

        match self.Runtime.get() {
            Some(Rt) => { Rt.spawn(Emit); }
            None => self.Voting.ReleaseBoardUpdate(),
        }
    }

    pub fn NotifySlides(&self) {
        if let Some(Handle) = self.Handle.get() {
            Handle.emit("slides-changed", slides::All(&self.Db)).ok();
        }
    }

    pub fn NotifyScreen(&self) {
        if let Some(Handle) = self.Handle.get() {
            Handle.emit("screen-changed", self.ScreenConfig()).ok();
        }
    }

    pub fn ScreenConfig(&self) -> serde_json::Value {
        serde_json::json!({
            "Weather": {
                "Enabled": self.Db.GetWeatherEnabled(),
                "Latitude": self.Db.GetWeatherLatitude(),
                "Longitude": self.Db.GetWeatherLongitude(),
            },
            "Lessons": {
                "Duration": self.Db.GetLessonDuration(),
            },
            "ScrobbleInLessons": self.Db.IsScrobbleInLessons(),
            "Notice": self.Db.GetScreenNotice(),
            "Pages": self.Db.GetScreenPages()
        })
    }

    pub fn CoverUrl(&self) -> Option<String> {
        self.CurrentCover
            .lock()
            .unwrap()
            .as_ref()
            .map(|C| format!("http://127.0.0.1:9999/api/playback/cover?v={}", C.Tag))
    }

    pub fn CoverPath(&self) -> Option<String> {
        self.CurrentCover
            .lock()
            .unwrap()
            .as_ref()
            .map(|C| format!("/api/playback/cover?v={}", C.Tag))
    }

    pub fn Whoami(&self, Panel: bool) -> String {
        format!("{}:{}", if Panel { "panel" } else { "vote" }, self.Probe)
    }

    pub fn CreateSession(&self) -> String {
        use rand::Rng;

        let mut Rng = rand::thread_rng();
        let Bytes: Vec<u8> = (0..32).map(|_| Rng.gen()).collect();
        let Token = hex::encode(Bytes);

        let mut Held = self.Sessions.lock().unwrap();
        Held.retain(|_, Expiry| *Expiry > Instant::now());
        Held.insert(Token.clone(), Instant::now() + SESSION_TTL);

        Token
    }

    pub fn ValidSession(&self, Token: &str) -> bool {
        if Token.is_empty() {
            return false;
        }

        let mut Held = self.Sessions.lock().unwrap();
        Held.retain(|_, Expiry| *Expiry > Instant::now());
        Held.contains_key(Token)
    }

    pub fn DropSessions(&self) {
        self.Sessions.lock().unwrap().clear();
    }

    pub fn KeepOnlySession(&self, Token: &str) {
        self.Sessions.lock().unwrap().retain(|Held, _| Held == Token);
    }

    pub fn SetSetupStage(&self, Stage: SetupStage) {
        self.SetupProgress.store(Stage as u8, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn GetSetupStage(&self) -> SetupStage {
        SetupStage::FromValue(self.SetupProgress.load(std::sync::atomic::Ordering::Relaxed))
    }
}

#[derive(serde::Serialize)]
struct NetworkInterface {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Address")]
    address: String,
}

#[tauri::command]
fn get_network_interfaces() -> Vec<NetworkInterface> {
    let Interfaces = match if_addrs::get_if_addrs() {
        Ok(List) => List,
        Err(_) => return Vec::new(),
    };

    Interfaces
        .into_iter()
        .filter(|Iface| !Iface.is_loopback() && Iface.addr.ip().is_ipv4())
        .map(|Iface| NetworkInterface {
            name: Iface.name.clone(),
            address: Iface.addr.ip().to_string(),
        })
        .collect()
}

#[tauri::command]
async fn check_reachability(Address: String) -> bool {
    let ParsedAddr: SocketAddr = match Address.parse() {
        Ok(Addr) => Addr,
        Err(_) => return false,
    };

    match tokio::time::timeout(
        Duration::from_secs(2),
        tokio::net::TcpStream::connect(ParsedAddr),
    )
    .await
    {
        Ok(Ok(_)) => true,
        _ => false,
    }
}

#[tauri::command]
fn get_pairing_code(State: tauri::State<'_, Arc<AppState>>) -> String {
    State.PairingCode.lock().unwrap().clone()
}

#[tauri::command]
fn is_client_connected(State: tauri::State<'_, Arc<AppState>>) -> bool {
    State.ClientConnected.load(std::sync::atomic::Ordering::Relaxed)
}

#[tauri::command]
fn is_code_accepted(State: tauri::State<'_, Arc<AppState>>) -> bool {
    State.CodeAccepted.load(std::sync::atomic::Ordering::Relaxed)
}

#[tauri::command]
fn reset_client_connection(State: tauri::State<'_, Arc<AppState>>) {
    State.ClientConnected.store(false, std::sync::atomic::Ordering::Relaxed);
    log!(Subsystem::Core, Level::Info, "client connection reset, awaiting rescan");
}

#[tauri::command]
fn is_setup_complete(State: tauri::State<'_, Arc<AppState>>) -> bool {
    State.Db.IsSetupComplete()
}

#[tauri::command]
fn get_setup_stage(State: tauri::State<'_, Arc<AppState>>) -> String {
    State.GetSetupStage().AsText().to_string()
}

#[tauri::command]
fn get_current_track(State: tauri::State<'_, Arc<AppState>>) -> Option<NowPlaying> {
    State.LastTrack.lock().unwrap().clone()
}

#[tauri::command]
async fn get_track_bpm(State: tauri::State<'_, Arc<AppState>>) -> Result<Option<f64>, ()> {
    let Track = match State.LastTrack.lock().unwrap().clone() {
        Some(Playing) => Playing,
        None => return Ok(None),
    };

    let Key = bpm::CacheKey(&Track.Artist, &Track.Title);

    if let Some(Known) = State.BpmCache.lock().unwrap().get(&Key) {
        return Ok(*Known);
    }

    let Found = bpm::Lookup(&Track.Artist, &Track.Title).await;
    State.BpmCache.lock().unwrap().insert(Key, Found);

    Ok(Found)
}

#[tauri::command]
fn get_lesson_times(State: tauri::State<'_, Arc<AppState>>) -> serde_json::Value {
    lessons::AsJson(State.Db.GetLessonDuration())
}

#[tauri::command]
fn get_setup_port(State: tauri::State<'_, Arc<AppState>>) -> u16 {
    State.SetupPort.load(std::sync::atomic::Ordering::Relaxed)
}

async fn _WaitForPort(Port: u16) {
    let Addr = format!("127.0.0.1:{}", Port);
    for _ in 0..50 {
        if tokio::net::TcpStream::connect(&Addr).await.is_ok() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    log!(Subsystem::Server, Level::Warn, "port {} not ready after 5s", Port);
}

#[tauri::command]
fn get_voting_board(State: tauri::State<'_, Arc<AppState>>) -> serde_json::Value {
    voting::TvBoard(&State.Db)
}

#[tauri::command]
fn report_page(State: tauri::State<'_, Arc<AppState>>, page: String, pages: Vec<String>, seconds: u32, elapsed: u64) {
    let Valid = State.Db.ScreenPageIds();
    let Known = |Id: &str| Valid.iter().any(|(Known, _)| Known == Id);
    if !Known(&page) || !pages.iter().all(|P| Known(P)) || !db::PAGE_SECONDS.contains(&(seconds.min(u16::MAX as u32) as u16)) {
        return;
    }
    let Since = Instant::now().checked_sub(Duration::from_millis(elapsed.min(seconds as u64 * 1000))).unwrap_or_else(Instant::now);
    *State.Showing.lock().unwrap() = Some(Showing { Page: page, Pages: pages, Seconds: seconds, Since });
}

#[tauri::command]
fn get_slides(State: tauri::State<'_, Arc<AppState>>) -> serde_json::Value {
    slides::All(&State.Db)
}

#[tauri::command]
fn get_news(State: tauri::State<'_, Arc<AppState>>) -> serde_json::Value {
    news::Current(&State.Db)
}

#[tauri::command]
fn report_frame_stats(State: tauri::State<'_, Arc<AppState>>, fps: f64, low: f64) {
    perf::Report(&State, fps, low);
}

#[tauri::command]
fn get_scrobble_api_key(State: tauri::State<'_, Arc<AppState>>) -> String {
    State.Db.GetApiKey()
}

#[tauri::command]
fn regenerate_scrobble_api_key(State: tauri::State<'_, Arc<AppState>>) -> String {
    State.Db.RegenerateApiKey()
}

#[tauri::command]
fn get_scrobble_method(State: tauri::State<'_, Arc<AppState>>) -> String {
    State.Db.GetScrobbleMethod()
}

#[tauri::command]
fn set_scrobble_method(State: tauri::State<'_, Arc<AppState>>, Method: String) {
    State.Db.SetScrobbleMethod(&Method);
}

#[tauri::command]
fn get_lastfm_api_key(State: tauri::State<'_, Arc<AppState>>) -> String {
    State.Db.GetLastfmApiKey()
}

#[tauri::command]
fn set_lastfm_api_key(State: tauri::State<'_, Arc<AppState>>, Key: String) {
    State.Db.SetLastfmApiKey(&Key);
}

#[tauri::command]
fn get_lastfm_username(State: tauri::State<'_, Arc<AppState>>) -> String {
    State.Db.GetLastfmUsername()
}

#[tauri::command]
fn set_lastfm_username(State: tauri::State<'_, Arc<AppState>>, Username: String) {
    State.Db.SetLastfmUsername(&Username);
}

#[tauri::command]
async fn download_cloudflared(State: tauri::State<'_, Arc<AppState>>) -> Result<bool, String> {
    let Path = cloudflared_dl::EnsureBinary().await?;
    *State.CloudflaredBin.lock().unwrap() = Path;
    Ok(true)
}

#[tauri::command]
fn get_download_progress() -> u8 {
    cloudflared_dl::GetProgress()
}

fn _RevealWindow(Handle: &tauri::AppHandle, Window: &tauri::WebviewWindow) {
    #[cfg(target_os = "macos")]
    Handle.show().ok();
    #[cfg(not(target_os = "macos"))]
    let _ = Handle;

    if Window.is_minimized().unwrap_or(false) {
        Window.unminimize().ok();
    }

    Window.show().ok();
    Window.set_focus().ok();
}

#[tauri::command]
fn show_window(Handle: tauri::AppHandle, Window: tauri::WebviewWindow) {
    _RevealWindow(&Handle, &Window);
}

fn _Shutdown() -> ! {
    cloudflared::_StopDaemon();
    std::process::exit(0);
}

#[tauri::command]
fn toggle_fullscreen(Window: tauri::Window, State: tauri::State<Arc<AppState>>) -> bool {
    let Entering = !Window.is_fullscreen().unwrap_or(false);

    if Entering {
        if let (Ok(Size), Ok(Position)) = (Window.inner_size(), Window.outer_position()) {
            *State.Windowed.lock().unwrap() = Some((Size, Position));
        }
        Window.set_fullscreen(true).ok();
    } else {
        Window.set_fullscreen(false).ok();

        if let Some((Size, Position)) = State.Windowed.lock().unwrap().take() {
            Window.set_size(Size).ok();
            Window.set_position(Position).ok();
        }
    }

    Entering
}

#[tauri::command]
fn get_cover_url(State: tauri::State<Arc<AppState>>) -> Option<String> {
    State.CoverUrl()
}

#[tauri::command]
fn get_screen_config(State: tauri::State<Arc<AppState>>) -> serde_json::Value {
    State.ScreenConfig()
}

#[tauri::command]
fn is_fullscreen(Window: tauri::Window) -> bool {
    Window.is_fullscreen().unwrap_or(false)
}

#[tauri::command]
fn quit_app() {
    log!(Subsystem::Core, Level::Info, "quit requested from the interface");
    _Shutdown();
}

#[cfg(unix)]
extern "C" fn _OnTerminationSignal(_Signal: libc::c_int) {
    cloudflared::_StopDaemon();
    unsafe { libc::_exit(0) };
}

fn _InstallSignalHandlers() {
    #[cfg(unix)]
    unsafe {
        libc::signal(libc::SIGINT, _OnTerminationSignal as *const () as libc::sighandler_t);
        libc::signal(libc::SIGTERM, _OnTerminationSignal as *const () as libc::sighandler_t);
        libc::signal(libc::SIGHUP, _OnTerminationSignal as *const () as libc::sighandler_t);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use Subsystem::Core;
    use Level::Info;

    STARTED_AT.get_or_init(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|D| D.as_secs())
            .unwrap_or(0)
    });

    let DbPath = DatabasePath();

    log::_InstallPanicHook();
    _InstallSignalHandlers();

    let Db = db::Database::Open(&DbPath);
    let SetupDone = Db.IsSetupComplete();
    log!(Core, Info, "setup complete: {}", SetupDone);

    let Code = code::_GenerateCode();
    log!(Core, Info, "pairing code: {}", Code);

    let (LastTrack, LastCover) = match Db.LastPlayed() {
        Some((Title, Artist, Cover)) => (
            Some(NowPlaying { Title, Artist }),
            Cover.map(|(Bytes, Mime)| CoverArt::New(Bytes, Mime)),
        ),
        None => (None, None),
    };

    let State = Arc::new(AppState {
        PairingCode: Mutex::new(Code),
        ClientConnected: AtomicBool::new(false),
        CodeAccepted: AtomicBool::new(false),
        Db,
        CloudflaredBin: Mutex::new(cloudflared_dl::BinPath()),
        SetupShutdown: tokio::sync::Notify::new(),
        CurrentTrack: Mutex::new(None),
        CurrentCover: Mutex::new(LastCover),
        LastTrack: Mutex::new(LastTrack),
        LastfmError: Mutex::new(None),
        SerialError: Mutex::new(None),
        SerialOpen: AtomicBool::new(false),
        SerialEpoch: AtomicU64::new(0),
        BpmCache: Mutex::new(HashMap::new()),
        Meter: Mutex::new(Default::default()),
        PerfTick: tokio::sync::watch::channel(0).0,
        Frames: Mutex::new(Default::default()),
        Windowed: Mutex::new(None),
        Sessions: Mutex::new(HashMap::new()),
        Handle: OnceLock::new(),
        SetupPort: AtomicU16::new(0),
        SetupProgress: AtomicU8::new(SetupStage::Pairing as u8),
        Voting: voting::Runtime::New(),
        Runtime: OnceLock::new(),
        Probe: {
            use rand::Rng;
            hex::encode(rand::thread_rng().gen::<[u8; 16]>())
        },
        Traffic: Arc::new(server::traffic::Totals::default()),
        Showing: Mutex::new(None),
        Extensions: extensions::Registry::Open(ExtensionsPath()),
    });
    State.Extensions.Load(&State.Db);

    perf::Spawn(Arc::clone(&State));

    let ServerState = Arc::clone(&State);
    std::thread::spawn(move || {
        let Rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("failed to create tokio runtime for axum");

        ServerState.Runtime.set(Rt.handle().clone()).ok();

        Rt.block_on(async {
            if !SetupDone {
                let Shutdown = Arc::clone(&ServerState);
                let Router = server::_CreateRouter(Arc::clone(&ServerState));
                let Listener = match tokio::net::TcpListener::bind("0.0.0.0:8080").await {
                    Ok(L) => L,
                    Err(_) => tokio::net::TcpListener::bind("0.0.0.0:0")
                        .await
                        .expect("failed to bind setup server"),
                };
                let Port = Listener.local_addr().unwrap().port();
                ServerState.SetupPort.store(Port, std::sync::atomic::Ordering::Relaxed);
                log!(Subsystem::Server, Level::Info, "setup listening on :{}", Port);
                tokio::select! {
                    _ = axum::serve(Listener, Router) => {}
                    _ = Shutdown.SetupShutdown.notified() => {
                        log!(Subsystem::Server, Level::Info, "setup server stopped forcefully");
                    }
                }
            }

            let PanelState = Arc::clone(&ServerState);
            tokio::spawn(async move {
                let Router = server::_CreatePanelRouter(Arc::clone(&PanelState));
                let Listener = match tokio::net::TcpListener::bind("0.0.0.0:9999").await {
                    Ok(L) => L,
                    Err(E) => { fail!(Subsystem::Server, E, "panel server could not bind 0.0.0.0:9999"); return; }
                };
                log!(Subsystem::Server, Level::Info, "panel listening on :9999");
                let Counted = server::traffic::Listener::New(Listener, Arc::clone(&PanelState.Traffic));
                axum::serve(Counted, Router.into_make_service_with_connect_info::<server::traffic::Peer>()).await.ok();
            });

            let VoteState = Arc::clone(&ServerState);
            tokio::spawn(async move {
                let Router = server::_CreateVoteRouter(Arc::clone(&VoteState));
                let Listener = match tokio::net::TcpListener::bind("127.0.0.1:10000").await {
                    Ok(L) => L,
                    Err(E) => { fail!(Subsystem::Server, E, "vote server could not bind 127.0.0.1:10000"); return; }
                };
                log!(Subsystem::Server, Level::Info, "vote listening on :10000");
                let Counted = server::traffic::Listener::New(Listener, Arc::clone(&VoteState.Traffic));
                axum::serve(Counted, Router).await.ok();
            });

            let PurgeState = Arc::clone(&ServerState);
            tokio::spawn(async move {
                loop {
                    voting::Purge(&PurgeState.Db);
                    slides::Sweep(&PurgeState.Db);
                    let Removed = PurgeState.Db.CachePrune(CACHE_BUDGET);
                    if Removed > 0 {
                        log!(Subsystem::Core, Level::Info, "cache pruned: {} entries", Removed);
                    }
                    tokio::time::sleep(Duration::from_secs(12 * 60 * 60)).await;
                }
            });

            news::Spawn(Arc::clone(&ServerState));

            serial::StartListener(Arc::clone(&ServerState));

            let LastfmState = Arc::clone(&ServerState);
            tokio::spawn(async move {
                lastfm::StartPoller(LastfmState).await;
            });

            let SpotifyState = Arc::clone(&ServerState);
            tokio::spawn(async move {
                spotify::StartPoller(SpotifyState).await;
            });

            _WaitForPort(9999).await;
            _WaitForPort(10000).await;
            let mut Backoff = Duration::from_secs(5);
            loop {
                let Token = ServerState.Db.GetTunnelToken();
                if Token.is_empty() {
                    log!(Subsystem::Cloudflared, Level::Warn, "no tunnel token, servers stay local only");
                    std::future::pending::<()>().await;
                }

                let Bin = ServerState.CloudflaredBin.lock().unwrap().clone();
                let Started = Instant::now();
                cloudflared::_RunDaemon(&Bin, &Token).await;

                if Started.elapsed() > Duration::from_secs(120) {
                    Backoff = Duration::from_secs(5);
                }
                log!(Subsystem::Cloudflared, Level::Warn, "tunnel daemon stopped, restarting in {}s", Backoff.as_secs());
                tokio::time::sleep(Backoff).await;
                Backoff = (Backoff * 2).min(Duration::from_secs(120));
            }
        });
    });

    let SetupState = Arc::clone(&State);
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .manage(State)
        .on_window_event(|_Window, Event| {
            if matches!(Event, tauri::WindowEvent::CloseRequested { .. } | tauri::WindowEvent::Destroyed) {
                log!(Core, Info, "window closed, shutting down");
                _Shutdown();
            }
        })
        .setup(move |App| {
            SetupState.Handle.set(App.handle().clone()).ok();

            if let Some(Window) = App.get_webview_window("main") {
                let Handle = App.handle().clone();
                std::thread::spawn(move || {
                    std::thread::sleep(_WINDOW_REVEAL_FALLBACK);
                    if !Window.is_visible().unwrap_or(true) {
                        _RevealWindow(&Handle, &Window);
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_network_interfaces,
            check_reachability,
            get_pairing_code,
            is_client_connected,
            is_code_accepted,
            reset_client_connection,
            is_setup_complete,
            get_setup_stage,
            get_current_track,
            get_lesson_times,
            get_track_bpm,
            get_voting_board,
            report_frame_stats,
            get_news,
            get_slides,
            report_page,
            get_setup_port,
            download_cloudflared,
            get_download_progress,
            show_window,
            quit_app,
            toggle_fullscreen,
            is_fullscreen,
            get_cover_url,
            get_screen_config,
            get_scrobble_api_key,
            regenerate_scrobble_api_key,
            get_scrobble_method,
            set_scrobble_method,
            get_lastfm_api_key,
            set_lastfm_api_key,
            get_lastfm_username,
            set_lastfm_username
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_Handle, Event| {
            if let tauri::RunEvent::ExitRequested { .. } = Event {
                log!(Core, Info, "exit requested, shutting down");
                _Shutdown();
            }
        });
}
