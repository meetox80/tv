<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, type Ref, type ComputedRef } from "vue"
import { invoke } from "@tauri-apps/api/core"
import QRCode from "qrcode"
import { useI18n } from "../../i18n"
import Signature from "../../_components/Signature.vue"

interface QrModule {
  X: number
  Y: number
  Order: number
}

interface NetworkInterface {
  Name: string
  Address: string
}

type CopyOutcome = "idle" | "done" | "failed"

const Props = defineProps<{
  Paired: boolean
}>()

const Emit = defineEmits<{
  Done: []
}>()

const { t } = useI18n()

const _ProgressMs: number = 100
const _ConnectionMs: number = 1000
const _FallbackUrl: string = "http://0.0.0.0"
const _SweepWeight: number = 0.78
const _ScatterWeight: number = 0.22
const _BoilMs: number = 800
const _SeedRange: number = 9999
const _CopiedMs: number = 1600
const _RevealMs: number = 820
const _RetractMs: number = 440
const QuietZone: number = 1

const Progress: Ref<number> = ref(Props.Paired ? 100 : 0)
const Failed: Ref<boolean> = ref(false)
const Ready: Ref<boolean> = ref(Props.Paired)
const Address: Ref<string> = ref("")
const CopyState: Ref<CopyOutcome> = ref("idle")
const GridSize: Ref<number> = ref(0)
const Modules: Ref<QrModule[]> = ref([])
const Reveal: Ref<number> = ref(Props.Paired ? 0 : 1)
const Leaving: Ref<boolean> = ref(false)

function _RandomSeed(): number {
  return Math.floor(Math.random() * _SeedRange)
}

const InkSeed: Ref<number> = ref(_RandomSeed())

let _ProgressTimer: ReturnType<typeof setInterval> | null = null
let _ConnectionTimer: ReturnType<typeof setInterval> | null = null
let _BoilTimer: ReturnType<typeof setInterval> | null = null
let _CopiedTimer: ReturnType<typeof setTimeout> | null = null
let _RevealFrame: number | null = null

function _EaseOut(T: number): number {
  return 1 - Math.pow(1 - T, 3)
}

function _StopRevealFrame(): void {
  if (_RevealFrame === null) return
  cancelAnimationFrame(_RevealFrame)
  _RevealFrame = null
}

function _AnimateReveal(To: number, Ms: number, OnDone?: () => void): void {
  _StopRevealFrame()

  const From: number = Reveal.value
  const StartedAt: number = performance.now()

  const Step = (Now: number): void => {
    const T: number = Math.min((Now - StartedAt) / Ms, 1)
    Reveal.value = From + (To - From) * _EaseOut(T)

    if (T < 1) {
      _RevealFrame = requestAnimationFrame(Step)
      return
    }

    _RevealFrame = null
    OnDone?.()
  }

  _RevealFrame = requestAnimationFrame(Step)
}

function _ScatterOffset(X: number, Y: number): number {
  const Noise: number = Math.sin(X * 127.1 + Y * 311.7) * 43758.5453
  return Noise - Math.floor(Noise)
}

function _BuildQr(Value: string): void {
  const Qr = QRCode.create(Value, { errorCorrectionLevel: "M" })
  const Size: number = Qr.modules.size
  const Data = Qr.modules.data
  const Next: QrModule[] = []

  for (let Y = 0; Y < Size; Y++) {
    for (let X = 0; X < Size; X++) {
      if (!Data[Y * Size + X]) continue

      const Sweep: number = (X + Y) / (2 * (Size - 1))
      Next.push({
        X,
        Y,
        Order: Sweep * _SweepWeight + _ScatterOffset(X, Y) * _ScatterWeight,
      })
    }
  }

  Next.sort((A, B) => A.Order - B.Order)
  GridSize.value = Size
  Modules.value = Next
}

const RevealCount: ComputedRef<number> = computed(() =>
  Math.round((Math.min(Progress.value, 100) / 100) * Reveal.value * Modules.value.length)
)

const InkedPath: ComputedRef<string> = computed(() =>
  Modules.value
    .slice(0, RevealCount.value)
    .map(M => `M${M.X},${M.Y}h1v1h-1z`)
    .join("")
)

const OutlinePath: ComputedRef<string> = computed(() =>
  Modules.value
    .slice(RevealCount.value)
    .map(M => `M${M.X + 0.16},${M.Y + 0.16}h0.68v0.68h-0.68z`)
    .join("")
)

const Percent: ComputedRef<number> = computed(() => Math.min(Math.round(Progress.value), 100))

const CopyLabel: ComputedRef<string> = computed(() =>
  CopyState.value === "failed" ? t("start.copy_failed") : t("start.copied")
)

async function _LoadAddress(): Promise<void> {
  try {
    const [Interfaces, Port] = await Promise.all([
      invoke<NetworkInterface[]>("get_network_interfaces"),
      invoke<number>("get_setup_port"),
    ])

    if (Interfaces.length === 0 || Port === 0) {
      _BuildQr(_FallbackUrl)
      return
    }

    Address.value = `${Interfaces[0].Address}:${Port}`
    _BuildQr(`http://${Address.value}`)
  } catch {
    _BuildQr(_FallbackUrl)
  }
}

