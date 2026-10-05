<script setup lang="ts">
import { ref, computed, onMounted, type Ref } from "vue"
import { useI18n } from "../../i18n"
import type { AdminSong } from "../../_api/voting"
import Cover from "./Cover.vue"
import { Relative, Duration, ProviderName } from "./_format"
import { Current, Progress as Played, Seek, State } from "./_preview"

const { t } = useI18n()

const Props = defineProps<{
  Song: AdminSong
  Depth: number
  EnterFrom: -1 | 0 | 1
  Tick: number
  Pull: number
}>()

const Emit = defineEmits<{
  Decide: [direction: -1 | 1]
  Fling: [direction: -1 | 1]
  Pull: [progress: number]
  Listen: []
  Ban: []
}>()

const _Threshold = 120
const _TapSlop = 6
const _FlingVelocity = 0.55
const _FlingMinDistance = 36
const _MaxTilt = 16
const _Reduced = typeof window !== "undefined" && window.matchMedia?.("(prefers-reduced-motion: reduce)").matches

const X: Ref<number> = ref(0)
const Y: Ref<number> = ref(0)
const Lean: Ref<1 | -1> = ref(1)
const Dragging: Ref<boolean> = ref(false)
const Settling: Ref<"none" | "spring" | "fling" | "enter" | "place"> = ref(Props.EnterFrom ? "place" : "none")
const Gone: Ref<boolean> = ref(false)

let _PointerId: number | null = null
let _StartX = 0
let _StartY = 0
let _OriginX = 0
let _OriginY = 0
let _Travel = 0
let _OnArt = false
let _Samples: { X: number; T: number }[] = []

const Top = computed(() => Props.Depth === 0)

const Progress = computed(() => Math.max(-1.4, Math.min(1.4, X.value / _Threshold)))

const Verdict = computed(() => {
  const P = Progress.value
  const Strength = Math.min(1, Math.abs(P))
  const Draw = Math.min(1, Math.max(0, (Strength - 0.12) / 0.88))
  const Part = (From: number, To: number): number => Math.min(1, Math.max(0, (Draw - From) / (To - From)))
  return {
    Approve: P > 0,
    Armed: Math.abs(P) >= 1,
    Strokes: P > 0 ? [Draw] : [Part(0, 0.55), Part(0.45, 1)],
    Shown: Math.min(1, Draw * 4),
    Veil: Strength,
  }
})

const Live = computed(() => Current.value === Props.Song.Id)
const Sounding = computed(() => Live.value && State.value === "playing")

const CardStyle = computed(() => {
  if (!Top.value) {
    const Lift = Props.Depth === 1 ? Math.min(1, Math.abs(Props.Pull)) : 0
    const Level = Props.Depth - Lift
    return {
      transform: `translate3d(0, ${Level * 14}px, 0) scale(${1 - Level * 0.05})`,
      opacity: String(Props.Depth > 2 ? 0 : Math.max(0, 1 - Level * 0.4)),
      "--reveal": String(Lift),
    }
  }

  const Tilt = Math.max(-_MaxTilt, Math.min(_MaxTilt, X.value * 0.04 * Lean.value))
  return {
    transform: `translate3d(${X.value}px, ${Y.value * 0.3}px, 0) rotate(${Tilt}deg)`,
  }
})

const Length = computed(() => Duration(Props.Song.DurationMs))

const Requested = computed(() => {
  void Props.Tick
  return Relative(Props.Song.Created)
})

function _Blocked(Target: EventTarget | null): boolean {
  return Target instanceof Element && !!Target.closest("a, button, input")
}

function OnDown(Event: PointerEvent): void {
  if (!Top.value || Gone.value || _Blocked(Event.target)) return
  if (Event.button !== 0 && Event.pointerType === "mouse") return

  const Box = (Event.currentTarget as HTMLElement).getBoundingClientRect()
  Lean.value = Event.clientY - Box.top < Box.height / 2 ? 1 : -1
  _OnArt = Event.target instanceof Element && !!Event.target.closest(".Art")
  _PointerId = Event.pointerId
  _StartX = Event.clientX - X.value
  _StartY = Event.clientY - Y.value
  _OriginX = Event.clientX
  _OriginY = Event.clientY
  _Travel = 0
  _Samples = [{ X: Event.clientX, T: performance.now() }]
  Dragging.value = true
  Settling.value = "none"
  ;(Event.currentTarget as HTMLElement).setPointerCapture(Event.pointerId)
}

