<script setup lang="ts">
import { ref, computed, type Ref } from "vue"
import { Palette } from "./model"

const Props = withDefaults(defineProps<{ Value: string | null; AllowNone?: boolean; NoneLabel?: string }>(), {
  AllowNone: false,
  NoneLabel: "Brak",
})

const Emit = defineEmits<{ Pick: [Value: string | null] }>()

const Open: Ref<boolean> = ref(false)
const Draft: Ref<string | null> = ref(null)

const Shown = computed(() => Draft.value ?? (Props.Value ? Props.Value.replace("#", "") : ""))
const Picker = computed(() => (Props.Value && /^#[0-9a-f]{6}$/i.test(Props.Value) ? Props.Value : "#ffffff"))

function _Normalize(Raw: string): string | null {
  const Hex = Raw.trim().replace(/^#/, "").toLowerCase()
  if (/^[0-9a-f]{3}$/.test(Hex)) return `#${Hex.split("").map(C => C + C).join("")}`
  if (/^[0-9a-f]{6}$/.test(Hex)) return `#${Hex}`
  return null
}

function Commit(): void {
  if (Draft.value === null) return
  const Next = _Normalize(Draft.value)
  Draft.value = null
  if (Next && Next !== Props.Value) Emit("Pick", Next)
}

function OnKey(Event: KeyboardEvent): void {
  if (Event.key === "Enter") (Event.target as HTMLInputElement).blur()
  if (Event.key === "Escape") {
    Draft.value = null
    ;(Event.target as HTMLInputElement).blur()
  }
}

function Choose(Value: string | null): void {
  Emit("Pick", Value)
}
</script>

<template>
  <div class="Color">
    <div class="Line">
      <button
        type="button"
        class="Chip"
        :class="{ None: !Value, Open }"
        :style="Value ? { background: Value } : undefined"
        :aria-expanded="Open"
        aria-label="Wybierz kolor"
        @click="Open = !Open"
      />
      <input
        class="Hex"
        type="text"
        maxlength="7"
        spellcheck="false"
        :value="Shown"
        :placeholder="NoneLabel"
        @input="Draft = ($event.target as HTMLInputElement).value"
        @blur="Commit"
        @keydown="OnKey"
      />
      <button v-if="AllowNone && Value" type="button" class="Clear" :title="NoneLabel" :aria-label="NoneLabel" @click="Choose(null)">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"><path d="M6 12h12" /></svg>
      </button>
    </div>

    <div class="Fold" :class="{ Shown: Open }">
      <div class="FoldInner">
        <div class="Swatches">
          <button
            v-if="AllowNone"
            type="button"
            class="Swatch None"
            :class="{ On: !Value }"
            :title="NoneLabel"
            :aria-label="NoneLabel"
            @click="Choose(null)"
          />
          <button
            v-for="Color in Palette"
            :key="Color"
            type="button"
            class="Swatch"
            :class="{ On: Value === Color }"
            :style="{ background: Color }"
            :aria-label="Color"
            @click="Choose(Color)"
          />
          <label class="Swatch Custom" title="Własny kolor">
            <input type="color" :value="Picker" @input="Choose(($event.target as HTMLInputElement).value)" />
          </label>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.Color {
  display: flex;
  flex-direction: column;
}

.Line {
  display: flex;
  height: 30px;
  align-items: center;
  gap: 8px;
  border-radius: 7px;
  background: rgba(255, 255, 255, 0.035);
  padding: 0 6px;
  transition: box-shadow 140ms ease;
}

.Line:hover,
.Line:focus-within {
  box-shadow: inset 0 0 0 1px var(--edge);
}

.Chip {
  height: 18px;
  width: 18px;
  flex-shrink: 0;
  border-radius: 4px;
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.16);
  cursor: pointer;
}

.Chip.Open {
  box-shadow: 0 0 0 1.5px var(--ink);
}

.None {
  background:
    linear-gradient(135deg, transparent calc(50% - 1px), var(--danger) calc(50% - 1px), var(--danger) calc(50% + 1px), transparent calc(50% + 1px)),
    rgba(255, 255, 255, 0.04);
}

.Hex {
  min-width: 0;
  flex: 1;
  border: 0;
  background: transparent;
  font-family: var(--mono);
  font-size: 12px;
  color: var(--ink);
  outline: none;
}

.Hex::placeholder {
  font-family: inherit;
  color: var(--ink-faint);
}

.Clear {
  display: flex;
  height: 22px;
  width: 22px;
  align-items: center;
  justify-content: center;
  border-radius: 5px;
  color: var(--ink-faint);
  cursor: pointer;
}

.Clear svg {
  height: 13px;
  width: 13px;
}

.Clear:hover {
  background: rgba(255, 255, 255, 0.06);
  color: var(--ink);
}

.Fold {
  display: grid;
  grid-template-rows: 0fr;
  opacity: 0;
  transition: grid-template-rows 260ms var(--ease), opacity 180ms ease;
}

.Fold.Shown {
  grid-template-rows: 1fr;
  opacity: 1;
}

.FoldInner {
  min-height: 0;
  overflow: hidden;
}

.Swatches {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  gap: 6px;
  padding: 10px 2px 2px;
}

.Swatch {
  position: relative;
  aspect-ratio: 1;
  border-radius: 5px;
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.12);
  cursor: pointer;
  transition: transform 140ms var(--ease);
}

.Swatch:hover {
  transform: scale(1.1);
}

.Swatch.On {
  box-shadow: 0 0 0 2px #09080b, 0 0 0 3.5px var(--ink);
}

.Custom {
  overflow: hidden;
  background: conic-gradient(#ef8a8a, #e8c07a, #8fd9a8, #8ab4f8, #b89cf5, #ef8a8a);
}

.Custom input {
  position: absolute;
  inset: 0;
  height: 100%;
  width: 100%;
  opacity: 0;
  cursor: pointer;
}
</style>
