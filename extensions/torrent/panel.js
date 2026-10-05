const Route = "/torrent"
const MaxBytes = 16 * 1024 * 1024
const FirstFiles = 100
const Units = ["B", "KB", "MB", "GB", "TB", "PB"]
const Utf8 = new TextDecoder("utf-8")

const Style = `
.TorrentPage {
  position: fixed;
  inset: 0;
  z-index: 2147483000;
  overflow-y: auto;
  background: var(--bg, #050507);
  color: var(--ink, #f4f2f7);
  font-family: Inter, system-ui, sans-serif;
  -webkit-font-smoothing: antialiased;
  animation: TorrentFade 320ms ease both;
}

.TorrentTop {
  position: sticky;
  top: 0;
  z-index: 2;
  display: flex;
  height: 64px;
  align-items: center;
  gap: 18px;
  border-bottom: 1px solid var(--line, rgba(255, 255, 255, 0.06));
  background: rgba(5, 5, 7, 0.92);
  padding: 0 max(24px, calc((100vw - 880px) / 2));
}

.TorrentBack {
  display: inline-flex;
  height: 34px;
  align-items: center;
  gap: 8px;
  margin-left: -10px;
  border-radius: 9px;
  padding: 0 10px;
  font-size: 14px;
  color: var(--ink-soft, #b9b5c6);
  text-decoration: none;
  transition: background 160ms ease, color 160ms ease;
}

.TorrentBack:hover {
  background: rgba(255, 255, 255, 0.06);
  color: var(--ink, #f4f2f7);
}

.TorrentBack svg {
  height: 16px;
  width: 16px;
}

.TorrentBrand {
  font-family: var(--display, "Space Grotesk", Inter, sans-serif);
  font-size: 17px;
  font-weight: 600;
  letter-spacing: -0.02em;
}

.TorrentMain {
  width: min(880px, 100%);
  margin: 0 auto;
  padding: 56px 24px 120px;
}

.TorrentTitle {
  margin: 0;
  font-family: var(--display, "Space Grotesk", Inter, sans-serif);
  font-size: 34px;
  font-weight: 600;
  line-height: 1.1;
  letter-spacing: -0.035em;
}

.TorrentLead {
  margin: 10px 0 32px;
  font-size: 15px;
  color: var(--ink-mute, #8f8a9b);
}

.TorrentDrop {
  display: flex;
  min-height: 240px;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 6px;
  border-radius: 22px;
  background: rgba(255, 255, 255, 0.025);
  box-shadow: inset 0 0 0 1px var(--edge, rgba(255, 255, 255, 0.08));
  padding: 32px 24px;
  text-align: center;
  cursor: pointer;
  transition: background 220ms ease, box-shadow 220ms ease, transform 320ms var(--ease, cubic-bezier(0.16, 1, 0.3, 1));
}

.TorrentDrop:hover,
.TorrentDrop:focus-visible {
  background: rgba(255, 255, 255, 0.04);
  box-shadow: inset 0 0 0 1px var(--edge-strong, rgba(255, 255, 255, 0.14));
  outline: none;
}

.TorrentDrop.Compact {
  min-height: 0;
  flex-direction: row;
  justify-content: space-between;
  gap: 16px;
  border-radius: 16px;
  padding: 14px 18px;
  text-align: left;
}

.TorrentDrop.Compact .TorrentIcon {
  display: none;
}

.TorrentIcon {
  height: 40px;
  width: 40px;
  margin-bottom: 10px;
  color: var(--ink-mute, #8f8a9b);
  transition: transform 320ms var(--ease, cubic-bezier(0.16, 1, 0.3, 1)), color 200ms ease;
}

.TorrentDrop:hover .TorrentIcon {
  color: var(--ink-soft, #b9b5c6);
  transform: translateY(2px);
}

.TorrentDropTitle {
  margin: 0;
  font-size: 16px;
  font-weight: 500;
}

.TorrentDropHint {
  margin: 0;
  font-size: 13.5px;
  color: var(--ink-mute, #8f8a9b);
}

.TorrentButton {
  display: inline-flex;
  height: 36px;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  border: 0;
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.07);
  box-shadow: inset 0 0 0 1px var(--edge, rgba(255, 255, 255, 0.08));
  padding: 0 14px;
  font: inherit;
  font-size: 13.5px;
  font-weight: 500;
  color: var(--ink, #f4f2f7);
  cursor: pointer;
  transition: background 160ms ease;
}

.TorrentButton:hover {
  background: rgba(255, 255, 255, 0.11);
}

.TorrentButton.Primary {
  background: var(--ink, #f4f2f7);
  box-shadow: none;
  color: #0c0b0f;
}

.TorrentButton.Primary:hover {
  background: #fff;
}

.TorrentError {
  margin: 16px 0 0;
  border-radius: 14px;
  background: rgba(239, 138, 138, 0.06);
  box-shadow: inset 0 0 0 1px rgba(239, 138, 138, 0.16);
  padding: 12px 16px;
  font-size: 14px;
  color: var(--danger, #ef8a8a);
  animation: TorrentIn 320ms var(--ease, cubic-bezier(0.16, 1, 0.3, 1)) both;
}

.TorrentResult > * {
  animation: TorrentIn 460ms var(--ease, cubic-bezier(0.16, 1, 0.3, 1)) both;
}

.TorrentResult > *:nth-child(2) { animation-delay: 40ms; }
.TorrentResult > *:nth-child(3) { animation-delay: 80ms; }
.TorrentResult > *:nth-child(4) { animation-delay: 120ms; }
.TorrentResult > *:nth-child(5) { animation-delay: 160ms; }
.TorrentResult > *:nth-child(n + 6) { animation-delay: 200ms; }

.TorrentHero {
  margin-top: 40px;
}

.TorrentName {
  margin: 0;
  font-family: var(--display, "Space Grotesk", Inter, sans-serif);
  font-size: 28px;
  font-weight: 600;
  line-height: 1.15;
  letter-spacing: -0.03em;
  overflow-wrap: anywhere;
}

.TorrentMeta {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 6px 10px;
  margin: 10px 0 0;
  font-size: 14.5px;
  font-variant-numeric: tabular-nums;
  color: var(--ink-soft, #b9b5c6);
}

.TorrentDot {
  height: 3px;
  width: 3px;
  border-radius: 999px;
  background: var(--ink-faint, #5d5868);
}

.TorrentTag {
  border-radius: 6px;
  box-shadow: inset 0 0 0 1px var(--edge, rgba(255, 255, 255, 0.08));
  padding: 1px 8px 2px;
  font-size: 12px;
  font-weight: 500;
  color: var(--ink-mute, #8f8a9b);
}

.TorrentActions {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  margin-top: 22px;
}

.TorrentSection {
  margin-top: 44px;
}

.TorrentHeading {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 16px;
  margin: 0 0 12px;
  font-family: var(--display, "Space Grotesk", Inter, sans-serif);
  font-size: 19px;
  font-weight: 600;
  letter-spacing: -0.025em;
}

.TorrentCount {
  font-family: Inter, system-ui, sans-serif;
  font-size: 13px;
  font-weight: 400;
  letter-spacing: 0;
  font-variant-numeric: tabular-nums;
  color: var(--ink-mute, #8f8a9b);
}

.TorrentList {
  margin: 0;
  padding: 0;
  border-bottom: 1px solid var(--line, rgba(255, 255, 255, 0.06));
  list-style: none;
}

.TorrentRow {
  display: flex;
  min-height: 48px;
  align-items: center;
  justify-content: space-between;
  gap: 24px;
  border-top: 1px solid var(--line, rgba(255, 255, 255, 0.06));
  padding: 12px 0;
  font-size: 14px;
}

.TorrentLabel {
  flex-shrink: 0;
  color: var(--ink-mute, #8f8a9b);
}

.TorrentValue {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: 10px;
  text-align: right;
  overflow-wrap: anywhere;
  color: var(--ink-soft, #b9b5c6);
}

.TorrentMono {
  font-family: var(--mono, ui-monospace, "SF Mono", Menlo, monospace);
  font-size: 12.5px;
}

.TorrentCopy {
  display: inline-flex;
  height: 28px;
  flex-shrink: 0;
  align-items: center;
  border: 0;
  border-radius: 7px;
  background: transparent;
  padding: 0 8px;
  font: inherit;
  font-size: 12.5px;
  color: var(--ink-mute, #8f8a9b);
  cursor: pointer;
  transition: background 150ms ease, color 150ms ease;
}

.TorrentCopy:hover {
  background: rgba(255, 255, 255, 0.06);
  color: var(--ink, #f4f2f7);
}

.TorrentFile {
  display: flex;
  min-height: 44px;
  align-items: center;
  justify-content: space-between;
  gap: 20px;
  border-top: 1px solid var(--line, rgba(255, 255, 255, 0.06));
  padding: 10px 0;
  font-size: 14px;
}

.TorrentPath {
  min-width: 0;
  overflow-wrap: anywhere;
}

.TorrentFolder {
  color: var(--ink-faint, #5d5868);
}

.TorrentSize {
  flex-shrink: 0;
  font-variant-numeric: tabular-nums;
  color: var(--ink-mute, #8f8a9b);
}

.TorrentMore {
  margin-top: 14px;
}

.TorrentEmpty {
  margin: 0;
  border-top: 1px solid var(--line, rgba(255, 255, 255, 0.06));
  padding: 14px 0;
  font-size: 14px;
  color: var(--ink-mute, #8f8a9b);
}

.TorrentVeil {
  position: fixed;
  inset: 0;
  z-index: 3;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(5, 5, 7, 0.86);
  opacity: 0;
  pointer-events: none;
  transition: opacity 200ms ease;
}

.TorrentVeil.Shown {
  opacity: 1;
}

.TorrentVeilBox {
  display: flex;
  height: calc(100% - 48px);
  width: calc(100% - 48px);
  align-items: center;
  justify-content: center;
  border-radius: 28px;
  box-shadow: inset 0 0 0 1px var(--edge-strong, rgba(255, 255, 255, 0.14));
  font-family: var(--display, "Space Grotesk", Inter, sans-serif);
  font-size: 24px;
  font-weight: 600;
  letter-spacing: -0.02em;
  transform: scale(0.985);
  transition: transform 320ms var(--ease, cubic-bezier(0.16, 1, 0.3, 1));
}

.TorrentVeil.Shown .TorrentVeilBox {
  transform: none;
}

@keyframes TorrentFade {
  from { opacity: 0; }
}

@keyframes TorrentIn {
  from {
    opacity: 0;
    transform: translateY(8px);
  }
}

@media (max-width: 639px) {
  .TorrentMain {
    padding: 36px 16px 96px;
  }

  .TorrentTitle {
    font-size: 28px;
  }

  .TorrentName {
    font-size: 23px;
  }

  .TorrentRow {
    align-items: flex-start;
    flex-direction: column;
    gap: 4px;
  }

  .TorrentValue {
    text-align: left;
  }

  .TorrentDrop.Compact {
    align-items: flex-start;
    flex-direction: column;
  }
}

@media (prefers-reduced-motion: reduce) {
  .TorrentPage,
  .TorrentResult > *,
  .TorrentError {
    animation: none;
  }
}
`

