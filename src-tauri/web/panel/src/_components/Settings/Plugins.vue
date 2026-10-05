<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, type Ref } from "vue"
import { useI18n } from "../../i18n"
import { Copy } from "../../_api/clipboard"
import * as Api from "../../_api/extensions"
import type { ExtensionInfo } from "../../_api/extensions"
import * as Extensions from "../../extensions"
import Pane from "./ui/Pane.vue"
import Group from "./ui/Group.vue"
import Row from "./ui/Row.vue"
import Switch from "./ui/Switch.vue"
import Glyph from "./ui/Glyph.vue"

const { t } = useI18n()

const _SpinMs = 650
const _Settle = [700, 5600]
const _CopiedMs = 1600

const Items: Ref<ExtensionInfo[]> = ref([])
const Folder: Ref<string> = ref("")
const Ready: Ref<boolean> = ref(false)
const Failed: Ref<boolean> = ref(false)
const Rescanning: Ref<boolean> = ref(false)
const Busy: Ref<Set<string>> = ref(new Set())
const Expanded: Ref<Set<string>> = ref(new Set())
const LoadErrors: Ref<Record<string, string>> = ref({})
const Copied: Ref<boolean> = ref(false)

let _Timers: ReturnType<typeof setTimeout>[] = []
let _CopiedTimer: ReturnType<typeof setTimeout> | null = null

const EnabledCount = computed(() => Items.value.filter(I => I.Enabled).length)

const Summary = computed(() => {
  if (!Ready.value || !Items.value.length) return ""
  return `${t("settings.plugins.enabled")}: ${EnabledCount.value} ${t("settings.plugins.of")} ${Items.value.length}`
})

function _Apply(List: Api.ExtensionList): void {
  Folder.value = List.Folder
  Items.value = List.Items.map(Fresh => {
    const Pending = Busy.value.has(Fresh.Id)
    const Known = Items.value.find(I => I.Id === Fresh.Id)
    return Pending && Known ? { ...Fresh, Enabled: Known.Enabled } : Fresh
  })
  Failed.value = false
}

async function _Refresh(): Promise<void> {
  try {
    _Apply(await Api.List())
  } catch {
    if (!Ready.value) Failed.value = true
  } finally {
    Ready.value = true
  }
}

function _Later(Run: () => void, Ms: number): void {
  _Timers.push(setTimeout(Run, Ms))
}

async function Rescan(): Promise<void> {
  if (Rescanning.value) return
  Rescanning.value = true
  const Started = performance.now()

  try {
    _Apply(await Api.Reload())
  } catch {
    Failed.value = !Items.value.length
  } finally {
    const Left = Math.max(0, _SpinMs - (performance.now() - Started))
    _Later(() => { Rescanning.value = false }, Left)
  }
}

function Problem(Item: ExtensionInfo): { Head: string; Rest: string } | null {
  const Text = LoadErrors.value[Item.Id] ?? Item.Error
  if (!Text) return null
  const [Head, ...Rest] = Text.split("\n")
  return { Head, Rest: Rest.join("\n").trim() }
}

function Stale(Item: ExtensionInfo): boolean {
  return Extensions.Stale.has(Item.Id)
}

function ToggleDetails(Id: string): void {
  const Next = new Set(Expanded.value)
  if (Next.has(Id)) Next.delete(Id)
  else Next.add(Id)
  Expanded.value = Next
}

function _SetBusy(Id: string, On: boolean): void {
  const Next = new Set(Busy.value)
  if (On) Next.add(Id)
  else Next.delete(Id)
  Busy.value = Next
}

async function Toggle(Item: ExtensionInfo): Promise<void> {
  if (Busy.value.has(Item.Id)) return
  const Next = !Item.Enabled

  _SetBusy(Item.Id, true)
  Item.Enabled = Next
  const Errors = { ...LoadErrors.value }
  delete Errors[Item.Id]
  LoadErrors.value = Errors

  try {
    await Api.SetEnabled(Item.Id, Next)
  } catch {
    Item.Enabled = !Next
    LoadErrors.value = { ...LoadErrors.value, [Item.Id]: t("settings.plugins.toggle.error") }
    _SetBusy(Item.Id, false)
    return
  }

  if (Next) {
    try {
      await Extensions.Enable(Item)
    } catch (Caught) {
      const Detail = Caught instanceof Error ? Caught.message : String(Caught)
      LoadErrors.value = { ...LoadErrors.value, [Item.Id]: `${t("settings.plugins.load.error")}\n${Detail}` }
    }
  } else {
    Extensions.Disable(Item.Id)
  }

  _SetBusy(Item.Id, false)
  for (const Ms of _Settle) _Later(() => void _Refresh(), Ms)
}

