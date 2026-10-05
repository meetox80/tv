const _TokenKey: string = "setup_token"

export function GetToken(): string | null {
  try {
    return localStorage.getItem(_TokenKey)
  } catch (_) {
    return null
  }
}

export function SetToken(Token: string): void {
  try {
    localStorage.setItem(_TokenKey, Token)
  } catch (_) {}
}

export function ClearToken(): void {
  try {
    localStorage.removeItem(_TokenKey)
  } catch (_) {}
}

export async function Authed(Url: string, Init: RequestInit = {}): Promise<Response> {
  const Token = GetToken()
  const Head = new Headers(Init.headers ?? {})
  if (Token) Head.set("Authorization", `Bearer ${Token}`)

  return fetch(Url, { ...Init, headers: Head })
}
