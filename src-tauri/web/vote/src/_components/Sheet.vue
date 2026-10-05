<script setup lang="ts">
import { ref, watch, nextTick, onUnmounted, type Ref } from "vue"

const Props = defineProps<{
  Open: boolean
  Label: string
  Dismissable?: boolean
}>()

const Emit = defineEmits<{ Close: [] }>()

const Panel: Ref<HTMLElement | null> = ref(null)
let _Returning: HTMLElement | null = null

const _Focusable = "button:not([disabled]), a[href], input:not([disabled]), [tabindex]:not([tabindex='-1'])"

function _Close(): void {
  if (Props.Dismissable === false) return
  Emit("Close")
}

function _Trap(Event: KeyboardEvent): void {
  if (Event.key === "Escape") {
    Event.preventDefault()
    _Close()
    return
  }

  if (Event.key !== "Tab" || !Panel.value) return

  const Items = Array.from(Panel.value.querySelectorAll<HTMLElement>(_Focusable))
  if (!Items.length) return

  const First = Items[0]
  const Last = Items[Items.length - 1]

  if (Event.shiftKey && document.activeElement === First) {
    Event.preventDefault()
    Last.focus()
  } else if (!Event.shiftKey && document.activeElement === Last) {
    Event.preventDefault()
    First.focus()
  }
}

function _Lock(Locked: boolean): void {
  document.documentElement.style.overflow = Locked ? "hidden" : ""
}

watch(() => Props.Open, async (Now) => {
  if (Now) {
    _Returning = document.activeElement instanceof HTMLElement ? document.activeElement : null
    _Lock(true)
    document.addEventListener("keydown", _Trap)
    await nextTick()
    Panel.value?.focus()
    return
  }

  _Lock(false)
  document.removeEventListener("keydown", _Trap)
  _Returning?.focus?.()
  _Returning = null
}, { immediate: true })

onUnmounted(() => {
  _Lock(false)
  document.removeEventListener("keydown", _Trap)
})
</script>

<template>
  <Teleport to="body">
    <Transition name="sheet">
      <div v-if="Open" class="Layer">
        <div class="Backdrop" @click="_Close" />

        <div
          ref="Panel"
          class="Panel"
          role="dialog"
          aria-modal="true"
          :aria-label="Label"
          tabindex="-1"
        >
          <div class="Grip" aria-hidden="true" />
          <button
            v-if="Dismissable !== false"
            type="button"
            class="Dismiss"
            aria-label="Zamknij"
            @click="_Close"
          >
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round">
              <path d="M18 6 6 18M6 6l12 12" />
            </svg>
          </button>

          <slot />
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.Layer {
  position: fixed;
  inset: 0;
  z-index: 50;
  display: flex;
  align-items: flex-end;
  justify-content: center;
}

.Backdrop {
  position: absolute;
  inset: 0;
  background: rgba(3, 3, 5, 0.66);
  backdrop-filter: blur(6px);
  -webkit-backdrop-filter: blur(6px);
}

.Panel {
  position: relative;
  width: 100%;
  max-width: 520px;
  max-height: calc(100dvh - 24px);
  overflow-y: auto;
  overscroll-behavior: contain;
  border-radius: 28px 28px 0 0;
  background: linear-gradient(180deg, rgba(28, 26, 32, 0.96), rgba(14, 13, 17, 0.98));
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.08),
    inset 0 0 0 1px rgba(255, 255, 255, 0.06),
    0 -30px 80px -20px rgba(0, 0, 0, 0.8);
  padding: 14px 22px calc(26px + env(safe-area-inset-bottom));
  outline: none;
}

@media (min-width: 640px) {
  .Layer {
    align-items: center;
    padding: 24px;
  }

  .Panel {
    border-radius: 28px;
    padding-bottom: 28px;
  }

  .Grip {
    visibility: hidden;
  }
}

.Grip {
  margin: 0 auto 8px;
  height: 4px;
  width: 38px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.14);
}

.Dismiss {
  position: absolute;
  top: 10px;
  right: 10px;
  display: flex;
  height: 44px;
  width: 44px;
  align-items: center;
  justify-content: center;
  border-radius: 14px;
  color: rgba(255, 255, 255, 0.4);
  transition: color 160ms ease, background 160ms ease;
}

.Dismiss:hover {
  color: rgba(255, 255, 255, 0.8);
  background: rgba(255, 255, 255, 0.06);
}

.Dismiss svg {
  height: 18px;
  width: 18px;
}

.sheet-enter-active,
.sheet-leave-active {
  transition: opacity 260ms ease;
}

.sheet-enter-active .Panel,
.sheet-leave-active .Panel {
  transition: transform 460ms var(--ease-smooth);
}

.sheet-enter-from,
.sheet-leave-to {
  opacity: 0;
}

.sheet-enter-from .Panel,
.sheet-leave-to .Panel {
  transform: translateY(40px);
}
</style>