async function CopyFolder(): Promise<void> {
  if (!Folder.value) return
  await Copy(Folder.value)
  Copied.value = true
  if (_CopiedTimer) clearTimeout(_CopiedTimer)
  _CopiedTimer = setTimeout(() => { Copied.value = false }, _CopiedMs)
}

function ReloadPanel(): void {
  location.reload()
}

onMounted(() => {
  void _Refresh()
})

onUnmounted(() => {
  _Timers.forEach(clearTimeout)
  _Timers = []
  if (_CopiedTimer) clearTimeout(_CopiedTimer)
})
</script>

<template>
  <Pane Id="plugins">
    <Group :Title="t('settings.plugins.installed')">
      <template #aside>
        <Transition name="fade">
          <span v-if="Summary" class="Summary">{{ Summary }}</span>
        </Transition>
        <button
          type="button"
          class="Rescan"
          :class="{ Spinning: Rescanning }"
          :disabled="Rescanning"
          @click="Rescan"
        >
          <Glyph Name="refresh" class="RescanIcon" />
          {{ t("settings.plugins.rescan") }}
        </button>
      </template>

      <template v-if="!Ready">
        <div v-for="N in 2" :key="N" class="Ghost">
          <span class="GhostLine Wide" />
          <span class="GhostLine" />
          <span class="GhostSwitch" />
        </div>
      </template>

      <Row v-else-if="Failed" :Label="t('settings.plugins.failed')" :Detail="t('settings.plugins.failed.detail')">
        <template #end>
          <button type="button" class="Btn" @click="Rescan">{{ t("settings.plugins.retry") }}</button>
        </template>
      </Row>

      <Row v-else-if="!Items.length" Block>
        <div class="Empty">
          <p class="EmptyTitle">{{ t("settings.plugins.empty") }}</p>
          <p class="EmptyText">{{ t("settings.plugins.empty.detail") }}</p>
          <div class="Path">
            <span class="PathText Monospace">{{ Folder }}</span>
            <button type="button" class="PathCopy" :aria-label="t('settings.copy')" @click="CopyFolder">
              <Glyph :Name="Copied ? 'check' : 'copy'" />
            </button>
          </div>
        </div>
      </Row>

      <TransitionGroup v-else tag="div" name="ext" class="Items">
        <Row v-for="Item in Items" :key="Item.Id" class="Ext" :class="{ Off: !Item.Enabled }">
          <template #label>
            <p class="Name">
              <span class="NameText">{{ Item.Name }}</span>
              <span class="Version">{{ Item.Version }}</span>
            </p>
          </template>

          <template #detail>
            <span v-if="Item.Description" class="Description">{{ Item.Description }}</span>
            <span class="Meta">
              <span class="Monospace">{{ Item.Id }}</span>
              <span v-if="Item.Panel" class="Part">{{ t("settings.plugins.part.panel") }}</span>
              <span v-if="Item.Server" class="Part">{{ t("settings.plugins.part.server") }}</span>
            </span>
          </template>

          <template #end>
            <Switch
              :On="Item.Enabled"
              :Disabled="Busy.has(Item.Id)"
              :Label="`${t('settings.plugins.toggle')}: ${Item.Name}`"
              @Toggle="Toggle(Item)"
            />
          </template>

          <template v-if="Problem(Item) || (Stale(Item) && !Item.Enabled)" #below>
            <Transition name="note" appear>
              <div v-if="Problem(Item)" :key="`problem-${Item.Id}`" class="Note Bad">
                <div class="NoteHead">
                  <span class="NoteText">{{ Problem(Item)?.Head }}</span>
                  <button
                    v-if="Problem(Item)?.Rest"
                    type="button"
                    class="NoteAction"
                    :aria-expanded="Expanded.has(Item.Id)"
                    @click="ToggleDetails(Item.Id)"
                  >
                    {{ Expanded.has(Item.Id) ? t("settings.plugins.details.hide") : t("settings.plugins.details.show") }}
                  </button>
                </div>
                <div class="Fold" :class="{ Open: Expanded.has(Item.Id) }">
                  <div class="FoldInner">
                    <pre class="Trace">{{ Problem(Item)?.Rest }}</pre>
                  </div>
                </div>
              </div>

              <div v-else :key="`stale-${Item.Id}`" class="Note">
                <div class="NoteHead">
                  <span class="NoteText">{{ t("settings.plugins.reload.needed") }}</span>
                  <button type="button" class="NoteAction" @click="ReloadPanel">{{ t("settings.plugins.reload") }}</button>
                </div>
              </div>
            </Transition>
          </template>
        </Row>
      </TransitionGroup>

      <template #footer>
        {{ t("settings.plugins.safe") }}
        <a href="/safe" class="SafeLink">{{ t("settings.plugins.safe.link") }}</a>.
      </template>
    </Group>
  </Pane>
