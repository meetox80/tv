#![allow(non_snake_case)]

use std::sync::Arc;
use std::time::{Duration, Instant};
use base64::Engine;
use crate::AppState;
use crate::NowPlaying;
use crate::cover;
use crate::metadata;
use crate::CoverLookup;
use crate::db::Database;
use crate::log::{Subsystem::Spotify, Level::*};

const POLL_INTERVAL: Duration = Duration::from_secs(10);
const AUTHORIZE_BASE: &str = "https://accounts.spotify.com/authorize";
const TOKEN_URL: &str = "https://accounts.spotify.com/api/token";
const CURRENTLY_PLAYING_URL: &str = "https://api.spotify.com/v1/me/player/currently-playing";
const SCOPES: &str = "user-read-currently-playing user-read-playback-state";
const LOOPBACK_REDIRECT: &str = "http://127.0.0.1:9999/api/spotify/callback";
const TOKEN_SAFETY_MARGIN: Duration = Duration::from_secs(60);

pub fn RedirectUri(Db: &Database) -> String {
    let Host = Db.GetManagementHostname();
    if Host.is_empty() {
        return LOOPBACK_REDIRECT.to_string();
    }

    let Trimmed = Host.trim().trim_end_matches('/');
    if Trimmed.starts_with("http://") || Trimmed.starts_with("https://") {
        format!("{}/api/spotify/callback", Trimmed)
    } else {
        format!("https://{}/api/spotify/callback", Trimmed)
    }
}

pub fn AuthorizeUrl(ClientId: &str, Redirect: &str, State: &str) -> String {
    format!(
        "{}?client_id={}&response_type=code&redirect_uri={}&scope={}&state={}&show_dialog=true",
        AUTHORIZE_BASE,
        urlencoding::encode(ClientId),
        urlencoding::encode(Redirect),
        urlencoding::encode(SCOPES),
        urlencoding::encode(State),
    )
}

fn BasicAuth(ClientId: &str, ClientSecret: &str) -> String {
    let Raw = format!("{}:{}", ClientId, ClientSecret);
    format!("Basic {}", base64::engine::general_purpose::STANDARD.encode(Raw))
}

async fn PostToken(
    Client: &reqwest::Client,
    ClientId: &str,
    ClientSecret: &str,
    Form: &[(&str, &str)],
) -> Result<serde_json::Value, String> {
    let Resp = Client
        .post(TOKEN_URL)
        .header("Authorization", BasicAuth(ClientId, ClientSecret))
        .form(Form)
        .send()
        .await
        .map_err(|E| E.to_string())?;

    let Status = Resp.status();
    let Text = Resp.text().await.map_err(|E| E.to_string())?;
    let Body: serde_json::Value = serde_json::from_str(&Text).map_err(|E| E.to_string())?;

    if !Status.is_success() {
        let Detail = Body["error_description"]
            .as_str()
            .or_else(|| Body["error"].as_str())
            .unwrap_or("unknown error");
        return Err(format!("{}: {}", Status, Detail));
    }

    Ok(Body)
}

pub async fn ExchangeCode(
    ClientId: &str,
    ClientSecret: &str,
    Redirect: &str,
    Code: &str,
) -> Result<String, String> {
    let Client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|E| E.to_string())?;

    let Body = PostToken(&Client, ClientId, ClientSecret, &[
        ("grant_type", "authorization_code"),
        ("code", Code),
        ("redirect_uri", Redirect),
    ]).await?;

    match Body["refresh_token"].as_str() {
        Some(Token) if !Token.is_empty() => Ok(Token.to_string()),
        _ => Err("no refresh token in response".to_string()),
    }
}

async fn RefreshAccessToken(
    Client: &reqwest::Client,
    ClientId: &str,
    ClientSecret: &str,
    RefreshToken: &str,
) -> Result<(String, Duration), String> {
    let Body = PostToken(Client, ClientId, ClientSecret, &[
        ("grant_type", "refresh_token"),
        ("refresh_token", RefreshToken),
    ]).await?;

    let Access = Body["access_token"]
        .as_str()
        .ok_or_else(|| "no access token in response".to_string())?
        .to_string();

    let Lifetime = Duration::from_secs(Body["expires_in"].as_u64().unwrap_or(3600));

    Ok((Access, Lifetime))
}

pub async fn AppToken(
    Client: &reqwest::Client,
    ClientId: &str,
    ClientSecret: &str,
) -> Result<(String, Duration), String> {
    let Body = PostToken(Client, ClientId, ClientSecret, &[("grant_type", "client_credentials")]).await?;

    let Access = Body["access_token"]
        .as_str()
        .ok_or_else(|| "no access token in response".to_string())?
        .to_string();

    Ok((Access, Duration::from_secs(Body["expires_in"].as_u64().unwrap_or(3600))))
}