function OnMove(Event: PointerEvent): void {
  if (!Dragging.value || Event.pointerId !== _PointerId) return

  X.value = Event.clientX - _StartX
  Y.value = Event.clientY - _StartY
  _Travel = Math.max(_Travel, Math.hypot(Event.clientX - _OriginX, Event.clientY - _OriginY))
  Emit("Pull", Progress.value)

  const Now = performance.now()
  _Samples.push({ X: Event.clientX, T: Now })
  _Samples = _Samples.filter(S => Now - S.T < 120)
}

function _Return(): void {
  Settling.value = "spring"
  X.value = 0
  Y.value = 0
  Emit("Pull", 0)
}

function OnUp(Event: PointerEvent): void {
  if (!Dragging.value || Event.pointerId !== _PointerId) return

  Dragging.value = false
  _PointerId = null

  if (_Travel < _TapSlop) {
    _Return()
    if (_OnArt && Event.type === "pointerup") Emit("Listen")
    return
  }

  const First = _Samples[0]
  const Last = _Samples[_Samples.length - 1]
  const Elapsed = Last && First ? Last.T - First.T : 0
  const Velocity = Elapsed > 0 ? (Last.X - First.X) / Elapsed : 0

  if (Math.abs(X.value) > _Threshold) {
    Fling(X.value > 0 ? 1 : -1)
    return
  }

  if (Math.abs(Velocity) > _FlingVelocity && Math.abs(X.value) > _FlingMinDistance && Math.sign(Velocity) === Math.sign(X.value)) {
    Fling(Velocity > 0 ? 1 : -1)
    return
  }

  _Return()
}

function Fling(Direction: -1 | 1): void {
  if (Gone.value) return

  Gone.value = true
  Settling.value = "fling"
  Emit("Fling", Direction)
  const Width = typeof window !== "undefined" ? window.innerWidth : 1200
  X.value = Direction * (Width * 0.7 + 420)
  Y.value = Y.value + 36

  setTimeout(() => Emit("Decide", Direction), _Reduced ? 60 : 340)
}

function OnSeek(Event: Event): void {
  Seek(Number((Event.target as HTMLInputElement).value))
}

function OnSettled(): void {
  if (Settling.value === "spring" || Settling.value === "enter") Settling.value = "none"
}

onMounted(() => {
  if (!Props.EnterFrom || !Top.value) return

  X.value = Props.EnterFrom * 520
  Y.value = 30
  requestAnimationFrame(() => {
    requestAnimationFrame(() => {
      Settling.value = "enter"
      X.value = 0
      Y.value = 0
    })
  })
})

defineExpose({ Fling })
</script>

