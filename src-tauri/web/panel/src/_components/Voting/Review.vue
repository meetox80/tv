<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted, onUnmounted, type Ref, type ComputedRef } from "vue"
import { useI18n } from "../../i18n"
import * as Api from "../../_api/voting"
import { Copy } from "../../_api/clipboard"
import type { AdminSong, SongStatus } from "../../_api/voting"
import SwipeCard from "./SwipeCard.vue"
import SongRow from "./SongRow.vue"
import Confirm from "./Confirm.vue"
import Bans from "./Bans.vue"
import PlayButton from "./PlayButton.vue"
import { Line, type RowAction } from "./_format"
import { Close as Silence, Current, Elapsed, Follow, Length, Missing, Prefetch, State as Sound, Toggle } from "./_preview"

const { t } = useI18n()

const Props = defineProps<{ Open: boolean }>()
const Emit = defineEmits<{ Close: []; Changed: [] }>()

interface Decision {
  Song: AdminSong
  Direction: -1 | 1
}

type Pane = SongStatus | "bans"

interface Tab {
  Key: Pane
  Label: string
}

const _PollMs = 15 * 1000
const _TickMs = 30 * 1000
const _StackDepth = 3

const Tabs: Tab[] = [
  { Key: "pending", Label: "voting.counts.pending" },
  { Key: "approved", Label: "voting.counts.approved" },
  { Key: "added", Label: "voting.counts.added" },
  { Key: "rejected", Label: "voting.counts.rejected" },
  { Key: "bans", Label: "voting.bans" },
]

const Active: Ref<Pane> = ref("pending")
const Lists: Ref<Record<SongStatus, AdminSong[]>> = ref({ pending: [], approved: [], added: [], rejected: [] })
const Loaded: Ref<Record<SongStatus, boolean>> = ref({ pending: false, approved: false, added: false, rejected: false })
const Counts: Ref<Api.Counts> = ref({ Pending: 0, Approved: 0, Added: 0, Rejected: 0 })
const Banned: Ref<number> = ref(0)
const BansRevision: Ref<number> = ref(0)
const History: Ref<Decision[]> = ref([])
const EnterFrom: Ref<Record<number, -1 | 1>> = ref({})
const Busy: Ref<number | null> = ref(null)
const Problem: Ref<string> = ref("")
const Tick: Ref<number> = ref(0)
const Toast: Ref<string> = ref("")
const Pull: Ref<number> = ref(0)

const BanTarget: Ref<AdminSong | null> = ref(null)
const DeleteTarget: Ref<AdminSong | null> = ref(null)
const ConfirmBusy: Ref<boolean> = ref(false)

const TopCard: Ref<InstanceType<typeof SwipeCard> | null> = ref(null)

let _PollTimer: ReturnType<typeof setInterval> | null = null
let _TickTimer: ReturnType<typeof setInterval> | null = null
let _ToastTimer: ReturnType<typeof setTimeout> | null = null
let _ErrorTimer: ReturnType<typeof setTimeout> | null = null
let _Flinging = false

const Queue: ComputedRef<AdminSong[]> = computed(() => Lists.value.pending)

const Stack: ComputedRef<AdminSong[]> = computed(() => Queue.value.slice(0, _StackDepth).reverse())

const Top: ComputedRef<AdminSong | null> = computed(() => Queue.value[0] ?? null)

const RejectCharge = computed(() => Math.min(1, Math.max(0, -Pull.value)))
const ApproveCharge = computed(() => Math.min(1, Math.max(0, Pull.value)))

function _Clock(Seconds: number): string {
  const Whole = Math.max(0, Math.floor(Seconds))
  return `${Math.floor(Whole / 60)}:${String(Whole % 60).padStart(2, "0")}`
}

const TopLive = computed(() => !!Top.value && Current.value === Top.value.Id)

const ListenText = computed(() => {
  const Song = Top.value
  if (!Song) return ""
  if (Missing.value.has(Song.Id)) return t("voting.review.preview.none")
  if (!TopLive.value) return t("voting.review.preview.listen")
  if (Sound.value === "loading" && Elapsed.value === 0) return t("voting.review.preview.loading")
  return Length.value > 0 ? `${_Clock(Elapsed.value)} / ${_Clock(Length.value)}` : _Clock(Elapsed.value)
})

