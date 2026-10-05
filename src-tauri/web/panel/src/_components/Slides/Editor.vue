<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted, onUnmounted, type Ref, type CSSProperties } from "vue"
import * as Api from "../../_api/slides"
import SlideView from "./SlideView.vue"
import Rail from "./Rail.vue"
import TvSkeleton from "./TvSkeleton.vue"
import Icon from "./Icon.vue"
import Layers from "./Layers.vue"
import Inspector, { type AlignTo, type Arrange, type Axis, type Dimension } from "./Inspector.vue"
import Confirm from "../Voting/Confirm.vue"
import TvFrame, { type TvState } from "./TvFrame.vue"
import { ReadWeather } from "@tv/weather"
import * as Screen from "../../_api/screen"
import {
  AssetUrl, BlankSlide, Bounds, Clone, ElementStyle, NewEllipse, NewImage, NewRect, NewText,
  SlideHeight, SlideWidth, Uid, type Slide, type SlideElement,
} from "./model"

const Props = defineProps<{ Id: string }>()

const Emit = defineEmits<{ Close: []; Deleted: [] }>()

type Handle = "n" | "s" | "e" | "w" | "ne" | "nw" | "se" | "sw"
type SaveState = "idle" | "saving" | "saved" | "error"
type SheetName = "slides" | "layers" | "add" | "style" | "text"

interface Box { X: number; Y: number; W: number; H: number }

interface Drag {
  Mode: "move" | "rotate" | Handle
  Ids: string[]
  StartX: number
  StartY: number
  Origins: Record<string, Box>
  Group: Box
  Moved: boolean
  Pointer: number
  Spin: number
  Angle: number
}

const _SaveMs = 700
const _HistoryCap = 100
const _SnapPx = 6
const _Pad = 48
const _NarrowPad = 14
const _ZoomMin = 0.5
const _ZoomMax = 5
const _ZoomStep = 1.25
const _PrimeLimit = 8000
const _BackdropFade = 1150
const _ScreenWidth = 1920
const _ScreenHeight = 1080
const _ClockMs = 15 * 1000
const _PreviewMs = 30 * 1000
const _Cascade = 28
const _NarrowPx = 900
const _ImageMax = { W: 900, H: 560 }
const _AllHandles: Handle[] = ["nw", "n", "ne", "e", "se", "s", "sw", "w"]
const _TextHandles: Handle[] = ["e", "w"]
const _Slide: Box = { X: 0, Y: 0, W: SlideWidth, H: SlideHeight }

const Ready: Ref<boolean> = ref(false)
const Missing: Ref<boolean> = ref(false)
const Narrow: Ref<boolean> = ref(false)
const Name: Ref<string> = ref("")
const Slides: Ref<Slide[]> = ref([])
const Current: Ref<number> = ref(0)
const Selected: Ref<string[]> = ref([])
const Hovered: Ref<string | null> = ref(null)
const Editing: Ref<string | null> = ref(null)
const Saving: Ref<SaveState> = ref("idle")
const Uploads: Ref<number> = ref(0)
const Problem: Ref<string> = ref("")
const Fit: Ref<number> = ref(0.5)
const Zoom: Ref<number> = ref(1)
const Scale = computed(() => Fit.value * Zoom.value)
const Sheet: Ref<SheetName | null> = ref(null)
const MultiMode: Ref<boolean> = ref(false)
const HasClipboard: Ref<boolean> = ref(false)
const Primed: Ref<boolean> = ref(false)
const Measured: Ref<number> = ref(0)
const Guides: Ref<{ X: number[]; Y: number[] }> = ref({ X: [], Y: [] })
const Dropping: Ref<boolean> = ref(false)
const Deleting: Ref<boolean> = ref(false)
const DeleteBusy: Ref<boolean> = ref(false)
const CanUndo: Ref<boolean> = ref(false)
const CanRedo: Ref<boolean> = ref(false)
const Tv: Ref<TvState> = ref({
  Track: null,
  Cover: null,
  Notice: "",
  Pages: [],
  Active: 0,
  Now: new Date(),
  Temperature: null,
  Condition: null,
})

const RootEl: Ref<HTMLElement | null> = ref(null)
const StageEl: Ref<HTMLElement | null> = ref(null)
const CanvasEl: Ref<HTMLElement | null> = ref(null)
const TextEl: Ref<HTMLTextAreaElement | null> = ref(null)
const FileEl: Ref<HTMLInputElement | null> = ref(null)

let _Past: string[] = []
let _Future: string[] = []
let _Before: string | null = null
let _SaveTimer: ReturnType<typeof setTimeout> | null = null
let _Chain: Promise<unknown> = Promise.resolve()
let _Clipboard: SlideElement[] = []
let _PasteSeen = false
let _Observer: ResizeObserver | null = null
let _Drag: Drag | null = null
let _Touches = new Map<number, { X: number; Y: number }>()
let _Pinch: { Distance: number; Zoom: number } | null = null
let _TvFirst: Promise<void> | null = null
let _Replacing: string | null = null
let _DropDepth = 0
let _ClockTimer: ReturnType<typeof setInterval> | null = null
let _PreviewTimer: ReturnType<typeof setInterval> | null = null
let _WeatherKey = ""

const Active = computed<Slide | null>(() => Slides.value[Current.value] ?? null)
const Picked = computed<SlideElement[]>(() => (Active.value?.Elements ?? []).filter(E => Selected.value.includes(E.Id)))
const Chosen = computed<SlideElement | null>(() => (Picked.value.length === 1 ? Picked.value[0] : null))
const Editable = computed<SlideElement | null>(() => Active.value?.Elements.find(E => E.Id === Editing.value) ?? null)

const CanvasStyle = computed<CSSProperties>(() => ({
  width: `${SlideWidth * Scale.value}px`,
  height: `${SlideHeight * Scale.value}px`,
}))

function _BoxStyle(B: Box, Degrees = 0): CSSProperties {
  const S = Scale.value
  return {
    left: `${B.X * S}px`,
    top: `${B.Y * S}px`,
    width: `${B.W * S}px`,
    height: `${B.H * S}px`,
    transform: Degrees ? `rotate(${Degrees}deg)` : undefined,
  }
}

function _Height(E: SlideElement): number {
  if (E.Kind !== "text") return E.H
  const Node = CanvasEl.value?.querySelector<HTMLElement>(`[data-id="${E.Id}"]`)
  return Node?.offsetHeight || E.H
}

function _Box(E: SlideElement): Box {
  return { X: E.X, Y: E.Y, W: E.W, H: _Height(E) }
}

function _Turn(E: SlideElement): number {
  return ((E.Rotation ?? 0) * Math.PI) / 180
}

function _Wrap(Degrees: number): number {
  const Turned = (((Degrees % 360) + 540) % 360) - 180
  return Turned === -180 ? 180 : Turned
}

function _Outline(E: SlideElement): Box {
  const B = _Box(E)
  const Angle = _Turn(E)
  if (!Angle) return B
  const Cos = Math.abs(Math.cos(Angle))
  const Sin = Math.abs(Math.sin(Angle))
  const W = B.W * Cos + B.H * Sin
  const H = B.W * Sin + B.H * Cos
  return { X: B.X + (B.W - W) / 2, Y: B.Y + (B.H - H) / 2, W, H }
}

function _Span(List: SlideElement[]): Box | null {
  if (!List.length) return null
  const Boxes = List.map(_Outline)
  const L = Math.min(...Boxes.map(B => B.X))
  const T = Math.min(...Boxes.map(B => B.Y))
  const R = Math.max(...Boxes.map(B => B.X + B.W))
  const Bottom = Math.max(...Boxes.map(B => B.Y + B.H))
  return { X: L, Y: T, W: R - L, H: Bottom - T }
}

const Frame = computed(() => {
  void Measured.value
  const List = Picked.value.filter(E => !E.Hidden && E.Id !== Editing.value)
  if (!List.length) return null
  const One = List.length === 1 ? List[0] : null
  const Group = One ? _Box(One) : _Span(List)
  if (!Group) return null
  return {
    Style: _BoxStyle(Group, One?.Rotation),
    Handles: One && !One.Locked ? (One.Kind === "text" ? _TextHandles : _AllHandles) : [],
    Spin: !!One && !One.Locked,
    Parts: List.length > 1 ? List.map(E => ({ Id: E.Id, Style: _BoxStyle(_Box(E), E.Rotation) })) : [],
  }
})

const HoverStyle = computed<CSSProperties | null>(() => {
  const Id = Hovered.value
  if (!Id || Selected.value.includes(Id)) return null
  const E = Active.value?.Elements.find(O => O.Id === Id)
  return E && !E.Hidden ? _BoxStyle(_Box(E), E.Rotation) : null
})

const EditStyle = computed<CSSProperties>(() => {
  const E = Editable.value
  if (!E) return {}
  return { ...ElementStyle(E), height: "auto" }
})

const StatusText = computed(() => {
  if (Uploads.value > 0) return "Wysyłanie obrazu…"
  if (Saving.value === "saving") return "Zapisywanie…"
  if (Saving.value === "saved") return "Zapisano"
  if (Saving.value === "error") return "Nie zapisano"
  return ""
})

function _Snapshot(): string {
  return JSON.stringify(Slides.value)
}

function _SyncHistory(): void {
  CanUndo.value = _Past.length > 0
  CanRedo.value = _Future.length > 0
}

function Begin(): void {
  if (_Before === null) _Before = _Snapshot()
}

