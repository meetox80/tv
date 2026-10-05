<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, type Ref } from "vue"
import { useI18n } from "../../i18n"
import * as Api from "../../_api/performance"
import Pane from "./ui/Pane.vue"
import Group from "./ui/Group.vue"
import Row from "./ui/Row.vue"
import Stats, { type StatItem } from "./ui/Stats.vue"

const { t } = useI18n()

const _Kilobyte = 1024
const _Megabyte = 1024 * 1024
const _MemoryStep = 64 * _Megabyte
const _MemoryFloor = 128 * _Megabyte
const _RateFloor = 16 * _Kilobyte
const _Units = ["B", "KB", "MB", "GB", "TB"]
const _Count = new Intl.NumberFormat("pl")
const _Decimal = new Intl.NumberFormat("pl", { maximumFractionDigits: 1 })

const Data: Ref<Api.Performance | null> = ref(null)

let _Stream: Api.Stream | null = null

const Span = computed(() => Math.max((Data.value?.History ?? 60) - 1, 1))

const Supported = computed(() => Data.value?.Supported !== false)

const Cpu = computed(() => Data.value?.Cpu ?? [])
const Rss = computed(() => Data.value?.Rss ?? [])
const Up = computed(() => Data.value?.Network?.Up ?? [])
const Down = computed(() => Data.value?.Network?.Down ?? [])

const Ready = computed(() => Cpu.value.length >= 2)
const WireReady = computed(() => Up.value.length >= 2)

const CpuNow = computed(() => Cpu.value[Cpu.value.length - 1] ?? 0)
const RssNow = computed(() => Rss.value[Rss.value.length - 1] ?? 0)
const UpNow = computed(() => Up.value[Up.value.length - 1] ?? 0)
const DownNow = computed(() => Down.value[Down.value.length - 1] ?? 0)

const Ceiling = computed(() => {
  const Peak = Rss.value.reduce((Top, V) => Math.max(Top, V), 0)
  return Math.max(Math.ceil(Peak / _MemoryStep) * _MemoryStep, _MemoryFloor)
})

const RateCeiling = computed(() => {
  const Peak = [...Up.value, ...Down.value].reduce((Top, V) => Math.max(Top, V), 0)
  let Top = _RateFloor
  while (Top < Peak) Top *= 2
  return Top
})

function _Plot(Series: number[], Top: number): { Line: string; Area: string } {
  if (Series.length < 2 || Top <= 0) return { Line: "", Area: "" }

  const Start = Span.value - (Series.length - 1)

  const Line = Series
    .map((Value, Index) => {
      const X = Start + Index
      const Y = 100 - Math.min(100, Math.max(0, (Value / Top) * 100))
      return `${X},${Y.toFixed(2)}`
    })
    .join(" ")

  return { Line, Area: `${Start},100 ${Line} ${Span.value},100` }
}

const CpuPlot = computed(() => _Plot(Cpu.value, 100))
const RssPlot = computed(() => _Plot(Rss.value, Ceiling.value))
const UpPlot = computed(() => _Plot(Up.value, RateCeiling.value))
const DownPlot = computed(() => _Plot(Down.value, RateCeiling.value))

const Megabytes = computed(() => Math.round(RssNow.value / _Megabyte))
const CeilingMb = computed(() => Math.round(Ceiling.value / _Megabyte))

function _Size(Bytes: number): { Value: string; Unit: string } {
  let Value = Math.max(0, Bytes)
  let Index = 0
  while (Value >= 1024 && Index < _Units.length - 1) {
    Value /= 1024
    Index += 1
  }
  const Text = Index === 0 || Value >= 100 ? _Count.format(Math.round(Value)) : _Decimal.format(Value)
  return { Value: Text, Unit: _Units[Index] }
}

function Rate(Bytes: number): string {
  const Size = _Size(Bytes)
  return `${Size.Value} ${Size.Unit}/s`
}

