<script setup lang="ts">
import { ref, computed, type Ref } from "vue"

const Props = withDefaults(defineProps<{
  Value: number | null
  Label?: string
  Title?: string
  Suffix?: string
  Min?: number
  Max?: number
  Step?: number
  Disabled?: boolean
  Placeholder?: string
}>(), {
  Label: "",
  Title: "",
  Suffix: "",
  Min: -100000,
  Max: 100000,
  Step: 1,
  Disabled: false,
  Placeholder: "",
})

const Emit = defineEmits<{ Change: [Value: number, Final: boolean] }>()

const Focused: Ref<boolean> = ref(false)
const Draft: Ref<string> = ref("")
const Scrubbing: Ref<boolean> = ref(false)

let _Start = 0
let _Origin = 0
let _Last = 0
let _Pointer = -1

const Shown = computed(() => {
  if (Focused.value) return Draft.value
  if (Props.Value === null) return ""
  return String(Math.round(Props.Value))
})

function _Clamp(Value: number): number {
  return Math.min(Props.Max, Math.max(Props.Min, Math.round(Value)))
}

function OnFocus(Event: FocusEvent): void {
  Focused.value = true
  Draft.value = Props.Value === null ? "" : String(Math.round(Props.Value))
  ;(Event.target as HTMLInputElement).select()
}

function _Commit(): void {
  const Parsed = Number(Draft.value.replace(",", ".").trim())
  if (Draft.value.trim() !== "" && Number.isFinite(Parsed)) Emit("Change", _Clamp(Parsed), true)
}

function OnBlur(): void {
  _Commit()
  Focused.value = false
}

function OnKey(Event: KeyboardEvent): void {
  if (Event.key === "Enter") {
    _Commit()
    ;(Event.target as HTMLInputElement).blur()
    return
  }
  if (Event.key === "Escape") {
    Draft.value = Props.Value === null ? "" : String(Math.round(Props.Value))
    ;(Event.target as HTMLInputElement).blur()
    return
  }
  if (Event.key !== "ArrowUp" && Event.key !== "ArrowDown") return
  Event.preventDefault()
  const Base = Number(Draft.value.replace(",", ".")) || Props.Value || 0
  const Next = _Clamp(Base + (Event.key === "ArrowUp" ? 1 : -1) * (Event.shiftKey ? 10 : 1))
  Draft.value = String(Next)
  Emit("Change", Next, true)
}

function OnScrub(Event: PointerEvent): void {
  if (Props.Disabled || Event.button !== 0) return
  Event.preventDefault()
  _Start = Event.clientX
  _Origin = Props.Value ?? 0
  _Last = _Origin
  _Pointer = Event.pointerId
  Scrubbing.value = true
  ;(Event.currentTarget as HTMLElement).setPointerCapture(Event.pointerId)
}

function OnScrubMove(Event: PointerEvent): void {
  if (!Scrubbing.value || Event.pointerId !== _Pointer) return
  const Factor = Event.shiftKey ? 10 : 1
  const Next = _Clamp(_Origin + Math.round((Event.clientX - _Start) / 2) * Props.Step * Factor)
  if (Next === _Last) return
  _Last = Next
  Emit("Change", Next, false)
}

function OnScrubEnd(Event: PointerEvent): void {
  if (!Scrubbing.value || Event.pointerId !== _Pointer) return
  Scrubbing.value = false
  Emit("Change", _Last, true)
}
</script>

<template>
  <div class="Num" :class="{ Disabled, Focus: Focused, Scrubbing }" :title="Title">
    <span
      class="Tag"
      @pointerdown="OnScrub"
      @pointermove="OnScrubMove"
      @pointerup="OnScrubEnd"
      @pointercancel="OnScrubEnd"
    >
      <slot>{{ Label }}</slot>
    </span>
    <input
      class="Field"
      type="text"
      inputmode="decimal"
      :value="Shown"
      :placeholder="Placeholder"
      :disabled="Disabled"
      spellcheck="false"
      @focus="OnFocus"
      @blur="OnBlur"
      @input="Draft = ($event.target as HTMLInputElement).value"
      @keydown="OnKey"
    />
    <span v-if="Suffix" class="Suffix">{{ Suffix }}</span>
  </div>
</template>

<style scoped>
.Num {
  display: flex;
  height: 30px;
  min-width: 0;
  align-items: center;
  border-radius: 7px;
  background: rgba(255, 255, 255, 0.035);
  box-shadow: inset 0 0 0 1px transparent;
  transition: box-shadow 140ms ease, background 140ms ease;
}

.Num:hover {
  box-shadow: inset 0 0 0 1px var(--edge);
}

.Num.Focus {
  background: rgba(255, 255, 255, 0.05);
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.3);
}

.Num.Disabled {
  opacity: 0.55;
}

.Tag {
  display: flex;
  height: 100%;
  width: 26px;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  font-size: 11px;
  color: var(--ink-faint);
  cursor: ew-resize;
  touch-action: none;
  user-select: none;
}

.Disabled .Tag {
  cursor: default;
}

.Scrubbing .Tag,
.Tag:hover {
  color: var(--ink-soft);
}

.Tag :deep(.Icon) {
  height: 13px;
  width: 13px;
}

.Field {
  min-width: 0;
  flex: 1;
  height: 100%;
  border: 0;
  background: transparent;
  padding: 0 6px 0 0;
  font: inherit;
  font-size: 12px;
  font-variant-numeric: tabular-nums;
  color: var(--ink);
  outline: none;
}

.Field::placeholder {
  color: var(--ink-faint);
}

.Suffix {
  flex-shrink: 0;
  padding-right: 8px;
  font-size: 11px;
  color: var(--ink-faint);
}
</style>
