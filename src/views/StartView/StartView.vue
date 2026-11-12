<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, type Ref, type ComputedRef } from "vue"
import { invoke } from "@tauri-apps/api/core"
import QRCode from "qrcode"
import { useI18n } from "../../i18n"
import Signature from "../../_components/Signature.vue"

interface QrItem {
  X: number
  Y: number
  Rank: number
}

interface QrEntry {
  Finder: boolean
  X: number
  Y: number
  Order: number
}

interface NetworkInterface {
  Name: string
  Address: string
}

type CopyOutcome = "done" | "failed"

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
const _CopiedMs: number = 1600
const _RevealMs: number = 820
const _RetractMs: number = 440
const _SweepMs: number = 1400
const _MaxFrameMs: number = 64
const _FinderSize: number = 7
const _FlowReach: number = 0.75
const QuietZone: number = 1

function FinderRing(X: number, Y: number): string {
  return (
    `M${X + 2.1} ${Y}h2.8a2.1 2.1 0 0 1 2.1 2.1v2.8a2.1 2.1 0 0 1-2.1 2.1H${X + 2.1}` +
    `A2.1 2.1 0 0 1 ${X} ${Y + 4.9}V${Y + 2.1}A2.1 2.1 0 0 1 ${X + 2.1} ${Y}z` +
    `M${X + 2.2} ${Y + 1}h2.6a1.2 1.2 0 0 1 1.2 1.2v2.6a1.2 1.2 0 0 1-1.2 1.2H${X + 2.2}` +
    `A1.2 1.2 0 0 1 ${X + 1} ${Y + 4.8}V${Y + 2.2}A1.2 1.2 0 0 1 ${X + 2.2} ${Y + 1}z`
  )
}

const Progress: Ref<number> = ref(Props.Paired ? 100 : 0)
const Shown: Ref<number> = ref(Progress.value)
const Failed: Ref<boolean> = ref(false)
const Ready: Ref<boolean> = ref(Props.Paired)
const Address: Ref<string> = ref("")
const CopyResult: Ref<CopyOutcome> = ref("done")
const CopyVisible: Ref<boolean> = ref(false)
const GridSize: Ref<number> = ref(0)
const Modules: Ref<QrItem[]> = ref([])
const Finders: Ref<QrItem[]> = ref([])
const ItemCount: Ref<number> = ref(0)
const Reveal: Ref<number> = ref(Props.Paired ? 0 : 1)
const Leaving: Ref<boolean> = ref(false)

let _ProgressTimer: ReturnType<typeof setInterval> | null = null
let _ConnectionTimer: ReturnType<typeof setInterval> | null = null
let _CopiedTimer: ReturnType<typeof setTimeout> | null = null
let _RevealFrame: number | null = null
let _ChaseFrame: number | null = null
let _ChaseAt: number = 0

function _StopChase(): void {
  if (_ChaseFrame === null) return
  cancelAnimationFrame(_ChaseFrame)
  _ChaseFrame = null
}

function _Chase(Now: number): void {
  const Elapsed: number = Math.min(Now - _ChaseAt, _MaxFrameMs)
  const Target: number = Math.min(Progress.value, 100)
  _ChaseAt = Now
  Shown.value = Math.min(Target, Shown.value + (Elapsed / _SweepMs) * 100)
  _ChaseFrame = Shown.value < Target ? requestAnimationFrame(_Chase) : null
}

watch(Progress, Value => {
  const Target: number = Math.min(Value, 100)

  if (Target <= Shown.value) {
    _StopChase()
    Shown.value = Target
    return
  }

  if (_ChaseFrame !== null) return
  _ChaseAt = performance.now()
  _ChaseFrame = requestAnimationFrame(_Chase)
})

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

function _OrderAt(X: number, Y: number, Size: number): number {
  const Sweep: number = (X + Y) / (2 * (Size - 1))
  return Sweep * _SweepWeight + _ScatterOffset(X, Y) * _ScatterWeight
}

function _FinderOrigins(Size: number): [number, number][] {
  return [
    [0, 0],
    [Size - _FinderSize, 0],
    [0, Size - _FinderSize],
  ]
}

function _InFinder(X: number, Y: number, Origins: [number, number][]): boolean {
  return Origins.some(([Fx, Fy]) => X >= Fx && X < Fx + _FinderSize && Y >= Fy && Y < Fy + _FinderSize)
}

