<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, onActivated, onDeactivated, type Ref } from "vue"
import Cover from "../_components/Cover.vue"
import Explicit from "../_components/Explicit.vue"
import ConfirmSheet from "../_components/ConfirmSheet.vue"
import * as Api from "../_api/vote"
import { Describe, StatusLabel } from "../_api/messages"
import { Duration } from "../_api/format"
import type { Track } from "../_api/types"
import { Absorb } from "../_state/store"

const Emit = defineEmits<{ Docked: [Value: boolean] }>()

type Phase = "idle" | "loading" | "done" | "failed"

const _DebounceMs = 300
const _MinLength = 2
const _MaxLength = 100

const Query: Ref<string> = ref("")
const Current: Ref<Phase> = ref("idle")
const Results: Ref<Track[]> = ref([])
const Problem: Ref<string> = ref("")
const Picked: Ref<Track | null> = ref(null)
const Field: Ref<HTMLInputElement | null> = ref(null)
const Dock: Ref<HTMLElement | null> = ref(null)
const Docked: Ref<boolean> = ref(false)

let _Timer: ReturnType<typeof setTimeout> | null = null
let _Inflight: AbortController | null = null

const Trimmed = computed(() => Query.value.trim())

const Announcement = computed(() => {
  if (Current.value === "loading") return "Szukanie…"
  if (Current.value === "failed") return Problem.value
  if (Current.value === "done" && !Results.value.length) return "Brak wyników."
  if (Current.value === "done") return `Znaleziono wyniki: ${Results.value.length}.`
  return ""
})

function _Cancel(): void {
  if (_Timer) clearTimeout(_Timer)
  _Timer = null
  _Inflight?.abort()
  _Inflight = null
}

async function _Run(Text: string): Promise<void> {
  _Inflight?.abort()
  const Controller = new AbortController()
  _Inflight = Controller

  Current.value = "loading"
  Problem.value = ""

  try {
    const Data = await Api.Search(Text, Controller.signal)
    if (Controller.signal.aborted) return
    Results.value = Data.Results
    Current.value = "done"
  } catch (Err) {
    if (Controller.signal.aborted) return
    if (Err instanceof DOMException && Err.name === "AbortError") return
    Absorb(Err)
    Problem.value = Describe(Err)
    Current.value = "failed"
  } finally {
    if (_Inflight === Controller) _Inflight = null
  }
}

watch(Trimmed, (Text) => {
  _Cancel()

  if (Text.length < _MinLength) {
    Current.value = "idle"
    Results.value = []
    return
  }

  _Timer = setTimeout(() => void _Run(Text.slice(0, _MaxLength)), _DebounceMs)
})

function Submit(): void {
  const Text = Trimmed.value
  if (Text.length < _MinLength) return
  _Cancel()
  void _Run(Text.slice(0, _MaxLength))
  Field.value?.blur()
}

function Clear(): void {
  Query.value = ""
  Field.value?.focus()
}

function Retry(): void {
  if (Trimmed.value.length >= _MinLength) void _Run(Trimmed.value.slice(0, _MaxLength))
}

function Pick(Item: Track): void {
  Picked.value = Item
}

function OnSent(): void {
  const Hit = Picked.value
  if (!Hit) return
  Results.value = Results.value.map(R =>
    R.Provider === Hit.Provider && R.Id === Hit.Id ? { ...R, Status: "pending" } : R
  )
}

function _Measure(): void {
  const El = Dock.value
  if (!El || !El.isConnected) return
  const Stick = parseFloat(getComputedStyle(El).top)
  Docked.value = getComputedStyle(El).position === "sticky" && El.getBoundingClientRect().top <= Stick + 0.5
}

function _Listen(): void {
  window.addEventListener("scroll", _Measure, { passive: true })
  window.addEventListener("resize", _Measure, { passive: true })
  _Measure()
}

function _Unlisten(): void {
  window.removeEventListener("scroll", _Measure)
  window.removeEventListener("resize", _Measure)
  Docked.value = false
}

watch(Docked, Value => Emit("Docked", Value))

onMounted(_Listen)
onActivated(_Listen)
onDeactivated(_Unlisten)
onUnmounted(() => {
  _Cancel()
  _Unlisten()
})
</script>