struct CachedToken {
    Value: String,
    ExpiresAt: Instant,
    ForRefreshToken: String,
}

pub async fn StartPoller(State: Arc<AppState>) {
    let Client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();

    let mut Cached: Option<CachedToken> = None;

    loop {
        tokio::time::sleep(POLL_INTERVAL).await;

        if State.Db.GetScrobbleMethod() != "spotify" {
            continue;
        }

        let ClientId = State.Db.GetSpotifyClientId();
        let ClientSecret = State.Db.GetSpotifyClientSecret();
        let RefreshToken = State.Db.GetSpotifyRefreshToken();

        if ClientId.is_empty() || ClientSecret.is_empty() || RefreshToken.is_empty() {
            continue;
        }

        let Stale = match &Cached {
            Some(C) => C.ForRefreshToken != RefreshToken || Instant::now() >= C.ExpiresAt,
            None => true,
        };

        if Stale {
            match RefreshAccessToken(&Client, &ClientId, &ClientSecret, &RefreshToken).await {
                Ok((Value, Lifetime)) => {
                    Cached = Some(CachedToken {
                        Value,
                        ExpiresAt: Instant::now() + Lifetime.saturating_sub(TOKEN_SAFETY_MARGIN),
                        ForRefreshToken: RefreshToken.clone(),
                    });
                }
                Err(E) => {
                    log!(Spotify, Error, "token refresh failed: {}", E);
                    Cached = None;
                    continue;
                }
            }
        }

        let Access = match &Cached {
            Some(C) => C.Value.clone(),
            None => continue,
        };

        match FetchNowPlaying(&Client, &Access).await {
            Ok(Some((Track, ArtUrl))) => {
                let Changed = State.CurrentTrack.lock().unwrap()
                    .as_ref()
                    .map_or(true, |C| C.Title != Track.Title || C.Artist != Track.Artist);
                if Changed {
                    log!(Spotify, Info, "now playing: {} - {}", Track.Artist, Track.Title);

                    let Art = match ArtUrl.filter(|U| !metadata::IsLastfmPlaceholder(U)) {
                        Some(Url) => {
                            let Fetched = cover::Fetch(&Client, &Url).await.and_then(cover::Accept);
                            if Fetched.is_none() {
                                log!(Spotify, Warn, "cover fetch failed: {}", Url);
                            }
                            Fetched
                        }
                        None => None,
                    };

                    let (Title, Artist) = (Track.Title.clone(), Track.Artist.clone());
                    State.SetTrack(Some(Track));

                    if Art.is_some() {
                        State.SetCover(Art);
                    } else {
                        State.LookupCover(Title, Artist, CoverLookup::Chain);
                    }
                }
            }
            Ok(None) => {
                if State.CurrentTrack.lock().unwrap().is_some() {
                    log!(Spotify, Info, "nothing playing");
                    State.SetTrack(None);
                }
            }
            Err(E) => {
                log!(Spotify, Error, "poll failed: {}", E);
                Cached = None;
            }
        }
    }
}

fn PickAlbumArt(Images: &serde_json::Value) -> Option<String> {
    let List = Images.as_array()?;

    let Best = List
        .iter()
        .filter(|I| I["url"].as_str().is_some())
        .min_by_key(|I| (I["width"].as_i64().unwrap_or(0) - 320).abs())?;

    Best["url"].as_str().map(|U| U.to_string())
}

async fn FetchNowPlaying(
    Client: &reqwest::Client,
    AccessToken: &str,
) -> Result<Option<(NowPlaying, Option<String>)>, String> {
    let Resp = Client
        .get(CURRENTLY_PLAYING_URL)
        .header("Authorization", format!("Bearer {}", AccessToken))
        .send()
        .await
        .map_err(|E| E.to_string())?;

    if Resp.status() == reqwest::StatusCode::NO_CONTENT {
        return Ok(None);
    }

    if !Resp.status().is_success() {
        return Err(format!("HTTP {}", Resp.status()));
    }

    let Text = Resp.text().await.map_err(|E| E.to_string())?;
    let Body: serde_json::Value = serde_json::from_str(&Text).map_err(|E| E.to_string())?;

    if !Body["is_playing"].as_bool().unwrap_or(false) {
        return Ok(None);
    }

    let Item = &Body["item"];
    let Title = Item["name"].as_str().unwrap_or("").to_string();

    if Title.is_empty() {
        return Ok(None);
    }

    let Artist = Item["artists"]
        .as_array()
        .map(|A| {
            A.iter()
                .filter_map(|X| X["name"].as_str())
                .collect::<Vec<&str>>()
                .join(", ")
        })
        .unwrap_or_default();

    let ArtUrl = PickAlbumArt(&Item["album"]["images"]);

    Ok(Some((NowPlaying { Title, Artist }, ArtUrl)))
}