function Duration(Ms: number | null | undefined): string {
  if (typeof Ms !== "number") return "—"
  if (Ms < 1000) return `${Ms} ms`
  return `${_Decimal.format(Ms / 1000)} s`
}

function Count(Value: number | null | undefined): string {
  return typeof Value === "number" ? _Count.format(Value) : "—"
}

function Stat(Value: number | null | undefined): string {
  return typeof Value === "number" ? Value.toFixed(1) : "—"
}

const ServerTiles = computed<StatItem[]>(() => {
  const Net = Data.value?.Network
  const Sent = _Size(Net?.Sent ?? 0)
  const Received = _Size(Net?.Received ?? 0)
  return [
    { Key: "sent", Label: t("settings.performance.sent"), Value: Net ? Sent.Value : "—", Unit: Net ? Sent.Unit : undefined },
    { Key: "received", Label: t("settings.performance.received"), Value: Net ? Received.Value : "—", Unit: Net ? Received.Unit : undefined },
    { Key: "requests", Label: t("settings.performance.requests"), Value: Count(Net?.Requests) },
  ]
})

const Voting = computed(() => Data.value?.Voting ?? null)

const VotingTiles = computed<StatItem[]>(() => {
  const V = Voting.value
  return [
    { Key: "searches", Label: t("settings.performance.searches"), Value: Count(V?.Searches), Faint: !V?.Searches },
    { Key: "votes", Label: t("settings.performance.votes"), Value: Count(V?.Votes), Faint: !V?.Votes },
    { Key: "submissions", Label: t("settings.performance.submissions"), Value: Count(V?.Requests), Faint: !V?.Requests },
  ]
})

const ActiveText = computed(() => {
  const V = Voting.value
  return V ? `${Count(V.Active)} z ${Count(V.Devices)}` : "—"
})

onMounted(() => {
  _Stream = Api.Open((Value) => {
    Data.value = Value
  })
})

onUnmounted(() => {
  _Stream?.Close()
  _Stream = null
})
</script>

