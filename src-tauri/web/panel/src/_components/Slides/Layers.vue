<script setup lang="ts">
import { ref, computed, nextTick, onMounted, onUnmounted, type Ref } from "vue"
import Icon from "./Icon.vue"
import { LayerName, type SlideElement } from "./model"

const Props = withDefaults(defineProps<{ Elements: SlideElement[]; Selected: string[]; Docked?: boolean; Multi?: boolean }>(), {
  Docked: false,
  Multi: false,
})

const Emit = defineEmits<{
  Select: [Ids: string[]]
  Toggle: [Id: string, Key: "Hidden" | "Locked"]
  Rename: [Id: string, Name: string]
  Reorder: [From: number, To: number]
  Hover: [Id: string | null]
}>()

const _OpenKey = "slides_layers_open"
const _PlaceKey = "slides_layers_place"
const _Margin = 12

interface Place { Right: number; Bottom: number }

interface RowDrag {
  From: number
  StartY: number
  Tops: number[]
  Heights: number[]
  Moved: boolean
  Pointer: number
  Id: string
  Shift: boolean
  Toggle: boolean
}

function _RememberedOpen(): boolean {
  try {
    return localStorage.getItem(_OpenKey) !== "0"
  } catch {
    return true
  }
}

function _RememberedPlace(): Place {
  try {
    const Stored = JSON.parse(localStorage.getItem(_PlaceKey) ?? "null") as Place | null
    if (Stored && Number.isFinite(Stored.Right) && Number.isFinite(Stored.Bottom)) return Stored
  } catch {}
  return { Right: _Margin + 2, Bottom: _Margin + 2 }
}

const Open: Ref<boolean> = ref(_RememberedOpen())
const Spot: Ref<Place> = ref(_RememberedPlace())
const Moving: Ref<boolean> = ref(false)
const Renaming: Ref<string | null> = ref(null)
const Draft: Ref<string> = ref("")
const RootEl: Ref<HTMLElement | null> = ref(null)
const ListEl: Ref<HTMLElement | null> = ref(null)
const NameEl: Ref<HTMLInputElement[] | null> = ref(null)
const DragFrom: Ref<number> = ref(-1)
const DragTarget: Ref<number> = ref(-1)
const DragOffset: Ref<number> = ref(0)

let _Anchor: string | null = null
let _Row: RowDrag | null = null
let _Move: { StartX: number; StartY: number; Origin: Place; Pointer: number; Moved: boolean } | null = null
let _SkipClick = false

const Rows = computed(() => [...Props.Elements].reverse())
const Expanded = computed(() => Props.Docked || Open.value)

const PanelStyle = computed(() => {
  if (Props.Docked) return undefined
  return {
    right: `${Spot.value.Right}px`,
    bottom: `${Spot.value.Bottom}px`,
    maxHeight: `calc(100% - ${Spot.value.Bottom + _Margin}px)`,
  }
})

function _Pick(Id: string, Shift: boolean, Toggle: boolean): void {
  const Order = Rows.value.map(E => E.Id)
  if (Shift) {
    const Anchor = _Anchor && Order.includes(_Anchor)
      ? _Anchor
      : Order.find(Known => Props.Selected.includes(Known)) ?? null
    if (Anchor) {
      const From = Order.indexOf(Anchor)
      const To = Order.indexOf(Id)
      _Anchor = Anchor
      Emit("Select", Order.slice(Math.min(From, To), Math.max(From, To) + 1))
      return
    }
  }
  _Anchor = Id
  if (Toggle || Props.Multi) {
    Emit("Select", Props.Selected.includes(Id) ? Props.Selected.filter(Known => Known !== Id) : [...Props.Selected, Id])
    return
  }
  Emit("Select", [Id])
}

function _Clamp(Next: Place): Place {
  const Panel = RootEl.value
  const Parent = Panel?.offsetParent as HTMLElement | null
  if (!Panel || !Parent) return Next
  const MaxRight = Math.max(_Margin, Parent.clientWidth - Panel.offsetWidth - _Margin)
  const MaxBottom = Math.max(_Margin, Parent.clientHeight - Panel.offsetHeight - _Margin)
  return {
    Right: Math.round(Math.min(MaxRight, Math.max(_Margin, Next.Right))),
    Bottom: Math.round(Math.min(MaxBottom, Math.max(_Margin, Next.Bottom))),
  }
}

function _Refit(): void {
  if (!Props.Docked) Spot.value = _Clamp(Spot.value)
}

function OnHeadDown(Event: PointerEvent): void {
  if (Props.Docked || Event.button !== 0) return
  if ((Event.target as HTMLElement).closest(".Collapse")) return
  _Move = { StartX: Event.clientX, StartY: Event.clientY, Origin: { ...Spot.value }, Pointer: Event.pointerId, Moved: false }
  ;(Event.currentTarget as HTMLElement).setPointerCapture(Event.pointerId)
}

