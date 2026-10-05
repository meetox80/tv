<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted, onUnmounted, type Ref, type ComputedRef } from "vue"
import { useI18n } from "../../i18n"
import Glyph from "./ui/Glyph.vue"
import { Sections, type GlyphName } from "./sections"

const { t } = useI18n()

const Props = defineProps<{ Current: string; Owner: string; Pending: number }>()
const Emit = defineEmits<{ Navigate: [id: string, highlight: boolean]; Review: []; Spotlight: [] }>()

interface Item {
  Id: string
  Label: string
  Icon: GlyphName | null
  Child: boolean
}

const _SettleMs = 520

const ListEl: Ref<HTMLElement | null> = ref(null)
const Marker: Ref<{ Top: number; Height: number; Visible: boolean }> = ref({ Top: 0, Height: 0, Visible: false })

let _Frame = 0
let _Until = 0

const Items: ComputedRef<Item[]> = computed(() => {
  const Out: Item[] = []
  for (const S of Sections) {
    Out.push({ Id: S.Id, Label: t(S.Label), Icon: S.Icon, Child: false })
    if (S.Id !== Props.Owner) continue
    for (const A of S.Anchors) Out.push({ Id: A.Id, Label: t(A.Label), Icon: null, Child: true })
  }
  return Out
})

const PendingText = computed(() => (Props.Pending > 99 ? "99+" : String(Props.Pending)))

const MarkerStyle = computed(() => ({
  transform: `translateY(${Marker.value.Top}px)`,
  height: `${Marker.value.Height}px`,
  opacity: Marker.value.Visible ? "1" : "0",
}))

function _Measure(): void {
  const List = ListEl.value
  const El = List?.querySelector<HTMLElement>(`[data-id="${Props.Current}"]`)
  if (!El) {
    Marker.value = { ...Marker.value, Visible: false }
    return
  }
  Marker.value = { Top: El.offsetTop, Height: El.offsetHeight, Visible: true }
}

function _Track(): void {
  _Until = performance.now() + _SettleMs
  if (_Frame) return
  const Step = (): void => {
    _Measure()
    if (performance.now() < _Until) {
      _Frame = requestAnimationFrame(Step)
      return
    }
    _Frame = 0
  }
  _Frame = requestAnimationFrame(Step)
}

function Pick(Id: string, Highlight: boolean): void {
  Emit("Navigate", Id, Highlight)
}

const Mac = typeof navigator !== "undefined" && /Mac|iPhone|iPad/.test(navigator.platform)

watch([() => Props.Current, () => Props.Owner], () => {
  void nextTick(_Track)
})

onMounted(_Track)

onUnmounted(() => {
  cancelAnimationFrame(_Frame)
})
</script>

<template>
  <nav class="Side">
    <div class="Brand">
      <p class="BrandTitle">{{ t("settings.title") }}</p>
    </div>

    <button type="button" class="Search" @click="Emit('Spotlight')">
      <Glyph Name="search" class="SearchIcon" />
      <span class="SearchText">{{ t("settings.search") }}</span>
      <span class="Keys"><kbd>{{ Mac ? "⌘" : "Ctrl" }}</kbd><kbd>K</kbd></span>
    </button>

    <div class="Scroll">
        <div ref="ListEl" class="List">
          <span class="Marker" :style="MarkerStyle" />
          <TransitionGroup name="nav">
            <button
              v-for="I in Items"
              :key="I.Id"
              type="button"
              :data-id="I.Id"
              class="Item"
              :class="{ Child: I.Child, Active: Current === I.Id, Owner: !I.Child && Owner === I.Id }"
              @click="Pick(I.Id, false)"
            >
              <Glyph v-if="I.Icon" :Name="I.Icon" class="ItemIcon" />
              <span class="ItemLabel">{{ I.Label }}</span>
            </button>
          </TransitionGroup>
        </div>
    </div>

    <div class="Footer">
      <button type="button" class="Foot" @click="Emit('Review')">
        <Glyph Name="tray" class="ItemIcon" />
        <span class="ItemLabel">{{ t("voting.review") }}</span>
        <Transition name="pop">
          <span v-if="Pending > 0" :key="PendingText" class="Badge">{{ PendingText }}</span>
        </Transition>
      </button>

      <a class="Foot" href="/llms.txt" target="_blank" rel="noopener noreferrer">
        <Glyph Name="book" class="ItemIcon" />
        <span class="ItemLabel">{{ t("voting.docs") }}</span>
        <Glyph Name="external" class="Out" />
      </a>
    </div>
  </nav>
</template>

<style scoped>
.Side {
  display: flex;
  min-height: 0;
  flex-direction: column;
  padding: 68px 0 32px;
}

.Brand {
  padding: 0 12px 24px;
}

