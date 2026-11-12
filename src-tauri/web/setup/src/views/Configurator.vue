<script setup lang="ts">
import { ref, computed, watch, onUnmounted, type Ref, type ComputedRef } from "vue"
import { useI18n } from "../i18n"
import { Authed } from "../session"
import Icon from "../_components/Icon.vue"
import Activity from "../_components/Activity.vue"

const { t } = useI18n()

const Emit = defineEmits<{
  Unauthorized: []
}>()

type Step = "token" | "verifying" | "ports" | "password" | "done"

const _MinimumVerifyingMs: number = 650

const CurrentStep: Ref<Step> = ref("token")
const Token: Ref<string> = ref("")
const Password: Ref<string> = ref("")
const PasswordConfirm: Ref<string> = ref("")
const VerifyError: Ref<string> = ref("")

const MgmtHostname: Ref<string> = ref("")
const VoteHostname: Ref<string> = ref("")
const HostnameVerifying: Ref<boolean> = ref(false)
const MgmtVerified: Ref<boolean | null> = ref(null)
const VoteVerified: Ref<boolean | null> = ref(null)
const HostnameError: Ref<string> = ref("")
const Skipping: Ref<boolean> = ref(false)
const SkipError: Ref<string> = ref("")

const LocalOnly: Ref<boolean> = ref(false)

const Flow: ComputedRef<Step[]> = computed(() =>
  LocalOnly.value ? ["token", "password"] : ["token", "ports", "password"]
)

const ProgressIndex: ComputedRef<number> = computed(() =>
  Flow.value.indexOf(CurrentStep.value === "verifying" ? "token" : CurrentStep.value)
)

const ProgressLabel: ComputedRef<string> = computed(() =>
  t("config.progress")
    .replace("{n}", String(ProgressIndex.value + 1))
    .replace("{total}", String(Flow.value.length))
)

const PasswordValid: ComputedRef<boolean> = computed(
  () => Password.value.length > 0 && Password.value === PasswordConfirm.value
)

const PasswordMismatch: ComputedRef<boolean> = computed(
  () => PasswordConfirm.value.length > 0 && !PasswordValid.value
)

const CanVerifyHostnames: ComputedRef<boolean> = computed(
  () => MgmtHostname.value.trim().length > 0 && VoteHostname.value.trim().length > 0 && !HostnameVerifying.value
)

function _ExtractToken(Raw: string): string {
  const Match: RegExpMatchArray | null = Raw.match(/eyJ[A-Za-z0-9+/=]+/)
  return Match ? Match[0] : Raw.trim()
}

function _Wait(Ms: number): Promise<void> {
  return new Promise(Resolve => setTimeout(Resolve, Ms))
}

async function _KeepVerifyingVisible(StartedAt: number): Promise<void> {
  const Remaining: number = _MinimumVerifyingMs - (Date.now() - StartedAt)
  if (Remaining > 0) await _Wait(Remaining)
}

async function SubmitToken(): Promise<void> {
  if (!Token.value.trim()) return
  CurrentStep.value = "verifying"
  VerifyError.value = ""
  const StartedAt: number = Date.now()

  try {
    const Res = await Authed("/api/cloudflared", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        TunnelToken: _ExtractToken(Token.value),
      }),
    })

    if (Res.status === 401) {
      Emit("Unauthorized")
      return
    }

    if (!Res.ok) {
      await _KeepVerifyingVisible(StartedAt)
      VerifyError.value = t("config.verifying.error.save")
      return
    }

    const Data: { Ok: boolean; Error?: string } = await Res.json()
    if (!Data.Ok) {
      await _KeepVerifyingVisible(StartedAt)
      VerifyError.value = Data.Error ?? t("config.verifying.error.save")
      return
    }

    await _KeepVerifyingVisible(StartedAt)
    CurrentStep.value = "ports"
  } catch (_) {
    await _KeepVerifyingVisible(StartedAt)
    VerifyError.value = t("config.verifying.error.connection")
  }
}

async function SkipTunnel(): Promise<void> {
  if (Skipping.value) return
  Skipping.value = true
  SkipError.value = ""

  try {
    const Res = await Authed("/api/setup/local", { method: "POST" })

    if (Res.status === 401) {
      Emit("Unauthorized")
      return
    }

    if (!Res.ok) {
      SkipError.value = t("config.verifying.error.save")
      return
    }

    LocalOnly.value = true
    CurrentStep.value = "password"
  } catch (_) {
    SkipError.value = t("config.verifying.error.connection")
  } finally {
    Skipping.value = false
  }
}

