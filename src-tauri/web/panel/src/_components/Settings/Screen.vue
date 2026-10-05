<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, type Ref } from "vue"
import { useI18n } from "../../i18n"
import * as Api from "../../_api/screen"
import * as Cache from "../../_api/cache"
import LocationPicker from "./LocationPicker.vue"
import LessonTimes from "./LessonTimes.vue"
import LessonRemote from "./LessonRemote.vue"
import Rotation from "./Rotation.vue"
import type { LessonDuration, ScreenPage } from "../../_api/screen"
import Pane from "./ui/Pane.vue"
import Group from "./ui/Group.vue"
import Row from "./ui/Row.vue"
import Switch from "./ui/Switch.vue"
import Segmented from "./ui/Segmented.vue"

const { t } = useI18n()

const _CacheKey = "panel_screen_cache"
const _Cached = Cache.Read<Api.ScreenConfig>(_CacheKey)

const Enabled: Ref<boolean> = ref(_Cached?.Weather.Enabled ?? false)
const Latitude: Ref<number> = ref(_Cached?.Weather.Latitude ?? 50.04)
const Longitude: Ref<number> = ref(_Cached?.Weather.Longitude ?? 22.62)
const PickerOpen: Ref<boolean> = ref(false)
const Duration: Ref<LessonDuration> = ref(_Cached?.Lessons?.Duration ?? 45)
const TimesOpen: Ref<boolean> = ref(false)
const RemoteOpen: Ref<boolean> = ref(false)
const RemoteMode: Ref<"api" | "serial"> = ref("api")
const Notice: Ref<string> = ref(_Cached?.Notice ?? "")
const NoticeSaving: Ref<boolean> = ref(false)
const NoticeStored: Ref<string> = ref(_Cached?.Notice ?? "")
const NoticeSaved: Ref<boolean> = ref(false)
const Pages: Ref<ScreenPage[]> = ref(_Cached?.Pages ?? [
  { Id: "lesson", Enabled: true, Seconds: 30 },
  { Id: "news", Enabled: true, Seconds: 30 },
  { Id: "voting", Enabled: true, Seconds: 30 },
])

let _SavedTimer: ReturnType<typeof setTimeout> | null = null

const _Durations: LessonDuration[] = [45, 30]

const Props = defineProps<{ ApiKey: string; VotingOn: boolean; Revision: number }>()

const Emit = defineEmits<{ ToggleVoting: [] }>()

function OpenRemote(Mode: "api" | "serial"): void {
  RemoteMode.value = Mode
  RemoteOpen.value = true
}

function _Degrees(Value: number, Positive: string, Negative: string): string {
  const Text = Math.abs(Value).toFixed(2).replace(".", ",")
  return `${Text}° ${Value < 0 ? Negative : Positive}`
}

const Coords = computed(() => `${_Degrees(Latitude.value, "N", "S")}, ${_Degrees(Longitude.value, "E", "W")}`)

const DurationOptions = computed(() => _Durations.map(Minutes => ({ Value: Minutes, Label: `${Minutes} min` })))

const NoticeDirty = computed(() => Notice.value.trim() !== NoticeStored.value.trim())

function _Snapshot(): Api.ScreenConfig {
  return {
    Weather: {
      Enabled: Enabled.value,
      Latitude: Latitude.value,
      Longitude: Longitude.value,
    },
    Lessons: {
      Duration: Duration.value,
    },
    Notice: Notice.value,
    Pages: Pages.value,
  }
}

async function _Persist(): Promise<void> {
  Cache.Write(_CacheKey, _Snapshot())
  try {
    await Api.SaveWeather(_Snapshot().Weather)
  } catch {}
}

async function Toggle(): Promise<void> {
  Enabled.value = !Enabled.value
  await _Persist()
}

async function SaveNotice(): Promise<void> {
  if (NoticeSaving.value) return
  NoticeSaving.value = true

  try {
    const Text = Notice.value.trim()
    await Api.SaveNotice(Text)
    Notice.value = Text
    NoticeStored.value = Text
    Cache.Write(_CacheKey, _Snapshot())
    NoticeSaved.value = true
    if (_SavedTimer) clearTimeout(_SavedTimer)
    _SavedTimer = setTimeout(() => { NoticeSaved.value = false }, 1800)
  } catch {} finally {
    NoticeSaving.value = false
  }
}

async function SetDuration(Minutes: LessonDuration): Promise<void> {
  if (Duration.value === Minutes) return

  Duration.value = Minutes
  Cache.Write(_CacheKey, _Snapshot())

  try {
    await Api.SaveLessons(_Snapshot().Lessons)
  } catch {}
}

async function _ReloadPages(): Promise<void> {
  try {
    const Data = await Api.Load()
    if (Data.Pages?.length) Pages.value = Data.Pages
    Cache.Write(_CacheKey, _Snapshot())
  } catch {}
}

