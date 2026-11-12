<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, type Ref } from "vue"
import { invoke } from "@tauri-apps/api/core"
import { listen, type UnlistenFn } from "@tauri-apps/api/event"
import Player from "./_components/Player.vue"
import LessonTimer from "./_components/LessonTimer.vue"
import NewsPage from "./_components/NewsPage.vue"
import VotingPage from "./_components/VotingPage.vue"
import SlidesPage from "./_components/SlidesPage.vue"
import TopBar from "./_components/TopBar.vue"
import Backdrop from "./_components/Backdrop.vue"
import Clock from "./_components/Clock.vue"
import Skeleton from "./_components/Skeleton.vue"
import { ReadWeather, type WeatherConfig } from "./_components/weather"
import { SlideImages, type Slide, type SlideCollection } from "./_components/slides"
import { Timeline, Resolve, type Schedule } from "./_schedule"
import { Preload, Decoded } from "./_images"

interface Track {
  Title: string
  Artist: string
}


type PageId = string

interface ScreenPage {
  Id: PageId
  Enabled: boolean
  Seconds: number
}

const _SlidesPrefix = "slides:"

interface ScreenConfig {
  Weather: WeatherConfig
  ScrobbleInLessons?: boolean
  Notice?: string
  Pages?: ScreenPage[]
}

interface BoardSong {
  Id: number
  Title: string
  Artist: string
  Cover: string | null
  Score: number
  Explicit: boolean | null
}

interface VotingBoard {
  Enabled: boolean
  Url: string | null
  Songs: BoardSong[]
}

interface Post {
  Id: number
  Title: string
  Date: string
  Image: string | null
  Body: string[]
}

interface CachedPost {
  Id: number
  Date: string
  Title: string
  Content: string
  Image: string | null
}

const _StageWidth = 1920
const _StageHeight = 1080
const _WeatherRefreshMs = 10 * 60 * 1000
const _BoardRefreshMs = 60 * 1000
const _DefaultRotation: ScreenPage[] = [
  { Id: "lesson", Enabled: true, Seconds: 30 },
  { Id: "news", Enabled: true, Seconds: 30 },
  { Id: "voting", Enabled: true, Seconds: 30 },
]
const _MinuteMs = 60 * 1000
const _HoldMs = 700
const _ImageWaitMs = 2500
const _TrackWaitMs = 3000


const ShownTrack: Ref<Track | null> = ref(null)
const CoverUrl: Ref<string | null> = ref(null)
const ShownCover: Ref<string | null> = ref(null)
const Lessons: Ref<Schedule | null> = ref(null)
const ScrobbleInLessons: Ref<boolean> = ref(true)
const Notice: Ref<string> = ref("")
const Bpm: Ref<number | null> = ref(null)
const Article: Ref<Post | null> = ref(null)
const Board: Ref<VotingBoard | null> = ref(null)
const Rotation: Ref<ScreenPage[]> = ref(_DefaultRotation)
const Collections: Ref<SlideCollection[]> = ref([])
const SlideCursor: Ref<Record<string, number>> = ref({})
const PageIndex: Ref<number> = ref(0)
const PageTick: Ref<number> = ref(0)
const Weather: Ref<WeatherConfig | null> = ref(null)
const Temperature: Ref<number | null> = ref(null)
const Condition: Ref<string | null> = ref(null)
const Ready: Ref<boolean> = ref(false)
const _Opened = performance.now()

const Now: Ref<Date> = ref(new Date())
const Scale: Ref<number> = ref(1)

function _Fit(): void {
  Scale.value = Math.min(
    window.innerWidth / _StageWidth,
    window.innerHeight / _StageHeight,
  )
}


let _Listeners: UnlistenFn[] = []
let _CoverToken = 0
let _BoardToken = 0
let _NewsToken = 0
let _SlidesToken = 0
let _PendingTrack: Track | null = null
let _PendingTimer: ReturnType<typeof setTimeout> | null = null
let _WeatherTimer: ReturnType<typeof setInterval> | null = null
let _BoardTimer: ReturnType<typeof setInterval> | null = null
let _PageTimer: ReturnType<typeof setTimeout> | null = null
let _ClockTimer: ReturnType<typeof setTimeout> | null = null

