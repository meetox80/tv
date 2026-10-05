#![allow(non_snake_case)]

use std::time::Duration;
use base64::Engine;

pub const MAX_BYTES: usize = 2 * 1024 * 1024;

pub fn SniffMime(Bytes: &[u8]) -> Option<&'static str> {
    if Bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Some("image/jpeg");
    }
    if Bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Some("image/png");
    }
    if Bytes.len() > 12 && &Bytes[0..4] == b"RIFF" && &Bytes[8..12] == b"WEBP" {
        return Some("image/webp");
    }
    if Bytes.starts_with(b"GIF87a") || Bytes.starts_with(b"GIF89a") {
        return Some("image/gif");
    }
    None
}

pub fn Accept(Bytes: Vec<u8>) -> Option<(Vec<u8>, String)> {
    if Bytes.is_empty() || Bytes.len() > MAX_BYTES {
        return None;
    }

    SniffMime(&Bytes).map(|Mime| (Bytes, Mime.to_string()))
}

pub fn DecodeBase64(Raw: &str) -> Option<Vec<u8>> {
    let Trimmed = Raw.trim();
    let Payload = match Trimmed.find(";base64,") {
        Some(At) if Trimmed.starts_with("data:") => &Trimmed[At + 8..],
        _ => Trimmed,
    };

    if Payload.is_empty() {
        return None;
    }

    base64::engine::general_purpose::STANDARD
        .decode(Payload)
        .or_else(|_| base64::engine::general_purpose::STANDARD_NO_PAD.decode(Payload))
        .ok()
}

pub async fn Fetch(Client: &reqwest::Client, Url: &str) -> Option<Vec<u8>> {
    if !Url.starts_with("http://") && !Url.starts_with("https://") {
        return None;
    }

    let Resp = Client.get(Url).timeout(Duration::from_secs(8)).send().await.ok()?;
    if !Resp.status().is_success() {
        return None;
    }

    if let Some(Length) = Resp.content_length() {
        if Length as usize > MAX_BYTES {
            return None;
        }
    }

    let Bytes = Resp.bytes().await.ok()?;
    if Bytes.len() > MAX_BYTES {
        return None;
    }

    Some(Bytes.to_vec())
}
