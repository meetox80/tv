export type ImageState = "pending" | "ready" | "failed"

interface Entry {
  State: ImageState
  Done: Promise<boolean>
  Img: HTMLImageElement
}

const _Entries = new Map<string, Entry>()
const _Scopes = new Map<string, Set<string>>()

function _Load(Url: string): Entry {
  const Known = _Entries.get(Url)
  if (Known && Known.State !== "failed") return Known

  const Img = new Image()
  Img.decoding = "async"
  Img.src = Url

  const Fresh: Entry = { State: "pending", Img, Done: Promise.resolve(false) }
  Fresh.Done = Img.decode().then(
    () => {
      Fresh.State = "ready"
      return true
    },
    () => {
      Fresh.State = "failed"
      return false
    },
  )

  _Entries.set(Url, Fresh)
  return Fresh
}

function _Prune(): void {
  const Wanted = new Set<string>()
  for (const Urls of _Scopes.values()) {
    for (const Url of Urls) Wanted.add(Url)
  }

  for (const Url of [..._Entries.keys()]) {
    if (!Wanted.has(Url)) _Entries.delete(Url)
  }
}

function _Wait(Ms: number): Promise<void> {
  return new Promise(Resolve => setTimeout(Resolve, Ms))
}

export async function Preload(
  Scope: string,
  Urls: (string | null | undefined)[],
  WaitMs: number,
): Promise<Map<string, ImageState>> {
  const List: string[] = [...new Set(Urls.filter((Url): Url is string => !!Url))]
  _Scopes.set(Scope, new Set(List))
  _Prune()

  const Loads: Entry[] = List.map(_Load)
  await Promise.race([Promise.all(Loads.map(Load => Load.Done)), _Wait(WaitMs)])

  return new Map(List.map((Url, Index) => [Url, Loads[Index].State]))
}

export function Decoded(Url: string): Promise<boolean> {
  return _Load(Url).Done
}
