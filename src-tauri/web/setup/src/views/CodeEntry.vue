<script setup lang="ts">
import { computed, nextTick, ref, type Ref } from "vue"
import { useI18n } from "../i18n"
import { SetToken } from "../session"

const { t } = useI18n()

const Emit = defineEmits<{
  Accepted: [token: string]
}>()

const Length = 6

const Digits: Ref<string[]> = ref(Array(Length).fill(""))
const Inputs: Ref<HTMLInputElement[]> = ref([])
const Error: Ref<string> = ref("")
const IsLoading: Ref<boolean> = ref(false)

const Code = computed<string>(() => Digits.value.join(""))

function Focus(Index: number): void {
  const El = Inputs.value[Index]
  if (!El) return
  El.focus()
  El.select()
}

function Fill(Index: number, Text: string): void {
  const Clean = Text.replace(/\D/g, "")
  if (!Clean) return

  const Next = [...Digits.value]
  for (let I = 0; I < Clean.length && Index + I < Length; I++) Next[Index + I] = Clean[I]
  Digits.value = Next

  Focus(Math.min(Index + Clean.length, Length - 1))
  MaybeSubmit()
}

function OnInput(Index: number, E: Event): void {
  const Target = E.target as HTMLInputElement
  const Raw = Target.value
  Target.value = Digits.value[Index]

  Error.value = ""
  Fill(Index, Raw)
}

function OnKeydown(Index: number, E: KeyboardEvent): void {
  if (E.key === "Backspace") {
    E.preventDefault()
    Error.value = ""

    const Next = [...Digits.value]
    if (Next[Index]) {
      Next[Index] = ""
      Digits.value = Next
      return
    }

    if (Index > 0) {
      Next[Index - 1] = ""
      Digits.value = Next
      Focus(Index - 1)
    }
    return
  }

  if (E.key === "ArrowLeft" && Index > 0) {
    E.preventDefault()
    Focus(Index - 1)
    return
  }

  if (E.key === "ArrowRight" && Index < Length - 1) {
    E.preventDefault()
    Focus(Index + 1)
  }
}

function OnPaste(Index: number, E: ClipboardEvent): void {
  E.preventDefault()
  Error.value = ""
  Fill(Index, E.clipboardData?.getData("text") ?? "")
}

function MaybeSubmit(): void {
  if (Code.value.length === Length && !IsLoading.value) void Submit()
}

async function Submit(): Promise<void> {
  if (Code.value.length !== Length || IsLoading.value) return
  IsLoading.value = true
  Error.value = ""

  try {
    const Res = await fetch("/api/auth", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ code: Code.value }),
    })

    if (Res.ok) {
      const Data: { Token?: string } = await Res.json().catch(() => ({}))
      if (!Data.Token) {
        Error.value = t("code.error.generic")
      } else {
        SetToken(Data.Token)
        Emit("Accepted", Data.Token)
        return
      }
    } else {
      Error.value = Res.status === 401 ? t("code.error.wrong") : t("code.error.generic")
    }
  } catch (_) {
    Error.value = t("code.error.connection")
  }

  IsLoading.value = false
  Digits.value = Array(Length).fill("")
  await nextTick()
  Focus(0)
}
</script>

<template>
  <div class="setup-scroll min-h-screen flex items-center justify-center px-5 py-24 lg:px-10">
    <main class="w-full max-w-[440px]">
      <div class="mb-9">
        <h1 class="font-display text-[38px] font-semibold leading-[1.05] tracking-[-0.035em] lg:text-[46px]">{{ t("code.title") }}</h1>
        <p class="muted mt-3 max-w-[340px] text-[15px] leading-6">{{ t("code.subtitle") }}</p>
      </div>

      <form class="setup-card" @submit.prevent="Submit">
        <label for="pairing-code-0" class="muted mb-3 block text-[13px] font-medium">{{ t("code.label") }}</label>

        <div class="flex justify-between gap-2.5">
          <input
            v-for="(Digit, Index) in Digits"
            :key="Index"
            :id="`pairing-code-${Index}`"
            :ref="(El) => { if (El) Inputs[Index] = El as HTMLInputElement }"
            :value="Digit"
            :autofocus="Index === 0"
            :disabled="IsLoading"
            @input="OnInput(Index, $event)"
            @keydown="OnKeydown(Index, $event)"
            @paste="OnPaste(Index, $event)"
            @focus="($event.target as HTMLInputElement).select()"
            type="text"
            inputmode="numeric"
            :autocomplete="Index === 0 ? 'one-time-code' : 'off'"
            maxlength="1"
            :aria-label="`${t('code.label')} ${Index + 1}`"
            class="digit-box"
            :class="[Error ? 'is-error' : '', Digit ? 'is-filled' : '']"
          />
        </div>

        <p v-if="Error" class="mt-4 text-[13px] font-medium text-accent">{{ Error }}</p>

        <button type="submit" :disabled="Code.length !== Length || IsLoading" class="primary-button mt-5 w-full">
          {{ IsLoading ? t("code.button.loading") : t("code.button") }}
          <svg v-if="!IsLoading" class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M5 12h14"/><path d="m12 5 7 7-7 7"/></svg>
        </button>
      </form>
    </main>
  </div>
</template>

<style scoped>
.setup-card {
  border-radius: var(--radius-xl);
  background: var(--surface);
  padding: 26px;
  box-shadow:
    0 1px 0 rgb(var(--hairline) / 0.05) inset,
    0 30px 60px -20px rgb(0 0 0 / 0.7);
}

.digit-box {
  height: 72px;
  min-width: 0;
  flex: 1;
  border: 0;
  border-radius: var(--radius-lg);
  background: var(--sunken);
  color: rgb(var(--text));
  text-align: center;
  font-size: 30px;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
  line-height: 1;
  caret-color: transparent;
  outline: none;
  box-shadow: 0 1px 0 rgb(var(--hairline) / 0.04) inset;
  transition: background 160ms ease, box-shadow 160ms ease, transform 160ms ease;
}

.digit-box.is-filled {
  background: var(--surface-raised);
}

.digit-box:focus {
  background: var(--surface-raised);
  box-shadow: 0 0 0 2px rgb(var(--accent) / 0.75);
  transform: translateY(-1px);
}

.digit-box.is-error {
  box-shadow: 0 0 0 2px rgb(var(--accent) / 0.45);
  animation: shake 260ms ease;
}

.digit-box:disabled {
  opacity: 0.4;
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

@keyframes shake {
  0%, 100% { transform: translateX(0) }
  25% { transform: translateX(-4px) }
  75% { transform: translateX(4px) }
}
</style>
