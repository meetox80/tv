#![allow(non_snake_case)]

use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

use rand::Rng;
use rusqlite::{params, OptionalExtension};
use serde_json::{json, Value};

use crate::cover;
use crate::db::Database;
use crate::log::{Subsystem::Core, Level::*};

pub const WIDTH: f64 = 1776.0;
pub const HEIGHT: f64 = 698.0;
pub const ASSET_LIMIT: usize = 12 * 1024 * 1024;

const MAX_SLIDES: usize = 100;
const MAX_ELEMENTS: usize = 200;
const MAX_TEXT: usize = 4000;
const MAX_NAME: usize = 80;
const MAX_LAYER_NAME: usize = 60;
const ASSET_GRACE: i64 = 24 * 60 * 60;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Element {
    pub Id: String,
    pub Kind: String,
    pub X: f64,
    pub Y: f64,
    pub W: f64,
    pub H: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub Text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub Size: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub Weight: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub Color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub Align: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub Font: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub Fill: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub Radius: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub Corners: Option<[f64; 4]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub Rotation: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub Opacity: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub Asset: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub Fit: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub Name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub Hidden: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub Locked: Option<bool>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Slide {
    pub Id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub Background: Option<String>,
    #[serde(default)]
    pub Elements: Vec<Element>,
}

fn _Now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|D| D.as_secs() as i64).unwrap_or(0)
}

fn _NewId(Bytes: usize) -> String {
    let mut Raw = vec![0u8; Bytes];
    rand::rand_core::UnwrapErr(rand::rngs::SysRng).fill_bytes(&mut Raw);
    hex::encode(Raw)
}

fn _ValidId(Id: &str) -> bool {
    (1..=40).contains(&Id.len()) && Id.bytes().all(|B| B.is_ascii_alphanumeric() || B == b'-' || B == b'_')
}

pub fn ValidAsset(Id: &str) -> bool {
    Id.len() == 32 && Id.bytes().all(|B| B.is_ascii_hexdigit())
}

fn _Color(Value: Option<String>) -> Option<String> {
    Value.filter(|C| {
        let Hex = C.strip_prefix('#').unwrap_or("");
        (Hex.len() == 6 || Hex.len() == 8) && Hex.bytes().all(|B| B.is_ascii_hexdigit())
    }).map(|C| C.to_ascii_lowercase())
}

fn _Clamp(Value: f64, Low: f64, High: f64) -> Result<f64, String> {
    if !Value.is_finite() {
        return Err("niepoprawna liczba".into());
    }
    Ok(Value.clamp(Low, High))
}

fn _Corners(Raw: Option<[f64; 4]>) -> Result<Option<[f64; 4]>, String> {
    let Some(Raw) = Raw else { return Ok(None) };
    let mut Out = [0.0; 4];
    for (Slot, Value) in Out.iter_mut().zip(Raw) {
        *Slot = _Clamp(Value, 0.0, 2000.0)?;
    }
    Ok(Some(Out))
}