const ListenLabel = computed(() =>
  TopLive.value && Sound.value !== "paused" ? t("voting.review.preview.pause") : t("voting.review.preview.play"),
)

const Vacant = computed(() => {
  const Key = Active.value
  if (Key === "bans") return null
  if (Key === "pending") {
    if (!Loaded.value.pending || Queue.value.length) return null
    return { Title: t("voting.review.empty.title"), Body: t("voting.review.empty.body") }
  }
  if (!Loaded.value[Key] || Lists.value[Key].length) return null
  return { Title: t("voting.review.list.empty"), Body: "" }
})

const TabsEl: Ref<HTMLElement | null> = ref(null)
const Thumb: Ref<{ X: number; W: number; Ready: boolean }> = ref({ X: 0, W: 0, Ready: false })

const ThumbStyle = computed(() => ({
  transform: `translateX(${Thumb.value.X}px)`,
  width: `${Thumb.value.W}px`,
  opacity: Thumb.value.Ready ? "1" : "0",
}))

const CountFor = (Key: Pane): number => {
  if (Key === "bans") return Banned.value
  if (Key === "pending") return Counts.value.Pending
  if (Key === "approved") return Counts.value.Approved
  if (Key === "added") return Counts.value.Added
  return Counts.value.Rejected
}

const RowActions: ComputedRef<RowAction[]> = computed(() => {
  if (Active.value === "approved") {
    return [
      { Key: "added", Label: t("voting.review.added"), Tone: "primary" },
      { Key: "pending", Label: t("voting.review.back.pending") },
      { Key: "rejected", Label: t("voting.review.reject"), Tone: "danger" },
    ]
  }
  if (Active.value === "added") {
    return [{ Key: "approved", Label: t("voting.review.back.approved") }]
  }
  if (Active.value === "rejected") {
    return [
      { Key: "pending", Label: t("voting.review.restore") },
      { Key: "delete", Label: t("voting.review.delete"), Tone: "danger" },
    ]
  }
  return []
})

function _ShowError(E: unknown): void {
  Problem.value = t("voting.error.generic")
  if (E instanceof Api.ApiError) {
    const Key = `voting.error.${E.Code}`
    const Text = t(Key)
    if (Text !== Key) Problem.value = Text
  }
  if (_ErrorTimer) clearTimeout(_ErrorTimer)
  _ErrorTimer = setTimeout(() => { Problem.value = "" }, 3200)
}

function _ShowToast(Text: string): void {
  Toast.value = Text
  if (_ToastTimer) clearTimeout(_ToastTimer)
  _ToastTimer = setTimeout(() => { Toast.value = "" }, 1600)
}

async function _LoadCounts(): Promise<void> {
  try {
    const Data = await Api.Load()
    Counts.value = Data.Counts
    Banned.value = Data.Banned
  } catch {}
}

async function _Load(Status: SongStatus): Promise<void> {
  try {
    const Fresh = await Api.LoadSongs(Status)

    if (Status === "pending" && Loaded.value.pending) {
      const Known = new Set(Lists.value.pending.map(S => S.Id))
      const Incoming = new Map(Fresh.map(S => [S.Id, S]))
      const Kept = Lists.value.pending.filter(S => Incoming.has(S.Id)).map(S => Incoming.get(S.Id) as AdminSong)
      const Added = Fresh.filter(S => !Known.has(S.Id))
      Lists.value.pending = [...Kept, ...Added]
    } else {
      Lists.value[Status] = Fresh
    }

    Loaded.value[Status] = true
  } catch (E) {
    _ShowError(E)
  }
}

function _Refresh(): void {
  if (document.hidden || !Props.Open) return
  void _LoadCounts()
  void _Load("pending")
  if (Active.value === "bans") BansRevision.value += 1
  else if (Active.value !== "pending") void _Load(Active.value)
}

function _Adjust(From: SongStatus, To: SongStatus): void {
  const Next = { ...Counts.value }
  const Key = (S: SongStatus): keyof Api.Counts =>
    (S.charAt(0).toUpperCase() + S.slice(1)) as keyof Api.Counts
  Next[Key(From)] = Math.max(0, Next[Key(From)] - 1)
  Next[Key(To)] += 1
  Counts.value = Next
}

function Swipe(Direction: -1 | 1): void {
  if (_Flinging || !Top.value || !TopCard.value) return
  _Flinging = true
  TopCard.value.Fling(Direction)
}

