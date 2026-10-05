export function Plural(Count: number, One: string, Few: string, Many: string): string {
  const Abs = Math.abs(Count)
  if (Abs === 1) return One
  const Tens = Abs % 100
  const Units = Abs % 10
  if (Units >= 2 && Units <= 4 && (Tens < 12 || Tens > 14)) return Few
  return Many
}

export function Proposals(Count: number): string {
  return `${Count} ${Plural(Count, "propozycja", "propozycje", "propozycji")}`
}

export function Duration(Ms: number | null): string {
  if (!Ms || Ms <= 0) return ""
  const Total = Math.round(Ms / 1000)
  const Minutes = Math.floor(Total / 60)
  const Seconds = String(Total % 60).padStart(2, "0")
  return `${Minutes}:${Seconds}`
}

const _Months = [
  "stycznia", "lutego", "marca", "kwietnia", "maja", "czerwca",
  "lipca", "sierpnia", "września", "października", "listopada", "grudnia",
]

function _Clock(At: Date): string {
  return `${String(At.getHours()).padStart(2, "0")}:${String(At.getMinutes()).padStart(2, "0")}`
}

function _SameDay(A: Date, B: Date): boolean {
  return A.getFullYear() === B.getFullYear() && A.getMonth() === B.getMonth() && A.getDate() === B.getDate()
}

export function When(Unix: number): string {
  const At = new Date(Unix * 1000)
  const Now = new Date()
  const Yesterday = new Date(Now.getFullYear(), Now.getMonth(), Now.getDate() - 1)

  if (_SameDay(At, Now)) return `dziś, ${_Clock(At)}`
  if (_SameDay(At, Yesterday)) return `wczoraj, ${_Clock(At)}`
  return `${At.getDate()} ${_Months[At.getMonth()]}, ${_Clock(At)}`
}

export function Score(Value: number): string {
  if (Math.abs(Value) < 1000) return String(Value)
  return `${(Value / 1000).toFixed(1).replace(".", ",")} tys.`
}

export function SameSong(A: { Title: string; Artist: string }, B: { Title: string; Artist: string }): boolean {
  const _Key = (S: string) => S.trim().toLocaleLowerCase("pl")
  return _Key(A.Title) === _Key(B.Title) && _Key(A.Artist) === _Key(B.Artist)
}
