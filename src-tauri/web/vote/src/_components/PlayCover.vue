<script setup lang="ts">
import { computed, onUnmounted, ref, watch, type Ref } from "vue"
import Cover from "./Cover.vue"
import { Current, FadeMs, Progress, State, Toggle, Unavailable } from "../_state/preview"

const Props = withDefaults(defineProps<{
  Id: number
  Src: string | null
  Title: string
  Size?: number
  Radius?: number
  Fill?: boolean
  Large?: boolean
}>(), { Size: 56, Radius: 12, Fill: false, Large: false })

const Center = 24
const Radius = 18.5
const Amplitude = 0.9
const Waves = 12
const Settle = 0.55
const Step = 0.05
const Turn = Math.PI * 2
const Top = -Math.PI / 2
const DriftMs = 2600
const SpinMs = 1100
const LoadingSpan = 0.28
const EaseMs = FadeMs

const _Still = typeof matchMedia === "function" && matchMedia("(prefers-reduced-motion: reduce)").matches

const Active = computed(() => Current.value === Props.Id)
const Playing = computed(() => Active.value && State.value === "playing")
const Missing = computed(() => Unavailable.value.has(Props.Id))

const Presence: Ref<number> = ref(0)
const Mix: Ref<number> = ref(0)
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
  if (Active.value) {
    Mix.value = _Approach(Mix.value, Playing.value ? 1 : 0, Delta)
    Shown.value = Progress.value
    if (!Playing.value) Spin.value = (Spin.value + (Delta / SpinMs) * Turn) % Turn
  }
  if (!_Still) Phase.value = (Phase.value + (Delta / DriftMs) * Turn) % Turn

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
  const Amp = Amplitude * P

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

const Box = computed(() => {
  if (Props.Fill) return { width: "100%", height: "100%", borderRadius: "inherit" }
  return { width: `${Props.Size}px`, height: `${Props.Size}px`, borderRadius: `${Props.Radius}px` }
})

const Label = computed(() => (Active.value ? "Zatrzymaj fragment" : `Posłuchaj fragmentu: ${Props.Title}`))
</script>

<template>
  <div class="PlayCover" :class="{ Active, Large }" :style="Box">
    <Cover :Src="Src" :Title="Title" :Size="Size" :Radius="Radius" :Fill="Fill" />

    <button
      v-if="!Missing"
      type="button"
      class="Hit"
      :aria-label="Label"
      :aria-pressed="Active"
      @click="Toggle(Id)"
    >
      <span class="Knob" aria-hidden="true">
        <svg v-if="Presence > 0" class="Ring" viewBox="0 0 48 48" :style="{ opacity: Geometry.Opacity }">
          <path v-if="Geometry.Track" class="Track" :d="Geometry.Track" />
          <path v-if="Geometry.Wave" class="Wave" :d="Geometry.Wave" />
        </svg>
        <Transition name="glyph" mode="out-in">
          <svg v-if="Active" key="pause" class="Glyph" viewBox="0 0 24 24">
            <rect x="7" y="6" width="3.4" height="12" rx="1.2" />
            <rect x="13.6" y="6" width="3.4" height="12" rx="1.2" />
          </svg>
          <svg v-else key="play" class="Glyph Play" viewBox="0 0 24 24">
            <path d="M7.1 6.2v11.6a1 1 0 0 0 1.52.85l9.2-5.8a1 1 0 0 0 0-1.7l-9.2-5.8a1 1 0 0 0-1.52.85Z" />
          </svg>
        </Transition>
      </span>
    </button>
  </div>
</template>

<style scoped>
.PlayCover {
  --knob: 24px;
  --inset: 4px;
  --playing: 36px;
  --glyph: 12px;
  --glyph-playing: 14px;
  position: relative;
  flex-shrink: 0;
  overflow: hidden;
}

.PlayCover.Large {
  --knob: 26px;
  --inset: 8px;
  --playing: 58px;
  --glyph: 12px;
  --glyph-playing: 20px;
}

.PlayCover::after {
  content: "";
  position: absolute;
  inset: 0;
  border-radius: inherit;
  background: rgba(6, 5, 8, 0.5);
  opacity: 0;
  transition: opacity 320ms ease;
  pointer-events: none;
}

.PlayCover.Active::after {
  opacity: 1;
}

.Hit {
  position: absolute;
  inset: 0;
  z-index: 1;
  display: block;
  border-radius: inherit;
}

.Hit:focus-visible {
  outline-offset: -2px;
}

.Knob {
  position: absolute;
  top: var(--inset);
  right: var(--inset);
  display: flex;
  height: var(--knob);
  width: var(--knob);
  align-items: center;
  justify-content: center;
  border-radius: 999px;
  background: rgba(12, 11, 14, 0.55);
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.14);
  backdrop-filter: blur(10px) saturate(1.4);
  -webkit-backdrop-filter: blur(10px) saturate(1.4);
  color: #fff;
  transition:
    top 420ms var(--ease-smooth),
    right 420ms var(--ease-smooth),
    height 420ms var(--ease-smooth),
    width 420ms var(--ease-smooth),
    background 280ms ease,
    box-shadow 280ms ease,
    transform 160ms ease;
}

.Hit:hover .Knob {
  background: rgba(12, 11, 14, 0.72);
}

.Hit:active .Knob {
  transform: scale(0.92);
}

.Active .Knob {
  top: calc(50% - var(--playing) / 2);
  right: calc(50% - var(--playing) / 2);
  height: var(--playing);
  width: var(--playing);
  background: rgba(12, 11, 14, 0);
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0);
  backdrop-filter: none;
  -webkit-backdrop-filter: none;
}

.Ring {
  position: absolute;
  inset: 0;
  height: 100%;
  width: 100%;
  overflow: visible;
  fill: none;
  stroke-width: 2.2;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.Wave {
  stroke: rgba(255, 255, 255, 0.95);
}

.Track {
  stroke: rgba(255, 255, 255, 0.22);
}

.Glyph {
  position: relative;
  height: var(--glyph);
  width: var(--glyph);
  fill: currentColor;
  transition: height 420ms var(--ease-smooth), width 420ms var(--ease-smooth);
}

.Active .Glyph {
  height: var(--glyph-playing);
  width: var(--glyph-playing);
}

.glyph-enter-active,
.glyph-leave-active {
  transition: opacity 160ms ease, transform 200ms var(--ease-smooth);
}

.glyph-enter-from,
.glyph-leave-to {
  opacity: 0;
  transform: scale(0.6);
}

@media (min-width: 900px) {
  .PlayCover.Large {
    --knob: 32px;
    --inset: 14px;
    --playing: 76px;
    --glyph: 14px;
    --glyph-playing: 24px;
  }
}
</style>
