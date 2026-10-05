<script setup lang="ts">
import { ref, watch, type Ref } from "vue"
import { useI18n } from "../../i18n"
import * as Api from "../../_api/screen"
import type { LessonDuration } from "../../_api/screen"
import Sheet from "./ui/Sheet.vue"

const { t } = useI18n()

const Props = defineProps<{
  Open: boolean
  Duration: LessonDuration
}>()

const Emit = defineEmits<{ Close: [] }>()

interface Row {
  Number: number
  Lesson: string
  Break: string
}

const Rows: Ref<Row[]> = ref([])

watch(
  () => [Props.Open, Props.Duration] as const,
  async ([Open, Duration]) => {
    if (!Open) return

    try {
      const Plan = await Api.LoadLessons(Duration)
      Rows.value = Plan.LessonTimes.map((Lesson, Index) => ({
        Number: Index + 1,
        Lesson,
        Break: Plan.BreakTimes[Index] ?? "",
      }))
    } catch {}
  },
  { immediate: true }
)
</script>

<template>
  <Sheet :Open="Open" :Title="`${t('settings.lessons.times')} · ${Duration} min`" :Width="460" @Close="Emit('Close')">
    <div class="Plan">
      <div class="Line Head">
        <span />
        <span>{{ t("settings.lessons.lesson") }}</span>
        <span>{{ t("settings.lessons.break") }}</span>
      </div>

      <div v-for="Row in Rows" :key="Row.Number" class="Line">
        <span class="Num">{{ Row.Number }}</span>
        <span class="Time">{{ Row.Lesson }}</span>
        <span class="Time Soft">{{ Row.Break }}</span>
      </div>
    </div>

    <template #actions>
      <button type="button" class="Btn" @click="Emit('Close')">{{ t("settings.close") }}</button>
    </template>
  </Sheet>
</template>

<style scoped>
.Plan {
  max-height: min(56vh, 480px);
  overflow-y: auto;
  margin: -4px -4px 0;
  padding: 0 4px;
  scrollbar-width: thin;
}

.Line {
  display: grid;
  grid-template-columns: 32px 1fr 1fr;
  align-items: center;
  gap: 12px;
  border-bottom: 1px solid var(--line);
  padding: 10px 0;
}

.Line:last-child {
  border-bottom: 0;
}

.Head {
  position: sticky;
  top: 0;
  z-index: 1;
  background: #16151a;
  padding: 6px 0 8px;
  font-size: 12px;
  color: var(--ink-faint);
}

.Num {
  font-size: 13px;
  font-variant-numeric: tabular-nums;
  color: var(--ink-faint);
}

.Time {
  font-size: 14.5px;
  font-variant-numeric: tabular-nums;
  color: var(--ink);
}

.Time.Soft {
  color: var(--ink-mute);
}
</style>