function _ScheduleClock(): void {
  _ClockTimer = setTimeout(() => {
    Now.value = new Date()
    _ScheduleClock()
  }, _MinuteMs - (Date.now() % _MinuteMs))
}

async function _FetchWeather(): Promise<void> {
  const Reading = await ReadWeather(Weather.value)
  if (!Reading) return
  Temperature.value = Reading.Temperature
  Condition.value = Reading.Condition
}

async function _FetchBpm(): Promise<void> {
  try {
    Bpm.value = await invoke<number | null>("get_track_bpm")
  } catch {
    Bpm.value = null
  }
}

function _SameTrack(A: Track | null, B: Track | null): boolean {
  return !!A && !!B && A.Title === B.Title && A.Artist === B.Artist
}

function _StopPendingTimer(): void {
  if (!_PendingTimer) return
  clearTimeout(_PendingTimer)
  _PendingTimer = null
}

function _ShowTrack(Next: Track): void {
  ShownTrack.value = Next
  Bpm.value = null
  void _FetchBpm()
}

function _FlushTrack(Cover: string | null): void {
  const Next = _PendingTrack
  if (!Next) return

  _PendingTrack = null
  _StopPendingTimer()
  ShownCover.value = Cover
  _ShowTrack(Next)
}

function _ApplyTrack(Next: Track | null, Immediate = false): void {
  if (!Next) return

  if (_SameTrack(Next, ShownTrack.value)) {
    _PendingTrack = null
    _StopPendingTimer()
    return
  }

  if (Immediate) {
    _ShowTrack(Next)
    return
  }

  if (_SameTrack(Next, _PendingTrack)) return

  _PendingTrack = Next
  _StopPendingTimer()
  _PendingTimer = setTimeout(() => _FlushTrack(null), _TrackWaitMs)
}

function _RevealCover(Cover: string | null): void {
  if (_PendingTrack) _FlushTrack(Cover)
  else ShownCover.value = Cover
}

const _Line = computed(() => Timeline(Lessons.value))

const Muted = computed(() => {
  if (ScrobbleInLessons.value) return false
  return Resolve(_Line.value, Now.value.getTime())?.IsLesson ?? false
})

const VisibleCover = computed(() => (Muted.value ? null : ShownCover.value))

const VotingUrl = computed(() => (Board.value?.Enabled && Board.value.Url ? Board.value.Url : null))

function _Collection(Id: PageId): SlideCollection | null {
  if (!Id.startsWith(_SlidesPrefix)) return null
  const Key = Id.slice(_SlidesPrefix.length)
  return Collections.value.find(C => C.Id === Key) ?? null
}

const Pages = computed<PageId[]>(() => {
  const List = Rotation.value
    .filter(P => {
      if (P.Id === "voting") return !!VotingUrl.value
      if (P.Id === "news") return P.Enabled && !!Article.value
      if (P.Id.startsWith(_SlidesPrefix)) return P.Enabled && !!_Collection(P.Id)?.Slides.length
      return P.Enabled
    })
    .map(P => P.Id)
  return List.length ? List : ["lesson"]
})

const ActivePage = computed(() => Pages.value[PageIndex.value % Pages.value.length])

const ActiveSlide = computed<Slide | null>(() => {
  const Collection = _Collection(ActivePage.value)
  if (!Collection?.Slides.length) return null
  const At = SlideCursor.value[Collection.Id] ?? 0
  return Collection.Slides[At % Collection.Slides.length]
})

function _AdvanceSlides(): void {
  const Collection = _Collection(ActivePage.value)
  if (!Collection?.Slides.length) return
  const Previous = SlideCursor.value[Collection.Id]
  const Next = Previous === undefined ? 0 : (Previous + 1) % Collection.Slides.length
  SlideCursor.value = { ...SlideCursor.value, [Collection.Id]: Next }
}

const RotateMs = computed(() => (Rotation.value.find(P => P.Id === ActivePage.value)?.Seconds ?? 30) * 1000)

let _PageStartedAt = 0

function _Report(): void {
  invoke("report_page", {
    page: ActivePage.value,
    pages: Pages.value,
    seconds: Math.round(RotateMs.value / 1000),
    elapsed: Math.max(0, Math.round(performance.now() - _PageStartedAt)),
  }).catch(() => {})
}

