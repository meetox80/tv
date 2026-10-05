<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, type Ref } from "vue"
import { useI18n } from "../../i18n"
import * as Api from "../../_api/tunnel"
import type { HostKind } from "../../_api/tunnel"
import Pane from "./ui/Pane.vue"
import Group from "./ui/Group.vue"
import Row from "./ui/Row.vue"
import Glyph from "./ui/Glyph.vue"

const { t } = useI18n()

const _Kinds: HostKind[] = ["management", "voting"]
const _SavedMs = 2200

const Stored: Ref<Record<HostKind, string>> = ref({ management: "", voting: "" })
const Draft: Ref<Record<HostKind, string>> = ref({ management: "", voting: "" })
const Busy: Ref<HostKind | null> = ref(null)
const Fault: Ref<Record<HostKind, string>> = ref({ management: "", voting: "" })
const Saved: Ref<HostKind | null> = ref(null)
const Local: Ref<boolean> = ref(true)
const Ready: Ref<boolean> = ref(false)

let _SavedTimer: ReturnType<typeof setTimeout> | null = null

function _Bare(Text: string): string {
  return Text.trim().replace(/^https?:\/\//i, "").split("/")[0].toLowerCase()
}

function Locked(Kind: HostKind): boolean {
  return Kind === "management" && !Local.value
}

const Dirty = computed(() => ({
  management: _Bare(Draft.value.management) !== Stored.value.management,
  voting: _Bare(Draft.value.voting) !== Stored.value.voting,
}))

function Edit(Kind: HostKind, Value: string): void {
  Draft.value[Kind] = Value
  Fault.value[Kind] = ""
}

function Reset(Kind: HostKind): void {
  Draft.value[Kind] = Stored.value[Kind]
  Fault.value[Kind] = ""
}

async function Save(Kind: HostKind, Force = false): Promise<void> {
  if (Busy.value || Locked(Kind) || !Dirty.value[Kind]) return

  Busy.value = Kind
  Fault.value[Kind] = ""

  try {
    const Result = await Api.Save(Kind, _Bare(Draft.value[Kind]), Force)
    if (!Result.Ok) {
      Fault.value[Kind] = Result.Error ?? "unknown"
      return
    }

    const Host = Result.Hostname ?? _Bare(Draft.value[Kind])
    Stored.value[Kind] = Host
    Draft.value[Kind] = Host
    Saved.value = Kind
    if (_SavedTimer) clearTimeout(_SavedTimer)
    _SavedTimer = setTimeout(() => { Saved.value = null }, _SavedMs)
  } catch {
    Fault.value[Kind] = "unknown"
  } finally {
    Busy.value = null
  }
}

function FaultText(Kind: HostKind): string {
  const Code = Fault.value[Kind]
  const Key = `settings.cloudflare.error.${Code}`
  const Text = t(Key)
  return Text === Key ? t("settings.cloudflare.error.unknown") : Text
}

onMounted(async () => {
  try {
    const Data = await Api.Load()
    Stored.value = { management: Data.Management, voting: Data.Voting }
    Draft.value = { management: Data.Management, voting: Data.Voting }
    Local.value = Data.Local
  } catch {} finally {
    Ready.value = true
  }
})

onUnmounted(() => {
  if (_SavedTimer) clearTimeout(_SavedTimer)
})
</script>

<template>
  <Pane Id="cloudflare">
    <Group :Title="t('settings.cloudflare.addresses')">
      <Row v-for="Kind in _Kinds" :key="Kind" Stack>
        <template #label>
          <p class="Label">{{ t(`settings.cloudflare.${Kind}`) }}</p>
        </template>
        <template #detail>
          <span v-if="Locked(Kind)" class="Lock">
            <Glyph Name="lock" class="LockIcon" />
            {{ t("settings.cloudflare.locked") }}
          </span>
          <a
            v-else-if="Stored[Kind]"
            :href="`https://${Stored[Kind]}`"
            target="_blank"
            rel="noopener"
            class="Open"
          >
            {{ t("settings.cloudflare.open") }}
            <Glyph Name="external" class="OpenIcon" />
          </a>
        </template>

        <template #end>
          <label class="Field" :class="{ Locked: Locked(Kind), Bad: !!Fault[Kind], Idle: !Ready }">
            <span class="Scheme">https://</span>
            <input
              :value="Draft[Kind]"
              type="text"
              inputmode="url"
              autocomplete="off"
              autocapitalize="off"
              spellcheck="false"
              class="Host"
              :readonly="Locked(Kind)"
              :aria-label="t(`settings.cloudflare.${Kind}`)"
              @input="Edit(Kind, ($event.target as HTMLInputElement).value)"
              @keydown.enter="Save(Kind)"
              @keydown.esc.stop="Reset(Kind)"
            />
          </label>

          <button
            v-if="Dirty[Kind] || Saved === Kind"
            type="button"
            class="Btn Primary Commit"
            :disabled="Busy !== null || Saved === Kind"
            @click="Save(Kind)"
          >
            {{ Busy === Kind ? t("settings.cloudflare.checking") : Saved === Kind ? t("settings.saved") : t("settings.save") }}
          </button>
        </template>

        <template v-if="Fault[Kind]" #below>
          <p class="Fault">
            <span>{{ FaultText(Kind) }}</span>
            <button
              v-if="Fault[Kind] === 'unreachable'"
              type="button"
              class="Force"
              :disabled="Busy !== null"
              @click="Save(Kind, true)"
            >
              {{ t("settings.cloudflare.force") }}
            </button>
          </p>
        </template>
      </Row>

      <template #footer>{{ t("settings.cloudflare.footer") }}</template>
    </Group>
  </Pane>
</template>

<style scoped>
.Label {
  margin: 0;
  font-size: 15px;
  font-weight: 500;
  line-height: 1.35;
  letter-spacing: -0.01em;
  color: var(--ink);
}

.Lock,
.Open {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.LockIcon,
.OpenIcon {
  height: 13px;
  width: 13px;
  flex-shrink: 0;
}

.Open {
  color: var(--ink-mute);
  text-decoration: none;
  transition: color 150ms ease;
}

.Open:hover {
  color: var(--ink);
}

.Field {
  display: flex;
  height: 42px;
  min-width: 0;
  flex: 1 1 280px;
  align-items: center;
  border-radius: 14px;
  background: rgba(255, 255, 255, 0.035);
  box-shadow: inset 0 0 0 1px var(--edge);
  padding: 0 14px;
  cursor: text;
  transition: box-shadow 200ms ease, background 200ms ease, opacity 200ms ease;
}

.Field:hover {
  box-shadow: inset 0 0 0 1px var(--edge-strong);
}

.Field:focus-within {
  background: rgba(255, 255, 255, 0.05);
  box-shadow: inset 0 0 0 1px rgba(224, 168, 168, 0.55), 0 0 0 4px rgba(224, 168, 168, 0.1);
}

.Field.Bad {
  box-shadow: inset 0 0 0 1px rgba(239, 138, 138, 0.6);
}

.Field.Locked {
  opacity: 0.55;
  cursor: not-allowed;
}

.Field.Locked:hover,
.Field.Locked:focus-within {
  background: rgba(255, 255, 255, 0.035);
  box-shadow: inset 0 0 0 1px var(--edge);
}

.Field.Idle {
  opacity: 0.4;
}

.Scheme {
  flex-shrink: 0;
  font-size: 14px;
  color: var(--ink-faint);
  user-select: none;
}

.Host {
  min-width: 0;
  flex: 1;
  border: 0;
  background: transparent;
  padding: 0;
  font: inherit;
  font-size: 14px;
  color: var(--ink);
  outline: none;
}

.Locked .Host {
  cursor: not-allowed;
}

.Commit {
  flex-shrink: 0;
}

.Fault {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 6px 14px;
  margin: 0;
  font-size: 13.5px;
  line-height: 1.5;
  color: var(--danger);
}

.Force {
  font-size: 13.5px;
  color: var(--ink-soft);
  cursor: pointer;
  transition: color 150ms ease;
}

.Force:hover:not(:disabled) {
  color: var(--ink);
}
</style>