.BrandTitle {
  margin: 0;
  font-family: var(--display);
  font-size: 26px;
  font-weight: 600;
  letter-spacing: -0.03em;
  color: var(--ink);
}

.Search {
  display: flex;
  height: 40px;
  flex-shrink: 0;
  align-items: center;
  gap: 10px;
  margin: 0 2px;
  border-radius: 14px;
  background: rgba(255, 255, 255, 0.035);
  box-shadow: inset 0 0 0 1px var(--edge);
  padding: 0 8px 0 13px;
  text-align: left;
  cursor: pointer;
  transition: box-shadow 200ms ease, background 200ms ease;
}

.Search:hover {
  background: rgba(255, 255, 255, 0.05);
  box-shadow: inset 0 0 0 1px var(--edge-strong);
}

.SearchIcon {
  font-size: 15px;
  color: var(--ink-faint);
}

.SearchText {
  flex: 1;
  font-size: 14px;
  color: var(--ink-faint);
}

.Keys {
  display: flex;
  gap: 3px;
}

.Keys kbd {
  display: inline-flex;
  height: 22px;
  min-width: 22px;
  align-items: center;
  justify-content: center;
  border-radius: 7px;
  box-shadow: inset 0 0 0 1px var(--edge);
  padding: 0 6px;
  font-family: inherit;
  font-size: 11.5px;
  color: var(--ink-faint);
}

.Scroll {
  min-height: 0;
  flex: 1;
  overflow-y: auto;
  margin-top: 20px;
  padding: 0 2px;
  scrollbar-width: none;
}

.Scroll::-webkit-scrollbar {
  display: none;
}

.List {
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.Marker {
  position: absolute;
  top: 0;
  right: 0;
  left: 0;
  border-radius: 14px;
  background: rgba(255, 255, 255, 0.07);
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.05), inset 0 0 0 1px rgba(255, 255, 255, 0.05);
  pointer-events: none;
  transition: transform 420ms var(--ease), height 420ms var(--ease), opacity 200ms ease;
}

.Item,
.Foot {
  position: relative;
  z-index: 1;
  display: flex;
  min-height: 40px;
  width: 100%;
  align-items: center;
  gap: 12px;
  border-radius: 14px;
  padding: 0 12px;
  text-align: left;
  text-decoration: none;
  font-size: 14px;
  color: var(--ink-mute);
  cursor: pointer;
  transition: color 220ms ease, background 220ms ease;
}

.Item:hover,
.Foot:hover {
  color: var(--ink-soft);
}

.Item.Owner,
.Item.Active {
  color: var(--ink);
}

.Item.Child {
  min-height: 34px;
  padding-left: 42px;
  font-size: 13.5px;
}

.Item.Child::before {
  content: "";
  position: absolute;
  top: 6px;
  bottom: 6px;
  left: 20px;
  width: 1.5px;
  border-radius: 2px;
  background: rgba(255, 255, 255, 0.08);
  transition: background 260ms ease;
}

.Item.Child.Active::before {
  background: var(--rose);
}

.ItemIcon {
  font-size: 17px;
  stroke-width: 1.7;
  opacity: 0.9;
}

.ItemLabel {
  min-width: 0;
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.Footer {
  display: flex;
  flex-direction: column;
  gap: 2px;
  margin-top: 12px;
  border-top: 1px solid var(--line);
  padding: 12px 2px 0;
}

.Foot:hover {
  background: rgba(255, 255, 255, 0.04);
}

.Badge {
  display: inline-flex;
  height: 22px;
  min-width: 22px;
  align-items: center;
  justify-content: center;
  border-radius: 999px;
  background: linear-gradient(180deg, var(--rose-light), var(--rose));
  box-shadow: 0 6px 16px -6px rgba(224, 168, 168, 0.7);
  padding: 0 7px;
  font-size: 12px;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
  color: var(--rose-ink);
}

.Out {
  font-size: 14px;
  opacity: 0;
  transform: translate(-4px, 4px);
  transition: opacity 200ms ease, transform 280ms var(--ease);
}

.Foot:hover .Out {
  opacity: 0.8;
  transform: none;
}

.pop-enter-active {
  transition: opacity 200ms ease, transform 380ms var(--spring);
}

.pop-leave-active {
  position: absolute;
  right: 12px;
  transition: opacity 140ms ease, transform 180ms ease;
}

.pop-enter-from,
.pop-leave-to {
  opacity: 0;
  transform: scale(0.5);
}

.nav-move {
  transition: transform 380ms var(--ease);
}

.nav-enter-active {
  transition: opacity 260ms ease 80ms, transform 380ms var(--ease) 40ms;
}

.nav-leave-active {
  position: absolute;
  right: 0;
  left: 0;
  transition: opacity 140ms ease, transform 200ms ease;
}

.nav-enter-from,
.nav-leave-to {
  opacity: 0;
  transform: translateX(-6px);
}
</style>
