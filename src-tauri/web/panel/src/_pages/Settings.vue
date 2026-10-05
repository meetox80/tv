<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick, watch, type Ref } from "vue"
import * as Api from "../_api/settings"
import * as Cache from "../_api/cache"
import * as VotingApi from "../_api/voting"
import type { ScrobbleMethod, SystemInfo } from "../_api/settings"
import Sidebar from "../_components/Settings/Sidebar.vue"
import MobileBar from "../_components/Settings/MobileBar.vue"
import Spotlight from "../_components/Settings/Spotlight.vue"
import type { Hit } from "../_components/Settings/search"
import Scrobbling from "../_components/Settings/Scrobbling.vue"
import Management from "../_components/Settings/Management.vue"
import Cloudflare from "../_components/Settings/Cloudflare.vue"
import Plugins from "../_components/Settings/Plugins.vue"
import Screen from "../_components/Settings/Screen.vue"
import Slides from "../_components/Settings/Slides.vue"
import About from "../_components/Settings/About.vue"
import Performance from "../_components/Settings/Performance.vue"
import Voting from "../_components/Settings/Voting.vue"
import Review from "../_components/Voting/Review.vue"
import { Wide } from "../_components/Settings/wide"
import { OwnerOf, Sections } from "../_components/Settings/sections"

const _StorageKey = "panel_settings_section"
const _CacheKey = "panel_settings_cache"
const _PollMs = 15 * 1000
const _FlashMs = 1700
const _LockMs = 1200

const _Cached = Cache.Read<Api.Settings>(_CacheKey)

const Current: Ref<string> = ref(_RestoreSection())
const Method = ref<ScrobbleMethod | null>(_Cached?.ScrobbleMethod ?? null)
const InLessons = ref(_Cached?.ScrobbleInLessons ?? true)
const LastfmApiKey = ref(_Cached?.LastfmApiKey ?? "")
const LastfmUsername = ref(_Cached?.LastfmUsername ?? "")
const LastfmSaved = ref(_Cached?.LastfmVerified ?? false)
const LastfmError = ref("")
const LastfmStale = ref("")
const LastfmBusy = ref(false)
const ApiKey = ref(_Cached?.ApiKey ?? "")
const System: Ref<SystemInfo | null> = ref(_Cached?.System ?? null)
const Ready = ref(_Cached !== null)
const Reviewing = ref(false)
const Searching = ref(false)
const VotingRevision = ref(0)
const VotingOn = ref(false)
const ScreenRevision = ref(0)
const VotingPane: Ref<InstanceType<typeof Voting> | null> = ref(null)

let _VotingTimer: ReturnType<typeof setTimeout> | null = null

function ToggleVoting(): void {
  Navigate("voting")
  if (_VotingTimer) clearTimeout(_VotingTimer)
  _VotingTimer = setTimeout(() => {
    VotingPane.value?.Spotlight()
    void VotingPane.value?.Toggle()
  }, 620)
}
const Pending: Ref<number> = ref(0)
const Scrolled: Ref<boolean> = ref(false)

const RootEl: Ref<HTMLElement | null> = ref(null)

const Owner = computed(() => OwnerOf(Current.value)?.Id ?? Sections[0].Id)

let _LockUntil = 0
let _Frame = 0
let _PollTimer: ReturnType<typeof setInterval> | null = null
let _FlashTimer: ReturnType<typeof setTimeout> | null = null

function _RestoreSection(): string {
  try {
    const Saved = localStorage.getItem(_StorageKey) ?? ""
    if (OwnerOf(Saved)) return Saved
    return Sections.find(S => Saved.startsWith(`${S.Id}-`))?.Id ?? Sections[0].Id
  } catch {
    return Sections[0].Id
  }
}

function _RememberSection(Id: string): void {
  try {
    localStorage.setItem(_StorageKey, Id)
  } catch {}
}

function _Scroller(): HTMLElement | null {
  return RootEl.value
}

function _Offset(): number {
  if (Wide.value) return 56
  const Bar = RootEl.value?.querySelector<HTMLElement>(".MobileBar")
  return (Bar?.offsetHeight ?? 0) + 16
}

function _Spy(): void {
  const Box = _Scroller()
  if (!Box) return

  Scrolled.value = Box.scrollTop > 4
  if (performance.now() < _LockUntil) return

  const Marks = Array.from(Box.querySelectorAll<HTMLElement>("[data-section], [data-anchor]"))
    .filter(El => OwnerOf(El.dataset.anchor || El.dataset.section || ""))
  if (!Marks.length) return

  const Top = Box.getBoundingClientRect().top
  const Line = Top + _Offset() + Math.min(160, Box.clientHeight * 0.25)
  const AtEnd = Box.scrollTop + Box.clientHeight >= Box.scrollHeight - 4

  let Found = Marks[0]
  for (const El of Marks) {
    if (El.getBoundingClientRect().top <= Line) Found = El
  }
  if (AtEnd) Found = Marks[Marks.length - 1]

  const Id = Found.dataset.anchor || Found.dataset.section || ""
  if (Id && Id !== Current.value) {
    Current.value = Id
    _RememberSection(Id)
  }
}