function End(): void {
  if (_Before === null) return
  const Before = _Before
  _Before = null
  if (Before === _Snapshot()) return
  _Past.push(Before)
  if (_Past.length > _HistoryCap) _Past.shift()
  _Future = []
  _SyncHistory()
  _Queue()
}

function Mutate(Run: () => void): void {
  Begin()
  Run()
  End()
}

function _Restore(Raw: string): void {
  Slides.value = JSON.parse(Raw) as Slide[]
  Current.value = Math.min(Current.value, Slides.value.length - 1)
  Editing.value = null
  const Known = new Set((Active.value?.Elements ?? []).map(E => E.Id))
  Selected.value = Selected.value.filter(Id => Known.has(Id))
}

function Undo(): void {
  StopEditing()
  if (!_Past.length) return
  _Future.push(_Snapshot())
  _Restore(_Past.pop() as string)
  _SyncHistory()
  _Queue()
}

function Redo(): void {
  StopEditing()
  if (!_Future.length) return
  _Past.push(_Snapshot())
  _Restore(_Future.pop() as string)
  _SyncHistory()
  _Queue()
}

function _Queue(): void {
  Saving.value = "saving"
  if (_SaveTimer) clearTimeout(_SaveTimer)
  _SaveTimer = setTimeout(() => void Flush(), _SaveMs)
}

function Flush(): Promise<boolean> {
  if (_SaveTimer) {
    clearTimeout(_SaveTimer)
    _SaveTimer = null
  }

  const Title = Name.value.trim() || "Bez nazwy"
  const Payload = Clone(Slides.value)
  Saving.value = "saving"

  const Run = _Chain.then(async () => {
    try {
      await Api.Save(Props.Id, Title, Payload)
      return true
    } catch (Caught) {
      Problem.value = Caught instanceof Error ? `Nie udało się zapisać: ${Caught.message}` : "Nie udało się zapisać."
      return false
    }
  })
  _Chain = Run

  return Run.then(Ok => {
    if (_Chain === Run) Saving.value = Ok ? "saved" : "error"
    if (Ok) Problem.value = ""
    return Ok
  })
}

function _Measure(): void {
  const E = Chosen.value
  if (!E || E.Kind !== "text") {
    Measured.value = 0
    return
  }
  const H = _Height(E)
  Measured.value = H
  if (Math.abs(E.H - H) > 0.5) E.H = Math.round(H)
}

function _Fit(): void {
  const El = StageEl.value
  if (!El) return
  const Pad = Narrow.value ? _NarrowPad : _Pad
  const W = El.clientWidth - Pad * 2
  const H = El.clientHeight - Pad * 2
  Fit.value = Math.max(0.05, Math.min(W / _ScreenWidth, H / _ScreenHeight))
}

const SpaceStyle = computed<CSSProperties>(() => ({ padding: `${Narrow.value ? _NarrowPad : _Pad}px` }))

const ScreenStyle = computed<CSSProperties>(() => ({
  width: `${_ScreenWidth * Scale.value}px`,
  height: `${_ScreenHeight * Scale.value}px`,
}))

const ZoomText = computed(() => `${Math.round(Zoom.value * 100)}%`)

function _SetZoom(Next: number): void {
  Zoom.value = Math.min(_ZoomMax, Math.max(_ZoomMin, Math.round(Next * 100) / 100))
}

function ZoomIn(): void {
  _SetZoom(Zoom.value * _ZoomStep)
}

function ZoomOut(): void {
  _SetZoom(Zoom.value / _ZoomStep)
}

function ZoomFit(): void {
  Zoom.value = 1
}

function OnWheel(Event: WheelEvent): void {
  if (!Event.ctrlKey && !Event.metaKey) return
  Event.preventDefault()
  _SetZoom(Zoom.value * Math.exp(-Event.deltaY * 0.0022))
}

function _Distance(): number {
  const [A, B] = [..._Touches.values()]
  return A && B ? Math.hypot(A.X - B.X, A.Y - B.Y) : 0
}

function OnTouchDown(Event: PointerEvent): void {
  if (Event.pointerType !== "touch") return
  _Touches.set(Event.pointerId, { X: Event.clientX, Y: Event.clientY })
  if (_Touches.size !== 2) return
  if (_Drag) {
    const Moved = _Drag.Moved
    _Drag = null
    Guides.value = { X: [], Y: [] }
    window.removeEventListener("pointermove", _OnDragMove)
    window.removeEventListener("pointerup", _OnDragEnd)
    window.removeEventListener("pointercancel", _OnDragEnd)
    if (Moved) End()
  }
  _Pinch = { Distance: _Distance(), Zoom: Zoom.value }
  window.addEventListener("pointermove", _OnTouchMove)
  window.addEventListener("pointerup", _OnTouchUp)
  window.addEventListener("pointercancel", _OnTouchUp)
}

function _OnTouchMove(Event: PointerEvent): void {
  if (!_Touches.has(Event.pointerId)) return
  _Touches.set(Event.pointerId, { X: Event.clientX, Y: Event.clientY })
  if (!_Pinch || _Pinch.Distance <= 0) return
  _SetZoom(_Pinch.Zoom * (_Distance() / _Pinch.Distance))
}

function _OnTouchUp(Event: PointerEvent): void {
  _Touches.delete(Event.pointerId)
  if (_Touches.size >= 2) return
  _Pinch = null
  if (_Touches.size) return
  window.removeEventListener("pointermove", _OnTouchMove)
  window.removeEventListener("pointerup", _OnTouchUp)
  window.removeEventListener("pointercancel", _OnTouchUp)
}

function _Wait(Ms: number): Promise<void> {
  return new Promise(Resolve => setTimeout(Resolve, Ms))
}

async function _Decode(Url: string): Promise<void> {
  const Img = new Image()
  Img.src = Url
  try {
    await Img.decode()
  } catch {}
}

async function _Prime(): Promise<void> {
  const Work = (async () => {
    await (_TvFirst ?? Promise.resolve())
    const Urls: string[] = []
    if (Tv.value.Cover) Urls.push(Tv.value.Cover)
    for (const E of Active.value?.Elements ?? []) {
      if (E.Kind === "image" && E.Asset) Urls.push(AssetUrl(E.Asset))
    }
    await Promise.all([document.fonts?.ready, ...Urls.map(_Decode)])
    if (Tv.value.Cover) await _Wait(_BackdropFade)
  })()
  await Promise.race([Work, _Wait(_PrimeLimit)])
  Primed.value = true
}

function OpenSheet(Name: SheetName): void {
  if (Sheet.value === "text") End()
  Sheet.value = Sheet.value === Name ? null : Name
}

function CloseSheet(): void {
  if (Sheet.value === "text") End()
  Sheet.value = null
}

const SheetTitle = computed(() => {
  if (Sheet.value === "slides") return "Slajdy"
  if (Sheet.value === "layers") return "Warstwy"
  if (Sheet.value === "add") return "Dodaj"
  if (Sheet.value === "text") return "Tekst"
  return "Styl"
})

function EditText(): void {
  const E = Chosen.value
  if (!E || E.Kind !== "text" || E.Locked) return
  if (Narrow.value) {
    Begin()
    Sheet.value = "text"
    return
  }
  StartEditing(E.Id, false)
}

function OnSheetText(Event: Event): void {
  const E = Chosen.value
  if (!E || E.Kind !== "text") return
  Begin()
  E.Text = (Event.target as HTMLTextAreaElement).value
}

function AddFromSheet(Kind: "text" | "rect" | "ellipse" | "image" | "paste"): void {
  Sheet.value = null
  if (Kind === "text") AddText()
  else if (Kind === "rect") AddRect()
  else if (Kind === "ellipse") AddEllipse()
  else if (Kind === "image") PickImage(null)
  else _PasteInternal()
}

function OnSlidePick(Index: number): void {
  SelectSlide(Index)
  if (Narrow.value) Sheet.value = null
}

function OnSlideMove(From: number, To: number): void {
  Mutate(() => {
    const [Moved] = Slides.value.splice(From, 1)
    Slides.value.splice(To, 0, Moved)
  })
  SelectSlide(To)
}

function _Point(ClientX: number, ClientY: number): { X: number; Y: number } | null {
  const Rect = CanvasEl.value?.getBoundingClientRect()
  if (!Rect) return null
  return { X: (ClientX - Rect.left) / Scale.value, Y: (ClientY - Rect.top) / Scale.value }
}

function _Release(): void {
  const Focused = document.activeElement
  if (Focused instanceof HTMLElement && Focused !== TextEl.value && RootEl.value?.contains(Focused)) Focused.blur()
}

function Select(Ids: string[]): void {
  Selected.value = Ids
}

function _Place(E: SlideElement): SlideElement {
  const List = Active.value?.Elements ?? []
  while (List.some(O => Math.abs(O.X - E.X) < 2 && Math.abs(O.Y - E.Y) < 2)) {
    E.X += _Cascade
    E.Y += _Cascade
  }
  return E
}

function _Insert(List: SlideElement[]): void {
  const Target = Active.value
  if (!Target || !List.length) return
  Mutate(() => Target.Elements.push(...List))
  Select(List.map(E => E.Id))
}

function AddText(): void {
  const E = _Place(NewText((SlideWidth - 720) / 2, (SlideHeight - 80) / 2))
  _Insert([E])
  if (Narrow.value) {
    Begin()
    Sheet.value = "text"
    return
  }
  void nextTick(() => StartEditing(E.Id, true))
}

function AddRect(): void {
  _Insert([_Place(NewRect((SlideWidth - 420) / 2, (SlideHeight - 260) / 2))])
}

