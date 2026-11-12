<template>
  <svg class="activity" viewBox="0 0 24 24" aria-hidden="true">
    <line
      v-for="Spoke in 8"
      :key="Spoke"
      x1="12"
      y1="2.75"
      x2="12"
      y2="6.75"
      stroke="currentColor"
      stroke-width="2.1"
      stroke-linecap="round"
      :opacity="1 - (Spoke - 1) * 0.11"
      :transform="`rotate(${-(Spoke - 1) * 45} 12 12)`"
    />
  </svg>
</template>

<style scoped>
.activity {
  animation: activity-spin 0.85s steps(8) infinite;
}

@keyframes activity-spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
