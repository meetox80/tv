<script setup lang="ts">
type IconName = "tv" | "key" | "globe" | "lock" | "check" | "alert" | "close" | "external"

const Props = defineProps<{
  Name: IconName
}>()

const _Paths: Record<IconName, string[]> = {
  tv: [
    "M5.5 4h13A2.5 2.5 0 0 1 21 6.5v7a2.5 2.5 0 0 1-2.5 2.5h-13A2.5 2.5 0 0 1 3 13.5v-7A2.5 2.5 0 0 1 5.5 4z",
    "M8.5 20h7",
    "M12 16v4",
  ],
  key: [
    "M12 15a4 4 0 1 1-8 0a4 4 0 0 1 8 0z",
    "M10.8 12.2 20 3",
    "m15.5 7.5 2 2",
    "m18 5 2 2",
  ],
  globe: [
    "M21 12a9 9 0 1 1-18 0a9 9 0 0 1 18 0z",
    "M3 12h18",
    "M12 3c2.5 2.6 3.8 5.6 3.8 9s-1.3 6.4-3.8 9c-2.5-2.6-3.8-5.6-3.8-9s1.3-6.4 3.8-9z",
  ],
  lock: [
    "M7 10.5h10a2.5 2.5 0 0 1 2.5 2.5v5.5A2.5 2.5 0 0 1 17 21H7a2.5 2.5 0 0 1-2.5-2.5V13A2.5 2.5 0 0 1 7 10.5z",
    "M8 10.5V7.5a4 4 0 0 1 8 0v3",
  ],
  check: ["m5 12.5 4.5 4.5L19 7.5"],
  alert: ["M12 6.5v7", "M12 17.5h.01"],
  close: ["M7 7l10 10", "M17 7 7 17"],
  external: [
    "M14 4h6v6",
    "M20 4l-9 9",
    "M18 14v4.5a1.5 1.5 0 0 1-1.5 1.5h-11A1.5 1.5 0 0 1 4 18.5v-11A1.5 1.5 0 0 1 5.5 6H10",
  ],
}
</script>

<template>
  <svg
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="1.8"
    stroke-linecap="round"
    stroke-linejoin="round"
    aria-hidden="true"
  >
    <path v-for="D in _Paths[Props.Name]" :key="D" :d="D" />
  </svg>
</template>