<template>
  <Pane Id="performance">
    <div class="Charts">
      <template v-if="Supported">
        <div class="Chart Cpu">
          <div class="ChartHead">
            <span class="ChartLabel">{{ t("settings.performance.cpu") }}</span>
            <span class="ChartValue">{{ Ready ? `${CpuNow.toFixed(1)}%` : "—" }}</span>
          </div>

          <svg class="Plot" :viewBox="`0 0 ${Span} 100`" preserveAspectRatio="none">
            <defs>
              <linearGradient id="perf-cpu" x1="0" y1="0" x2="0" y2="1">
                <stop offset="0" stop-color="#f4f2f7" stop-opacity="0.16" />
                <stop offset="1" stop-color="#f4f2f7" stop-opacity="0" />
              </linearGradient>
            </defs>
            <line v-for="Y in [25, 50, 75]" :key="Y" class="Grid" x1="0" :y1="Y" :x2="Span" :y2="Y" vector-effect="non-scaling-stroke" />
            <polygon v-if="CpuPlot.Area" fill="url(#perf-cpu)" :points="CpuPlot.Area" />
            <polyline v-if="CpuPlot.Line" class="Line Ink" :points="CpuPlot.Line" vector-effect="non-scaling-stroke" />
          </svg>

          <div class="Axis">
            <span>-{{ Span }} s</span>
            <span>100%</span>
          </div>
        </div>

        <div class="Chart Memory">
          <div class="ChartHead">
            <span class="ChartLabel">{{ t("settings.performance.memory") }}</span>
            <span class="ChartValue">{{ Ready ? `${Megabytes} MB` : "—" }}</span>
          </div>

          <svg class="Plot" :viewBox="`0 0 ${Span} 100`" preserveAspectRatio="none">
            <defs>
              <linearGradient id="perf-memory" x1="0" y1="0" x2="0" y2="1">
                <stop offset="0" stop-color="#e0a8a8" stop-opacity="0.22" />
                <stop offset="1" stop-color="#e0a8a8" stop-opacity="0" />
              </linearGradient>
            </defs>
            <line v-for="Y in [25, 50, 75]" :key="Y" class="Grid" x1="0" :y1="Y" :x2="Span" :y2="Y" vector-effect="non-scaling-stroke" />
            <polygon v-if="RssPlot.Area" fill="url(#perf-memory)" :points="RssPlot.Area" />
            <polyline v-if="RssPlot.Line" class="Line Rose" :points="RssPlot.Line" vector-effect="non-scaling-stroke" />
          </svg>

          <div class="Axis">
            <span>-{{ Span }} s</span>
            <span>{{ CeilingMb }} MB</span>
          </div>
        </div>
      </template>

      <p v-else class="Unsupported">{{ t("settings.performance.unsupported") }}</p>

      <div class="Chart Network">
        <div class="ChartHead">
          <span class="ChartLabel">{{ t("settings.performance.network") }}</span>
          <span class="Legend">
            <span class="Key">
              <span class="Swatch Ink" />
              {{ t("settings.performance.upload") }}
              <span class="KeyValue">{{ WireReady ? Rate(UpNow) : "—" }}</span>
            </span>
            <span class="Key">
              <span class="Swatch Rose" />
              {{ t("settings.performance.download") }}
              <span class="KeyValue">{{ WireReady ? Rate(DownNow) : "—" }}</span>
            </span>
          </span>
        </div>

        <svg class="Plot" :viewBox="`0 0 ${Span} 100`" preserveAspectRatio="none">
          <defs>
            <linearGradient id="perf-up" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0" stop-color="#f4f2f7" stop-opacity="0.12" />
              <stop offset="1" stop-color="#f4f2f7" stop-opacity="0" />
            </linearGradient>
            <linearGradient id="perf-down" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0" stop-color="#e0a8a8" stop-opacity="0.14" />
              <stop offset="1" stop-color="#e0a8a8" stop-opacity="0" />
            </linearGradient>
          </defs>
          <line v-for="Y in [25, 50, 75]" :key="Y" class="Grid" x1="0" :y1="Y" :x2="Span" :y2="Y" vector-effect="non-scaling-stroke" />
          <polygon v-if="DownPlot.Area" fill="url(#perf-down)" :points="DownPlot.Area" />
          <polygon v-if="UpPlot.Area" fill="url(#perf-up)" :points="UpPlot.Area" />
          <polyline v-if="DownPlot.Line" class="Line Rose" :points="DownPlot.Line" vector-effect="non-scaling-stroke" />
          <polyline v-if="UpPlot.Line" class="Line Ink" :points="UpPlot.Line" vector-effect="non-scaling-stroke" />
        </svg>

        <div class="Axis">
          <span>-{{ Span }} s</span>
          <span>{{ Rate(RateCeiling) }}</span>
        </div>
      </div>
    </div>

    <Group :Title="t('settings.performance.server')">
      <template #aside>{{ t("settings.performance.since") }}</template>
      <Row Block class="StatsRow">
        <Stats :Items="ServerTiles" />
      </Row>
    </Group>

    <Group :Title="t('settings.performance.voting')">
      <template #aside>{{ t("settings.performance.since") }}</template>
      <Row Block class="StatsRow">
        <Stats :Items="VotingTiles" />
      </Row>
      <Row :Label="t('settings.performance.search.average')">
        <template #end>
          <span class="Value Numeric">{{ Duration(Voting?.Average) }}</span>
        </template>
      </Row>
      <Row :Label="t('settings.performance.search.slow')">
        <template #end>
          <span class="Value Numeric">{{ Duration(Voting?.Slow) }}</span>
        </template>
      </Row>
      <Row :Label="t('settings.performance.search.failed')">
        <template #end>
          <span class="Value Numeric" :class="{ Bad: (Voting?.Failures ?? 0) > 0 }">{{ Count(Voting?.Failures) }}</span>
        </template>
      </Row>
      <Row :Label="t('settings.performance.limited')">
        <template #end>
          <span class="Value Numeric">{{ Count(Voting?.Limited) }}</span>
        </template>
      </Row>
      <Row :Label="t('settings.performance.active')">
        <template #end>
          <span class="Value Numeric">{{ ActiveText }}</span>
        </template>
      </Row>
    </Group>

    <Group :Title="t('settings.performance.smoothness')">
      <Row :Label="t('settings.performance.fps')">
        <template #end>
          <span class="Value Numeric">{{ Stat(Data?.Fps) }}</span>
        </template>
      </Row>
      <Row :Label="t('settings.performance.low')">
        <template #end>
          <span class="Value Numeric">{{ Stat(Data?.Low) }}</span>
        </template>
      </Row>
    </Group>
  </Pane>
