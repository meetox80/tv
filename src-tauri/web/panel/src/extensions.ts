import { reactive } from "vue"
import * as Api from "./_api/extensions"

export interface PanelHost {
  Id: string
  Name: string
  Version: string
}

type Cleanup = () => void

const _Active = new Map<string, Cleanup | null>()

export const Stale: Set<string> = reactive(new Set<string>())

let _Loading: Promise<void> | null = null

async function _Activate(Info: Api.ExtensionInfo): Promise<void> {
  const Source = await Api.PanelModule(Info.Id)
  const Url = URL.createObjectURL(new Blob([Source], { type: "text/javascript" }))

  try {
    const Module = await import(Url)
    const Activate = typeof Module.activate === "function" ? Module.activate : Module.default
    if (typeof Activate !== "function") {
      _Active.set(Info.Id, null)
      return
    }

    const Host: PanelHost = { Id: Info.Id, Name: Info.Name, Version: Info.Version }
    const Result = await Activate(Host)
    _Active.set(Info.Id, typeof Result === "function" ? Result : null)
  } finally {
    URL.revokeObjectURL(Url)
  }
}

export async function Enable(Info: Api.ExtensionInfo): Promise<void> {
  if (!Info.Panel || _Active.has(Info.Id) || Stale.has(Info.Id)) return
  await _Activate(Info)
}

export function Disable(Id: string): void {
  if (!_Active.has(Id)) return
  const Cleanup = _Active.get(Id) ?? null
  _Active.delete(Id)

  if (!Cleanup) {
    Stale.add(Id)
    return
  }

  try {
    Cleanup()
  } catch (Problem) {
    Stale.add(Id)
    console.error(`[wtyczka ${Id}]`, Problem)
  }
}

export function Load(): Promise<void> {
  if (_Loading) return _Loading

  _Loading = (async () => {
    let Items: Api.ExtensionInfo[] = []
    try {
      Items = (await Api.List()).Items
    } catch {
      _Loading = null
      return
    }

    for (const Info of Items) {
      if (!Info.Enabled) continue
      try {
        await Enable(Info)
      } catch (Problem) {
        console.error(`[wtyczka ${Info.Id}]`, Problem)
      }
    }
  })()

  return _Loading
}