function Fail() {
  throw new Error("To nie jest poprawny plik .torrent.")
}

function Latin(Bytes) {
  let Out = ""
  for (const Byte of Bytes) Out += String.fromCharCode(Byte)
  return Out
}

function Decode(Bytes) {
  let At = 0
  let InfoStart = -1
  let InfoEnd = -1

  function Text() {
    const Colon = Bytes.indexOf(0x3a, At)
    if (Colon < 0) Fail()
    const Raw = Latin(Bytes.subarray(At, Colon))
    if (!/^\d+$/.test(Raw)) Fail()
    const Start = Colon + 1
    const End = Start + Number(Raw)
    if (End > Bytes.length) Fail()
    At = End
    return Bytes.subarray(Start, End)
  }

  function Next(Depth) {
    if (At >= Bytes.length || Depth > 64) Fail()
    const Char = Bytes[At]

    if (Char === 0x69) {
      const End = Bytes.indexOf(0x65, At + 1)
      if (End < 0) Fail()
      const Raw = Latin(Bytes.subarray(At + 1, End))
      if (!/^-?\d+$/.test(Raw)) Fail()
      At = End + 1
      return Number(Raw)
    }

    if (Char === 0x6c) {
      At += 1
      const List = []
      while (Bytes[At] !== 0x65) {
        if (At >= Bytes.length) Fail()
        List.push(Next(Depth + 1))
      }
      At += 1
      return List
    }

    if (Char === 0x64) {
      At += 1
      const Dict = new Map()
      while (Bytes[At] !== 0x65) {
        if (At >= Bytes.length) Fail()
        const Key = Utf8.decode(Text())
        const Start = At
        const Item = Next(Depth + 1)
        if (Depth === 0 && Key === "info") {
          InfoStart = Start
          InfoEnd = At
        }
        Dict.set(Key, Item)
      }
      At += 1
      return Dict
    }

    if (Char >= 0x30 && Char <= 0x39) return Text()
    Fail()
  }

  const Root = Next(0)
  if (!(Root instanceof Map) || InfoStart < 0) Fail()
  return { Root, Info: Bytes.subarray(InfoStart, InfoEnd) }
}

