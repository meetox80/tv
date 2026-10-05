export function Read<T>(Key: string): T | null {
  try {
    const Raw = localStorage.getItem(Key)
    return Raw ? (JSON.parse(Raw) as T) : null
  } catch {
    return null
  }
}

export function Write<T>(Key: string, Value: T): void {
  try {
    localStorage.setItem(Key, JSON.stringify(Value))
  } catch {}
}
