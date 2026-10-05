export function BlockZoom(): void {
  const Prevent = (E: Event): void => E.preventDefault()

  for (const Name of ["gesturestart", "gesturechange", "gestureend"]) {
    document.addEventListener(Name, Prevent, { passive: false })
  }

  document.addEventListener(
    "touchmove",
    (E: TouchEvent): void => {
      if (E.touches.length > 1) E.preventDefault()
    },
    { passive: false },
  )

  document.addEventListener(
    "wheel",
    (E: WheelEvent): void => {
      if (E.ctrlKey) E.preventDefault()
    },
    { passive: false },
  )

  document.addEventListener("keydown", (E: KeyboardEvent): void => {
    if ((E.ctrlKey || E.metaKey) && ["+", "-", "=", "_", "0"].includes(E.key)) E.preventDefault()
  })
}