<template>
  <article
    class="Card"
    :class="{ Top, Behind: !Top, Dragging, [Settling]: Settling !== 'none' }"
    :style="CardStyle"
    :aria-hidden="!Top"
    @pointerdown="OnDown"
    @pointermove="OnMove"
    @pointerup="OnUp"
    @pointercancel="OnUp"
    @transitionend.self="OnSettled"
  >
    <div class="Art" :class="{ Live, Sounding }">
      <Cover :Src="Song.Cover" class="ArtImage" />

      <span v-if="Top" class="Hint" aria-hidden="true">
        <svg v-if="Sounding" viewBox="0 0 24 24">
          <rect x="7" y="6" width="3.4" height="12" rx="1.2" />
          <rect x="13.6" y="6" width="3.4" height="12" rx="1.2" />
        </svg>
        <svg v-else viewBox="0 0 24 24">
          <path d="M7.1 6.2v11.6a1 1 0 0 0 1.52.85l9.2-5.8a1 1 0 0 0 0-1.7l-9.2-5.8a1 1 0 0 0-1.52.85Z" />
        </svg>
      </span>

      <span v-if="Live" class="Meter" :class="{ Still: !Sounding }" aria-hidden="true">
        <i /><i /><i />
      </span>

      <input
        v-if="Live && Top"
        class="Scrub"
        type="range"
        min="0"
        max="1"
        step="0.001"
        :value="Played"
        :style="{ '--fill': `${Played * 100}%` }"
        :aria-label="t('voting.review.preview.seek')"
        @input="OnSeek"
      />

      <div
        v-if="Top"
        class="Verdict"
        :class="{ Approve: Verdict.Approve, Armed: Verdict.Armed }"
        :style="{ '--veil': Verdict.Veil }"
        aria-hidden="true"
      >
        <svg viewBox="0 0 48 48" :style="{ opacity: Verdict.Shown }">
          <path
            v-for="(Line, Index) in Verdict.Approve ? ['M13.5 25 20.8 32.2 35 16.8'] : ['M16 16 32 32', 'M32 16 16 32']"
            :key="Line"
            pathLength="1"
            :d="Line"
            :style="{ strokeDashoffset: 1 - (Verdict.Strokes[Index] ?? 0) }"
          />
        </svg>
      </div>
    </div>

    <div class="Body">
      <div class="Head">
        <h2 class="Title">
          <span class="TitleText">{{ Song.Title }}</span>
          <span v-if="Song.Explicit" class="Explicit" :title="t('voting.review.explicit')">E</span>
        </h2>
        <p class="Artist">{{ Song.Artist }}</p>
      </div>

      <dl class="Facts">
        <div v-if="Song.Album" class="Fact">
          <dt>{{ t("voting.review.album") }}</dt>
          <dd>{{ Song.Album }}</dd>
        </div>
        <div v-if="Length" class="Fact">
          <dt>{{ t("voting.review.length") }}</dt>
          <dd class="tabular-nums">{{ Length }}</dd>
        </div>
        <div class="Fact">
          <dt>{{ t("voting.review.requested") }}</dt>
          <dd>{{ Requested }}</dd>
        </div>
        <div v-if="Song.Device" class="Fact">
          <dt>{{ t("voting.review.device") }}</dt>
          <dd :class="{ Banned: Song.DeviceBanned }">
            <span class="Monospace">{{ Song.Device }}</span>
            <span v-if="Song.DeviceBanned" class="BannedTag">{{ t("voting.review.device.banned") }}</span>
          </dd>
        </div>
      </dl>

      <div class="Links">
        <a
          v-if="Song.Link"
          :href="Song.Link"
          target="_blank"
          rel="noopener noreferrer"
          class="Link"
          :tabindex="Top ? 0 : -1"
        >
          {{ t("voting.review.open") }} {{ ProviderName(Song.Provider) }}
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
            <path d="M7 17 17 7M9 7h8v8" />
          </svg>
        </a>

        <button
          v-if="Song.Device && !Song.DeviceBanned"
          type="button"
          class="Link Ban"
          :tabindex="Top ? 0 : -1"
          @click="Emit('Ban')"
        >
          {{ t("voting.review.ban") }}
        </button>
      </div>
    </div>
  </article>
</template>

<style scoped>
.Card {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  gap: 18px;
  overflow: hidden;
  border-radius: 20px;
  background: #0e0d11;
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.07);
  padding: 14px;
  transform-origin: 50% 100%;
  transition: transform 420ms var(--ease), opacity 320ms ease;
  will-change: transform;
  touch-action: pan-y;
}

.Card.Behind {
  --reveal: 0;
  pointer-events: none;
  transition: transform 260ms var(--ease), opacity 260ms ease;
}

.Card.Behind > * {
  opacity: var(--reveal);
  transition: opacity 200ms ease;
}

.Card.Top {
  position: relative;
  transform-origin: 50% 50%;
  cursor: grab;
  touch-action: none;
}

.Card.Top.Dragging,
.Card.Top.place {
  cursor: grabbing;
  transition: none;
}

.Card.Top.spring {
  transition: transform 480ms cubic-bezier(0.2, 1.3, 0.4, 1);
}