<template>
  <section class="View" aria-labelledby="request-heading">
    <h1 id="request-heading" class="Heading">Zaproponuj utwór</h1>

    <div ref="Dock" class="Dock" :class="{ Docked }">
      <form class="Search" role="search" @submit.prevent="Submit">
        <label for="search-input" class="SrOnly">Szukaj utworu lub wykonawcy</label>
        <svg class="Glyph" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true">
          <circle cx="11" cy="11" r="7" />
          <path d="m20 20-3.6-3.6" stroke-linecap="round" />
        </svg>
        <input
          id="search-input"
          ref="Field"
          v-model="Query"
          type="search"
          inputmode="search"
          enterkeyhint="search"
          autocomplete="off"
          autocorrect="off"
          autocapitalize="off"
          spellcheck="false"
          :maxlength="_MaxLength"
          placeholder="Tytuł albo wykonawca"
          class="Input"
        />
        <button v-if="Query" type="button" class="Clear" aria-label="Wyczyść" @click="Clear">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
            <path d="M16 8 8 16M8 8l8 8" />
          </svg>
        </button>
      </form>
    </div>

    <p class="SrOnly" aria-live="polite">{{ Announcement }}</p>

    <Transition name="swap" mode="out-in">
      <ul v-if="Current === 'loading'" key="loading" class="Panel" aria-hidden="true">
        <li v-for="I in 6" :key="I" class="Row">
          <span class="Skeleton GhostCover" />
          <span class="Text">
            <span class="Skeleton GhostLine" :style="{ width: `${52 + ((I * 17) % 30)}%` }" />
            <span class="Skeleton GhostLine Short" :style="{ width: `${28 + ((I * 11) % 22)}%` }" />
          </span>
        </li>
      </ul>

      <div v-else-if="Current === 'failed'" key="failed" class="Empty" role="alert">
        <p>{{ Problem }}</p>
        <button type="button" class="Action" @click="Retry">Spróbuj ponownie</button>
      </div>

      <div v-else-if="Current === 'done' && !Results.length" key="empty" class="Empty">
        <p class="EmptyTitle">Brak wyników</p>
        <p>Sprawdź pisownię albo wpisz samego wykonawcę.</p>
      </div>

      <TransitionGroup v-else-if="Current === 'done'" key="results" tag="ul" name="rise" class="Panel" appear>
        <li v-for="(Item, Index) in Results" :key="`${Item.Provider}:${Item.Id}`" :style="{ '--Delay': `${Index * 30}ms` }">
          <button type="button" class="Row Pickable" @click="Pick(Item)">
            <Cover :Src="Item.Cover" :Title="Item.Title" :Size="52" :Radius="11" />
            <span class="Text">
              <span class="Name">
                <span class="Clip">{{ Item.Title }}</span>
                <Explicit v-if="Item.Explicit" />
              </span>
              <span class="By">
                <span class="Clip">{{ Item.Artist }}</span>
                <span v-if="Item.DurationMs" class="Time">{{ Duration(Item.DurationMs) }}</span>
              </span>
            </span>
            <span v-if="Item.Status" class="Chip" :class="Item.Status">{{ StatusLabel[Item.Status] }}</span>
            <span v-else class="Add" aria-hidden="true">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                <path d="M12 6v12M6 12h12" />
              </svg>
            </span>
          </button>
        </li>
      </TransitionGroup>

      <div v-else key="idle" />
    </Transition>

    <ConfirmSheet :Track="Picked" @Close="Picked = null" @Sent="OnSent" />
  </section>
</template>

<style scoped>
.View {
  display: flex;
  width: 100%;
  max-width: 760px;
  flex-direction: column;
  gap: 20px;
  margin: 0 auto;
}

.Heading {
  font-family: var(--font-display);
  font-size: 34px;
  font-weight: 600;
  line-height: 1.05;
  letter-spacing: -0.035em;
}

.Dock {
  position: sticky;
  top: var(--head);
  z-index: 5;
  isolation: isolate;
  margin: -8px 0 -12px;
  padding: 8px 0 12px;
}

.Dock::before {
  content: "";
  position: absolute;
  top: 0;
  bottom: 0;
  left: 50%;
  z-index: -1;
  width: 100vw;
  transform: translateX(-50%);
  background: rgba(8, 7, 10, 0.8);
  box-shadow: inset 0 -1px 0 rgba(255, 255, 255, 0.06);
  backdrop-filter: blur(24px) saturate(1.5);
  -webkit-backdrop-filter: blur(24px) saturate(1.5);
  opacity: 0;
  transition: opacity 200ms ease;
  pointer-events: none;
}

.Dock.Docked::before {
  opacity: 1;
}

.Search {
  display: flex;
  height: 56px;
  align-items: center;
  gap: 2px;
  border-radius: 18px;
  background: rgba(22, 21, 26, 0.82);
  box-shadow:
    inset 0 0 0 1px rgba(255, 255, 255, 0.08),
    0 20px 50px -24px rgba(0, 0, 0, 0.9);
  backdrop-filter: blur(24px) saturate(1.3);
  -webkit-backdrop-filter: blur(24px) saturate(1.3);
  padding: 0 6px 0 16px;
  transition: box-shadow 200ms ease;
}

.Search:focus-within {
  box-shadow:
    inset 0 0 0 1px rgba(255, 255, 255, 0.2),
    0 20px 50px -24px rgba(0, 0, 0, 0.9);
}

.Glyph {
  height: 20px;
  width: 20px;
  flex-shrink: 0;
  color: var(--color-ink-mute);
}

.Input {
  min-width: 0;
  height: 100%;
  flex: 1;
  border: 0;
  background: transparent;
  padding: 0 10px;
  font-size: 17px;
  color: var(--color-ink);
  outline: none;
}

.Input::placeholder {
  color: var(--color-ink-faint);
}

