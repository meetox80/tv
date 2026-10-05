export function activate() {
  const Label = document.createElement("div")
  Label.textContent = "hello world"
  Label.style.cssText = [
    "position: fixed",
    "top: 14px",
    "left: 18px",
    "z-index: 2147483647",
    "color: #ff2d2d",
    "font: 600 15px Inter, system-ui, sans-serif",
    "pointer-events: none",
  ].join(";")
  document.body.append(Label)
  return () => Label.remove()
}