</template>

<style scoped>
.Charts {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
}

.Chart {
  position: relative;
  min-width: 0;
  border-radius: 22px;
  background: rgba(255, 255, 255, 0.03);
  box-shadow: inset 0 0 0 1px var(--edge);
  padding: 18px 20px 14px;
}

.Network,
.Unsupported {
  grid-column: 1 / -1;
}

.Unsupported {
  margin: 0;
  border-radius: 22px;
  box-shadow: inset 0 0 0 1px var(--edge);
  padding: 18px 20px;
  font-size: 14px;
  color: var(--ink-mute);
}

.ChartHead {
  display: flex;
  min-height: 34px;
  flex-wrap: wrap;
  align-items: baseline;
  justify-content: space-between;
  gap: 6px 16px;
  margin-bottom: 12px;
}

.ChartLabel {
  font-size: 13px;
  color: var(--ink-mute);
}

.ChartValue {
  font-family: var(--display);
  font-size: 28px;
  font-weight: 600;
  letter-spacing: -0.02em;
  font-variant-numeric: tabular-nums;
  color: var(--ink);
}

.Legend {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 6px 22px;
}

.Key {
  display: inline-flex;
  align-items: baseline;
  gap: 8px;
  font-size: 13px;
  color: var(--ink-mute);
}

.Swatch {
  height: 8px;
  width: 8px;
  flex-shrink: 0;
  align-self: center;
  border-radius: 2px;
}

.Swatch.Ink {
  background: var(--ink);
}

.Swatch.Rose {
  background: var(--rose);
}

.KeyValue {
  min-width: 76px;
  font-family: var(--display);
  font-size: 20px;
  font-weight: 600;
  letter-spacing: -0.02em;
  font-variant-numeric: tabular-nums;
  color: var(--ink);
}

.Plot {
  display: block;
  height: 84px;
  width: 100%;
  overflow: visible;
}

.Grid {
  stroke: rgba(255, 255, 255, 0.06);
  stroke-width: 1;
}

.Line {
  fill: none;
  stroke-width: 1.75;
  stroke-linejoin: round;
  stroke-linecap: round;
}

.Line.Ink {
  stroke: var(--ink);
}

.Line.Rose {
  stroke: var(--rose);
}

.Axis {
  display: flex;
  justify-content: space-between;
  margin-top: 6px;
  font-size: 10.5px;
  font-variant-numeric: tabular-nums;
  color: var(--ink-faint);
}

.StatsRow {
  padding: 26px 0 24px !important;
}

.Numeric {
  font-variant-numeric: tabular-nums;
}

.Numeric.Bad {
  color: var(--danger);
}

@media (max-width: 1023px) {
  .Charts {
    grid-template-columns: minmax(0, 1fr);
  }

  .ChartLabel,
  .Key {
    font-size: 15px;
  }

  .ChartValue {
    font-size: 22px;
  }

  .KeyValue {
    font-size: 17px;
  }

  .Axis {
    font-size: 12px;
  }
}
</style>
