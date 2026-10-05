<script setup lang="ts">
import { ref, computed, onUnmounted, type Ref } from "vue"
import { useI18n } from "../../i18n"
import type { AdminSong } from "../../_api/voting"
import { Copy } from "../../_api/clipboard"
import Cover from "./Cover.vue"
import { Relative, Day, Line, ProviderName, type RowAction } from "./_format"

const { t } = useI18n()

const Props = defineProps<{
  Song: AdminSong
  Rank?: number
  Actions: RowAction[]
  Busy: boolean
  Tick: number
}>()

const Emit = defineEmits<{ Action: [key: string] }>()

const _MenuRoom = 230

const Copied: Ref<boolean> = ref(false)
const Open: Ref<boolean> = ref(false)
const Up: Ref<boolean> = ref(false)
const MoreEl: Ref<HTMLElement | null> = ref(null)

let _CopiedTimer: ReturnType<typeof setTimeout> | null = null

const Showing = computed(() => Props.Song.Status === "approved" || Props.Song.Status === "added")

const When = computed(() => {
  void Props.Tick
  return Relative(Props.Song.Decided ?? Props.Song.Created)
})

const WhenTitle = computed(() => Day(Props.Song.Decided ?? Props.Song.Created))

const Lead = computed(() => Props.Actions.find(A => A.Tone === "primary") ?? Props.Actions.find(A => A.Tone !== "danger") ?? null)

const Rest = computed(() => Props.Actions.filter(A => A !== Lead.value))

function _OnPointer(E: PointerEvent): void {
  if (MoreEl.value && E.target instanceof Node && MoreEl.value.contains(E.target)) return
  Close()
}

function _OnKey(E: KeyboardEvent): void {
  if (E.key !== "Escape") return
  E.preventDefault()
  Close()
}

function Close(): void {
  if (!Open.value) return
  Open.value = false
  document.removeEventListener("pointerdown", _OnPointer, true)
  document.removeEventListener("keydown", _OnKey)
}

function Toggle(): void {
  if (Open.value) {
    Close()
    return
  }
  const Box = MoreEl.value?.getBoundingClientRect()
  Up.value = !!Box && window.innerHeight - Box.bottom < _MenuRoom
  Open.value = true
  document.addEventListener("pointerdown", _OnPointer, true)
  document.addEventListener("keydown", _OnKey)
}

function Pick(Key: string): void {
  Close()
  Emit("Action", Key)
}

async function CopyLine(): Promise<void> {
  await Copy(Line(Props.Song))
  Copied.value = true
  if (_CopiedTimer) clearTimeout(_CopiedTimer)
  _CopiedTimer = setTimeout(() => {
    Copied.value = false
    Close()
  }, 700)
}

onUnmounted(() => {
  Close()
  if (_CopiedTimer) clearTimeout(_CopiedTimer)
})
</script>

