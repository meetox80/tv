<script setup lang="ts">
import { computed, nextTick, ref, type Ref } from "vue"
import { useI18n } from "../i18n"
import { SetToken } from "../session"
import Icon from "../_components/Icon.vue"
import Activity from "../_components/Activity.vue"

const { t } = useI18n()

const Emit = defineEmits<{
  Accepted: [token: string]
}>()

const Length = 6

const Digits: Ref<string[]> = ref(Array(Length).fill(""))
const Inputs: Ref<HTMLInputElement[]> = ref([])
const ErrorText: Ref<string> = ref("")
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

  ErrorText.value = ""
  Fill(Index, Raw)
}

function OnKeydown(Index: number, E: KeyboardEvent): void {
  if (E.key === "Backspace") {
    E.preventDefault()
    ErrorText.value = ""

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
  ErrorText.value = ""
  Fill(Index, E.clipboardData?.getData("text") ?? "")
}

function MaybeSubmit(): void {
  if (Code.value.length === Length && !IsLoading.value) void Submit()
}

async function Submit(): Promise<void> {
  if (Code.value.length !== Length || IsLoading.value) return
  IsLoading.value = true
  ErrorText.value = ""

  try {
    const Res = await fetch("/api/auth", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ code: Code.value }),
    })

    if (Res.ok) {
      const Data: { Token?: string } = await Res.json().catch(() => ({}))
      if (!Data.Token) {
        ErrorText.value = t("code.error.generic")
      } else {
        SetToken(Data.Token)
        Emit("Accepted", Data.Token)
        return
      }
    } else {
      ErrorText.value = Res.status === 401 ? t("code.error.wrong") : t("code.error.generic")
    }
  } catch (_) {
    ErrorText.value = t("code.error.connection")
  }

  IsLoading.value = false
  Digits.value = Array(Length).fill("")
  await nextTick()
  Focus(0)
}
</script>

<template>
  <div class="screen">
    <main class="screen-body">
      <div class="icon-tile">
        <Icon Name="tv" />
      </div>
      <h1 class="screen-title">{{ t("code.title") }}</h1>
      <p class="screen-subtitle">{{ t("code.subtitle") }}</p>

      <form class="screen-content" @submit.prevent="Submit">
        <label for="pairing-code-0" class="sr-only">{{ t("code.label") }}</label>

        <div class="digits" :class="{ 'is-error': ErrorText }">
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
            :class="{ 'is-filled': Digit, 'digit-split': Index === Length / 2 }"
          />
        </div>

        <p v-if="ErrorText" class="form-error">{{ ErrorText }}</p>

        <div class="screen-actions">
          <button type="submit" :disabled="Code.length !== Length || IsLoading" class="primary-button">
            <Activity v-if="IsLoading" class="size-5" />
            {{ IsLoading ? t("code.button.loading") : t("code.button") }}
          </button>
        </div>
      </form>
    </main>
  </div>
</template>

<style scoped>
.digits {
  display: flex;
  gap: 8px;
}

.digits.is-error {
  animation: shake 300ms ease;
}

.digit-box {
  height: 64px;
  min-width: 0;
  flex: 1;
  border: 0;
  border-radius: 14px;
  background: var(--group);
  color: rgb(var(--text));
  text-align: center;
  font-size: 28px;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
  line-height: 1;
  caret-color: transparent;
  outline: none;
  box-shadow: inset 0 0 0 1px rgb(var(--hairline) / 0.04);
  transition:
    background 160ms ease,
    box-shadow 160ms ease;
}

.digit-split {
  margin-left: 10px;
}

.digit-box.is-filled {
  background: var(--group-raised);
}

.digit-box:focus {
  background: var(--group-raised);
  box-shadow: inset 0 0 0 2px rgb(var(--hairline) / 0.6);
}

.digits.is-error .digit-box {
  box-shadow: inset 0 0 0 2px rgb(var(--danger) / 0.75);
}

.digit-box:disabled {
  opacity: 0.45;
}

@media (min-width: 640px) {
  .digit-box {
    height: 72px;
    font-size: 32px;
  }
}

@keyframes shake {
  0%, 100% { transform: translateX(0) }
  25% { transform: translateX(-5px) }
  75% { transform: translateX(5px) }
}
</style>