function OnFling(Direction: -1 | 1): void {
  Pull.value = Direction
  Follow(Queue.value[1]?.Id ?? null)
}

function Listen(): void {
  if (Top.value) Toggle(Top.value.Id)
}

async function OnDecide(Song: AdminSong, Direction: -1 | 1): Promise<void> {
  _Flinging = false
  Pull.value = 0
  const Target: SongStatus = Direction > 0 ? "approved" : "rejected"

  Lists.value.pending = Lists.value.pending.filter(S => S.Id !== Song.Id)
  History.value = [...History.value.slice(-19), { Song, Direction }]
  _Adjust("pending", Target)

  try {
    const Updated = await Api.SetStatus(Song.Id, Target)
    if (Loaded.value[Target]) Lists.value[Target] = [Updated, ...Lists.value[Target].filter(S => S.Id !== Song.Id)]
    Emit("Changed")
  } catch (E) {
    History.value = History.value.filter(H => H.Song.Id !== Song.Id)
    EnterFrom.value = { ...EnterFrom.value, [Song.Id]: Direction }
    Lists.value.pending = [Song, ...Lists.value.pending]
    _Adjust(Target, "pending")
    _ShowError(E)
  }
}

async function Undo(): Promise<void> {
  const Last = History.value[History.value.length - 1]
  if (!Last || Busy.value !== null) return

  const From: SongStatus = Last.Direction > 0 ? "approved" : "rejected"
  History.value = History.value.slice(0, -1)
  Busy.value = Last.Song.Id

  if (Active.value !== "pending") Active.value = "pending"

  EnterFrom.value = { ...EnterFrom.value, [Last.Song.Id]: Last.Direction }
  Lists.value.pending = [Last.Song, ...Lists.value.pending.filter(S => S.Id !== Last.Song.Id)]
  Lists.value[From] = Lists.value[From].filter(S => S.Id !== Last.Song.Id)
  _Adjust(From, "pending")

  try {
    await Api.SetStatus(Last.Song.Id, "pending")
    Emit("Changed")
  } catch (E) {
    Lists.value.pending = Lists.value.pending.filter(S => S.Id !== Last.Song.Id)
    History.value = [...History.value, Last]
    _Adjust("pending", From)
    _ShowError(E)
  } finally {
    Busy.value = null
  }
}

async function OnRowAction(Song: AdminSong, Key: string): Promise<void> {
  if (Key === "delete") {
    DeleteTarget.value = Song
    return
  }

  const To = Key as SongStatus
  const From = Song.Status
  Busy.value = Song.Id

  try {
    const Updated = await Api.SetStatus(Song.Id, To)
    Lists.value[From] = Lists.value[From].filter(S => S.Id !== Song.Id)
    if (To === "pending") {
      Lists.value.pending = [...Lists.value.pending.filter(S => S.Id !== Song.Id), Updated]
    } else if (Loaded.value[To]) {
      Lists.value[To] = [Updated, ...Lists.value[To].filter(S => S.Id !== Song.Id)]
    }
    History.value = History.value.filter(H => H.Song.Id !== Song.Id)
    _Adjust(From, To)
    Emit("Changed")
  } catch (E) {
    _ShowError(E)
  } finally {
    Busy.value = null
  }
}

async function ConfirmDelete(): Promise<void> {
  const Song = DeleteTarget.value
  if (!Song) return
  ConfirmBusy.value = true

  try {
    await Api.Remove(Song.Id)
    Lists.value[Song.Status] = Lists.value[Song.Status].filter(S => S.Id !== Song.Id)
    const Next = { ...Counts.value }
    if (Song.Status === "rejected") Next.Rejected = Math.max(0, Next.Rejected - 1)
    Counts.value = Next
    DeleteTarget.value = null
    Emit("Changed")
  } catch (E) {
    _ShowError(E)
  } finally {
    ConfirmBusy.value = false
  }
}

