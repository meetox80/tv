import { ref, type Ref } from "vue"
import { PreviewClip } from "../../_api/voting"

export type Phase = "idle" | "loading" | "playing" | "paused"

interface Voice {
  Audio: HTMLAudioElement
  Gain: GainNode | null
  Id: number | null
  Timer: number
  Ramp: number
}

export const FadeMs = 140

export const Current: Ref<number | null> = ref(null)
export const State: Ref<Phase> = ref("idle")
export const Progress: Ref<number> = ref(0)
export const Elapsed: Ref<number> = ref(0)
export const Length: Ref<number> = ref(0)
export const Missing: Ref<Set<number>> = ref(new Set())
export const Listening: Ref<boolean> = ref(false)

const _TailSeconds = 0.5
const _SettleMs = FadeMs * 6
const _ClipLimit = 8

const _Clips = new Map<number, Promise<string | null>>()
const _Urls = new Map<number, string>()
const _Voices: Voice[] = []
let _Context: AudioContext | null = null
let _Live: Voice | null = null
let _Frame = 0

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

function _Mark(Id: number, Gone: boolean): void {
  if (Missing.value.has(Id) === Gone) return
  const Next = new Set(Missing.value)
  if (Gone) Next.add(Id)
  else Next.delete(Id)
  Missing.value = Next
}

function _Evict(): void {
  for (const [Id, Url] of _Urls) {
    if (_Urls.size <= _ClipLimit) return
    if (_Voices.some(V => V.Id === Id)) continue
    URL.revokeObjectURL(Url)
    _Urls.delete(Id)
    _Clips.delete(Id)
  }
}

function _Clip(Id: number): Promise<string | null> {
  const Known = _Clips.get(Id)
  if (Known) return Known

  const Pending = PreviewClip(Id)
    .then(Blob => (Blob ? URL.createObjectURL(Blob) : null))
    .catch(() => null)
    .then(Url => {
      if (Url) {
        _Urls.set(Id, Url)
        _Evict()
      } else {
        _Clips.delete(Id)
      }
      _Mark(Id, !Url)
      return Url
    })

  _Clips.set(Id, Pending)
  return Pending
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
    if (!V || State.value === "paused") return
    const A = V.Audio
    if (A.duration > 0 && Number.isFinite(A.duration)) {
      Length.value = A.duration
      Elapsed.value = A.currentTime
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
  _Mark(V.Id, true)
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
    if (V !== _Live || State.value === "paused") return
    State.value = "playing"
    _Fade(V, 1)
    _Tick()
  })
  A.addEventListener("waiting", () => {
    if (V === _Live && State.value === "playing") State.value = "loading"
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

async function _Resume(V: Voice): Promise<void> {
  clearTimeout(V.Timer)
  if (!V.Audio.paused) {
    State.value = "playing"
    _Fade(V, 1)
    _Tick()
    return
  }
  State.value = "loading"
  try {
    await V.Audio.play()
  } catch {
    if (_Live === V) Stop()
  }
}

export function Stop(): void {
  cancelAnimationFrame(_Frame)
  const V = _Live
  _Live = null
  Current.value = null
  State.value = "idle"
  Progress.value = 0
  Elapsed.value = 0
  if (V) _Release(V)
}

export function Close(): void {
  Listening.value = false
  Stop()
}

export function Prefetch(Id: number): void {
  void _Clip(Id)
}

export async function Play(Id: number): Promise<void> {
  const Live = _Live
  if (Live && Current.value === Id) {
    if (State.value === "paused") await _Resume(Live)
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
  Length.value = 0

  const Url = await _Clip(Id)
  if (_Live !== V) return
  if (!Url) {
    Stop()
    return
  }

  if (V.Audio.src !== Url) V.Audio.src = Url
  V.Audio.currentTime = 0
  if ((State.value as Phase) === "paused") return

  try {
    await V.Audio.play()
  } catch (Err) {
    if (_Live !== V) return
    if (Err instanceof DOMException && Err.name === "AbortError") return
    if (Err instanceof DOMException && Err.name === "NotAllowedError") {
      Listening.value = false
      Stop()
      return
    }
    _Fail(V)
  }
}

export function Pause(): void {
  const V = _Live
  if (!V || State.value === "paused") return
  Listening.value = false
  State.value = "paused"
  cancelAnimationFrame(_Frame)
  _Fade(V, 0)
  clearTimeout(V.Timer)
  V.Timer = window.setTimeout(() => {
    if (_Live === V && State.value === "paused") V.Audio.pause()
  }, _SettleMs)
}

export function Toggle(Id: number): void {
  if (Current.value === Id && (State.value === "playing" || State.value === "loading")) {
    Pause()
    return
  }
  _Mark(Id, false)
  Listening.value = true
  void Play(Id)
}

export function Follow(Id: number | null): void {
  if (Id !== null && Listening.value) {
    void Play(Id)
    return
  }
  if (Current.value !== Id) Stop()
}

export function Seek(Fraction: number): void {
  const V = _Live
  if (!V) return
  const A = V.Audio
  if (!(A.duration > 0) || !Number.isFinite(A.duration)) return
  const At = Math.min(1, Math.max(0, Fraction)) * A.duration
  A.currentTime = Math.min(At, A.duration - _TailSeconds - 0.05)
  Elapsed.value = A.currentTime
  Progress.value = A.currentTime / A.duration
}
