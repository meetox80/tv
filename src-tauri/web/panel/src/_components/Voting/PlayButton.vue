<script setup lang="ts">
import { computed, onUnmounted, ref, watch, type Ref } from "vue"
import { Current, FadeMs, Missing, Progress, State, Toggle } from "./_preview"

const Props = defineProps<{ Id: number | null; Label: string }>()

const Center = 32
const Radius = 30
const Amplitude = 1.15
const Waves = 14
const Settle = 0.5
const Step = 0.04
const Turn = Math.PI * 2
const Top = -Math.PI / 2
const DriftMs = 2600
const SpinMs = 1100
const LoadingSpan = 0.28
const EaseMs = FadeMs * 1.4

const _Still = typeof matchMedia === "function" && matchMedia("(prefers-reduced-motion: reduce)").matches

const Active = computed(() => Props.Id !== null && Current.value === Props.Id)
const Playing = computed(() => Active.value && State.value === "playing")
const Paused = computed(() => Active.value && State.value === "paused")
const Gone = computed(() => Props.Id !== null && Missing.value.has(Props.Id))

const Presence: Ref<number> = ref(0)
const Mix: Ref<number> = ref(0)
const Calm: Ref<number> = ref(0)
const Phase: Ref<number> = ref(0)
const Spin: Ref<number> = ref(0)
const Shown: Ref<number> = ref(0)

let _Frame = 0
let _Last = 0

function _Approach(Value: number, Target: number, Delta: number): number {
  const Next = Value + (Target - Value) * (1 - Math.exp(-Delta / EaseMs))
  return Math.abs(Target - Next) < 0.002 ? Target : Next
}

function _Loop(Stamp: number): void {
  const Delta = _Last === 0 ? 16 : Math.min(Stamp - _Last, 64)
  _Last = Stamp

  Presence.value = _Approach(Presence.value, Active.value ? 1 : 0, Delta)
  Calm.value = _Approach(Calm.value, Playing.value ? 1 : 0, Delta)
  if (Active.value) {
    Mix.value = _Approach(Mix.value, Playing.value || Paused.value ? 1 : 0, Delta)
    Shown.value = Progress.value
    if (!Playing.value && !Paused.value) Spin.value = (Spin.value + (Delta / SpinMs) * Turn) % Turn
  }
  if (!_Still && Playing.value) Phase.value = (Phase.value + (Delta / DriftMs) * Turn) % Turn

  if (!Active.value && Presence.value === 0) {
    _Frame = 0
    _Last = 0
    return
  }
  _Frame = requestAnimationFrame(_Loop)
}

function _Wake(): void {
  if (_Frame) return
  _Last = 0
  _Frame = requestAnimationFrame(_Loop)
}

watch(Active, On => {
  if (On) {
    Spin.value = 0
    Shown.value = 0
    Mix.value = 0
  }
  _Wake()
})

watch(State, _Wake)

onUnmounted(() => cancelAnimationFrame(_Frame))

function _Radius(Theta: number, Edge: number, Amp: number): number {
  const Damp = Theta <= Edge ? 1 : Math.max(0, 1 - (Theta - Edge) / Settle)
  return Radius + Amp * Damp * Math.sin(Waves * Theta - Phase.value)
}

function _Path(From: number, To: number, Edge: number, Amp: number): string {
  if (To - From < 0.01) return ""
  const Out: string[] = []
  const Put = (Theta: number): void => {
    const R = _Radius(Theta, Edge, Amp)
    const X = Center + R * Math.cos(Theta)
    const Y = Center + R * Math.sin(Theta)
    Out.push(`${Out.length === 0 ? "M" : "L"}${X.toFixed(2)} ${Y.toFixed(2)}`)
  }
  for (let Theta = From; Theta < To; Theta += Step) Put(Theta)
  Put(To)
  return Out.join(" ")
}

const Geometry = computed(() => {
  const P = Presence.value
  const M = Mix.value
  const Amp = Amplitude * P * Calm.value

  const SpinStart = Top + Spin.value - Turn
  const Start = SpinStart + (Top - SpinStart) * M
  const Length = (LoadingSpan + (Shown.value - LoadingSpan) * M) * Turn * P
  const Sweep = Turn * P

  const Edge = Start + Math.min(Length, Sweep)
  return {
    Wave: _Path(Start, Edge, Edge, Amp),
    Track: _Path(Edge, Start + Sweep, Edge, Amp),
    Opacity: Math.min(1, P * 1.4),
  }
})

function OnClick(): void {
  if (Props.Id !== null) Toggle(Props.Id)
}
</script>

<template>
  <button
    type="button"
    class="Play"
    :class="{ Active, Playing, Gone }"
    :disabled="Id === null"
    :aria-label="Label"
    :aria-pressed="Active && !Paused"
    @click="OnClick"
  >
    <svg v-if="Presence > 0" class="Ring" viewBox="0 0 64 64" :style="{ opacity: Geometry.Opacity }" aria-hidden="true">
      <path v-if="Geometry.Track" class="Track" :d="Geometry.Track" />
      <path v-if="Geometry.Wave" class="Wave" :d="Geometry.Wave" />
    </svg>
    <Transition name="glyph" mode="out-in">
      <svg v-if="Active && !Paused" key="pause" class="Glyph" viewBox="0 0 24 24" aria-hidden="true">
        <rect x="7" y="6" width="3.4" height="12" rx="1.2" />
        <rect x="13.6" y="6" width="3.4" height="12" rx="1.2" />
      </svg>
      <svg v-else key="play" class="Glyph Triangle" viewBox="0 0 24 24" aria-hidden="true">
        <path d="M7.1 6.2v11.6a1 1 0 0 0 1.52.85l9.2-5.8a1 1 0 0 0 0-1.7l-9.2-5.8a1 1 0 0 0-1.52.85Z" />
      </svg>
    </Transition>
  </button>
</template>

<style scoped>
.Play {
  position: relative;
  display: flex;
  height: 64px;
  width: 64px;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.06);
  box-shadow: inset 0 0 0 1px var(--edge);
  color: var(--ink);
  cursor: pointer;
  transition: background 200ms ease, box-shadow 200ms ease, transform 160ms ease, color 200ms ease;
}

.Play:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.1);
}

.Play:active:not(:disabled) {
  transform: scale(0.94);
}

.Play:disabled {
  opacity: 0.35;
  cursor: default;
}

.Play.Active {
  background: rgba(255, 255, 255, 0.04);
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0);
}

.Play.Gone {
  color: var(--ink-faint);
}

.Ring {
  position: absolute;
  inset: 0;
  height: 100%;
  width: 100%;
  overflow: visible;
  fill: none;
  stroke-width: 2;
  stroke-linecap: round;
  stroke-linejoin: round;
  pointer-events: none;
}

.Wave {
  stroke: rgba(255, 255, 255, 0.95);
}

.Track {
  stroke: rgba(255, 255, 255, 0.18);
}

.Glyph {
  height: 22px;
  width: 22px;
  fill: currentColor;
}

.Glyph.Triangle {
  transform: translateX(1px);
}

.glyph-enter-active,
.glyph-leave-active {
  transition: opacity 140ms ease, transform 200ms var(--ease);
}

.glyph-enter-from,
.glyph-leave-to {
  opacity: 0;
  transform: scale(0.6);
}
</style>
