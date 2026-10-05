export function Copy(Text: string): Promise<void> {
  if (navigator.clipboard?.writeText) {
    return navigator.clipboard.writeText(Text).catch(() => FallbackCopy(Text))
  }
  return FallbackCopy(Text)
}

function FallbackCopy(Text: string): Promise<void> {
  const Area = document.createElement("textarea")
  Area.value = Text
  Area.style.position = "fixed"
  Area.style.left = "-9999px"
  document.body.appendChild(Area)
  Area.select()
  document.execCommand("copy")
  document.body.removeChild(Area)
  return Promise.resolve()
}
