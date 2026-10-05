<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch, type Ref } from "vue"
import { useI18n } from "../../../i18n"
import RollingDigits from "./RollingDigits.vue"
import { Timeline, Resolve, type Period, type Schedule } from "../_schedule"

const Props = defineProps<{
  Schedule: Schedule | null
  Bpm: number | null
}>()

const { t } = useI18n()

const RestingDriftMs = 2600

const Now: Ref<number> = ref(Date.now())
const Current: Ref<Period | null> = ref(null)

let _Frame: number | null = null
let _Last = 0
let _Speed = RestingDriftMs

const Line = computed(() => Timeline(Props.Schedule))

function _Resolve(At: number): Period | null {
  return Resolve(Line.value, At)
}

const TimeText = computed(() => {
  if (!Current.value) return "00:00"

  const Remaining = Math.max(0, Math.floor((Current.value.EndMs - Now.value) / 1000))
  const Minutes = Math.floor(Remaining / 60)
  const Seconds = Remaining % 60

  return `${String(Minutes).padStart(2, "0")}:${String(Seconds).padStart(2, "0")}`
})

const Progress = computed(() => {
  const Period = Current.value
  if (!Period) return 0

  const Total = Period.EndMs - Period.StartMs
  if (Total <= 0) return 0

  return Math.min(1, Math.max(0, (Now.value - Period.StartMs) / Total))
})

const Label = computed(() => {
  const Period = Current.value
  if (!Period) return t("lesson.after")

  if (Period.IsLesson) return `${t("lesson.lesson")} ${Period.Number}`

  const Minutes = Math.round((Period.EndMs - Period.StartMs) / 60000)
  return `${t("lesson.break")} ${Minutes} ${t("lesson.minutes")}`
})

const BarWidth = 838
const BarHeight = 46
const WaveLength = 120
const WaveAmplitude = 9

const Mid = BarHeight / 2
const Inset = 6
const Span = BarWidth - Inset * 2
const Settle = 90

const BeatsPerWave = 4
const TempoEaseMs = 2200

const TargetDriftMs = computed(() => {
  const Tempo = Props.Bpm
  if (!Tempo || Tempo <= 0) return RestingDriftMs
  return (60000 / Tempo) * BeatsPerWave
})
const Step = 3

const Phase: Ref<number> = ref(0)

function _WaveY(X: number, Active: number, Shift: number): number {
  const Damp = X <= Active ? 1 : Math.max(0, 1 - (X - Active) / Settle)
  if (Damp <= 0) return Mid
  return Mid + WaveAmplitude * Damp * Math.sin(((X - Shift) / WaveLength) * Math.PI * 2)
}

function _Path(From: number, To: number, Active: number, Shift: number): string {
  if (To - From < 0.5) return ""

  const Out: string[] = []
  for (let X = From; X < To; X += Step) {
    Out.push(`${Out.length === 0 ? "M" : "L"}${X.toFixed(1)} ${_WaveY(X, Active, Shift).toFixed(2)}`)
  }
  Out.push(`L${To.toFixed(1)} ${_WaveY(To, Active, Shift).toFixed(2)}`)

  return Out.join(" ")
}

const ActiveEdge = computed(() => Inset + Progress.value * Span)

const ActivePath = computed(() => _Path(Inset, ActiveEdge.value, ActiveEdge.value, Phase.value))

const RestPath = computed(() =>
  _Path(ActiveEdge.value, BarWidth - Inset, ActiveEdge.value, Phase.value)
)

function _Refresh(Stamp: number): void {
  const Delta = _Last === 0 ? 0 : Math.min(Stamp - _Last, 250)
  _Last = Stamp

  const Ease = 1 - Math.exp(-Delta / TempoEaseMs)
  _Speed += (TargetDriftMs.value - _Speed) * Ease

  Phase.value = (Phase.value + (Delta / _Speed) * WaveLength) % WaveLength
  Now.value = Date.now()

  if (!Current.value || Now.value >= Current.value.EndMs) Current.value = _Resolve(Now.value)

  _Frame = requestAnimationFrame(_Refresh)
}

watch(() => Props.Schedule, () => {
  Current.value = _Resolve(Date.now())
}, { immediate: true })

onMounted(() => {
  _Frame = requestAnimationFrame(_Refresh)
})

onUnmounted(() => {
  if (_Frame) cancelAnimationFrame(_Frame)
})
</script>

<template>
  <div class="Timer">
    <Transition name="fade" mode="out-in">
      <div v-if="Current" key="running" class="Running">
        <p class="Eyebrow">{{ Label }}</p>

        <RollingDigits :Value="TimeText" class="Digits" />

        <svg class="Bar" :viewBox="`0 0 ${BarWidth} ${BarHeight}`" :width="BarWidth" :height="BarHeight">
          <path v-if="RestPath" class="Rest" :d="RestPath" />
          <path v-if="ActivePath" class="Wave" :d="ActivePath" />
        </svg>

      </div>

      <p v-else key="idle" class="Resting">{{ Label }}</p>
    </Transition>
  </div>
</template>

<style scoped>
.Timer {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
}

.Running {
  display: flex;
  flex-direction: column;
  align-items: center;
}

.Eyebrow {
  font-family: var(--font-body);
  margin-bottom: 14px;
  border-radius: 999px;
  padding: 11px 26px;
  background: rgba(255, 255, 255, 0.06);
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.08);
  backdrop-filter: blur(16px);
  font-size: 27px;
  font-weight: 500;
  letter-spacing: -0.01em;
  line-height: 1;
  color: rgba(255, 255, 255, 0.72);
}

.Digits {
  font-family: var(--font-timer);
  font-size: 286px;
  font-feature-settings: "tnum" 1;
  color: rgba(255, 255, 255, 0.98);
}

.Bar {
  margin-top: 9px;
  overflow: visible;
}

.Wave {
  fill: none;
  stroke: rgba(255, 255, 255, 0.92);
  stroke-width: 11;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.Rest {
  fill: none;
  stroke: rgba(255, 255, 255, 0.19);
  stroke-width: 11;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.Resting {
  font-family: var(--font-body);
  font-size: 84px;
  font-weight: 500;
  letter-spacing: -0.035em;
  line-height: 1;
  color: rgba(255, 255, 255, 0.82);
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 500ms ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