<template>
  <div class="Row" :class="{ Raised: Open }">
    <div class="Art">
      <Cover :Src="Song.Cover" class="Thumb" />
      <span v-if="Rank !== undefined" class="Rank">{{ Rank }}</span>
    </div>

    <div class="Main">
      <p class="Title">
        <span class="Clip">{{ Song.Title }}</span>
        <span v-if="Song.Explicit" class="Explicit" :title="t('voting.review.explicit')">E</span>
      </p>
      <p class="Meta">
        <span v-if="Song.Artist" class="Clip">{{ Song.Artist }}</span>
        <span v-if="Song.Artist" class="Sep" />
        <span class="Keep" :title="WhenTitle">{{ When }}</span>
        <template v-if="Song.Device">
          <span class="Sep Hide" />
          <span class="Device Hide" :class="{ Banned: Song.DeviceBanned }">{{ Song.Device }}</span>
        </template>
      </p>
    </div>

    <div v-if="Showing" class="Score" :title="t('voting.review.score')">
      <span class="ScoreValue">{{ Song.Score }}</span>
      <span class="Votes">
        <span class="Up">↑{{ Song.Up }}</span>
        <span class="Down">↓{{ Song.Down }}</span>
      </span>
    </div>

    <div class="Tools">
      <button
        v-if="Lead"
        type="button"
        class="Action"
        :class="Lead.Tone"
        :disabled="Busy"
        @click="Emit('Action', Lead.Key)"
      >
        {{ Lead.Label }}
      </button>

      <div ref="MoreEl" class="More">
        <button
          type="button"
          class="Dots"
          :class="{ On: Open }"
          :aria-label="t('voting.review.more')"
          :aria-expanded="Open"
          :disabled="Busy"
          @click="Toggle"
        >
          <svg viewBox="0 0 24 24" fill="currentColor">
            <circle cx="5.5" cy="12" r="1.7" />
            <circle cx="12" cy="12" r="1.7" />
            <circle cx="18.5" cy="12" r="1.7" />
          </svg>
        </button>

        <Transition name="menu">
          <div v-if="Open" class="Menu" :class="{ Up }" role="menu">
            <button
              v-for="A in Rest"
              :key="A.Key"
              type="button"
              role="menuitem"
              class="Item"
              :class="A.Tone"
              @click="Pick(A.Key)"
            >
              {{ A.Label }}
            </button>

            <span v-if="Rest.length" class="Divider" />

            <a
              v-if="Song.Link"
              :href="Song.Link"
              target="_blank"
              rel="noopener noreferrer"
              role="menuitem"
              class="Item"
              @click="Close"
            >
              {{ t("voting.review.open") }} {{ ProviderName(Song.Provider) }}
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                <path d="M7 17 17 7M9 7h8v8" />
              </svg>
            </a>

            <button type="button" role="menuitem" class="Item" :class="{ Done: Copied }" @click="CopyLine">
              {{ Copied ? t("voting.review.copied") : t("voting.review.copy") }}
              <svg v-if="!Copied" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
                <rect x="9" y="9" width="11" height="11" rx="2.5" />
                <path d="M5 15V6.5A2.5 2.5 0 0 1 7.5 4H15" />
              </svg>
              <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="m5 12.5 4.5 4.5L19 7.5" />
              </svg>
            </button>
          </div>
        </Transition>
      </div>
    </div>
  </div>
</template>

<style scoped>
.Row {
  position: relative;
  display: grid;
  grid-template-columns: 56px minmax(0, 1fr) auto auto;
  align-items: center;
  column-gap: 18px;
  margin: 0 -14px;
  border-radius: 22px;
  padding: 14px;
  transition: background 200ms ease;
}

.Row.Raised {
  z-index: 5;
}

.Row + .Row::before {
  content: "";
  position: absolute;
  top: 0;
  right: 14px;
  left: 84px;
  height: 1px;
  background: var(--line);
  transition: opacity 200ms ease;
}

.Row:hover,
.Row.Raised {
  background: rgba(255, 255, 255, 0.035);
}

.Row:hover::before,
.Row:hover + .Row::before,
.Row.Raised::before,
.Row.Raised + .Row::before {
  opacity: 0;
}

.Art {
  position: relative;
  height: 56px;
  width: 56px;
}

.Thumb {
  height: 100%;
  width: 100%;
  border-radius: 14px;
  box-shadow: 0 12px 26px -14px rgba(0, 0, 0, 0.95);
  font-size: 22px;
}

.Rank {
  position: absolute;
  top: -6px;
  left: -6px;
  display: flex;
  height: 22px;
  min-width: 22px;
  align-items: center;
  justify-content: center;
  border-radius: 999px;
  background: var(--rose);
  box-shadow: 0 0 0 3px var(--bg), 0 6px 14px -6px rgba(0, 0, 0, 0.8);
  padding: 0 6px;
  font-family: var(--display);
  font-size: 12px;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
  color: var(--rose-ink);
}

.Main {
  min-width: 0;
}

.Title {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: 8px;
  margin: 0;
  font-size: 16px;
  font-weight: 500;
  letter-spacing: -0.01em;
  color: var(--ink);
}

.Clip {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.Explicit {
  display: inline-flex;
  height: 16px;
  width: 16px;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.14);
  font-size: 10px;
  font-weight: 700;
  color: var(--ink-soft);
}

.Meta {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: 8px;
  margin: 5px 0 0;
  font-size: 14px;
  color: var(--ink-mute);
}

.Keep {
  flex-shrink: 0;
  white-space: nowrap;
}

.Sep {
  height: 3px;
  width: 3px;
  flex-shrink: 0;
  border-radius: 999px;
  background: var(--ink-faint);
}

.Device {
  flex-shrink: 0;
  font-family: var(--mono);
  font-size: 12px;
  color: var(--ink-faint);
}

.Device.Banned {
  color: var(--danger);
}

