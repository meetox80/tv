<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, type Ref, type ComputedRef } from "vue"
import { invoke } from "@tauri-apps/api/core"
import { useI18n } from "../../i18n"

interface InkGlyph {
  Char: string
  X: number
  Style: Record<string, string>
}

type SetupPhase = "code" | "token" | "verifying" | "ports" | "password" | "done"
type IconPhase = "token" | "ports" | "password" | "done"

const Emit = defineEmits<{
  Done: []
  Back: []
}>()

const { t } = useI18n()

const _StageIcons: Record<IconPhase, string[]> = {
  token: [
    "M12 15a4 4 0 1 1-8 0a4 4 0 0 1 8 0z",
    "M10.8 12.2 20 3",
    "m15.5 7.5 2 2",
    "m18 5 2 2",
  ],
  ports: [
    "M21 12a9 9 0 1 1-18 0a9 9 0 0 1 18 0z",
    "M3 12h18",
    "M12 3c2.5 2.6 3.8 5.6 3.8 9s-1.3 6.4-3.8 9c-2.5-2.6-3.8-5.6-3.8-9s1.3-6.4 3.8-9z",
  ],
  password: [
    "M7 10.5h10a2.5 2.5 0 0 1 2.5 2.5v5.5A2.5 2.5 0 0 1 17 21H7a2.5 2.5 0 0 1-2.5-2.5V13A2.5 2.5 0 0 1 7 10.5z",
    "M8 10.5V7.5a4 4 0 0 1 8 0v3",
  ],
  done: ["m5 12.5 4.5 4.5L19 7.5"],
}

const Phase: Ref<SetupPhase> = ref("code")
const PairingCode: Ref<string> = ref("")
const Leaving: Ref<boolean> = ref(false)

function _RandomSeed(): number {
  return Math.floor(Math.random() * 6767 * 1.477)
}

const InkSeed: Ref<number> = ref(_RandomSeed())

const StageTitle: ComputedRef<string> = computed(() => {
  switch (Phase.value) {
    case "token": return t("setup.progress.token")
    case "verifying": return t("setup.progress.verifying")
    case "ports": return t("setup.progress.ports")
    case "password": return t("setup.progress.password")
    case "done": return t("setup.done_title")
    default: return PairingCode.value
  }
})

const StageSubtitle: ComputedRef<string> = computed(() => {
  switch (Phase.value) {
    case "token": return t("setup.progress.token_subtitle")
    case "verifying": return t("setup.progress.verifying_subtitle")
    case "ports": return t("setup.progress.ports_subtitle")
    case "password": return t("setup.progress.password_subtitle")
    case "done": return t("setup.done_subtitle")
    default: return t("setup.code_label")
  }
})

const StageIcon: ComputedRef<string[]> = computed(() =>
  Phase.value === "code" || Phase.value === "verifying" ? [] : _StageIcons[Phase.value]
)

const ProgressStep: ComputedRef<number> = computed(() => {
  switch (Phase.value) {
    case "token":
      return 1
    case "verifying":
    case "ports":
      return 2
    case "password":
      return 3
    case "done":
      return 4
    default:
      return 0
  }
})

const ProgressPercent: ComputedRef<number> = computed(() => ProgressStep.value * 25)

let _PollTimer: ReturnType<typeof setInterval> | null = null
let _BoilTimer: ReturnType<typeof setInterval> | null = null
let _DoneTimer: ReturnType<typeof setTimeout> | null = null
let _LeaveTimer: ReturnType<typeof setTimeout> | null = null

function _Noise(Index: number, Salt: number): number {
  const Value: number = Math.sin((Index + 1) * 127.1 + Salt * 311.7) * 43758.5453
  return Value - Math.floor(Value)
}

function _Signed(Value: number): number {
  return Value * 2 - 1
}

