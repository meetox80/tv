import * as Session from "./session"

export interface NetworkStats {
  Up: number[]
  Down: number[]
  Sent: number
  Received: number
  Requests: number
}

export interface VotingStats {
  Searches: number
  Failures: number
  Votes: number
  Requests: number
  Limited: number
  Average: number | null
  Slow: number | null
  Active: number
  Devices: number
}

export interface Performance {
  Supported: boolean
  Cpu: number[]
  Rss: number[]
  Fps: number | null
  Low: number | null
  History: number
  Network?: NetworkStats
  Voting?: VotingStats
}

const _RetryMs = 2000
const _MaxRetryMs = 15000

export interface Stream {
  Close: () => void
}

function _Url(): string {
  const Scheme = globalThis.location.protocol === "https:" ? "wss:" : "ws:"
  return `${Scheme}//${globalThis.location.host}/api/performance/ws`
}

export function Open(OnData: (Value: Performance) => void): Stream {
  let Socket: WebSocket | null = null
  let Timer: ReturnType<typeof setTimeout> | null = null
  let Backoff = _RetryMs
  let Closed = false

  function _Retry(): void {
    if (Closed || Timer !== null) return

    Timer = setTimeout(() => {
      Timer = null
      _Connect()
    }, Backoff)

    Backoff = Math.min(Backoff * 2, _MaxRetryMs)
  }

  function _Connect(): void {
    if (Closed) return

    const Token = Session.Token()

    if (!Token) {
      globalThis.dispatchEvent(new Event("panel-unauthorized"))
      return
    }

    let Live: WebSocket

    try {
      Live = new WebSocket(_Url())
    } catch {
      _Retry()
      return
    }

    Socket = Live

    Live.onopen = () => Live.send(Token)

    Live.onmessage = (Event) => {
      try {
        OnData(JSON.parse(String(Event.data)) as Performance)
        Backoff = _RetryMs
      } catch {}
    }

    Live.onerror = () => Live.close()

    Live.onclose = () => {
      if (Socket === Live) Socket = null
      _Retry()
    }
  }

  _Connect()

  return {
    Close: () => {
      Closed = true
      if (Timer !== null) clearTimeout(Timer)
      Timer = null
      Socket?.close()
      Socket = null
    },
  }
}