function OnScroll(): void {
  if (_Frame) return
  _Frame = requestAnimationFrame(() => {
    _Frame = 0
    _Spy()
  })
}

function _Jump(Id: string, Behavior: ScrollBehavior): void {
  const Box = _Scroller()
  const El = document.getElementById(`s-${Id}`)
  if (!Box || !El) return

  const Delta = El.getBoundingClientRect().top - Box.getBoundingClientRect().top
  Box.scrollTo({ top: Math.max(Box.scrollTop + Delta - _Offset(), 0), behavior: Behavior })
}

function _Flash(Id: string): void {
  const El = document.getElementById(`s-${Id}`)
  if (!El || !El.dataset.anchor) return
  El.classList.remove("Flash")
  void El.offsetWidth
  El.classList.add("Flash")
  if (_FlashTimer) clearTimeout(_FlashTimer)
  _FlashTimer = setTimeout(() => El.classList.remove("Flash"), _FlashMs)
}

function Navigate(Id: string, Highlight = false): void {
  if (!OwnerOf(Id)) return
  Current.value = Id
  _RememberSection(Id)
  _LockUntil = performance.now() + _LockMs
  _Jump(Id, "smooth")
  if (Highlight) _Flash(Id)
}

function OnPick(Hit: Hit): void {
  Searching.value = false
  if (Hit.Kind === "review") {
    Reviewing.value = true
    return
  }
  if (Hit.Kind === "docs") {
    window.open("/llms.txt", "_blank", "noopener")
    return
  }
  Navigate(Hit.Id, Hit.Kind === "anchor")
}

function _Typing(Target: EventTarget | null): boolean {
  const El = Target as HTMLElement | null
  if (!El) return false
  return El.tagName === "INPUT" || El.tagName === "TEXTAREA" || El.tagName === "SELECT" || El.isContentEditable
}

function _OnKey(E: KeyboardEvent): void {
  const Combo = (E.metaKey || E.ctrlKey) && !E.altKey && E.key.toLowerCase() === "k"
  const Slash = E.key === "/" && !E.metaKey && !E.ctrlKey && !E.altKey && !_Typing(E.target)
  if (!Combo && !Slash) return
  if (Reviewing.value) return
  E.preventDefault()
  Searching.value = Combo ? !Searching.value : true
}

function _OnScrollEnd(): void {
  _LockUntil = 0
}

async function _Poll(): Promise<void> {
  if (document.hidden) return
  try {
    const Data = await VotingApi.Load()
    Pending.value = Data.Counts.Pending
  } catch {}
}

function _OnVisibility(): void {
  if (!document.hidden) void _Poll()
}

onMounted(async () => {
  void _Poll()
  _PollTimer = setInterval(_Poll, _PollMs)
  document.addEventListener("visibilitychange", _OnVisibility)
  document.addEventListener("keydown", _OnKey)

  try {
    const Data = await Api.Load()
    Method.value = Data.ScrobbleMethod
    InLessons.value = Data.ScrobbleInLessons ?? true
    LastfmApiKey.value = Data.LastfmApiKey ?? ""
    LastfmUsername.value = Data.LastfmUsername ?? ""
    LastfmSaved.value = Data.LastfmVerified
    LastfmStale.value = Data.LastfmError ?? ""
    ApiKey.value = Data.ApiKey ?? ""
    System.value = Data.System ?? null
    Cache.Write(_CacheKey, Data)
  } catch {}

  Ready.value = true
  await nextTick()
  _LockUntil = performance.now() + 400
  _Jump(Current.value, "auto")
})

onUnmounted(() => {
  if (_PollTimer) clearInterval(_PollTimer)
  if (_FlashTimer) clearTimeout(_FlashTimer)
  if (_VotingTimer) clearTimeout(_VotingTimer)
  cancelAnimationFrame(_Frame)
  document.removeEventListener("visibilitychange", _OnVisibility)
  document.removeEventListener("keydown", _OnKey)
})

watch(VotingRevision, () => {
  void _Poll()
})

watch(Wide, () => {
  void nextTick(() => _Jump(Current.value, "auto"))
})