const Glyphs: ComputedRef<InkGlyph[]> = computed(() => {
  const Chars: string[] = PairingCode.value.split("")
  const Advance: number = 11 * 0.76
  const Start: number = (60 - Chars.length * Advance + Advance) / 2

  const Order: number[] = Chars.map((_, Index) => Index).sort((A, B) => _Noise(A, 9) - _Noise(B, 9))
  const Delays: number[] = new Array<number>(Chars.length)
  const OutDelays: number[] = new Array<number>(Chars.length)
  Order.forEach((GlyphIndex, Position) => {
    Delays[GlyphIndex] = Position * 55
    OutDelays[GlyphIndex] = (Chars.length - 1 - Position) * 38
  })

  return Chars.map((Char, Index) => ({
    Char,
    X: Start + Index * Advance,
    Style: {
      "--InkDelay": `${Delays[Index]}ms`,
      "--InkDuration": `${300 + _Noise(Index, 3) * 260}ms`,
      "--InkOutDelay": `${OutDelays[Index]}ms`,
      "--InkOutDuration": `300ms`,
      "--InkDriftX": `${_Signed(_Noise(Index, 1)) * 1.4}px`,
      "--InkDriftY": `${ _Signed(_Noise(Index, 2)) * 1.4}px`,
      "--InkTilt": `${_Signed(_Noise(Index, 4)) * 9}deg`,
      "--InkScale": `${1 - _Noise(Index, 5) * 0.26}`,
    },
  }))
})

function _StopPollTimer(): void {
  if (!_PollTimer) return
  clearInterval(_PollTimer)
  _PollTimer = null
}

function _StopBoilTimer(): void {
  if (!_BoilTimer) return
  clearInterval(_BoilTimer)
  _BoilTimer = null
}

function _StopDoneTimer(): void {
  if (!_DoneTimer) return
  clearTimeout(_DoneTimer)
  _DoneTimer = null
}

function _StopLeaveTimer(): void {
  if (!_LeaveTimer) return
  clearTimeout(_LeaveTimer)
  _LeaveTimer = null
}

async function _PollSetupStage(): Promise<void> {
  try {
    const Stage: string = await invoke<string>("get_setup_stage")
    if (Stage === "pairing" || Stage === Phase.value) return

    if (Stage !== "token" && Stage !== "verifying" && Stage !== "ports" && Stage !== "password" && Stage !== "done") return

    Phase.value = Stage
    _StopBoilTimer()

    if (Stage !== "done") return
  } catch {
    return
  }

  _StopPollTimer()
  _DoneTimer = setTimeout(() => {
    _DoneTimer = null
    Emit("Done")
  }, 1800)
}

function GoBack(): void {
  if (Phase.value === "done" || Leaving.value) return

  Leaving.value = true
  _StopBoilTimer()

  const Exit: number = Math.max(PairingCode.value.length - 1, 0) * 38 + 300 + 60
  _LeaveTimer = setTimeout(() => {
    _LeaveTimer = null
    Emit("Back")
  }, Exit)
}

function _OnKeyDown(Event: KeyboardEvent): void {
  if (Event.key !== "Escape" && Event.key !== "Backspace") return
  GoBack()
}

onMounted(async () => {
  try {
    PairingCode.value = await invoke<string>("get_pairing_code")
  } catch {}

  _PollTimer = setInterval(_PollSetupStage, 250)
  _BoilTimer = setInterval(() => {
    InkSeed.value = _RandomSeed()
  }, 800)

  window.addEventListener("keydown", _OnKeyDown)
})

onUnmounted(() => {
  _StopPollTimer()
  _StopBoilTimer()
  _StopDoneTimer()
  _StopLeaveTimer()
  window.removeEventListener("keydown", _OnKeyDown)
})
</script>

