// a bad joke

#![allow(non_snake_case)]

use std::io::{ErrorKind, Read};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use base64::Engine;
use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use sha2::{Digest, Sha256};
use x25519_dalek::{PublicKey, StaticSecret};

use crate::AppState;
use crate::NowPlaying;
use crate::CoverLookup;
use crate::cover;
use crate::db::Database;
use crate::log::{Subsystem::Serial, Level::*};

pub const DEFAULT_BAUD: u32 = 115200;

const MAGIC: &[u8] = b"ZFM1:";
const CONTEXT: &[u8] = b"zstio-fm serial v1";
const NONCE_BYTES: usize = 24;
const KEY_BYTES: usize = 32;
const MAX_LINE: usize = 512 * 1024;
const MAX_SKEW_SECS: u64 = 120;
const READ_TIMEOUT: Duration = Duration::from_millis(250);
const REOPEN_DELAY: Duration = Duration::from_secs(3);
const IDLE_DELAY: Duration = Duration::from_millis(750);

pub struct Failure {
    pub Code: String,
    pub Detail: String,
}

impl Failure {
    fn New(Code: &str, Detail: impl Into<String>) -> Self {
        Self { Code: Code.to_string(), Detail: Detail.into() }
    }
}

#[derive(serde::Deserialize)]
struct Envelope {
    #[serde(rename = "Seq")]
    seq: u64,
    #[serde(rename = "At")]
    at: u64,
    #[serde(rename = "Title", default)]
    title: Option<String>,
    #[serde(rename = "Artist", default)]
    artist: String,
    #[serde(rename = "Cover", default)]
    cover: Option<String>,
    #[serde(rename = "Lesson", default)]
    lesson: Option<u8>,
    #[serde(rename = "CoverLookup", default)]
    cover_lookup: Option<String>,
}

fn _Encode(Bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(Bytes)
}

fn _Decode(Text: &str) -> Option<Vec<u8>> {
    let Trimmed = Text.trim();

    base64::engine::general_purpose::STANDARD
        .decode(Trimmed)
        .or_else(|_| base64::engine::general_purpose::STANDARD_NO_PAD.decode(Trimmed))
        .ok()
}

fn _Array(Bytes: &[u8]) -> Option<[u8; KEY_BYTES]> {
    if Bytes.len() != KEY_BYTES {
        return None;
    }

    let mut Out = [0u8; KEY_BYTES];
    Out.copy_from_slice(Bytes);
    Some(Out)
}

pub fn Fingerprint(Key: &[u8; KEY_BYTES]) -> String {
    let Digest = Sha256::digest(Key);
    Digest[..8]
        .iter()
        .map(|B| format!("{:02x}", B))
        .collect::<Vec<_>>()
        .join(":")
}

pub fn ParsePeerKey(Text: &str) -> Option<[u8; KEY_BYTES]> {
    _Decode(Text).as_deref().and_then(_Array)
}

fn _Secret(Db: &Database) -> StaticSecret {
    if let Some(Bytes) = _Decode(&Db.GetSerialSecretKey()).as_deref().and_then(_Array) {
        return StaticSecret::from(Bytes);
    }

    let Fresh = StaticSecret::random_from_rng(rand::rngs::OsRng);
    Db.SetSerialSecretKey(&_Encode(Fresh.as_bytes()));
    log!(Serial, Info, "generated a new device keypair");
    Fresh
}

pub fn EnsureKeypair(Db: &Database) -> ([u8; KEY_BYTES], String) {
    let Secret = _Secret(Db);
    let Public = PublicKey::from(&Secret);
    (*Public.as_bytes(), _Encode(Public.as_bytes()))
}

pub fn Regenerate(Db: &Database) -> ([u8; KEY_BYTES], String) {
    let Fresh = StaticSecret::random_from_rng(rand::rngs::OsRng);
    Db.SetSerialSecretKey(&_Encode(Fresh.as_bytes()));
    Db.SetSerialLastSeq(0);

    let Public = PublicKey::from(&Fresh);
    log!(Serial, Warn, "device keypair regenerated, peers must be updated");
    (*Public.as_bytes(), _Encode(Public.as_bytes()))
}