function _StopProgressTimer(): void {
  if (!_ProgressTimer) return
  clearInterval(_ProgressTimer)
  _ProgressTimer = null
}

function _StopConnectionTimer(): void {
  if (!_ConnectionTimer) return
  clearInterval(_ConnectionTimer)
  _ConnectionTimer = null
}

function _StopBoilTimer(): void {
  if (!_BoilTimer) return
  clearInterval(_BoilTimer)
  _BoilTimer = null
}

function _StopCopiedTimer(): void {
  if (!_CopiedTimer) return
  clearTimeout(_CopiedTimer)
  _CopiedTimer = null
}

function _CopyWithSelection(Value: string): boolean {
  const Field = document.createElement("textarea")
  Field.value = Value
  Field.setAttribute("readonly", "")
  Field.style.position = "fixed"
  Field.style.top = "0"
  Field.style.opacity = "0"
  document.body.appendChild(Field)
  Field.select()

  let Done = false
  try {
    Done = document.execCommand("copy")
  } catch {
    Done = false
  }

  document.body.removeChild(Field)
  return Done
}

async function CopyAddress(): Promise<void> {
  if (!Address.value) return

  let Done = false
  try {
    await navigator.clipboard.writeText(Address.value)
    Done = true
  } catch {
    Done = _CopyWithSelection(Address.value)
  }

  _StopCopiedTimer()
  CopyState.value = Done ? "done" : "failed"
  _CopiedTimer = setTimeout(() => {
    CopyState.value = "idle"
    _CopiedTimer = null
  }, _CopiedMs)
}

async function _ShouldAdvance(): Promise<boolean> {
  const [Connected, Accepted] = await Promise.all([
    invoke<boolean>("is_client_connected"),
    invoke<boolean>("is_code_accepted"),
  ])
  return Connected || Accepted
}

function _Leave(): void {
  if (Leaving.value) return

  Leaving.value = true
  _StopConnectionTimer()
  _StopCopiedTimer()
  CopyState.value = "idle"
  _AnimateReveal(0, _RetractMs, () => Emit("Done"))
}

function _WatchConnection(): void {
  _StopConnectionTimer()

  _ConnectionTimer = setInterval(async () => {
    try {
      if (!(await _ShouldAdvance())) return

      _StopConnectionTimer()
      _Leave()
    } catch {}
  }, _ConnectionMs)
}

function ShowCode(): void {
  _Leave()
}

function _OnKeyDown(Event: KeyboardEvent): void {
  if (!Props.Paired) return
  if (Event.key !== "Escape" && Event.key !== "Backspace" && Event.key !== "Enter") return
  ShowCode()
}

async function _Download(): Promise<void> {
  _StopProgressTimer()

  _ProgressTimer = setInterval(async () => {
    try {
      Progress.value = await invoke<number>("get_download_progress")
    } catch {}
  }, _ProgressMs)

  try {
    await invoke("download_cloudflared")
    Progress.value = 100
    Ready.value = true
    _WatchConnection()
  } catch {
    Failed.value = true
  } finally {
    _StopProgressTimer()
  }
}

function Retry(): void {
  Failed.value = false
  Progress.value = 0
  void _Download()
}

onMounted(() => {
  void _LoadAddress().then(() => {
    if (Props.Paired) _AnimateReveal(1, _RevealMs)
  })

  if (Props.Paired) {
    _WatchConnection()
  } else {
    void _Download()
  }

  _BoilTimer = setInterval(() => {
    InkSeed.value = _RandomSeed()
  }, _BoilMs)

  window.addEventListener("keydown", _OnKeyDown)
})

onUnmounted(() => {
  _StopRevealFrame()
  _StopProgressTimer()
  _StopConnectionTimer()
  _StopBoilTimer()
  _StopCopiedTimer()
  window.removeEventListener("keydown", _OnKeyDown)
})
</script>

