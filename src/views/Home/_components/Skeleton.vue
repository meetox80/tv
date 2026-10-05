<template>
  <div class="Skeleton">
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
</template>

<style scoped>
.Skeleton {
  position: absolute;
  inset: 0;
  z-index: 10;
  pointer-events: none;
}

.Block {
  position: absolute;
  border-radius: 18px;
  background: rgba(255, 255, 255, 0.05);
  animation: Pulse 1500ms ease-in-out infinite;
}

.Logo { top: 53px; left: 72px; height: 56px; width: 56px; border-radius: 16px; }
.Notice { top: 52px; left: 660px; height: 58px; width: 600px; border-radius: 999px; animation-delay: 80ms; }
.Dots { top: 77px; right: 72px; height: 8px; width: 64px; border-radius: 999px; animation-delay: 120ms; }
.Page { top: 168px; right: 72px; bottom: 214px; left: 72px; border-radius: 28px; background: rgba(255, 255, 255, 0.03); animation-delay: 160ms; }
.Time { bottom: 98px; left: 72px; height: 76px; width: 210px; animation-delay: 200ms; }
.Meta { bottom: 62px; left: 74px; height: 22px; width: 340px; border-radius: 999px; animation-delay: 240ms; }
.Title { bottom: 106px; right: 190px; height: 34px; width: 300px; border-radius: 12px; animation-delay: 200ms; }
.Artist { bottom: 74px; right: 190px; height: 25px; width: 200px; border-radius: 10px; animation-delay: 240ms; }
.Sleeve { bottom: 62px; right: 72px; height: 92px; width: 92px; border-radius: 14px; animation-delay: 280ms; }

@keyframes Pulse {
  50% { opacity: 0.45; }
}

.reveal-leave-active .Block {
  animation: Sink 420ms ease both;
}

.reveal-leave-active .Page { animation-delay: 90ms; }

.reveal-leave-active .Time,
.reveal-leave-active .Meta,
.reveal-leave-active .Title,
.reveal-leave-active .Artist,
.reveal-leave-active .Sleeve { animation-delay: 180ms; }

.reveal-leave-active .Logo,
.reveal-leave-active .Notice,
.reveal-leave-active .Dots { animation-delay: 0ms; }

@keyframes Sink {
  to {
    opacity: 0;
    transform: scale(0.985);
  }
}

@media (prefers-reduced-motion: reduce) {
  .Block {
    animation: none;
  }
}
</style>