async function VerifyHostnames(): Promise<void> {
  if (!CanVerifyHostnames.value) return
  HostnameVerifying.value = true
  HostnameError.value = ""
  MgmtVerified.value = null
  VoteVerified.value = null

  try {
    const Res = await Authed("/api/cloudflared/verify", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        ManagementHostname: MgmtHostname.value.trim(),
        VotingHostname: VoteHostname.value.trim(),
      }),
    })

    if (Res.status === 401) {
      Emit("Unauthorized")
      return
    }

    const Data: { Management: boolean; Voting: boolean; Error?: string } = await Res.json()
    MgmtVerified.value = Data.Management
    VoteVerified.value = Data.Voting

    if (Data.Error) {
      HostnameError.value = Data.Error
    } else if (Data.Management && Data.Voting) {
      const SaveRes = await Authed("/api/cloudflared", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          TunnelToken: _ExtractToken(Token.value),
          ManagementHostname: MgmtHostname.value.trim(),
          VotingHostname: VoteHostname.value.trim(),
        }),
      })

      if (SaveRes.status === 401) {
        Emit("Unauthorized")
        return
      }

      if (!SaveRes.ok) {
        HostnameError.value = t("config.verifying.error.save")
        return
      }

      const SaveData: { Ok: boolean; Error?: string } = await SaveRes.json()
      if (!SaveData.Ok) {
        HostnameError.value = SaveData.Error ?? t("config.verifying.error.save")
        return
      }

      CurrentStep.value = "password"
    }
  } catch (_) {
    HostnameError.value = t("config.verifying.error.connection")
  } finally {
    HostnameVerifying.value = false
  }
}

const RedirectCountdown: Ref<number> = ref(5)
let _RedirectTimer: ReturnType<typeof setInterval> | null = null

function _DashboardUrl(): string {
  const Host = MgmtHostname.value.trim()
  if (!Host) return `http://${window.location.hostname}:9999`
  return Host.startsWith("http") ? Host : `https://${Host}`
}

watch(CurrentStep, (Step) => {
  if (Step !== "done") return
  RedirectCountdown.value = 5
  _RedirectTimer = setInterval(() => {
    RedirectCountdown.value--
    if (RedirectCountdown.value <= 0) {
      if (_RedirectTimer) clearInterval(_RedirectTimer)
      const Url = _DashboardUrl()
      if (Url) window.location.href = Url
    }
  }, 1000)
})

onUnmounted(() => {
  if (_RedirectTimer) clearInterval(_RedirectTimer)
})

async function Finish(): Promise<void> {
  if (!PasswordValid.value) return

  try {
    const Res = await Authed("/api/setup/password", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ Password: Password.value }),
    })

    if (Res.status === 401) {
      Emit("Unauthorized")
      return
    }

    if (!Res.ok) return

    CurrentStep.value = "done"
  } catch (_) {}
}
</script>

