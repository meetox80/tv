<script setup lang="ts">
export type IconName =
  | "text" | "rect" | "ellipse" | "image"
  | "radius" | "corners" | "rotate" | "opacity"
  | "front" | "forward" | "backward" | "back"
  | "align-left" | "align-center" | "align-right" | "align-top" | "align-middle" | "align-bottom"
  | "distribute-h" | "distribute-v"
  | "text-left" | "text-center" | "text-right"
  | "duplicate" | "trash" | "eye" | "eye-off" | "lock" | "unlock" | "chevron" | "layers" | "grip"
  | "plus" | "minus" | "sliders" | "copy" | "paste" | "pencil" | "multi" | "slides" | "close"

defineProps<{ Name: IconName }>()
</script>

<template>
  <svg class="Icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <path v-if="Name === 'text'" d="M5 6.5V5h14v1.5M12 5v14M9 19h6" />
    <rect v-else-if="Name === 'rect'" x="4" y="6" width="16" height="12" rx="2.5" />
    <circle v-else-if="Name === 'ellipse'" cx="12" cy="12" r="7.5" />
    <template v-else-if="Name === 'image'">
      <rect x="3.5" y="5" width="17" height="14" rx="2.5" />
      <circle cx="9" cy="10" r="1.6" />
      <path d="m20.5 16-5-5-8.5 8" />
    </template>
    <path v-else-if="Name === 'radius'" d="M5 19v-6a8 8 0 0 1 8-8h6" />
    <path v-else-if="Name === 'corners'" d="M4 9V7a3 3 0 0 1 3-3h2M15 4h2a3 3 0 0 1 3 3v2M20 15v2a3 3 0 0 1-3 3h-2M9 20H7a3 3 0 0 1-3-3v-2" />
    <template v-else-if="Name === 'rotate'">
      <path d="M20 12a8 8 0 1 1-8-8c2.24 0 4.38.89 5.99 2.44L20 8.5" />
      <path d="M20 3.5v5h-5" />
    </template>
    <path v-else-if="Name === 'opacity'" d="M12 3.5c3 3.6 5.5 6.6 5.5 9.6a5.5 5.5 0 0 1-11 0c0-3 2.5-6 5.5-9.6Z" />
    <template v-else-if="Name === 'front'">
      <path d="M5 4h14" />
      <path d="M12 20V8m-4.5 4.5L12 8l4.5 4.5" />
    </template>
    <path v-else-if="Name === 'forward'" d="M12 19V6m-5 5 5-5 5 5" />
    <path v-else-if="Name === 'backward'" d="M12 5v13m-5-5 5 5 5-5" />
    <template v-else-if="Name === 'back'">
      <path d="M5 20h14" />
      <path d="M12 4v12m-4.5-4.5L12 16l4.5-4.5" />
    </template>
    <template v-else-if="Name === 'align-left'">
      <path d="M4 3.5v17" />
      <rect x="7" y="6" width="12" height="4" rx="1.2" />
      <rect x="7" y="14" width="7" height="4" rx="1.2" />
    </template>
    <template v-else-if="Name === 'align-center'">
      <path d="M12 3.5v17" />
      <rect x="5" y="6" width="14" height="4" rx="1.2" />
      <rect x="7.5" y="14" width="9" height="4" rx="1.2" />
    </template>
    <template v-else-if="Name === 'align-right'">
      <path d="M20 3.5v17" />
      <rect x="5" y="6" width="12" height="4" rx="1.2" />
      <rect x="10" y="14" width="7" height="4" rx="1.2" />
    </template>
    <template v-else-if="Name === 'align-top'">
      <path d="M3.5 4h17" />
      <rect x="6" y="7" width="4" height="12" rx="1.2" />
      <rect x="14" y="7" width="4" height="7" rx="1.2" />
    </template>
    <template v-else-if="Name === 'align-middle'">
      <path d="M3.5 12h17" />
      <rect x="6" y="5" width="4" height="14" rx="1.2" />
      <rect x="14" y="7.5" width="4" height="9" rx="1.2" />
    </template>
    <template v-else-if="Name === 'align-bottom'">
      <path d="M3.5 20h17" />
      <rect x="6" y="5" width="4" height="12" rx="1.2" />
      <rect x="14" y="10" width="4" height="7" rx="1.2" />
    </template>
    <template v-else-if="Name === 'distribute-h'">
      <path d="M4 4v16M20 4v16" />
      <rect x="9.5" y="7" width="5" height="10" rx="1.2" />
    </template>
    <template v-else-if="Name === 'distribute-v'">
      <path d="M4 4h16M4 20h16" />
      <rect x="7" y="9.5" width="10" height="5" rx="1.2" />
    </template>
    <path v-else-if="Name === 'text-left'" d="M4 6h16M4 10h10M4 14h16M4 18h10" />
    <path v-else-if="Name === 'text-center'" d="M4 6h16M7 10h10M4 14h16M7 18h10" />
    <path v-else-if="Name === 'text-right'" d="M4 6h16M10 10h10M4 14h16M10 18h10" />
    <template v-else-if="Name === 'duplicate'">
      <rect x="8" y="8" width="12" height="12" rx="2.5" />
      <path d="M16 8V6a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v8a2 2 0 0 0 2 2h2" />
    </template>
    <path v-else-if="Name === 'trash'" d="M4 7h16M9 7V4.5h6V7M6.5 7l1 12.5h9l1-12.5" />
    <template v-else-if="Name === 'eye'">
      <path d="M2.5 12S6 5.5 12 5.5 21.5 12 21.5 12 18 18.5 12 18.5 2.5 12 2.5 12Z" />
      <circle cx="12" cy="12" r="2.8" />
    </template>
    <template v-else-if="Name === 'eye-off'">
      <path d="M10.2 5.7A9.6 9.6 0 0 1 12 5.5c6 0 9.5 6.5 9.5 6.5a16 16 0 0 1-2.7 3.4M6.3 7.6C3.9 9.3 2.5 12 2.5 12S6 18.5 12 18.5c1.6 0 3-.4 4.2-1" />
      <path d="M4 4l16 16" />
    </template>
    <template v-else-if="Name === 'lock'">
      <rect x="5" y="10.5" width="14" height="9.5" rx="2.2" />
      <path d="M8 10.5V8a4 4 0 0 1 8 0v2.5" />
    </template>
    <template v-else-if="Name === 'unlock'">
      <rect x="5" y="10.5" width="14" height="9.5" rx="2.2" />
      <path d="M8 10.5V8a4 4 0 0 1 7.6-1.7" />
    </template>
    <path v-else-if="Name === 'chevron'" d="m7 10 5 5 5-5" />
    <template v-else-if="Name === 'layers'">
      <path d="m12 4 8.5 4.5L12 13 3.5 8.5 12 4Z" />
      <path d="m3.5 12.5 8.5 4.5 8.5-4.5" />
      <path d="m3.5 16.5 8.5 4.5 8.5-4.5" />
    </template>
    <path v-else-if="Name === 'plus'" d="M12 5v14M5 12h14" />
    <path v-else-if="Name === 'minus'" d="M5 12h14" />
    <template v-else-if="Name === 'sliders'">
      <path d="M4 7h9M17 7h3M4 17h3M11 17h9" />
      <circle cx="15" cy="7" r="2" />
      <circle cx="9" cy="17" r="2" />
    </template>
    <template v-else-if="Name === 'copy'">
      <rect x="8" y="8" width="12" height="12" rx="2.5" />
      <path d="M16 8V6a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v8a2 2 0 0 0 2 2h2" />
    </template>
    <template v-else-if="Name === 'paste'">
      <rect x="5" y="5" width="14" height="16" rx="2.5" />
      <path d="M9 3.5h6v3H9z" />
      <path d="M9 12h6M9 16h4" />
    </template>
    <path v-else-if="Name === 'pencil'" d="M4 20h4L19 9a2.8 2.8 0 0 0-4-4L4 16v4ZM13.5 6.5l4 4" />
    <template v-else-if="Name === 'multi'">
      <rect x="3.5" y="3.5" width="11" height="11" rx="2.5" />
      <path d="M18 9.5h.5a2 2 0 0 1 2 2v7a2 2 0 0 1-2 2h-7a2 2 0 0 1-2-2V18" />
      <path d="m6.5 9 2 2 3.5-4" />
    </template>
    <template v-else-if="Name === 'slides'">
      <rect x="3" y="6.5" width="14" height="11" rx="2.2" />
      <path d="M20.5 8.5v7" />
    </template>
    <path v-else-if="Name === 'close'" d="M18 6 6 18M6 6l12 12" />
    <template v-else-if="Name === 'grip'">
      <circle cx="9" cy="7" r="1" fill="currentColor" stroke="none" />
      <circle cx="15" cy="7" r="1" fill="currentColor" stroke="none" />
      <circle cx="9" cy="12" r="1" fill="currentColor" stroke="none" />
      <circle cx="15" cy="12" r="1" fill="currentColor" stroke="none" />
      <circle cx="9" cy="17" r="1" fill="currentColor" stroke="none" />
      <circle cx="15" cy="17" r="1" fill="currentColor" stroke="none" />
    </template>
  </svg>
</template>

<style scoped>
.Icon {
  display: block;
  height: 16px;
  width: 16px;
  flex-shrink: 0;
}
</style>
