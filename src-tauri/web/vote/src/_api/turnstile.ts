interface RenderOptions {
  sitekey: string
  action: string
  theme: "dark" | "light" | "auto"
  language: string
  appearance: "always" | "execute" | "interaction-only"
  callback: (Token: string) => void
  "error-callback": () => void
  "expired-callback": () => void
  "timeout-callback": () => void
}

interface Turnstile {
  render: (Target: HTMLElement, Options: RenderOptions) => string | undefined
  remove: (WidgetId: string) => void
  reset: (WidgetId: string) => void
}

declare global {
  interface Window {
    turnstile?: Turnstile
  }
}

const _ScriptUrl = "https://challenges.cloudflare.com/turnstile/v0/api.js?render=explicit"

let _Loading: Promise<Turnstile> | null = null

export function Load(): Promise<Turnstile> {
  if (window.turnstile) return Promise.resolve(window.turnstile)
  if (_Loading) return _Loading

  _Loading = new Promise<Turnstile>((Resolve, Reject) => {
    const Script = document.createElement("script")
    Script.src = _ScriptUrl
    Script.async = true
    Script.defer = true
    Script.referrerPolicy = "strict-origin"

    Script.onload = () => {
      if (window.turnstile) Resolve(window.turnstile)
      else Reject(new Error("turnstile"))
    }

    Script.onerror = () => {
      Script.remove()
      _Loading = null
      Reject(new Error("turnstile"))
    }

    document.head.appendChild(Script)
  })

  return _Loading
}

export type { Turnstile }
