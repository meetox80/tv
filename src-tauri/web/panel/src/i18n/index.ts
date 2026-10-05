import Pl from "./locales/pl"

function t(Key: string): string {
  return Pl[Key] ?? Key
}

export function useI18n(): { t: (Key: string) => string } {
  return { t }
}