fn _Element(Raw: Element) -> Result<Element, String> {
    if !_ValidId(&Raw.Id) {
        return Err("niepoprawny identyfikator elementu".into());
    }
    let Kind = Raw.Kind.as_str();
    if !matches!(Kind, "text" | "rect" | "ellipse" | "image") {
        return Err(format!("nieznany rodzaj elementu: {}", Kind));
    }

    let mut E = Element {
        Id: Raw.Id,
        Kind: Raw.Kind.clone(),
        X: _Clamp(Raw.X, -WIDTH * 2.0, WIDTH * 3.0)?,
        Y: _Clamp(Raw.Y, -HEIGHT * 2.0, HEIGHT * 3.0)?,
        W: _Clamp(Raw.W, 1.0, WIDTH * 4.0)?,
        H: _Clamp(Raw.H, 1.0, HEIGHT * 4.0)?,
        Text: None,
        Size: None,
        Weight: None,
        Color: None,
        Align: None,
        Font: None,
        Fill: None,
        Radius: None,
        Corners: None,
        Rotation: match Raw.Rotation {
            Some(R) => Some(_Clamp(R, -180.0, 180.0)?).filter(|R| *R != 0.0),
            None => None,
        },
        Opacity: match Raw.Opacity {
            Some(O) => Some(_Clamp(O, 0.0, 1.0)?),
            None => None,
        },
        Asset: None,
        Fit: None,
        Name: Raw.Name.map(|N| N.trim().chars().take(MAX_LAYER_NAME).collect::<String>()).filter(|N| !N.is_empty()),
        Hidden: Raw.Hidden.filter(|H| *H),
        Locked: Raw.Locked.filter(|L| *L),
    };

    match Kind {
        "text" => {
            let Text = Raw.Text.unwrap_or_default();
            if Text.chars().count() > MAX_TEXT {
                return Err("tekst jest za długi".into());
            }
            E.Text = Some(Text);
            E.Size = Some(_Clamp(Raw.Size.unwrap_or(48.0), 8.0, 400.0)?);
            E.Weight = Some((Raw.Weight.unwrap_or(500).clamp(300, 800) / 100) * 100);
            E.Color = _Color(Raw.Color).or(Some("#f4f2f7".into()));
            E.Align = Raw.Align.filter(|A| matches!(A.as_str(), "left" | "center" | "right")).or(Some("left".into()));
            E.Font = Raw.Font.filter(|F| matches!(F.as_str(), "display" | "body")).or(Some("display".into()));
        }
        "rect" | "ellipse" => {
            E.Fill = _Color(Raw.Fill).or(Some("#e0a8a8".into()));
            if Kind == "rect" {
                E.Radius = Some(_Clamp(Raw.Radius.unwrap_or(0.0), 0.0, 2000.0)?);
                E.Corners = _Corners(Raw.Corners)?;
            }
        }
        _ => {
            let Asset = Raw.Asset.filter(|A| ValidAsset(A)).ok_or("brak obrazu")?;
            E.Asset = Some(Asset);
            E.Fit = Raw.Fit.filter(|F| matches!(F.as_str(), "cover" | "contain" | "fill")).or(Some("cover".into()));
            E.Radius = Some(_Clamp(Raw.Radius.unwrap_or(0.0), 0.0, 2000.0)?);
            E.Corners = _Corners(Raw.Corners)?;
        }
    }

    Ok(E)
}

pub fn Clean(Slides: Vec<Slide>) -> Result<Vec<Slide>, String> {
    if Slides.is_empty() {
        return Err("kolekcja musi mieć co najmniej jeden slajd".into());
    }
    if Slides.len() > MAX_SLIDES {
        return Err(format!("kolekcja może mieć najwyżej {} slajdów", MAX_SLIDES));
    }

    let mut Seen: HashSet<String> = HashSet::new();
    let mut Out = Vec::with_capacity(Slides.len());
    for S in Slides {
        if !_ValidId(&S.Id) || !Seen.insert(S.Id.clone()) {
            return Err("niepoprawny identyfikator slajdu".into());
        }
        if S.Elements.len() > MAX_ELEMENTS {
            return Err(format!("slajd może mieć najwyżej {} elementów", MAX_ELEMENTS));
        }
        let Elements = S.Elements.into_iter().map(_Element).collect::<Result<Vec<_>, _>>()?;
        Out.push(Slide { Id: S.Id, Background: _Color(S.Background), Elements });
    }
    Ok(Out)
}

fn _Name(Raw: &str) -> Option<String> {
    let Name = Raw.trim();
    (!Name.is_empty() && Name.chars().count() <= MAX_NAME).then(|| Name.to_string())
}

fn _Slides(Raw: &str) -> Vec<Slide> {
    serde_json::from_str(Raw).unwrap_or_default()
}

fn _Blank() -> Slide {
    Slide { Id: _NewId(6), Background: None, Elements: Vec::new() }
}

