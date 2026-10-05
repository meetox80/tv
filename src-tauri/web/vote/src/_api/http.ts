export class ApiError extends Error {
  readonly Status: number
  readonly Code: string
  readonly Body: Record<string, unknown>

  constructor(Status: number, Code: string, Body: Record<string, unknown>) {
    super(Code)
    this.Status = Status
    this.Code = Code
    this.Body = Body
  }

  get RetryAfter(): number | null {
    const Value = this.Body.RetryAfter
    return typeof Value === "number" && Value > 0 ? Math.ceil(Value) : null
  }
}

const _Headers: Record<string, string> = {
  "Content-Type": "application/json",
  "X-Requested-With": "zstio-vote",
}

async function _Read(Res: Response): Promise<Record<string, unknown>> {
  try {
    const Parsed: unknown = await Res.json()
    return Parsed && typeof Parsed === "object" ? (Parsed as Record<string, unknown>) : {}
  } catch {
    return {}
  }
}

function _Fallback(Status: number): string {
  if (Status === 429) return "slow_down"
  if (Status === 503) return "closed"
  if (Status === 403) return "banned"
  if (Status === 401) return "device"
  if (Status === 404) return "not_found"
  if (Status >= 500) return "server"
  return "unknown"
}

async function _Send<T>(Path: string, Init: RequestInit): Promise<T> {
  let Res: Response

  try {
    Res = await fetch(Path, { credentials: "same-origin", cache: "no-store", ...Init })
  } catch (Err) {
    if (Err instanceof DOMException && Err.name === "AbortError") throw Err
    throw new ApiError(0, "network", {})
  }

  const Body = await _Read(Res)

  if (!Res.ok) {
    const Code = typeof Body.Error === "string" ? Body.Error : _Fallback(Res.status)
    throw new ApiError(Res.status, Code, Body)
  }

  return Body as T
}

export function Get<T>(Path: string, Signal?: AbortSignal): Promise<T> {
  return _Send<T>(Path, { method: "GET", signal: Signal })
}

export function Post<T>(Path: string, Payload: unknown): Promise<T> {
  return _Send<T>(Path, { method: "POST", headers: _Headers, body: JSON.stringify(Payload) })
}
