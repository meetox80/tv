import * as Session from "./session"

export class Unauthorized extends Error {}

export async function Fetch(Path: string, Init: RequestInit = {}): Promise<Response> {
  const Token = Session.Token()
  const Merged = new Headers(Init.headers)
  if (Token) Merged.set("Authorization", `Bearer ${Token}`)

  const Res = await fetch(Path, { ...Init, headers: Merged })

  if (Res.status === 401) {
    Session.Clear()
    globalThis.dispatchEvent(new Event("panel-unauthorized"))
    throw new Unauthorized()
  }

  return Res
}

export function PostJson(Path: string, Body: unknown): Promise<Response> {
  return Fetch(Path, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(Body),
  })
}