async function ConfirmBan(Reason: string): Promise<void> {
  const Song = BanTarget.value
  if (!Song?.Device) return
  ConfirmBusy.value = true

  try {
    await Api.SetBan(Song.Device, true, Reason)
    const Handle = Song.Device
    const Swept = Lists.value.pending.filter(S => S.Device === Handle && S.Id !== Song.Id)
    Lists.value.pending = Lists.value.pending
      .filter(S => S.Device !== Handle || S.Id === Song.Id)
      .map(S => (S.Id === Song.Id ? { ...S, DeviceBanned: true } : S))
    Counts.value = {
      ...Counts.value,
      Pending: Math.max(0, Counts.value.Pending - Swept.length),
      Rejected: Counts.value.Rejected + Swept.length,
    }
    if (Loaded.value.rejected) void _Load("rejected")
    Banned.value += 1
    BanTarget.value = null
    if (Top.value?.Id === Song.Id) Swipe(-1)
    else void OnDecide(Song, -1)
    Emit("Changed")
  } catch (E) {
    _ShowError(E)
  } finally {
    ConfirmBusy.value = false
  }
}

async function CopyAll(): Promise<void> {
  const Text = Lists.value.approved.map(Line).join("\n")
  if (!Text) return
  await Copy(Text)
  _ShowToast(t("voting.review.copied"))
}

function _Typing(Target: EventTarget | null): boolean {
  return Target instanceof HTMLElement && (Target.tagName === "INPUT" || Target.tagName === "TEXTAREA" || Target.isContentEditable)
}

function _OnKey(Event: KeyboardEvent): void {
  if (!Props.Open || Event.defaultPrevented) return

  if (Event.key === "Escape") {
    if (BanTarget.value || DeleteTarget.value) {
      BanTarget.value = null
      DeleteTarget.value = null
      return
    }
    Emit("Close")
    return
  }

  if (BanTarget.value || DeleteTarget.value || _Typing(Event.target)) return

  const UndoKey = Event.key === "Backspace" || ((Event.ctrlKey || Event.metaKey) && Event.key.toLowerCase() === "z")
  if (UndoKey) {
    Event.preventDefault()
    void Undo()
    return
  }

  if (Active.value !== "pending") return

  if (Event.key === " ") {
    Event.preventDefault()
    Listen()
    return
  }

  if (Event.key === "ArrowRight") {
    Event.preventDefault()
    Swipe(1)
  } else if (Event.key === "ArrowLeft") {
    Event.preventDefault()
    Swipe(-1)
  }
}

function OnBansChanged(Count: number): void {
  if (Banned.value === Count) return
  Banned.value = Count
  Emit("Changed")
}

function Select(Key: Pane): void {
  Active.value = Key
}

function _PlaceThumb(Smooth: boolean): void {
  const Bar = TabsEl.value
  const El = Bar?.querySelector<HTMLElement>(".Tab.On")
  if (!Bar || !El) return
  Thumb.value = { X: El.offsetLeft, W: El.offsetWidth, Ready: true }
  if (Bar.scrollWidth <= Bar.clientWidth) return
  const Left = El.offsetLeft - (Bar.clientWidth - El.offsetWidth) / 2
  Bar.scrollTo({ left: Math.max(0, Left), behavior: Smooth ? "smooth" : "auto" })
}

function _OnResize(): void {
  _PlaceThumb(false)
}

function _Start(): void {
  _Stop()
  _Refresh()
  _PollTimer = setInterval(_Refresh, _PollMs)
  _TickTimer = setInterval(() => { Tick.value += 1 }, _TickMs)
}

function _Stop(): void {
  if (_PollTimer) clearInterval(_PollTimer)
  if (_TickTimer) clearInterval(_TickTimer)
  _PollTimer = null
  _TickTimer = null
}

watch(() => (Props.Open && Active.value === "pending" ? Top.value?.Id ?? null : null), (Id) => {
  Follow(Id)
  if (Id === null) return
  Prefetch(Id)
  const Next = Queue.value[1]
  if (Next) Prefetch(Next.Id)
})

watch(Active, (Key) => {
  if (Key !== "pending") Silence()
  if (Key !== "bans") void _Load(Key)
  void nextTick(() => _PlaceThumb(true))
})

watch([Counts, Banned], () => {
  void nextTick(() => _PlaceThumb(false))
}, { deep: true })

watch(() => Props.Open, (Now) => {
  if (Now) {
    document.documentElement.style.overflow = "hidden"
    _Start()
    void nextTick(() => _PlaceThumb(false))
    void document.fonts?.ready.then(() => _PlaceThumb(false))
  } else {
    document.documentElement.style.overflow = ""
    _Stop()
    Silence()
    Pull.value = 0
    BanTarget.value = null
    DeleteTarget.value = null
  }
}, { immediate: true })

function _OnVisibility(): void {
  if (!document.hidden) _Refresh()
}

