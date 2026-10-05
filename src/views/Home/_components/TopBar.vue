<script setup lang="ts">
defineProps<{
  Notice: string
  Pages: string[]
  Active: number
}>()
</script>

<template>
  <div class="TopBar">
    <div class="Brand">
      <img src="/fm-512.png" alt="" class="Mark" />
    </div>

    <Transition name="notice">
      <div v-if="Notice" class="Notice">
        <p class="Text">{{ Notice }}</p>
      </div>
    </Transition>

    <Transition name="dots">
      <TransitionGroup v-if="Pages.length > 1" tag="div" name="dot" class="Dots">
        <span
          v-for="(Page, Index) in Pages"
          :key="Page"
          class="Dot"
          :class="{ On: Index === Active }"
        />
      </TransitionGroup>
    </Transition>
  </div>
</template>

<style scoped>
.TopBar {
  position: absolute;
  top: 52px;
  right: 72px;
  left: 72px;
  z-index: 20;
  display: grid;
  grid-template-columns: minmax(120px, 1fr) minmax(0, auto) minmax(120px, 1fr);
  align-items: center;
  gap: 28px;
}

.Brand {
  grid-column: 1;
  justify-self: start;
  display: flex;
  flex-shrink: 0;
  align-items: center;
}

.Mark {
  height: 56px;
  width: 56px;
  object-fit: contain;
  opacity: 0.85;
}

.Notice {
  grid-column: 2;
  display: flex;
  min-width: 0;
  align-items: center;
  justify-content: center;
  border-radius: 999px;
  padding: 15px 32px;
  background: rgba(255, 255, 255, 0.05);
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.07);
  backdrop-filter: blur(20px) saturate(1.3);
}

.Text {
  min-width: 0;
  overflow: hidden;
  font-family: var(--font-clock, "Inter", system-ui, sans-serif);
  font-size: 23px;
  font-weight: 500;
  line-height: 1.2;
  letter-spacing: -0.01em;
  color: rgba(255, 255, 255, 0.9);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.Dots {
  grid-column: 3;
  justify-self: end;
  display: flex;
  flex-shrink: 0;
  align-items: center;
  gap: 10px;
}

.Dot {
  position: relative;
  height: 8px;
  width: 8px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.22);
  transition: width 420ms var(--ease-smooth, cubic-bezier(0.16, 1, 0.3, 1)), background 420ms ease;
}

.Dot.On {
  width: 28px;
  background: rgba(255, 255, 255, 0.85);
}

.dot-enter-active {
  transition:
    width 900ms cubic-bezier(0.2, 0.9, 0.3, 1),
    margin 900ms cubic-bezier(0.2, 0.9, 0.3, 1),
    opacity 500ms ease 120ms,
    transform 900ms cubic-bezier(0.3, 1.5, 0.5, 1);
}

.dot-enter-active::after {
  content: "";
  position: absolute;
  inset: -6px;
  border-radius: 999px;
  box-shadow: 0 0 0 1px rgba(255, 255, 255, 0.5);
  animation: dot-halo 700ms cubic-bezier(0.2, 0.9, 0.3, 1) 200ms both;
}

@keyframes dot-halo {
  0% { opacity: 0; transform: scale(0.4); }
  35% { opacity: 1; }
  100% { opacity: 0; transform: scale(1.6); }
}

.dot-leave-active {
  transition:
    width 600ms cubic-bezier(0.4, 0, 0.2, 1),
    margin 600ms cubic-bezier(0.4, 0, 0.2, 1),
    opacity 300ms ease,
    transform 600ms cubic-bezier(0.4, 0, 0.2, 1);
}

.dot-enter-from,
.dot-leave-to {
  width: 0;
  margin-left: -10px;
  opacity: 0;
  transform: scale(0.2);
}

.dot-move {
  transition: transform 600ms cubic-bezier(0.2, 0.9, 0.3, 1);
}

.dots-enter-active,
.dots-leave-active {
  transition: opacity 500ms ease, transform 700ms cubic-bezier(0.2, 0.9, 0.3, 1);
}

.dots-enter-from,
.dots-leave-to {
  opacity: 0;
  transform: translateX(14px);
}

.notice-enter-active,
.notice-leave-active {
  transition: opacity 500ms ease, transform 620ms cubic-bezier(0.2, 0.9, 0.3, 1);
}

.notice-enter-from,
.notice-leave-to {
  opacity: 0;
  transform: translateY(-14px);
}
</style>