<template>
  <div class="screen">
    <header
      v-if="CurrentStep !== 'done'"
      class="steps"
      role="progressbar"
      :aria-label="ProgressLabel"
      aria-valuemin="1"
      :aria-valuemax="Flow.length"
      :aria-valuenow="ProgressIndex + 1"
    >
      <div class="steps-track">
        <span
          v-for="(S, I) in Flow"
          :key="S"
          class="steps-segment"
          :class="{ 'is-filled': I <= ProgressIndex }"
        />
      </div>
      <span class="steps-label">{{ ProgressLabel }}</span>
    </header>

    <div class="screen-stage">
      <Transition name="slide" mode="out-in">
        <section v-if="CurrentStep === 'token'" key="token" class="screen-body">
          <div class="icon-tile">
            <Icon Name="key" />
          </div>
          <h1 class="screen-title">{{ t("config.token.title") }}</h1>
          <p class="screen-subtitle">{{ t("config.token.subtitle") }}</p>

          <form class="screen-content" @submit.prevent="SubmitToken">
            <div class="group">
              <textarea
                v-model="Token"
                rows="4"
                spellcheck="false"
                autocomplete="off"
                autocapitalize="off"
                autocorrect="off"
                placeholder="cloudflared.exe service install eyJ..."
                class="field token-field"
                @keydown.enter.exact.prevent="SubmitToken"
              />
            </div>

            <a
              href="https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/get-started/create-remote-tunnel/"
              target="_blank"
              rel="noopener"
              class="help-link"
            >
              {{ t("config.token.help") }}
              <Icon Name="external" class="size-3.5" />
            </a>

            <div class="screen-actions">
              <button type="submit" :disabled="!Token.trim()" class="primary-button">
                {{ t("config.token.continue") }}
              </button>
              <button type="button" :disabled="Skipping" class="text-button" @click="SkipTunnel">
                <Activity v-if="Skipping" class="size-4" />
                {{ t("config.token.skip") }}
              </button>
            </div>

            <p v-if="SkipError" class="form-error">{{ SkipError }}</p>
          </form>
        </section>

        <section v-else-if="CurrentStep === 'verifying'" key="verifying" class="screen-body">
          <template v-if="!VerifyError">
            <div class="icon-tile">
              <Activity class="size-7" />
            </div>
            <h1 class="screen-title">{{ t("config.verifying.title") }}</h1>
            <p class="screen-subtitle">{{ t("config.verifying.saving") }}</p>
          </template>

          <template v-else>
            <div class="icon-tile is-danger">
              <Icon Name="alert" />
            </div>
            <h1 class="screen-title">{{ t("config.verifying.error.title") }}</h1>
            <p class="screen-subtitle">{{ VerifyError }}</p>

            <div class="screen-actions mt-8">
              <button type="button" class="secondary-button" @click="CurrentStep = 'token'">
                {{ t("config.verifying.back") }}
              </button>
            </div>
          </template>
        </section>

        <section v-else-if="CurrentStep === 'ports'" key="ports" class="screen-body">
          <div class="icon-tile">
            <Icon Name="globe" />
          </div>
          <h1 class="screen-title">{{ t("config.ports.title") }}</h1>
          <p class="screen-subtitle">{{ t("config.ports.subtitle") }}</p>

          <form class="screen-content space-y-3" @submit.prevent="VerifyHostnames">
            <div class="group" :class="{ 'is-success': MgmtVerified === true, 'is-danger': MgmtVerified === false }">
              <div class="host-head">
                <div class="min-w-0">
                  <p class="host-name">{{ t("config.ports.management") }}</p>
                  <p class="host-desc">{{ t("config.ports.management.desc") }}</p>
                </div>
                <span class="host-target">http://127.0.0.1:9999</span>
              </div>
              <div class="group-divider" />
              <div class="host-row">
                <input
                  v-model="MgmtHostname"
                  :placeholder="t('config.ports.hostname.placeholder')"
                  inputmode="url"
                  autocapitalize="off"
                  autocorrect="off"
                  spellcheck="false"
                  class="field"
                />
                <Icon v-if="MgmtVerified === true" Name="check" class="host-mark text-success" />
                <Icon v-else-if="MgmtVerified === false" Name="close" class="host-mark text-danger" />
              </div>
            </div>

            <div class="group" :class="{ 'is-success': VoteVerified === true, 'is-danger': VoteVerified === false }">
              <div class="host-head">
                <div class="min-w-0">
                  <p class="host-name">{{ t("config.ports.voting") }}</p>
                  <p class="host-desc">{{ t("config.ports.voting.desc") }}</p>
                </div>
                <span class="host-target">http://127.0.0.1:10000</span>
              </div>
              <div class="group-divider" />
              <div class="host-row">
                <input
                  v-model="VoteHostname"
                  :placeholder="t('config.ports.hostname.placeholder')"
                  inputmode="url"
                  autocapitalize="off"
                  autocorrect="off"
                  spellcheck="false"
                  class="field"
                />
                <Icon v-if="VoteVerified === true" Name="check" class="host-mark text-success" />
                <Icon v-else-if="VoteVerified === false" Name="close" class="host-mark text-danger" />
              </div>
            </div>

            <p v-if="HostnameError" class="form-error">{{ HostnameError }}</p>

            <div class="screen-actions">
              <button type="submit" :disabled="!CanVerifyHostnames" class="primary-button">
                <Activity v-if="HostnameVerifying" class="size-5" />
                {{ HostnameVerifying ? t("config.ports.verifying") : t("config.ports.verify") }}
              </button>
            </div>
          </form>
        </section>

        <section v-else-if="CurrentStep === 'password'" key="password" class="screen-body">
          <div class="icon-tile">
            <Icon Name="lock" />
          </div>
          <h1 class="screen-title">{{ t("config.password.title") }}</h1>
          <p class="screen-subtitle">{{ t("config.password.subtitle") }}</p>

          <form class="screen-content" @submit.prevent="Finish">
            <div class="group" :class="{ 'is-danger': PasswordMismatch }">
              <input
                v-model="Password"
                type="password"
                autocomplete="new-password"
                :placeholder="t('config.password.placeholder')"
                class="field"
              />
              <div class="group-divider" />
              <input
                v-model="PasswordConfirm"
                type="password"
                autocomplete="new-password"
                :placeholder="t('config.password.confirm')"
                class="field"
              />
            </div>

            <p v-if="PasswordMismatch" class="form-error">{{ t("config.password.mismatch") }}</p>

            <div class="screen-actions">
              <button type="submit" :disabled="!PasswordValid" class="primary-button">
                {{ t("config.password.finish") }}
              </button>
            </div>
          </form>
        </section>

        <section v-else key="done" class="screen-body">
          <div class="icon-tile is-success done-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <path class="checkmark-path" d="m5 12.5 4.5 4.5L19 7.5" />
            </svg>
          </div>
          <h1 class="screen-title">{{ t("config.done.title") }}</h1>
          <p class="screen-subtitle">{{ t("config.done.subtitle") }}</p>
          <span class="countdown">
            {{ t("config.done.redirect").replace("{s}", String(RedirectCountdown)) }}
          </span>
        </section>
      </Transition>
    </div>
  </div>
