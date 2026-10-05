<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, type Ref } from "vue"
import { useI18n } from "../../i18n"
import { Copy } from "../../_api/clipboard"
import * as Api from "../../_api/voting"
import * as Cache from "../../_api/cache"
import Pane from "./ui/Pane.vue"
import Group from "./ui/Group.vue"
import Stats, { type StatItem } from "./ui/Stats.vue"
import Row from "./ui/Row.vue"
import Switch from "./ui/Switch.vue"

const { t } = useI18n()

const Props = defineProps<{ Revision: number }>()
const Emit = defineEmits<{ Review: []; Enabled: [Value: boolean] }>()

const _CacheKey = "panel_voting_cache"
const _Cached = Cache.Read<Api.VotingConfig>(_CacheKey)

const Config: Ref<Api.VotingConfig | null> = ref(_Cached)
const Enabled: Ref<boolean> = ref(_Cached?.Enabled ?? false)
const Toggling: Ref<boolean> = ref(false)
const ToggleError: Ref<string> = ref("")

const SiteKey: Ref<string> = ref(_Cached?.SiteKey ?? "")
const Secret: Ref<string> = ref("")
const CaptchaBusy: Ref<boolean> = ref(false)
const CaptchaError: Ref<string> = ref("")
const Check: Ref<Api.CaptchaStatus | "checking"> = ref(_Cached?.SiteKey && _Cached?.HasSecret ? "checking" : "unconfigured")
const Clearing: Ref<boolean> = ref(false)
const ClearArmed: Ref<boolean> = ref(false)

const UrlCopied: Ref<boolean> = ref(false)

let _ArmTimer: ReturnType<typeof setTimeout> | null = null
let _CopiedTimer: ReturnType<typeof setTimeout> | null = null

const Configured = computed(() => !!Config.value?.SiteKey && !!Config.value?.HasSecret)

const ShowForm = computed(() => !Configured.value || Check.value === "invalid")

const CanSaveCaptcha = computed(() => {
  if (CaptchaBusy.value) return false
  return !!SiteKey.value.trim() && !!Secret.value.trim()
})

const StatusDot = computed(() => {
  if (Check.value === "valid") return "Good"
  if (Check.value === "unreachable") return "Warn"
  return "Busy"
})

const Pending = computed(() => Config.value?.Counts.Pending ?? 0)

const StatusText = computed(() => {
  if (Check.value === "valid") return t("voting.captcha.status.valid")
  if (Check.value === "unreachable") return t("voting.captcha.status.unreachable")
  return t("voting.captcha.status.checking")
})

const Hostname = computed(() => {
  const Url = Config.value?.Url
  if (!Url) return ""
  try {
    return new URL(Url).hostname
  } catch {
    return Url
  }
})

const Tiles = computed<StatItem[]>(() => {
  const C = Config.value?.Counts
  return [
    { Key: "pending", Label: t("voting.counts.pending"), Count: C?.Pending ?? 0 },
    { Key: "approved", Label: t("voting.counts.approved"), Count: C?.Approved ?? 0 },
    { Key: "added", Label: t("voting.counts.added"), Count: C?.Added ?? 0 },
    { Key: "rejected", Label: t("voting.counts.rejected"), Count: C?.Rejected ?? 0 },
  ].map(Tile => ({
    Key: Tile.Key,
    Label: Tile.Label,
    Value: String(Tile.Count),
    Accent: Tile.Key === "pending",
    Faint: Tile.Count === 0,
  }))
})

function _Message(Error: unknown): string {
  if (Error instanceof Api.ApiError) {
    const Key = `voting.error.${Error.Code}`
    const Text = t(Key)
    return Text === Key ? t("voting.error.generic") : Text
  }
  return t("voting.error.generic")
}

async function LoadConfig(): Promise<void> {
  try {
    const Data = await Api.Load()
    Config.value = Data
    Enabled.value = Data.Enabled
    if (!SiteKey.value.trim() || SiteKey.value === (_Cached?.SiteKey ?? "")) SiteKey.value = Data.SiteKey
    Cache.Write(_CacheKey, Data)
  } catch {}
}

async function Revalidate(): Promise<void> {
  if (!Configured.value) {
    Check.value = "unconfigured"
    return
  }

  Check.value = "checking"
  try {
    Check.value = await Api.CheckTurnstile()
  } catch {
    Check.value = "unreachable"
  }
  if (Check.value === "invalid") SiteKey.value = Config.value?.SiteKey ?? ""
}

