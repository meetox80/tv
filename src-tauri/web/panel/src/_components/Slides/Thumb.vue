<script setup lang="ts">
import { ref, onMounted, onUnmounted, type Ref } from "vue"
import SlideView from "./SlideView.vue"
import { SlideWidth, type Slide } from "./model"

defineProps<{ Slide: Slide | null }>()

const El: Ref<HTMLElement | null> = ref(null)
const Scale: Ref<number> = ref(0)

let _Observer: ResizeObserver | null = null

onMounted(() => {
  if (!El.value) return
  _Observer = new ResizeObserver(([Entry]) => {
    Scale.value = Entry.contentRect.width / SlideWidth
  })
  _Observer.observe(El.value)
})

onUnmounted(() => _Observer?.disconnect())
</script>

<template>
  <div ref="El" class="Thumb">
    <SlideView v-if="Slide && Scale > 0" :Slide="Slide" :Scale="Scale" />
  </div>
</template>

<style scoped>
.Thumb {
  position: relative;
  overflow: hidden;
  width: 100%;
  aspect-ratio: 1776 / 698;
  border-radius: 10px;
  background: #0b0a0e;
  box-shadow: inset 0 0 0 1px var(--edge);
}

.Thumb :deep(.View) {
  position: absolute;
  inset: 0;
}
</style>