function AddEllipse(): void {
  _Insert([_Place(NewEllipse((SlideWidth - 280) / 2, (SlideHeight - 280) / 2))])
}

function PickImage(Replace: string | null = null): void {
  _Replacing = Replace
  if (!FileEl.value) return
  FileEl.value.value = ""
  FileEl.value.click()
}

async function _Natural(File: Blob): Promise<{ W: number; H: number }> {
  const Bitmap = await createImageBitmap(File)
  const Size = { W: Bitmap.width, H: Bitmap.height }
  Bitmap.close()
  return Size
}

async function _AddImage(File: File, At: { X: number; Y: number } | null): Promise<void> {
  if (!File.type.startsWith("image/")) return
  const Target = Active.value
  if (!Target) return
  Uploads.value += 1
  Problem.value = ""

  try {
    const [Asset, Size] = await Promise.all([Api.Upload(File), _Natural(File)])
    const Ratio = Math.min(1, _ImageMax.W / Size.W, _ImageMax.H / Size.H)
    const W = Math.max(8, Math.round(Size.W * Ratio))
    const H = Math.max(8, Math.round(Size.H * Ratio))

    if (_Replacing) {
      const Id = _Replacing
      _Replacing = null
      const E = Target.Elements.find(O => O.Id === Id)
      if (E) Mutate(() => { E.Asset = Asset })
      return
    }

    const X = At ? At.X - W / 2 : (SlideWidth - W) / 2
    const Y = At ? At.Y - H / 2 : (SlideHeight - H) / 2
    _Insert([_Place(NewImage(Math.round(X), Math.round(Y), W, H, Asset))])
  } catch (Caught) {
    Problem.value = Caught instanceof Error ? `Nie udało się dodać obrazu: ${Caught.message}` : "Nie udało się dodać obrazu."
  } finally {
    Uploads.value -= 1
  }
}

function OnFile(): void {
  const Files = Array.from(FileEl.value?.files ?? [])
  for (const File of Files) void _AddImage(File, null)
}

function _Targets(Skip: string[]): { Xs: number[]; Ys: number[] } {
  const Xs = [0, SlideWidth / 2, SlideWidth]
  const Ys = [0, SlideHeight / 2, SlideHeight]
  for (const O of Active.value?.Elements ?? []) {
    if (Skip.includes(O.Id) || O.Hidden) continue
    const B = _Outline(O)
    Xs.push(B.X, B.X + B.W / 2, B.X + B.W)
    Ys.push(B.Y, B.Y + B.H / 2, B.Y + B.H)
  }
  return { Xs, Ys }
}

function _Nearest(Lines: number[], Targets: number[], Limit: number): { Offset: number; At: number } | null {
  let Best: { Offset: number; At: number } | null = null
  for (const Line of Lines) {
    for (const Target of Targets) {
      const Offset = Target - Line
      if (Math.abs(Offset) > Limit) continue
      if (!Best || Math.abs(Offset) < Math.abs(Best.Offset)) Best = { Offset, At: Target }
    }
  }
  return Best
}

function OnCanvasDown(Event: PointerEvent): void {
  if (Event.button !== 0 || _Pinch) return
  const Target = Event.target as HTMLElement
  if (Target.closest(".TextEdit")) return
  if (Editing.value) StopEditing()
  _Release()

  const Hit = Target.closest<HTMLElement>("[data-id]")
  const E = Hit ? Active.value?.Elements.find(O => O.Id === Hit.dataset.id) : null
  const Additive = Event.shiftKey || MultiMode.value
  if (!E) {
    if (!Additive) Select([])
    return
  }

  if (Additive) {
    if (Selected.value.includes(E.Id)) {
      Select(Selected.value.filter(Id => Id !== E.Id))
      return
    }
    Select([...Selected.value, E.Id])
  } else if (!Selected.value.includes(E.Id)) {
    Select([E.Id])
  }

  const Movable = Picked.value.filter(O => !O.Locked)
  if (Movable.length) _Begin(Event, "move", Movable)
}

function OnHandleDown(Event: PointerEvent, Mode: Handle): void {
  if (Event.button !== 0 || !Chosen.value) return
  Event.stopPropagation()
  _Release()
  _Begin(Event, Mode, [Chosen.value])
}

function OnSpinDown(Event: PointerEvent): void {
  if (Event.button !== 0 || !Chosen.value) return
  Event.stopPropagation()
  _Release()
  _Begin(Event, "rotate", [Chosen.value])
}

function _Begin(Event: PointerEvent, Mode: Drag["Mode"], List: SlideElement[]): void {
  Event.preventDefault()
  const Origins: Record<string, Box> = {}
  for (const E of List) Origins[E.Id] = _Box(E)
  const First = Origins[List[0].Id]
  const At = _Point(Event.clientX, Event.clientY)
  _Drag = {
    Mode,
    Ids: List.map(E => E.Id),
    StartX: Event.clientX,
    StartY: Event.clientY,
    Origins,
    Group: _Span(List) ?? First,
    Moved: false,
    Pointer: Event.pointerId,
    Spin: List[0].Rotation ?? 0,
    Angle: At ? Math.atan2(At.Y - (First.Y + First.H / 2), At.X - (First.X + First.W / 2)) : 0,
  }
  window.addEventListener("pointermove", _OnDragMove)
  window.addEventListener("pointerup", _OnDragEnd)
  window.addEventListener("pointercancel", _OnDragEnd)
}

function _OnDragMove(Event: PointerEvent): void {
  const D = _Drag
  if (!D || Event.pointerId !== D.Pointer) return
  const S = Scale.value
  const Dx = (Event.clientX - D.StartX) / S
  const Dy = (Event.clientY - D.StartY) / S
  if (!D.Moved && Math.hypot(Dx * S, Dy * S) < 3) return
  if (!D.Moved) {
    D.Moved = true
    Begin()
  }

  const Elements = Active.value?.Elements ?? []

  if (D.Mode === "rotate") {
    const E = Elements.find(O => O.Id === D.Ids[0])
    const O = D.Origins[D.Ids[0]]
    const At = _Point(Event.clientX, Event.clientY)
    if (!E || !O || !At) return
    const Swept = Math.atan2(At.Y - (O.Y + O.H / 2), At.X - (O.X + O.W / 2)) - D.Angle
    const Step = Event.shiftKey ? 15 : 1
    E.Rotation = _Wrap(Math.round((D.Spin + (Swept * 180) / Math.PI) / Step) * Step)
    return
  }

  const Limit = _SnapPx / S
  const Free = Event.altKey
  const { Xs, Ys } = Free ? { Xs: [], Ys: [] } : _Targets(D.Ids)

  if (D.Mode === "move") {
    const G = D.Group
    let OffX = Dx
    let OffY = Dy
    const SnapX = _Nearest([G.X + Dx, G.X + Dx + G.W / 2, G.X + Dx + G.W], Xs, Limit)
    const SnapY = _Nearest([G.Y + Dy, G.Y + Dy + G.H / 2, G.Y + Dy + G.H], Ys, Limit)
    if (SnapX) OffX += SnapX.Offset
    if (SnapY) OffY += SnapY.Offset
    for (const Id of D.Ids) {
      const E = Elements.find(O => O.Id === Id)
      const O = D.Origins[Id]
      if (!E || !O) continue
      E.X = Math.round(O.X + OffX)
      E.Y = Math.round(O.Y + OffY)
    }
    Guides.value = { X: SnapX ? [SnapX.At] : [], Y: SnapY ? [SnapY.At] : [] }
    return
  }

  const E = Elements.find(O => O.Id === D.Ids[0])
  const O = D.Origins[D.Ids[0]]
  if (!E || !O) return
  const Mode = D.Mode
  const Angle = _Turn(E)
  const Cos = Math.cos(Angle)
  const Sin = Math.sin(Angle)
  const Lx = Dx * Cos + Dy * Sin
  const Ly = Dy * Cos - Dx * Sin
  let L = O.X
  let T = O.Y
  let R = O.X + O.W
  let B = O.Y + O.H
  if (Mode.includes("w")) L += Lx
  if (Mode.includes("e")) R += Lx
  if (Mode.includes("n")) T += Ly
  if (Mode.includes("s")) B += Ly

  const GuideX: number[] = []
  const GuideY: number[] = []
  const Edge = (Value: number, Targets: number[], Into: number[]): number => {
    const Hit = _Nearest([Value], Targets, Limit)
    if (!Hit) return Value
    Into.push(Hit.At)
    return Value + Hit.Offset
  }
  if (!Angle) {
    if (Mode.includes("w")) L = Edge(L, Xs, GuideX)
    if (Mode.includes("e")) R = Edge(R, Xs, GuideX)
    if (Mode.includes("n")) T = Edge(T, Ys, GuideY)
    if (Mode.includes("s")) B = Edge(B, Ys, GuideY)
  }

  const Corner = Mode.length === 2
  const Keep = Corner && (E.Kind === "image" ? !Event.shiftKey : Event.shiftKey)
  if (Keep && O.H > 0) {
    const Ratio = O.W / O.H
    let W = R - L
    let H = B - T
    if (Math.abs(W / Ratio) > Math.abs(H)) H = W / Ratio
    else W = H * Ratio
    if (Mode.includes("w")) L = R - W
    else R = L + W
    if (Mode.includes("n")) T = B - H
    else B = T + H
    GuideX.length = 0
    GuideY.length = 0
  }

  const Min = 8
  if (R - L < Min) {
    if (Mode.includes("w")) L = R - Min
    else R = L + Min
  }
  if (B - T < Min) {
    if (Mode.includes("n")) T = B - Min
    else B = T + Min
  }

  const Wide = R - L
  const Tall = B - T
  const Shift = { X: (L + R) / 2 - (O.X + O.W / 2), Y: (T + B) / 2 - (O.Y + O.H / 2) }
  const Left = O.X + O.W / 2 + Shift.X * Cos - Shift.Y * Sin - Wide / 2
  const Top = O.Y + O.H / 2 + Shift.X * Sin + Shift.Y * Cos - Tall / 2
  E.X = Math.round(Left)
  E.W = Math.round(Wide)
  if (E.Kind !== "text") {
    E.Y = Math.round(Top)
    E.H = Math.round(Tall)
  } else if (Angle) {
    E.Y = Math.round(Top)
  }
  Guides.value = { X: GuideX, Y: GuideY }
}