async function SavePages(Next: ScreenPage[]): Promise<void> {
  Pages.value = Next
  Cache.Write(_CacheKey, _Snapshot())

  try {
    await Api.SavePages(Next)
  } catch {
    await _ReloadPages()
  }
}

watch(() => Props.Revision, () => void _ReloadPages())

async function OnPick(At: { Latitude: number; Longitude: number }): Promise<void> {
  Latitude.value = At.Latitude
  Longitude.value = At.Longitude
  PickerOpen.value = false
  await _Persist()
}

onMounted(async () => {
  try {
    const Data = await Api.Load()
    Enabled.value = Data.Weather.Enabled
    Latitude.value = Data.Weather.Latitude
    Longitude.value = Data.Weather.Longitude
    Duration.value = Data.Lessons?.Duration ?? 45
    Notice.value = Data.Notice ?? ""
    NoticeStored.value = Data.Notice ?? ""
    if (Data.Pages?.length) Pages.value = Data.Pages
    Cache.Write(_CacheKey, Data)
  } catch {}
})

onUnmounted(() => {
  if (_SavedTimer) clearTimeout(_SavedTimer)
})
</script>

<template>
  <Pane Id="screen">
    <Rotation :Pages="Pages" :VotingOn="Props.VotingOn" @Change="SavePages" @ToggleVoting="Emit('ToggleVoting')" />

    <Group :Title="t('settings.weather')" Id="screen-weather">
      <Row :Label="t('settings.weather.show')">
        <template #end>
          <Switch :On="Enabled" :Label="t('settings.weather.show')" @Toggle="Toggle" />
        </template>
      </Row>
      <Row :Label="t('settings.weather.location')" Chevron @Press="PickerOpen = true">
        <template #end>
          <span class="Value Coords">{{ Coords }}</span>
        </template>
      </Row>
    </Group>

    <Group :Title="t('settings.lessons')" Id="screen-lessons">
      <Row :Label="t('settings.lessons.duration')">
        <template #end>
          <Segmented :Options="DurationOptions" :Value="Duration" @Change="SetDuration" />
        </template>
      </Row>
      <Row :Label="t('settings.lessons.times')" Chevron @Press="TimesOpen = true" />
      <Row :Label="t('settings.lessons.remote')" :Detail="t('settings.lessons.remote.detail')">
        <template #end>
          <button type="button" class="Btn" @click="OpenRemote('api')">{{ t("settings.spotify.api") }}</button>
          <button type="button" class="Btn" @click="OpenRemote('serial')">{{ t("settings.serial") }}</button>
        </template>
      </Row>
    </Group>

    <Group :Title="t('settings.notice')" Id="screen-notice">
      <Row Block>
        <div class="NoticeField">
          <textarea
            v-model="Notice"
            rows="1"
            maxlength="300"
            :placeholder="t('settings.notice.placeholder')"
            class="Input Area"
            @keydown.enter.exact.prevent="NoticeDirty && SaveNotice()"
          />
          <span class="Count" :class="{ Near: Notice.length > 260 }">{{ Notice.length }}/300</span>
        </div>
      </Row>

      <template #footer>{{ t("settings.notice.footer") }}</template>
      <template #action>
        <button type="button" class="Btn Primary" :disabled="(!NoticeDirty && !NoticeSaved) || NoticeSaving" @click="SaveNotice">
          {{ NoticeSaved ? t("settings.saved") : t("settings.save") }}
        </button>
      </template>
    </Group>

    <LessonTimes :Open="TimesOpen" :Duration="Duration" @Close="TimesOpen = false" />

    <LessonRemote
      :Open="RemoteOpen"
      :Mode="RemoteMode"
      :Duration="Duration"
      :ApiKey="Props.ApiKey"
      @Close="RemoteOpen = false"
    />

    <LocationPicker
      :Open="PickerOpen"
      :Latitude="Latitude"
      :Longitude="Longitude"
      @Close="PickerOpen = false"
      @Pick="OnPick"
    />
  </Pane>
</template>

<style scoped>
.Coords {
  font-variant-numeric: tabular-nums;
}

.NoticeField {
  position: relative;
}

.Area {
  display: block;
  height: auto;
  min-height: 96px;
  resize: none;
  field-sizing: content;
  padding: 14px 16px 34px;
  font-size: 15px;
  line-height: 1.5;
}

.Count {
  position: absolute;
  right: 14px;
  bottom: 11px;
  font-size: 12px;
  font-variant-numeric: tabular-nums;
  color: var(--ink-faint);
  pointer-events: none;
  transition: color 200ms ease;
}

.Count.Near {
  color: var(--rose);
}
</style>