.Score {
  display: flex;
  min-width: 52px;
  flex-direction: column;
  align-items: flex-end;
  gap: 5px;
}

.ScoreValue {
  font-family: var(--display);
  font-size: 24px;
  font-weight: 600;
  line-height: 1;
  letter-spacing: -0.03em;
  font-variant-numeric: tabular-nums;
  color: var(--ink);
}

.Votes {
  display: flex;
  gap: 8px;
  font-size: 12px;
  font-variant-numeric: tabular-nums;
}

.Up {
  color: var(--rose);
}

.Down {
  color: #8fa3c9;
}

.Tools {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 6px;
}

.Action {
  display: inline-flex;
  height: 36px;
  align-items: center;
  border-radius: 999px;
  background: var(--glass-strong);
  box-shadow: inset 0 0 0 1px var(--edge);
  padding: 0 16px;
  font-size: 13.5px;
  font-weight: 500;
  white-space: nowrap;
  color: var(--ink-soft);
  cursor: pointer;
  transition: background 180ms ease, color 180ms ease, transform 140ms ease, opacity 180ms ease;
}

.Action:hover {
  background: rgba(255, 255, 255, 0.1);
  color: var(--ink);
}

.Action:active {
  transform: scale(0.97);
}

.Action.primary {
  background: var(--ink);
  font-weight: 600;
  color: #0c0b0f;
}

.Action.primary:hover {
  background: #fff;
}

.Action:disabled,
.Dots:disabled {
  opacity: 0.4;
  pointer-events: none;
}

.More {
  position: relative;
}

.Dots {
  display: flex;
  height: 36px;
  width: 36px;
  align-items: center;
  justify-content: center;
  border-radius: 999px;
  color: var(--ink-mute);
  cursor: pointer;
  transition: background 180ms ease, color 180ms ease;
}

.Dots svg {
  height: 18px;
  width: 18px;
}

.Dots:hover,
.Dots.On {
  background: rgba(255, 255, 255, 0.08);
  color: var(--ink);
}

.Menu {
  position: absolute;
  top: calc(100% + 8px);
  right: 0;
  z-index: 10;
  display: flex;
  min-width: 232px;
  flex-direction: column;
  border-radius: 18px;
  background: rgba(22, 21, 26, 0.94);
  box-shadow:
    inset 0 0 0 1px var(--edge),
    inset 0 1px 0 rgba(255, 255, 255, 0.06),
    0 30px 60px -20px rgba(0, 0, 0, 0.9);
  backdrop-filter: blur(24px) saturate(1.3);
  -webkit-backdrop-filter: blur(24px) saturate(1.3);
  padding: 6px;
  transform-origin: top right;
}

.Menu.Up {
  top: auto;
  bottom: calc(100% + 8px);
  transform-origin: bottom right;
}

.Item {
  display: flex;
  min-height: 40px;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  border-radius: 12px;
  padding: 0 12px;
  text-align: left;
  font-size: 14px;
  white-space: nowrap;
  color: var(--ink);
  text-decoration: none;
  cursor: pointer;
  transition: background 150ms ease;
}

.Item svg {
  height: 15px;
  width: 15px;
  flex-shrink: 0;
  color: var(--ink-mute);
}

.Item:hover {
  background: rgba(255, 255, 255, 0.07);
}

.Item.danger {
  color: var(--danger);
}

.Item.danger:hover {
  background: rgba(239, 138, 138, 0.1);
}

.Item.Done,
.Item.Done svg {
  color: var(--good);
}

.Divider {
  margin: 5px 10px;
  height: 1px;
  background: var(--line);
}

.menu-enter-active {
  transition: opacity 160ms ease, transform 260ms var(--ease);
}

.menu-leave-active {
  transition: opacity 120ms ease, transform 160ms ease;
}

.menu-enter-from,
.menu-leave-to {
  opacity: 0;
  transform: scale(0.94);
}

@media (max-width: 899px) {
  .Hide {
    display: none;
  }
}

@media (max-width: 639px) {
  .Row {
    grid-template-columns: 52px minmax(0, 1fr) auto;
    column-gap: 14px;
    row-gap: 12px;
    padding: 16px 14px;
  }

  .Row + .Row::before {
    left: 14px;
  }

  .Art {
    height: 52px;
    width: 52px;
  }

  .Tools {
    grid-column: 2 / -1;
    justify-content: flex-start;
  }

  .More {
    margin-left: auto;
    margin-right: -6px;
  }
}
</style>