pub fn ListPorts() -> Vec<String> {
    let Found = match serialport::available_ports() {
        Ok(List) => List,
        Err(E) => {
            log!(Serial, Warn, "port enumeration failed: {}", E);
            return Vec::new();
        }
    };

    let mut Names: Vec<String> = Found.into_iter().map(|P| P.port_name).collect();
    Names.sort();
    Names.dedup();
    Names
}

fn _SessionKey(Secret: &StaticSecret, Peer: &PublicKey) -> [u8; KEY_BYTES] {
    let Ours = PublicKey::from(Secret);
    let Shared = Secret.diffie_hellman(Peer);

    let mut Hash = Sha256::new();
    Hash.update(CONTEXT);
    Hash.update(Shared.as_bytes());
    Hash.update(Peer.as_bytes());
    Hash.update(Ours.as_bytes());
    Hash.finalize().into()
}

fn _Now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|D| D.as_secs())
        .unwrap_or(0)
}

fn _Open(State: &Arc<AppState>, Line: &[u8]) -> Result<Envelope, Failure> {
    if !Line.starts_with(MAGIC) {
        return Err(Failure::New("magic", "line has no ZFM1 prefix"));
    }

    let Body = std::str::from_utf8(&Line[MAGIC.len()..])
        .map_err(|_| Failure::New("base64", "frame is not utf8"))?;

    let Raw = _Decode(Body).ok_or_else(|| Failure::New("base64", "frame is not valid base64"))?;

    if Raw.len() <= NONCE_BYTES {
        return Err(Failure::New("short", format!("frame is {} bytes", Raw.len())));
    }

    let PeerKey = ParsePeerKey(&State.Db.GetSerialPeerKey())
        .ok_or_else(|| Failure::New("no_peer", "no sender key configured"))?;

    let Secret = _Secret(&State.Db);
    let Key = _SessionKey(&Secret, &PublicKey::from(PeerKey));

    let Cipher = XChaCha20Poly1305::new_from_slice(&Key)
        .map_err(|_| Failure::New("auth", "key setup failed"))?;

    let Nonce = XNonce::from_slice(&Raw[..NONCE_BYTES]);

    let Plain = Cipher
        .decrypt(Nonce, Payload { msg: &Raw[NONCE_BYTES..], aad: MAGIC })
        .map_err(|_| Failure::New("auth", "authentication failed"))?;

    serde_json::from_slice::<Envelope>(&Plain)
        .map_err(|E| Failure::New("payload", format!("bad payload: {}", E)))
}

fn _Accept(State: &Arc<AppState>, Msg: Envelope) -> Result<(), Failure> {
    let Now = _Now();
    let Skew = if Msg.at > Now { Msg.at - Now } else { Now - Msg.at };

    if Skew > MAX_SKEW_SECS {
        return Err(Failure::New("skew", format!("timestamp off by {}s", Skew)));
    }

    if Msg.seq <= State.Db.GetSerialLastSeq() {
        return Err(Failure::New("replay", format!("replayed sequence {}", Msg.seq)));
    }

    let Title = Msg
        .title
        .as_deref()
        .map(|T| T.trim().to_string())
        .filter(|T| !T.is_empty());

    if let Some(Minutes) = Msg.lesson {
        if Minutes != 30 && Minutes != 45 {
            return Err(Failure::New("bad_lesson", format!("lesson duration {}", Minutes)));
        }
    }

    let Lookup = match Msg.cover_lookup.as_deref().map(str::trim).filter(|L| !L.is_empty()) {
        None => None,
        Some(Raw) => Some(
            CoverLookup::Parse(Raw)
                .ok_or_else(|| Failure::New("payload", format!("unknown CoverLookup {:?}", Raw)))?,
        ),
    };

    if Title.is_none() && Msg.lesson.is_none() {
        return Err(Failure::New("empty", "frame carries neither title nor lesson"));
    }

    State.Db.SetSerialLastSeq(Msg.seq);

    if let Some(Minutes) = Msg.lesson {
        State.Db.SetLessonDuration(Minutes);
        State.NotifyScreen();
        log!(Serial, Info, "lesson duration set to {}", Minutes);
    }

    if let Some(Name) = Title {
        let Art = Msg
            .cover
            .as_deref()
            .filter(|C| !C.trim().is_empty())
            .and_then(cover::DecodeBase64)
            .and_then(cover::Accept);

        let Artist = Msg.artist.trim().to_string();
        log!(Serial, Info, "now playing: {} - {}", Artist, Name);
        State.SetTrack(Some(NowPlaying { Title: Name.clone(), Artist: Artist.clone() }));

        if Art.is_some() {
            State.SetCover(Art);
        } else if let Some(Mode) = Lookup {
            State.LookupCover(Name, Artist, Mode);
        }
    }

    Ok(())
}

