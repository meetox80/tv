<script setup lang="ts">
import { ref, computed, watch, type Ref } from "vue"
import Sheet from "./Sheet.vue"
import Cover from "./Cover.vue"
import Explicit from "./Explicit.vue"
import Captcha from "./Captcha.vue"
import * as Api from "../_api/vote"
import { ApiError } from "../_api/http"
import { Describe, DuplicateMessage } from "../_api/messages"
import { Duration, Plural, Proposals, SameSong } from "../_api/format"
import type { SongStatus, Track } from "../_api/types"
import {
  Absorb, ActiveTab, Board, Device, ForgetDevice, GoVote, Left, Limit, RefreshSongs, SetLeft,
} from "../_state/store"

const Props = defineProps<{ Track: Track | null }>()
const Emit = defineEmits<{ Close: []; Sent: [] }>()

type Step = "confirm" | "captcha" | "sending" | "done" | "duplicate"

const Current: Ref<Step> = ref("confirm")
const Problem: Ref<string> = ref("")
const Known: Ref<SongStatus | null> = ref(null)
const Shown: Ref<Track | null> = ref(null)

watch(() => Props.Track, (Next) => {
  if (!Next) return
  Shown.value = Next
  Problem.value = ""
  Known.value = Next.Status
  Current.value = Next.Status ? "duplicate" : "confirm"
}, { immediate: true })

const Meta = computed(() => {
  const T = Shown.value
  if (!T) return ""
  return [T.Album, Duration(T.DurationMs)].filter(Boolean).join(" · ")
})

const LeftAfter = computed(() => Math.max(0, Left.value - 1))

const Hint = computed(() => {
  if (!Device.value) return `Możesz wysłać ${Proposals(Limit.value)} dziennie.`
  if (Left.value === 0) return "Dzisiejsze propozycje są już wykorzystane. Nowe pojawią się po północy."
  if (LeftAfter.value === 0) return "To będzie Twoja ostatnia propozycja na dziś."
  return `Po wysłaniu ${Plural(LeftAfter.value, "zostanie", "zostaną", "zostanie")} ${LeftAfter.value} ${Plural(LeftAfter.value, "propozycja", "propozycje", "propozycji")} na dziś.`
})

const Blocked = computed(() => Boolean(Device.value) && Left.value === 0)

const DuplicateText = computed(() => DuplicateMessage(Known.value))

const OnBoard = computed(() => {
  const T = Shown.value
  if (!T) return null
  return Board.value.find(S => SameSong(S, T)) ?? null
})

async function _Send(): Promise<void> {
  const T = Shown.value
  if (!T) return

  Current.value = "sending"
  Problem.value = ""

  try {
    const Result = await Api.SubmitRequest(T.Provider, T.Id)
    SetLeft(Result.Left)
    Current.value = "done"
    Emit("Sent")
    void RefreshSongs()
  } catch (Err) {
    if (Err instanceof ApiError && Err.Code === "device") {
      ForgetDevice()
      Current.value = "captcha"
      return
    }

    if (Err instanceof ApiError && Err.Code === "duplicate") {
      Known.value = typeof Err.Body.Status === "string" ? (Err.Body.Status as SongStatus) : "pending"
      Current.value = "duplicate"
      void RefreshSongs()
      return
    }

    Absorb(Err)
    Problem.value = Describe(Err)
    Current.value = "confirm"
  }
}

function Submit(): void {
  if (Blocked.value) return
  if (!Device.value) {
    Current.value = "captcha"
    return
  }
  void _Send()
}

function GoToBoard(): void {
  GoVote(OnBoard.value?.Id ?? null)
  Emit("Close")
}

function GoToMine(): void {
  ActiveTab.value = "mine"
  Emit("Close")
}
</script>

<template>
  <Sheet :Open="Track !== null" Label="Propozycja utworu" :Dismissable="Current !== 'sending'" @Close="Emit('Close')">
    <div v-if="Shown" class="Body">
      <div class="Hero">
        <Cover :Src="Shown.Cover" :Title="Shown.Title" :Size="148" :Radius="22" class="HeroCover" />
      </div>

      <div class="Heading">
        <h2 class="Title">
          <span>{{ Shown.Title }}</span>
          <Explicit v-if="Shown.Explicit" />
        </h2>
        <p class="Artist">{{ Shown.Artist }}</p>
        <p v-if="Meta" class="Meta">{{ Meta }}</p>
      </div>

      <Transition name="step" mode="out-in">
        <div v-if="Current === 'confirm' || Current === 'sending'" key="confirm" class="Step">
          <div aria-live="assertive">
            <p v-if="Problem" class="Problem" role="alert">{{ Problem }}</p>
          </div>

          <button
            type="button"
            class="Primary"
            :disabled="Blocked || Current === 'sending'"
            @click="Submit"
          >
            <span v-if="Current === 'sending'" class="Spinner" aria-hidden="true" />
            <span>{{ Current === "sending" ? "Wysyłanie…" : "Wyślij propozycję" }}</span>
          </button>

          <p class="Hint">{{ Hint }}</p>
        </div>

        <div v-else-if="Current === 'captcha'" key="captcha" class="Step">
          <p class="Lead">Zanim wyślesz, potwierdź, że nie jesteś botem.</p>
          <Captcha @Done="_Send" />
        </div>

        <div v-else-if="Current === 'done'" key="done" class="Step Done" role="status">
          <svg class="Check" viewBox="0 0 52 52" aria-hidden="true">
            <circle cx="26" cy="26" r="24" fill="none" />
            <path d="M15 27l7 7 15-16" fill="none" />
          </svg>
          <p class="DoneTitle">Wysłane</p>
          <p class="Lead">
            Propozycja czeka teraz na zatwierdzenie. Jeśli przejdzie, trafi do rankingu i będzie można na nią głosować.
          </p>
          <div class="Actions">
            <button type="button" class="Secondary" @click="GoToMine">Moje propozycje</button>
            <button type="button" class="Primary" @click="Emit('Close')">Zaproponuj kolejną</button>
          </div>
        </div>

        <div v-else key="duplicate" class="Step">
          <p class="Lead" role="status">{{ DuplicateText }}</p>
          <div class="Actions">
            <button type="button" class="Secondary" @click="Emit('Close')">Szukaj dalej</button>
            <button v-if="Known === 'approved'" type="button" class="Primary" @click="GoToBoard">
              Zagłosuj na nią
            </button>
          </div>
        </div>
      </Transition>
    </div>
  </Sheet>
