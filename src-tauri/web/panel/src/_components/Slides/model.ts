import type { CSSProperties } from "vue"

export const SlideWidth = 1776
export const SlideHeight = 698

export type ElementKind = "text" | "rect" | "ellipse" | "image"
export type TextAlign = "left" | "center" | "right"
export type TextFont = "display" | "body"
export type ImageFit = "cover" | "contain" | "fill"

export type CornerRadii = [number, number, number, number]

export interface SlideElement {
  Id: string
  Kind: ElementKind
  X: number
  Y: number
  W: number
  H: number
  Text?: string
  Size?: number
  Weight?: number
  Color?: string
  Align?: TextAlign
  Font?: TextFont
  Fill?: string
  Radius?: number
  Corners?: CornerRadii
  Rotation?: number
  Opacity?: number
  Asset?: string
  Fit?: ImageFit
  Name?: string
  Hidden?: boolean
  Locked?: boolean
}

export interface Slide {
  Id: string
  Background?: string
  Elements: SlideElement[]
}

export interface Collection {
  Id: string
  Name: string
  Slides: Slide[]
  Updated: number
}

export interface CollectionSummary {
  Id: string
  Name: string
  Count: number
  Updated: number
  Cover: Slide | null
}

export const Palette = [
  "#f4f2f7", "#b9b5c6", "#5d5868", "#16151a", "#050507",
  "#e0a8a8", "#7a1f1f", "#ef8a8a", "#e8c07a", "#8fd9a8", "#8ab4f8",
]

export function Uid(): string {
  const Bytes = new Uint8Array(6)
  crypto.getRandomValues(Bytes)
  return Array.from(Bytes, B => B.toString(16).padStart(2, "0")).join("")
}

export function Clone<T>(Value: T): T {
  return JSON.parse(JSON.stringify(Value)) as T
}

export function AssetUrl(Id: string): string {
  return `/api/slides/assets/${Id}`
}

export function BlankSlide(): Slide {
  return { Id: Uid(), Elements: [] }
}

export function NewText(X: number, Y: number): SlideElement {
  return { Id: Uid(), Kind: "text", X, Y, W: 720, H: 80, Text: "Nowy tekst", Size: 64, Weight: 600, Color: "#f4f2f7", Align: "left", Font: "display", Opacity: 1 }
}

export function NewRect(X: number, Y: number): SlideElement {
  return { Id: Uid(), Kind: "rect", X, Y, W: 420, H: 260, Fill: "#e0a8a8", Radius: 24, Opacity: 1 }
}

export function NewEllipse(X: number, Y: number): SlideElement {
  return { Id: Uid(), Kind: "ellipse", X, Y, W: 280, H: 280, Fill: "#8ab4f8", Opacity: 1 }
}

export function NewImage(X: number, Y: number, W: number, H: number, Asset: string): SlideElement {
  return { Id: Uid(), Kind: "image", X, Y, W, H, Asset, Fit: "cover", Radius: 0, Opacity: 1 }
}

const _Kinds: Record<ElementKind, string> = {
  text: "Tekst",
  rect: "Prostokąt",
  ellipse: "Elipsa",
  image: "Obraz",
}

export function KindName(Kind: ElementKind): string {
  return _Kinds[Kind]
}

export function LayerName(Element: SlideElement): string {
  if (Element.Name) return Element.Name
  if (Element.Kind === "text") {
    const Line = (Element.Text ?? "").split("\n").find(L => L.trim())?.trim()
    if (Line) return Line.length > 40 ? `${Line.slice(0, 40)}…` : Line
  }
  return _Kinds[Element.Kind]
}

export function Bounds(List: SlideElement[], Height: (E: SlideElement) => number): { X: number; Y: number; W: number; H: number } | null {
  if (!List.length) return null
  let L = Infinity
  let T = Infinity
  let R = -Infinity
  let B = -Infinity
  for (const E of List) {
    L = Math.min(L, E.X)
    T = Math.min(T, E.Y)
    R = Math.max(R, E.X + E.W)
    B = Math.max(B, E.Y + Height(E))
  }
  return { X: L, Y: T, W: R - L, H: B - T }
}

function _Rounding(Element: SlideElement): string {
  if (Element.Corners) return Element.Corners.map(R => `${R}px`).join(" ")
  return `${Element.Radius ?? 0}px`
}

export function ElementStyle(Element: SlideElement): CSSProperties {
  const Base: CSSProperties = {
    position: "absolute",
    left: `${Element.X}px`,
    top: `${Element.Y}px`,
    width: `${Element.W}px`,
    opacity: Element.Opacity ?? 1,
    transform: Element.Rotation ? `rotate(${Element.Rotation}deg)` : undefined,
  }

  if (Element.Kind === "text") {
    const Body = Element.Font === "body"
    return {
      ...Base,
      margin: 0,
      fontFamily: Body ? '"Inter", system-ui, sans-serif' : '"Space Grotesk", "Inter", sans-serif',
      fontSize: `${Element.Size ?? 48}px`,
      fontWeight: Element.Weight ?? 500,
      lineHeight: Body ? 1.35 : 1.12,
      letterSpacing: Body ? "-0.005em" : "-0.025em",
      color: Element.Color ?? "#f4f2f7",
      textAlign: Element.Align ?? "left",
      whiteSpace: "pre-wrap",
      overflowWrap: "anywhere",
    }
  }

  const Sized: CSSProperties = { ...Base, height: `${Element.H}px` }

  if (Element.Kind === "rect") return { ...Sized, background: Element.Fill, borderRadius: _Rounding(Element) }
  if (Element.Kind === "ellipse") return { ...Sized, background: Element.Fill, borderRadius: "50%" }

  return { ...Sized, overflow: "hidden", borderRadius: _Rounding(Element) }
}
