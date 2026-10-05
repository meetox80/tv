<script setup lang="ts">
import { ref, computed, onUnmounted, type Ref } from "vue"
import { useI18n } from "../../i18n"
import { Copy } from "../../_api/clipboard"
import type { LessonDuration } from "../../_api/screen"
import Sheet from "./ui/Sheet.vue"

const { t } = useI18n()

const Props = defineProps<{
  Open: boolean
  Mode: "api" | "serial"
  Duration: LessonDuration
  ApiKey: string
}>()

const Emit = defineEmits<{ Close: [] }>()

const Copied: Ref<boolean> = ref(false)

let _CopiedTimer: ReturnType<typeof setTimeout> | null = null

const Origin = computed(() => globalThis.location?.origin || "http://localhost:9999")

const Target = computed<LessonDuration>(() => (Props.Duration === 45 ? 30 : 45))

const Title = computed(() =>
  `${t("settings.lessons.remote")} · ${Props.Mode === "api" ? t("settings.scrobbling.api") : t("settings.serial")}`
)

const Snippet = computed(() => {
  if (Props.Mode === "api") {
    return [
      `curl -X POST "${Origin.value}/api/lessons/poke?key=${Props.ApiKey}" \\`,
      `  -H "Content-Type: application/json" \\`,
      `  -d '{"Duration": ${Target.value}}'`,
    ].join("\n")
  }

  return [
    `{`,
    `  "Seq": ${Date.now()},`,
    `  "At": ${Math.floor(Date.now() / 1000)},`,
    `  "Lesson": ${Target.value}`,
    `}`,
  ].join("\n")
})

const DocsUrl = computed(() =>
  Props.Mode === "api" ? "/llms.txt?documentation=lessons" : "/llms.txt?documentation=serial"
)

async function CopySnippet(): Promise<void> {
  await Copy(Snippet.value)
  Copied.value = true
  if (_CopiedTimer) clearTimeout(_CopiedTimer)
  _CopiedTimer = setTimeout(() => { Copied.value = false }, 1600)
}

onUnmounted(() => {
  if (_CopiedTimer) clearTimeout(_CopiedTimer)
})
</script>

<template>
  <Sheet
    :Open="Open"
    :Title="Title"
    :Message="Mode === 'api' ? t('settings.lessons.remote.api') : t('settings.lessons.remote.serial')"
    :Width="580"
    @Close="Emit('Close')"
  >
    <pre class="Code">{{ Snippet }}</pre>

    <template #actions>
      <a :href="DocsUrl" target="_blank" rel="noopener" class="Btn">{{ t("settings.docs") }}</a>
      <button type="button" class="Btn Primary" @click="CopySnippet">
        {{ Copied ? t("settings.copied") : t("settings.copy") }}
      </button>
    </template>
  </Sheet>
</template>

<style scoped>
.Code {
  margin: 0;
  overflow-x: auto;
  border-radius: 14px;
  background: rgba(255, 255, 255, 0.03);
  box-shadow: inset 0 0 0 1px var(--edge);
  padding: 14px 16px;
  font-family: var(--mono);
  font-size: 12.5px;
  line-height: 1.7;
  white-space: pre;
  color: var(--ink-soft);
  scrollbar-width: thin;
}
</style>