function _OnDragEnd(Event: PointerEvent): void {
  const D = _Drag
  if (!D || Event.pointerId !== D.Pointer) return
  _Drag = null
  Guides.value = { X: [], Y: [] }
  window.removeEventListener("pointermove", _OnDragMove)
  window.removeEventListener("pointerup", _OnDragEnd)
  window.removeEventListener("pointercancel", _OnDragEnd)
  if (D.Moved) End()
}

function OnCanvasDouble(Event: MouseEvent): void {
  const Hit = (Event.target as HTMLElement).closest<HTMLElement>("[data-id]")
  const E = Hit ? Active.value?.Elements.find(O => O.Id === Hit.dataset.id) : null
  if (!E || E.Locked) return
  if (E.Kind === "text") StartEditing(E.Id, false)
  else if (E.Kind === "image") PickImage(E.Id)
}

function _Grow(): void {
  const Area = TextEl.value
  if (!Area) return
  Area.style.height = "0px"
  Area.style.height = `${Area.scrollHeight}px`
}

function StartEditing(Id: string, SelectAll: boolean): void {
  Select([Id])
  Editing.value = Id
  Begin()
  void nextTick(() => {
    const Area = TextEl.value
    if (!Area) return
    _Grow()
    Area.focus({ preventScroll: true })
    if (SelectAll) Area.select()
    else Area.setSelectionRange(Area.value.length, Area.value.length)
  })
}

function OnTextInput(Event: Event): void {
  const E = Editable.value
  if (!E) return
  E.Text = (Event.target as HTMLTextAreaElement).value
  _Grow()
}

function StopEditing(): void {
  if (!Editing.value) return
  Editing.value = null
  End()
}

function OnPatch(Patch: Partial<SlideElement>, Final: boolean): void {
  const List = Picked.value
  if (!List.length) return
  Begin()
  for (const E of List) Object.assign(E, Patch)
  if (Final) End()
}

function OnMove(Key: Dimension, Value: number, Final: boolean): void {
  const E = Chosen.value
  if (!E) return
  Begin()
  E[Key] = Value
  if (Final) End()
}

function OnBackground(Value: string | null): void {
  const Target = Active.value
  if (!Target) return
  Mutate(() => {
    if (Value) Target.Background = Value
    else delete Target.Background
  })
}

function OnArrange(How: Arrange): void {
  const Target = Active.value
  if (!Target || !Selected.value.length) return
  const Marked = new Set(Selected.value)
  Mutate(() => {
    const List = [...Target.Elements]
    if (How === "front") {
      Target.Elements = [...List.filter(E => !Marked.has(E.Id)), ...List.filter(E => Marked.has(E.Id))]
      return
    }
    if (How === "back") {
      Target.Elements = [...List.filter(E => Marked.has(E.Id)), ...List.filter(E => !Marked.has(E.Id))]
      return
    }
    if (How === "forward") {
      for (let I = List.length - 2; I >= 0; I--) {
        if (Marked.has(List[I].Id) && !Marked.has(List[I + 1].Id)) [List[I], List[I + 1]] = [List[I + 1], List[I]]
      }
    } else {
      for (let I = 1; I < List.length; I++) {
        if (Marked.has(List[I].Id) && !Marked.has(List[I - 1].Id)) [List[I], List[I - 1]] = [List[I - 1], List[I]]
      }
    }
    Target.Elements = List
  })
}

function OnAlign(How: AlignTo): void {
  const List = Picked.value.filter(E => !E.Locked)
  if (!List.length) return
  const Frame = List.length > 1 ? Bounds(List, _Height) ?? _Slide : _Slide
  Mutate(() => {
    for (const E of List) {
      const H = _Height(E)
      if (How === "left") E.X = Math.round(Frame.X)
      if (How === "center") E.X = Math.round(Frame.X + (Frame.W - E.W) / 2)
      if (How === "right") E.X = Math.round(Frame.X + Frame.W - E.W)
      if (How === "top") E.Y = Math.round(Frame.Y)
      if (How === "middle") E.Y = Math.round(Frame.Y + (Frame.H - H) / 2)
      if (How === "bottom") E.Y = Math.round(Frame.Y + Frame.H - H)
    }
  })
}

function OnDistribute(Along: Axis): void {
  const List = Picked.value.filter(E => !E.Locked)
  if (List.length < 3) return
  const Horizontal = Along === "horizontal"
  const Sorted = [...List].sort((A, B) => (Horizontal ? A.X - B.X : A.Y - B.Y))
  const Size = (E: SlideElement): number => (Horizontal ? E.W : _Height(E))
  const Start = Horizontal ? Sorted[0].X : Sorted[0].Y
  const Last = Sorted[Sorted.length - 1]
  const Finish = (Horizontal ? Last.X : Last.Y) + Size(Last)
  const Gap = (Finish - Start - Sorted.reduce((Sum, E) => Sum + Size(E), 0)) / (Sorted.length - 1)
  Mutate(() => {
    let At = Start
    for (const E of Sorted) {
      if (Horizontal) E.X = Math.round(At)
      else E.Y = Math.round(At)
      At += Size(E) + Gap
    }
  })
}

function _Copies(List: SlideElement[], Offset: number): SlideElement[] {
  return List.map(E => ({ ...Clone(E), Id: Uid(), X: E.X + Offset, Y: E.Y + Offset }))
}

function DuplicateElement(): void {
  if (!Picked.value.length) return
  _Insert(_Copies(Picked.value, _Cascade))
}

function RemoveElement(): void {
  const Target = Active.value
  if (!Target || !Selected.value.length) return
  StopEditing()
  const Gone = new Set(Selected.value)
  Mutate(() => {
    Target.Elements = Target.Elements.filter(E => !Gone.has(E.Id))
  })
  Select([])
}

function _Nudge(Dx: number, Dy: number): void {
  const List = Picked.value.filter(E => !E.Locked)
  if (!List.length) return
  Mutate(() => {
    for (const E of List) {
      E.X += Dx
      E.Y += Dy
    }
  })
}

function _Copy(): void {
  if (!Picked.value.length) return
  _Clipboard = Clone(Picked.value)
  HasClipboard.value = true
}

function CopySelection(): void {
  _Copy()
}

function PasteClipboard(): void {
  _PasteInternal()
}

function _PasteInternal(): void {
  if (!_Clipboard.length) return
  const Copies = _Copies(_Clipboard, _Cascade)
  _Clipboard = Clone(Copies)
  _Insert(Copies)
}

function OnLayerSelect(Ids: string[]): void {
  StopEditing()
  _Release()
  Select(Ids)
}

function OnLayerToggle(Id: string, Key: "Hidden" | "Locked"): void {
  const E = Active.value?.Elements.find(O => O.Id === Id)
  if (!E) return
  Mutate(() => {
    if (E[Key]) delete E[Key]
    else E[Key] = true
  })
}

function OnLayerRename(Id: string, Value: string): void {
  const E = Active.value?.Elements.find(O => O.Id === Id)
  if (!E) return
  Mutate(() => {
    if (Value) E.Name = Value
    else delete E.Name
  })
}

function OnLayerReorder(From: number, To: number): void {
  const Target = Active.value
  if (!Target) return
  Mutate(() => {
    const List = [...Target.Elements]
    const [Moved] = List.splice(From, 1)
    List.splice(To, 0, Moved)
    Target.Elements = List
  })
}

function SelectSlide(Index: number): void {
  StopEditing()
  Current.value = Index
  Select([])
  Hovered.value = null
}

function AddSlide(): void {
  StopEditing()
  const At = Current.value + 1
  Mutate(() => Slides.value.splice(At, 0, BlankSlide()))
  SelectSlide(At)
}

function DuplicateSlide(Index: number): void {
  const Source = Slides.value[Index]
  if (!Source) return
  const Copy: Slide = { ...Clone(Source), Id: Uid() }
  Copy.Elements = Copy.Elements.map(E => ({ ...E, Id: Uid() }))
  Mutate(() => Slides.value.splice(Index + 1, 0, Copy))
  SelectSlide(Index + 1)
}

function RemoveSlide(Index: number): void {
  if (Slides.value.length <= 1) return
  StopEditing()
  Mutate(() => Slides.value.splice(Index, 1))
  Current.value = Math.min(Current.value, Slides.value.length - 1)
  Select([])
}

function _HasFiles(Event: DragEvent): boolean {
  return Array.from(Event.dataTransfer?.types ?? []).includes("Files")
}

function Swallow(Event: DragEvent): void {
  if (_HasFiles(Event)) Event.preventDefault()
}

function OnDragEnter(Event: DragEvent): void {
  if (!_HasFiles(Event)) return
  Event.preventDefault()
  _DropDepth += 1
  Dropping.value = true
}