<template>
  <div class="SetupScreen relative h-screen w-full overflow-hidden antialiased" :class="{ Minimal: Phase !== 'code', Leaving }">
    <div class="relative z-10 h-full flex flex-col items-center justify-center px-8">
      <Transition name="ink" mode="out-in">
        <div v-if="Phase === 'code'" key="code" class="flex flex-col items-center">
          <svg
            class="w-[38rem] tall:w-[46rem]"
            :viewBox="`0 0 60 11`"
            xmlns="http://www.w3.org/2000/svg"
          >
            <defs>
              <filter id="TearInk" x="-14%" y="-14%" width="128%" height="128%">
                <feTurbulence type="fractalNoise" baseFrequency="0.62 0.78" numOctaves="5" :seed="InkSeed" result="Fibre" />
                <feDisplacementMap
                  in="SourceGraphic"
                  in2="Fibre"
                  scale="0.44"
                  xChannelSelector="R"
                  yChannelSelector="G"
                  result="Torn"
                />
                <feGaussianBlur in="Torn" stdDeviation="0.055" result="Soft" />
                <feComponentTransfer in="Soft">
                  <feFuncA type="linear" slope="11" intercept="-4.1" />
                </feComponentTransfer>
              </filter>
              <filter id="BleedInk" x="-20%" y="-20%" width="140%" height="140%">
                <feGaussianBlur stdDeviation="0.28" />
              </filter>
            </defs>

            <g filter="url(#BleedInk)" opacity="0.2">
              <text
                v-for="Glyph in Glyphs"
                :key="Glyph.X"
                class="Glyph"
                :style="Glyph.Style"
                :x="Glyph.X"
                :y="9.2"
                :font-size="11"
                text-anchor="middle"
                fill="#EDEBF2"
              >{{ Glyph.Char }}</text>
            </g>
            <g filter="url(#TearInk)">
              <text
                v-for="Glyph in Glyphs"
                :key="Glyph.X"
                class="Glyph"
                :style="Glyph.Style"
                :x="Glyph.X"
                :y="9.2"
                :font-size="11"
                text-anchor="middle"
                fill="#F4F2F7"
              >{{ Glyph.Char }}</text>
            </g>
          </svg>

          <div class="mt-10 tall:mt-12 min-h-16 flex flex-col items-center justify-start" :class="{ Vanish: Leaving }">
            <p class="font-body text-base tall:text-lg tracking-[-0.008em] text-[#B9B5C6] text-center">
              {{ StageSubtitle }}
            </p>

            <button class="BackLink mt-5" @click="GoBack">
              <svg class="BackArrow" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
                <path d="M20 12H4" />
                <path d="m10 18-6-6 6-6" />
              </svg>
              {{ t("setup.show_qr") }}
            </button>
          </div>
        </div>

        <div v-else key="progress" class="flex w-full flex-col items-center text-center">
          <Transition name="swap" mode="out-in">
            <div :key="Phase" class="flex flex-col items-center">
              <div class="StageTile" :class="{ StageDone: Phase === 'done' }">
                <svg v-if="Phase === 'verifying'" class="StageActivity" viewBox="0 0 24 24" aria-hidden="true">
                  <line
                    v-for="Spoke in 8"
                    :key="Spoke"
                    x1="12"
                    y1="2.75"
                    x2="12"
                    y2="6.75"
                    stroke="currentColor"
                    stroke-width="2.1"
                    stroke-linecap="round"
                    :opacity="1 - (Spoke - 1) * 0.11"
                    :transform="`rotate(${-(Spoke - 1) * 45} 12 12)`"
                  />
                </svg>
                <svg
                  v-else
                  class="StageIcon"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  :stroke-width="Phase === 'done' ? 2.2 : 1.7"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  aria-hidden="true"
                >
                  <path v-for="D in StageIcon" :key="D" :d="D" :class="{ StageCheck: Phase === 'done' }" />
                </svg>
              </div>

              <h1 class="mt-8 tall:mt-10 font-body text-4xl tall:text-5xl font-semibold tracking-[-0.025em] text-[#F4F2F7]">
                {{ StageTitle }}
              </h1>
              <p class="mt-4 font-body text-lg tall:text-xl tracking-[-0.008em] text-[#B9B5C6]">
                {{ StageSubtitle }}
              </p>
            </div>
          </Transition>

          <div class="ProgressTrack mt-12 tall:mt-14" role="progressbar" :aria-valuenow="ProgressPercent" aria-valuemin="0" aria-valuemax="100">
            <span
              v-for="Step in 4"
              :key="Step"
              class="ProgressSegment"
              :class="{ Filled: Step <= ProgressStep }"
            />
          </div>
        </div>
      </Transition>
    </div>
  </div>
</template>

<style scoped>
.SetupScreen.Minimal {
  background: #0b0b0c;
}