function _Schedule(): void {
  if (_PageTimer) clearTimeout(_PageTimer)
  _PageStartedAt = performance.now()
  _AdvanceSlides()
  _Report()
  _PageTimer = setTimeout(() => {
    PageIndex.value = (PageIndex.value + 1) % Pages.value.length
    PageTick.value += 1
  }, RotateMs.value)
}

watch(PageTick, _Schedule)

watch(Pages, (Next, Prev) => {
  const Current = Prev[PageIndex.value % Prev.length]
  const Kept = Next.indexOf(Current)

  if (Kept >= 0) {
    PageIndex.value = Kept
    _Report()
    return
  }

  PageIndex.value = 0
  PageTick.value += 1
})

async function _ApplyBoard(Next: VotingBoard | null): Promise<void> {
  const Token = ++_BoardToken

  if (!Next) {
    Board.value = Next
    return
  }

  const States = await Preload("board", Next.Songs.map(Song => Song.Cover), _ImageWaitMs)
  if (Token !== _BoardToken) return

  Board.value = {
    ...Next,
    Songs: Next.Songs.map(Song => ({
      ...Song,
      Cover: Song.Cover && States.get(Song.Cover) !== "failed" ? Song.Cover : null,
    })),
  }
}

async function _FetchBoard(): Promise<void> {
  try {
    await _ApplyBoard(await invoke<VotingBoard>("get_voting_board"))
  } catch {}
}

const _Parser = new DOMParser()

function _Decode(Html: string): string {
  const Doc = _Parser.parseFromString(Html, "text/html")
  return (Doc.body.textContent ?? "").replace(/\s+/g, " ").trim()
}

const _BodyBudget = 420

function _Paragraphs(Html: string): string[] {
  const Doc = _Parser.parseFromString(Html, "text/html")
  const Out: string[] = []
  let Used = 0

  for (const Node of Array.from(Doc.querySelectorAll("p"))) {
    const Text = (Node.textContent ?? "").replace(/\s+/g, " ").trim()
    if (Text.length < 2) continue

    if (Used + Text.length > _BodyBudget) {
      const Room = _BodyBudget - Used
      if (Room > 120) {
        const Cut = Text.slice(0, Room).replace(/\s+\S*$/, "").replace(/[\s.,;:–—-]+$/, "")
        Out.push(`${Cut}…`)
      }
      break
    }

    Out.push(Text)
    Used += Text.length
  }

  return Out
}

async function _ApplyNews(Cached: CachedPost | null): Promise<void> {
  if (!Cached) return

  const Title = _Decode(Cached.Title)
  if (!Title) return

  const Token = ++_NewsToken
  const States = await Preload("news", [Cached.Image], _ImageWaitMs)
  if (Token !== _NewsToken) return

  Article.value = {
    Id: Cached.Id,
    Title,
    Date: Cached.Date,
    Image: Cached.Image && States.get(Cached.Image) !== "failed" ? Cached.Image : null,
    Body: _Paragraphs(Cached.Content),
  }
}

async function _FetchNews(): Promise<void> {
  try {
    await _ApplyNews(await invoke<CachedPost | null>("get_news"))
  } catch {}
}

async function _ApplySlides(Next: SlideCollection[] | null): Promise<void> {
  const Token = ++_SlidesToken
  const List = Next ?? []

  await Preload("slides", SlideImages(List), _ImageWaitMs)
  if (Token !== _SlidesToken) return

  Collections.value = List
}

async function _FetchLessons(): Promise<void> {
  try {
    Lessons.value = await invoke<Schedule>("get_lesson_times")
  } catch {}
}

function _ApplyScreen(Config: ScreenConfig | null): void {
  Weather.value = Config?.Weather ?? null
  ScrobbleInLessons.value = Config?.ScrobbleInLessons ?? true
  Notice.value = (Config?.Notice ?? "").trim()
  const Next = Config?.Pages?.length ? Config.Pages : _DefaultRotation
  if (JSON.stringify(Next) !== JSON.stringify(Rotation.value)) {
    Rotation.value = Next
    PageTick.value += 1
  }
  void _FetchWeather()
  void _FetchLessons()
}