function Sha1(Bytes) {
  const Length = Bytes.length
  const Words = new Uint32Array((((Length + 8) >> 6) + 1) * 16)
  for (let I = 0; I < Length; I++) Words[I >> 2] |= Bytes[I] << (24 - (I & 3) * 8)
  Words[Length >> 2] |= 0x80 << (24 - (Length & 3) * 8)
  Words[Words.length - 2] = Math.floor(Length / 0x20000000)
  Words[Words.length - 1] = (Length * 8) >>> 0

  const State = [0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476, 0xc3d2e1f0]
  const W = new Uint32Array(80)

  for (let Block = 0; Block < Words.length; Block += 16) {
    for (let T = 0; T < 16; T++) W[T] = Words[Block + T]
    for (let T = 16; T < 80; T++) {
      const X = W[T - 3] ^ W[T - 8] ^ W[T - 14] ^ W[T - 16]
      W[T] = (X << 1) | (X >>> 31)
    }

    let [A, B, C, D, E] = State
    for (let T = 0; T < 80; T++) {
      const F = T < 20 ? (B & C) | (~B & D) : T < 40 ? B ^ C ^ D : T < 60 ? (B & C) | (B & D) | (C & D) : B ^ C ^ D
      const K = T < 20 ? 0x5a827999 : T < 40 ? 0x6ed9eba1 : T < 60 ? 0x8f1bbcdc : 0xca62c1d6
      const Temp = (((A << 5) | (A >>> 27)) + F + E + K + W[T]) >>> 0
      E = D
      D = C
      C = ((B << 30) | (B >>> 2)) >>> 0
      B = A
      A = Temp
    }

    State[0] = (State[0] + A) >>> 0
    State[1] = (State[1] + B) >>> 0
    State[2] = (State[2] + C) >>> 0
    State[3] = (State[3] + D) >>> 0
    State[4] = (State[4] + E) >>> 0
  }

  return State.map(V => V.toString(16).padStart(8, "0")).join("")
}