</template>

<style scoped>
.steps {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
}

.steps-track {
  display: flex;
  gap: 6px;
}

.steps-segment {
  width: 28px;
  height: 4px;
  border-radius: 9999px;
  background: rgb(var(--hairline) / 0.14);
  transition: background 400ms var(--ease-smooth);
}

.steps-segment.is-filled {
  background: rgb(var(--text));
}

.steps-label {
  color: rgb(var(--text) / 0.5);
  font-size: 13px;
  font-weight: 500;
  font-variant-numeric: tabular-nums;
}

.token-field {
  resize: none;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 14px;
  line-height: 1.55;
  word-break: break-all;
}

.help-link {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  margin-top: 12px;
  padding: 0 4px;
  color: rgb(var(--text) / 0.6);
  font-size: 14px;
  font-weight: 500;
  transition: color 160ms ease;
}

.help-link:hover {
  color: rgb(var(--text));
}

.host-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
  padding: 14px 16px 12px;
}

.host-name {
  font-size: 15px;
  font-weight: 600;
  letter-spacing: -0.01em;
}

.host-desc {
  margin-top: 2px;
  color: rgb(var(--text) / 0.5);
  font-size: 13px;
}

.host-target {
  flex-shrink: 0;
  border-radius: 8px;
  background: rgb(var(--hairline) / 0.06);
  padding: 4px 8px;
  color: rgb(var(--text) / 0.6);
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 12px;
  user-select: text;
}

.host-row {
  display: flex;
  align-items: center;
}

.host-mark {
  width: 20px;
  height: 20px;
  flex-shrink: 0;
  margin-right: 16px;
}

.countdown {
  margin-top: 24px;
  border-radius: 9999px;
  background: rgb(var(--hairline) / 0.08);
  padding: 8px 14px;
  color: rgb(var(--text) / 0.6);
  font-size: 13px;
  font-weight: 500;
  font-variant-numeric: tabular-nums;
}

.slide-enter-active,
.slide-leave-active {
  transition:
    opacity 320ms var(--ease-smooth),
    transform 320ms var(--ease-smooth);
}

.slide-enter-from {
  opacity: 0;
  transform: translateX(28px);
}

.slide-leave-to {
  opacity: 0;
  transform: translateX(-28px);
}

.done-icon {
  animation: pop 450ms var(--ease-smooth) both;
}

.checkmark-path {
  stroke-dasharray: 24;
  stroke-dashoffset: 24;
  animation: draw 500ms 200ms ease forwards;
}

@keyframes pop {
  0% { transform: scale(0.6); opacity: 0 }
  100% { transform: scale(1); opacity: 1 }
}

@keyframes draw {
  to { stroke-dashoffset: 0 }
}
</style>
