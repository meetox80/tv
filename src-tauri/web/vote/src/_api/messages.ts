import { ApiError } from "./http"
import type { SongStatus } from "./types"

export const StatusLabel: Record<SongStatus, string> = {
  pending: "oczekuje",
  approved: "zatwierdzona",
  added: "dodana do playlisty",
  rejected: "odrzucona",
}

export function DuplicateMessage(Status: SongStatus | null): string {
  if (Status === "approved") return "Ta piosenka jest już w rankingu. Zamiast zgłaszać ją jeszcze raz, oddaj na nią głos."
  if (Status === "added") return "Ta piosenka jest już na szkolnej playliście."
  if (Status === "rejected") return "Ta piosenka była już zgłoszona i została odrzucona."
  return "Ktoś już zgłosił tę piosenkę. Czeka na zatwierdzenie."
}

function _Seconds(Value: number): string {
  if (Value < 60) return `${Value} s`
  return `${Math.ceil(Value / 60)} min`
}

export function Describe(Err: unknown): string {
  if (!(Err instanceof ApiError)) return "Coś poszło nie tak. Spróbuj ponownie."

  switch (Err.Code) {
    case "network":
      return "Brak połączenia z serwerem. Sprawdź internet i spróbuj ponownie."
    case "slow_down": {
      const Wait = Err.RetryAfter
      return Wait ? `Za dużo prób naraz. Spróbuj ponownie za ${_Seconds(Wait)}.` : "Za dużo prób naraz. Odczekaj chwilę."
    }
    case "closed":
      return "Głosowanie jest teraz wyłączone."
    case "banned":
      return "To urządzenie zostało zablokowane."
    case "captcha":
      return "Weryfikacja się nie powiodła. Spróbuj jeszcze raz."
    case "captcha_unconfigured":
      return "Weryfikacja nie jest jeszcze skonfigurowana. Wróć później."
    case "query":
      return "Wpisz od 2 do 100 znaków."
    case "upstream":
      return "Wyszukiwarka utworów chwilowo nie odpowiada. Spróbuj za moment."
    case "not_found":
      return "Nie udało się znaleźć tego utworu. Wybierz inny wynik."
    case "limit":
      return "Dzisiejsze propozycje z tego urządzenia są już wykorzystane. Nowe pojawią się po północy."
    case "duplicate": {
      const Status = typeof Err.Body.Status === "string" ? (Err.Body.Status as SongStatus) : null
      return DuplicateMessage(Status)
    }
    case "origin":
      return "Przeglądarka wysłała żądanie z nieoczekiwanego miejsca. Odśwież stronę i spróbuj ponownie."
    case "device":
      return "Musimy najpierw potwierdzić, że jesteś człowiekiem."
    default:
      return "Coś poszło nie tak. Spróbuj ponownie."
  }
}
