import type { Directive } from "vue"

const _Widths = new WeakMap<HTMLElement, number>()
let _Observer: ResizeObserver | null = null

function _Px(El: HTMLElement, Name: string): number {
  return parseFloat(getComputedStyle(El).getPropertyValue(Name)) || 0
}

function _Fit(El: HTMLElement): void {
  if (!El.isConnected) return
  const Min = _Px(El, "--fit-min")
  const Max = _Px(El, "--fit-max")
  if (!Min || !Max) return

  El.dataset.wrap = "0"
  El.style.fontSize = `${Max}px`

  const Available = El.clientWidth
  const Natural = El.scrollWidth
  if (!Available || Natural <= Available) return

  El.style.fontSize = `${Math.max(Min, Math.floor(((Max * Available) / Natural) * 10) / 10)}px`
  if (El.scrollWidth > El.clientWidth + 0.5) El.dataset.wrap = "1"
}

function _Watch(): ResizeObserver {
  if (_Observer) return _Observer
  _Observer = new ResizeObserver(Entries => {
    for (const Entry of Entries) {
      const El = Entry.target as HTMLElement
      const Width = Math.round(Entry.contentRect.width)
      if (_Widths.get(El) === Width) continue
      _Widths.set(El, Width)
      _Fit(El)
    }
  })
  return _Observer
}

export const vFit: Directive<HTMLElement> = {
  mounted(El) {
    _Fit(El)
    _Watch().observe(El)
    void document.fonts?.ready.then(() => _Fit(El))
  },
  updated(El) {
    _Fit(El)
  },
  unmounted(El) {
    _Observer?.unobserve(El)
    _Widths.delete(El)
  },
}
