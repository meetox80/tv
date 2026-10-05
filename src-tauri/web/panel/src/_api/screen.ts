import { Fetch, PostJson } from "./http"

export interface WeatherConfig {
  Enabled: boolean
  Latitude: number
  Longitude: number
}

export type LessonDuration = 30 | 45

export interface LessonsConfig {
  Duration: LessonDuration
}

export interface LessonSchedule {
  Duration: LessonDuration
  LessonTimes: string[]
  BreakTimes: string[]
}

export type PageId = string

export interface ScreenPage {
  Id: PageId
  Enabled: boolean
  Seconds: number
  Label?: string
}

export interface LiveState {
  Page: PageId
  Pages: PageId[]
  Seconds: number
  Elapsed: number
}

export interface ScreenConfig {
  Weather: WeatherConfig
  Lessons: LessonsConfig
  Notice: string
  Pages?: ScreenPage[]
}

export async function Load(): Promise<ScreenConfig> {
  const Res = await Fetch("/api/screen")
  if (!Res.ok) throw new Error(`${Res.status}`)
  return Res.json()
}

export async function SaveWeather(Config: WeatherConfig): Promise<void> {
  const Res = await PostJson("/api/screen/weather", Config)
  if (!Res.ok) throw new Error(`${Res.status}`)
}

export async function LoadLessons(Duration: LessonDuration): Promise<LessonSchedule> {
  const Res = await Fetch(`/api/screen/lessons?duration=${Duration}`)
  if (!Res.ok) throw new Error(`${Res.status}`)
  return Res.json()
}

export async function SaveNotice(Notice: string): Promise<void> {
  const Res = await PostJson("/api/screen/notice", { Notice })
  if (!Res.ok) throw new Error(`${Res.status}`)
}

export interface Preview {
  Track: { Title: string; Artist: string } | null
  Cover: string | null
  Notice: string
  Weather: WeatherConfig
  Pages: PageId[]
}

export async function LoadPreview(): Promise<Preview> {
  const Res = await Fetch("/api/screen/preview")
  if (!Res.ok) throw new Error(`${Res.status}`)
  return Res.json()
}

export async function LoadLive(): Promise<LiveState | null> {
  const Res = await Fetch("/api/screen/live")
  if (!Res.ok) throw new Error(`${Res.status}`)
  return Res.json()
}

export async function SavePages(Pages: ScreenPage[]): Promise<void> {
  const Res = await PostJson("/api/screen/pages", { Pages })
  if (!Res.ok) throw new Error(`${Res.status}`)
}

export async function SaveLessons(Config: LessonsConfig): Promise<void> {
  const Res = await PostJson("/api/screen/lessons", Config)
  if (!Res.ok) throw new Error(`${Res.status}`)
}