.Input::-webkit-search-cancel-button {
  display: none;
}

.Clear {
  display: flex;
  height: 44px;
  width: 44px;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  border-radius: 12px;
  color: var(--color-ink-mute);
}

.Clear svg {
  height: 18px;
  width: 18px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.1);
  padding: 2px;
}

.Panel {
  margin: 0;
  border-radius: 24px;
  background: var(--glass);
  box-shadow:
    inset 0 0 0 1px var(--edge),
    0 24px 60px -30px rgba(0, 0, 0, 0.9);
  backdrop-filter: blur(22px) saturate(1.25);
  -webkit-backdrop-filter: blur(22px) saturate(1.25);
  padding: 4px 0;
  list-style: none;
}

.Panel > li {
  position: relative;
}

.Panel > li + li::before {
  content: "";
  position: absolute;
  top: 0;
  right: 16px;
  left: 78px;
  height: 1px;
  background: rgba(255, 255, 255, 0.05);
}

.Row {
  display: flex;
  width: 100%;
  min-height: 68px;
  align-items: center;
  gap: 14px;
  padding: 8px 14px 8px 12px;
  text-align: left;
}

.Pickable {
  transition: background 160ms ease;
}

.Pickable:hover {
  background: rgba(255, 255, 255, 0.035);
}

.Pickable:active {
  background: rgba(255, 255, 255, 0.06);
}

.Panel > li:first-child .Row {
  border-radius: 20px 20px 0 0;
}

.Panel > li:last-child .Row {
  border-radius: 0 0 20px 20px;
}

.Text {
  display: flex;
  min-width: 0;
  flex: 1;
  flex-direction: column;
  gap: 3px;
}

.Name,
.By {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: 6px;
}

.Name {
  font-size: 16px;
  font-weight: 500;
  letter-spacing: -0.01em;
}

.By {
  font-size: 14px;
  color: var(--color-ink-mute);
}

.Clip {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.Time {
  flex-shrink: 0;
  color: var(--color-ink-faint);
  font-variant-numeric: tabular-nums;
}

.Time::before {
  content: "·";
  margin-right: 6px;
}

.Add {
  display: flex;
  height: 34px;
  width: 34px;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.07);
  color: var(--color-ink-soft);
  transition: background 160ms ease, color 160ms ease;
}

.Add svg {
  height: 16px;
  width: 16px;
}

.Pickable:hover .Add {
  background: var(--color-ink);
  color: #0b0a0d;
}

.Chip {
  flex-shrink: 0;
  border-radius: 999px;
  padding: 6px 11px;
  font-size: 12px;
  font-weight: 500;
  background: rgba(255, 255, 255, 0.07);
  color: var(--color-ink-soft);
}

.Chip.approved {
  background: rgba(224, 168, 168, 0.14);
  color: var(--color-rose);
}

.Chip.added {
  background: rgba(140, 200, 160, 0.12);
  color: #a9d8b8;
}

.Chip.rejected {
  color: var(--color-ink-faint);
}

.GhostCover {
  height: 52px;
  width: 52px;
  flex-shrink: 0;
  border-radius: 11px;
}

.GhostLine {
  display: block;
  height: 12px;
  border-radius: 6px;
}

.GhostLine.Short {
  height: 10px;
}

.Empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 56px 20px;
  text-align: center;
  font-size: 15px;
  line-height: 1.5;
  color: var(--color-ink-mute);
}

.EmptyTitle {
  font-family: var(--font-display);
  font-size: 20px;
  font-weight: 600;
  letter-spacing: -0.02em;
  color: var(--color-ink);
}

.Action {
  min-height: 44px;
  margin-top: 10px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.08);
  padding: 0 20px;
  font-size: 14px;
  font-weight: 500;
  color: var(--color-ink);
}

@media (min-width: 900px) {
  .View {
    gap: 24px;
  }

  .Heading {
    font-size: 44px;
  }

  .Dock {
    position: static;
    margin: 0;
    padding: 0;
  }

  .Dock::before {
    display: none;
  }

  .Search {
    height: 62px;
    border-radius: 20px;
    padding-left: 20px;
  }

  .Input {
    font-size: 18px;
  }

  .Panel {
    border-radius: 28px;
    padding: 6px 0;
  }

  .Row {
    min-height: 76px;
    padding: 10px 18px 10px 16px;
  }

  .Panel > li:first-child .Row {
    border-radius: 24px 24px 0 0;
  }

  .Panel > li:last-child .Row {
    border-radius: 0 0 24px 24px;
  }

  .Panel > li + li::before {
    left: 82px;
    right: 22px;
  }
}

.swap-enter-active,
.swap-leave-active {
  transition: opacity 180ms ease;
}

.swap-enter-from,
.swap-leave-to {
  opacity: 0;
}

.rise-enter-active {
  transition: opacity 380ms ease, transform 480ms var(--ease-smooth);
  transition-delay: var(--Delay);
}

.rise-enter-from {
  opacity: 0;
  transform: translateY(8px);
}
</style>
