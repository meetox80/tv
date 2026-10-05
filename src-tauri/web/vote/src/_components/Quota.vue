<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, type Ref } from "vue"
import { Device, Left, Limit, RefreshState, ResetAt } from "../_state/store"
import { Plural } from "../_api/format"

const _TickMs = 15000

const Now: Ref<number> = ref(Date.now())

let _Timer: ReturnType<typeof setInterval> | null = null
let _Refreshing = false

const Text = computed(() => {
  if (!Device.value) return `${Limit.value} ${Plural(Limit.value, "propozycja", "propozycje", "propozycji")} dziennie`
  if (Left.value === 0) return "Na dziś to wszystko"
  const Verb = Plural(Left.value, "Została", "Zostały", "Zostało")
  return `${Verb} ${Left.value} ${Plural(Left.value, "propozycja", "propozycje", "propozycji")} na dziś`
})

const Countdown = computed(() => {
  if (!Device.value || ResetAt.value === null) return ""
  const Minutes = Math.max(1, Math.ceil((ResetAt.value - Now.value) / 60000))
  const Hours = Math.floor(Minutes / 60)
  const Rest = Minutes % 60
  return Hours > 0 ? `${Hours}h ${Rest}m` : `${Rest}m`
})

function _Tick(): void {
  Now.value = Date.now()
  if (ResetAt.value === null || Now.value < ResetAt.value || _Refreshing) return
  _Refreshing = true
  void RefreshState().finally(() => {
    _Refreshing = false
  })
}

onMounted(() => {
  _Timer = setInterval(_Tick, _TickMs)
})

onUnmounted(() => {
  if (_Timer) clearInterval(_Timer)
})
</script>

<template>
  <div class="Quota" role="status" :title="Text">
    <span v-if="Countdown" class="Reset">{{ Countdown }} do resetu</span>
    <span class="Pips" aria-hidden="true">
      <span v-for="I in Limit" :key="I" class="Pip" :class="{ On: I <= Left }" />
    </span>
    <span class="SrOnly">{{ Text }}</span>
  </div>
</template>

<style scoped>
.Quota {
  display: inline-flex;
  align-items: center;
  gap: 12px;
  padding: 8px 0;
}

.Reset {
  font-size: 13px;
  font-variant-numeric: tabular-nums;
  color: var(--color-ink-mute);
  white-space: nowrap;
}

.Pips {
  display: inline-flex;
  gap: 5px;
}

.Pip {
  height: 7px;
  width: 7px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.14);
  transition: background 400ms ease, width 400ms var(--ease-smooth);
}

.Pip.On {
  width: 18px;
  background: var(--color-rose);
}
</style>