.Card.Top.fling {
  opacity: 0;
  transition: transform 440ms cubic-bezier(0.2, 0.7, 0.3, 1), opacity 240ms ease 140ms;
}

.Card.Top.enter {
  transition: transform 420ms cubic-bezier(0.16, 1.1, 0.3, 1);
}

.Art {
  position: relative;
  width: min(100%, max(200px, calc(100dvh - 560px)));
  flex-shrink: 0;
  align-self: center;
  aspect-ratio: 1 / 1;
}

.ArtImage {
  position: absolute;
  inset: 0;
  border-radius: 10px;
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.06);
  font-size: 40px;
}

.Hint {
  position: absolute;
  top: 50%;
  left: 50%;
  display: flex;
  height: 56px;
  width: 56px;
  align-items: center;
  justify-content: center;
  border-radius: 999px;
  background: rgba(12, 11, 14, 0.55);
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.14);
  backdrop-filter: blur(10px) saturate(1.4);
  -webkit-backdrop-filter: blur(10px) saturate(1.4);
  color: #fff;
  opacity: 0;
  transform: translate(-50%, -50%) scale(0.86);
  transition: opacity 200ms ease, transform 320ms var(--ease);
  pointer-events: none;
}

.Hint svg {
  height: 20px;
  width: 20px;
  fill: currentColor;
}

@media (hover: hover) {
  .Card.Top:not(.Dragging) .Art:hover .Hint {
    opacity: 1;
    transform: translate(-50%, -50%) scale(1);
  }

  .Card.Top .Art {
    cursor: pointer;
  }
}

.Meter {
  position: absolute;
  top: 10px;
  left: 10px;
  display: flex;
  height: 26px;
  align-items: flex-end;
  gap: 2.5px;
  border-radius: 8px;
  background: rgba(12, 11, 14, 0.55);
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.12);
  backdrop-filter: blur(10px) saturate(1.4);
  -webkit-backdrop-filter: blur(10px) saturate(1.4);
  padding: 7px 8px;
  pointer-events: none;
}

.Meter i {
  display: block;
  height: 100%;
  width: 2.5px;
  border-radius: 2px;
  background: #fff;
  transform-origin: 50% 100%;
  animation: Meter 900ms ease-in-out infinite;
}

.Meter i:nth-child(2) {
  animation-delay: -300ms;
  animation-duration: 760ms;
}

.Meter i:nth-child(3) {
  animation-delay: -560ms;
  animation-duration: 1040ms;
}

.Meter.Still i {
  animation-play-state: paused;
  opacity: 0.6;
}

@keyframes Meter {
  0%, 100% { transform: scaleY(0.3); }
  50% { transform: scaleY(1); }
}

.Scrub {
  --fill: 0%;
  position: absolute;
  right: 12px;
  bottom: 8px;
  left: 12px;
  height: 20px;
  margin: 0;
  appearance: none;
  -webkit-appearance: none;
  background: transparent;
  cursor: pointer;
  touch-action: none;
}

