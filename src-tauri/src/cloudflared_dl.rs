#![allow(non_snake_case)]

use std::path::PathBuf;
use std::sync::atomic::{AtomicU8, Ordering};
use crate::log::{Subsystem::Cloudflared, Level::*};

static _PROGRESS: AtomicU8 = AtomicU8::new(0);

pub fn GetProgress() -> u8 {
    _PROGRESS.load(Ordering::Relaxed)
}

fn _BinDir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("zstio-tv")
        .join("bin")
}

fn _BinName() -> &'static str {
    if cfg!(target_os = "windows") { "cloudflared.exe" } else { "cloudflared" }
}

pub fn BinPath() -> PathBuf {
    _BinDir().join(_BinName())
}

fn _DownloadUrl() -> String {
    let Base = "https://github.com/cloudflare/cloudflared/releases/latest/download";

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    let Asset = "cloudflared-darwin-arm64.tgz";
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    let Asset = "cloudflared-darwin-amd64.tgz";
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    let Asset = "cloudflared-linux-amd64";
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    let Asset = "cloudflared-linux-arm64";
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    let Asset = "cloudflared-windows-amd64.exe";
    #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
    let Asset = "cloudflared-windows-arm64.exe";

    format!("{}/{}", Base, Asset)
}

fn _IsTarball() -> bool {
    cfg!(target_os = "macos")
}

pub async fn EnsureBinary() -> Result<PathBuf, String> {
    let Path = BinPath();

    if Path.exists() {
        log!(Cloudflared, Info, "binary exists: {:?}", Path);
        _PROGRESS.store(100, Ordering::Relaxed);
        return Ok(Path);
    }

    let Dir = _BinDir();
    std::fs::create_dir_all(&Dir).map_err(|E| format!("mkdir failed: {}", E))?;

    let Url = _DownloadUrl();
    log!(Cloudflared, Info, "downloading {}", Url);
    _PROGRESS.store(5, Ordering::Relaxed);

    let Client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|E| E.to_string())?;

    let Res = Client.get(&Url)
        .send()
        .await
        .map_err(|E| format!("download failed: {}", E))?;

    if !Res.status().is_success() {
        return Err(format!("HTTP {}", Res.status()));
    }

    let TotalSize = Res.content_length().unwrap_or(0);
    let mut Downloaded: u64 = 0;

    use futures_util::StreamExt;
    let mut Stream = Res.bytes_stream();
    let mut Bytes: Vec<u8> = Vec::with_capacity(TotalSize as usize);

    while let Some(Chunk) = Stream.next().await {
        let Chunk = Chunk.map_err(|E| format!("stream error: {}", E))?;
        Downloaded += Chunk.len() as u64;
        Bytes.extend_from_slice(&Chunk);

        if TotalSize > 0 {
            let Pct = ((Downloaded as f64 / TotalSize as f64) * 90.0) as u8 + 5;
            _PROGRESS.store(Pct.min(95), Ordering::Relaxed);
        }
    }

    log!(Cloudflared, Info, "downloaded {} bytes", Bytes.len());
    _PROGRESS.store(95, Ordering::Relaxed);

    if _IsTarball() {
        _ExtractTarGz(&Bytes, &Path)?;
    } else {
        std::fs::write(&Path, &Bytes).map_err(|E| format!("write failed: {}", E))?;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&Path, std::fs::Permissions::from_mode(0o755))
            .map_err(|E| format!("chmod failed: {}", E))?;
    }

    log!(Cloudflared, Info, "installed to {:?}", Path);
    _PROGRESS.store(100, Ordering::Relaxed);
    Ok(Path)
}

fn _ExtractTarGz(Data: &[u8], OutPath: &PathBuf) -> Result<(), String> {
    use std::io::Read;
    let Decoder = flate2::read::GzDecoder::new(Data);
    let mut Archive = tar::Archive::new(Decoder);

    for Entry in Archive.entries().map_err(|E| format!("tar error: {}", E))? {
        let mut Entry = Entry.map_err(|E| format!("tar entry: {}", E))?;
        let Name = Entry.path().map_err(|E| format!("tar path: {}", E))?;

        if Name.file_name().map_or(false, |N| N == "cloudflared") {
            let mut Buf: Vec<u8> = Vec::new();
            Entry.read_to_end(&mut Buf).map_err(|E| format!("tar read: {}", E))?;
            std::fs::write(OutPath, &Buf).map_err(|E| format!("write: {}", E))?;
            return Ok(());
        }
    }

    Err("cloudflared not found in archive".to_string())
}