async function Clear(): Promise<void> {
  if (Clearing.value) return

  if (!ClearArmed.value) {
    ClearArmed.value = true
    if (_ArmTimer) clearTimeout(_ArmTimer)
    _ArmTimer = setTimeout(() => { ClearArmed.value = false }, 3000)
    return
  }

  if (_ArmTimer) clearTimeout(_ArmTimer)
  ClearArmed.value = false
  Clearing.value = true
  CaptchaError.value = ""

  try {
    await Api.ClearTurnstile()
    SiteKey.value = ""
    Secret.value = ""
    Check.value = "unconfigured"
    await LoadConfig()
  } catch (E) {
    CaptchaError.value = _Message(E)
  } finally {
    Clearing.value = false
  }
}

async function Toggle(): Promise<void> {
  if (Toggling.value) return
  if (!Configured.value && !Enabled.value) {
    ToggleError.value = t("voting.error.captcha_unconfigured")
    return
  }

  const Next = !Enabled.value
  Enabled.value = Next
  Toggling.value = true
  ToggleError.value = ""

  try {
    await Api.SetEnabled(Next)
    if (Config.value) Config.value = { ...Config.value, Enabled: Next }
    if (Config.value) Cache.Write(_CacheKey, Config.value)
  } catch (E) {
    Enabled.value = !Next
    ToggleError.value = _Message(E)
  } finally {
    Toggling.value = false
  }
}

async function SaveCaptcha(): Promise<void> {
  if (!CanSaveCaptcha.value) return

  CaptchaBusy.value = true
  CaptchaError.value = ""

  try {
    await Api.SaveTurnstile(SiteKey.value.trim(), Secret.value.trim())
    Secret.value = ""
    ToggleError.value = ""
    await LoadConfig()
    Check.value = "valid"
  } catch (E) {
    CaptchaError.value = _Message(E)
  } finally {
    CaptchaBusy.value = false
  }
}

async function CopyUrl(): Promise<void> {
  const Url = Config.value?.Url
  if (!Url) return
  await Copy(Url)
  UrlCopied.value = true
  if (_CopiedTimer) clearTimeout(_CopiedTimer)
  _CopiedTimer = setTimeout(() => { UrlCopied.value = false }, 1600)
}

watch(() => Props.Revision, () => {
  void LoadConfig()
})

watch(Enabled, (Value) => Emit("Enabled", Value), { immediate: true })

onMounted(async () => {
  await LoadConfig()
  await Revalidate()
})

const StateGroup: Ref<InstanceType<typeof Group> | null> = ref(null)

let _FlashTimer: ReturnType<typeof setTimeout> | null = null

function Spotlight(): void {
  const El = StateGroup.value?.$el as HTMLElement | undefined
  if (!El) return
  El.classList.remove("Flash")
  void El.offsetWidth
  El.classList.add("Flash")
  if (_FlashTimer) clearTimeout(_FlashTimer)
  _FlashTimer = setTimeout(() => El.classList.remove("Flash"), 1800)
}

defineExpose({ Toggle, Spotlight })

onUnmounted(() => {
  if (_ArmTimer) clearTimeout(_ArmTimer)
  if (_CopiedTimer) clearTimeout(_CopiedTimer)
  if (_FlashTimer) clearTimeout(_FlashTimer)
})
</script>

