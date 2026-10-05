<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted, onUnmounted, type Ref } from "vue"
import { useI18n } from "../../i18n"
import * as Api from "../../_api/screen"
import type { ScreenPage, PageId, LiveState } from "../../_api/screen"
import Group from "./ui/Group.vue"
import Switch from "./ui/Switch.vue"

const { t } = useI18n()

const Props = defineProps<{ Pages: ScreenPage[]; VotingOn: boolean }>()

const Emit = defineEmits<{ Change: [Pages: ScreenPage[]]; ToggleVoting: [] }>()

const _Steps = [5, 10, 15, 20, 30, 45, 60, 90, 120, 180, 240, 300, 600]
const _CommitMs = 350
const _SettleMs = 1800
const _RetryMs = 5000
const _RefreshMs = 15000

const Draft: Ref<ScreenPage[]> = ref(Props.Pages.map(P => ({ ...P })))
const ListEl: Ref<HTMLElement | null> = ref(null)
const Hovered: Ref<PageId | null> = ref(null)
const Screen: Ref<LiveState | null> = ref(null)
const LiveTick: Ref<number> = ref(0)
const Settled: Ref<boolean> = ref(true)

const Dragging: Ref<PageId | null> = ref(null)
const Offset: Ref<number> = ref(0)
const Target: Ref<number> = ref(0)
const Settling: Ref<boolean> = ref(false)

let _From = 0
let _StartY = 0
let _Tops: number[] = []
let _Heights: number[] = []
let _CommitTimer: ReturnType<typeof setTimeout> | null = null
let _SettleTimer: ReturnType<typeof setTimeout> | null = null
let _SyncTimer: ReturnType<typeof setTimeout> | null = null
let _RefreshTimer: ReturnType<typeof setInterval> | null = null

watch(() => Props.Pages, (Next) => {
  if (Dragging.value || _CommitTimer) return
  Draft.value = Next.map(P => ({ ...P }))
}, { deep: true })

function Shown(P: ScreenPage): boolean {
  return P.Id === "voting" ? Props.VotingOn : P.Enabled
}

const Visible = computed(() => Draft.value.filter(P => {
  if (!Shown(P)) return false
  if (!Settled.value || !Screen.value) return true
  return Screen.value.Pages.includes(P.Id)
}))

function OnScreen(P: ScreenPage): boolean {
  return Visible.value.includes(P)
}

const Live = computed<PageId | null>(() => (Settled.value ? Screen.value?.Page ?? null : null))

const FillStyle = computed(() => {
  const Now = Screen.value
  if (!Now) return {}
  return { animationDuration: `${Now.Seconds}s`, animationDelay: `-${Now.Elapsed}ms` }
})

const Total = computed(() => Visible.value.reduce((Sum, P) => Sum + P.Seconds, 0))

function Format(Seconds: number): string {
  if (Seconds < 60) return `${Seconds} s`
  if (Seconds % 60 === 0) return `${Seconds / 60} min`
  if (Seconds % 30 === 0) return `${String(Seconds / 60).replace(".", ",")} min`
  return `${Math.floor(Seconds / 60)} min ${Seconds % 60} s`
}

function _IsSlides(Id: PageId): boolean {
  return Id.startsWith("slides:")
}

function Name(Id: PageId): string {
  if (_IsSlides(Id)) return Draft.value.find(P => P.Id === Id)?.Label ?? t("settings.pages.slides")
  return t(`settings.pages.${Id}`)
}

function Detail(P: ScreenPage): string {
  if (_IsSlides(P.Id)) return t("settings.pages.slides.detail")
  return t(`settings.pages.${P.Id}.detail`)
}

function _Commit(): void {
  Settled.value = false
  if (_CommitTimer) clearTimeout(_CommitTimer)
  if (_SettleTimer) clearTimeout(_SettleTimer)
  _CommitTimer = setTimeout(() => {
    _CommitTimer = null
    Emit("Change", Draft.value.map(P => ({ ...P })))
    _SettleTimer = setTimeout(() => void _Sync(true), _SettleMs)
  }, _CommitMs)
}

async function _Sync(Settle = false): Promise<void> {
  if (_SyncTimer) clearTimeout(_SyncTimer)
  _SyncTimer = null

  try {
    const Now = await Api.LoadLive()
    Screen.value = Now
    LiveTick.value += 1
    if (Settle) Settled.value = true
    const Left = Now ? Math.max(0, Now.Seconds * 1000 - Now.Elapsed) + 300 : _RetryMs
    _SyncTimer = setTimeout(() => void _Sync(), Left)
  } catch {
    _SyncTimer = setTimeout(() => void _Sync(Settle), _RetryMs)
  }
}

