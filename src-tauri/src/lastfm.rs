#![allow(non_snake_case)]

use std::sync::Arc;
use std::time::Duration;
use crate::AppState;
use crate::NowPlaying;
use crate::cover;
use crate::metadata;
use crate::CoverLookup;
use crate::log::{Subsystem::Lastfm, Level::*};

const POLL_INTERVAL: Duration = Duration::from_secs(10);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);
const API_BASE: &str = "https://ws.audioscrobbler.com/2.0/";

pub struct Failure {
    pub Code: String,
    pub Detail: String,
}

impl Failure {
    fn New(Code: impl Into<String>, Detail: impl Into<String>) -> Self {
        Self { Code: Code.into(), Detail: Detail.into() }
    }
}

pub async fn StartPoller(State: Arc<AppState>) {
    let Client = reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()
        .unwrap();

    loop {
        tokio::time::sleep(POLL_INTERVAL).await;

        if State.Db.GetScrobbleMethod() != "lastfm" {
            continue;
        }

        let ApiKey = State.Db.GetLastfmApiKey();
        let Username = State.Db.GetLastfmUsername();

        if ApiKey.is_empty() || Username.is_empty() {
            continue;
        }

        match FetchNowPlaying(&Client, &ApiKey, &Username).await {
            Ok(Some((Track, ArtUrl))) => {
                State.SetLastfmError(None);

                let Changed = State.CurrentTrack.lock().unwrap()
                    .as_ref()
                    .map_or(true, |C| C.Title != Track.Title || C.Artist != Track.Artist);
                if Changed {
                    log!(Lastfm, Info, "now playing: {} - {}", Track.Artist, Track.Title);

                    let Art = match ArtUrl.filter(|U| !metadata::IsLastfmPlaceholder(U)) {
                        Some(Url) => {
                            let Fetched = cover::Fetch(&Client, &Url).await.and_then(cover::Accept);
                            if Fetched.is_none() {
                                log!(Lastfm, Warn, "cover fetch failed: {}", Url);
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
                State.SetLastfmError(None);

                if State.CurrentTrack.lock().unwrap().is_some() {
                    log!(Lastfm, Info, "nothing playing");
                    State.SetTrack(None);
                }
            }
            Err(F) => {
                log!(Lastfm, Error, "poll failed: {}", F.Detail);
                State.SetLastfmError(Some(F.Code));
            }
        }
    }
}

pub async fn Verify(ApiKey: &str, Username: &str) -> Result<(), Failure> {
    let Client = reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()
        .map_err(|E| Failure::New("network", E.to_string()))?;

    RecentTracks(&Client, ApiKey, Username).await.map(|_| ())
}

fn PickAlbumArt(Images: &serde_json::Value) -> Option<String> {
    let List = Images.as_array()?;
    const ORDER: [&str; 3] = ["extralarge", "large", "medium"];

    for Size in ORDER {
        let Found = List
            .iter()
            .find(|I| I["size"].as_str() == Some(Size))
            .and_then(|I| I["#text"].as_str())
            .filter(|U| !U.is_empty());

        if let Some(Url) = Found {
            return Some(Url.to_string());
        }
    }

    None
}

async fn RecentTracks(
    Client: &reqwest::Client,
    ApiKey: &str,
    Username: &str,
) -> Result<serde_json::Value, Failure> {
    let Url = format!(
        "{}?method=user.getrecenttracks&user={}&api_key={}&format=json&limit=1",
        API_BASE,
        urlencoding::encode(Username),
        urlencoding::encode(ApiKey),
    );

    let Resp = Client
        .get(&Url)
        .send()
        .await
        .map_err(|E| Failure::New("network", E.to_string()))?;

    let Status = Resp.status();
    let Text = Resp
        .text()
        .await
        .map_err(|E| Failure::New("network", E.to_string()))?;

    let Body: serde_json::Value = match serde_json::from_str(&Text) {
        Ok(Parsed) => Parsed,
        Err(E) => {
            return Err(Failure::New(
                format!("http.{}", Status.as_u16()),
                format!("HTTP {}: {}", Status, E),
            ));
        }
    };

    if let Some(Code) = Body["error"].as_u64() {
        let Message = Body["message"].as_str().unwrap_or("unknown");
        return Err(Failure::New(format!("code.{}", Code), format!("{} ({})", Message, Code)));
    }

    if !Status.is_success() {
        return Err(Failure::New(format!("http.{}", Status.as_u16()), format!("HTTP {}", Status)));
    }

    Ok(Body)
}

async fn FetchNowPlaying(
    Client: &reqwest::Client,
    ApiKey: &str,
    Username: &str,
) -> Result<Option<(NowPlaying, Option<String>)>, Failure> {
    let Body = RecentTracks(Client, ApiKey, Username).await?;

    let Tracks = &Body["recenttracks"]["track"];
    let First = match Tracks.as_array().and_then(|A: &Vec<serde_json::Value>| A.first()) {
        Some(T) => T,
        None => return Ok(None),
    };

    let IsNowPlaying = First["@attr"]["nowplaying"]
        .as_str()
        .map_or(false, |V| V == "true");

    if !IsNowPlaying {
        return Ok(None);
    }

    let Title = First["name"].as_str().unwrap_or("").to_string();
    let Artist = First["artist"]["#text"].as_str().unwrap_or("").to_string();

    if Title.is_empty() {
        return Ok(None);
    }

    let ArtUrl = PickAlbumArt(&First["image"]);

    Ok(Some((NowPlaying { Title, Artist }, ArtUrl)))
}
