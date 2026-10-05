<script setup lang="ts">
import { ref, computed, onMounted, type Ref } from "vue"
import { useI18n } from "../../i18n"
import * as Api from "../../_api/slides"
import type { CollectionSummary } from "../Slides/model"
import Pane from "./ui/Pane.vue"
import Group from "./ui/Group.vue"
import Row from "./ui/Row.vue"
import Thumb from "../Slides/Thumb.vue"
import Editor from "../Slides/Editor.vue"

const { t } = useI18n()

const Emit = defineEmits<{ Changed: [] }>()

const Items: Ref<CollectionSummary[]> = ref([])
const Ready: Ref<boolean> = ref(false)
const Failed: Ref<boolean> = ref(false)
const Creating: Ref<boolean> = ref(false)
const Editing: Ref<string | null> = ref(null)
const Problem: Ref<string> = ref("")

const _Date = new Intl.DateTimeFormat("pl", { day: "numeric", month: "long", hour: "2-digit", minute: "2-digit" })

const Summary = computed(() => {
  if (!Items.value.length) return ""
  const Count = Items.value.length
  return `${Count} ${_Plural(Count, "kolekcja", "kolekcje", "kolekcji")}`
})

function _Plural(Count: number, One: string, Few: string, Many: string): string {
  if (Count === 1) return One
  const Ones = Count % 10
  const Tens = Count % 100
  return Ones >= 2 && Ones <= 4 && (Tens < 12 || Tens > 14) ? Few : Many
}

function Meta(Item: CollectionSummary): string {
  const Slides = `${Item.Count} ${_Plural(Item.Count, "slajd", "slajdy", "slajdów")}`
  return `${Slides} · ${t("settings.slides.updated")} ${_Date.format(new Date(Item.Updated * 1000))}`
}

async function _Refresh(): Promise<void> {
  try {
    const Before = Items.value.map(I => `${I.Id}:${I.Name}`).join("|")
    Items.value = await Api.List()
    const After = Items.value.map(I => `${I.Id}:${I.Name}`).join("|")
    if (Ready.value && Before !== After) Emit("Changed")
    Failed.value = false
  } catch {
    if (!Ready.value) Failed.value = true
  } finally {
    Ready.value = true
  }
}

function _NextName(): string {
  const Base = t("settings.slides.default")
  const Taken = new Set(Items.value.map(I => I.Name))
  if (!Taken.has(Base)) return Base
  let N = 2
  while (Taken.has(`${Base} ${N}`)) N += 1
  return `${Base} ${N}`
}

async function Create(): Promise<void> {
  if (Creating.value) return
  Creating.value = true
  Problem.value = ""
  try {
    const Made = await Api.Create(_NextName())
    await _Refresh()
    Editing.value = Made.Id
  } catch (Caught) {
    Problem.value = Caught instanceof Error ? Caught.message : t("settings.slides.error")
  } finally {
    Creating.value = false
  }
}

function Open(Id: string): void {
  Editing.value = Id
}

function OnClosed(): void {
  Editing.value = null
  void _Refresh()
}

onMounted(() => {
  void _Refresh()
})
</script>

<template>
  <Pane Id="slides">
    <Group :Title="t('settings.slides.collections')">
      <template #aside>
        <Transition name="fade">
          <span v-if="Summary" class="Summary">{{ Summary }}</span>
        </Transition>
        <button type="button" class="Btn Primary New" :disabled="Creating" @click="Create">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M12 5v14M5 12h14" /></svg>
          {{ t("settings.slides.new") }}
        </button>
      </template>

      <Row v-if="!Ready" Block>
        <div class="Grid">
          <div v-for="N in 2" :key="N" class="Card Ghost">
            <span class="GhostThumb" />
            <span class="GhostLine" />
          </div>
        </div>
      </Row>

      <Row v-else-if="Failed" :Label="t('settings.slides.failed')">
        <template #end>
          <button type="button" class="Btn" @click="_Refresh">{{ t("settings.plugins.retry") }}</button>
        </template>
      </Row>

      <Row v-else-if="!Items.length" Block>
        <div class="Empty">
          <p class="EmptyTitle">{{ t("settings.slides.empty") }}</p>
          <p class="EmptyText">{{ t("settings.slides.empty.detail") }}</p>
        </div>
      </Row>

      <Row v-else Block>
        <TransitionGroup tag="div" name="card" class="Grid">
          <button v-for="Item in Items" :key="Item.Id" type="button" class="Card" @click="Open(Item.Id)">
            <Thumb :Slide="Item.Cover" />
            <span class="CardText">
              <span class="CardName">{{ Item.Name }}</span>
              <span class="CardMeta">{{ Meta(Item) }}</span>
            </span>
          </button>
        </TransitionGroup>
      </Row>

      <template #footer>
        <span v-if="Problem" class="Fault">{{ Problem }}</span>
        <template v-else>{{ t("settings.slides.footer") }}</template>
      </template>
    </Group>

    <Editor v-if="Editing" :key="Editing" :Id="Editing" @Close="OnClosed" @Deleted="OnClosed" />
  </Pane>
</template>

<style scoped>
.Summary {
  font-variant-numeric: tabular-nums;
}

.New {
  display: inline-flex;
  height: 34px;
  align-items: center;
  gap: 6px;
  padding: 0 12px;
  font-size: 13px;
}

.New svg {
  height: 14px;
  width: 14px;
}

.Grid {
  position: relative;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
  gap: 18px 16px;
}

.Card {
  display: flex;
  min-width: 0;
  flex-direction: column;
  gap: 10px;
  border-radius: 14px;
  padding: 0;
  text-align: left;
  cursor: pointer;
}

.Card :deep(.Thumb) {
  transition: box-shadow 200ms ease, transform 320ms var(--ease);
}

.Card:hover :deep(.Thumb) {
  box-shadow: inset 0 0 0 1px var(--edge-strong), 0 18px 36px -22px rgba(0, 0, 0, 0.9);
  transform: translateY(-2px);
}

.Card:focus-visible {
  outline: 2px solid var(--ink);
  outline-offset: 4px;
}

.CardText {
  display: flex;
  min-width: 0;
  flex-direction: column;
  gap: 2px;
  padding: 0 2px;
}

.CardName {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 14.5px;
  font-weight: 500;
  color: var(--ink);
}

.CardMeta {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 12.5px;
  color: var(--ink-faint);
}

.Ghost {
  cursor: default;
}

.GhostThumb,
.GhostLine {
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.05);
  animation: Pulse 1400ms ease-in-out infinite;
}

.GhostThumb {
  width: 100%;
  aspect-ratio: 1776 / 698;
}

.GhostLine {
  height: 12px;
  width: 50%;
  border-radius: 6px;
}

@keyframes Pulse {
  50% { opacity: 0.45; }
}

.Empty {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 6px;
  padding: 6px 0;
}

.EmptyTitle {
  margin: 0;
  font-family: var(--display);
  font-size: 18px;
  font-weight: 600;
  letter-spacing: -0.02em;
}

.EmptyText {
  margin: 0;
  max-width: 480px;
  font-size: 13.5px;
  line-height: 1.55;
  color: var(--ink-mute);
}

.Fault {
  color: var(--danger);
}

.card-enter-active {
  transition: opacity 260ms ease, transform 360ms var(--ease);
}

.card-enter-from {
  opacity: 0;
  transform: translateY(8px);
}

.card-leave-active {
  display: none;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 200ms ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

@media (prefers-reduced-motion: reduce) {
  .GhostThumb,
  .GhostLine {
    animation: none;
  }
}
</style>
