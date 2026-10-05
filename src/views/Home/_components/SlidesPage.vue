<script setup lang="ts">
import { ElementStyle, SlideHeight, SlideWidth, type Slide } from "./slides"

defineProps<{ Slide: Slide }>()

const _AssetBase = "http://127.0.0.1:9999/api/slides/assets"
</script>

<template>
  <div class="Slides">
    <div
      class="Canvas"
      :class="{ Filled: !!Slide.Background }"
      :style="{ width: `${SlideWidth}px`, height: `${SlideHeight}px`, background: Slide.Background ?? 'transparent' }"
    >
      <template v-for="Element in Slide.Elements.filter(E => !E.Hidden)" :key="Element.Id">
        <p v-if="Element.Kind === 'text'" :style="ElementStyle(Element)">{{ Element.Text }}</p>
        <div v-else-if="Element.Kind === 'image'" :style="ElementStyle(Element)">
          <img
            :src="`${_AssetBase}/${Element.Asset}`"
            alt=""
            class="Image"
            :style="{ objectFit: Element.Fit ?? 'cover' }"
          />
        </div>
        <div v-else :style="ElementStyle(Element)" />
      </template>
    </div>
  </div>
</template>

<style scoped>
.Slides {
  display: flex;
  align-items: center;
  justify-content: center;
}

.Canvas {
  position: relative;
  flex-shrink: 0;
  overflow: hidden;
}

.Canvas.Filled {
  border-radius: 28px;
}

.Image {
  display: block;
  height: 100%;
  width: 100%;
}
</style>
