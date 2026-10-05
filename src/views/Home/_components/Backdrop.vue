<script setup lang="ts">
defineProps<{ Cover: string | null }>()
</script>

<template>
  <div class="Ambient">
    <Transition name="field">
      <div v-if="Cover" :key="Cover" class="Field">
        <div class="Bloom">
          <div class="Haze">
            <img :src="Cover" alt="" class="Art" />
          </div>
        </div>

        <div class="Bloom Counter">
          <div class="Haze">
            <img :src="Cover" alt="" class="Art" />
          </div>
        </div>
      </div>
    </Transition>

    <div class="Veil" />
    <div class="Grain" />
  </div>
</template>

<style scoped>
.Ambient {
  position: absolute;
  inset: 0;
  pointer-events: none;
}

.Field {
  position: absolute;
  inset: 0;
  overflow: hidden;
}

.Bloom {
  position: absolute;
  right: -16%;
  bottom: -42%;
  height: 1000px;
  width: 1000px;
  will-change: transform;
  animation: drift 128s ease-in-out infinite alternate;
}

.Haze {
  height: 100%;
  width: 100%;
  filter: blur(86px) saturate(1.3) brightness(0.84);
}

.Art {
  height: 100%;
  width: 100%;
  object-fit: cover;
  opacity: 0.7;
  -webkit-mask-image: radial-gradient(closest-side, #000 0%, rgba(0, 0, 0, 0.52) 44%, rgba(0, 0, 0, 0.12) 76%, transparent 100%);
  mask-image: radial-gradient(closest-side, #000 0%, rgba(0, 0, 0, 0.52) 44%, rgba(0, 0, 0, 0.12) 76%, transparent 100%);
}

.Counter {
  top: -44%;
  right: auto;
  bottom: auto;
  left: -12%;
  height: 920px;
  width: 920px;
  animation: counter-drift 186s ease-in-out infinite alternate;
}

.Counter .Haze {
  filter: blur(94px) saturate(1.16) brightness(0.7);
}

.Counter .Art {
  opacity: 0.42;
}

@keyframes drift {
  from {
    transform: scale(1.02) translate(-1.6%, -1.1%);
  }
  to {
    transform: scale(1.16) translate(1.4%, 1.6%);
  }
}

@keyframes counter-drift {
  from {
    transform: scale(1.14) rotate(180deg) translate(1.3%, 0.9%);
  }
  to {
    transform: scale(1) rotate(150deg) translate(-1.4%, -1.3%);
  }
}

.Veil {
  position: absolute;
  inset: 0;
  background: linear-gradient(
    to bottom,
    rgba(5, 5, 8, 0.7) 0%,
    rgba(5, 5, 8, 0.2) 17%,
    rgba(5, 5, 8, 0.14) 50%,
    rgba(5, 5, 8, 0.5) 84%,
    rgba(5, 5, 8, 0.82) 100%
  );
  box-shadow: inset 0 0 520px 210px rgba(5, 5, 8, 0.4);
}

.Grain {
  position: absolute;
  inset: 0;
  opacity: 0.055;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='200' height='200'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='4' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='200' height='200' filter='url(%23n)'/%3E%3C/svg%3E");
}

.field-enter-active,
.field-leave-active {
  transition: opacity 1100ms ease;
}

.field-enter-from,
.field-leave-to {
  opacity: 0;
}
</style>
