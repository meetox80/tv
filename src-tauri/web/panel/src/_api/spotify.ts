import { Fetch, PostJson } from "./http"

export interface SpotifyStatus {
  ClientId: string
  HasSecret: boolean
  Connected: boolean
  RedirectUri: string
}

export async function LoadStatus(): Promise<SpotifyStatus> {
  const Res = await Fetch("/api/spotify/status")
  if (!Res.ok) throw new Error(`${Res.status}`)
  return Res.json()
}

export async function SaveCredentials(ClientId: string, ClientSecret: string): Promise<void> {
  const Res = await PostJson("/api/spotify/credentials", { ClientId, ClientSecret })
  if (!Res.ok) throw new Error(`${Res.status}`)
}

export function Disconnect() {
  return Fetch("/api/spotify/disconnect", { method: "POST" })
}

export async function BeginAuthorize(): Promise<void> {
  const Res = await Fetch("/api/spotify/authorize", { method: "POST" })
  if (!Res.ok) throw new Error(`${Res.status}`)

  const Data: { Url: string } = await Res.json()
  globalThis.location.href = Data.Url
}