pub fn List(Db: &Database) -> Value {
    let Conn = Db.Lock();
    let Rows = Conn
        .prepare("SELECT id, name, slides, updated FROM slide_collections ORDER BY position, created")
        .and_then(|mut Stmt| {
            Stmt.query_map([], |R| Ok((
                R.get::<_, String>(0)?,
                R.get::<_, String>(1)?,
                R.get::<_, String>(2)?,
                R.get::<_, i64>(3)?,
            )))?.collect::<Result<Vec<_>, _>>()
        })
        .unwrap_or_default();

    Value::Array(Rows.into_iter().map(|(Id, Name, Raw, Updated)| {
        let Slides = _Slides(&Raw);
        json!({
            "Id": Id,
            "Name": Name,
            "Count": Slides.len(),
            "Updated": Updated,
            "Cover": Slides.first(),
        })
    }).collect())
}

pub fn Get(Db: &Database, Id: &str) -> Option<Value> {
    let Row: Option<(String, String, i64)> = Db.Lock().query_row(
        "SELECT name, slides, updated FROM slide_collections WHERE id = ?1",
        [Id],
        |R| Ok((R.get(0)?, R.get(1)?, R.get(2)?)),
    ).optional().ok().flatten();

    Row.map(|(Name, Raw, Updated)| json!({
        "Id": Id,
        "Name": Name,
        "Slides": _Slides(&Raw),
        "Updated": Updated,
    }))
}

pub fn All(Db: &Database) -> Value {
    let Conn = Db.Lock();
    let Rows = Conn
        .prepare("SELECT id, name, slides FROM slide_collections ORDER BY position, created")
        .and_then(|mut Stmt| {
            Stmt.query_map([], |R| Ok((R.get::<_, String>(0)?, R.get::<_, String>(1)?, R.get::<_, String>(2)?)))?
                .collect::<Result<Vec<_>, _>>()
        })
        .unwrap_or_default();

    Value::Array(Rows.into_iter().map(|(Id, Name, Raw)| json!({
        "Id": Id,
        "Name": Name,
        "Slides": _Slides(&Raw),
    })).collect())
}

pub fn Create(Db: &Database, Name: &str) -> Result<Value, String> {
    let Name = _Name(Name).ok_or("niepoprawna nazwa")?;
    let Id = _NewId(8);
    let Slides = serde_json::to_string(&vec![_Blank()]).map_err(|E| E.to_string())?;
    let At = _Now();
    Db.Lock().execute(
        "INSERT INTO slide_collections (id, name, position, slides, created, updated)
         VALUES (?1, ?2, (SELECT COALESCE(MAX(position), 0) + 1 FROM slide_collections), ?3, ?4, ?4)",
        params![Id, Name, Slides, At],
    ).map_err(|E| E.to_string())?;
    Get(Db, &Id).ok_or_else(|| "nie udało się utworzyć kolekcji".into())
}

pub fn Save(Db: &Database, Id: &str, Name: &str, Slides: Vec<Slide>) -> Result<i64, String> {
    let Name = _Name(Name).ok_or("niepoprawna nazwa")?;
    let Slides = Clean(Slides)?;
    let Encoded = serde_json::to_string(&Slides).map_err(|E| E.to_string())?;
    let At = _Now();
    let Changed = Db.Lock().execute(
        "UPDATE slide_collections SET name = ?2, slides = ?3, updated = ?4 WHERE id = ?1",
        params![Id, Name, Encoded, At],
    ).map_err(|E| E.to_string())?;
    if Changed == 0 {
        return Err("nie znaleziono kolekcji".into());
    }
    Ok(At)
}

pub fn Delete(Db: &Database, Id: &str) -> bool {
    let Removed = Db.Lock().execute("DELETE FROM slide_collections WHERE id = ?1", [Id]).unwrap_or(0) > 0;
    if Removed {
        Sweep(Db);
    }
    Removed
}

pub fn PutAsset(Db: &Database, Bytes: &[u8]) -> Result<String, String> {
    if Bytes.is_empty() || Bytes.len() > ASSET_LIMIT {
        return Err("obraz jest pusty albo za duży".into());
    }
    let Mime = cover::SniffMime(Bytes).ok_or("nieobsługiwany format obrazu")?;
    let Id = _NewId(16);
    Db.Lock().execute(
        "INSERT INTO slide_assets (id, mime, bytes, created) VALUES (?1, ?2, ?3, ?4)",
        params![Id, Mime, Bytes, _Now()],
    ).map_err(|E| E.to_string())?;
    Ok(Id)
}