function _Wait(Ms: number): Promise<void> {
  return new Promise(Resolve => setTimeout(Resolve, Ms))
}

function _Frames(Count: number): Promise<void> {
  return new Promise(Resolve => {
    const Step = (Left: number): void => {
      if (Left <= 0) Resolve()
      else requestAnimationFrame(() => Step(Left - 1))
    }
    Step(Count)
  })
}

async function _Settle(): Promise<void> {
  const Cover = CoverUrl.value
  const Art = Cover ? Preload("cover", [Cover], _ImageWaitMs) : Promise.resolve()
  const Hold = _Wait(Math.max(0, _HoldMs - (performance.now() - _Opened)))
  await Promise.all([Art, Hold])
  await _Frames(2)
}

async function _ApplyCover(Next: string | null): Promise<void> {
  CoverUrl.value = Next
  const Token = ++_CoverToken

  if (!Next) {
    _RevealCover(null)
    return
  }

  if (Next === ShownCover.value && !_PendingTrack) return

  const States = await Preload("cover", [Next], _ImageWaitMs)
  if (Token !== _CoverToken) return

  const State = States.get(Next)
  if (State !== "pending") {
    _RevealCover(State === "ready" ? Next : null)
    return
  }

  _FlushTrack(null)
  const Loaded = await Decoded(Next)
  if (Token !== _CoverToken) return

  ShownCover.value = Loaded ? Next : null
}

const _FullscreenKeys = ["f11", "f", "escape"]

function _OnKeyDown(Event: KeyboardEvent): void {
  if (!_FullscreenKeys.includes(Event.key.toLowerCase())) return
  Event.preventDefault()
  invoke("toggle_fullscreen").catch(() => {})
}

onMounted(async () => {
  _Fit()
  window.addEventListener("resize", _Fit)

  _Listeners.push(await listen<Track | null>("track-changed", (E) => {
    _ApplyTrack(E.payload)
  }))

  _Listeners.push(await listen<string | null>("cover-changed", (E) => {
    void _ApplyCover(E.payload)
  }))

  _Listeners.push(await listen<VotingBoard>("voting-changed", (E) => {
    void _ApplyBoard(E.payload)
  }))

  _Listeners.push(await listen<ScreenConfig>("screen-changed", (E) => {
    _ApplyScreen(E.payload)
  }))

  _Listeners.push(await listen<CachedPost | null>("news-changed", (E) => {
    void _ApplyNews(E.payload)
  }))

  _Listeners.push(await listen<SlideCollection[]>("slides-changed", (E) => {
    void _ApplySlides(E.payload)
  }))

  try {
    await _ApplySlides(await invoke<SlideCollection[]>("get_slides"))
  } catch {}

  try {
    _ApplyTrack(await invoke<Track | null>("get_current_track"), true)
  } catch {}

  try {
    void _ApplyCover(await invoke<string | null>("get_cover_url"))
  } catch {}

  try {
    _ApplyScreen(await invoke<ScreenConfig>("get_screen_config"))
  } catch {}

  await Promise.all([_FetchLessons(), _FetchNews(), _FetchBoard()])
  await _Settle()
  Ready.value = true

  _WeatherTimer = setInterval(_FetchWeather, _WeatherRefreshMs)
  _BoardTimer = setInterval(_FetchBoard, _BoardRefreshMs)
  _Schedule()
  _ScheduleClock()
  window.addEventListener("keydown", _OnKeyDown)
})

onUnmounted(() => {
  _Listeners.forEach(Off => Off())
  _Listeners = []
  if (_WeatherTimer) clearInterval(_WeatherTimer)
  if (_BoardTimer) clearInterval(_BoardTimer)
  if (_PageTimer) clearTimeout(_PageTimer)
  if (_ClockTimer) clearTimeout(_ClockTimer)
  _StopPendingTimer()
  window.removeEventListener("keydown", _OnKeyDown)
  window.removeEventListener("resize", _Fit)
})
</script>

