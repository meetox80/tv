<script setup lang="ts">
defineProps<{ Title: string; Text: string }>()
</script>

<template>
  <div class="Blank">
    <div class="Stack" aria-hidden="true">
      <span class="Tile TileLeft" />
      <span class="Tile TileRight" />
      <span class="Tile TileFront">
        <slot name="front">
          <img src="/zstio-fm.png" alt="" class="TileLogo" />
        </slot>
      </span>
    </div>

    <h2 class="Title">{{ Title }}</h2>
    <p class="Text">{{ Text }}</p>
    <div v-if="$slots.default" class="Actions"><slot /></div>
  </div>
</template>

<style scoped>
.Blank {
  display: flex;
  min-height: 62dvh;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 24px 8px 8px;
  text-align: center;
}

.Stack {
  position: relative;
  height: 150px;
  width: 150px;
  margin-bottom: 44px;
}

.Tile {
  position: absolute;
  inset: 0;
  border-radius: 30px;
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.18),
    inset 0 0 0 1px rgba(255, 255, 255, 0.08),
    0 30px 60px -24px rgba(0, 0, 0, 0.85);
  animation: Fan 1100ms var(--ease-smooth) 120ms both;
}

.TileLeft {
  background: linear-gradient(150deg, #4a2338, #1c1119);
  --To: translateX(-58%) translateY(8%) rotate(-13deg) scale(0.86);
}

.TileRight {
  background: linear-gradient(150deg, #33294d, #15121e);
  --To: translateX(58%) translateY(8%) rotate(13deg) scale(0.86);
}

.TileFront {
  display: flex;
  align-items: center;
  justify-content: center;
  background:
    radial-gradient(120% 90% at 30% 0%, rgba(255, 255, 255, 0.28), transparent 55%),
    linear-gradient(160deg, #d98d8d, #7a1f1f 75%);
  color: rgba(255, 255, 255, 0.95);
  --To: translateY(0) rotate(0deg) scale(1);
}

.TileLogo {
  height: 64%;
  width: 64%;
  object-fit: contain;
  filter: brightness(1.2);
}

@keyframes Fan {
  from {
    opacity: 0;
    transform: translateY(18px) scale(0.9);
  }
  to {
    opacity: 1;
    transform: var(--To);
  }
}

.Title {
  font-family: var(--font-display);
  font-size: 30px;
  font-weight: 600;
  line-height: 1.1;
  letter-spacing: -0.035em;
  color: var(--color-ink);
  text-wrap: balance;
}

.Text {
  margin-top: 10px;
  max-width: 320px;
  font-size: 16px;
  line-height: 1.5;
  color: var(--color-ink-mute);
  text-wrap: balance;
}

.Actions {
  margin-top: 28px;
}

.Actions :deep(button) {
  min-height: 48px;
  border-radius: 999px;
  background: var(--color-ink);
  padding: 0 22px;
  font-size: 14px;
  font-weight: 600;
  color: #0b0a0d;
  transition: transform 180ms var(--ease-smooth);
}

.Actions :deep(button:active) {
  transform: scale(0.97);
}

@media (min-width: 900px) {
  .Blank {
    min-height: 58vh;
  }

  .Stack {
    height: 180px;
    width: 180px;
    margin-bottom: 52px;
  }

  .Tile {
    border-radius: 36px;
  }

  .Title {
    font-size: 44px;
  }

  .Text {
    max-width: 460px;
    font-size: 17px;
  }
}
</style>
