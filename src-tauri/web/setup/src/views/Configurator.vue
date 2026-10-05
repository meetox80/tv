<script setup lang="ts">
import { ref, computed, watch, onUnmounted, type Ref, type ComputedRef } from "vue"
import { useI18n } from "../i18n"
import { Authed } from "../session"

const { t } = useI18n()

const Emit = defineEmits<{
  Unauthorized: []
}>()

type Step = "token" | "verifying" | "ports" | "password" | "done"

const _Steps: Step[] = ["token", "verifying", "ports", "password", "done"]
const _MinimumVerifyingMs: number = 650
const _LabelKeys: Record<Step, string> = {
  token: "config.step.token",
  verifying: "config.step.verifying",
  ports: "config.step.ports",
  password: "config.step.password",
  done: "config.step.done",
}

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

const StepIndex: ComputedRef<number> = computed(() => _Steps.indexOf(CurrentStep.value))

const PasswordValid: ComputedRef<boolean> = computed(
  () => Password.value.length > 0 && Password.value === PasswordConfirm.value
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
  if (!Host) return ""
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
  <div class="min-h-screen flex flex-col overflow-hidden px-5 lg:px-10">

    <div class="mx-auto w-full max-w-[920px] shrink-0 pt-24 pb-5 lg:pt-28">
      <div class="mb-3 flex items-center justify-between">
        <div class="flex items-center gap-3">
          <span class="text-xs muted">{{ t(_LabelKeys[CurrentStep]) }}</span>
        </div>
        <span class="text-[11px] tabular-nums subtle">{{ String(StepIndex + 1).padStart(2, '0') }} / {{ String(_Steps.length).padStart(2, '0') }}</span>
      </div>
      <div class="flex gap-1">
        <div
          v-for="(S, I) in _Steps"
          :key="S"
          class="h-0.5 flex-1 rounded-full transition-all duration-500"
          :class="I <= StepIndex ? 'bg-accent' : 'bg-white/8'"
        />
      </div>
    </div>

    <div class="setup-scroll flex flex-1 items-center justify-center pb-10">
      <div class="w-full max-w-[560px]">
        <Transition name="slide" mode="out-in">

          <div v-if="CurrentStep === 'token'" key="token" class="setup-card flex flex-col">
            <h1 class="font-display text-[30px] font-semibold tracking-[-0.035em]">{{ t("config.token.title") }}</h1>
            <p class="mt-2.5 mb-7 text-sm leading-6 muted">{{ t("config.token.subtitle") }}</p>

            <form @submit.prevent="SubmitToken">
              <textarea
                v-model="Token"
                rows="3"
                spellcheck="false"
                autocomplete="off"
                autocapitalize="off"
                autocorrect="off"
                placeholder="cloudflared.exe service install eyJ..."
                class="setup-input token-input font-mono"
                @keydown.enter.exact.prevent="SubmitToken"
              />

              <button type="submit" :disabled="!Token.trim()" class="primary-button mt-5 w-full">
                {{ t("config.token.continue") }}
                <svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M5 12h14"/><path d="m12 5 7 7-7 7"/></svg>
              </button>
            </form>

            <a
              href="https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/get-started/create-remote-tunnel/"
              target="_blank"
              rel="noopener"
              class="mt-3 flex items-center gap-1 text-xs muted transition-colors hover:text-accent"
            >
              {{ t("config.token.help") }}
              <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                <path stroke-linecap="round" stroke-linejoin="round" d="M13.5 6H5.25A2.25 2.25 0 0 0 3 8.25v10.5A2.25 2.25 0 0 0 5.25 21h10.5A2.25 2.25 0 0 0 18 18.75V10.5m-10.5 6L21 3m0 0h-5.25M21 3v5.25" />
              </svg>
            </a>

          </div>

          <div v-else-if="CurrentStep === 'verifying'" key="verifying" class="setup-card flex flex-col items-center">
            <template v-if="!VerifyError">
              <div class="w-14 h-14 rounded-full border-2 border-white/8 border-t-accent animate-spin mb-5" />
              <p class="text-sm muted">{{ t("config.verifying.saving") }}</p>
            </template>

            <template v-else>
              <svg class="w-8 h-8 text-accent mb-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v3.75m9-.75a9 9 0 1 1-18 0 9 9 0 0 1 18 0Zm-9 3.75h.008v.008H12v-.008Z" />
              </svg>
              <p class="text-accent text-sm font-medium mb-4">{{ VerifyError }}</p>
              <button
                @click="CurrentStep = 'token'"
                class="secondary-button"
              >
                {{ t("config.verifying.back") }}
              </button>
            </template>
          </div>

          <div v-else-if="CurrentStep === 'ports'" key="ports" class="setup-card flex flex-col">
            <h1 class="font-display text-[30px] font-semibold tracking-[-0.035em]">{{ t("config.ports.title") }}</h1>
            <p class="mt-2.5 mb-7 max-w-[460px] text-sm leading-6 muted">{{ t("config.ports.subtitle") }}</p>

            <div class="w-full space-y-3">
              <div class="setup-field" :class="MgmtVerified === true ? 'verified' : MgmtVerified === false ? 'invalid' : ''">
                <div class="flex items-center justify-between mb-2">
                  <div>
                    <p class="text-sm font-medium">{{ t("config.ports.management") }}</p>
                    <p class="text-xs subtle">{{ t("config.ports.management.desc") }}</p>
                  </div>
                  <span class="text-[11px] tabular-nums subtle">http://127.0.0.1:9999</span>
                </div>
                <input
                  v-model="MgmtHostname"
                  :placeholder="t('config.ports.hostname.placeholder')"
                  class="setup-input"
                />
              </div>

              <div class="setup-field" :class="VoteVerified === true ? 'verified' : VoteVerified === false ? 'invalid' : ''">
                <div class="flex items-center justify-between mb-2">
                  <div>
                    <p class="text-sm font-medium">{{ t("config.ports.voting") }}</p>
                    <p class="text-xs subtle">{{ t("config.ports.voting.desc") }}</p>
                  </div>
                  <span class="text-[11px] tabular-nums subtle">http://127.0.0.1:10000</span>
                </div>
                <input
                  v-model="VoteHostname"
                  :placeholder="t('config.ports.hostname.placeholder')"
                  class="setup-input"
                />
              </div>
            </div>

            <p v-if="HostnameError" class="text-accent text-xs mt-3 text-center">{{ HostnameError }}</p>

            <button
              @click="VerifyHostnames"
              :disabled="!CanVerifyHostnames"
              class="primary-button mt-5 w-full"
            >
              <div v-if="HostnameVerifying" class="w-4 h-4 rounded-full border-2 border-current/30 border-t-current animate-spin" />
              {{ HostnameVerifying ? t("config.ports.verifying") : t("config.ports.verify") }}
            </button>
          </div>

          <div v-else-if="CurrentStep === 'password'" key="password" class="setup-card flex flex-col">
            <h1 class="font-display text-[30px] font-semibold tracking-[-0.035em]">{{ t("config.password.title") }}</h1>
            <p class="mt-2.5 mb-7 text-sm leading-6 muted">{{ t("config.password.subtitle") }}</p>

            <div class="w-full space-y-3">
              <input
                v-model="Password"
                type="password"
                :placeholder="t('config.password.placeholder')"
                class="setup-input h-14"
              />
              <input
                v-model="PasswordConfirm"
                type="password"
                :placeholder="t('config.password.confirm')"
                class="setup-input h-14"
                :class="PasswordConfirm && !PasswordValid ? 'is-invalid' : ''"
              />
            </div>

            <button
              @click="Finish"
              :disabled="!PasswordValid"
              class="primary-button mt-5 w-full"
            >
              {{ t("config.password.finish") }}
            </button>
          </div>

          <div v-else key="done" class="setup-card flex flex-col items-center text-center">
            <div class="w-16 h-16 rounded-full bg-success/10 flex items-center justify-center mb-6 done-icon">
              <svg class="w-8 h-8 text-success" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                <path class="checkmark-path" stroke-linecap="round" stroke-linejoin="round" d="m4.5 12.75 6 6 9-13.5" />
              </svg>
            </div>

            <h1 class="text-2xl font-semibold mb-2">{{ t("config.done.title") }}</h1>
            <p class="muted text-sm text-center mb-6 max-w-[280px]">{{ t("config.done.subtitle") }}</p>
            <div class="px-3 py-1.5 rounded-full bg-white/5 subtle text-[12px] font-medium text-center tabular-nums">
              {{ t("config.done.redirect").replace("{s}", String(RedirectCountdown)) }}
            </div>
          </div>

        </Transition>
      </div>
    </div>
  </div>
</template>

<style scoped>
.setup-card {
  border-radius: var(--radius-xl);
  background: var(--surface);
  padding: 28px;
  box-shadow:
    0 1px 0 rgb(var(--hairline) / 0.05) inset,
    0 30px 60px -20px rgb(0 0 0 / 0.7);
}

.primary-button {
  display: flex;
  height: 54px;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  border-radius: var(--radius-lg);
  background: rgb(var(--text));
  color: #0A0A0B;
  font-size: 15px;
  font-weight: 600;
  letter-spacing: -0.01em;
  transition: opacity 160ms ease, transform 160ms ease;
}

.primary-button:hover:not(:disabled) {
  opacity: 0.9;
}

.primary-button:active:not(:disabled) {
  transform: scale(0.985);
}

.primary-button:disabled {
  pointer-events: none;
  background: var(--surface-raised);
  color: rgb(var(--text) / 0.3);
}

.secondary-button {
  display: flex;
  height: 46px;
  align-items: center;
  justify-content: center;
  padding: 0 20px;
  border: 0;
  border-radius: var(--radius-md);
  background: var(--surface-raised);
  color: rgb(var(--text) / 0.85);
  font-size: 14px;
  font-weight: 500;
  transition: background 160ms ease, color 160ms ease;
}

.secondary-button:hover {
  background: #232328;
  color: rgb(var(--text));
}

.setup-input {
  width: 100%;
  height: 50px;
  border: 0;
  border-radius: var(--radius-md);
  background: var(--sunken);
  padding: 0 14px;
  color: rgb(var(--text));
  font-size: 14px;
  outline: none;
  box-shadow: 0 1px 0 rgb(var(--hairline) / 0.04) inset;
  transition: background 160ms ease, box-shadow 160ms ease;
}

.setup-input::placeholder {
  color: rgb(var(--text) / 0.28);
}

.setup-input:focus {
  background: var(--surface-raised);
  box-shadow: 0 0 0 2px rgb(var(--accent) / 0.7);
}

.setup-input.is-invalid {
  box-shadow: 0 0 0 2px rgb(var(--accent) / 0.5);
}

.token-input {
  display: block;
  height: auto;
  padding: 13px 14px;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 13px;
  line-height: 1.6;
  resize: none;
  word-break: break-all;
}

.setup-field {
  border: 0;
  border-radius: var(--radius-lg);
  background: rgb(var(--hairline) / 0.03);
  padding: 15px;
  transition: background 160ms ease, box-shadow 160ms ease;
}

.setup-field.verified {
  box-shadow: 0 0 0 1px rgb(var(--success) / 0.45);
}

.setup-field.invalid {
  box-shadow: 0 0 0 1px rgb(var(--accent) / 0.45);
}

@media (max-width: 520px) {
  .setup-card {
    padding: 22px;
  }
}

.slide-enter-active,
.slide-leave-active {
  transition: all 300ms cubic-bezier(0.16, 1, 0.3, 1);
}

.slide-enter-from {
  opacity: 0;
  transform: translateX(40px);
}

.slide-leave-to {
  opacity: 0;
  transform: translateX(-40px);
}

.done-icon {
  animation: pop 400ms cubic-bezier(0.16, 1, 0.3, 1) both;
}

.checkmark-path {
  stroke-dasharray: 30;
  stroke-dashoffset: 30;
  animation: draw 500ms 200ms ease forwards;
}

@keyframes pop {
  0% { transform: scale(0); opacity: 0 }
  100% { transform: scale(1); opacity: 1 }
}

@keyframes draw {
  to { stroke-dashoffset: 0 }
}
</style>