function Str(Dict, Key) {
  const Value = Dict.get(Key)
  if (!(Value instanceof Uint8Array)) return null
  const Text = Utf8.decode(Value).trim()
  return Text || null
}

function Num(Dict, Key) {
  const Value = Dict.get(Key)
  return typeof Value === "number" ? Value : null
}

function Parts(List) {
  return Array.isArray(List) ? List.filter(P => P instanceof Uint8Array).map(P => Utf8.decode(P)) : []
}

function Tree(Node, Prefix, Out) {
  for (const [Key, Child] of Node) {
    if (!(Child instanceof Map)) continue
    const Leaf = Child.get("")
    if (Key === "" || !(Leaf instanceof Map)) {
      if (Key !== "") Tree(Child, [...Prefix, Key], Out)
      continue
    }
    Out.push({ Path: [...Prefix, Key], Size: Num(Leaf, "length") ?? 0 })
  }
}

function Describe(Bytes) {
  const { Root, Info: InfoBytes } = Decode(Bytes)
  const Info = Root.get("info")
  if (!(Info instanceof Map)) Fail()

  const Name = Str(Info, "name.utf-8") ?? Str(Info, "name") ?? "Bez nazwy"
  const Pieces = Info.get("pieces")
  const HasV1 = Pieces instanceof Uint8Array
  const HasV2 = Num(Info, "meta version") === 2

  let Files = []
  const Listed = Info.get("files")
  if (Array.isArray(Listed)) {
    for (const File of Listed) {
      if (!(File instanceof Map)) continue
      const Attr = Str(File, "attr") ?? ""
      if (Attr.includes("p")) continue
      const Path = Parts(File.get("path.utf-8")).length ? Parts(File.get("path.utf-8")) : Parts(File.get("path"))
      Files.push({ Path: [Name, ...Path], Size: Num(File, "length") ?? 0 })
    }
  } else if (Num(Info, "length") !== null) {
    Files = [{ Path: [Name], Size: Num(Info, "length") }]
  } else if (Info.get("file tree") instanceof Map) {
    Tree(Info.get("file tree"), [Name], Files)
  }

  const Trackers = []
  const Announce = Str(Root, "announce")
  if (Announce) Trackers.push(Announce)
  const Tiers = Root.get("announce-list")
  if (Array.isArray(Tiers)) {
    for (const Tier of Tiers) {
      for (const Url of Parts(Tier)) if (Url && !Trackers.includes(Url)) Trackers.push(Url)
    }
  }

  const Seeds = []
  const UrlList = Root.get("url-list")
  if (UrlList instanceof Uint8Array) Seeds.push(Utf8.decode(UrlList))
  else Seeds.push(...Parts(UrlList))

  const Hash = HasV1 ? Sha1(InfoBytes) : null
  const Magnet = Hash
    ? `magnet:?xt=urn:btih:${Hash}&dn=${encodeURIComponent(Name)}${Trackers.map(Url => `&tr=${encodeURIComponent(Url)}`).join("")}`
    : null

  const Created = Num(Root, "creation date")

  return {
    Name,
    Files,
    Total: Files.reduce((Sum, File) => Sum + File.Size, 0),
    PieceLength: Num(Info, "piece length"),
    PieceCount: HasV1 ? Math.floor(Pieces.length / 20) : null,
    Private: Num(Info, "private") === 1,
    Version: HasV1 && HasV2 ? "hybryda v1 + v2" : HasV2 ? "v2" : "v1",
    Hash,
    Magnet,
    Trackers,
    Seeds: Seeds.filter(Boolean),
    Created: Created ? new Date(Created * 1000) : null,
    CreatedBy: Str(Root, "created by"),
    Comment: Str(Root, "comment.utf-8") ?? Str(Root, "comment"),
    Source: Str(Info, "source"),
  }
}

