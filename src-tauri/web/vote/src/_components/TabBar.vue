<script setup lang="ts">
import { ActiveTab, Tabs, type Tab } from "../_state/store"

defineProps<{ Away: boolean }>()

const Emit = defineEmits<{ Pick: [Id: Tab] }>()
</script>

<template>
  <nav class="Bar" aria-label="Sekcje">
    <div class="Track">
      <button
        v-for="I in Tabs"
        :key="I.Id"
        type="button"
        class="Tab"
        :class="{ On: !Away && ActiveTab === I.Id }"
        :aria-current="!Away && ActiveTab === I.Id ? 'page' : undefined"
        @click="Emit('Pick', I.Id)"
      >
        <svg v-if="I.Id === 'board'" viewBox="0 0 24 24" aria-hidden="true">
          <path d="M4 20V13.5h4.5V20M9.75 20V5h4.5v15M15.5 20v-9.5H20V20" />
        </svg>
        <svg v-else-if="I.Id === 'request'" viewBox="0 0 24 24" aria-hidden="true">
          <circle cx="11" cy="11" r="6.5" />
          <path d="m20 20-4.2-4.2" />
        </svg>
        <svg v-else viewBox="0 0 24 24" aria-hidden="true">
          <path d="M9 18V6l10-2v12" />
          <circle cx="6.5" cy="18" r="2.5" />
          <circle cx="16.5" cy="16" r="2.5" />
        </svg>
        <span class="Label">{{ I.Label }}</span>
      </button>
    </div>
  </nav>
</template>

<style scoped>
.Bar {
  position: fixed;
  right: 0;
  bottom: var(--foot);
  left: 0;
  z-index: 30;
  display: flex;
  justify-content: center;
  padding: 0 var(--gutter) 10px;
  pointer-events: none;
}

.Track {
  display: flex;
  width: 100%;
  max-width: 400px;
  gap: 2px;
  border-radius: 22px;
  background: rgba(18, 17, 21, 0.78);
  box-shadow:
    inset 0 0 0 1px rgba(255, 255, 255, 0.07),
    0 20px 50px -20px rgba(0, 0, 0, 0.9);
  backdrop-filter: blur(24px) saturate(1.4);
  -webkit-backdrop-filter: blur(24px) saturate(1.4);
  padding: 5px;
  pointer-events: auto;
}

.Tab {
  display: flex;
  min-height: 54px;
  flex: 1;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 4px;
  border-radius: 17px;
  color: var(--color-ink-faint);
  transition: color 200ms ease, background 260ms var(--ease-smooth);
}

.Tab svg {
  height: 22px;
  width: 22px;
  fill: none;
  stroke: currentColor;
  stroke-width: 1.7;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.Label {
  font-size: 11px;
  font-weight: 500;
  letter-spacing: 0.005em;
}

.Tab.On {
  background: rgba(255, 255, 255, 0.07);
  color: var(--color-ink);
}

@media (min-width: 900px) {
  .Bar {
    display: none;
  }
}
</style>