</template>

<style scoped>
.Summary {
  font-variant-numeric: tabular-nums;
}

.Rescan {
  display: inline-flex;
  height: 32px;
  align-items: center;
  gap: 7px;
  border-radius: 9px;
  padding: 0 10px;
  font-size: 13px;
  color: var(--ink-soft);
  cursor: pointer;
  transition: background 160ms ease, color 160ms ease;
}

.Rescan:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.06);
  color: var(--ink);
}

.Rescan:disabled {
  cursor: default;
}

.RescanIcon {
  height: 14px;
  width: 14px;
}

.Spinning .RescanIcon {
  animation: Spin 650ms cubic-bezier(0.45, 0.05, 0.35, 1) infinite;
}

@keyframes Spin {
  to { transform: rotate(360deg); }
}

.Ghost {
  position: relative;
  display: grid;
  min-height: 84px;
  grid-template-columns: minmax(0, 1fr) auto;
  grid-template-rows: auto auto;
  align-content: center;
  column-gap: 24px;
  row-gap: 10px;
  padding: 16px 0;
}

.Ghost::before {
  content: "";
  position: absolute;
  top: 0;
  right: 0;
  left: 0;
  border-top: 1px solid var(--line);
}

.GhostLine,
.GhostSwitch {
  border-radius: 6px;
  background: rgba(255, 255, 255, 0.05);
  animation: Pulse 1400ms ease-in-out infinite;
}

.GhostLine {
  height: 12px;
  width: 38%;
}

.GhostLine.Wide {
  height: 15px;
  width: 24%;
}

.GhostSwitch {
  grid-column: 2;
  grid-row: 1 / span 2;
  align-self: center;
  height: 28px;
  width: 46px;
  border-radius: 999px;
}

@keyframes Pulse {
  50% { opacity: 0.45; }
}

.Items {
  position: relative;
}

.Name {
  display: flex;
  min-width: 0;
  align-items: baseline;
  gap: 10px;
  margin: 0;
  font-size: 15px;
  font-weight: 500;
  line-height: 1.35;
  letter-spacing: -0.01em;
  color: var(--ink);
}

