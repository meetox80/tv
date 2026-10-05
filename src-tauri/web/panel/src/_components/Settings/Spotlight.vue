<script setup lang="ts">
import { ref, computed, watch, nextTick, type Ref, type ComputedRef } from "vue"
import { useI18n } from "../../i18n"
import Glyph from "./ui/Glyph.vue"
import { Search as Find, type Hit } from "./search"

const { t } = useI18n()

const Props = defineProps<{ Open: boolean }>()
const Emit = defineEmits<{ Close: []; Pick: [hit: Hit] }>()

const Query: Ref<string> = ref("")
const Index: Ref<number> = ref(0)
const InputEl: Ref<HTMLInputElement | null> = ref(null)
const ListEl: Ref<HTMLElement | null> = ref(null)

const Results: ComputedRef<Hit[]> = computed(() => Find(Query.value, t))

const Typed = computed(() => Query.value.trim().length > 0)

function _Reveal(): void {
  const El = ListEl.value?.children[Index.value] as HTMLElement | undefined
  El?.scrollIntoView({ block: "nearest" })
}

function Move(Step: number): void {
  const Count = Results.value.length
  if (!Count) return
  Index.value = (Index.value + Step + Count) % Count
  void nextTick(_Reveal)
}

function Pick(Hit: Hit | undefined): void {
  if (!Hit) return
  Emit("Pick", Hit)
}

function OnKey(E: KeyboardEvent): void {
  if (E.key === "ArrowDown" || (E.key === "n" && E.ctrlKey)) {
    E.preventDefault()
    Move(1)
    return
  }
  if (E.key === "ArrowUp" || (E.key === "p" && E.ctrlKey)) {
    E.preventDefault()
    Move(-1)
    return
  }
  if (E.key === "Enter") {
    E.preventDefault()
    Pick(Results.value[Index.value])
    return
  }
  if (E.key === "Escape") {
    E.preventDefault()
    Emit("Close")
  }
}

watch(Query, () => {
  Index.value = 0
})

watch(() => Props.Open, async Now => {
  if (!Now) return
  Query.value = ""
  Index.value = 0
  await nextTick()
  InputEl.value?.focus()
})
</script>

<template>
  <Teleport to="body">
    <Transition name="spot">
      <div v-if="Open" class="Layer" @mousedown.self="Emit('Close')">
        <div class="Spot" :class="{ Typed }" role="dialog" aria-modal="true" :aria-label="t('settings.search')">
          <div class="Bar">
            <Glyph Name="search" class="BarIcon" />
            <input
              ref="InputEl"
              v-model="Query"
              type="text"
              autocomplete="off"
              spellcheck="false"
              enterkeyhint="go"
              :placeholder="t('settings.spotlight')"
              class="BarInput"
              role="combobox"
              aria-autocomplete="list"
              :aria-expanded="Typed"
              @keydown="OnKey"
            />
          </div>

          <div v-if="Typed" class="Body">
            <div v-if="Results.length" ref="ListEl" class="List" role="listbox">
              <button
                v-for="(R, I) in Results"
                :key="R.Kind + R.Id"
                type="button"
                role="option"
                class="Item"
                :class="{ Current: I === Index }"
                :aria-selected="I === Index"
                @mousemove="Index = I"
                @click="Pick(R)"
              >
                <span class="Mark"><Glyph :Name="R.Icon" /></span>
                <span class="Label">{{ R.Label }}</span>
                <span v-if="R.Context" class="Context">{{ R.Context }}</span>
                <Glyph v-if="R.Kind === 'docs'" Name="external" class="Trail" />
                <kbd v-else-if="I === Index" class="Enter">↵</kbd>
              </button>
            </div>
            <p v-else class="Empty">{{ t("settings.search.empty") }}</p>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.Layer {
  position: fixed;
  inset: 0;
  z-index: 70;
  display: flex;
  justify-content: center;
  background: rgba(3, 3, 5, 0.28);
  padding: 22vh 20px 20px;
}

