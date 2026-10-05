import { Fetch, PostJson } from "./http"

export interface SerialStatus {
  Port: string
  Baud: number
  Ports: string[]
  PublicKey: string
  Fingerprint: string
  PeerKey: string
  Open: boolean
  Error: string | null
}

export interface SaveResult {
  Ok: boolean
  Error?: string
}

export async function LoadStatus(): Promise<SerialStatus> {
  const Res = await Fetch("/api/serial")
  if (!Res.ok) throw new Error(`${Res.status}`)
  return Res.json()
}

export async function SaveConfig(Port: string, Baud: number, PeerKey: string): Promise<SaveResult> {
  const Res = await PostJson("/api/serial", { Port, Baud, PeerKey })
  if (!Res.ok) throw new Error(`${Res.status}`)
  return Res.json()
}

export async function RegenerateKeys(): Promise<SerialStatus> {
  const Res = await Fetch("/api/serial/regenerate", { method: "POST" })
  if (!Res.ok) throw new Error(`${Res.status}`)
  return Res.json()
}