onMounted(() => {
  window.addEventListener("resize", _OnResize)
  window.addEventListener("keydown", _OnKey)
  document.addEventListener("visibilitychange", _OnVisibility)
})

onUnmounted(() => {
  _Stop()
  Silence()
  document.documentElement.style.overflow = ""
  window.removeEventListener("resize", _OnResize)
  window.removeEventListener("keydown", _OnKey)
  document.removeEventListener("visibilitychange", _OnVisibility)
  if (_ToastTimer) clearTimeout(_ToastTimer)
  if (_ErrorTimer) clearTimeout(_ErrorTimer)
})
</script>

<template>
  <Teleport to="body">
    <Transition name="review">
      <div v-if="Open" class="Review" role="dialog" aria-modal="true" :aria-label="t('voting.review')">
        <header class="Header">
          <p class="Heading">{{ t("voting.review") }}</p>

          <nav ref="TabsEl" class="Tabs">
            <span class="TabThumb" :style="ThumbStyle" />
            <button
              v-for="T in Tabs"
              :key="T.Key"
              type="button"
              class="Tab"
              :class="{ On: Active === T.Key, Hot: T.Key === 'pending' && CountFor(T.Key) > 0 }"
              @click="Select(T.Key)"
            >
              <span>{{ t(T.Label) }}</span>
              <span class="TabCount">{{ CountFor(T.Key) }}</span>
            </button>
          </nav>

          <button type="button" class="Close" :aria-label="t('voting.review.close')" @click="Emit('Close')">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round">
              <path d="M18 6 6 18M6 6l12 12" />
            </svg>
          </button>
        </header>

        <main class="Stage">
          <Transition name="pane" mode="out-in">
            <section v-if="Active === 'pending'" key="pending" class="Deck">
              <template v-if="Queue.length">
                <div class="Pile">
                  <TransitionGroup name="card">
                    <SwipeCard
                      v-for="(Song, Index) in Stack"
                      :key="Song.Id"
                      :ref="(El) => { if (Song.Id === Top?.Id) TopCard = El as InstanceType<typeof SwipeCard> | null }"
                      :Song="Song"
                      :Depth="Stack.length - 1 - Index"
                      :EnterFrom="Song.Id === Top?.Id ? (EnterFrom[Song.Id] ?? 0) : 0"
                      :Tick="Tick"
                      :Pull="Pull"
                      @Decide="(Direction) => OnDecide(Song, Direction)"
                      @Fling="OnFling"
                      @Pull="(Value) => { Pull = Value }"
                      @Listen="Listen"
                      @Ban="BanTarget = Song"
                    />
                  </TransitionGroup>
                </div>

                <div class="Dock" :style="{ '--reject': RejectCharge, '--approve': ApproveCharge }">
                  <button
                    type="button"
                    class="Act Reject"
                    :class="{ Armed: Pull <= -1 }"
                    :disabled="!Top"
                    @click="Swipe(-1)"
                  >
                    <span class="Disc">
                      <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M7.5 7.5l9 9M16.5 7.5l-9 9" /></svg>
                    </span>
                    <span class="Caption">{{ t("voting.review.reject") }}<kbd>←</kbd></span>
                  </button>

                  <div class="Act Listen">
                    <PlayButton :Id="Top?.Id ?? null" :Label="ListenLabel" />
                    <span class="Caption Time">{{ ListenText }}<kbd>{{ t("voting.review.key.space") }}</kbd></span>
                  </div>

                  <button
                    type="button"
                    class="Act Approve"
                    :class="{ Armed: Pull >= 1 }"
                    :disabled="!Top"
                    @click="Swipe(1)"
                  >
                    <span class="Disc">
                      <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M6 12.6 10.2 16.8 18 8" /></svg>
                    </span>
                    <span class="Caption">{{ t("voting.review.approve") }}<kbd>→</kbd></span>
                  </button>
                </div>

                <p class="Remaining">
                  <span>{{ Queue.length }} {{ t("voting.review.left") }}</span>
                  <button
                    v-if="History.length"
                    type="button"
                    class="UndoLink"
                    :disabled="Busy !== null"
                    :title="`${t('voting.review.undo')} ⌫`"
                    @click="Undo"
                  >
                    {{ t("voting.review.undo") }}
                  </button>
                </p>
              </template>
            </section>

            <section v-else-if="Active === 'bans'" key="bans" class="List">
              <Bans :Tick="Tick" :Revision="BansRevision" @Changed="OnBansChanged" @Error="_ShowError" />
            </section>

            <section v-else :key="Active" class="List">
              <div v-if="Active === 'approved' && Lists.approved.length" class="ListTools">
                <button type="button" class="Btn CopyAll" @click="CopyAll">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
                    <rect x="9" y="9" width="11" height="11" rx="2.5" />
                    <path d="M5 15V6.5A2.5 2.5 0 0 1 7.5 4H15" />
                  </svg>
                  {{ t("voting.review.copy.all") }}
                </button>
              </div>

              <TransitionGroup tag="div" name="row" class="Rows">
                <SongRow
                  v-for="(Song, Index) in Lists[Active]"
                  :key="Song.Id"
                  :Song="Song"
                  :Rank="Active === 'approved' ? Index + 1 : undefined"
                  :Actions="RowActions"
                  :Busy="Busy === Song.Id"
                  :Tick="Tick"
                  @Action="(Key) => OnRowAction(Song, Key)"
                />
              </TransitionGroup>

            </section>
          </Transition>
        </main>

        <Transition name="empty">
          <div v-if="Vacant" :key="Active" class="Void">
            <p class="VoidTitle">{{ Vacant.Title }}</p>
            <p v-if="Vacant.Body" class="VoidBody">{{ Vacant.Body }}</p>
            <button
              v-if="Active === 'pending' && History.length"
              type="button"
              class="Btn VoidUndo"
              :disabled="Busy !== null"
              @click="Undo"
            >
              {{ t("voting.review.undo") }}
            </button>
          </div>
        </Transition>

        <Transition name="toast">
          <div v-if="Problem || Toast" class="Toast" :class="{ Bad: !!Problem }" role="status">{{ Problem || Toast }}</div>
        </Transition>

        <Confirm
          :Open="!!BanTarget"
          :Title="t('voting.review.ban.title')"
          :Body="t('voting.review.ban.body')"
          :ConfirmLabel="t('voting.review.ban.confirm')"
          :Busy="ConfirmBusy"
          WithReason
          @Confirm="ConfirmBan"
          @Cancel="BanTarget = null"
        />

        <Confirm
          :Open="!!DeleteTarget"
          :Title="t('voting.review.delete.title')"
          :Body="t('voting.review.delete.body')"
          :ConfirmLabel="t('voting.review.delete')"
          :Busy="ConfirmBusy"
          @Confirm="ConfirmDelete"
          @Cancel="DeleteTarget = null"
        />
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.Review {
  position: fixed;
  inset: 0;
  z-index: 60;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--bg);
  color: var(--ink);
}

