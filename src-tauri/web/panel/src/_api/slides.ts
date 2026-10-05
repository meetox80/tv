import { Fetch, PostJson } from "./http"
import type { Collection, CollectionSummary, Slide } from "../_components/Slides/model"

async function _Fail(Res: Response): Promise<never> {
  let Message = `${Res.status}`
  try {
    const Body = await Res.json()
    if (typeof Body?.Error === "string") Message = Body.Error
  } catch {}
  throw new Error(Message)
}

export async function List(): Promise<CollectionSummary[]> {
  const Res = await Fetch("/api/slides")
  if (!Res.ok) return _Fail(Res)
  return Res.json()
}

export async function Load(Id: string): Promise<Collection> {
  const Res = await Fetch(`/api/slides/${encodeURIComponent(Id)}`)
  if (!Res.ok) return _Fail(Res)
  return Res.json()
}

export async function Create(Name: string): Promise<Collection> {
  const Res = await PostJson("/api/slides", { Name })
  if (!Res.ok) return _Fail(Res)
  return Res.json()
}

export async function Save(Id: string, Name: string, Slides: Slide[]): Promise<number> {
  const Res = await Fetch(`/api/slides/${encodeURIComponent(Id)}`, {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ Name, Slides }),
  })
  if (!Res.ok) return _Fail(Res)
  return (await Res.json()).Updated
}

export async function Remove(Id: string): Promise<void> {
  const Res = await Fetch(`/api/slides/${encodeURIComponent(Id)}`, { method: "DELETE" })
  if (!Res.ok) return _Fail(Res)
}

export async function Upload(File: Blob): Promise<string> {
  const Res = await Fetch("/api/slides/assets", {
    method: "POST",
    headers: { "Content-Type": File.type || "application/octet-stream" },
    body: File,
  })
  if (!Res.ok) return _Fail(Res)
  return (await Res.json()).Id
}