.Scrub::-webkit-slider-runnable-track {
  height: 3px;
  border-radius: 999px;
  background: linear-gradient(to right, #fff var(--fill), rgba(255, 255, 255, 0.28) var(--fill));
}

.Scrub::-moz-range-track {
  height: 3px;
  border-radius: 999px;
  background: linear-gradient(to right, #fff var(--fill), rgba(255, 255, 255, 0.28) var(--fill));
}

.Scrub::-webkit-slider-thumb {
  height: 11px;
  width: 11px;
  margin-top: -4px;
  appearance: none;
  -webkit-appearance: none;
  border: 0;
  border-radius: 999px;
  background: #fff;
  box-shadow: 0 1px 6px rgba(0, 0, 0, 0.45);
  transform: scale(0);
  transition: transform 160ms var(--ease);
}

.Scrub::-moz-range-thumb {
  height: 11px;
  width: 11px;
  border: 0;
  border-radius: 999px;
  background: #fff;
  box-shadow: 0 1px 6px rgba(0, 0, 0, 0.45);
  transform: scale(0);
  transition: transform 160ms var(--ease);
}

.Art:hover .Scrub::-webkit-slider-thumb,
.Scrub:focus-visible::-webkit-slider-thumb,
.Scrub:active::-webkit-slider-thumb {
  transform: scale(1);
}

.Art:hover .Scrub::-moz-range-thumb,
.Scrub:focus-visible::-moz-range-thumb,
.Scrub:active::-moz-range-thumb {
  transform: scale(1);
}

.Live .ArtImage::after {
  content: "";
  position: absolute;
  inset: 0;
  background: linear-gradient(to top, rgba(6, 5, 8, 0.55), rgba(6, 5, 8, 0) 34%);
  pointer-events: none;
}

.Verdict {
  --veil: 0;
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 10px;
  background: rgba(6, 5, 8, calc(var(--veil) * 0.62));
  color: #fff;
  pointer-events: none;
}

.Verdict svg {
  height: 88px;
  width: 88px;
  overflow: visible;
  fill: none;
  stroke: currentColor;
  stroke-width: 3.2;
  stroke-linecap: round;
  stroke-linejoin: round;
  stroke-dasharray: 1;
  transform: scale(0.92);
  transition: transform 360ms var(--spring), color 200ms ease;
}

.Verdict.Armed svg {
  transform: scale(1.06);
}

.Verdict.Armed.Approve {
  color: var(--good);
}

.Verdict.Armed:not(.Approve) {
  color: var(--danger);
}

.Body {
  display: flex;
  min-width: 0;
  flex: 1;
  flex-direction: column;
  padding: 0 4px 2px;
}

.Title {
  display: flex;
  min-width: 0;
  align-items: baseline;
  gap: 8px;
  margin: 0;
  font-family: var(--display);
  font-size: 21px;
  font-weight: 600;
  line-height: 1.2;
  letter-spacing: -0.02em;
  color: var(--ink);
}

.TitleText {
  display: -webkit-box;
  min-width: 0;
  overflow: hidden;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
}

.Explicit {
  display: inline-flex;
  height: 16px;
  min-width: 16px;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  border-radius: 3px;
  background: rgba(255, 255, 255, 0.16);
  font-family: Inter, system-ui, sans-serif;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0;
  color: var(--ink-soft);
  transform: translateY(-2px);
}

.Artist {
  margin: 4px 0 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 15px;
  color: var(--ink-soft);
}

.Facts {
  margin: 16px 0 0;
  border-top: 1px solid var(--line);
}

.Fact {
  display: flex;
  min-width: 0;
  align-items: baseline;
  justify-content: space-between;
  gap: 16px;
  border-bottom: 1px solid var(--line);
  padding: 9px 0;
  font-size: 13.5px;
}

.Fact dt {
  flex-shrink: 0;
  color: var(--ink-mute);
}

.Fact dd {
  min-width: 0;
  margin: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  text-align: right;
  color: var(--ink-soft);
}

.Fact dd.Banned {
  color: var(--danger);
}

.BannedTag {
  margin-left: 8px;
}

.Links {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-top: auto;
  padding-top: 14px;
}

.Link {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-size: 13.5px;
  color: var(--ink-soft);
  text-decoration: none;
  cursor: pointer;
  transition: color 160ms ease;
}

.Link svg {
  height: 13px;
  width: 13px;
}

.Link:hover {
  color: var(--ink);
}

.Link.Ban {
  margin-left: auto;
  color: rgba(239, 138, 138, 0.7);
}

.Link.Ban:hover {
  color: var(--danger);
}

@media (min-width: 900px) {
  .Card {
    flex-direction: row;
    gap: 24px;
    padding: 16px;
  }

  .Art {
    width: 300px;
    align-self: flex-start;
  }

  .Body {
    padding: 6px 6px 2px 0;
  }

  .Title {
    font-size: 24px;
  }

  .Artist {
    font-size: 16px;
  }

  .Facts {
    margin-top: 20px;
  }
}

@media (prefers-reduced-motion: reduce) {
  .Card,
  .Card.Top.spring,
  .Card.Top.enter {
    transition: opacity 160ms ease;
  }

  .Meter i {
    animation: none;
  }

  .Card.Top.fling {
    transition: opacity 120ms ease;
  }
}
</style>