function OnDragOver(Event: DragEvent): void {
  if (!_HasFiles(Event)) return
  Event.preventDefault()
  if (Event.dataTransfer) Event.dataTransfer.dropEffect = "copy"
}

function OnDragLeave(Event: DragEvent): void {
  if (!_HasFiles(Event)) return
  _DropDepth = Math.max(0, _DropDepth - 1)
  if (!_DropDepth) Dropping.value = false
}

function OnDrop(Event: DragEvent): void {
  if (!_HasFiles(Event)) return
  Event.preventDefault()
  _DropDepth = 0
  Dropping.value = false
  const Point = _Point(Event.clientX, Event.clientY)
  const Inside = Point && Point.X >= 0 && Point.Y >= 0 && Point.X <= SlideWidth && Point.Y <= SlideHeight
  _Replacing = null
  for (const File of Array.from(Event.dataTransfer?.files ?? [])) void _AddImage(File, Inside ? Point : null)
}

function _Typing(Target: EventTarget | null): boolean {
  return Target instanceof HTMLElement && (Target.tagName === "INPUT" || Target.tagName === "TEXTAREA" || Target.tagName === "SELECT" || Target.isContentEditable)
}

function _OnKey(Event: KeyboardEvent): void {
  if (Deleting.value) return
  if (!RootEl.value?.contains(Event.target as Node)) Event.stopPropagation()

  const Mod = Event.ctrlKey || Event.metaKey
  const Key = Event.key.toLowerCase()

  if (Editing.value) {
    if (Event.key === "Escape" || (Mod && Event.key === "Enter")) {
      Event.preventDefault()
      StopEditing()
    }
    return
  }

  if (_Typing(Event.target)) return

  if (Mod && Key === "z") {
    Event.preventDefault()
    if (Event.shiftKey) Redo()
    else Undo()
    return
  }
  if (Mod && Key === "y") {
    Event.preventDefault()
    Redo()
    return
  }
  if (Mod && Key === "d") {
    Event.preventDefault()
    DuplicateElement()
    return
  }
  if (Mod && Key === "a") {
    Event.preventDefault()
    Select((Active.value?.Elements ?? []).filter(E => !E.Hidden && !E.Locked).map(E => E.Id))
    return
  }
  if (Mod && Key === "c") {
    _Copy()
    return
  }
  if (Mod && Key === "x") {
    _Copy()
    RemoveElement()
    return
  }
  if (Mod && Key === "v") {
    _PasteSeen = false
    setTimeout(() => {
      if (!_PasteSeen) _PasteInternal()
    }, 0)
    return
  }
  if ((Event.key === "Delete" || Event.key === "Backspace") && Selected.value.length) {
    Event.preventDefault()
    RemoveElement()
    return
  }
  if (Event.key === "Enter" && Chosen.value?.Kind === "text" && !Chosen.value.Locked) {
    Event.preventDefault()
    StartEditing(Chosen.value.Id, false)
    return
  }
  if (Event.key.startsWith("Arrow") && Selected.value.length) {
    Event.preventDefault()
    const Step = Event.shiftKey ? 10 : 1
    _Nudge(
      Event.key === "ArrowLeft" ? -Step : Event.key === "ArrowRight" ? Step : 0,
      Event.key === "ArrowUp" ? -Step : Event.key === "ArrowDown" ? Step : 0,
    )
    return
  }
  if (Event.key === "Escape") {
    Event.preventDefault()
    if (Sheet.value) CloseSheet()
    else if (Selected.value.length) Select([])
    else void Close()
  }
}

function OnRootKey(Event: KeyboardEvent): void {
  if (!Deleting.value) Event.stopPropagation()
}

function _OnPaste(Event: ClipboardEvent): void {
  if (Deleting.value || _Typing(Event.target)) return
  _PasteSeen = true
  const Files = Array.from(Event.clipboardData?.files ?? []).filter(F => F.type.startsWith("image/"))
  if (Files.length) {
    Event.preventDefault()
    _Replacing = null
    for (const File of Files) void _AddImage(File, null)
    return
  }
  if (_Clipboard.length) {
    Event.preventDefault()
    _PasteInternal()
  }
}

async function _LoadTv(): Promise<void> {
  let Data: Screen.Preview
  try {
    Data = await Screen.LoadPreview()
  } catch {
    return
  }

  const Own = `slides:${Props.Id}`
  const Pages = Data.Pages.includes(Own) ? Data.Pages : [...Data.Pages, Own]
  Tv.value = {
    ...Tv.value,
    Track: Data.Track,
    Cover: Data.Cover,
    Notice: Data.Notice.trim(),
    Pages,
    Active: Pages.indexOf(Own),
  }

  const Key = JSON.stringify(Data.Weather)
  if (Key === _WeatherKey) return
  _WeatherKey = Key
  void ReadWeather(Data.Weather).then(Reading => {
    if (Reading) Tv.value = { ...Tv.value, Temperature: Reading.Temperature, Condition: Reading.Condition }
  })
}

function OnFrameDown(Event: PointerEvent): void {
  if (CanvasEl.value?.contains(Event.target as Node)) return
  StopEditing()
  _Release()
  if (!Event.shiftKey) Select([])
}

function OnStageDown(Event: PointerEvent): void {
  const Target = Event.target as HTMLElement
  if (Target !== Event.currentTarget && !Target.classList.contains("Space")) return
  StopEditing()
  _Release()
  Select([])
}

function _OnResize(): void {
  Narrow.value = window.innerWidth < _NarrowPx
  if (!Narrow.value) {
    if (Sheet.value) CloseSheet()
    MultiMode.value = false
  }
  _Fit()
}

async function Close(): Promise<void> {
  StopEditing()
  End()
  if (Ready.value && (_SaveTimer || Saving.value === "error")) await Flush()
  Emit("Close")
}

async function ConfirmDelete(): Promise<void> {
  DeleteBusy.value = true
  try {
    if (_SaveTimer) {
      clearTimeout(_SaveTimer)
      _SaveTimer = null
    }
    await Api.Remove(Props.Id)
    Deleting.value = false
    Emit("Deleted")
  } catch (Caught) {
    Problem.value = Caught instanceof Error ? `Nie udało się usunąć: ${Caught.message}` : "Nie udało się usunąć."
    Deleting.value = false
  } finally {
    DeleteBusy.value = false
  }
}

watch(Name, () => {
  if (Ready.value) _Queue()
})

watch([Chosen, Scale, () => Editing.value], () => {
  void nextTick(_Measure)
}, { deep: true })

onMounted(async () => {
  document.documentElement.style.overflow = "hidden"
  window.addEventListener("keydown", _OnKey, true)
  window.addEventListener("paste", _OnPaste)
  window.addEventListener("resize", _OnResize)
  Narrow.value = window.innerWidth < _NarrowPx
  _TvFirst = _LoadTv()
  _ClockTimer = setInterval(() => { Tv.value = { ...Tv.value, Now: new Date() } }, _ClockMs)
  _PreviewTimer = setInterval(() => void _LoadTv(), _PreviewMs)

  try {
    const Loaded = await Api.Load(Props.Id)
    Name.value = Loaded.Name
    Slides.value = Loaded.Slides.length ? Loaded.Slides : [BlankSlide()]
    Ready.value = true
    await nextTick()
    if (StageEl.value) {
      _Observer = new ResizeObserver(_Fit)
      _Observer.observe(StageEl.value)
    }
    _Fit()
    void _Prime()
  } catch {
    Missing.value = true
  }
})

onUnmounted(() => {
  document.documentElement.style.overflow = ""
  window.removeEventListener("keydown", _OnKey, true)
  window.removeEventListener("paste", _OnPaste)
  window.removeEventListener("resize", _OnResize)
  window.removeEventListener("pointermove", _OnDragMove)
  window.removeEventListener("pointerup", _OnDragEnd)
  window.removeEventListener("pointercancel", _OnDragEnd)
  window.removeEventListener("pointermove", _OnTouchMove)
  window.removeEventListener("pointerup", _OnTouchUp)
  window.removeEventListener("pointercancel", _OnTouchUp)
  _Observer?.disconnect()
  if (_ClockTimer) clearInterval(_ClockTimer)
  if (_PreviewTimer) clearInterval(_PreviewTimer)
  if (_SaveTimer) {
    clearTimeout(_SaveTimer)
    void Flush()
  }
})
</script>