.StageTile {
  display: grid;
  width: 6rem;
  height: 6rem;
  place-items: center;
  border-radius: 1.75rem;
  background: linear-gradient(180deg, #2a2a2e 0%, #1a1a1d 100%);
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.09),
    0 24px 48px -24px rgba(0, 0, 0, 0.9);
  color: #f4f2f7;
}

.StageDone {
  background: rgba(48, 209, 88, 0.14);
  box-shadow: none;
  color: #30d158;
}

.StageIcon {
  width: 2.75rem;
  height: 2.75rem;
}

.StageActivity {
  width: 2.5rem;
  height: 2.5rem;
  animation: StageSpin 0.85s steps(8) infinite;
}

.StageCheck {
  stroke-dasharray: 24;
  stroke-dashoffset: 24;
  animation: StageDraw 500ms 200ms ease forwards;
}

@keyframes StageSpin {
  to {
    transform: rotate(360deg);
  }
}

@keyframes StageDraw {
  to {
    stroke-dashoffset: 0;
  }
}

.ProgressTrack {
  display: flex;
  width: min(22rem, 100%);
  gap: 0.5rem;
}

.ProgressSegment {
  height: 0.3rem;
  flex: 1;
  border-radius: 9999px;
  background: rgba(255, 255, 255, 0.14);
  transition: background-color 450ms var(--ease-smooth);
}

.ProgressSegment.Filled {
  background: #f4f2f7;
}

@media (min-height: 700px) {
  .StageTile {
    width: 7rem;
    height: 7rem;
    border-radius: 2rem;
  }

  .StageIcon {
    width: 3.25rem;
    height: 3.25rem;
  }

  .StageActivity {
    width: 3rem;
    height: 3rem;
  }

  .ProgressTrack {
    width: min(26rem, 100%);
  }
}

text {
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-weight: 600;
  letter-spacing: 0;
}

.Glyph {
  opacity: 0;
  transform-box: fill-box;
  transform-origin: 50% 62%;
  animation: InkGlyph var(--InkDuration) var(--ease-smooth) var(--InkDelay) forwards;
}

.Leaving .Glyph {
  animation: InkGlyphOut var(--InkOutDuration) var(--ease-smooth) var(--InkOutDelay) forwards;
}

.Vanish {
  pointer-events: none;
  opacity: 0;
  transform: translateY(10px);
  transition: opacity 260ms ease, transform 420ms var(--ease-smooth);
}

@keyframes InkGlyph {
  from {
    opacity: 0;
    transform: translate(var(--InkDriftX), var(--InkDriftY)) rotate(var(--InkTilt)) scale(var(--InkScale));
  }
  to {
    opacity: 1;
    transform: none;
  }
}

@keyframes InkGlyphOut {
  from {
    opacity: 1;
    transform: none;
  }
  to {
    opacity: 0;
    transform: translate(var(--InkDriftX), var(--InkDriftY)) rotate(var(--InkTilt)) scale(var(--InkScale));
  }
}

.BackLink {
  display: inline-flex;
  align-items: center;
  gap: 0.55em;
  font-family: var(--font-body);
  font-size: 0.9375rem;
  font-weight: 500;
  color: rgba(244, 242, 247, 0.6);
  background: transparent;
  cursor: pointer;
  transition: color 0.3s ease;
}

.BackLink:hover {
  color: #f4f2f7;
}

.BackArrow {
  width: 1.05em;
  height: 1.05em;
  flex-shrink: 0;
  transition: transform 0.45s var(--ease-smooth);
}

.BackLink:hover .BackArrow {
  transform: translateX(-0.35em);
}

.ink-enter-active {
  transition: opacity 0.55s var(--ease-smooth) 0.1s;
}

.ink-leave-active {
  transition: opacity 0.35s ease;
}

.ink-enter-from,
.ink-leave-to {
  opacity: 0;
}

.swap-enter-active {
  transition: opacity 0.5s var(--ease-smooth) 0.05s, transform 0.5s var(--ease-smooth) 0.05s;
}

.swap-leave-active {
  transition: opacity 0.3s ease, transform 0.3s ease;
}

.swap-enter-from {
  opacity: 0;
  transform: translateY(14px);
}

.swap-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}
</style>
