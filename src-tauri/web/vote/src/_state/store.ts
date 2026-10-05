import { ref, computed, type Ref, type ComputedRef } from "vue"
import * as Api from "../_api/vote"
import { ApiError } from "../_api/http"
import type { AppState, Ballot, BoardSong, Device as DeviceInfo, MySong } from "../_api/types"

export type Tab = "request" | "board" | "mine"

export const Tabs: { Id: Tab; Label: string }[] = [
  { Id: "board", Label: "Ranking" },
  { Id: "request", Label: "Zaproponuj" },
  { Id: "mine", Label: "Twoje" },
]

export interface CaptchaAsk {
  Reason: "vote" | "request"
  Resolve: () => void
  Reject: (Err: unknown) => void
}

export class Cancelled extends Error {}

const _PollMs = 20 * 1000
const _StateEvery = 3

export const State: Ref<AppState | null> = ref(null)
export const ResetAt: Ref<number | null> = ref(null)
export const StateError: Ref<unknown> = ref(null)
export const Board: Ref<BoardSong[]> = ref([])
export const Mine: Ref<MySong[]> = ref([])
export const SongsReady: Ref<boolean> = ref(false)
export const ActiveTab: Ref<Tab> = ref("board")
export const Highlight: Ref<number | null> = ref(null)
export const CaptchaPending: Ref<CaptchaAsk | null> = ref(null)

export const Device: ComputedRef<DeviceInfo | null> = computed(() => State.value?.Device ?? null)
export const Limit: ComputedRef<number> = computed(() => State.value?.Limit ?? 3)
export const Left: ComputedRef<number> = computed(() => Device.value?.Left ?? Limit.value)
export const Banned: ComputedRef<boolean> = computed(() => Device.value?.Banned ?? false)
export const Open: ComputedRef<boolean> = computed(() => State.value?.Enabled ?? false)

let _PollTimer: ReturnType<typeof setInterval> | null = null
let _Ticks = 0
let _Registering: Promise<void> | null = null

function _SetDevice(Next: DeviceInfo | null): void {
  if (!State.value) return
  State.value = { ...State.value, Device: Next }
}

export function MarkBanned(): void {
  if (Device.value) _SetDevice({ ...Device.value, Banned: true })
}

export function MarkClosed(): void {
  if (State.value) State.value = { ...State.value, Enabled: false }
}

export function SetLeft(Value: number): void {
  if (Device.value) _SetDevice({ ...Device.value, Left: Math.max(0, Value) })
}

export function ForgetDevice(): void {
  _SetDevice(null)
}

export function Absorb(Err: unknown): void {
  if (!(Err instanceof ApiError)) return
  if (Err.Code === "banned") MarkBanned()
  if (Err.Code === "closed") MarkClosed()
  if (Err.Code === "device") ForgetDevice()
  if (Err.Code === "limit") SetLeft(0)
}

export function AnswerCaptcha(Solved: boolean): void {
  const Ask = CaptchaPending.value
  if (!Ask) return
  if (Solved) Ask.Resolve()
  else Ask.Reject(new Cancelled())
}

export async function RefreshState(): Promise<void> {
  try {
    State.value = await Api.LoadState()
    ResetAt.value = Date.now() + State.value.ResetIn * 1000
    StateError.value = null
  } catch (Err) {
    StateError.value = Err
  }
}

export async function RefreshSongs(): Promise<void> {
  try {
    const Data = await Api.LoadSongs()
    Board.value = Data.Board
    Mine.value = Data.Mine
    SongsReady.value = true
  } catch (Err) {
    Absorb(Err)
  }
}

export async function Register(Token: string): Promise<void> {
  const Registered = await Api.RegisterDevice(Token)
  _SetDevice(Registered)
  await RefreshSongs()
}

export function EnsureDevice(Reason: CaptchaAsk["Reason"]): Promise<void> {
  if (Device.value) return Promise.resolve()
  if (_Registering) return _Registering

  _Registering = new Promise<void>((Resolve, Reject) => {
    CaptchaPending.value = { Reason, Resolve, Reject }
  }).finally(() => {
    _Registering = null
    CaptchaPending.value = null
  })

  return _Registering
}

export async function WithDevice<T>(Reason: CaptchaAsk["Reason"], Run: () => Promise<T>): Promise<T> {
  await EnsureDevice(Reason)

  try {
    return await Run()
  } catch (Err) {
    if (Err instanceof ApiError && Err.Code === "device") {
      ForgetDevice()
      await EnsureDevice(Reason)
      return Run()
    }
    throw Err
  }
}

function _Rescore(Song: BoardSong, Vote: Ballot): BoardSong {
  return { ...Song, Score: Song.Score - Song.Vote + Vote, Vote }
}

function _Replace(Id: number, Next: (Song: BoardSong) => BoardSong): void {
  Board.value = Board.value.map(S => (S.Id === Id ? Next(S) : S)).sort((A, B) => B.Score - A.Score)
}

export async function Vote(Id: number, Wanted: Ballot): Promise<void> {
  const Before = Board.value.find(S => S.Id === Id)
  if (!Before) return

  const Value: Ballot = Before.Vote === Wanted ? 0 : Wanted

  await EnsureDevice("vote")
  _Replace(Id, S => _Rescore(S, Value))

  try {
    const Result = await WithDevice("vote", () => Api.CastVote(Id, Value))
    _Replace(Id, S => ({ ...S, Score: Result.Score, Vote: Result.Vote }))
  } catch (Err) {
    _Replace(Id, S => ({ ...S, Score: Before.Score, Vote: Before.Vote }))
    Absorb(Err)
    if (Err instanceof ApiError && Err.Code === "not_found") void RefreshSongs()
    throw Err
  }
}

function _Tick(): void {
  _Ticks += 1
  void RefreshSongs()
  if (_Ticks % _StateEvery === 0) void RefreshState()
}

function _Visibility(): void {
  if (document.hidden) {
    StopPolling()
    return
  }

  void RefreshState()
  void RefreshSongs()
  StartPolling()
}

export function StartPolling(): void {
  if (_PollTimer) return
  _PollTimer = setInterval(_Tick, _PollMs)
}

export function StopPolling(): void {
  if (!_PollTimer) return
  clearInterval(_PollTimer)
  _PollTimer = null
}

export function Watch(): () => void {
  document.addEventListener("visibilitychange", _Visibility)
  if (!document.hidden) StartPolling()

  return () => {
    document.removeEventListener("visibilitychange", _Visibility)
    StopPolling()
  }
}

export function GoVote(Id: number | null): void {
  Highlight.value = Id
  ActiveTab.value = "board"
}