function _OnVisibility(): void {
  if (!document.hidden) void _Sync()
}

function Step(P: ScreenPage, Direction: -1 | 1): void {
  const Next = Direction > 0
    ? _Steps.find(S => S > P.Seconds)
    : [..._Steps].reverse().find(S => S < P.Seconds)
  if (Next === undefined) return
  P.Seconds = Next
  _Commit()
}

function CanStep(P: ScreenPage, Direction: -1 | 1): boolean {
  return Direction > 0 ? P.Seconds < _Steps[_Steps.length - 1] : P.Seconds > _Steps[0]
}

function Locked(P: ScreenPage): boolean {
  return P.Enabled && Visible.value.length === 1 && Shown(P)
}

function Toggle(P: ScreenPage): void {
  if (Locked(P)) return
  P.Enabled = !P.Enabled
  _Commit()
}

function _Move(From: number, To: number): void {
  if (From === To) return
  const List = [...Draft.value]
  const [Item] = List.splice(From, 1)
  List.splice(To, 0, Item)
  Draft.value = List
  _Commit()
}

function Grab(Event: PointerEvent, Index: number): void {
  if (Event.button !== 0 && Event.pointerType === "mouse") return
  const Items = Array.from(ListEl.value?.querySelectorAll<HTMLElement>(".Item") ?? [])
  if (!Items.length) return

  _Tops = Items.map(El => El.offsetTop)
  _Heights = Items.map(El => El.offsetHeight)
  _From = Index
  _StartY = Event.clientY
  Offset.value = 0
  Target.value = Index
  Dragging.value = Draft.value[Index].Id
  ;(Event.currentTarget as HTMLElement).setPointerCapture(Event.pointerId)
}

function Drag(Event: PointerEvent): void {
  if (!Dragging.value) return

  const Last = _Tops.length - 1
  const Min = _Tops[0] - _Tops[_From]
  const Max = _Tops[Last] + _Heights[Last] - _Heights[_From] - _Tops[_From]
  Offset.value = Math.max(Min, Math.min(Max, Event.clientY - _StartY))

  const Centre = _Tops[_From] + _Heights[_From] / 2 + Offset.value
  let Slot = 0
  for (let I = 0; I < _Tops.length; I++) {
    if (I !== _From && Centre > _Tops[I] + _Heights[I] / 2) Slot++
  }
  Target.value = Slot
}

function Drop(): void {
  if (!Dragging.value) return

  Settling.value = true
  _Move(_From, Target.value)
  Dragging.value = null
  Offset.value = 0
  requestAnimationFrame(() => requestAnimationFrame(() => { Settling.value = false }))
}

function Shift(Index: number): string | undefined {
  if (!Dragging.value) return undefined
  if (Index === _From) return `translateY(${Offset.value}px)`
  const Height = _Heights[_From]
  if (_From < Target.value && Index > _From && Index <= Target.value) return `translateY(${-Height}px)`
  if (_From > Target.value && Index >= Target.value && Index < _From) return `translateY(${Height}px)`
  return "translateY(0)"
}

async function Nudge(Event: KeyboardEvent, Index: number): Promise<void> {
  const Direction = Event.key === "ArrowUp" ? -1 : Event.key === "ArrowDown" ? 1 : 0
  if (!Direction) return
  Event.preventDefault()

  const To = Index + Direction
  if (To < 0 || To >= Draft.value.length) return

  const Id = Draft.value[Index].Id
  _Move(Index, To)
  await nextTick()
  ListEl.value?.querySelector<HTMLElement>(`[data-page="${Id}"] .Handle`)?.focus()
}

onMounted(() => {
  void _Sync()
  _RefreshTimer = setInterval(() => void _Sync(), _RefreshMs)
  document.addEventListener("visibilitychange", _OnVisibility)
})

onUnmounted(() => {
  if (_SyncTimer) clearTimeout(_SyncTimer)
  if (_SettleTimer) clearTimeout(_SettleTimer)
  if (_RefreshTimer) clearInterval(_RefreshTimer)
  document.removeEventListener("visibilitychange", _OnVisibility)
  if (_CommitTimer) {
    clearTimeout(_CommitTimer)
    Emit("Change", Draft.value.map(P => ({ ...P })))
  }
})
</script>