.Spot {
  display: flex;
  height: fit-content;
  max-height: 64vh;
  width: 100%;
  max-width: 680px;
  flex-direction: column;
  overflow: hidden;
  border-radius: 30px;
  background: rgba(34, 32, 38, 0.62);
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.1),
    inset 0 0 0 1px rgba(255, 255, 255, 0.08),
    0 40px 100px -20px rgba(0, 0, 0, 0.75),
    0 0 0 0.5px rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(48px) saturate(1.7);
  -webkit-backdrop-filter: blur(48px) saturate(1.7);
  transition: border-radius 240ms var(--ease);
}

.Spot.Typed {
  border-radius: 26px;
}

.Bar {
  position: relative;
  display: flex;
  height: 64px;
  flex-shrink: 0;
  align-items: center;
}

.BarIcon {
  position: absolute;
  left: 22px;
  font-size: 22px;
  color: var(--ink-mute);
  stroke-width: 2;
  pointer-events: none;
}

.BarInput {
  height: 100%;
  width: 100%;
  min-width: 0;
  border: 0;
  background: transparent;
  padding: 0 24px 0 58px;
  font: inherit;
  font-size: 23px;
  font-weight: 400;
  letter-spacing: -0.015em;
  color: var(--ink);
  caret-color: var(--rose);
  outline: none;
}

.BarInput::placeholder {
  color: rgba(244, 242, 247, 0.32);
}

.Body {
  min-height: 0;
  overflow-y: auto;
  border-top: 1px solid rgba(255, 255, 255, 0.07);
  padding: 8px;
  scrollbar-width: none;
}

.List {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.Item {
  display: flex;
  height: 46px;
  width: 100%;
  flex-shrink: 0;
  align-items: center;
  gap: 12px;
  border-radius: 14px;
  padding: 0 12px 0 8px;
  text-align: left;
  color: var(--ink-soft);
  cursor: default;
}

.Item.Current {
  background: rgba(224, 168, 168, 0.16);
  color: var(--ink);
}

.Mark {
  display: flex;
  height: 30px;
  width: 30px;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  border-radius: 9px;
  background: rgba(255, 255, 255, 0.06);
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.06);
  font-size: 16px;
  color: var(--ink-soft);
  stroke-width: 1.8;
}

.Current .Mark {
  background: rgba(224, 168, 168, 0.2);
  box-shadow: inset 0 0 0 1px rgba(224, 168, 168, 0.25);
  color: var(--rose-light);
}

.Label {
  min-width: 0;
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 15px;
  font-weight: 500;
}

.Context {
  flex-shrink: 0;
  font-size: 13px;
  color: var(--ink-faint);
}

.Current .Context {
  color: var(--ink-mute);
}

.Trail {
  font-size: 14px;
  color: var(--ink-faint);
}

.Enter {
  display: inline-flex;
  height: 22px;
  min-width: 22px;
  align-items: center;
  justify-content: center;
  border-radius: 7px;
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.14);
  font-family: inherit;
  font-size: 12px;
  color: var(--ink-mute);
}

.Empty {
  margin: 0;
  padding: 16px 14px;
  font-size: 14px;
  color: var(--ink-mute);
}

.spot-enter-active {
  transition: background 200ms ease;
}

.spot-leave-active {
  transition: background 160ms ease, opacity 160ms ease;
}

.spot-enter-active .Spot {
  transition: opacity 180ms ease, transform 320ms var(--spring);
}

.spot-leave-active .Spot {
  transition: opacity 140ms ease, transform 160ms ease;
}

.spot-enter-from,
.spot-leave-to {
  background: rgba(3, 3, 5, 0);
}

.spot-leave-to {
  opacity: 0;
}

.spot-enter-from .Spot {
  opacity: 0;
  transform: scale(0.94);
}

.spot-leave-to .Spot {
  opacity: 0;
  transform: scale(0.98);
}

@media (max-width: 1023px) {
  .Layer {
    padding: calc(env(safe-area-inset-top) + 12px) 12px 12px;
    background: rgba(3, 3, 5, 0.5);
  }

  .Spot {
    max-height: 70dvh;
    border-radius: 24px;
  }

  .Bar {
    height: 56px;
  }

  .BarIcon {
    left: 18px;
    font-size: 19px;
  }

  .BarInput {
    padding-left: 48px;
    font-size: 19px;
  }

  .Item {
    height: 52px;
  }

  .Label {
    font-size: 16px;
  }
}
</style>
