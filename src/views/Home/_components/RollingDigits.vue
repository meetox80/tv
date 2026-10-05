<script setup lang="ts">
import { computed, onMounted, ref, watch, type Ref } from "vue"

const Props = defineProps<{ Value: string }>()

const _Digits = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"]
const _Cycle = _Digits.length
const _Reach = _Cycle / 2
const _CellHeight = 2
const _SlotHeight = 1.25
const _CellPad = (_CellHeight - _SlotHeight) / 2

const Strip = Array.from(
  { length: _Cycle + _Reach },
  (_, Index) => _Digits[Index % _Cycle],
)

const Row: Ref<HTMLElement | null> = ref(null)

const Shape = computed(() => Props.Value.replace(/[0-9]/g, "#"))

const Cells = computed(() =>
  Shape.value.split("").map((Char, Index) => ({
    Key: `${Index}-${Char}`,
    IsColon: Char === ":",
  }))
)

const _Reels: (HTMLElement | null)[] = []
const _Shown: (number | undefined)[] = []

function Bind(Index: number, El: unknown): void {
  _Reels[Index] = (El as HTMLElement | null) ?? null
}

function _Shift(Index: number): string {
  return `translateY(-${(Index * _CellHeight + _CellPad).toFixed(4)}em)`
}

function _Delta(From: number, To: number): number {
  const Step = (To - From + _Cycle) % _Cycle
  return Step > _Reach ? Step - _Cycle : Step
}

function _Paint(Value: string, Animate: boolean): void {
  const Pending: { El: HTMLElement; Target: number | null }[] = []

  Value.split("").forEach((Char, Index) => {
    const Next = _Digits.indexOf(Char)
    if (Next < 0) return

    const El = _Reels[Index]
    if (!El) return

    const Prev = _Shown[Index]
    if (Animate && Prev === Next) return

    _Shown[Index] = Next

    if (!Animate || Prev === undefined) {
      El.style.transition = "none"
      El.style.transform = _Shift(Next)
      Pending.push({ El, Target: null })
      return
    }

    const Move = _Delta(Prev, Next)
    const Start = Prev + Move >= 0 ? Prev : Prev + _Cycle

    El.style.transition = "none"
    El.style.transform = _Shift(Start)
    Pending.push({ El, Target: Start + Move })
  })

  if (Pending.length === 0) return

  void Row.value?.offsetHeight

  Pending.forEach(({ El, Target }) => {
    El.style.transition = ""
    if (Target !== null) El.style.transform = _Shift(Target)
  })
}

watch(() => Props.Value, (Next) => _Paint(Next, true))

watch(Shape, () => {
  _Shown.length = 0
  _Paint(Props.Value, false)
}, { flush: "post" })

onMounted(() => _Paint(Props.Value, false))
</script>

<template>
  <div ref="Row" class="Row">
    <template v-for="(Cell, Index) in Cells" :key="Cell.Key">
      <div v-if="Cell.IsColon" class="Colon">
        <span class="Dot" />
        <span class="Dot" />
      </div>

      <div v-else class="Slot">
        <div :ref="(El) => Bind(Index, El)" class="Reel">
          <div v-for="(Digit, Cursor) in Strip" :key="Cursor" class="Cell">{{ Digit }}</div>
        </div>
      </div>
    </template>
  </div>
</template>

<style scoped>
.Row {
  display: flex;
  font-weight: 700;
  line-height: 1;
  letter-spacing: -0.02em;
  font-variant-numeric: tabular-nums;
}

.Slot {
  position: relative;
  height: 1.25em;
  width: 0.62em;
  overflow: hidden;
  -webkit-mask-image: linear-gradient(
    to bottom,
    rgba(0, 0, 0, 0) 0%,
    rgba(0, 0, 0, 1) 11%,
    rgba(0, 0, 0, 1) 89%,
    rgba(0, 0, 0, 0) 100%
  );
  mask-image: linear-gradient(
    to bottom,
    rgba(0, 0, 0, 0) 0%,
    rgba(0, 0, 0, 1) 11%,
    rgba(0, 0, 0, 1) 89%,
    rgba(0, 0, 0, 0) 100%
  );
  -webkit-mask-size: 100% 100%;
  mask-size: 100% 100%;
  -webkit-mask-repeat: no-repeat;
  mask-repeat: no-repeat;
}

.Reel {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  will-change: transform;
  transition: transform 500ms cubic-bezier(0.22, 0.61, 0.36, 1);
}

.Cell {
  display: flex;
  height: 2em;
  flex-shrink: 0;
  flex-grow: 0;
  align-items: center;
  justify-content: center;
}

.Colon {
  display: flex;
  height: 1.25em;
  width: 0.45em;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 0.18em;
}

.Dot {
  height: 0.12em;
  width: 0.12em;
  border-radius: 9999px;
  background: currentColor;
}
</style>