<template>
  <div class="Viewport">
    <div class="Stage" :style="{ transform: `translate(-50%, -50%) scale(${Scale})` }">
      <Backdrop :Cover="VisibleCover" class="Glow" :class="{ Lit: Ready }" />

      <Transition name="reveal" :duration="{ enter: 1200, leave: 820 }">
        <Skeleton v-if="!Ready" />

        <div v-else class="Content">
          <TopBar :Notice="Notice" :Pages="Pages" :Active="PageIndex % Pages.length" class="Rise" style="--delay: 0ms" />

          <div class="Pages Rise" style="--delay: 90ms">
            <Transition name="page" mode="out-in">
              <LessonTimer
                v-if="ActivePage === 'lesson'"
                key="lesson"
                :Schedule="Lessons"
                :Bpm="Muted ? null : Bpm"
                class="Centre"
              />
              <NewsPage v-else-if="ActivePage === 'news'" key="news" :Post="Article" class="Fill" />
              <VotingPage
                v-else-if="ActivePage === 'voting' && VotingUrl"
                key="voting"
                :Url="VotingUrl"
                :Songs="Board?.Songs ?? []"
                class="Fill"
              />
              <SlidesPage
                v-else-if="ActiveSlide"
                :key="`${ActivePage}:${ActiveSlide.Id}`"
                :Slide="ActiveSlide"
                class="Fill"
              />
            </Transition>
          </div>

          <div class="Bottom Rise" style="--delay: 180ms">
            <Clock :Now="Now" :Temperature="Temperature" :Condition="Condition" />

            <Player :Track="ShownTrack" :Cover="VisibleCover" :Muted="Muted" />
          </div>

          <div v-if="Pages.length > 1" class="Rail Rise" style="--delay: 260ms">
            <div :key="PageTick" class="RailFill" :style="{ animationDuration: `${RotateMs}ms` }" />
          </div>
        </div>
      </Transition>
    </div>
  </div>
</template>

<style scoped>
.Viewport {
  position: relative;
  height: 100vh;
  width: 100vw;
  overflow: hidden;
  background: #050507;
}

.Stage {
  position: absolute;
  top: 50%;
  left: 50%;
  height: 1080px;
  width: 1920px;
  overflow: hidden;
  background: #050507;
  transform-origin: center center;
}

.Content {
  position: absolute;
  inset: 0;
  z-index: 10;
}

.Glow {
  opacity: 0;
  transition: opacity 1400ms ease 120ms;
}

.Glow.Lit {
  opacity: 1;
}

.reveal-enter-active .Rise {
  animation: rise 680ms cubic-bezier(0.2, 0.8, 0.2, 1) both;
  animation-delay: calc(var(--delay) + 180ms);
}

@keyframes rise {
  from {
    opacity: 0;
    transform: translateY(14px);
  }
}

@media (prefers-reduced-motion: reduce) {
  .reveal-enter-active .Rise {
    animation-name: fade;
  }

  @keyframes fade {
    from {
      opacity: 0;
    }
  }
}

.Pages {
  position: absolute;
  top: 168px;
  right: 72px;
  bottom: 214px;
  left: 72px;
  z-index: 10;
  display: flex;
}

.Centre {
  margin: auto;
}

.Fill {
  height: 100%;
  width: 100%;
}

.page-enter-active,
.page-leave-active {
  transition: opacity 420ms ease, transform 620ms cubic-bezier(0.2, 0.9, 0.3, 1);
}

.page-enter-from {
  opacity: 0;
  transform: translateY(22px);
}

.page-leave-to {
  opacity: 0;
  transform: translateY(-22px);
}

.Bottom {
  position: absolute;
  right: 72px;
  bottom: 62px;
  left: 72px;
  z-index: 10;
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 48px;
}

.Rail {
  position: absolute;
  right: 0;
  bottom: 0;
  left: 0;
  z-index: 20;
  height: 2px;
  background: rgba(255, 255, 255, 0.05);
}

.RailFill {
  height: 100%;
  width: 100%;
  transform-origin: left center;
  background: linear-gradient(to right, rgba(255, 255, 255, 0.28), rgba(255, 255, 255, 0.62));
  animation: rail-grow linear forwards;
}

@keyframes rail-grow {
  from {
    transform: scaleX(0);
  }
  to {
    transform: scaleX(1);
  }
}
</style>
