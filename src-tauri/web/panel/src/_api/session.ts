const _StorageKey = "panel_session"
const _LegacyKey = "token"

export const SessionTtlMs = 15 * 60 * 1000

interface StoredSession {
  Token: string
  ExpiresAt: number
}

function _Read(): StoredSession | null {
  let Raw: string | null = null

  try {
    Raw = sessionStorage.getItem(_StorageKey)
  } catch {
    return null
  }

  if (!Raw) return null

  try {
    const Parsed = JSON.parse(Raw) as Partial<StoredSession>
    if (typeof Parsed.Token !== "string" || typeof Parsed.ExpiresAt !== "number") {
      Clear()
      return null
    }
    if (Date.now() >= Parsed.ExpiresAt) {
      Clear()
      return null
    }
    return { Token: Parsed.Token, ExpiresAt: Parsed.ExpiresAt }
  } catch {
    Clear()
    return null
  }
}

export function Save(Token: string): void {
  const Payload: StoredSession = { Token, ExpiresAt: Date.now() + SessionTtlMs }

  try {
    sessionStorage.setItem(_StorageKey, JSON.stringify(Payload))
    sessionStorage.removeItem(_LegacyKey)
  } catch {}
}

export function Clear(): void {
  try {
    sessionStorage.removeItem(_StorageKey)
    sessionStorage.removeItem(_LegacyKey)
  } catch {}
}

export function Token(): string | null {
  return _Read()?.Token ?? null
}

export function RemainingMs(): number {
  const Stored = _Read()
  if (!Stored) return 0
  return Math.max(Stored.ExpiresAt - Date.now(), 0)
}