function OnHeadMove(Event: PointerEvent): void {
  const M = _Move
  if (!M || Event.pointerId !== M.Pointer) return
  const Dx = Event.clientX - M.StartX
  const Dy = Event.clientY - M.StartY
  if (!M.Moved && Math.hypot(Dx, Dy) < 4) return
  M.Moved = true
  Moving.value = true
  Spot.value = _Clamp({ Right: M.Origin.Right - Dx, Bottom: M.Origin.Bottom - Dy })
}

function OnHeadUp(Event: PointerEvent): void {
  const M = _Move
  if (!M || Event.pointerId !== M.Pointer) return
  _Move = null
  Moving.value = false
  if (!M.Moved) return
  _SkipClick = true
  try {
    localStorage.setItem(_PlaceKey, JSON.stringify(Spot.value))
  } catch {}
}

function ToggleOpen(): void {
  if (_SkipClick) {
    _SkipClick = false
    return
  }
  Open.value = !Open.value
  void nextTick(_Refit)
  try {
    localStorage.setItem(_OpenKey, Open.value ? "1" : "0")
  } catch {}
}

function StartRename(Element: SlideElement): void {
  Renaming.value = Element.Id
  Draft.value = LayerName(Element)
  void nextTick(() => {
    const Input = NameEl.value?.[0]
    Input?.focus()
    Input?.select()
  })
}

function CommitRename(): void {
  const Id = Renaming.value
  if (!Id) return
  Renaming.value = null
  Emit("Rename", Id, Draft.value.trim())
}

function OnRenameKey(Event: KeyboardEvent): void {
  if (Event.key === "Enter") CommitRename()
  if (Event.key === "Escape") Renaming.value = null
}

function OnRowDown(Event: PointerEvent, Index: number, Id: string): void {
  if (Event.button !== 0 || Renaming.value === Id) return
  if ((Event.target as HTMLElement).closest(".RowTool")) return
  const Items = Array.from(ListEl.value?.querySelectorAll<HTMLElement>(".Row") ?? [])
  _Row = {
    From: Index,
    StartY: Event.clientY,
    Tops: Items.map(El => El.offsetTop),
    Heights: Items.map(El => El.offsetHeight),
    Moved: false,
    Pointer: Event.pointerId,
    Id,
    Shift: Event.shiftKey,
    Toggle: Event.ctrlKey || Event.metaKey,
  }
  window.addEventListener("pointermove", _OnRowMove)
  window.addEventListener("pointerup", _OnRowUp)
  window.addEventListener("pointercancel", _OnRowUp)
}

function _OnRowMove(Event: PointerEvent): void {
  const D = _Row
  if (!D || Event.pointerId !== D.Pointer) return
  const Dy = Event.clientY - D.StartY
  if (!D.Moved && Math.abs(Dy) < 5) return
  D.Moved = true
  DragFrom.value = D.From
  const Last = D.Tops.length - 1
  DragOffset.value = Math.max(D.Tops[0] - D.Tops[D.From], Math.min(D.Tops[Last] - D.Tops[D.From], Dy))
  const Centre = D.Tops[D.From] + D.Heights[D.From] / 2 + DragOffset.value
  let Slot = 0
  for (let I = 0; I < D.Tops.length; I++) {
    if (I !== D.From && Centre > D.Tops[I] + D.Heights[I] / 2) Slot++
  }
  DragTarget.value = Slot
}

function _OnRowUp(Event: PointerEvent): void {
  const D = _Row
  if (!D || Event.pointerId !== D.Pointer) return
  _Row = null
  window.removeEventListener("pointermove", _OnRowMove)
  window.removeEventListener("pointerup", _OnRowUp)
  window.removeEventListener("pointercancel", _OnRowUp)

  const To = DragTarget.value
  DragFrom.value = -1
  DragTarget.value = -1
  DragOffset.value = 0

  if (!D.Moved) {
    _Pick(D.Id, D.Shift, D.Toggle)
    return
  }
  if (To < 0 || To === D.From) return
  const Count = Props.Elements.length
  Emit("Reorder", Count - 1 - D.From, Count - 1 - To)
}

function Shift(Index: number): string | undefined {
  const D = _Row
  if (!D || DragFrom.value < 0) return undefined
  if (Index === D.From) return `translateY(${DragOffset.value}px)`
  const Gap = D.Heights[D.From] + 2
  const To = DragTarget.value
  if (D.From < To && Index > D.From && Index <= To) return `translateY(${-Gap}px)`
  if (D.From > To && Index >= To && Index < D.From) return `translateY(${Gap}px)`
  return undefined
}

onMounted(() => {
  void nextTick(_Refit)
  window.addEventListener("resize", _Refit)
})