fn _Feed(State: &Arc<AppState>, Buffer: &mut Vec<u8>) {
    loop {
        let Break = match Buffer.iter().position(|B| *B == b'\n') {
            Some(At) => At,
            None => break,
        };

        let Line: Vec<u8> = Buffer.drain(..=Break).take(Break).collect();
        let Trimmed: &[u8] = match Line.strip_suffix(b"\r") {
            Some(Cut) => Cut,
            None => &Line,
        };

        if Trimmed.is_empty() {
            continue;
        }

        match _Open(State, Trimmed).and_then(|Msg| _Accept(State, Msg)) {
            Ok(()) => State.SetSerialError(None),
            Err(F) => {
                log!(Serial, Warn, "frame rejected: {}", F.Detail);
                State.SetSerialError(Some(F.Code));
            }
        }
    }

    if Buffer.len() > MAX_LINE {
        log!(Serial, Warn, "line limit exceeded, dropping {} buffered bytes", Buffer.len());
        Buffer.clear();
    }
}

fn _Pump(State: &Arc<AppState>, Port: &mut Box<dyn serialport::SerialPort>, Epoch: u64) {
    let mut Buffer: Vec<u8> = Vec::new();
    let mut Chunk = [0u8; 4096];

    loop {
        if State.SerialEpoch.load(Ordering::Relaxed) != Epoch {
            return;
        }

        match Port.read(&mut Chunk) {
            Ok(0) => return,
            Ok(Count) => {
                Buffer.extend_from_slice(&Chunk[..Count]);
                _Feed(State, &mut Buffer);
            }
            Err(E) if E.kind() == ErrorKind::TimedOut => continue,
            Err(E) if E.kind() == ErrorKind::Interrupted => continue,
            Err(E) => {
                log!(Serial, Warn, "read failed: {}", E);
                State.SetSerialError(Some("read".into()));
                return;
            }
        }
    }
}

fn _OpenCode(E: &serialport::Error) -> String {
    match E.kind() {
        serialport::ErrorKind::NoDevice => "no_device",
        serialport::ErrorKind::InvalidInput => "bad_port",
        serialport::ErrorKind::Io(ErrorKind::PermissionDenied) => "denied",
        serialport::ErrorKind::Io(ErrorKind::NotFound) => "no_device",
        _ => "io",
    }
    .to_string()
}

pub fn StartListener(State: Arc<AppState>) {
    std::thread::spawn(move || {
        loop {
            let Epoch = State.SerialEpoch.load(Ordering::Relaxed);
            let Name = State.Db.GetSerialPort();

            if State.Db.GetScrobbleMethod() != "serial" || Name.is_empty() {
                State.SerialOpen.store(false, Ordering::Relaxed);
                std::thread::sleep(IDLE_DELAY);
                continue;
            }

            let Baud = State.Db.GetSerialBaud();

            match serialport::new(&Name, Baud).timeout(READ_TIMEOUT).open() {
                Ok(mut Port) => {
                    log!(Serial, Info, "listening on {} at {} baud", Name, Baud);
                    State.SerialOpen.store(true, Ordering::Relaxed);
                    State.SetSerialError(None);

                    _Pump(&State, &mut Port, Epoch);

                    State.SerialOpen.store(false, Ordering::Relaxed);
                    log!(Serial, Info, "closed {}", Name);
                }
                Err(E) => {
                    State.SerialOpen.store(false, Ordering::Relaxed);
                    State.SetSerialError(Some(_OpenCode(&E)));
                    log!(Serial, Warn, "cannot open {}: {}", Name, E);
                    std::thread::sleep(REOPEN_DELAY);
                }
            }
        }
    });
}
