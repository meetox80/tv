<script setup lang="ts">
import { ref, watch, type Ref } from "vue"
import { Score as FormatScore } from "../_api/format"
import type { Ballot } from "../_api/types"

const Props = defineProps<{
  Value: Ballot
  Score: number
  Title: string
  Busy: boolean
  ReadOnly?: boolean
  Wide?: boolean
}>()

const Emit = defineEmits<{ Cast: [Wanted: Ballot] }>()

const Bump: Ref<number> = ref(0)
const Direction: Ref<"up" | "down"> = ref("up")

watch(() => Props.Score, (Next, Prev) => {
  if (Prev === undefined || Next === Prev) return
  Direction.value = Next > Prev ? "up" : "down"
  Bump.value += 1
})
</script>

<template>
  <div class="Vote" :class="{ Up: Value === 1, Down: Value === -1, Wide }" role="group" :aria-label="`Głosowanie: ${Title}`">
    <button
      type="button"
      class="Arrow ArrowUp"
      :aria-pressed="Value === 1"
      :aria-label="Value === 1 ? `Cofnij głos za: ${Title}` : `Głosuj za: ${Title}`"
      :disabled="Busy || ReadOnly"
      @click="Emit('Cast', 1)"
    >
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <path d="M12 4 20 13h-5v7H9v-7H4z" />
      </svg>
    </button>

    <span class="Count" aria-live="polite">
      <Transition :name="`tick-${Direction}`" mode="out-in">
        <span :key="Bump" class="Number">{{ FormatScore(Score) }}</span>
      </Transition>
    </span>

    <button
      type="button"
      class="Arrow ArrowDown"
      :aria-pressed="Value === -1"
      :aria-label="Value === -1 ? `Cofnij głos przeciw: ${Title}` : `Głosuj przeciw: ${Title}`"
      :disabled="Busy || ReadOnly"
      @click="Emit('Cast', -1)"
    >
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <path d="M12 20 4 11h5V4h6v7h5z" />
      </svg>
    </button>
  </div>
</template>

<style scoped>
.Vote {
  display: flex;
  width: 44px;
  flex-shrink: 0;
  flex-direction: column;
  align-items: center;
}

.Vote.Wide {
  width: auto;
  flex-direction: row;
  gap: 2px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.05);
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.07);
  padding: 3px;
}

.Arrow {
  display: flex;
  height: 38px;
  width: 44px;
  align-items: center;
  justify-content: center;
  border-radius: 14px;
  color: var(--color-ink-faint);
  transition: color 180ms ease, background 180ms ease, transform 220ms var(--ease-smooth);
}

.Arrow svg {
  height: 20px;
  width: 20px;
  fill: none;
  stroke: currentColor;
  stroke-width: 1.7;
  stroke-linejoin: round;
  transition: fill 180ms ease;
}

.Arrow:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.05);
  color: var(--color-ink-soft);
}

.Wide .Arrow {
  height: 44px;
  width: 52px;
  border-radius: 999px;
}

.Arrow:active:not(:disabled) {
  transform: scale(0.88);
}

.Arrow:disabled {
  opacity: 0.6;
}

.Up .ArrowUp {
  color: var(--color-rose);
}

.Up .ArrowUp svg {
  fill: currentColor;
}

.Down .ArrowDown {
  color: #8fa3c9;
}

.Down .ArrowDown svg {
  fill: currentColor;
}

.Count {
  position: relative;
  display: flex;
  height: 22px;
  min-width: 40px;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}

.Wide .Count {
  height: 44px;
  min-width: 48px;
}

.Number {
  font-family: var(--font-display);
  font-size: 16px;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
  letter-spacing: -0.01em;
  color: var(--color-ink);
  transition: color 200ms ease;
}

.Wide .Number {
  font-size: 19px;
}

.Up .Number {
  color: var(--color-rose);
}

.Down .Number {
  color: #8fa3c9;
}

.tick-up-enter-active,
.tick-up-leave-active,
.tick-down-enter-active,
.tick-down-leave-active {
  transition: opacity 160ms ease, transform 260ms var(--ease-smooth);
}

.tick-up-enter-from,
.tick-down-leave-to {
  opacity: 0;
  transform: translateY(12px);
}

.tick-up-leave-to,
.tick-down-enter-from {
  opacity: 0;
  transform: translateY(-12px);
}
</style>