onUnmounted(() => {
  window.removeEventListener("resize", _Refit)
  window.removeEventListener("pointermove", _OnRowMove)
  window.removeEventListener("pointerup", _OnRowUp)
  window.removeEventListener("pointercancel", _OnRowUp)
})
</script>

<template>
  <section
    ref="RootEl"
    class="Layers"
    :class="{ Open: Expanded, Moving, Docked }"
    :style="PanelStyle"
    @pointerdown.stop
  >
    <header
      class="Head"
      @pointerdown="OnHeadDown"
      @pointermove="OnHeadMove"
      @pointerup="OnHeadUp"
      @pointercancel="OnHeadUp"
    >
      <span v-if="!Docked" class="Grip" aria-hidden="true"><Icon Name="grip" /></span>
      <span class="HeadTitle">Warstwy</span>
      <span class="Count">{{ Elements.length }}</span>
      <button
        v-if="!Docked"
        type="button"
        class="Collapse"
        :aria-expanded="Open"
        :aria-label="Open ? 'Zwiń warstwy' : 'Rozwiń warstwy'"
        @click="ToggleOpen"
      >
        <Icon Name="chevron" />
      </button>
    </header>

    <div class="Fold">
      <div class="FoldInner">
        <p v-if="!Elements.length" class="Empty">Brak elementów na tym slajdzie</p>
        <div
          v-else
          ref="ListEl"
          class="List"
          :class="{ Sorting: DragFrom >= 0 }"
          title="Shift zaznacza zakres, Ctrl pojedyncze warstwy"
          @pointerleave="Emit('Hover', null)"
        >
          <div
            v-for="(Element, Index) in Rows"
            :key="Element.Id"
            class="Row"
            :class="{ On: Selected.includes(Element.Id), Dim: Element.Hidden, Lifted: DragFrom === Index }"
            :style="{ transform: Shift(Index) }"
            @pointerdown="OnRowDown($event, Index, Element.Id)"
            @pointerenter="Emit('Hover', Element.Id)"
            @dblclick="StartRename(Element)"
          >
            <span class="Kind"><Icon :Name="Element.Kind" /></span>
            <input
              v-if="Renaming === Element.Id"
              ref="NameEl"
              v-model="Draft"
              class="Rename"
              maxlength="60"
              spellcheck="false"
              @blur="CommitRename"
              @keydown="OnRenameKey"
              @pointerdown.stop
            />
            <span v-else class="Name">{{ LayerName(Element) }}</span>
            <span class="Tools">
              <button
                v-if="Docked"
                type="button"
                class="RowTool Always"
                title="Zmień nazwę"
                aria-label="Zmień nazwę"
                @click="StartRename(Element)"
              >
                <Icon Name="text" />
              </button>
              <button
                type="button"
                class="RowTool"
                :class="{ Active: Element.Locked }"
                :title="Element.Locked ? 'Odblokuj' : 'Zablokuj'"
                :aria-label="Element.Locked ? 'Odblokuj' : 'Zablokuj'"
                @click="Emit('Toggle', Element.Id, 'Locked')"
              >
                <Icon :Name="Element.Locked ? 'lock' : 'unlock'" />
              </button>
              <button
                type="button"
                class="RowTool"
                :class="{ Active: Element.Hidden }"
                :title="Element.Hidden ? 'Pokaż' : 'Ukryj'"
                :aria-label="Element.Hidden ? 'Pokaż' : 'Ukryj'"
                @click="Emit('Toggle', Element.Id, 'Hidden')"
              >
                <Icon :Name="Element.Hidden ? 'eye-off' : 'eye'" />
              </button>
            </span>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.Layers {
  position: absolute;
  z-index: 30;
  display: flex;
  width: 264px;
  flex-direction: column;
  overflow: hidden;
  border-radius: 14px;
  background: #0e0d11;
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.08), 0 24px 48px -24px rgba(0, 0, 0, 0.9);
  transition: box-shadow 200ms ease;
}

.Layers.Moving {
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.14), 0 30px 60px -24px rgba(0, 0, 0, 0.95);
}

.Layers.Docked {
  position: static;
  width: 100%;
  min-height: 0;
  flex: 1;
  border-radius: 0;
  background: transparent;
  box-shadow: none;
}

.Head {
  display: flex;
  height: 42px;
  flex-shrink: 0;
  align-items: center;
  gap: 8px;
  padding: 0 6px 0 10px;
  cursor: grab;
  touch-action: none;
  user-select: none;
}

.Moving .Head {
  cursor: grabbing;
}

.Docked .Head {
  cursor: default;
  padding: 0 16px;
}

.Grip {
  display: flex;
  color: var(--ink-faint);
}

.Grip :deep(.Icon) {
  height: 14px;
  width: 14px;
}

