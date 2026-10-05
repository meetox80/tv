import { Fetch, PostJson } from "./http"

export type ScrobbleMethod = "lastfm" | "spotify" | "serial" | "api"

export interface Settings {
  ScrobbleMethod: ScrobbleMethod | null
  ScrobbleInLessons: boolean
  LastfmApiKey: string
  LastfmUsername: string
  LastfmError: string | null
  LastfmVerified: boolean
  ApiKey: string
  System?: SystemInfo
}

export interface SystemInfo {
  Version: string
  Address: string | null
  StartedAt: number
  DatabaseBytes: number
}

export interface SaveResult {
  Ok: boolean
  Error?: string
}

export async function Load(): Promise<Settings> {
  const Res = await Fetch("/api/settings")
  if (!Res.ok) throw new Error(`${Res.status}`)
  return Res.json()
}

export function SetMethod(Method: ScrobbleMethod) {
  return PostJson("/api/settings/scrobble-method", { Method })
}

export function SetScrobbleInLessons(Enabled: boolean) {
  return PostJson("/api/settings/scrobble-in-lessons", { Enabled })
}

export async function SaveLastfm(ApiKey: string, Username: string): Promise<SaveResult> {
  const Res = await PostJson("/api/settings/lastfm", { ApiKey, Username })
  if (!Res.ok) throw new Error(`${Res.status}`)
  return Res.json()
}

export async function RegenerateKey(): Promise<string> {
  const Res = await Fetch("/api/settings/regenerate-key", { method: "POST" })
  if (!Res.ok) throw new Error(`${Res.status}`)
  const Data = await Res.json()
  return Data.ApiKey
}

export function NukeDatabase() {
  return Fetch("/api/settings/nuke", { method: "POST" })
}

export async function ChangePassword(Current: string, Next: string): Promise<SaveResult> {
  const Res = await PostJson("/api/settings/password", { Current, Next })
  if (Res.ok) return { Ok: true }
  if (Res.status === 400 || Res.status === 403) {
    const Data = await Res.json().catch(() => ({}))
    return { Ok: false, Error: Data.Error ?? "unknown" }
  }
  throw new Error(`${Res.status}`)
}