<template>
  <Teleport to="body">
    <div
      ref="RootEl"
      class="Editor"
      :class="{ Narrow }"
      role="dialog"
      aria-modal="true"
      aria-label="Edytor slajdów"
      @dragover="Swallow"
      @drop="Swallow"
      @keydown="OnRootKey"
    >
      <header class="Bar">
        <div class="BarStart">
          <button type="button" class="IconButton" aria-label="Zamknij edytor" @click="Close">
            <Icon Name="close" />
          </button>
          <input v-model="Name" class="Name" maxlength="80" aria-label="Nazwa kolekcji" spellcheck="false" />
        </div>

        <div v-if="Ready && !Narrow" class="Tools" role="toolbar" aria-label="Dodaj element">
          <button type="button" class="Tool" @click="AddText"><Icon Name="text" />Tekst</button>
          <button type="button" class="Tool" @click="AddRect"><Icon Name="rect" />Prostokąt</button>
          <button type="button" class="Tool" @click="AddEllipse"><Icon Name="ellipse" />Elipsa</button>
          <button type="button" class="Tool" @click="PickImage(null)"><Icon Name="image" />Obraz</button>
        </div>

        <div class="BarEnd">
          <Transition name="fade" mode="out-in">
            <span v-if="!Narrow || Saving === 'error'" :key="StatusText" class="Status" :class="{ Bad: Saving === 'error' }">
              {{ StatusText }}
              <button v-if="Saving === 'error'" type="button" class="Retry" @click="Flush">Ponów</button>
            </span>
          </Transition>
          <button type="button" class="IconButton" :disabled="!CanUndo" aria-label="Cofnij" title="Cofnij (Ctrl+Z)" @click="Undo">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M9 14 4 9l5-5" /><path d="M4 9h10.5a5.5 5.5 0 0 1 0 11H11" /></svg>
          </button>
          <button type="button" class="IconButton" :disabled="!CanRedo" aria-label="Ponów" title="Ponów (Ctrl+Shift+Z)" @click="Redo">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="m15 14 5-5-5-5" /><path d="M20 9H9.5a5.5 5.5 0 0 0 0 11H13" /></svg>
          </button>
          <button type="button" class="IconButton Danger" aria-label="Usuń kolekcję" title="Usuń kolekcję" @click="Deleting = true">
            <Icon Name="trash" />
          </button>
          <button type="button" class="Done" @click="Close">Gotowe</button>
        </div>
      </header>

      <div v-if="Missing" class="Notice">
        <p class="NoticeTitle">Nie znaleziono kolekcji</p>
        <p class="NoticeText">Mogła zostać usunięta w innym oknie.</p>
      </div>

      <div v-else-if="Ready && Active" class="Body">
        <Rail
          v-if="!Narrow"
          class="Side"
          :Slides="Slides"
          :Current="Current"
          @Select="OnSlidePick"
          @Add="AddSlide"
          @Duplicate="DuplicateSlide"
          @Remove="RemoveSlide"
          @Move="OnSlideMove"
        />

        <div class="StageWrap">
          <main
            ref="StageEl"
            class="Stage"
            @pointerdown.capture="OnTouchDown"
            @pointerdown="OnStageDown"
            @wheel="OnWheel"
            @dragenter="OnDragEnter"
            @dragover="OnDragOver"
            @dragleave="OnDragLeave"
            @drop="OnDrop"
          >
            <div class="Space" :style="SpaceStyle">
              <div class="Screen" :style="ScreenStyle">
                <TvFrame :Scale="Scale" :State="Tv" @pointerdown="OnFrameDown">
                  <div
                    ref="CanvasEl"
                    class="Canvas"
                    :style="CanvasStyle"
                    @pointerdown="OnCanvasDown"
                    @dblclick="OnCanvasDouble"
                  >
                    <SlideView :Slide="Active" :Scale="Scale" :Hidden="Editing" Interactive />

                    <div class="Overlay">
                      <span v-for="X in Guides.X" :key="`x${X}`" class="Guide Vertical" :style="{ left: `${X * Scale}px` }" />
                      <span v-for="Y in Guides.Y" :key="`y${Y}`" class="Guide Horizontal" :style="{ top: `${Y * Scale}px` }" />
                      <div v-if="HoverStyle" class="Hover" :style="HoverStyle" />
                      <div v-for="Part in Frame?.Parts ?? []" :key="Part.Id" class="Part" :style="Part.Style" />
                      <div v-if="Frame" class="Selection" :class="{ Group: Frame.Parts.length > 0 }" :style="Frame.Style">
                        <span
                          v-for="H in Frame.Handles"
                          :key="H"
                          class="Handle"
                          :class="H"
                          @pointerdown="OnHandleDown($event, H)"
                        />
                        <template v-if="Frame.Spin">
                          <span class="Stem" />
                          <span class="Spin" title="Obróć (Shift: co 15°)" @pointerdown="OnSpinDown" />
                        </template>
                      </div>
                    </div>

                    <div v-if="Editable" class="EditLayer" :style="{ transform: `scale(${Scale})` }">
                      <textarea
                        ref="TextEl"
                        class="TextEdit"
                        :style="EditStyle"
                        :value="Editable.Text"
                        spellcheck="false"
                        rows="1"
                        maxlength="4000"
                        @input="OnTextInput"
                        @blur="StopEditing"
                      />
                    </div>
                  </div>
                </TvFrame>

                <Transition name="skeleton">
                  <TvSkeleton v-if="!Primed" :Scale="Scale" />
                </Transition>
              </div>
            </div>
          </main>

          <Transition name="rise">
            <div v-if="Narrow && (Selected.length || HasClipboard || MultiMode)" class="Actions" role="toolbar" aria-label="Działania">
              <button
                v-if="Chosen?.Kind === 'text' && !Chosen.Locked"
                type="button"
                class="Action"
                @click="EditText"
              >
                <Icon Name="pencil" />Edytuj
              </button>
              <button v-if="Selected.length" type="button" class="Action" @click="CopySelection"><Icon Name="copy" />Kopiuj</button>
              <button v-if="HasClipboard" type="button" class="Action" @click="PasteClipboard"><Icon Name="paste" />Wklej</button>
              <button v-if="Selected.length" type="button" class="Action" @click="DuplicateElement"><Icon Name="duplicate" />Duplikuj</button>
              <button v-if="Selected.length" type="button" class="Action Danger" @click="RemoveElement"><Icon Name="trash" />Usuń</button>
              <button type="button" class="Action" :class="{ On: MultiMode }" :aria-pressed="MultiMode" @click="MultiMode = !MultiMode">
                <Icon Name="multi" />Wiele
              </button>
            </div>
          </Transition>

          <div class="Zoom" role="group" aria-label="Powiększenie">
            <button type="button" class="ZoomButton" aria-label="Pomniejsz" :disabled="Zoom <= 0.5" @click="ZoomOut"><Icon Name="minus" /></button>
            <button type="button" class="ZoomValue" title="Dopasuj do okna" @click="ZoomFit">{{ ZoomText }}</button>
            <button type="button" class="ZoomButton" aria-label="Powiększ" :disabled="Zoom >= 5" @click="ZoomIn"><Icon Name="plus" /></button>
          </div>

          <Layers
            v-if="!Narrow"
            :Elements="Active.Elements"
            :Selected="Selected"
            @Select="OnLayerSelect"
            @Toggle="OnLayerToggle"
            @Rename="OnLayerRename"
            @Reorder="OnLayerReorder"
            @Hover="Hovered = $event"
          />

          <Transition name="fade">
            <div v-if="Dropping" class="DropHint">Upuść obraz, aby dodać go do slajdu</div>
          </Transition>

          <Transition name="toast">
            <p v-if="Problem" class="Toast" role="alert">
              {{ Problem }}
              <button type="button" class="ToastClose" aria-label="Zamknij" @click="Problem = ''">
                <Icon Name="close" />
              </button>
            </p>
          </Transition>
        </div>

        <Inspector
          v-if="!Narrow"
          :Slide="Active"
          :Elements="Picked"
          :Index="Current"
          :Count="Slides.length"
          :TextHeight="Measured"
          @Patch="OnPatch"
          @Move="OnMove"
          @Background="OnBackground"
          @Arrange="OnArrange"
          @Align="OnAlign"
          @Distribute="OnDistribute"
          @Duplicate="DuplicateElement"
          @Remove="RemoveElement"
          @Replace="PickImage(Chosen?.Id ?? null)"
        />
      </div>

      <div v-else class="Notice">
        <span class="Spinner" aria-label="Wczytywanie" />
      </div>

      <template v-if="Narrow && Ready && Active">
        <Transition name="sheet">
          <section v-if="Sheet" class="MobileSheet" role="dialog" :aria-label="SheetTitle">
            <header class="SheetHead">
              <span class="SheetGrip" aria-hidden="true" />
              <p class="SheetTitle">{{ SheetTitle }}</p>
              <button type="button" class="IconButton" aria-label="Zamknij" @click="CloseSheet"><Icon Name="close" /></button>
            </header>

            <div class="SheetBody">
              <Rail
                v-if="Sheet === 'slides'"
                Docked
                :Slides="Slides"
                :Current="Current"
                @Select="OnSlidePick"
                @Add="AddSlide"
                @Duplicate="DuplicateSlide"
                @Remove="RemoveSlide"
                @Move="OnSlideMove"
              />

              <Layers
                v-else-if="Sheet === 'layers'"
                Docked
                :Multi="MultiMode"
                :Elements="Active.Elements"
                :Selected="Selected"
                @Select="OnLayerSelect"
                @Toggle="OnLayerToggle"
                @Rename="OnLayerRename"
                @Reorder="OnLayerReorder"
                @Hover="Hovered = $event"
              />

              <div v-else-if="Sheet === 'add'" class="AddGrid">
                <button type="button" class="AddTile" @click="AddFromSheet('text')"><Icon Name="text" />Tekst</button>
                <button type="button" class="AddTile" @click="AddFromSheet('rect')"><Icon Name="rect" />Prostokąt</button>
                <button type="button" class="AddTile" @click="AddFromSheet('ellipse')"><Icon Name="ellipse" />Elipsa</button>
                <button type="button" class="AddTile" @click="AddFromSheet('image')"><Icon Name="image" />Obraz</button>
                <button v-if="HasClipboard" type="button" class="AddTile" @click="AddFromSheet('paste')"><Icon Name="paste" />Wklej</button>
              </div>

              <div v-else-if="Sheet === 'text'" class="TextSheet">
                <textarea
                  v-if="Chosen?.Kind === 'text'"
                  class="TextArea"
                  :value="Chosen.Text"
                  rows="5"
                  maxlength="4000"
                  placeholder="Wpisz tekst"
                  @input="OnSheetText"
                />
                <p v-else class="SheetHint">Zaznacz pole tekstowe, aby zmienić jego treść.</p>
                <button type="button" class="Done Full" @click="CloseSheet">Gotowe</button>
              </div>

              <Inspector
                v-else
                class="Docked"
                :Slide="Active"
                :Elements="Picked"
                :Index="Current"
                :Count="Slides.length"
                :TextHeight="Measured"
                @Patch="OnPatch"
                @Move="OnMove"
                @Background="OnBackground"
                @Arrange="OnArrange"
                @Align="OnAlign"
                @Distribute="OnDistribute"
                @Duplicate="DuplicateElement"
                @Remove="RemoveElement"
                @Replace="PickImage(Chosen?.Id ?? null)"
              />
            </div>
          </section>
        </Transition>

        <nav class="Dock" aria-label="Panele edytora">
          <button type="button" class="DockButton" :class="{ On: Sheet === 'slides' }" @click="OpenSheet('slides')">
            <Icon Name="slides" /><span>Slajdy</span>
          </button>
          <button type="button" class="DockButton" :class="{ On: Sheet === 'layers' }" @click="OpenSheet('layers')">
            <Icon Name="layers" /><span>Warstwy</span>
          </button>
          <button type="button" class="DockButton Main" :class="{ On: Sheet === 'add' }" @click="OpenSheet('add')">
            <Icon Name="plus" /><span>Dodaj</span>
          </button>
          <button type="button" class="DockButton" :class="{ On: Sheet === 'style' }" @click="OpenSheet('style')">
            <Icon Name="sliders" /><span>Styl</span>
          </button>
        </nav>
      </template>

      <input ref="FileEl" type="file" accept="image/png,image/jpeg,image/webp,image/gif" multiple hidden @change="OnFile" />

      <Confirm
        :Open="Deleting"
        Title="Usunąć kolekcję?"
        :Body="`„${Name || 'Bez nazwy'}” i wszystkie jej slajdy znikną z telewizora. Tego nie można cofnąć.`"
        ConfirmLabel="Usuń kolekcję"
        :Busy="DeleteBusy"
        @Confirm="ConfirmDelete"
        @Cancel="Deleting = false"
      />
    </div>
  </Teleport>