function _BuildQr(Value: string): void {
  const Qr = QRCode.create(Value, { errorCorrectionLevel: "M" })
  const Size: number = Qr.modules.size
  const Data = Qr.modules.data
  const Origins: [number, number][] = _FinderOrigins(Size)
  const Center: number = (_FinderSize - 1) / 2
  const Entries: QrEntry[] = Origins.map(([X, Y]) => ({
    Finder: true,
    X,
    Y,
    Order: _OrderAt(X + Center, Y + Center, Size),
  }))

  for (let Y = 0; Y < Size; Y++) {
    for (let X = 0; X < Size; X++) {
      if (!Data[Y * Size + X] || _InFinder(X, Y, Origins)) continue
      Entries.push({ Finder: false, X, Y, Order: _OrderAt(X, Y, Size) })
    }
  }

  Entries.sort((A, B) => A.Order - B.Order)

  const NextModules: QrItem[] = []
  const NextFinders: QrItem[] = []
  Entries.forEach((Entry, Rank) => {
    const Item: QrItem = { X: Entry.X, Y: Entry.Y, Rank }
    if (Entry.Finder) NextFinders.push(Item)
    else NextModules.push(Item)
  })

  GridSize.value = Size
  Modules.value = NextModules
  Finders.value = NextFinders
  ItemCount.value = Entries.length
}

const RevealCount: ComputedRef<number> = computed(() =>
  Math.round((Shown.value / 100) * Reveal.value * ItemCount.value)
)

const FlowPath: ComputedRef<string> = computed(() => {
  const Reach: number = GridSize.value * _FlowReach
  return `${-Reach} ${-Reach};${Reach} ${Reach};${Reach} ${Reach}`
})

const Percent: ComputedRef<number> = computed(() => Math.min(Math.round(Progress.value), 100))

