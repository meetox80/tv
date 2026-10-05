#![allow(non_snake_case)]

use std::collections::HashMap;
use std::sync::Arc;
use axum::{
    extract::{Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};

use crate::AppState;

fn _PublicBase(AppState: &Arc<AppState>) -> String {
    let Host = AppState.Db.GetManagementHostname();
    let Trimmed = Host.trim().trim_end_matches('/');

    if Trimmed.is_empty() {
        return "http://127.0.0.1:9999".to_string();
    }

    if Trimmed.starts_with("http://") || Trimmed.starts_with("https://") {
        Trimmed.to_string()
    } else {
        format!("https://{}", Trimmed)
    }
}

fn _Index(Base: &str) -> String {
    format!(
"# zstio-tv

Interaktywny llms.txt. Sekcję wybierasz przez ?documentation=

Scrobbling  -> {Base}/llms.txt?documentation=scrobbling
Serial      -> {Base}/llms.txt?documentation=serial
Lekcje      -> {Base}/llms.txt?documentation=lessons
Głosowanie  -> {Base}/llms.txt?documentation=voting
"
    )
}

fn _Lessons(Base: &str, ApiKey: &str, Duration: u8) -> String {
    format!(
"# zstio-tv / lekcje

Długość lekcji zmienia się w panelu ({Base} -> Ekran -> Lekcje) albo zdalnie.
Dozwolone wartości: 30 i 45. Teraz ustawione: {Duration}.

## HTTP

POST {Base}/api/lessons/poke?key={ApiKey}
Content-Type: application/json

{{
  \"Duration\": 30
}}

Odpowiedzi: 200 przy zmianie, 400 przy innej wartości niż 30 lub 45,
401 przy złym kluczu. Klucz jest ten sam co przy /api/playback/poke.

  curl -X POST \"{Base}/api/lessons/poke?key={ApiKey}\" \\
    -H \"Content-Type: application/json\" \\
    -d '{{\"Duration\": 30}}'

## Serial

Ta sama ramka co przy utworach, tylko z polem Lesson. Title jest opcjonalny,
więc ramka może zmieniać samą długość lekcji:

{{
  \"Seq\": 1758270000123,
  \"At\": 1758270000,
  \"Lesson\": 30
}}

Można też zmienić jedno i drugie naraz, dokładając Title i Artist.
Szyfrowanie, podpis i ochrona przed powtórkami działają tak samo:
{Base}/llms.txt?documentation=serial
"
    )
}


fn _Serial(Base: &str, PublicKey: &str) -> String {
    format!(
"# zstio-tv / serial

made for a specific retard, especially for you

---

Ekran nasłuchuje na porcie szeregowym i przyjmuje ramki z aktualnym utworem.
Port i prędkość ustawiasz w panelu: {Base} -> Scrobbling -> Serial.

## Ramka

Jedna linia ASCII zakończona znakiem nowej linii:

ZFM1:<base64(nonce || ciphertext)>

  nonce       24 losowe bajty
  ciphertext  XChaCha20-Poly1305, AAD = \"ZFM1:\"
  klucz       SHA-256(\"zstio-fm serial v1\" || X25519(nadawca, ekran) || pk_nadawcy || pk_ekranu)

Linie bez prefiksu ZFM1: są ignorowane, dzięki czemu odbiornik odzyskuje
synchronizację po śmieciach na łączu. Limit linii to 512 KiB.

## Klucze

Ekran ma parę X25519. Jego klucz publiczny:

  {PublicKey}

Nadawca generuje własną parę X25519 i wkleja swój klucz publiczny w panelu.
Bez tego żadna ramka nie przejdzie.

## Zawartość

Po odszyfrowaniu ramka to JSON:

{{
  \"Seq\": 1758270000123,
  \"At\": 1758270000,
  \"Title\": \"Never Gonna Give You Up\",
  \"Artist\": \"Rick Astley\",
  \"Cover\": \"<base64 jpeg/png/webp/gif, opcjonalnie>\",
  \"CoverLookup\": \"deezer\"
}}

  Seq          ściśle rosnący, np. czas w ms. Ramki z Seq mniejszym lub równym
               ostatnio przyjętemu są odrzucane, także po restarcie ekranu.
  At           czas uniksowy w sekundach, tolerancja 120 s w obie strony.
  Cover        do 2 MB, ale transfer przez UART jest wolny, więc lepiej go pomijać.
  CoverLookup  opcjonalne, domyślnie wyłączone. Jeśli ramka nie niesie okładki,
               ekran sam jej poszuka po tytule i wykonawcy:
                 \"deezer\"  tylko w Deezerze,
                 \"auto\"    po kolei w Last.fm, Deezerze i iTunes.
               Wymaga dostępu ekranu do internetu. Okładka pojawia się z
               opóźnieniem kilku sekund i tylko wtedy, gdy w międzyczasie nie
               przyszedł kolejny utwór. Inna wartość odrzuca całą ramkę.

Bez CoverLookup ekran niczego nie pobiera z sieci na podstawie ramki, więc
łącze szeregowe pozostaje jedynym źródłem danych.

## Nadajnik w Pythonie

  pip install pynacl pyserial pycryptodome

  import base64, json, os, time, serial
  from nacl.bindings import crypto_scalarmult
  from nacl.public import PrivateKey
  from nacl.hash import sha256
  from nacl.encoding import RawEncoder
  from Crypto.Cipher import ChaCha20_Poly1305

  Screen = base64.b64decode(\"{PublicKey}\")
  Mine = PrivateKey.generate()
  print(\"wklej w panelu:\", base64.b64encode(bytes(Mine.public_key)).decode())

  Shared = crypto_scalarmult(bytes(Mine), Screen)
  Key = sha256(b\"zstio-fm serial v1\" + Shared + bytes(Mine.public_key) + Screen,
               encoder=RawEncoder)

  def Send(Port, Title, Artist):
      Body = json.dumps({{
          \"Seq\": int(time.time() * 1000),
          \"At\": int(time.time()),
          \"Title\": Title,
          \"Artist\": Artist,
      }}).encode()

      Nonce = os.urandom(24)
      Box = ChaCha20_Poly1305.new(key=Key, nonce=Nonce)
      Box.update(b\"ZFM1:\")
      Cipher, Tag = Box.encrypt_and_digest(Body)
      Port.write(b\"ZFM1:\" + base64.b64encode(Nonce + Cipher + Tag) + b\"\\n\")

  with serial.Serial(\"/dev/ttyUSB0\", 115200) as Port:
      Send(Port, \"Never Gonna Give You Up\", \"Rick Astley\")
"
    )
}


fn _Scrobbling(Base: &str) -> String {
    format!(
"# zstio-tv / scrobbling

> Jak wysyłać do zstio-tv info o tym, co aktualnie gra.

zstio-tv bierze dane o odtwarzaniu z jednego z trzech źródeł: Last.fm, Spotify
albo prosto po HTTP. Ten dokument opisuje to trzecie, w panelu nazwane
\"API Poking\". Używasz go, jeśli masz własne źródło muzyki.

Adres bazowy: {Base}

## Zanim zaczniesz

Wejdź w panelu w Ustawienia, sekcja Scrobbling, i zaznacz metodę API. Jeśli
tego nie zrobisz, dane owszem wejdą, ale za chwilę nadpisze je poller Last.fm
albo Spotify.

Skopiuj stamtąd klucz API. Niżej piszę na niego TWOJ_KLUCZ. Idzie w URL-u jako
?key= i to jedyne, co chroni te endpointy, więc traktuj go jak hasło.

## Wysyłanie utworu

POST /api/playback/poke?key=TWOJ_KLUCZ
Content-Type: application/json

{{
  \"Title\": \"Never Gonna Give You Up\",
  \"Artist\": \"Rick Astley\"
}}

W odpowiedzi dostajesz 200. Puste Title to 400, zły klucz to 401.

Wysyłaj to przy każdej zmianie utworu, nie w pętli. To, co wyślesz, trzyma się
do następnego razu.

## Okładka

Masz trzy opcje.

1. Przez URL. Najprościej, serwer sam sobie pobierze obrazek.

{{
  \"Title\": \"Never Gonna Give You Up\",
  \"Artist\": \"Rick Astley\",
  \"CoverUrl\": \"https://example.com/cover.jpg\"
}}

2. Przez base64, w tym samym requeście. Wygodne, jeśli możesz wysłać tylko
JSON. Działa i z prefiksem data:, i bez niego. Tylko pamiętaj, że base64
powiększa obrazek o jakąś jedną trzecią.

{{
  \"Title\": \"Never Gonna Give You Up\",
  \"Artist\": \"Rick Astley\",
  \"Cover\": \"data:image/jpeg;base64,/9j/4AAQ...\"
}}

3. Surowym plikiem, osobnym requestem. Nic się nie powiększa, więc przy
większych okładkach to najlepsza opcja. Wyślij go PO utworze, bo poke czyści
poprzednią okładkę.

POST /api/playback/cover?key=TWOJ_KLUCZ
Content-Type: image/jpeg

<zawartość pliku>

Limit to 2 MB, formaty: jpeg, png, webp, gif. Format sprawdzam po zawartości
pliku, a nie po nagłówku, więc źle ustawiony Content-Type niczego nie zepsuje,
ale plik, który nie jest obrazkiem, dostanie 400.

4. Nie wysyłasz okładki wcale, tylko prosisz ekran, żeby jej poszukał.
Domyślnie jest to wyłączone, włączasz to polem CoverLookup:

{{
  \"Title\": \"Never Gonna Give You Up\",
  \"Artist\": \"Rick Astley\",
  \"CoverLookup\": \"auto\"
}}

  \"auto\"    szuka po kolei w Last.fm, Deezerze i iTunes,
  \"deezer\"  szuka tylko w Deezerze.

Inna wartość to 400. Wyszukiwanie trwa kilka sekund i dzieje się w tle, więc
poke odpowiada od razu. Jeżeli zanim coś się znajdzie, wgrasz okładkę przez
/api/playback/cover albo przyjdzie kolejny utwór, znaleziona okładka zostanie
pominięta.

Przy Last.fm i Spotify nic nie trzeba ustawiać: kiedy serwis nie poda okładki
(Last.fm często zwraca pusty obrazek zastępczy), ekran sam szuka jej w
Last.fm, Deezerze i iTunes.

## Sprawdzanie stanu

GET /api/playback/current

{{
  \"Title\": \"Never Gonna Give You Up\",
  \"Artist\": \"Rick Astley\",
  \"Cover\": \"/api/playback/cover?v=7\"
}}

Jak nic nie gra, dostajesz null. Numer v rośnie przy każdej nowej okładce,
więc ten adres możesz sobie cache'ować bez końca.

GET /api/playback/cover

Same bajty okładki, albo 404.

## Wszystko razem

curl -X POST \"{Base}/api/playback/poke?key=TWOJ_KLUCZ\" \\
  -H \"Content-Type: application/json\" \\
  -d '{{\"Title\": \"Never Gonna Give You Up\", \"Artist\": \"Rick Astley\"}}'

curl -X POST \"{Base}/api/playback/cover?key=TWOJ_KLUCZ\" \\
  -H \"Content-Type: image/jpeg\" \\
  --data-binary @cover.jpg
"
    )
}

fn _Voting(Base: &str, VoteUrl: &str) -> String {
    format!(
"# zstio-tv / głosowanie

> Uczniowie proponują utwory na szkolną playlistę i głosują na nie.

Strona głosowania: {VoteUrl}
Panel: {Base} -> Ustawienia -> Głosowanie

## Jak to działa

1. Uczeń skanuje kod QR z telewizora i otwiera stronę głosowania.
2. Wyszukuje utwór i wysyła propozycję. Przy pierwszym wysłaniu z danego
   urządzenia rozwiązuje captchę Cloudflare Turnstile.
3. Administrator przegląda propozycje w panelu (przycisk Zgłoszenia w lewym
   dolnym rogu ustawień): przesunięcie w prawo zatwierdza, w lewo odrzuca.
4. Zatwierdzone utwory trafiają na listę na stronie i na telewizorze.
   Każdy zaczyna z jednym głosem, a uczniowie mogą głosować w górę lub w dół.
5. Kiedy utwór trafi na szkolną playlistę, oznaczasz go w panelu jako dodany.

Jedno urządzenie może wysłać 3 propozycje dziennie. Limit odnawia się o
północy czasu lokalnego ekranu. Ten sam utwór nie może czekać w kolejce dwa
razy, nawet jeśli pochodzi z innego serwisu.

## Wyszukiwanie

Po kolei: Spotify (jeśli w panelu są zapisane Client ID i Client secret,
logowanie konta nie jest potrzebne), Last.fm (jeśli jest klucz API),
a na końcu Deezer, który działa bez żadnych kluczy. Brakujące okładki, albumy,
długości i oznaczenia treści dla dorosłych są uzupełniane z Last.fm, Deezera
i iTunes. Wszystkie te zapytania wykonuje ekran, a okładki idą przez jego
serwer, więc telefon ucznia łączy się tylko ze stroną głosowania i z Cloudflare.

## Konfiguracja

1. W panelu Cloudflare utwórz widżet Turnstile dla hosta strony głosowania.
2. Wklej w panelu klucz witryny (site key) i klucz tajny (secret key).
3. Włącz głosowanie. Bez kluczy Turnstile nie da się go włączyć.

Serwer głosowania nasłuchuje tylko na 127.0.0.1:10000, więc jest dostępny
wyłącznie przez tunel Cloudflare. Publiczny host w Cloudflare musi wskazywać
na http://localhost:10000.

## Prywatność i ochrona przed nadużyciami

Urządzenie dostaje losowy token w ciasteczku HttpOnly. Serwer przechowuje
tylko jego skrót HMAC-SHA-256 z kluczem, który nie opuszcza ekranu. Nie są
zapisywane imiona, adresy e-mail ani adresy IP. Adres IP służy wyłącznie do
limitów zapytań: jest haszowany kluczem trzymanym tylko w pamięci RAM
i nigdy nie trafia na dysk.

Urządzenia nieaktywne przez 180 dni są usuwane razem z ich głosami. Zgłoszone
przez nie utwory zostają, ale bez powiązania z urządzeniem. Zablokowane
urządzenia zostają w bazie, dopóki nie zdejmiesz blokady.

Identyfikator urządzenia widoczny w panelu (np. a3f9·c21e·07bd) to początek
skrótu. Po nim możesz zablokować urządzenie albo zdjąć blokadę. Blokada
odrzuca oczekujące propozycje tego urządzenia i przestaje liczyć jego głosy.
"
    )
}

pub async fn LlmsTxt(
    State(AppState): State<Arc<AppState>>,
    Query(Params): Query<HashMap<String, String>>,
) -> Response {
    let Base = _PublicBase(&AppState);

    let (Status, Body) = match Params.get("documentation").map(|S| S.trim()) {
        None | Some("") => (StatusCode::OK, _Index(&Base)),
        Some("scrobbling") => (StatusCode::OK, _Scrobbling(&Base)),
        Some("lessons") => (
            StatusCode::OK,
            _Lessons(&Base, &AppState.Db.GetApiKey(), AppState.Db.GetLessonDuration()),
        ),
        Some("voting") => {
            let Host = crate::cloudflared::NormalizeHostname(&AppState.Db.GetVotingHostname());
            let VoteUrl = if Host.is_empty() { "(nie ustawiono)".to_string() } else { format!("https://{}", Host) };
            (StatusCode::OK, _Voting(&Base, &VoteUrl))
        }
        Some("serial") => {
            let (_, Key) = crate::serial::EnsureKeypair(&AppState.Db);
            (StatusCode::OK, _Serial(&Base, &Key))
        }
        Some(Unknown) => (
            StatusCode::NOT_FOUND,
            format!(
                "Nie mam dokumentacji dla \"{}\".\n\nJest: scrobbling, serial, lessons, voting\nSpis: {}/llms.txt\n",
                Unknown, Base
            ),
        ),
    };

    (
        Status,
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        Body,
    ).into_response()
}