</template>

<style scoped>
.Editor {
  position: fixed;
  inset: 0;
  z-index: 70;
  display: grid;
  grid-template-rows: 60px minmax(0, 1fr);
  background: var(--bg);
  color: var(--ink);
  animation: EditorIn 320ms var(--ease) both;
}

@keyframes EditorIn {
  from {
    opacity: 0;
    transform: scale(0.99);
  }
}

.Bar {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto minmax(0, 1fr);
  align-items: center;
  gap: 16px;
  border-bottom: 1px solid var(--line);
  background: #09080b;
  padding: 0 14px;
}

.BarStart,
.BarEnd {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: 6px;
}

.BarEnd {
  justify-content: flex-end;
}

.Name {
  min-width: 0;
  max-width: 320px;
  flex: 1;
  height: 36px;
  border: 0;
  border-radius: 9px;
  background: transparent;
  padding: 0 10px;
  font-family: var(--display);
  font-size: 16px;
  font-weight: 600;
  letter-spacing: -0.02em;
  color: var(--ink);
  outline: none;
  transition: background 150ms ease, box-shadow 150ms ease;
}

.Name:hover {
  background: rgba(255, 255, 255, 0.04);
}

.Name:focus {
  background: rgba(255, 255, 255, 0.05);
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.24);
}

.IconButton {
  display: flex;
  height: 36px;
  width: 36px;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  border-radius: 9px;
  color: var(--ink-soft);
  cursor: pointer;
  transition: background 150ms ease, color 150ms ease, opacity 150ms ease;
}

.IconButton svg,
.IconButton :deep(.Icon) {
  height: 18px;
  width: 18px;
}

.IconButton:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.06);
  color: var(--ink);
}

.IconButton:disabled {
  opacity: 0.3;
  cursor: default;
}

.IconButton.Danger:hover:not(:disabled) {
  background: rgba(239, 138, 138, 0.1);
  color: var(--danger);
}

.Tools {
  display: flex;
  align-items: center;
  gap: 2px;
  border-radius: 12px;
  background: rgba(255, 255, 255, 0.035);
  box-shadow: inset 0 0 0 1px var(--edge);
  padding: 3px;
}

.Tool {
  display: inline-flex;
  height: 34px;
  align-items: center;
  gap: 7px;
  border-radius: 9px;
  padding: 0 12px;
  font-size: 13.5px;
  font-weight: 500;
  color: var(--ink-soft);
  cursor: pointer;
  transition: background 150ms ease, color 150ms ease;
}

.Tool :deep(.Icon) {
  height: 17px;
  width: 17px;
}

.Tool:hover {
  background: rgba(255, 255, 255, 0.07);
  color: var(--ink);
}

.Tool:active {
  background: rgba(255, 255, 255, 0.1);
}

.Status {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  margin-right: 8px;
  font-size: 13px;
  white-space: nowrap;
  color: var(--ink-faint);
}

.Status.Bad {
  color: var(--danger);
}

.Retry {
  font-size: 13px;
  font-weight: 500;
  color: var(--ink);
  cursor: pointer;
}

.Done {
  height: 36px;
  margin-left: 6px;
  border-radius: 10px;
  background: var(--ink);
  padding: 0 16px;
  font-size: 13.5px;
  font-weight: 600;
  color: #0c0b0f;
  cursor: pointer;
  transition: background 150ms ease;
}

.Done:hover {
  background: #fff;
}

.Body {
  display: grid;
  min-height: 0;
  grid-template-columns: 212px minmax(0, 1fr) 292px;
}

.Side {
  border-right: 1px solid var(--line);
  background: #09080b;
}

.StageWrap {
  position: relative;
  min-width: 0;
  min-height: 0;
  overflow: hidden;
}

.Stage {
  position: absolute;
  inset: 0;
  overflow: auto;
  touch-action: pan-x pan-y;
  overscroll-behavior: contain;
  scrollbar-width: thin;
  scrollbar-color: rgba(255, 255, 255, 0.1) transparent;
}

.Space {
  display: flex;
  min-height: 100%;
  min-width: 100%;
  width: max-content;
  align-items: center;
  justify-content: center;
  box-sizing: border-box;
}

.Screen {
  position: relative;
  flex-shrink: 0;
}

.Zoom {
  position: absolute;
  bottom: 14px;
  left: 14px;
  z-index: 25;
  display: flex;
  align-items: center;
  gap: 2px;
  border-radius: 10px;
  background: rgba(14, 13, 17, 0.96);
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.08), 0 16px 32px -20px rgba(0, 0, 0, 0.9);
  padding: 3px;
}

.ZoomButton,
.ZoomValue {
  display: flex;
  height: 28px;
  align-items: center;
  justify-content: center;
  border-radius: 7px;
  color: var(--ink-soft);
  cursor: pointer;
  transition: background 140ms ease, color 140ms ease;
}

.ZoomButton {
  width: 28px;
}

.ZoomButton :deep(.Icon) {
  height: 14px;
  width: 14px;
}

.ZoomValue {
  min-width: 52px;
  padding: 0 6px;
  font-size: 12px;
  font-variant-numeric: tabular-nums;
}

.ZoomButton:hover:not(:disabled),
.ZoomValue:hover {
  background: rgba(255, 255, 255, 0.07);
  color: var(--ink);
}

.ZoomButton:disabled {
  opacity: 0.3;
  cursor: default;
}

.Actions {
  position: absolute;
  top: 10px;
  left: 50%;
  z-index: 26;
  display: flex;
  max-width: calc(100% - 20px);
  gap: 2px;
  overflow-x: auto;
  border-radius: 14px;
  background: rgba(14, 13, 17, 0.97);
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.09), 0 18px 36px -20px rgba(0, 0, 0, 0.9);
  padding: 4px;
  scrollbar-width: none;
  transform: translateX(-50%);
}

.Actions::-webkit-scrollbar {
  display: none;
}

.Action {
  display: flex;
  height: 44px;
  min-width: 52px;
  flex-shrink: 0;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 3px;
  border-radius: 10px;
  padding: 0 8px;
  font-size: 10.5px;
  font-weight: 500;
  color: var(--ink-soft);
  cursor: pointer;
  transition: background 140ms ease, color 140ms ease;
}

.Action :deep(.Icon) {
  height: 17px;
  width: 17px;
}

.Action:active,
.Action.On {
  background: rgba(255, 255, 255, 0.1);
  color: var(--ink);
}

.Action.Danger {
  color: var(--danger);
}

.MobileSheet {
  position: absolute;
  right: 0;
  bottom: calc(65px + env(safe-area-inset-bottom));
  left: 0;
  z-index: 60;
  display: flex;
  max-height: min(62dvh, 560px);
  flex-direction: column;
  border-radius: 18px 18px 0 0;
  background: #0e0d11;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.08), 0 -24px 48px -24px rgba(0, 0, 0, 0.9);
}

.SheetHead {
  position: relative;
  display: flex;
  height: 52px;
  flex-shrink: 0;
  align-items: center;
  padding: 0 8px 0 18px;
}

.SheetGrip {
  position: absolute;
  top: 6px;
  left: 50%;
  height: 4px;
  width: 36px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.14);
  transform: translateX(-50%);
}

