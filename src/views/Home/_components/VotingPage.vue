<script setup lang="ts">
import { computed, reactive, ref, watch, onMounted, onUnmounted, type Ref } from "vue"
import QRCode from "qrcode"
import RollingDigits from "./RollingDigits.vue"
import { useI18n } from "../../../i18n"

interface BoardSong {
  Id: number
  Title: string
  Artist: string
  Cover: string | null
  Score: number
  Explicit: boolean | null
}

const Props = defineProps<{
  Url: string
  Songs: BoardSong[]
}>()

const { t } = useI18n()

const _Limit = 6
const _Quiet = 4
const _Pill = 40
const _Levels = ["H", "Q", "M", "L"] as const

const StageEl: Ref<HTMLElement | null> = ref(null)
const Unit: Ref<number> = ref(0)

let _Observer: ResizeObserver | null = null

const Host = computed(() =>
  Props.Url.replace(/^https?:\/\//i, "").replace(/\/+$/, "")
)

function _Encode(Text: string): QRCode.QRCode {
  const Floor = QRCode.create(Text, { errorCorrectionLevel: "L" })
  for (const Level of _Levels) {
    try {
      return QRCode.create(Text, { errorCorrectionLevel: Level, version: Floor.version })
    } catch {
      continue
    }
  }
  return Floor
}

const Code = computed(() => {
  const Qr = _Encode(Props.Url)
  const Size = Qr.modules.size
  const Data = Qr.modules.data

  let Modules = ""
  for (let Y = 0; Y < Size; Y++) {
    let X = 0
    while (X < Size) {
      if (!Data[Y * Size + X]) {
        X++
        continue
      }
      let End = X
      while (End < Size && Data[Y * Size + End]) End++
      Modules += `M${X},${Y}h${End - X}v1h-${End - X}z`
      X = End
    }
  }

  return { Size, Modules, Span: Size + _Quiet * 2 }
})

const Side = computed(() => Unit.value * Code.value.Span)

const PlateStyle = computed(() => ({
  width: `${Side.value}px`,
  height: `${Side.value + _Pill}px`,
  paddingBottom: `${_Pill}px`,
  marginBottom: `${_Pill}px`,
}))

function _Measure(): void {
  const El = StageEl.value
  if (!El) return
  const Room = Math.min(El.clientWidth, El.clientHeight - _Pill * 2)
  Unit.value = Math.max(Math.floor(Room / Code.value.Span), 1)
}

watch(() => Code.value.Span, _Measure)

onMounted(() => {
  _Measure()
  if (!StageEl.value) return
  _Observer = new ResizeObserver(_Measure)
  _Observer.observe(StageEl.value)
})

onUnmounted(() => {
  _Observer?.disconnect()
})

const Top = computed(() => Props.Songs.slice(0, _Limit))

function Points(Score: number): string {
  const Count = Math.abs(Score)
  const Tens = Count % 100
  const Units = Count % 10
  if (Count === 1) return t("voting.points.one")
  if (Units >= 2 && Units <= 4 && (Tens < 12 || Tens > 14)) return t("voting.points.few")
  return t("voting.points.many")
}

const Broken = reactive(new Set<number>())
</script>

<template>
  <div class="Voting">
    <div class="Invite">
      <div ref="StageEl" class="Stage">
        <div v-if="Unit" class="Plate" :style="PlateStyle">
          <svg
            class="Code"
            :width="Side"
            :height="Side"
            :viewBox="`${-_Quiet} ${-_Quiet} ${Code.Span} ${Code.Span}`"
            xmlns="http://www.w3.org/2000/svg"
            shape-rendering="crispEdges"
          >
            <path :d="Code.Modules" fill="#000" />
          </svg>
          <p class="Host">{{ Host }}</p>
        </div>
      </div>
    </div>

    <div class="Board">
      <Transition name="swap" mode="out-in">
        <TransitionGroup v-if="Top.length" key="list" tag="ol" name="rank" class="List">
          <li v-for="(Song, Index) in Top" :key="Song.Id" class="Row" :class="{ Lead: Index === 0 }">
            <span class="Rank">{{ Index + 1 }}</span>

            <div class="Art">
              <img
                v-if="Song.Cover && !Broken.has(Song.Id)"
                :src="Song.Cover"
                alt=""
                class="ArtImg"
                @error="Broken.add(Song.Id)"
              />
              <svg v-else class="ArtNone" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <path d="M9 18V5l12-2v13" />
                <circle cx="6" cy="18" r="3" />
                <circle cx="18" cy="16" r="3" />
              </svg>
            </div>

            <div class="Text">
              <p class="Title">
                <span class="TitleText">{{ Song.Title }}</span>
                <span v-if="Song.Explicit" class="Explicit">E</span>
              </p>
              <p class="Artist">{{ Song.Artist }}</p>
            </div>

            <div class="Score" :class="{ Sunk: Song.Score < 0 }">
              <span v-if="Song.Score < 0" class="Minus">−</span>
              <RollingDigits :Value="String(Math.abs(Song.Score))" class="Digits" />
              <span class="Unit">{{ Points(Song.Score) }}</span>
            </div>
          </li>
        </TransitionGroup>

        <div v-else key="empty" class="Empty">
          <p class="EmptyTitle">{{ t("voting.empty.title") }}</p>
          <p class="EmptyHint">{{ t("voting.empty.hint") }}</p>
        </div>
      </Transition>
    </div>
  </div>
</template>

<style scoped>
.Voting {
  display: flex;
  width: 100%;
  height: 100%;
  gap: 28px;
}

.Board {
  position: relative;
  overflow: hidden;
  border-radius: 22px;
  background: rgba(255, 255, 255, 0.04);
  box-shadow:
    inset 0 0 0 1px rgba(255, 255, 255, 0.08),
    0 26px 64px -26px rgba(0, 0, 0, 0.9);
  backdrop-filter: blur(20px) saturate(1.3);
}

.Invite {
  display: flex;
  width: 36%;
  flex-shrink: 0;
}

.Stage {
  display: flex;
  min-height: 0;
  width: 100%;
  flex: 1;
  align-items: center;
  justify-content: center;
}

.Plate {
  position: relative;
  box-sizing: border-box;
  flex-shrink: 0;
  border-radius: 28px;
  background: #fff;
  box-shadow: 0 30px 70px -30px rgba(0, 0, 0, 0.9);
}

.Code {
  display: block;
}

.Host {
  position: absolute;
  bottom: 0;
  left: 50%;
  transform: translate(-50%, 50%);
  border-radius: 999px;
  padding: 18px 34px;
  background: #0c0a09;
  box-shadow:
    0 0 0 6px #fff,
    0 18px 40px -18px rgba(0, 0, 0, 0.9);
  font-family: var(--font-timer);
  font-size: 28px;
  font-weight: 500;
  letter-spacing: -0.01em;
  line-height: 1;
  white-space: nowrap;
  color: rgba(255, 255, 255, 0.95);
}

.Board {
  display: flex;
  min-width: 0;
  flex: 1;
  flex-direction: column;
  padding: 44px 56px 40px;
}

.List {
  position: relative;
  display: flex;
  min-height: 0;
  flex: 1;
  flex-direction: column;
  justify-content: flex-start;
}

.Row {
  display: flex;
  height: 98px;
  flex-shrink: 0;
  align-items: center;
  gap: 26px;
  border-top: 1px solid rgba(255, 255, 255, 0.05);
}

.Row:first-child {
  border-top-color: transparent;
}

.Rank {
  width: 38px;
  flex-shrink: 0;
  font-family: var(--font-timer);
  font-size: 30px;
  font-weight: 600;
  letter-spacing: -0.03em;
  color: rgba(255, 255, 255, 0.3);
  font-variant-numeric: tabular-nums;
  text-align: center;
}

.Lead .Rank {
  color: rgba(255, 255, 255, 0.92);
}

.Art {
  display: flex;
  height: 70px;
  width: 70px;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  border-radius: 14px;
  background: rgba(255, 255, 255, 0.05);
  box-shadow:
    inset 0 0 0 1px rgba(255, 255, 255, 0.08),
    0 14px 30px -16px rgba(0, 0, 0, 0.9);
}

.ArtImg {
  height: 100%;
  width: 100%;
  object-fit: cover;
}

.ArtNone {
  height: 28px;
  width: 28px;
  color: rgba(255, 255, 255, 0.24);
}

.Text {
  display: flex;
  min-width: 0;
  flex: 1;
  flex-direction: column;
}

.Title {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: 12px;
  font-family: var(--font-clock);
  font-size: 29px;
  font-weight: 600;
  line-height: 1.2;
  letter-spacing: -0.022em;
  color: rgba(255, 255, 255, 0.95);
}

.TitleText {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.Explicit {
  display: inline-flex;
  height: 22px;
  width: 22px;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  border-radius: 5px;
  background: rgba(255, 255, 255, 0.16);
  font-family: var(--font-body);
  font-size: 13px;
  font-weight: 700;
  letter-spacing: 0;
  color: rgba(255, 255, 255, 0.78);
}

.Artist {
  margin-top: 4px;
  overflow: hidden;
  font-family: var(--font-clock);
  font-size: 22px;
  font-weight: 400;
  line-height: 1.25;
  letter-spacing: -0.008em;
  color: rgba(255, 255, 255, 0.5);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.Score {
  display: flex;
  flex-shrink: 0;
  align-items: center;
  gap: 2px;
  font-family: var(--font-timer);
  font-size: 38px;
  color: rgba(255, 255, 255, 0.92);
}

.Score.Sunk {
  color: rgba(255, 255, 255, 0.4);
}

.Unit {
  min-width: 4.6em;
  margin-left: 12px;
  font-family: var(--font-clock);
  font-size: 22px;
  font-weight: 500;
  line-height: 1;
  letter-spacing: -0.008em;
  color: rgba(255, 255, 255, 0.45);
  transform: translateY(0.22em);
}

.Minus {
  font-weight: 700;
  line-height: 1;
}

.Empty {
  display: flex;
  flex: 1;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
}

.EmptyTitle {
  font-family: var(--font-clock);
  font-size: 50px;
  font-weight: 600;
  line-height: 1.1;
  letter-spacing: -0.032em;
  color: rgba(255, 255, 255, 0.96);
  text-wrap: balance;
}

.EmptyHint {
  margin-top: 18px;
  max-width: 620px;
  font-family: var(--font-clock);
  font-size: 27px;
  font-weight: 400;
  line-height: 1.5;
  letter-spacing: -0.008em;
  color: rgba(255, 255, 255, 0.55);
  text-wrap: balance;
}

.rank-move {
  transition: transform 820ms cubic-bezier(0.2, 0.9, 0.3, 1);
}

.rank-enter-active {
  transition: opacity 520ms ease 160ms, transform 720ms cubic-bezier(0.2, 0.9, 0.3, 1) 160ms;
}

.rank-leave-active {
  position: absolute;
  right: 0;
  left: 0;
  transition: opacity 360ms ease, transform 420ms ease;
}

.rank-enter-from {
  opacity: 0;
  transform: translateX(28px);
}

.rank-leave-to {
  opacity: 0;
  transform: translateX(-20px);
}

.swap-enter-active,
.swap-leave-active {
  transition: opacity 420ms ease, transform 620ms cubic-bezier(0.2, 0.9, 0.3, 1);
}

.swap-enter-from,
.swap-leave-to {
  opacity: 0;
  transform: translateY(14px);
}
</style>
