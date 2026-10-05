#![allow(non_snake_case)]

use std::time::Duration;

use crate::log::{Subsystem::Core, Level::*};

const SEARCH_URL: &str = "https://api.deezer.com/search";
const TRACK_URL: &str = "https://api.deezer.com/track";
const AUDIODB_URL: &str = "https://www.theaudiodb.com/api/v1/json/2/searchtrack.php";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(6);

pub fn CacheKey(Artist: &str, Title: &str) -> String {
    format!("{}|{}", Artist.trim().to_lowercase(), Title.trim().to_lowercase())
}

async fn _Json(Client: &reqwest::Client, Url: &str) -> Option<serde_json::Value> {
    let Resp = Client.get(Url).send().await.ok()?;
    if !Resp.status().is_success() {
        return None;
    }

    let Text = Resp.text().await.ok()?;
    serde_json::from_str(&Text).ok()
}

async fn _Identify(Client: &reqwest::Client, Query: &str) -> Option<u64> {
    let Url = format!("{}?q={}&limit=1", SEARCH_URL, urlencoding::encode(Query));

    let Body = _Json(Client, &Url).await?;
    Body["data"].as_array()?.first()?["id"].as_u64()
}

async fn _Tempo(Client: &reqwest::Client, Id: u64) -> Option<f64> {
    let Url = format!("{}/{}", TRACK_URL, Id);

    let Body = _Json(Client, &Url).await?;
    _Sane(Body["bpm"].as_f64()?)
}

fn _Sane(Value: f64) -> Option<f64> {
    if Value < 40.0 || Value > 220.0 {
        return None;
    }

    Some(Value)
}

async fn _Deezer(Client: &reqwest::Client, Artist: &str, Title: &str) -> Option<f64> {
    let Id = _Identify(Client, &format!("{} {}", Artist, Title)).await?;
    _Tempo(Client, Id).await
}

async fn _AudioDb(Client: &reqwest::Client, Artist: &str, Title: &str) -> Option<f64> {
    let Url = format!(
        "{}?s={}&t={}",
        AUDIODB_URL,
        urlencoding::encode(Artist),
        urlencoding::encode(Title),
    );

    let Body = _Json(Client, &Url).await?;
    let Raw = Body["track"].as_array()?.first()?["intTempo"].as_str()?;

    _Sane(Raw.parse::<f64>().ok()?)
}

pub async fn Lookup(Artist: &str, Title: &str) -> Option<f64> {
    let Name = Title.trim();
    let By = Artist.trim();

    if Name.is_empty() {
        return None;
    }

    let Client = reqwest::Client::builder().timeout(REQUEST_TIMEOUT).build().ok()?;

    let Found = match _Deezer(&Client, By, Name).await {
        Some(Value) => Some(("deezer", Value)),
        None => _AudioDb(&Client, By, Name).await.map(|Value| ("audiodb", Value)),
    };

    match Found {
        Some((Source, Value)) => {
            log!(Core, Info, "bpm {:.1} from {} for {} - {}", Value, Source, By, Name);
            Some(Value)
        }
        None => {
            log!(Core, Debug, "no bpm for {} - {}", By, Name);
            None
        }
    }
}
