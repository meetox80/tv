import { ref, type Ref } from "vue"

const _Query = typeof matchMedia === "function" ? matchMedia("(min-width: 1024px)") : null

export const Wide: Ref<boolean> = ref(_Query?.matches ?? true)

_Query?.addEventListener("change", E => {
  Wide.value = E.matches
})