pub fn Asset(Db: &Database, Id: &str) -> Option<(Vec<u8>, String)> {
    if !ValidAsset(Id) {
        return None;
    }
    Db.Lock().query_row(
        "SELECT bytes, mime FROM slide_assets WHERE id = ?1",
        [Id],
        |R| Ok((R.get(0)?, R.get(1)?)),
    ).optional().ok().flatten()
}

pub fn Sweep(Db: &Database) {
    let Conn = Db.Lock();
    let Raws: Vec<String> = Conn
        .prepare("SELECT slides FROM slide_collections")
        .and_then(|mut Stmt| Stmt.query_map([], |R| R.get(0))?.collect::<Result<Vec<_>, _>>())
        .unwrap_or_default();

    let Used: HashSet<String> = Raws
        .iter()
        .flat_map(|Raw| _Slides(Raw))
        .flat_map(|S| S.Elements)
        .filter_map(|E| E.Asset)
        .collect();

    let Candidates: Vec<String> = Conn
        .prepare("SELECT id FROM slide_assets WHERE created < ?1")
        .and_then(|mut Stmt| Stmt.query_map([_Now() - ASSET_GRACE], |R| R.get(0))?.collect::<Result<Vec<_>, _>>())
        .unwrap_or_default();

    let mut Removed = 0;
    for Id in Candidates.into_iter().filter(|Id| !Used.contains(Id)) {
        Removed += Conn.execute("DELETE FROM slide_assets WHERE id = ?1", [&Id]).unwrap_or(0);
    }
    if Removed > 0 {
        log!(Core, Info, "removed {} unused slide image(s)", Removed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _Text(Id: &str) -> Element {
        Element {
            Id: Id.into(), Kind: "text".into(), X: 10.0, Y: 10.0, W: 400.0, H: 80.0,
            Text: Some("Witaj".into()), Size: Some(9999.0), Weight: Some(950), Color: Some("red".into()),
            Align: Some("justify".into()), Font: None, Fill: None, Radius: None, Corners: None, Rotation: Some(400.0), Opacity: Some(3.0), Asset: None, Fit: None,
            Name: Some("  ".into()), Hidden: Some(false), Locked: Some(true),
        }
    }

    #[test]
    fn sanitizes_elements() {
        let Out = Clean(vec![Slide { Id: "a".into(), Background: Some("#0C0B0F".into()), Elements: vec![_Text("t1")] }]).unwrap();
        let E = &Out[0].Elements[0];
        assert_eq!(E.Size, Some(400.0));
        assert_eq!(E.Weight, Some(800));
        assert_eq!(E.Color.as_deref(), Some("#f4f2f7"));
        assert_eq!(E.Align.as_deref(), Some("left"));
        assert_eq!(E.Opacity, Some(1.0));
        assert_eq!(E.Rotation, Some(180.0));
        assert_eq!(E.Corners, None);
        assert_eq!(Out[0].Background.as_deref(), Some("#0c0b0f"));
        assert_eq!(E.Name, None);
        assert_eq!(E.Hidden, None);
        assert_eq!(E.Locked, Some(true));
    }

    #[test]
    fn rejects_bad_documents() {
        assert!(Clean(Vec::new()).is_err());
        let mut Bad = _Text("t1");
        Bad.Kind = "script".into();
        assert!(Clean(vec![Slide { Id: "a".into(), Background: None, Elements: vec![Bad] }]).is_err());
        let mut Image = _Text("i1");
        Image.Kind = "image".into();
        Image.Asset = Some("../../etc/passwd".into());
        assert!(Clean(vec![Slide { Id: "a".into(), Background: None, Elements: vec![Image] }]).is_err());
        let Twin = Slide { Id: "same".into(), Background: None, Elements: Vec::new() };
        assert!(Clean(vec![Twin.clone(), Twin]).is_err());
        let mut Wild = _Text("t2");
        Wild.X = f64::NAN;
        assert!(Clean(vec![Slide { Id: "b".into(), Background: None, Elements: vec![Wild] }]).is_err());
    }
}
