<script setup lang="ts" generic="T extends string | number">
import { computed } from "vue"

const Props = defineProps<{ Options: { Value: T; Label: string }[]; Value: T }>()

const Emit = defineEmits<{ Change: [value: T] }>()

const Index = computed(() => Math.max(Props.Options.findIndex(O => O.Value === Props.Value), 0))

const ThumbStyle = computed(() => ({
  width: `calc((100% - 8px) / ${Props.Options.length})`,
  transform: `translateX(${Index.value * 100}%)`,
}))
</script>

<template>
  <div class="Segmented" role="radiogroup">
    <span class="Thumb" :style="ThumbStyle" />
    <button
      v-for="Option in Options"
      :key="String(Option.Value)"
      type="button"
      role="radio"
      class="Option"
      :class="{ Active: Option.Value === Value }"
      :aria-checked="Option.Value === Value"
      @click="Emit('Change', Option.Value)"
    >
      {{ Option.Label }}
    </button>
  </div>
</template>

<style scoped>
.Segmented {
  position: relative;
  display: inline-grid;
  grid-auto-columns: 1fr;
  grid-auto-flow: column;
  flex-shrink: 0;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.035);
  box-shadow: inset 0 0 0 1px var(--edge);
  padding: 4px;
}

.Thumb {
  position: absolute;
  top: 4px;
  bottom: 4px;
  left: 4px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.1);
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.08), inset 0 0 0 1px rgba(255, 255, 255, 0.06), 0 6px 16px -8px rgba(0, 0, 0, 0.8);
  transition: transform 420ms var(--ease);
}

.Option {
  position: relative;
  z-index: 1;
  min-width: 68px;
  height: 30px;
  padding: 0 14px;
  font-size: 13px;
  font-weight: 500;
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
  color: var(--ink-mute);
  cursor: pointer;
  transition: color 220ms ease;
}

.Option:hover {
  color: var(--ink-soft);
}

.Option.Active {
  color: var(--ink);
}

@media (max-width: 1023px) {
  .Option {
    height: 34px;
    min-width: 76px;
    font-size: 14px;
  }
}
</style>