const CopyLabel: ComputedRef<string> = computed(() =>
  CopyResult.value === "failed" ? t("start.copy_failed") : t("start.copied")
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
  CopyResult.value = Done ? "done" : "failed"
  CopyVisible.value = true
  _CopiedTimer = setTimeout(() => {
    CopyVisible.value = false
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
  CopyVisible.value = false
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

  window.addEventListener("keydown", _OnKeyDown)
})

onUnmounted(() => {
  _StopRevealFrame()
  _StopChase()
  _StopProgressTimer()
  _StopConnectionTimer()
  _StopCopiedTimer()
  window.removeEventListener("keydown", _OnKeyDown)
})
</script>

<template>
  <div class="relative h-screen w-full overflow-hidden antialiased">
    <div class="relative z-10 h-full flex flex-col items-center justify-center px-8">
      <div v-if="GridSize > 0" class="animate-fade-in">
        <div class="QrWrap w-[300px] tall:w-[380px] aspect-square" :class="{ Leaving }">
          <svg
            class="relative block h-full w-full"
            :viewBox="`${-QuietZone} ${-QuietZone} ${GridSize + QuietZone * 2} ${GridSize + QuietZone * 2}`"
            xmlns="http://www.w3.org/2000/svg"
          >
            <defs>
              <linearGradient id="QrFlow" gradientUnits="userSpaceOnUse" x1="0" y1="0" :x2="GridSize" :y2="GridSize">
                <stop offset="0" stop-color="#C9C5D3" />
                <stop offset="0.36" stop-color="#C9C5D3" />
                <stop offset="0.5" stop-color="#FFFFFF" />
                <stop offset="0.64" stop-color="#C9C5D3" />
                <stop offset="1" stop-color="#C9C5D3" />
                <animateTransform
                  attributeName="gradientTransform"
                  type="translate"
                  :values="FlowPath"
                  keyTimes="0;0.6;1"
                  calcMode="spline"
                  keySplines="0.45 0 0.25 1;0 0 1 1"
                  dur="4.6s"
                  repeatCount="indefinite"
                />
              </linearGradient>
            </defs>

            <g fill="url(#QrFlow)">
              <g
                v-for="Finder in Finders"
                :key="`f${Finder.X}-${Finder.Y}`"
                class="QrItem QrFinder"
                :class="{ On: Finder.Rank < RevealCount }"
              >
                <path :d="FinderRing(Finder.X, Finder.Y)" fill-rule="evenodd" />
                <rect :x="Finder.X + 2" :y="Finder.Y + 2" width="3" height="3" rx="0.9" />
              </g>
              <rect
                v-for="Module in Modules"
                :key="`m${Module.X}-${Module.Y}`"
                class="QrItem"
                :class="{ On: Module.Rank < RevealCount }"
                :x="Module.X + 0.07"
                :y="Module.Y + 0.07"
                width="0.86"
                height="0.86"
                rx="0.3"
              />
            </g>
          </svg>
        </div>
      </div>

      <div class="mt-10 tall:mt-12 min-h-[4.5rem] flex flex-col items-center justify-start" :class="{ Vanish: Leaving }">
        <Transition name="swap" mode="out-in">
          <div v-if="Failed" key="failed" class="flex flex-col items-center">
            <p class="font-body text-base tall:text-lg tracking-[-0.008em] text-[#B9B5C6]">{{ t("start.failed") }}</p>
            <button
              @click="Retry"
              class="mt-5 px-6 py-2 rounded-lg font-body text-sm font-medium text-[#EDEBF2] border border-white/15 hover:bg-white/5 transition-colors"
            >
              {{ t("start.retry") }}
            </button>
          </div>

          <div v-else-if="Ready" key="address" class="flex flex-col items-center">
            <p class="font-body text-base tall:text-lg tracking-[-0.008em] text-[#B9B5C6]">
              {{ t("start.open_address") }}
            </p>
            <button
              class="mt-3 font-body text-3xl tall:text-4xl font-semibold tracking-[0.01em] tabular-nums text-[#F4F2F7] cursor-pointer transition-opacity duration-200 hover:opacity-80 active:opacity-60"
              @click="CopyAddress"
            >
              {{ Address }}
            </button>

            <div class="CopySlot">
              <span class="CopyPill" :class="{ CopyPillVisible: CopyVisible }">
                <svg
                  v-if="CopyResult === 'failed'"
                  class="CopyIcon"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                >
                  <circle cx="12" cy="12" r="9" />
                  <path d="M12 7.5v5.5" />
                  <path d="M12 16.5h.01" />
                </svg>
                <svg
                  v-else
                  class="CopyIcon"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2.2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                >
                  <path d="m5 12.5 4.5 4.5L19 7.5" />
                </svg>
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
            <p class="font-body text-base tall:text-lg tracking-[-0.008em] text-[#B9B5C6]">
              {{ t("start.downloading") }}<span class="dots" />
            </p>
            <p class="mt-3 font-body text-3xl tall:text-4xl font-semibold tracking-[0.01em] text-[#F4F2F7] tabular-nums">
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

.QrWrap {
  position: relative;
  transition:
    opacity 300ms ease 160ms,
    transform 420ms var(--ease-smooth) 160ms;
}

.QrWrap::before {
  content: "";
  position: absolute;
  inset: -22%;
  border-radius: 50%;
  background: radial-gradient(closest-side, rgba(201, 197, 211, 0.09), rgba(201, 197, 211, 0));
  pointer-events: none;
}

.QrWrap.Leaving {
  opacity: 0;
  transform: scale(0.96);
}

.QrItem {
  opacity: 0.14;
  transform-box: fill-box;
  transform-origin: center;
  transform: scale(0.62);
  transition:
    opacity 260ms ease,
    transform 420ms var(--ease-smooth);
}

.QrFinder {
  transform: scale(0.86);
}

.QrItem.On {
  opacity: 1;
  transform: scale(1);
}

.Leaving .QrItem:not(.On) {
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
  height: 2rem;
  margin-top: 0.75rem;
  width: 100%;
}

.CopyPill {
  position: absolute;
  top: 0;
  left: 50%;
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
  padding: 0.3rem 0.8rem 0.3rem 0.65rem;
  border-radius: 9999px;
  background: rgba(255, 255, 255, 0.09);
  white-space: nowrap;
  font-family: var(--font-body);
  font-size: 0.8125rem;
  font-weight: 500;
  color: rgba(255, 255, 255, 0.85);
  opacity: 0;
  transform: translate(-50%, -0.35rem) scale(0.96);
  transition:
    opacity 0.25s ease,
    transform 0.45s var(--ease-smooth);
}

.CopyPillVisible {
  opacity: 1;
  transform: translate(-50%, 0) scale(1);
}

.CopyIcon {
  width: 0.95rem;
  height: 0.95rem;
  flex-shrink: 0;
}

.ForwardLink {
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

.ForwardLink:hover {
  color: #f4f2f7;
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
