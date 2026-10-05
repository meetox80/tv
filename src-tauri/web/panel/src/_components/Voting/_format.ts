import type { SearchSource } from "../../_api/voting"

export interface RowAction {
  Key: string
  Label: string
  Tone?: "primary" | "danger"
}

const _Relative = new Intl.RelativeTimeFormat("pl", { numeric: "auto" })
const _Short = new Intl.DateTimeFormat("pl", { day: "numeric", month: "short" })
const _Long = new Intl.DateTimeFormat("pl", { day: "numeric", month: "long", year: "numeric" })

const _Steps: [Intl.RelativeTimeFormatUnit, number][] = [
  ["second", 60],
  ["minute", 60],
  ["hour", 24],
  ["day", 7],
]

export function Relative(Seconds: number): string {
  let Delta = Seconds - Date.now() / 1000

  if (Math.abs(Delta) < 45) return "przed chwilą"

  for (const [Unit, Size] of _Steps) {
    if (Math.abs(Delta) < Size) return _Relative.format(Math.round(Delta), Unit)
    Delta /= Size
  }

  return _Short.format(new Date(Seconds * 1000))
}

export function Day(Seconds: number): string {
  return _Long.format(new Date(Seconds * 1000))
}

export function Duration(Ms: number | null): string {
  if (!Ms || Ms <= 0) return ""
  const Total = Math.round(Ms / 1000)
  return `${Math.floor(Total / 60)}:${String(Total % 60).padStart(2, "0")}`
}

export function ProviderName(Provider: SearchSource): string {
  if (Provider === "spotify") return "Spotify"
  if (Provider === "deezer") return "Deezer"
  return "Last.fm"
}

export function Line(Song: { Artist: string; Title: string }): string {
  return `${Song.Artist} - ${Song.Title}`
}

export function Plural(Count: number): "one" | "few" | "many" {
  if (Count === 1) return "one"
  const Tens = Count % 100
  const Units = Count % 10
  if (Units >= 2 && Units <= 4 && (Tens < 12 || Tens > 14)) return "few"
  return "many"
}
