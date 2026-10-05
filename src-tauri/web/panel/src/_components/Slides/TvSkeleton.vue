<script setup lang="ts">
defineProps<{ Scale: number }>()
</script>

<template>
  <div class="Skeleton" :style="{ width: `${1920 * Scale}px`, height: `${1080 * Scale}px` }" aria-label="Wczytywanie podglądu" role="status">
    <div class="Screen" :style="{ transform: `scale(${Scale})` }">
      <span class="Block Logo" />
      <span class="Block Notice" />
      <span class="Block Dots" />
      <span class="Block Page" />
      <span class="Block Time" />
      <span class="Block Meta" />
      <span class="Block Title" />
      <span class="Block Artist" />
      <span class="Block Sleeve" />
    </div>
  </div>
</template>

<style scoped>
.Skeleton {
  position: absolute;
  top: 0;
  left: 0;
  z-index: 40;
  overflow: hidden;
  border-radius: 10px;
  background: #07070a;
  pointer-events: none;
}

.Screen {
  position: absolute;
  top: 0;
  left: 0;
  height: 1080px;
  width: 1920px;
  transform-origin: 0 0;
}

.Block {
  position: absolute;
  border-radius: 18px;
  background: rgba(255, 255, 255, 0.05);
  animation: Pulse 1500ms ease-in-out infinite;
}

.Logo { top: 52px; left: 72px; height: 64px; width: 64px; border-radius: 16px; }
.Notice { top: 60px; left: 660px; height: 48px; width: 600px; border-radius: 999px; animation-delay: 80ms; }
.Dots { top: 76px; right: 72px; height: 16px; width: 120px; border-radius: 999px; animation-delay: 120ms; }
.Page { top: 168px; left: 72px; height: 698px; width: 1776px; border-radius: 28px; background: rgba(255, 255, 255, 0.03); animation-delay: 160ms; }
.Time { bottom: 118px; left: 72px; height: 76px; width: 230px; animation-delay: 200ms; }
.Meta { bottom: 62px; left: 74px; height: 24px; width: 340px; border-radius: 999px; animation-delay: 240ms; }
.Title { bottom: 124px; right: 222px; height: 34px; width: 300px; border-radius: 12px; animation-delay: 200ms; }
.Artist { bottom: 82px; right: 222px; height: 26px; width: 200px; border-radius: 10px; animation-delay: 240ms; }
.Sleeve { bottom: 62px; right: 72px; height: 124px; width: 124px; border-radius: 20px; animation-delay: 280ms; }

@keyframes Pulse {
  50% { opacity: 0.45; }
}

@media (prefers-reduced-motion: reduce) {
  .Block {
    animation: none;
  }
}
</style>