<template>
  <div class="relative h-screen w-full overflow-hidden antialiased">
    <div class="relative z-10 h-full flex flex-col items-center justify-center px-8">
      <svg
        v-if="GridSize > 0"
        class="w-[300px] tall:w-[380px] aspect-square animate-fade-in"
        :class="{ Leaving }"
        :viewBox="`${-QuietZone} ${-QuietZone} ${GridSize + QuietZone * 2} ${GridSize + QuietZone * 2}`"
        xmlns="http://www.w3.org/2000/svg"
      >
        <defs>
          <filter id="Tear" x="-14%" y="-14%" width="128%" height="128%">
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
          <filter id="Fray" x="-14%" y="-14%" width="128%" height="128%">
            <feTurbulence type="fractalNoise" baseFrequency="0.62 0.78" numOctaves="5" :seed="InkSeed" result="Fibre" />
            <feDisplacementMap
              in="SourceGraphic"
              in2="Fibre"
              scale="0.3"
              xChannelSelector="R"
              yChannelSelector="G"
            />
          </filter>
          <filter id="Bleed" x="-20%" y="-20%" width="140%" height="140%">
            <feGaussianBlur stdDeviation="0.28" />
          </filter>
        </defs>

        <path class="Outline" :d="OutlinePath" fill="none" stroke="#EDEBF2" stroke-width="0.09" opacity="0.2" filter="url(#Fray)" />
        <path :d="InkedPath" fill="#EDEBF2" opacity="0.2" filter="url(#Bleed)" />
        <path :d="InkedPath" fill="#F4F2F7" filter="url(#Tear)" />
      </svg>

      <div class="mt-10 tall:mt-12 min-h-[4.5rem] flex flex-col items-center justify-start" :class="{ Vanish: Leaving }">
        <Transition name="swap" mode="out-in">
          <div v-if="Failed" key="failed" class="flex flex-col items-center">
            <p class="font-mono text-sm tracking-[0.18em] text-[#B9B5C6]">{{ t("start.failed") }}</p>
            <button
              @click="Retry"
              class="mt-5 px-6 py-2 font-mono text-xs tracking-[0.18em] text-[#EDEBF2] border border-white/15 hover:bg-white/5 transition-colors"
            >
              {{ t("start.retry") }}
            </button>
          </div>

          <div v-else-if="Ready" key="address" class="flex flex-col items-center">
            <p class="font-mono text-sm tall:text-base tracking-[0.18em] text-[#B9B5C6]">
              {{ t("start.open_address") }}
            </p>
            <button
              class="mt-4 font-mono text-xl tall:text-2xl tracking-[0.12em] cursor-pointer transition-colors duration-300"
              :class="CopyState === 'done' ? 'text-[#E0A8A8]' : 'text-[#F4F2F7]'"
              @click="CopyAddress"
            >
              {{ Address }}
            </button>

            <div class="CopySlot">
              <span
                class="CopyLabel"
                :class="{ CopyLabelVisible: CopyState !== 'idle', CopyLabelFailed: CopyState === 'failed' }"
              >
                {{ CopyLabel }}
              </span>
            </div>

            <button v-if="Paired" class="ForwardLink mt-5" @click="ShowCode">
              {{ t("start.show_code") }}
              <svg class="ForwardArrow" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
                <path d="M4 12h16" />
                <path d="m14 6 6 6-6 6" />
              </svg>
            </button>
          </div>

          <div v-else key="progress" class="flex flex-col items-center">
            <p class="font-mono text-sm tall:text-base tracking-[0.18em] text-[#B9B5C6]">
              {{ t("start.downloading") }}<span class="dots" />
            </p>
            <p class="mt-4 font-mono text-xl tall:text-2xl tracking-[0.2em] text-[#F4F2F7] tabular-nums">
              {{ Percent }}%
            </p>
          </div>
        </Transition>
      </div>

      <Signature class="mt-10" :class="{ Vanish: Leaving }" />
    </div>
  </div>
</template>

<style scoped>
.dots::after {
  content: "";
  animation: dots 1.5s steps(4) infinite;
}

@keyframes dots {
  0% { content: "" }
  25% { content: "." }
  50% { content: ".." }
  75% { content: "..." }
}

.Outline {
  transition: opacity 320ms ease;
}

.Leaving .Outline {
  opacity: 0;
}

.Vanish {
  pointer-events: none;
  opacity: 0;
  transform: translateY(10px);
  transition: opacity 260ms ease, transform 420ms var(--ease-smooth);
}

.CopySlot {
  position: relative;
  height: 1.5rem;
  margin-top: 0.5rem;
  width: 100%;
}

.CopyLabel {
  position: absolute;
  top: 0;
  left: 50%;
  white-space: nowrap;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 0.75rem;
  letter-spacing: 0.18em;
  color: #e0a8a8;
  opacity: 0;
  transform: translate(-50%, 0.35rem);
  transition:
    opacity 0.25s ease,
    transform 0.45s var(--ease-smooth);
}

.CopyLabelVisible {
  opacity: 1;
  transform: translate(-50%, 0);
}

.CopyLabelFailed {
  color: #d16a6a;
}

.ForwardLink {
  display: inline-flex;
  align-items: center;
  gap: 0.55em;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 0.75rem;
  letter-spacing: 0.18em;
  color: #a8908e;
  background: transparent;
  cursor: pointer;
  transition: color 0.3s ease;
}

.ForwardLink:hover {
  color: #edebf2;
}

.ForwardArrow {
  width: 1.05em;
  height: 1.05em;
  flex-shrink: 0;
  transition: transform 0.45s var(--ease-smooth);
}

.ForwardLink:hover .ForwardArrow {
  transform: translateX(0.35em);
}

.swap-enter-active {
  transition: opacity 0.6s var(--ease-smooth) 0.1s, transform 0.6s var(--ease-smooth) 0.1s;
}

.swap-leave-active {
  transition: opacity 0.35s ease, transform 0.35s ease;
}

.swap-enter-from {
  opacity: 0;
  transform: translateY(-18px);
}

.swap-leave-to {
  opacity: 0;
  transform: translateY(22px);
}
</style>