</template>

<style scoped>
.Body {
  display: flex;
  flex-direction: column;
  gap: 22px;
  padding-top: 20px;
}

.Hero {
  position: relative;
  display: flex;
  justify-content: center;
  padding-top: 6px;
}

.HeroCover {
  position: relative;
  box-shadow:
    inset 0 0 0 1px rgba(255, 255, 255, 0.1),
    0 30px 60px -24px rgba(0, 0, 0, 0.9);
}

.Heading {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  text-align: center;
}

.Title {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  font-family: var(--font-display);
  font-size: 24px;
  font-weight: 600;
  line-height: 1.15;
  letter-spacing: -0.02em;
  text-wrap: balance;
}

.Artist {
  font-size: 16px;
  color: var(--color-ink-soft);
}

.Meta {
  font-size: 13px;
  color: var(--color-ink-mute);
}


.Step {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.Lead {
  text-align: center;
  font-size: 15px;
  line-height: 1.55;
  color: var(--color-ink-soft);
}

.Hint {
  text-align: center;
  font-size: 13px;
  color: var(--color-ink-mute);
}

.Problem {
  border-radius: 14px;
  background: rgba(224, 108, 108, 0.08);
  box-shadow: inset 0 0 0 1px rgba(224, 108, 108, 0.2);
  padding: 12px 14px;
  text-align: center;
  font-size: 14px;
  line-height: 1.45;
  color: #f0c4c4;
}

.Primary,
.Secondary {
  display: inline-flex;
  min-height: 52px;
  flex: 1;
  align-items: center;
  justify-content: center;
  gap: 10px;
  border-radius: 16px;
  padding: 0 20px;
  font-size: 16px;
  font-weight: 600;
  letter-spacing: -0.01em;
  transition: transform 160ms var(--ease-smooth), background 200ms ease, opacity 200ms ease;
}

.Primary {
  background: linear-gradient(180deg, #f1c6c6, #e0a8a8);
  color: #1d1113;
  box-shadow: 0 16px 40px -18px rgba(224, 168, 168, 0.7);
}

.Primary:disabled {
  opacity: 0.45;
  box-shadow: none;
}

.Secondary {
  background: rgba(255, 255, 255, 0.07);
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.08);
  color: var(--color-ink);
}

.Primary:not(:disabled):active,
.Secondary:active {
  transform: scale(0.98);
}

.Actions {
  display: flex;
  gap: 10px;
}

.Spinner {
  height: 16px;
  width: 16px;
  border-radius: 999px;
  border: 2px solid rgba(29, 17, 19, 0.2);
  border-top-color: #1d1113;
  animation: Spin 800ms linear infinite;
}

@keyframes Spin {
  to { transform: rotate(360deg); }
}

.Done {
  align-items: center;
}

.Check {
  height: 64px;
  width: 64px;
  stroke: var(--color-rose);
  stroke-width: 2.5;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.Check circle {
  stroke-dasharray: 151;
  stroke-dashoffset: 151;
  animation: Draw 700ms var(--ease-smooth) forwards;
}

.Check path {
  stroke-dasharray: 40;
  stroke-dashoffset: 40;
  animation: Draw 500ms var(--ease-smooth) 350ms forwards;
}

@keyframes Draw {
  to { stroke-dashoffset: 0; }
}

.DoneTitle {
  font-family: var(--font-display);
  font-size: 22px;
  font-weight: 600;
}

.Done .Actions {
  width: 100%;
}

.step-enter-active,
.step-leave-active {
  transition: opacity 220ms ease, transform 360ms var(--ease-smooth);
}

.step-enter-from {
  opacity: 0;
  transform: translateY(10px);
}

.step-leave-to {
  opacity: 0;
  transform: translateY(-6px);
}
</style>
