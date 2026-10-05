import { Fetch, PostJson } from "./http"

export type SongStatus = "pending" | "approved" | "added" | "rejected"
export type SearchSource = "spotify" | "lastfm" | "deezer"

export interface Counts {
  Pending: number
  Approved: number
  Added: number
  Rejected: number
}

export interface VotingConfig {
  Enabled: boolean
  SiteKey: string
  HasSecret: boolean
  Url: string | null
  Source: SearchSource
  Counts: Counts
  Banned: number
}

export interface AdminSong {
  Id: number
  Provider: SearchSource
  Title: string
  Artist: string
  Album: string | null
  Cover: string | null
  Explicit: boolean | null
  DurationMs: number | null
  Link: string | null
  Status: SongStatus
  Score: number
  Up: number
  Down: number
  Device: string | null
  DeviceBanned: boolean
  Created: number
  Decided: number | null
}

export interface BannedDevice {
  Handle: string
  Reason: string | null
  BannedAt: number
  Created: number
  Seen: number
  Requests: number
}

export class ApiError extends Error {
  Code: string
  Status: number

  constructor(Status: number, Code: string) {
    super(Code)
    this.Status = Status
    this.Code = Code
  }
}

async function _Fail(Res: Response): Promise<never> {
  let Code = `http.${Res.status}`
  try {
    const Body = await Res.json()
    if (typeof Body?.Error === "string") Code = Body.Error
  } catch {}
  throw new ApiError(Res.status, Code)
}

export async function Load(): Promise<VotingConfig> {
  const Res = await Fetch("/api/voting")
  if (!Res.ok) return _Fail(Res)
  return Res.json()
}

export async function SetEnabled(Enabled: boolean): Promise<void> {
  const Res = await PostJson("/api/voting/enabled", { Enabled })
  if (!Res.ok) await _Fail(Res)
}

export async function SaveTurnstile(SiteKey: string, Secret: string): Promise<void> {
  const Res = await PostJson("/api/voting/turnstile", { SiteKey, Secret })
  if (!Res.ok) await _Fail(Res)
}

export type CaptchaStatus = "unconfigured" | "valid" | "invalid" | "unreachable"

export async function CheckTurnstile(): Promise<CaptchaStatus> {
  const Res = await Fetch("/api/voting/turnstile")
  if (!Res.ok) return _Fail(Res)
  const Body = await Res.json()
  return Body.Status
}

export async function ClearTurnstile(): Promise<void> {
  const Res = await Fetch("/api/voting/turnstile", { method: "DELETE" })
  if (!Res.ok) await _Fail(Res)
}

export async function LoadSongs(Status: SongStatus): Promise<AdminSong[]> {
  const Res = await Fetch(`/api/voting/songs?status=${encodeURIComponent(Status)}`)
  if (!Res.ok) return _Fail(Res)
  return Res.json()
}

export async function SetStatus(Id: number, Status: SongStatus): Promise<AdminSong> {
  const Res = await PostJson(`/api/voting/songs/${Id}/status`, { Status })
  if (!Res.ok) return _Fail(Res)
  return Res.json()
}

export async function PreviewClip(Id: number): Promise<Blob | null> {
  const Res = await Fetch(`/api/voting/songs/${Id}/preview`)
  if (Res.status === 404) return null
  if (!Res.ok) return _Fail(Res)
  return Res.blob()
}

export async function Remove(Id: number): Promise<void> {
  const Res = await Fetch(`/api/voting/songs/${Id}`, { method: "DELETE" })
  if (!Res.ok) await _Fail(Res)
}

export async function LoadBans(): Promise<BannedDevice[]> {
  const Res = await Fetch("/api/voting/devices")
  if (!Res.ok) return _Fail(Res)
  return Res.json()
}

export async function SetBan(Handle: string, Banned: boolean, Reason?: string): Promise<void> {
  const Body: { Handle: string; Banned: boolean; Reason?: string } = { Handle, Banned }
  if (Reason && Reason.trim()) Body.Reason = Reason.trim()
  const Res = await PostJson("/api/voting/devices", Body)
  if (!Res.ok) await _Fail(Res)
}
