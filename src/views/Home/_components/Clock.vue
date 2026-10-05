<script setup lang="ts">
import { computed } from "vue"

const Props = defineProps<{
  Now: Date
  Temperature: number | null
  Condition: string | null
}>()

const _Days = ["Niedziela", "Poniedziałek", "Wtorek", "Środa", "Czwartek", "Piątek", "Sobota"]
const _Months = [
  "stycznia", "lutego", "marca", "kwietnia", "maja", "czerwca",
  "lipca", "sierpnia", "września", "października", "listopada", "grudnia",
]

const TimeText = computed(() => {
  const Hours = String(Props.Now.getHours()).padStart(2, "0")
  const Minutes = String(Props.Now.getMinutes()).padStart(2, "0")
  return `${Hours}:${Minutes}`
})

const DateText = computed(() =>
  `${_Days[Props.Now.getDay()]}, ${Props.Now.getDate()} ${_Months[Props.Now.getMonth()]}`
)
</script>

<template>
  <div class="Clock">
    <span class="Time">{{ TimeText }}</span>

    <span class="Meta">
      <template v-if="Temperature !== null">
        <span class="Temperature">{{ Temperature }}&deg;</span>
        <span v-if="Condition" class="Condition">{{ Condition }}</span>
        <span class="Dash" />
      </template>
      <span class="Date">{{ DateText }}</span>
    </span>
  </div>
</template>

<style scoped>
.Clock {
  display: flex;
  flex-shrink: 0;
  flex-direction: column;
  align-items: flex-start;
}

.Time {
  font-family: var(--font-clock, "Inter", system-ui, sans-serif);
  font-size: 76px;
  font-weight: 600;
  line-height: 1;
  letter-spacing: -0.035em;
  color: rgba(255, 255, 255, 0.97);
  font-variant-numeric: tabular-nums;
}

.Meta {
  display: flex;
  align-items: center;
  gap: 11px;
  margin-top: 14px;
  margin-left: 2px;
  font-family: var(--font-clock, "Inter", system-ui, sans-serif);
  font-size: 22px;
  font-weight: 500;
  line-height: 1;
  letter-spacing: -0.005em;
}

.Temperature {
  color: rgba(255, 255, 255, 0.82);
  font-variant-numeric: tabular-nums;
}

.Condition {
  color: rgba(255, 255, 255, 0.55);
}

.Dash {
  height: 16px;
  width: 1px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.18);
}

.Date {
  color: rgba(255, 255, 255, 0.45);
}
</style>