function Size(Bytes) {
  let Value = Bytes
  let Index = 0
  while (Value >= 1024 && Index < Units.length - 1) {
    Value /= 1024
    Index += 1
  }
  const Digits = Index === 0 || Value >= 100 ? 0 : 1
  return `${Value.toLocaleString("pl", { maximumFractionDigits: Digits })} ${Units[Index]}`
}

function Plural(Count, One, Few, Many) {
  if (Count === 1) return One
  const Tens = Count % 100
  const Ones = Count % 10
  return Ones >= 2 && Ones <= 4 && (Tens < 12 || Tens > 14) ? Few : Many
}

function Make(Tag, Class, Content) {
  const El = document.createElement(Tag)
  if (Class) El.className = Class
  if (typeof Content === "string") El.textContent = Content
  else if (Array.isArray(Content)) El.append(...Content.filter(Boolean))
  return El
}

async function Copy(Text) {
  try {
    await navigator.clipboard.writeText(Text)
    return
  } catch {}
  const Area = Make("textarea", "", Text)
  Area.style.cssText = "position:fixed;opacity:0;pointer-events:none"
  document.body.append(Area)
  Area.select()
  document.execCommand("copy")
  Area.remove()
}

function CopyButton(Label, Text, Class) {
  const Button = Make("button", Class, Label)
  Button.type = "button"
  let Timer = null
  Button.addEventListener("click", async () => {
    await Copy(Text)
    Button.textContent = "Skopiowano"
    clearTimeout(Timer)
    Timer = setTimeout(() => { Button.textContent = Label }, 1600)
  })
  return Button
}

function Fact(Label, Value, Extra) {
  if (Value === null || Value === undefined || Value === "") return null
  return Make("li", "TorrentRow", [
    Make("span", "TorrentLabel", Label),
    Make("span", "TorrentValue", [typeof Value === "string" ? Make("span", "", Value) : Value, Extra]),
  ])
}

function FileRow(File) {
  const Folder = File.Path.slice(0, -1).join("/")
  return Make("li", "TorrentFile", [
    Make("span", "TorrentPath", [
      Folder ? Make("span", "TorrentFolder", `${Folder}/`) : null,
      Make("span", "", File.Path[File.Path.length - 1]),
    ]),
    Make("span", "TorrentSize", Size(File.Size)),
  ])
}

