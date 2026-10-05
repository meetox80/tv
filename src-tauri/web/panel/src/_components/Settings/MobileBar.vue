<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted, type Ref } from "vue"
import { useI18n } from "../../i18n"
import Glyph from "./ui/Glyph.vue"
import { Sections } from "./sections"

const { t } = useI18n()

const Props = defineProps<{ Owner: string; Pending: number; Scrolled: boolean }>()
const Emit = defineEmits<{ Navigate: [id: string, highlight: boolean]; Review: []; Spotlight: [] }>()

const ChipsEl: Ref<HTMLElement | null> = ref(null)

const PendingText = computed(() => (Props.Pending > 99 ? "99+" : String(Props.Pending)))

function _Center(Smooth: boolean): void {
  const Box = ChipsEl.value
  const Chip = Box?.querySelector<HTMLElement>(`[data-id="${Props.Owner}"]`)
  if (!Box || !Chip) return
  const Left = Chip.offsetLeft - (Box.clientWidth - Chip.offsetWidth) / 2
  Box.scrollTo({ left: Math.max(Left, 0), behavior: Smooth ? "smooth" : "auto" })
}

watch(() => Props.Owner, () => {
  void nextTick(() => _Center(true))
})

onMounted(() => _Center(false))
</script>

<template>
  <header class="MobileBar" :class="{ Solid: Scrolled }">
    <div class="Top">
      <p class="Title">{{ t("settings.title") }}</p>

      <a class="Round" href="/llms.txt" target="_blank" rel="noopener noreferrer" :aria-label="t('voting.docs')">
        <Glyph Name="book" />
      </a>
      <button type="button" class="Round" :aria-label="t('voting.review')" @click="Emit('Review')">
        <Glyph Name="tray" />
        <Transition name="pop">
          <span v-if="Pending > 0" :key="PendingText" class="Badge">{{ PendingText }}</span>
        </Transition>
      </button>
    </div>

    <div ref="ChipsEl" class="Chips">
      <button type="button" class="Chip Find" :aria-label="t('settings.search')" @click="Emit('Spotlight')">
        <Glyph Name="search" />
      </button>
      <button
        v-for="S in Sections"
        :key="S.Id"
        type="button"
        :data-id="S.Id"
        class="Chip"
        :class="{ On: Owner === S.Id }"
        @click="Emit('Navigate', S.Id, false)"
      >
        {{ t(S.Label) }}
      </button>
    </div>
  </header>
</template>

<style scoped>
.MobileBar {
  position: sticky;
  top: 0;
  z-index: 20;
  padding: calc(env(safe-area-inset-top) + 12px) 0 12px;
}

.MobileBar::before {
  content: "";
  position: absolute;
  inset: 0;
  z-index: -1;
  border-bottom: 1px solid var(--line);
  background: rgba(8, 8, 11, 0.72);
  backdrop-filter: blur(24px) saturate(1.4);
  -webkit-backdrop-filter: blur(24px) saturate(1.4);
  opacity: 0;
  transition: opacity 260ms ease;
}

.MobileBar.Solid::before {
  opacity: 1;
}

.Top {
  display: flex;
  height: 44px;
  align-items: center;
  gap: 12px;
  padding: 0 16px;
}

.Logo {
  height: 32px;
  width: 32px;
  flex-shrink: 0;
  border-radius: 10px;
  object-fit: cover;
  box-shadow: inset 0 0 0 1px var(--edge);
}

.Title {
  margin: 0;
  flex: 1;
  font-family: var(--display);
  font-size: 24px;
  font-weight: 600;
  letter-spacing: -0.035em;
  color: var(--ink);
}

.Round {
  position: relative;
  display: flex;
  height: 42px;
  width: 42px;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  border-radius: 999px;
  background: var(--glass-strong);
  box-shadow: inset 0 0 0 1px var(--edge);
  font-size: 19px;
  color: var(--ink-soft);
  stroke-width: 1.7;
}

.Round:active {
  transform: scale(0.95);
}

.Badge {
  position: absolute;
  top: -4px;
  right: -4px;
  display: inline-flex;
  height: 20px;
  min-width: 20px;
  align-items: center;
  justify-content: center;
  border-radius: 999px;
  background: linear-gradient(180deg, var(--rose-light), var(--rose));
  box-shadow: 0 0 0 2px var(--bg);
  padding: 0 6px;
  font-size: 11.5px;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
  color: var(--rose-ink);
}

.Chips {
  display: flex;
  gap: 8px;
  overflow-x: auto;
  margin-top: 12px;
  padding: 2px 16px;
  scrollbar-width: none;
  -webkit-mask-image: linear-gradient(90deg, transparent, #000 12px, #000 calc(100% - 24px), transparent);
  mask-image: linear-gradient(90deg, transparent, #000 12px, #000 calc(100% - 24px), transparent);
}

.Chips::-webkit-scrollbar {
  display: none;
}

.Chip {
  display: inline-flex;
  height: 38px;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.035);
  box-shadow: inset 0 0 0 1px var(--edge);
  padding: 0 16px;
  font-size: 14.5px;
  font-weight: 500;
  white-space: nowrap;
  color: var(--ink-mute);
  transition: background 260ms ease, color 260ms ease, box-shadow 260ms ease;
}

.Chip.On {
  background: rgba(255, 255, 255, 0.11);
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.08), inset 0 0 0 1px rgba(255, 255, 255, 0.1);
  color: var(--ink);
}

.Chip.Find {
  width: 38px;
  padding: 0;
  font-size: 17px;
  color: var(--ink-soft);
}

.pop-enter-active {
  transition: opacity 200ms ease, transform 380ms var(--spring);
}

.pop-leave-active {
  transition: opacity 140ms ease, transform 180ms ease;
}

.pop-enter-from,
.pop-leave-to {
  opacity: 0;
  transform: scale(0.5);
}
</style>
