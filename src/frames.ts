import { invoke } from "@tauri-apps/api/core"

const _Window = 600
const _ReportMs = 2000
const _MinSamples = 30
const _MaxFrameMs = 1000

const _Times = new Float64Array(_Window)

let _Count = 0
let _Cursor = 0
let _Last = 0
let _Frame: number | null = null
let _Timer: ReturnType<typeof setInterval> | null = null

function _Record(Delta: number): void {
  _Times[_Cursor] = Delta
  _Cursor = (_Cursor + 1) % _Window
  if (_Count < _Window) _Count += 1
}

function _Tick(Stamp: number): void {
  if (_Last > 0) {
    const Delta = Stamp - _Last
    if (Delta > 0 && Delta < _MaxFrameMs) _Record(Delta)
  }

  _Last = Stamp
  _Frame = requestAnimationFrame(_Tick)
}

function _Stats(): { Fps: number; Low: number } | null {
  if (_Count < _MinSamples) return null

  let Total = 0
  for (let I = 0; I < _Count; I++) Total += _Times[I]
  if (Total <= 0) return null

  const Sorted = Array.from(_Times.subarray(0, _Count)).sort((A, B) => B - A)
  const Slice = Math.max(1, Math.round(_Count / 100))

  let Worst = 0
  for (let I = 0; I < Slice; I++) Worst += Sorted[I]

  return {
    Fps: 1000 / (Total / _Count),
    Low: 1000 / (Worst / Slice),
  }
}

export function Start(): void {
  if (_Frame !== null) return

  _Frame = requestAnimationFrame(_Tick)

  _Timer = setInterval(() => {
    const Stats = _Stats()
    if (!Stats) return

    invoke("report_frame_stats", { fps: Stats.Fps, low: Stats.Low }).catch(() => {})
  }, _ReportMs)
}

export function Stop(): void {
  if (_Frame !== null) cancelAnimationFrame(_Frame)
  if (_Timer !== null) clearInterval(_Timer)

  _Frame = null
  _Timer = null
  _Last = 0
}