.HeadTitle {
  font-size: 12.5px;
  font-weight: 600;
  color: var(--ink);
}

.Count {
  min-width: 20px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.06);
  padding: 1px 7px;
  text-align: center;
  font-size: 11px;
  font-weight: 500;
  font-variant-numeric: tabular-nums;
  color: var(--ink-mute);
}

.Collapse {
  display: flex;
  height: 28px;
  width: 28px;
  margin-left: auto;
  align-items: center;
  justify-content: center;
  border-radius: 7px;
  color: var(--ink-mute);
  cursor: pointer;
  transition: background 140ms ease, color 140ms ease;
}

.Collapse:hover {
  background: rgba(255, 255, 255, 0.06);
  color: var(--ink);
}

.Collapse :deep(.Icon) {
  height: 15px;
  width: 15px;
  transform: rotate(-90deg);
  transition: transform 260ms var(--ease);
}

.Open .Collapse :deep(.Icon) {
  transform: none;
}

.Fold {
  display: grid;
  min-height: 0;
  grid-template-rows: 0fr;
  transition: grid-template-rows 300ms var(--ease);
}

.Open .Fold {
  grid-template-rows: 1fr;
}

.Docked .Fold {
  flex: 1;
}

.FoldInner {
  min-height: 0;
  overflow: hidden;
}

.Open .FoldInner {
  overflow-y: auto;
  scrollbar-width: thin;
  scrollbar-color: rgba(255, 255, 255, 0.08) transparent;
}

.Empty {
  margin: 0;
  border-top: 1px solid var(--line);
  padding: 18px 14px 20px;
  text-align: center;
  font-size: 12px;
  color: var(--ink-faint);
}

.List {
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 2px;
  border-top: 1px solid var(--line);
  padding: 6px;
}

.Docked .List {
  padding: 8px 10px 16px;
}

.Row {
  position: relative;
  display: flex;
  height: 32px;
  flex-shrink: 0;
  align-items: center;
  gap: 9px;
  border-radius: 8px;
  padding: 0 4px 0 7px;
  font-size: 12.5px;
  color: var(--ink-soft);
  touch-action: none;
  user-select: none;
  transition: transform 220ms var(--ease), background 120ms ease, color 120ms ease;
}

.Docked .Row {
  height: 44px;
  font-size: 14px;
}

.Row:hover {
  background: rgba(255, 255, 255, 0.045);
}

.Row.On {
  background: rgba(224, 168, 168, 0.13);
  color: var(--ink);
}

.Row.On:hover {
  background: rgba(224, 168, 168, 0.17);
}

.Row.Lifted {
  z-index: 2;
  background: #1f1d24;
  box-shadow: 0 12px 26px -12px rgba(0, 0, 0, 0.9), inset 0 0 0 1px rgba(255, 255, 255, 0.08);
  transition: none;
}

.Sorting .Row:hover:not(.Lifted):not(.On) {
  background: transparent;
}

.Kind {
  display: flex;
  height: 20px;
  width: 20px;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  border-radius: 5px;
  background: rgba(255, 255, 255, 0.05);
  color: var(--ink-mute);
}

.Kind :deep(.Icon) {
  height: 12px;
  width: 12px;
}

.On .Kind {
  background: rgba(224, 168, 168, 0.18);
  color: var(--rose);
}

.Name {
  min-width: 0;
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.Dim .Name,
.Dim .Kind {
  opacity: 0.4;
}

.Rename {
  min-width: 0;
  flex: 1;
  height: 24px;
  border: 0;
  border-radius: 5px;
  background: rgba(255, 255, 255, 0.06);
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.28);
  padding: 0 6px;
  font: inherit;
  color: var(--ink);
  outline: none;
}

.Tools {
  display: flex;
  flex-shrink: 0;
  gap: 1px;
}

.RowTool {
  display: flex;
  height: 24px;
  width: 24px;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  color: var(--ink-faint);
  opacity: 0;
  cursor: pointer;
  transition: opacity 120ms ease, background 120ms ease, color 120ms ease;
}

.Docked .RowTool {
  height: 36px;
  width: 36px;
  opacity: 1;
}

.RowTool :deep(.Icon) {
  height: 14px;
  width: 14px;
}

.Row:hover .RowTool,
.RowTool.Active,
.RowTool.Always {
  opacity: 1;
}

.RowTool:hover {
  background: rgba(255, 255, 255, 0.07);
  color: var(--ink);
}

.RowTool.Active {
  color: var(--ink-soft);
}

@media (pointer: coarse) {
  .RowTool {
    opacity: 1;
  }
}

@media (prefers-reduced-motion: reduce) {
  .Row,
  .Fold,
  .Collapse :deep(.Icon) {
    transition: none;
  }
}
</style>