<template>
  <Pane Id="voting">
    <Group ref="StateGroup">
      <Row :Label="t('voting.enabled')" :Detail="Configured ? t('voting.enabled.description') : t('voting.enabled.locked')">
        <template #end>
          <Switch
            :On="Enabled"
            :Label="t('voting.enabled')"
            :Disabled="Toggling"
            :Dim="!Configured && !Enabled"
            @Toggle="Toggle"
          />
        </template>
      </Row>

      <template v-if="ToggleError" #footer>
        <span class="Fault">{{ ToggleError }}</span>
      </template>
    </Group>

    <Group :Title="t('voting.review')">
      <Row Block class="StatsRow">
        <Stats :Items="Tiles" />
      </Row>
      <Row :Label="t('voting.review.row')" Chevron @Press="Emit('Review')">
        <template #end>
          <span v-if="Pending > 0" class="Badge">{{ Pending > 99 ? "99+" : Pending }}</span>
        </template>
      </Row>
    </Group>

    <Group :Title="t('voting.page')" Id="voting-page">
      <Row v-if="Config?.Url" Stack>
        <template #label>
          <a :href="Config.Url" target="_blank" rel="noopener" class="Url Monospace">{{ Config.Url }}</a>
        </template>
        <template #end>
          <button type="button" class="Btn" @click="CopyUrl">
            {{ UrlCopied ? t("settings.copied") : t("settings.copy") }}
          </button>
        </template>
      </Row>
      <Row v-else :Label="t('voting.url.missing')" />
    </Group>

    <Group :Title="t('voting.captcha')" Id="voting-captcha">
      <template v-if="!ShowForm">
        <Row>
          <template #label>
            <p class="Status" :class="StatusDot">{{ StatusText }}</p>
          </template>
          <template #detail>
            <span class="Monospace">{{ Config?.SiteKey }}</span>
          </template>
          <template #end>
            <button
              type="button"
              class="Btn"
              :class="ClearArmed ? 'Armed' : 'Danger'"
              :disabled="Clearing"
              @click="Clear"
            >
              {{ ClearArmed ? t("voting.captcha.clear.short") : t("voting.captcha.clear") }}
            </button>
          </template>
        </Row>
      </template>

      <template v-else>
        <Row v-if="Check === 'invalid'">
          <template #label>
            <p class="Status Fault">{{ t("voting.captcha.status.invalid") }}</p>
          </template>
        </Row>
        <Row :Label="t('voting.captcha.sitekey')" Stack>
          <template #end>
            <input
              v-model="SiteKey"
              type="text"
              spellcheck="false"
              autocomplete="off"
              :placeholder="t('voting.captcha.sitekey.placeholder')"
              class="Input Monospace Stretch"
            />
          </template>
        </Row>
        <Row :Label="t('voting.captcha.secret')" Stack>
          <template #end>
            <input
              v-model="Secret"
              type="password"
              spellcheck="false"
              autocomplete="new-password"
              :placeholder="t('voting.captcha.secret.placeholder')"
              class="Input Monospace Stretch"
              @keydown.enter="SaveCaptcha"
            />
          </template>
        </Row>
      </template>

      <template #footer>
        <span v-if="CaptchaError" class="Fault">{{ CaptchaError }}</span>
        <span v-else-if="ClearArmed" class="Fault">{{ t("voting.captcha.clear.confirm") }}</span>
        <template v-else-if="ShowForm">
          {{ t("voting.captcha.hint") }}
          <a href="https://dash.cloudflare.com/?to=/:account/turnstile" target="_blank" rel="noopener noreferrer" class="Link">dash.cloudflare.com</a>.
          <template v-if="Hostname">
            {{ t("voting.captcha.hostname") }} <span class="Monospace Host">{{ Hostname }}</span>.
          </template>
        </template>
      </template>
      <template v-if="ShowForm" #action>
        <button type="button" class="Btn Primary" :disabled="!CanSaveCaptcha" @click="SaveCaptcha">
          {{ CaptchaBusy ? t("voting.captcha.verifying") : t("settings.save") }}
        </button>
      </template>
    </Group>
  </Pane>
</template>

<style scoped>
.StatsRow {
  padding: 26px 0 24px !important;
}

.Badge {
  display: inline-flex;
  height: 22px;
  min-width: 22px;
  align-items: center;
  justify-content: center;
  border-radius: 999px;
  background: linear-gradient(180deg, var(--rose-light), var(--rose));
  padding: 0 7px;
  font-size: 12.5px;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
  color: var(--rose-ink);
}

.Url {
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 13.5px;
  color: var(--ink);
  text-decoration: none;
  user-select: text;
}

.Url:hover {
  color: var(--rose);
}

.Status {
  margin: 0;
  font-size: 15px;
  font-weight: 500;
  line-height: 1.3;
  color: var(--ink);
}

.Status.Warn {
  color: var(--warn);
}

.Status.Busy {
  color: var(--ink-mute);
  animation: breathe 1.4s ease-in-out infinite;
}

.Host {
  color: var(--ink);
}

@media (max-width: 1023px) {
  .Url,
  .Status {
    font-size: 16px;
  }
}
</style>
