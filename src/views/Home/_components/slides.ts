import type { CSSProperties } from "vue"

export const SlideWidth = 1776
export const SlideHeight = 698

export type ElementKind = "text" | "rect" | "ellipse" | "image"

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
  Align?: "left" | "center" | "right"
  Font?: "display" | "body"
  Fill?: string
  Radius?: number
  Corners?: CornerRadii
  Rotation?: number
  Opacity?: number
  Asset?: string
  Fit?: "cover" | "contain" | "fill"
  Hidden?: boolean
}

export interface Slide {
  Id: string
  Background?: string
  Elements: SlideElement[]
}

export interface SlideCollection {
  Id: string
  Name: string
  Slides: Slide[]
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
