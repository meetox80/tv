<script setup lang="ts">
import { onMounted, onUnmounted } from "vue"

const Props = withDefaults(defineProps<{ Open: boolean; Title: string; Message?: string; Layer?: number; Width?: number }>(), {
  Message: "",
  Layer: 60,
  Width: 440,
})

const Emit = defineEmits<{ Close: []; Closed: [] }>()

function _OnKey(E: KeyboardEvent): void {
  if (E.key !== "Escape" || !Props.Open) return
  E.preventDefault()
  Emit("Close")
}

onMounted(() => document.addEventListener("keydown", _OnKey))
onUnmounted(() => document.removeEventListener("keydown", _OnKey))
</script>

<template>
  <Teleport to="body">
    <Transition name="sheet" @after-leave="Emit('Closed')">
      <div v-if="Open" class="Layer" :style="{ zIndex: Layer }">
        <div class="Backdrop" @click="Emit('Close')" />

        <div class="Panel" role="dialog" aria-modal="true" :aria-label="Title" :style="{ maxWidth: `${Width}px` }">
          <div class="Grip" aria-hidden="true" />
          <p class="Title">{{ Title }}</p>
          <p v-if="Message" class="Message">{{ Message }}</p>
          <div v-if="$slots.default" class="Body"><slot /></div>
          <div class="Actions"><slot name="actions" /></div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.Layer {
  position: fixed;
  inset: 0;
  z-index: 60;
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
  display: flex;
  width: 100%;
  max-width: 440px;
  flex-direction: column;
  border-radius: 28px 28px 0 0;
  background: linear-gradient(180deg, rgba(28, 26, 32, 0.96), rgba(14, 13, 17, 0.98));
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.08),
    inset 0 0 0 1px rgba(255, 255, 255, 0.06),
    0 -30px 80px -20px rgba(0, 0, 0, 0.8);
  padding: 14px 24px calc(24px + env(safe-area-inset-bottom));
}

.Grip {
  margin: 0 auto 16px;
  height: 4px;
  width: 38px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.14);
}

.Title {
  margin: 0;
  font-family: var(--display);
  font-size: 24px;
  font-weight: 600;
  line-height: 1.15;
  letter-spacing: -0.03em;
  color: var(--ink);
}

.Message {
  margin: 10px 0 0;
  font-size: 14px;
  line-height: 1.55;
  color: var(--ink-mute);
}

.Body {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin-top: 20px;
}

.Actions {
  display: flex;
  gap: 10px;
  margin-top: 24px;
}

.Actions :deep(.Btn) {
  height: 48px;
  flex: 1;
  font-size: 15px;
}

.sheet-enter-active,
.sheet-leave-active {
  transition: opacity 260ms ease;
}

.sheet-enter-active .Panel,
.sheet-leave-active .Panel {
  transition: transform 460ms var(--ease);
}

.sheet-enter-from,
.sheet-leave-to {
  opacity: 0;
}

.sheet-enter-from .Panel,
.sheet-leave-to .Panel {
  transform: translateY(40px);
}

@media (min-width: 640px) {
  .Layer {
    align-items: center;
    padding: 24px;
  }

  .Panel {
    border-radius: 28px;
    padding: 28px;
  }

  .Grip {
    display: none;
  }
}
</style>
