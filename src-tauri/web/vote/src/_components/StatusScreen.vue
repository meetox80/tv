<script setup lang="ts">
defineProps<{
  Tone: "closed" | "banned" | "offline" | "loading"
  Title: string
  Handle?: string | null
}>()

const Emit = defineEmits<{ Retry: [] }>()
</script>

<template>
  <section class="Status" :class="Tone" aria-live="polite">
    <div class="Emblem" aria-hidden="true">
      <svg v-if="Tone === 'loading'" viewBox="0 0 64 64" class="Bars">
        <rect v-for="I in 5" :key="I" :x="8 + (I - 1) * 11" y="18" width="5" height="28" rx="2.5" :style="{ animationDelay: `${I * -0.18}s` }" />
      </svg>
      <svg v-else-if="Tone === 'closed'" viewBox="0 0 24 24">
        <rect x="5" y="10.5" width="14" height="10" rx="3" />
        <path d="M8.5 10.5V8a3.5 3.5 0 0 1 7 0v2.5" />
      </svg>
      <svg v-else-if="Tone === 'banned'" viewBox="0 0 24 24">
        <circle cx="12" cy="12" r="8.5" />
        <path d="m6 6 12 12" />
      </svg>
      <svg v-else viewBox="0 0 24 24">
        <path d="M3 9.5a14 14 0 0 1 18 0M6.5 13a9 9 0 0 1 11 0M10 16.5a4 4 0 0 1 4 0" />
        <path d="m3 3 18 18" />
      </svg>
    </div>

    <h1 class="Title">{{ Title }}</h1>
    <div class="Body"><slot /></div>

    <code v-if="Handle" class="Handle">{{ Handle }}</code>

    <button v-if="Tone !== 'loading' && Tone !== 'banned'" type="button" class="Retry" @click="Emit('Retry')">
      Sprawdź ponownie
    </button>
  </section>
</template>

<style scoped>
.Status {
  display: flex;
  min-height: 62dvh;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 14px;
  padding: 40px 8px;
  text-align: center;
}

.Emblem {
  display: flex;
  height: 84px;
  width: 84px;
  align-items: center;
  justify-content: center;
  margin-bottom: 8px;
  border-radius: 28px;
  background: rgba(255, 255, 255, 0.04);
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.08);
  color: var(--color-rose);
}

.Emblem svg {
  height: 34px;
  width: 34px;
  fill: none;
  stroke: currentColor;
  stroke-width: 1.6;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.loading .Emblem {
  background: transparent;
  box-shadow: none;
}

.Bars {
  height: 56px !important;
  width: 56px !important;
}

.Bars rect {
  fill: var(--color-rose);
  stroke: none;
  transform-origin: center;
  transform-box: fill-box;
  animation: Bar 1.2s ease-in-out infinite;
}

@keyframes Bar {
  0%, 100% { transform: scaleY(0.3); opacity: 0.4; }
  50% { transform: scaleY(1); opacity: 1; }
}

.banned .Emblem {
  color: #f0a0a0;
}

.Title {
  font-family: var(--font-display);
  font-size: 28px;
  font-weight: 600;
  line-height: 1.15;
  letter-spacing: -0.03em;
  text-wrap: balance;
}

.Body {
  max-width: 360px;
  font-size: 15px;
  line-height: 1.6;
  color: var(--color-ink-mute);
}

.Handle {
  border-radius: 12px;
  background: rgba(255, 255, 255, 0.05);
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.08);
  padding: 10px 14px;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 15px;
  letter-spacing: 0.04em;
  color: var(--color-ink-soft);
  user-select: all;
}

.Retry {
  min-height: 48px;
  margin-top: 8px;
  border-radius: 14px;
  background: rgba(255, 255, 255, 0.07);
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.08);
  padding: 0 22px;
  font-size: 15px;
  font-weight: 500;
  color: var(--color-ink);
  transition: background 160ms ease;
}

.Retry:hover {
  background: rgba(255, 255, 255, 0.11);
}
</style>