.Header {
  position: relative;
  z-index: 2;
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  grid-template-areas:
    "heading close"
    "tabs tabs";
  align-items: center;
  gap: 6px 12px;
  border-bottom: 1px solid var(--line);
  padding: calc(14px + env(safe-area-inset-top)) 16px 0;
}

.Heading {
  grid-area: heading;
  margin: 0;
  font-family: var(--display);
  font-size: 19px;
  font-weight: 600;
  line-height: 1.1;
  letter-spacing: -0.02em;
}

.Tabs {
  position: relative;
  grid-area: tabs;
  display: flex;
  min-width: 0;
  gap: 4px;
  overflow-x: auto;
  scrollbar-width: none;
  margin: 0 -4px;
}

.Tabs::-webkit-scrollbar {
  display: none;
}

.TabThumb {
  position: absolute;
  bottom: 0;
  left: 0;
  height: 2px;
  border-radius: 2px;
  background: var(--ink);
  transition: transform 380ms var(--ease), width 380ms var(--ease), opacity 200ms ease;
}

.Tab {
  display: flex;
  height: 44px;
  flex-shrink: 0;
  align-items: center;
  gap: 6px;
  padding: 0 10px;
  font-size: 14px;
  font-weight: 500;
  white-space: nowrap;
  color: var(--ink-mute);
  cursor: pointer;
  transition: color 180ms ease;
}

.Tab:hover {
  color: var(--ink-soft);
}

.Tab.On {
  color: var(--ink);
}

.TabCount {
  font-size: 12.5px;
  font-variant-numeric: tabular-nums;
  color: var(--ink-faint);
}

.Tab.Hot .TabCount {
  color: var(--rose);
}

