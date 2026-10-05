import { Fetch, PostJson } from "./http"

export interface ExtensionInfo {
  Id: string
  Name: string
  Version: string
  Description: string | null
  Enabled: boolean
  Panel: boolean
  Server: boolean
  Error: string | null
}

export interface ExtensionList {
  Folder: string
  Items: ExtensionInfo[]
}

export async function List(): Promise<ExtensionList> {
  const Res = await Fetch("/api/extensions")
  if (!Res.ok) throw new Error(`${Res.status}`)
  return Res.json()
}

export async function Reload(): Promise<ExtensionList> {
  const Res = await PostJson("/api/extensions/reload", {})
  if (!Res.ok) throw new Error(`${Res.status}`)
  return Res.json()
}

export async function SetEnabled(Id: string, Enabled: boolean): Promise<void> {
  const Res = await PostJson(`/api/extensions/${encodeURIComponent(Id)}`, { Enabled })
  if (!Res.ok) throw new Error(`${Res.status}`)
}

export async function PanelModule(Id: string): Promise<string> {
  const Res = await Fetch(`/api/extensions/${encodeURIComponent(Id)}/panel.js`)
  if (!Res.ok) throw new Error(`${Res.status}`)
  return Res.text()
}