<template>
  <Group :Title="t('settings.pages')" Id="screen-pages">
    <template #aside>
      <span class="Cycle">{{ t("settings.pages.cycle") }} · {{ Format(Total) }}</span>
    </template>

    <div class="Preview">
      <TransitionGroup tag="div" name="seg" class="Strip">
        <div
          v-for="P in Draft"
          :key="P.Id"
          class="Segment"
          :class="{ Off: !OnScreen(P), Hover: Hovered === P.Id, Live: Live === P.Id }"
          :style="{ flexGrow: OnScreen(P) ? P.Seconds : 0 }"
          @pointerenter="Hovered = P.Id"
          @pointerleave="Hovered = null"
        >
          <span class="SegName">{{ Name(P.Id) }}</span>
          <span class="SegTime">{{ Format(P.Seconds) }}</span>
          <span v-if="Live === P.Id" :key="LiveTick" class="SegFill" :style="FillStyle" />
        </div>
      </TransitionGroup>
    </div>

    <div ref="ListEl" class="Order" :class="{ Sorting: !!Dragging, Settling }">
      <div
        v-for="(P, Index) in Draft"
        :key="P.Id"
        :data-page="P.Id"
        class="Item"
        :class="{ Lifted: Dragging === P.Id, Hover: Hovered === P.Id, Dim: !Shown(P) }"
        :style="{ transform: Shift(Index) }"
        @pointerenter="Hovered = P.Id"
        @pointerleave="Hovered = null"
      >
        <button
          type="button"
          class="Handle"
          :aria-label="`${t('settings.pages.move')}: ${Name(P.Id)}`"
          @pointerdown="Grab($event, Index)"
          @pointermove="Drag"
          @pointerup="Drop"
          @pointercancel="Drop"
          @keydown="Nudge($event, Index)"
        >
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <circle cx="9" cy="6" r="1.4" /><circle cx="15" cy="6" r="1.4" />
            <circle cx="9" cy="12" r="1.4" /><circle cx="15" cy="12" r="1.4" />
            <circle cx="9" cy="18" r="1.4" /><circle cx="15" cy="18" r="1.4" />
          </svg>
        </button>

        <div class="Text">
          <p class="Label">{{ Name(P.Id) }}</p>
          <p class="Detail">{{ Detail(P) }}</p>
        </div>

        <div class="Stepper" role="group" :aria-label="`${Name(P.Id)}: ${Format(P.Seconds)}`">
          <button type="button" class="StepBtn" :disabled="!CanStep(P, -1)" :aria-label="t('settings.pages.shorter')" @click="Step(P, -1)">
            <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M6 12h12" /></svg>
          </button>
          <span class="StepValue">{{ Format(P.Seconds) }}</span>
          <button type="button" class="StepBtn" :disabled="!CanStep(P, 1)" :aria-label="t('settings.pages.longer')" @click="Step(P, 1)">
            <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M6 12h12M12 6v12" /></svg>
          </button>
        </div>

        <div class="End">
          <Switch
            v-if="P.Id === 'voting'"
            :On="VotingOn"
            :Label="`${t('settings.pages.show')}: ${Name(P.Id)}`"
            @Toggle="Emit('ToggleVoting')"
          />
          <Switch
            v-else
            :On="P.Enabled"
            :Disabled="Locked(P)"
            :Label="`${t('settings.pages.show')}: ${Name(P.Id)}`"
            @Toggle="Toggle(P)"
          />
        </div>
      </div>
    </div>
  </Group>
</template>

<style scoped>
.Cycle {
  font-variant-numeric: tabular-nums;
}

.Preview {
  position: relative;
  border-top: 1px solid var(--line);
  padding: 18px 0 20px;
}

.Strip {
  display: flex;
  height: 56px;
  gap: 4px;
}

.Segment {
  position: relative;
  display: flex;
  min-width: 0;
  flex-basis: 0;
  flex-direction: column;
  justify-content: center;
  gap: 2px;
  overflow: hidden;
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.045);
  box-shadow: inset 0 0 0 1px var(--line);
  padding: 0 12px;
  transition: flex-grow 420ms var(--ease), opacity 260ms ease, padding 420ms var(--ease), background 180ms ease;
}

.Segment.Off {
  opacity: 0;
  padding: 0;
  margin-left: -4px;
}

.Segment.Hover,
.Segment.Live {
  background: rgba(255, 255, 255, 0.08);
}

