<script setup lang="ts">
import { ref, onUnmounted, type Ref } from "vue"
import Thumb from "./Thumb.vue"
import Icon from "./Icon.vue"
import type { Slide } from "./model"

withDefaults(defineProps<{ Slides: Slide[]; Current: number; Docked?: boolean }>(), { Docked: false })

const Emit = defineEmits<{
  Select: [Index: number]
  Add: []
  Duplicate: [Index: number]
  Remove: [Index: number]
  Move: [From: number, To: number]
}>()

interface Drag {
  From: number
  StartY: number
  Tops: number[]
  Heights: number[]
  Moved: boolean
  Pointer: number
}

const ListEl: Ref<HTMLElement | null> = ref(null)
const From: Ref<number> = ref(-1)
const Target: Ref<number> = ref(-1)
const Offset: Ref<number> = ref(0)

let _Drag: Drag | null = null

function OnDown(Event: PointerEvent, Index: number): void {
  if (Event.button !== 0) return
  if ((Event.target as HTMLElement).closest(".ThumbTool")) return
  const Items = Array.from(ListEl.value?.querySelectorAll<HTMLElement>(".Frame") ?? [])
  _Drag = {
    From: Index,
    StartY: Event.clientY,
    Tops: Items.map(El => El.offsetTop),
    Heights: Items.map(El => El.offsetHeight),
    Moved: false,
    Pointer: Event.pointerId,
  }
  window.addEventListener("pointermove", _OnMove)
  window.addEventListener("pointerup", _OnUp)
  window.addEventListener("pointercancel", _OnUp)
}

function _OnMove(Event: PointerEvent): void {
  const D = _Drag
  if (!D || Event.pointerId !== D.Pointer) return
  const Dy = Event.clientY - D.StartY
  if (!D.Moved && Math.abs(Dy) < 6) return
  D.Moved = true
  From.value = D.From
  const Last = D.Tops.length - 1
  Offset.value = Math.max(D.Tops[0] - D.Tops[D.From], Math.min(D.Tops[Last] - D.Tops[D.From], Dy))
  const Centre = D.Tops[D.From] + D.Heights[D.From] / 2 + Offset.value
  let Slot = 0
  for (let I = 0; I < D.Tops.length; I++) {
    if (I !== D.From && Centre > D.Tops[I] + D.Heights[I] / 2) Slot++
  }
  Target.value = Slot
}

function _OnUp(Event: PointerEvent): void {
  const D = _Drag
  if (!D || Event.pointerId !== D.Pointer) return
  _Drag = null
  window.removeEventListener("pointermove", _OnMove)
  window.removeEventListener("pointerup", _OnUp)
  window.removeEventListener("pointercancel", _OnUp)

  const To = Target.value
  From.value = -1
  Target.value = -1
  Offset.value = 0

  if (!D.Moved) {
    Emit("Select", D.From)
    return
  }
  if (To >= 0 && To !== D.From) Emit("Move", D.From, To)
}

function Shift(Index: number): string | undefined {
  const D = _Drag
  if (!D || From.value < 0) return undefined
  if (Index === D.From) return `translateY(${Offset.value}px)`
  const Gap = D.Heights[D.From] + 10
  if (D.From < Target.value && Index > D.From && Index <= Target.value) return `translateY(${-Gap}px)`
  if (D.From > Target.value && Index >= Target.value && Index < D.From) return `translateY(${Gap}px)`
  return undefined
}

onUnmounted(() => {
  window.removeEventListener("pointermove", _OnMove)
  window.removeEventListener("pointerup", _OnUp)
  window.removeEventListener("pointercancel", _OnUp)
})

</script>