.NameText {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.Version {
  flex-shrink: 0;
  font-size: 12.5px;
  font-weight: 400;
  font-variant-numeric: tabular-nums;
  color: var(--ink-faint);
}

.Description {
  display: block;
  color: var(--ink-mute);
}

.Meta {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 6px 10px;
  margin-top: 6px;
  font-size: 12.5px;
  color: var(--ink-faint);
}

.Part {
  border-radius: 6px;
  box-shadow: inset 0 0 0 1px var(--edge);
  padding: 1px 7px 2px;
  font-size: 11.5px;
  font-weight: 500;
  color: var(--ink-mute);
}

.Ext .Name,
.Ext .Description,
.Ext .Meta {
  transition: opacity 240ms ease;
}

.Ext.Off .Name,
.Ext.Off .Description,
.Ext.Off .Meta {
  opacity: 0.55;
}

.Note {
  border-radius: 12px;
  background: rgba(255, 255, 255, 0.035);
  box-shadow: inset 0 0 0 1px var(--line);
  padding: 11px 14px;
}

.Note.Bad {
  background: rgba(239, 138, 138, 0.06);
  box-shadow: inset 0 0 0 1px rgba(239, 138, 138, 0.16);
}

.NoteHead {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  justify-content: space-between;
  gap: 6px 16px;
}

.NoteText {
  min-width: 0;
  font-size: 13.5px;
  line-height: 1.5;
  overflow-wrap: anywhere;
  color: var(--ink-soft);
}

.Bad .NoteText {
  color: var(--danger);
}

.NoteAction {
  flex-shrink: 0;
  font-size: 13px;
  font-weight: 500;
  color: var(--ink-soft);
  cursor: pointer;
  transition: color 150ms ease;
}

.NoteAction:hover {
  color: var(--ink);
}

.Fold {
  display: grid;
  grid-template-rows: 0fr;
  opacity: 0;
  transition: grid-template-rows 360ms var(--ease), opacity 240ms ease;
}

.Fold.Open {
  grid-template-rows: 1fr;
  opacity: 1;
}

.FoldInner {
  min-height: 0;
  overflow: hidden;
}

.Trace {
  max-height: 240px;
  margin: 10px 0 0;
  overflow: auto;
  border-top: 1px solid rgba(239, 138, 138, 0.14);
  padding-top: 10px;
  font-family: var(--mono);
  font-size: 12px;
  line-height: 1.6;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
  color: rgba(239, 138, 138, 0.8);
  scrollbar-width: thin;
}

.Empty {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  padding: 8px 0 4px;
}

.EmptyTitle {
  margin: 0;
  font-family: var(--display);
  font-size: 18px;
  font-weight: 600;
  letter-spacing: -0.02em;
  color: var(--ink);
}

.EmptyText {
  margin: 6px 0 0;
  max-width: 460px;
  font-size: 13.5px;
  line-height: 1.55;
  color: var(--ink-mute);
}

.Path {
  display: flex;
  max-width: 100%;
  align-items: center;
  gap: 4px;
  margin-top: 14px;
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.035);
  box-shadow: inset 0 0 0 1px var(--edge);
  padding: 4px 4px 4px 12px;
}

.PathText {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--ink-soft);
}

.PathCopy {
  display: flex;
  height: 30px;
  width: 30px;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  border-radius: 7px;
  color: var(--ink-mute);
  cursor: pointer;
  transition: background 150ms ease, color 150ms ease;
}

.PathCopy svg {
  height: 15px;
  width: 15px;
}

.PathCopy:hover {
  background: rgba(255, 255, 255, 0.06);
  color: var(--ink);
}

.SafeLink {
  color: var(--ink-soft);
  text-decoration: underline;
  text-decoration-color: rgba(255, 255, 255, 0.2);
  text-underline-offset: 3px;
  transition: color 150ms ease, text-decoration-color 150ms ease;
}

.SafeLink:hover {
  color: var(--ink);
  text-decoration-color: currentColor;
}

.ext-move {
  transition: transform 360ms var(--ease);
}

.ext-enter-active {
  transition: opacity 280ms ease, transform 380ms var(--ease);
}

.ext-leave-active {
  position: absolute;
  right: 0;
  left: 0;
  transition: opacity 180ms ease;
}

.ext-enter-from {
  opacity: 0;
  transform: translateY(8px);
}

.ext-leave-to {
  opacity: 0;
}

.note-enter-active {
  transition: opacity 260ms ease, transform 320ms var(--ease);
}

.note-leave-active {
  transition: opacity 160ms ease;
}

.note-enter-from {
  opacity: 0;
  transform: translateY(-4px);
}

.note-leave-to {
  opacity: 0;
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
  .Spinning .RescanIcon,
  .GhostLine,
  .GhostSwitch {
    animation: none;
  }

  .ext-move,
  .ext-enter-active,
  .note-enter-active,
  .Fold {
    transition: opacity 120ms ease;
  }

  .ext-enter-from,
  .note-enter-from {
    transform: none;
  }
}
</style>
