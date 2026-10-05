<script setup lang="ts">
import { ref, watch, computed, type Ref } from "vue"

const Props = withDefaults(defineProps<{
  Src: string | null
  Title: string
  Size?: number
  Radius?: number
  Fill?: boolean
}>(), { Size: 52, Radius: 12, Fill: false })

const Failed: Ref<boolean> = ref(false)
const Loaded: Ref<boolean> = ref(false)

watch(() => Props.Src, () => {
  Failed.value = false
  Loaded.value = false
})

const Box = computed(() => {
  if (Props.Fill) return { width: "100%", height: "100%", borderRadius: "inherit", fontSize: `${Math.round(Props.Size * 0.4)}px` }
  return {
    width: `${Props.Size}px`,
    height: `${Props.Size}px`,
    borderRadius: `${Props.Radius}px`,
    fontSize: `${Math.round(Props.Size * 0.4)}px`,
  }
})
</script>

<template>
  <div class="Cover" :style="Box" aria-hidden="true">
    <div v-if="!Src || Failed" class="Placeholder Blank">?</div>
    <div v-else class="Placeholder Skeleton" :class="{ Still: Loaded }" />
    <img
      v-if="Src && !Failed"
      :src="Src"
      alt=""
      class="Image"
      :class="{ Shown: Loaded }"
      loading="lazy"
      decoding="async"
      referrerpolicy="no-referrer"
      @load="Loaded = true"
      @error="Failed = true"
    />
  </div>
</template>

<style scoped>
.Cover {
  position: relative;
  flex-shrink: 0;
  overflow: hidden;
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.08);
}

.Placeholder {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: var(--font-display);
  font-weight: 600;
  color: rgba(255, 255, 255, 0.55);
}

.Blank {
  background: #121114;
  font-size: 0.8em;
  color: rgba(255, 255, 255, 0.22);
}

.Still {
  animation: none;
}

.Image {
  position: absolute;
  inset: 0;
  height: 100%;
  width: 100%;
  object-fit: cover;
  opacity: 0;
  transition: opacity 420ms ease;
}

.Image.Shown {
  opacity: 1;
}
</style>