<template>
  <nav ref="ListEl" class="Rail" :class="{ Docked }" aria-label="Slajdy">
    <div
      v-for="(Item, Index) in Slides"
      :key="Item.Id"
      class="Frame"
      :class="{ On: Index === Current, Lifted: From === Index }"
      :style="{ transform: Shift(Index) }"
      @pointerdown="OnDown($event, Index)"
    >
      <span class="Number">{{ Index + 1 }}</span>
      <div class="Picture">
        <Thumb :Slide="Item" />
        <div class="ThumbTools">
          <button type="button" class="ThumbTool" aria-label="Duplikuj slajd" title="Duplikuj" @click.stop="Emit('Duplicate', Index)">
            <Icon Name="duplicate" />
          </button>
          <button
            v-if="Slides.length > 1"
            type="button"
            class="ThumbTool"
            aria-label="Usuń slajd"
            title="Usuń"
            @click.stop="Emit('Remove', Index)"
          >
            <Icon Name="trash" />
          </button>
        </div>
      </div>
    </div>

    <button type="button" class="Add" @click="Emit('Add')">
      <Icon Name="plus" />
      Nowy slajd
    </button>
  </nav>
</template>

<style scoped>
.Rail {
  position: relative;
  display: flex;
  min-height: 0;
  flex-direction: column;
  gap: 10px;
  overflow-y: auto;
  padding: 16px 14px 24px 10px;
  scrollbar-width: thin;
  scrollbar-color: rgba(255, 255, 255, 0.08) transparent;
}

.Rail.Docked {
  flex: 1;
  padding: 12px 16px 20px 10px;
}

.Frame {
  position: relative;
  display: grid;
  grid-template-columns: 18px minmax(0, 1fr);
  align-items: start;
  gap: 6px;
  cursor: pointer;
  touch-action: none;
  user-select: none;
  transition: transform 240ms var(--ease);
}

.Frame.Lifted {
  z-index: 2;
  transition: none;
}

.Number {
  padding-top: 4px;
  text-align: right;
  font-size: 11.5px;
  font-variant-numeric: tabular-nums;
  color: var(--ink-faint);
}

.Frame.On .Number {
  color: var(--ink);
}

.Picture {
  position: relative;
  border-radius: 10px;
  transition: box-shadow 160ms ease;
}

.Frame:hover .Picture {
  box-shadow: 0 0 0 1.5px rgba(255, 255, 255, 0.22);
}

.Frame.On .Picture {
  box-shadow: 0 0 0 2px var(--ink);
}

.Frame.Lifted .Picture {
  box-shadow: 0 0 0 2px var(--ink), 0 16px 32px -12px rgba(0, 0, 0, 0.9);
}

.ThumbTools {
  position: absolute;
  top: 5px;
  right: 5px;
  display: flex;
  gap: 4px;
  opacity: 0;
  transition: opacity 150ms ease;
}

.Frame:hover .ThumbTools {
  opacity: 1;
}

.ThumbTool {
  display: flex;
  height: 24px;
  width: 24px;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  background: rgba(9, 8, 11, 0.9);
  box-shadow: inset 0 0 0 1px var(--edge);
  color: var(--ink-soft);
  cursor: pointer;
}

.ThumbTool :deep(.Icon) {
  height: 13px;
  width: 13px;
}

.ThumbTool:hover {
  color: var(--ink);
}

.Add {
  display: inline-flex;
  height: 38px;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  gap: 6px;
  margin: 4px 0 0 24px;
  border-radius: 10px;
  box-shadow: inset 0 0 0 1px var(--edge);
  font-size: 13px;
  font-weight: 500;
  color: var(--ink-soft);
  cursor: pointer;
  transition: background 150ms ease, color 150ms ease;
}

.Add :deep(.Icon) {
  height: 15px;
  width: 15px;
}

.Add:hover {
  background: rgba(255, 255, 255, 0.05);
  color: var(--ink);
}

@media (pointer: coarse) {
  .ThumbTools {
    opacity: 1;
  }

  .ThumbTool {
    height: 32px;
    width: 32px;
  }
}

@media (prefers-reduced-motion: reduce) {
  .Frame {
    transition: none;
  }
}
</style>