.SheetTitle {
  margin: 0;
  flex: 1;
  font-family: var(--display);
  font-size: 16px;
  font-weight: 600;
  letter-spacing: -0.02em;
}

.SheetBody {
  display: flex;
  min-height: 0;
  flex: 1;
  flex-direction: column;
  overflow-y: auto;
  border-top: 1px solid var(--line);
}

.SheetBody .Docked {
  border-left: 0;
  background: transparent;
}

.AddGrid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
  padding: 16px;
}

.AddTile {
  display: flex;
  height: 84px;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  border-radius: 14px;
  background: rgba(255, 255, 255, 0.045);
  box-shadow: inset 0 0 0 1px var(--edge);
  font-size: 13.5px;
  font-weight: 500;
  color: var(--ink);
  cursor: pointer;
  transition: background 140ms ease;
}

.AddTile :deep(.Icon) {
  height: 22px;
  width: 22px;
  color: var(--ink-soft);
}

.AddTile:active {
  background: rgba(255, 255, 255, 0.09);
}

.TextSheet {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 16px;
}

.TextArea {
  min-height: 120px;
  resize: vertical;
  border: 0;
  border-radius: 12px;
  background: rgba(255, 255, 255, 0.04);
  box-shadow: inset 0 0 0 1px var(--edge);
  padding: 12px 14px;
  font: inherit;
  font-size: 16px;
  line-height: 1.5;
  color: var(--ink);
  outline: none;
}

.TextArea:focus {
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.3);
}

.SheetHint {
  margin: 0;
  font-size: 14px;
  color: var(--ink-mute);
}

.Done.Full {
  height: 46px;
  width: 100%;
  margin: 0;
  font-size: 15px;
}

.Dock {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  border-top: 1px solid var(--line);
  background: #09080b;
  padding: 6px 8px calc(6px + env(safe-area-inset-bottom));
}

.DockButton {
  display: flex;
  height: 52px;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 4px;
  border-radius: 12px;
  font-size: 11px;
  font-weight: 500;
  color: var(--ink-mute);
  cursor: pointer;
  transition: background 140ms ease, color 140ms ease;
}

.DockButton :deep(.Icon) {
  height: 20px;
  width: 20px;
}

.DockButton.On {
  background: rgba(255, 255, 255, 0.07);
  color: var(--ink);
}

.DockButton.Main :deep(.Icon) {
  height: 22px;
  width: 22px;
  border-radius: 7px;
  background: var(--ink);
  padding: 3px;
  color: #0c0b0f;
}

.Narrow {
  grid-template-rows: 52px minmax(0, 1fr) auto;
}

.Narrow .Bar {
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 8px;
  padding: 0 6px;
}

.Narrow .Name {
  max-width: none;
  font-size: 15px;
}

.Narrow .Done {
  height: 34px;
  margin-left: 2px;
  padding: 0 12px;
}

.Narrow .Body {
  grid-template-columns: minmax(0, 1fr);
}

.Narrow .Zoom {
  right: 10px;
  bottom: 10px;
  left: auto;
}

.Narrow .Toast {
  bottom: 70px;
}

@media (pointer: coarse) {
  .Handle {
    height: 20px;
    width: 20px;
    border-radius: 6px;
  }

  .Handle.nw { top: -10px; left: -10px; }
  .Handle.n { top: -10px; left: calc(50% - 10px); }
  .Handle.ne { top: -10px; right: -10px; }
  .Handle.e { top: calc(50% - 10px); right: -10px; }
  .Handle.se { right: -10px; bottom: -10px; }
  .Handle.s { bottom: -10px; left: calc(50% - 10px); }
  .Handle.sw { bottom: -10px; left: -10px; }
  .Handle.w { top: calc(50% - 10px); left: -10px; }

  .ZoomButton,
  .ZoomValue {
    height: 36px;
  }

  .ZoomButton {
    width: 36px;
  }
}

.skeleton-leave-active {
  transition: opacity 420ms ease;
}

.skeleton-leave-to {
  opacity: 0;
}

.rise-enter-active,
.rise-leave-active {
  transition: opacity 180ms ease, transform 260ms var(--ease);
}

.rise-enter-from,
.rise-leave-to {
  opacity: 0;
  transform: translate(-50%, -6px);
}

.sheet-enter-active,
.sheet-leave-active {
  transition: opacity 200ms ease, transform 340ms var(--ease);
}

.sheet-enter-from,
.sheet-leave-to {
  opacity: 0;
  transform: translateY(24px);
}

.Canvas {
  position: relative;
  outline: 1px dashed rgba(255, 255, 255, 0.18);
  outline-offset: 0;
}

.Overlay {
  position: absolute;
  inset: 0;
  pointer-events: none;
}

.Guide {
  position: absolute;
  background: var(--rose);
}

.Guide.Vertical {
  top: -12px;
  bottom: -12px;
  width: 1px;
  transform: translateX(-0.5px);
}

.Guide.Horizontal {
  right: -12px;
  left: -12px;
  height: 1px;
  transform: translateY(-0.5px);
}

.Selection {
  position: absolute;
  box-shadow: 0 0 0 1.5px var(--ink);
}

.Selection.Group {
  box-shadow: 0 0 0 1px rgba(244, 242, 247, 0.7);
}

.Part {
  position: absolute;
  box-shadow: 0 0 0 1px var(--rose);
}

.Hover {
  position: absolute;
  box-shadow: 0 0 0 1px rgba(224, 168, 168, 0.8);
}

.Handle {
  position: absolute;
  height: 10px;
  width: 10px;
  border-radius: 3px;
  background: var(--ink);
  box-shadow: 0 0 0 1.5px #0c0b0f;
  pointer-events: auto;
  touch-action: none;
}

.Handle.nw { top: -5px; left: -5px; cursor: nwse-resize; }
.Handle.n { top: -5px; left: calc(50% - 5px); cursor: ns-resize; }
.Handle.ne { top: -5px; right: -5px; cursor: nesw-resize; }
.Handle.e { top: calc(50% - 5px); right: -5px; cursor: ew-resize; }
.Handle.se { right: -5px; bottom: -5px; cursor: nwse-resize; }
.Handle.s { bottom: -5px; left: calc(50% - 5px); cursor: ns-resize; }
.Handle.sw { bottom: -5px; left: -5px; cursor: nesw-resize; }
.Handle.w { top: calc(50% - 5px); left: -5px; cursor: ew-resize; }

.Stem {
  position: absolute;
  bottom: 100%;
  left: calc(50% - 0.75px);
  height: 22px;
  width: 1.5px;
  background: var(--ink);
}

.Spin {
  position: absolute;
  bottom: calc(100% + 22px);
  left: calc(50% - 6px);
  height: 12px;
  width: 12px;
  border-radius: 999px;
  background: var(--ink);
  box-shadow: 0 0 0 1.5px #0c0b0f;
  cursor: grab;
  pointer-events: auto;
  touch-action: none;
}

.Spin:active {
  cursor: grabbing;
}

.EditLayer {
  position: absolute;
  top: 0;
  left: 0;
  width: 1776px;
  height: 698px;
  transform-origin: 0 0;
  pointer-events: none;
}

.TextEdit {
  display: block;
  overflow: hidden;
  resize: none;
  border: 0;
  background: transparent;
  padding: 0;
  outline: 2px solid var(--rose);
  outline-offset: 6px;
  pointer-events: auto;
  caret-color: currentColor;
}

.DropHint {
  position: absolute;
  inset: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 20px;
  background: rgba(5, 5, 7, 0.78);
  box-shadow: inset 0 0 0 1px var(--edge-strong);
  font-family: var(--display);
  font-size: 20px;
  font-weight: 600;
  letter-spacing: -0.02em;
  pointer-events: none;
}

.Toast {
  position: absolute;
  bottom: 22px;
  left: 50%;
  display: flex;
  max-width: calc(100% - 48px);
  align-items: center;
  gap: 12px;
  margin: 0;
  border-radius: 12px;
  background: #16151a;
  box-shadow: inset 0 0 0 1px rgba(239, 138, 138, 0.2);
  padding: 10px 10px 10px 16px;
  font-size: 13.5px;
  color: var(--danger);
  transform: translateX(-50%);
}

.ToastClose {
  display: flex;
  height: 26px;
  width: 26px;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  border-radius: 7px;
  color: var(--ink-mute);
  cursor: pointer;
}

.ToastClose :deep(.Icon) {
  height: 14px;
  width: 14px;
}

.ToastClose:hover {
  background: rgba(255, 255, 255, 0.06);
  color: var(--ink);
}

.Notice {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 24px;
  text-align: center;
}

.NoticeTitle {
  margin: 0;
  font-family: var(--display);
  font-size: 20px;
  font-weight: 600;
  letter-spacing: -0.02em;
}

.NoticeText {
  margin: 0;
  font-size: 14px;
  color: var(--ink-mute);
}

.Spinner {
  height: 22px;
  width: 22px;
  border-radius: 999px;
  box-shadow: inset 0 0 0 2px rgba(255, 255, 255, 0.12);
  border-top: 2px solid var(--ink);
  animation: Spin 800ms linear infinite;
}

@keyframes Spin {
  to { transform: rotate(360deg); }
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 160ms ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.toast-enter-active,
.toast-leave-active {
  transition: opacity 200ms ease, transform 300ms var(--ease);
}

.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translate(-50%, 8px);
}

@media (prefers-reduced-motion: reduce) {
  .Editor,
  .Spinner {
    animation: none;
  }

  .Frame {
    transition: none;
  }
}
</style>