.Close {
  grid-area: close;
  display: flex;
  height: 34px;
  width: 34px;
  align-items: center;
  justify-content: center;
  border-radius: 9px;
  color: var(--ink-mute);
  cursor: pointer;
  transition: background 160ms ease, color 160ms ease;
}

.Close svg {
  height: 18px;
  width: 18px;
}

.Close:hover {
  background: rgba(255, 255, 255, 0.06);
  color: var(--ink);
}

.Stage {
  position: relative;
  z-index: 1;
  display: flex;
  min-height: 0;
  flex: 1;
}

.Deck {
  display: flex;
  min-height: 0;
  flex: 1;
  flex-direction: column;
  align-items: center;
  overflow-y: auto;
  padding: 20px 20px calc(24px + env(safe-area-inset-bottom));
}

.Pile {
  position: relative;
  width: min(100%, 420px);
  margin-top: auto;
}

.Dock {
  --reject: 0;
  --approve: 0;
  display: flex;
  align-items: flex-start;
  justify-content: center;
  gap: 18px;
  margin-top: 22px;
}

.Act {
  display: flex;
  width: 96px;
  flex-direction: column;
  align-items: center;
  gap: 9px;
}

button.Act {
  cursor: pointer;
}

button.Act:disabled {
  opacity: 0.35;
  cursor: default;
}

.Listen {
  gap: 5px;
}

.Reject,
.Approve {
  padding-top: 4px;
}

.Disc {
  position: relative;
  display: flex;
  height: 56px;
  width: 56px;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  border-radius: 999px;
  transition: transform 260ms var(--spring), background 200ms ease, box-shadow 200ms ease, color 200ms ease;
}

.Disc::before {
  content: "";
  position: absolute;
  inset: 0;
  border-radius: inherit;
  transform: scale(0);
  transition: transform 90ms linear;
}