function _CacheCurrent(): void {
  Cache.Write<Api.Settings>(_CacheKey, {
    ScrobbleMethod: Method.value,
    ScrobbleInLessons: InLessons.value,
    LastfmApiKey: LastfmApiKey.value,
    LastfmUsername: LastfmUsername.value,
    LastfmError: LastfmStale.value || null,
    LastfmVerified: LastfmSaved.value,
    ApiKey: ApiKey.value,
    System: System.value ?? undefined,
  })
}

async function OnMethodChange(M: ScrobbleMethod): Promise<void> {
  Method.value = M
  _CacheCurrent()
  await Api.SetMethod(M)
}

async function OnInLessonsChange(Enabled: boolean): Promise<void> {
  InLessons.value = Enabled
  _CacheCurrent()
  await Api.SetScrobbleInLessons(Enabled)
}

async function OnSaveLastfm(): Promise<void> {
  LastfmBusy.value = true
  LastfmError.value = ""

  try {
    const Result = await Api.SaveLastfm(LastfmApiKey.value, LastfmUsername.value)

    if (!Result.Ok) {
      LastfmError.value = Result.Error || "unknown"
      return
    }

    await Api.SetMethod("lastfm")
    Method.value = "lastfm"
    LastfmSaved.value = true
    LastfmStale.value = ""
    _CacheCurrent()
  } catch {
    LastfmError.value = "network"
  } finally {
    LastfmBusy.value = false
  }
}

async function OnRegenerateKey(): Promise<void> {
  ApiKey.value = await Api.RegenerateKey()
  _CacheCurrent()
}
</script>

<template>
  <div ref="RootEl" class="Settings" @scroll.passive="OnScroll" @scrollend="_OnScrollEnd">
    <div class="Frame">
      <Sidebar
        v-if="Wide"
        class="Side"
        :Current="Current"
        :Owner="Owner"
        :Pending="Pending"
        @Navigate="Navigate"
        @Review="Reviewing = true"
        @Spotlight="Searching = true"
      />

      <MobileBar
        v-else
        :Owner="Owner"
        :Pending="Pending"
        :Scrolled="Scrolled"
        @Navigate="Navigate"
        @Review="Reviewing = true"
        @Spotlight="Searching = true"
      />

      <main class="Content">
        <div v-if="Ready" class="Column">
          <Scrobbling
            :Method="Method"
            :LastfmApiKey="LastfmApiKey"
            :LastfmUsername="LastfmUsername"
            :LastfmSaved="LastfmSaved"
            :LastfmError="LastfmError"
            :LastfmStale="LastfmStale"
            :LastfmBusy="LastfmBusy"
            :ApiKey="ApiKey"
            :InLessons="InLessons"
            @update:Method="OnMethodChange"
            @update:InLessons="OnInLessonsChange"
            @update:LastfmApiKey="LastfmApiKey = $event"
            @update:LastfmUsername="LastfmUsername = $event"
            @SaveLastfm="OnSaveLastfm"
            @RegenerateKey="OnRegenerateKey"
          />
          <Screen :ApiKey="ApiKey" :VotingOn="VotingOn" :Revision="ScreenRevision" @ToggleVoting="ToggleVoting" />
          <Slides @Changed="ScreenRevision++" />
          <Voting ref="VotingPane" :Revision="VotingRevision" @Review="Reviewing = true" @Enabled="VotingOn = $event" />
          <Cloudflare />
          <Plugins />
          <Management />
          <Performance />
          <About :System="System" />
        </div>
      </main>
    </div>

    <Spotlight :Open="Searching" @Close="Searching = false" @Pick="OnPick" />

    <Review :Open="Reviewing" @Close="Reviewing = false" @Changed="VotingRevision++" />
  </div>
</template>

<style scoped>
.Settings {
  position: relative;
  height: 100dvh;
  overflow-y: auto;
  overscroll-behavior-y: none;
  scrollbar-width: thin;
  scrollbar-color: rgba(255, 255, 255, 0.1) transparent;
}

.Column {
  display: flex;
  flex-direction: column;
  gap: 128px;
}

@media (min-width: 1024px) {
  .Frame {
    display: grid;
    width: 100%;
    max-width: 1180px;
    grid-template-columns: 232px minmax(0, 1fr);
    column-gap: clamp(48px, 7vw, 112px);
    margin: 0 auto;
    padding: 0 40px;
  }

  .Side {
    position: sticky;
    top: 0;
    height: 100dvh;
  }

  .Column {
    max-width: 760px;
    padding: 72px 0 40vh;
  }
}

@media (max-width: 1023px) {
  .Settings {
    -webkit-overflow-scrolling: touch;
  }

  .Column {
    gap: 88px;
    padding: 20px 16px calc(40vh + env(safe-area-inset-bottom));
  }
}
</style>