function Render(Data, OnPick) {
  const Result = Make("section", "TorrentResult")

  const Meta = Make("p", "TorrentMeta", [
    Make("span", "", Size(Data.Total)),
    Make("span", "TorrentDot"),
    Make("span", "", `${Data.Files.length.toLocaleString("pl")} ${Plural(Data.Files.length, "plik", "pliki", "plików")}`),
    Data.Private ? Make("span", "TorrentTag", "prywatny") : null,
    Make("span", "TorrentTag", Data.Version),
  ])
  Result.append(Make("div", "TorrentHero", [Make("h2", "TorrentName", Data.Name), Meta]))

  const Actions = Make("div", "TorrentActions", [
    Data.Magnet ? CopyButton("Kopiuj link magnet", Data.Magnet, "TorrentButton Primary") : null,
  ])
  const Another = Make("button", "TorrentButton", "Wczytaj inny plik")
  Another.type = "button"
  Another.addEventListener("click", OnPick)
  Actions.append(Another)
  Result.append(Actions)

  const Facts = Make("ul", "TorrentList", [
    Data.Hash ? Fact("Hash info", Make("span", "TorrentMono", Data.Hash), CopyButton("Kopiuj", Data.Hash, "TorrentCopy")) : null,
    Fact("Rozmiar części", Data.PieceLength ? Size(Data.PieceLength) : null),
    Fact("Liczba części", Data.PieceCount !== null ? Data.PieceCount.toLocaleString("pl") : null),
    Fact("Utworzono", Data.Created ? Data.Created.toLocaleString("pl", { dateStyle: "long", timeStyle: "short" }) : null),
    Fact("Program", Data.CreatedBy),
    Fact("Komentarz", Data.Comment),
    Fact("Źródło", Data.Source),
  ])
  Result.append(Make("div", "TorrentSection", [Make("h3", "TorrentHeading", "Szczegóły"), Facts]))

  const FileList = Make("ul", "TorrentList", Data.Files.slice(0, FirstFiles).map(FileRow))
  const FileSection = Make("div", "TorrentSection", [
    Make("h3", "TorrentHeading", [
      Make("span", "", "Pliki"),
      Make("span", "TorrentCount", Data.Files.length.toLocaleString("pl")),
    ]),
    Data.Files.length ? FileList : Make("p", "TorrentEmpty", "Brak plików w opisie torrenta."),
  ])
  if (Data.Files.length > FirstFiles) {
    const More = Make("button", "TorrentButton TorrentMore", `Pokaż wszystkie (${Data.Files.length.toLocaleString("pl")})`)
    More.type = "button"
    More.addEventListener("click", () => {
      FileList.append(...Data.Files.slice(FirstFiles).map(FileRow))
      More.remove()
    })
    FileSection.append(More)
  }
  Result.append(FileSection)

  Result.append(Make("div", "TorrentSection", [
    Make("h3", "TorrentHeading", [
      Make("span", "", "Trackery"),
      Make("span", "TorrentCount", Data.Trackers.length.toLocaleString("pl")),
    ]),
    Data.Trackers.length
      ? Make("ul", "TorrentList", Data.Trackers.map(Url => Make("li", "TorrentRow", [Make("span", "TorrentValue TorrentMono", Url)])))
      : Make("p", "TorrentEmpty", "Brak trackerów. Ten torrent korzysta tylko z DHT i wymiany peerów."),
  ]))

  if (Data.Seeds.length) {
    Result.append(Make("div", "TorrentSection", [
      Make("h3", "TorrentHeading", "Serwery HTTP"),
      Make("ul", "TorrentList", Data.Seeds.map(Url => Make("li", "TorrentRow", [Make("span", "TorrentValue TorrentMono", Url)]))),
    ]))
  }

  return Result
}

