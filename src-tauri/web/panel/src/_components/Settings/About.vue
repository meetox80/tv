<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, type Ref } from "vue"
import { useI18n } from "../../i18n"
import type { SystemInfo } from "../../_api/settings"
import Pane from "./ui/Pane.vue"
import Group from "./ui/Group.vue"
import Row from "./ui/Row.vue"

const { t } = useI18n()

const Props = defineProps<{ System: SystemInfo | null }>()

const _TickMs = 30 * 1000
const _Number = new Intl.NumberFormat("pl", { maximumFractionDigits: 1 })

const Now: Ref<number> = ref(Date.now() / 1000)

let _Timer: ReturnType<typeof setInterval> | null = null

const Uptime = computed(() => {
  const Started = Props.System?.StartedAt ?? 0
  if (!Started) return "—"

  const Minutes = Math.max(0, Math.floor((Now.value - Started) / 60))
  if (Minutes < 1) return t("settings.about.uptime.now")

  const Days = Math.floor(Minutes / 1440)
  const Hours = Math.floor((Minutes % 1440) / 60)
  const Rest = Minutes % 60

  if (Days > 0) {
    const Unit = Days === 1 ? t("settings.about.uptime.day") : t("settings.about.uptime.days")
    return `${Days} ${Unit} ${Hours} ${t("settings.about.uptime.hours")}`
  }
  if (Hours > 0) return `${Hours} ${t("settings.about.uptime.hours")} ${Rest} ${t("settings.about.uptime.minutes")}`
  return `${Rest} ${t("settings.about.uptime.minutes")}`
})

const Database = computed(() => {
  const Bytes = Props.System?.DatabaseBytes ?? 0
  if (!Bytes) return "—"
  if (Bytes < 1024 * 1024) return `${_Number.format(Bytes / 1024)} KB`
  if (Bytes < 1024 * 1024 * 1024) return `${_Number.format(Bytes / 1024 / 1024)} MB`
  return `${_Number.format(Bytes / 1024 / 1024 / 1024)} GB`
})

onMounted(() => {
  _Timer = setInterval(() => { Now.value = Date.now() / 1000 }, _TickMs)
})

onUnmounted(() => {
  if (_Timer) clearInterval(_Timer)
})
</script>

<template>
  <Pane Id="about">
    <Group>
      <Row :Label="t('settings.about.version')">
        <template #end>
          <span class="Value Monospace">{{ System?.Version || "—" }}</span>
        </template>
      </Row>
      <Row :Label="t('settings.about.address')">
        <template #end>
          <a v-if="System?.Address" :href="System.Address" target="_blank" rel="noopener noreferrer" class="Link Monospace Address">{{ System.Address }}</a>
          <span v-else class="Value">—</span>
        </template>
      </Row>
      <Row :Label="t('settings.about.uptime')">
        <template #end>
          <span class="Value Numeric">{{ Uptime }}</span>
        </template>
      </Row>
      <Row :Label="t('settings.about.database')">
        <template #end>
          <span class="Value Numeric">{{ Database }}</span>
        </template>
      </Row>
    </Group>
  </Pane>
</template>

<style scoped>
.Address {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  user-select: text;
}

.Numeric {
  font-variant-numeric: tabular-nums;
}
</style>