.Disc svg {
  position: relative;
  height: 22px;
  width: 22px;
  fill: none;
  stroke: currentColor;
  stroke-width: 2.2;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.Reject .Disc {
  background: rgba(255, 255, 255, 0.06);
  box-shadow: inset 0 0 0 1px var(--edge);
  color: var(--ink-soft);
}

.Reject .Disc::before {
  background: rgba(239, 138, 138, 0.18);
  transform: scale(var(--reject));
}

.Reject:hover:not(:disabled) .Disc {
  background: rgba(239, 138, 138, 0.1);
  color: var(--danger);
}

.Reject.Armed .Disc {
  box-shadow: inset 0 0 0 1px rgba(239, 138, 138, 0.55);
  color: var(--danger);
  transform: scale(1.08);
}

.Approve .Disc {
  background: var(--ink);
  color: #0c0b0f;
}

.Approve .Disc::before {
  background: var(--good);
  transform: scale(var(--approve));
}

.Approve:hover:not(:disabled) .Disc {
  background: #fff;
}

.Approve.Armed .Disc {
  transform: scale(1.08);
}

button.Act:active:not(:disabled) .Disc {
  transform: scale(0.94);
}

.Caption {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12.5px;
  font-weight: 500;
  white-space: nowrap;
  color: var(--ink-mute);
  transition: color 200ms ease;
}

.Time {
  font-variant-numeric: tabular-nums;
}

.Reject.Armed .Caption {
  color: var(--danger);
}

.Approve.Armed .Caption {
  color: var(--good);
}

kbd {
  display: none;
  height: 17px;
  min-width: 17px;
  align-items: center;
  justify-content: center;
  border-radius: 5px;
  background: rgba(255, 255, 255, 0.05);
  box-shadow: inset 0 0 0 1px var(--edge);
  padding: 0 4px;
  font-family: inherit;
  font-size: 10.5px;
  color: var(--ink-faint);
}

@media (hover: hover) and (pointer: fine) {
  kbd {
    display: inline-flex;
  }
}

.Remaining {
  display: flex;
  align-items: center;
  gap: 14px;
  margin: 14px 0 auto;
  font-size: 13px;
  font-variant-numeric: tabular-nums;
  color: var(--ink-faint);
}

.UndoLink {
  font-size: 13px;
  color: var(--ink-soft);
  cursor: pointer;
  transition: color 160ms ease;
}

.UndoLink:hover:not(:disabled) {
  color: var(--ink);
}

.UndoLink:disabled {
  opacity: 0.4;
  cursor: default;
}

.Void {
  position: absolute;
  inset: 0;
  z-index: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 0 24px;
  text-align: center;
  pointer-events: none;
}

.VoidTitle {
  margin: 0;
  font-family: var(--display);
  font-size: 20px;
  font-weight: 600;
  letter-spacing: -0.02em;
  color: var(--ink);
}

.VoidBody {
  margin: 8px 0 0;
  max-width: 340px;
  font-size: 14px;
  line-height: 1.55;
  color: var(--ink-mute);
  text-wrap: balance;
}

.VoidUndo {
  margin-top: 20px;
  pointer-events: auto;
}

.List {
  display: flex;
  min-height: 0;
  flex: 1;
  flex-direction: column;
  overflow-y: auto;
  padding: 20px 16px calc(40px + env(safe-area-inset-bottom));
  scrollbar-width: thin;
  scrollbar-color: rgba(255, 255, 255, 0.08) transparent;
}

.Rows,
.ListTools {
  width: 100%;
  max-width: 820px;
  margin: 0 auto;
}

.Rows {
  position: relative;
  display: flex;
  flex-direction: column;
}

.ListTools {
  display: flex;
  justify-content: flex-end;
  margin-bottom: 12px;
}

.CopyAll svg {
  height: 15px;
  width: 15px;
}

.Toast {
  position: absolute;
  bottom: calc(28px + env(safe-area-inset-bottom));
  left: 50%;
  z-index: 5;
  border-radius: 10px;
  background: #16151a;
  box-shadow: inset 0 0 0 1px var(--edge);
  padding: 10px 16px;
  font-size: 14px;
  color: var(--ink);
  transform: translateX(-50%);
}

.Toast.Bad {
  color: var(--danger);
}

@media (min-width: 900px) {
  .Header {
    grid-template-columns: minmax(0, 1fr) auto minmax(0, 1fr);
    grid-template-areas: "heading tabs close";
    gap: 24px;
    padding: 0 max(40px, calc((100vw - 1180px) / 2));
  }

  .Tabs {
    margin: 0;
  }

  .Tab {
    height: 60px;
    padding: 0 12px;
  }

  .Close {
    justify-self: end;
  }

  .Deck {
    padding: 32px 40px 48px;
  }

  .Pile {
    width: min(100%, 760px);
  }

  .Dock {
    gap: 28px;
    margin-top: 26px;
  }

  .List {
    padding: 32px 40px 56px;
  }
}

.review-enter-active {
  transition: opacity 240ms ease, transform 320ms var(--ease);
}

.review-leave-active {
  transition: opacity 180ms ease;
}

.review-enter-from,
.review-leave-to {
  opacity: 0;
}

.review-enter-from {
  transform: translateY(8px);
}

.pane-enter-active,
.pane-leave-active {
  transition: opacity 180ms ease, transform 280ms var(--ease);
}

.pane-enter-from {
  opacity: 0;
  transform: translateY(8px);
}

.pane-leave-to {
  opacity: 0;
}

.card-enter-from {
  opacity: 0;
}

.card-enter-active {
  transition: opacity 260ms ease;
}

.card-leave-active {
  display: none;
}

.empty-enter-active {
  transition: opacity 360ms ease 200ms;
}

.empty-leave-active {
  transition: opacity 150ms ease;
}

.empty-enter-from,
.empty-leave-to {
  opacity: 0;
}

.row-move {
  transition: transform 300ms var(--ease);
}

.row-enter-active,
.row-leave-active {
  transition: opacity 200ms ease, transform 300ms var(--ease);
}

.row-leave-active {
  position: absolute;
  right: 0;
  left: 0;
}

.row-enter-from {
  opacity: 0;
  transform: translateY(-6px);
}

.row-leave-to {
  opacity: 0;
  transform: translateX(24px);
}

.toast-enter-active,
.toast-leave-active {
  transition: opacity 180ms ease, transform 260ms var(--ease);
}

.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translate(-50%, 8px);
}

@media (prefers-reduced-motion: reduce) {
  .review-enter-active,
  .pane-enter-active,
  .pane-leave-active,
  .row-move,
  .row-enter-active,
  .row-leave-active,
  .TabThumb {
    transition: opacity 120ms ease;
  }

  .review-enter-from,
  .pane-enter-from,
  .row-enter-from,
  .row-leave-to {
    transform: none;
  }
}
</style>