function Build() {
  const Page = Make("div", "TorrentPage")
  Page.addEventListener("keydown", (Event) => Event.stopPropagation())

  const Back = Make("a", "TorrentBack")
  Back.href = "/"
  Back.innerHTML = '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M15 18 9 12l6-6"/></svg>'
  Back.append("Panel")

  const Main = Make("main", "TorrentMain", [
    Make("h1", "TorrentTitle", "Torrent"),
    Make("p", "TorrentLead", "Upuść plik .torrent, żeby zobaczyć, co jest w środku. Plik nie opuszcza tej przeglądarki."),
  ])

  const Input = Make("input")
  Input.type = "file"
  Input.accept = ".torrent,application/x-bittorrent"
  Input.hidden = true

  const Drop = Make("div", "TorrentDrop")
  Drop.tabIndex = 0
  Drop.setAttribute("role", "button")
  Drop.innerHTML = '<svg class="TorrentIcon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M12 3v11"/><path d="m7.5 9.5 4.5 4.5 4.5-4.5"/><path d="M4 15v3.5A2.5 2.5 0 0 0 6.5 21h11a2.5 2.5 0 0 0 2.5-2.5V15"/></svg>'
  const DropText = Make("div", "", [
    Make("p", "TorrentDropTitle", "Upuść plik .torrent"),
    Make("p", "TorrentDropHint", "albo kliknij, żeby wybrać go z dysku"),
  ])
  Drop.append(DropText)

  const Veil = Make("div", "TorrentVeil", [Make("div", "TorrentVeilBox", "Upuść, aby wczytać")])
  Veil.setAttribute("aria-hidden", "true")

  let Current = null
  let Problem = null

  function Pick() {
    Input.value = ""
    Input.click()
  }

  async function Open(File) {
    Problem?.remove()
    Problem = null
    try {
      if (File.size > MaxBytes) throw new Error("Ten plik jest za duży jak na plik .torrent.")
      const Data = Describe(new Uint8Array(await File.arrayBuffer()))
      const Next = Render(Data, Pick)
      if (Current) Current.replaceWith(Next)
      else Main.append(Next)
      Current = Next
      Drop.classList.add("Compact")
      DropText.firstChild.textContent = File.name
      DropText.lastChild.textContent = "Upuść inny plik albo kliknij, żeby wybrać"
      Page.scrollTo({ top: 0, behavior: "smooth" })
    } catch (Caught) {
      Problem = Make("p", "TorrentError", Caught instanceof RangeError ? "Plik jest uszkodzony." : Caught.message)
      Drop.after(Problem)
    }
  }

  Drop.addEventListener("click", Pick)
  Drop.addEventListener("keydown", (Event) => {
    if (Event.key !== "Enter" && Event.key !== " ") return
    Event.preventDefault()
    Pick()
  })
  Input.addEventListener("change", () => {
    if (Input.files?.[0]) void Open(Input.files[0])
  })

  let Depth = 0
  const HasFiles = (Event) => Array.from(Event.dataTransfer?.types ?? []).includes("Files")
  const OnEnter = (Event) => {
    if (!HasFiles(Event)) return
    Event.preventDefault()
    Depth += 1
    Veil.classList.add("Shown")
  }
  const OnOver = (Event) => {
    if (!HasFiles(Event)) return
    Event.preventDefault()
    Event.dataTransfer.dropEffect = "copy"
  }
  const OnLeave = (Event) => {
    if (!HasFiles(Event)) return
    Depth = Math.max(0, Depth - 1)
    if (!Depth) Veil.classList.remove("Shown")
  }
  const OnDrop = (Event) => {
    if (!HasFiles(Event)) return
    Event.preventDefault()
    Depth = 0
    Veil.classList.remove("Shown")
    const File = Event.dataTransfer.files?.[0]
    if (File) void Open(File)
  }

  Main.append(Drop, Input)
  Page.append(Make("header", "TorrentTop", [Back, Make("span", "TorrentBrand", "Torrent")]), Main, Veil)

  const Events = [["dragenter", OnEnter], ["dragover", OnOver], ["dragleave", OnLeave], ["drop", OnDrop]]
  for (const [Name, Handler] of Events) window.addEventListener(Name, Handler)

  return {
    Page,
    Dispose() {
      for (const [Name, Handler] of Events) window.removeEventListener(Name, Handler)
      Page.remove()
    },
  }
}

export function activate() {
  if (location.pathname.replace(/\/+$/, "") !== Route) return () => {}

  const Sheet = Make("style", "", Style)
  document.head.append(Sheet)

  const App = document.getElementById("app")
  App?.setAttribute("inert", "")

  const View = Build()
  document.body.append(View.Page)

  const Guard = (Event) => {
    if (!View.Page.contains(Event.target)) Event.stopPropagation()
  }
  window.addEventListener("keydown", Guard, true)

  const Title = document.title
  document.title = "Torrent · zstio-tv"

  return () => {
    View.Dispose()
    Sheet.remove()
    App?.removeAttribute("inert")
    window.removeEventListener("keydown", Guard, true)
    document.title = Title
  }
}
