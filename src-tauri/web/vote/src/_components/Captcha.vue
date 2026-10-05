<script setup lang="ts">
import { ref, onMounted, onUnmounted, type Ref } from "vue"
import * as Turnstile from "../_api/turnstile"
import { Describe } from "../_api/messages"
import { Absorb, Register, State } from "../_state/store"

const Emit = defineEmits<{ Done: [] }>()

type Phase = "loading" | "ready" | "checking" | "failed"

const Host: Ref<HTMLElement | null> = ref(null)
const Current: Ref<Phase> = ref("loading")
const Problem: Ref<string> = ref("")

let _Widget: string | null = null
let _Api: Turnstile.Turnstile | null = null
let _Alive = true

function _Fail(Message: string): void {
  Current.value = "failed"
  Problem.value = Message
}

async function _Solved(Token: string): Promise<void> {
  Current.value = "checking"
  Problem.value = ""

  try {
    await Register(Token)
    if (_Alive) Emit("Done")
  } catch (Err) {
    Absorb(Err)
    _Fail(Describe(Err))
    if (_Api && _Widget) _Api.reset(_Widget)
  }
}

async function _Mount(): Promise<void> {
  const SiteKey = State.value?.SiteKey
  if (!SiteKey) {
    _Fail("Weryfikacja nie jest jeszcze skonfigurowana. Wróć później.")
    return
  }

  Current.value = "loading"
  Problem.value = ""

  try {
    _Api = await Turnstile.Load()
  } catch {
    _Fail("Nie udało się wczytać weryfikacji Cloudflare. Jeśli masz bloker treści, wyłącz go dla tej strony.")
    return
  }

  if (!_Alive || !Host.value) return

  if (_Widget) {
    _Api.remove(_Widget)
    _Widget = null
  }

  _Widget = _Api.render(Host.value, {
    sitekey: SiteKey,
    action: "device",
    theme: "dark",
    language: "pl",
    appearance: "always",
    callback: (Token: string) => void _Solved(Token),
    "error-callback": () => _Fail("Weryfikacja nie zadziałała. Spróbuj ponownie."),
    "expired-callback": () => _Fail("Weryfikacja wygasła. Zrób ją jeszcze raz."),
    "timeout-callback": () => _Fail("Weryfikacja trwała zbyt długo. Spróbuj ponownie."),
  }) ?? null

  Current.value = "ready"
}

function Retry(): void {
  if (_Api && _Widget) {
    _Api.reset(_Widget)
    Current.value = "ready"
    Problem.value = ""
    return
  }

  void _Mount()
}

onMounted(() => void _Mount())

onUnmounted(() => {
  _Alive = false
  if (_Api && _Widget) _Api.remove(_Widget)
})
</script>

<template>
  <div class="Captcha">
    <div class="Frame">
      <div ref="Host" class="Host" />

      <div v-if="Current === 'loading'" class="Status">
        <span class="Spinner" aria-hidden="true" />
        <span>Wczytywanie weryfikacji…</span>
      </div>

      <div v-else-if="Current === 'checking'" class="Status Overlay">
        <span class="Spinner" aria-hidden="true" />
        <span>Sprawdzanie…</span>
      </div>
    </div>

    <p class="Note">
      Jednorazowo na tym urządzeniu. Weryfikację obsługuje Cloudflare Turnstile.
    </p>

    <div aria-live="assertive" class="Live">
      <div v-if="Current === 'failed'" class="Problem" role="alert">
        <p>{{ Problem }}</p>
        <button type="button" class="Retry" @click="Retry">Spróbuj ponownie</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.Captcha {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
}

.Frame {
  position: relative;
  display: flex;
  min-height: 72px;
  width: 100%;
  align-items: center;
  justify-content: center;
}

.Host {
  display: flex;
  justify-content: center;
}

.Status {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 14px;
  color: var(--color-ink-mute);
}

.Overlay {
  position: absolute;
  inset: 0;
  justify-content: center;
  border-radius: 14px;
  background: rgba(14, 13, 17, 0.85);
}

.Spinner {
  height: 16px;
  width: 16px;
  border-radius: 999px;
  border: 2px solid rgba(255, 255, 255, 0.12);
  border-top-color: var(--color-rose);
  animation: Spin 800ms linear infinite;
}

@keyframes Spin {
  to { transform: rotate(360deg); }
}

.Note {
  max-width: 300px;
  text-align: center;
  font-size: 12px;
  line-height: 1.5;
  color: var(--color-ink-faint);
}

.Live {
  width: 100%;
}

.Problem {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  border-radius: 16px;
  background: rgba(224, 108, 108, 0.08);
  box-shadow: inset 0 0 0 1px rgba(224, 108, 108, 0.2);
  padding: 14px 16px;
  text-align: center;
  font-size: 14px;
  line-height: 1.45;
  color: #f0c4c4;
}

.Retry {
  min-height: 44px;
  border-radius: 12px;
  background: rgba(255, 255, 255, 0.08);
  padding: 0 18px;
  font-size: 14px;
  font-weight: 500;
  color: var(--color-ink);
  transition: background 160ms ease;
}

.Retry:hover {
  background: rgba(255, 255, 255, 0.12);
}
</style>
