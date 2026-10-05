import { Fetch, PostJson } from "./http"

export type HostKind = "management" | "voting"

export interface Tunnel {
  Management: string
  Voting: string
  Local: boolean
}

export interface TunnelResult {
  Ok: boolean
  Error?: string
  Hostname?: string
}

export async function Load(): Promise<Tunnel> {
  const Res = await Fetch("/api/tunnel")
  if (!Res.ok) throw new Error(`${Res.status}`)
  return Res.json()
}

export async function Save(Kind: HostKind, Hostname: string, Force = false): Promise<TunnelResult> {
  const Res = await PostJson("/api/tunnel", { Kind, Hostname, Force })
  if (Res.status === 403) return { Ok: false, Error: "remote" }
  if (!Res.ok) return { Ok: false, Error: "unknown" }
  return Res.json()
}