.SegName,
.SegTime {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.SegName {
  font-size: 13px;
  font-weight: 500;
  color: var(--ink-soft);
}

.Segment.Live .SegName {
  color: var(--ink);
}

.SegTime {
  font-size: 12px;
  font-variant-numeric: tabular-nums;
  color: var(--ink-faint);
}

.SegFill {
  position: absolute;
  bottom: 0;
  left: 0;
  height: 2px;
  width: 100%;
  background: var(--ink);
  transform-origin: left center;
  animation: Fill linear forwards;
}

@keyframes Fill {
  from { transform: scaleX(0); }
  to { transform: scaleX(1); }
}

.seg-move {
  transition: transform 420ms var(--ease), flex-grow 420ms var(--ease), opacity 260ms ease;
}

.Order {
  position: relative;
}

.Item {
  position: relative;
  display: grid;
  grid-template-columns: 32px minmax(0, 1fr) auto auto;
  align-items: center;
  column-gap: 16px;
  min-height: 76px;
  padding: 14px 0;
  background: var(--bg);
  transition: transform 260ms var(--ease), opacity 200ms ease;
}

.Item::before {
  content: "";
  position: absolute;
  top: 0;
  right: 0;
  left: 0;
  border-top: 1px solid var(--line);
}

.Sorting .Item {
  user-select: none;
}

.Settling .Item {
  transition: none;
}

.Item.Lifted {
  z-index: 2;
  border-radius: 14px;
  background: #121116;
  box-shadow: inset 0 0 0 1px var(--edge-strong), 0 18px 40px -18px rgba(0, 0, 0, 0.9);
  transition: none;
}

.Item.Lifted::before,
.Item.Lifted + .Item::before {
  opacity: 0;
}

.Handle {
  display: flex;
  height: 40px;
  width: 32px;
  align-items: center;
  justify-content: center;
  border-radius: 9px;
  color: var(--ink-faint);
  cursor: grab;
  touch-action: none;
  transition: background 160ms ease, color 160ms ease;
}

.Handle svg {
  height: 18px;
  width: 18px;
  fill: currentColor;
}

.Handle:hover,
.Item.Hover .Handle {
  color: var(--ink-soft);
}

.Handle:hover {
  background: rgba(255, 255, 255, 0.05);
}

.Item.Lifted .Handle {
  color: var(--ink);
  cursor: grabbing;
}

.Text {
  min-width: 0;
  transition: opacity 200ms ease;
}

.Item.Dim .Text,
.Item.Dim .Stepper {
  opacity: 0.45;
}

.Label {
  margin: 0;
  font-size: 15px;
  font-weight: 500;
  line-height: 1.35;
  letter-spacing: -0.01em;
  color: var(--ink);
}

.Detail {
  margin: 3px 0 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 13.5px;
  color: var(--ink-mute);
}

.Stepper {
  display: flex;
  height: 36px;
  align-items: center;
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.04);
  box-shadow: inset 0 0 0 1px var(--edge);
  transition: opacity 200ms ease;
}

.StepBtn {
  display: flex;
  height: 36px;
  width: 34px;
  align-items: center;
  justify-content: center;
  border-radius: 10px;
  color: var(--ink-soft);
  cursor: pointer;
  transition: background 150ms ease, color 150ms ease;
}

.StepBtn svg {
  height: 15px;
  width: 15px;
  fill: none;
  stroke: currentColor;
  stroke-width: 1.8;
  stroke-linecap: round;
}

.StepBtn:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.06);
  color: var(--ink);
}

.StepBtn:active:not(:disabled) {
  background: rgba(255, 255, 255, 0.09);
}

.StepBtn:disabled {
  opacity: 0.3;
  cursor: default;
}

.StepValue {
  min-width: 64px;
  text-align: center;
  font-size: 14px;
  font-variant-numeric: tabular-nums;
  color: var(--ink);
}

.End {
  display: flex;
  min-width: 46px;
  justify-content: flex-end;
}

@media (max-width: 639px) {
  .Item {
    grid-template-columns: 28px minmax(0, 1fr) auto;
    grid-template-areas:
      "handle text end"
      "handle step step";
    row-gap: 12px;
    column-gap: 12px;
  }

  .Handle { grid-area: handle; width: 28px; }
  .Text { grid-area: text; }
  .Stepper { grid-area: step; justify-self: start; }
  .End { grid-area: end; }

  .Segment {
    padding: 0 8px;
  }

  .SegTime {
    display: none;
  }
}

@media (prefers-reduced-motion: reduce) {
  .Item,
  .Segment,
  .seg-move {
    transition: none;
  }
}
</style>
