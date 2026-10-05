import { ref, type Ref } from "vue"

type Phase = "idle" | "loading" | "playing"

interface Voice {
  Audio: HTMLAudioElement
  Gain: GainNode | null
  Id: number | null
  Timer: number
  Ramp: number
}

export const FadeMs = 120

export const Current: Ref<number | null> = ref(null)
export const State: Ref<Phase> = ref("idle")
export const Progress: Ref<number> = ref(0)
export const Unavailable: Ref<Set<number>> = ref(new Set())

const _TailSeconds = 0.6
const _SettleMs = FadeMs * 6

const _Voices: Voice[] = []
let _Context: AudioContext | null = null
let _Live: Voice | null = null
let _Frame = 0

function _Path(Id: number): string {
  return `/api/songs/${Id}/preview`
}

function _Ctx(): AudioContext | null {
  if (_Context) return _Context
  const Ctor = window.AudioContext ?? (window as unknown as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext
  if (!Ctor) return null
  try {
    _Context = new Ctor()
  } catch {
    _Context = null
  }
  return _Context
}

function _Level(V: Voice, To: number): void {
  cancelAnimationFrame(V.Ramp)
  if (V.Gain && _Context) {
    const Now = _Context.currentTime
    V.Gain.gain.cancelScheduledValues(Now)
    V.Gain.gain.setValueAtTime(To, Now)
    return
  }
  V.Audio.volume = To
}

function _Fade(V: Voice, To: number): void {
  cancelAnimationFrame(V.Ramp)
  if (V.Gain && _Context) {
    const Param = V.Gain.gain
    const Now = _Context.currentTime
    Param.cancelScheduledValues(Now)
    Param.setValueAtTime(Param.value, Now)
    Param.setTargetAtTime(To, Now, FadeMs / 1000)
    return
  }

  const From = V.Audio.volume
  const Start = performance.now()
  const Step = (Stamp: number): void => {
    const Done = 1 - Math.exp(-(Stamp - Start) / FadeMs)
    if (Done > 0.995) {
      V.Audio.volume = To
      return
    }
    V.Audio.volume = From + (To - From) * Done
    V.Ramp = requestAnimationFrame(Step)
  }
  V.Ramp = requestAnimationFrame(Step)
}

function _Tick(): void {
  cancelAnimationFrame(_Frame)
  const Step = (): void => {
    const V = _Live
    if (!V) return
    const A = V.Audio
    if (A.duration > 0) {
      Progress.value = Math.min(1, A.currentTime / A.duration)
      if (A.duration - A.currentTime <= _TailSeconds) {
        Stop()
        return
      }
    }
    if (!A.paused) _Frame = requestAnimationFrame(Step)
  }
  Step()
}

function _Fail(V: Voice): void {
  if (V !== _Live || V.Id === null) return
  Unavailable.value = new Set(Unavailable.value).add(V.Id)
  Stop()
}

function _Make(): Voice {
  const A = new Audio()
  A.preload = "auto"
  const V: Voice = { Audio: A, Gain: null, Id: null, Timer: 0, Ramp: 0 }

  const Ctx = _Ctx()
  if (Ctx) {
    try {
      const Gain = Ctx.createGain()
      Gain.gain.value = 0
      Ctx.createMediaElementSource(A).connect(Gain).connect(Ctx.destination)
      V.Gain = Gain
    } catch {
      V.Gain = null
    }
  }

  A.addEventListener("playing", () => {
    if (V !== _Live) return
    State.value = "playing"
    _Fade(V, 1)
    _Tick()
  })
  A.addEventListener("waiting", () => {
    if (V === _Live) State.value = "loading"
  })
  A.addEventListener("ended", () => {
    if (V === _Live) Stop()
  })
  A.addEventListener("error", () => _Fail(V))

  return V
}

function _Release(V: Voice): void {
  _Fade(V, 0)
  clearTimeout(V.Timer)
  V.Timer = window.setTimeout(() => {
    if (V === _Live) return
    V.Audio.pause()
    V.Id = null
  }, _SettleMs)
}

export function Stop(): void {
  cancelAnimationFrame(_Frame)
  const V = _Live
  _Live = null
  Current.value = null
  State.value = "idle"
  if (V) _Release(V)
}

export async function Toggle(Id: number): Promise<void> {
  if (Current.value === Id) {
    Stop()
    return
  }

  const Ctx = _Ctx()
  if (Ctx && Ctx.state === "suspended") void Ctx.resume()
  while (_Voices.length < 2) _Voices.push(_Make())

  const Previous = _Live
  Stop()

  const V = _Voices.find(X => X !== Previous && X.Id === null) ?? _Voices.find(X => X !== Previous) ?? _Voices[0]
  clearTimeout(V.Timer)
  V.Audio.pause()
  _Level(V, 0)

  V.Id = Id
  _Live = V
  Current.value = Id
  State.value = "loading"
  Progress.value = 0
  V.Audio.src = _Path(Id)

  try {
    await V.Audio.play()
  } catch (Err) {
    if (_Live !== V) return
    if (Err instanceof DOMException && Err.name === "AbortError") return
    if (Err instanceof DOMException && Err.name === "NotAllowedError") {
      Stop()
      return
    }
    _Fail(V)
  }
}

if (typeof document !== "undefined") {
  document.addEventListener("visibilitychange", () => {
    if (document.hidden) Stop()
  })
}
