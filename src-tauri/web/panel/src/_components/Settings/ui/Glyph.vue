<script setup lang="ts">
import type { GlyphName } from "../sections"

defineProps<{ Name: GlyphName }>()
</script>

<template>
  <svg
    class="Glyph"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="2"
    stroke-linecap="round"
    stroke-linejoin="round"
    aria-hidden="true"
  >
    <template v-if="Name === 'music'">
      <path d="M9 17.5V5.8l11-2.3v11.6" />
      <circle cx="6.2" cy="17.6" r="2.8" fill="currentColor" stroke="none" />
      <circle cx="17.2" cy="15.2" r="2.8" fill="currentColor" stroke="none" />
    </template>
    <template v-else-if="Name === 'display'">
      <rect x="2.5" y="4" width="19" height="12.5" rx="2.2" />
      <path d="M8.5 20.5h7M12 16.5v4" />
    </template>
    <template v-else-if="Name === 'vote'">
      <path d="m8.5 11.5 2.4 2.4 4.6-4.6" />
      <path d="M5 19.5V6.2A2.2 2.2 0 0 1 7.2 4h9.6A2.2 2.2 0 0 1 19 6.2v13.3" />
      <path d="M2.5 19.5h19" />
    </template>
    <template v-else-if="Name === 'tray'">
      <path d="M21.5 12.5h-5.2l-1.8 2.8h-5l-1.8-2.8H2.5" />
      <path d="M5.4 5.6 2.5 12.5v5.3a2.2 2.2 0 0 0 2.2 2.2h14.6a2.2 2.2 0 0 0 2.2-2.2v-5.3l-2.9-6.9A2.2 2.2 0 0 0 16.6 4H7.4a2.2 2.2 0 0 0-2 1.6Z" />
    </template>
    <template v-else-if="Name === 'book'">
      <path d="M2.5 4.5h5.5a4 4 0 0 1 4 4v12a3 3 0 0 0-3-3H2.5Z" />
      <path d="M21.5 4.5H16a4 4 0 0 0-4 4v12a3 3 0 0 1 3-3h6.5Z" />
    </template>
    <template v-else-if="Name === 'gear'">
      <circle cx="12" cy="12" r="3" />
      <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1Z" />
    </template>
    <template v-else-if="Name === 'pulse'">
      <path d="M2.5 12.5h4l3-8 5 15 3-7h4" />
    </template>
    <template v-else-if="Name === 'info'">
      <path d="M12 10.8V18" stroke-width="2.6" />
      <circle cx="12" cy="6.6" r="1.7" fill="currentColor" stroke="none" />
    </template>
    <template v-else-if="Name === 'weather'">
      <path d="M12 2.5v1.6M5.3 5.3l1.1 1.1M18.7 5.3l-1.1 1.1M21.5 11.5h-1.6" />
      <path d="M15.9 12.3a4 4 0 0 0-7.3-3.3" />
      <path d="M13.2 21.5H7a4.6 4.6 0 1 1 4.4-6h1.8a3 3 0 0 1 0 6Z" />
    </template>
    <template v-else-if="Name === 'clock'">
      <circle cx="12" cy="12" r="9" />
      <path d="M12 7.2V12l3.2 2" />
    </template>
    <template v-else-if="Name === 'megaphone'">
      <path d="M3.5 10.5 19.5 5v14l-16-4.5Z" />
      <path d="M11.2 16.8a3 3 0 0 1-5.7-1.6" />
    </template>
    <template v-else-if="Name === 'shield'">
      <path d="M12 21.5s7.5-3.6 7.5-9.4V5.6L12 2.8 4.5 5.6v6.5c0 5.8 7.5 9.4 7.5 9.4Z" />
      <path d="m9.2 12 2 2 3.7-3.8" />
    </template>
    <template v-else-if="Name === 'globe'">
      <circle cx="12" cy="12" r="9" />
      <path d="M3 12h18M12 3a13.8 13.8 0 0 1 3.6 9A13.8 13.8 0 0 1 12 21a13.8 13.8 0 0 1-3.6-9A13.8 13.8 0 0 1 12 3Z" />
    </template>
    <template v-else-if="Name === 'key'">
      <circle cx="7.5" cy="15.5" r="4.5" />
      <path d="m10.7 12.3 9.3-9.3M16.5 6.5l3 3M14 9l2.5 2.5" />
    </template>
    <template v-else-if="Name === 'lock'">
      <rect x="4.5" y="10.5" width="15" height="11" rx="2.4" />
      <path d="M8 10.5V7.4a4 4 0 0 1 8 0v3.1" />
    </template>
    <template v-else-if="Name === 'trash'">
      <path d="M3.5 6.5h17M18.5 6.5l-.9 12.7a2 2 0 0 1-2 1.8H8.4a2 2 0 0 1-2-1.8L5.5 6.5M9.5 6.5V4.2a1 1 0 0 1 1-1h3a1 1 0 0 1 1 1v2.3" />
    </template>
    <template v-else-if="Name === 'code'">
      <path d="m15.5 17.5 5.5-5.5-5.5-5.5M8.5 6.5 3 12l5.5 5.5" />
    </template>
    <template v-else-if="Name === 'slides'">
      <rect x="3" y="6.5" width="14" height="11" rx="2.2" />
      <path d="M20.5 8.5v7M7 10.5h6M7 13.5h3.5" />
    </template>
    <template v-else-if="Name === 'puzzle'">
      <path d="M19.44 7.85c-.05.32.06.65.29.88l1.57 1.57a2.4 2.4 0 0 1 0 3.4l-1.61 1.62a.98.98 0 0 1-.84.27c-.47-.07-.8-.48-.97-.92a2.5 2.5 0 1 0-3.21 3.21c.44.17.85.5.92.97a.98.98 0 0 1-.27.84l-1.61 1.61a2.4 2.4 0 0 1-3.41 0l-1.57-1.57a1.03 1.03 0 0 0-.88-.29c-.49.07-.84.5-1.02.97a2.5 2.5 0 1 1-3.24-3.24c.47-.18.9-.53.97-1.02a1.03 1.03 0 0 0-.29-.88l-1.57-1.57a2.4 2.4 0 0 1 0-3.4L4.23 8.77c.24-.24.58-.35.92-.3.51.08.88.53 1.07 1.01a2.5 2.5 0 1 0 3.26-3.26c-.48-.19-.93-.56-1.01-1.07-.05-.34.06-.68.3-.92l1.53-1.52a2.4 2.4 0 0 1 3.4 0l1.57 1.57c.23.23.56.34.88.29.49-.07.84-.5 1.02-.97a2.5 2.5 0 1 1 3.24 3.24c-.47.18-.9.53-.97 1.02Z" />
    </template>
    <template v-else-if="Name === 'plug'">
      <path d="M12 21.5v-4.5M9 8V2.5M15 8V2.5M18 8v4a6 6 0 0 1-12 0V8Z" />
    </template>
    <template v-else-if="Name === 'lastfm'">
      <path
        fill="currentColor"
        stroke="none"
        d="M10.584 17.21l-.88-2.392s-1.43 1.594-3.573 1.594c-1.897 0-3.244-1.649-3.244-4.288 0-3.382 1.704-4.591 3.381-4.591 2.422 0 3.19 1.567 3.849 3.574l.88 2.749c.88 2.666 2.529 4.81 7.285 4.81 3.409 0 5.718-1.044 5.718-3.793 0-2.227-1.265-3.381-3.63-3.932l-1.758-.385c-1.21-.275-1.567-.77-1.567-1.594 0-.935.742-1.484 1.952-1.484 1.32 0 2.034.495 2.144 1.677l2.749-.33c-.22-2.474-1.924-3.492-4.729-3.492-2.474 0-4.893.935-4.893 3.932 0 1.87.907 3.051 3.189 3.601l1.87.44c1.402.33 1.869.825 1.869 1.677 0 1.044-1.073 1.484-3.079 1.484-2.98 0-4.207-1.566-4.893-3.712l-.907-2.749c-1.155-3.574-3.052-4.866-6.544-4.866C1.841 5.544 0 8.24 0 12.471c0 4.015 1.841 6.544 6.049 6.544 3.079.027 4.535-1.072 4.535-1.072v-.742z"
      />
    </template>
    <template v-else-if="Name === 'spotify'">
      <path
        fill="currentColor"
        stroke="none"
        d="M12 0C5.4 0 0 5.4 0 12s5.4 12 12 12 12-5.4 12-12S18.66 0 12 0zm5.521 17.34c-.24.359-.66.48-1.021.24-2.82-1.74-6.36-2.101-10.561-1.141-.418.122-.779-.179-.899-.539-.12-.421.18-.78.54-.9 4.56-1.021 8.52-.6 11.64 1.32.42.18.479.659.301 1.02zm1.44-3.3c-.301.42-.841.6-1.262.3-3.239-1.98-8.159-2.58-11.939-1.38-.479.12-1.02-.12-1.14-.6-.12-.48.12-1.021.6-1.141C9.6 9.9 15 10.561 18.72 12.84c.361.181.54.78.241 1.2zm.12-3.36C15.24 8.4 8.82 8.16 5.16 9.301c-.6.179-1.2-.181-1.38-.721-.18-.601.18-1.2.72-1.381 4.26-1.26 11.28-1.02 15.721 1.621.539.3.719 1.02.419 1.56-.299.421-1.02.599-1.559.3z"
      />
    </template>
    <template v-else-if="Name === 'external'">
      <path d="M7.5 16.5 16.5 7.5M8.5 7.5h8v8" />
    </template>
    <template v-else-if="Name === 'chevron'">
      <path d="m9.5 6 6 6-6 6" />
    </template>
    <template v-else-if="Name === 'back'">
      <path d="M15 5 8 12l7 7" />
    </template>
    <template v-else-if="Name === 'check'">
      <path d="M19.5 6.5 9.2 17 4.5 12.3" />
    </template>
    <template v-else-if="Name === 'search'">
      <circle cx="10.8" cy="10.8" r="6.8" />
      <path d="m20.5 20.5-4.8-4.8" />
    </template>
    <template v-else-if="Name === 'close'">
      <path d="M17.5 6.5 6.5 17.5M6.5 6.5l11 11" />
    </template>
    <template v-else-if="Name === 'copy'">
      <rect x="8.5" y="8.5" width="12" height="12" rx="2.4" />
      <path d="M15.5 8.5V5.9a2.4 2.4 0 0 0-2.4-2.4H5.9a2.4 2.4 0 0 0-2.4 2.4v7.2a2.4 2.4 0 0 0 2.4 2.4h2.6" />
    </template>
    <template v-else-if="Name === 'refresh'">
      <path d="M20.5 12a8.5 8.5 0 0 1-14.6 5.9M3.5 12a8.5 8.5 0 0 1 14.6-5.9" />
      <path d="M18.5 2.8v3.6h-3.6M5.5 21.2v-3.6h3.6" />
    </template>
  </svg>
</template>

<style scoped>
.Glyph {
  display: block;
  height: 1em;
  width: 1em;
  flex-shrink: 0;
}
</style>
