export interface Schedule {
  Duration: number
  LessonTimes: string[]
  BreakTimes: string[]
}

export interface Period {
  IsLesson: boolean
  Number: number
  StartMs: number
  EndMs: number
}

const _DayMs = 24 * 60 * 60 * 1000

function _AtToday(Text: string): number {
  const [Hours, Minutes] = Text.split(":").map(Number)
  const Day = new Date()
  return new Date(Day.getFullYear(), Day.getMonth(), Day.getDate(), Hours, Minutes, 0, 0).getTime()
}

function _Range(Text: string): [number, number] {
  const [Start, End] = Text.split(" - ")
  const From = _AtToday(Start)
  let To = _AtToday(End)
  if (To <= From) To += _DayMs
  return [From, To]
}

export function Timeline(Plan: Schedule | null): Period[] {
  if (!Plan) return []

  const All: Period[] = []

  Plan.LessonTimes.forEach((Text, Index) => {
    const [From, To] = _Range(Text)
    All.push({ IsLesson: true, Number: Index + 1, StartMs: From, EndMs: To })
  })

  Plan.BreakTimes.forEach((Text, Index) => {
    const [From, To] = _Range(Text)
    All.push({ IsLesson: false, Number: Index + 1, StartMs: From, EndMs: To })
  })

  return All.sort((A, B) => A.StartMs - B.StartMs)
}

export function Resolve(Line: Period[], At: number): Period | null {
  return Line.find((P) => At >= P.StartMs && At < P.EndMs) ?? null
}
