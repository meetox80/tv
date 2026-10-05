<script setup lang="ts">
import { AssetUrl, ElementStyle, SlideHeight, SlideWidth, type Slide } from "./model"

withDefaults(defineProps<{ Slide: Slide; Scale?: number; Hidden?: string | null; Interactive?: boolean }>(), {
  Scale: 1,
  Hidden: null,
  Interactive: false,
})
</script>

<template>
  <div
    class="View"
    :class="{ Interactive }"
    :style="{ width: `${SlideWidth * Scale}px`, height: `${SlideHeight * Scale}px`, '--Hair': `${1 / Scale}px` }"
  >
    <div
      class="Canvas"
      :class="{ Filled: !!Slide.Background }"
      :style="{ transform: `scale(${Scale})`, background: Slide.Background ?? 'transparent' }"
    >
      <template v-for="Element in Slide.Elements.filter(E => !E.Hidden)" :key="Element.Id">
        <p
          v-if="Element.Kind === 'text'"
          :data-id="Element.Id"
          class="Item"
          :class="{ Ghost: Hidden === Element.Id, Locked: Element.Locked }"
          :style="ElementStyle(Element)"
        >{{ Element.Text || " " }}</p>
        <div
          v-else-if="Element.Kind === 'image'"
          :data-id="Element.Id"
          class="Item"
          :class="{ Locked: Element.Locked }"
          :style="ElementStyle(Element)"
        >
          <img :src="AssetUrl(Element.Asset ?? '')" alt="" class="Image" draggable="false" :style="{ objectFit: Element.Fit ?? 'cover' }" />
        </div>
        <div v-else :data-id="Element.Id" class="Item" :class="{ Locked: Element.Locked }" :style="ElementStyle(Element)" />
      </template>
    </div>
  </div>
</template>

<style scoped>
.View {
  position: relative;
  flex-shrink: 0;
  overflow: hidden;
}

.Canvas {
  position: absolute;
  top: 0;
  left: 0;
  width: 1776px;
  height: 698px;
  overflow: hidden;
  transform-origin: 0 0;
}

.Canvas.Filled {
  border-radius: 28px;
}

.Image {
  display: block;
  height: 100%;
  width: 100%;
  pointer-events: none;
  user-select: none;
}

.Ghost {
  visibility: hidden;
}

.Interactive .Item {
  cursor: move;
  touch-action: none;
}

.Interactive .Item:hover {
  outline: calc(var(--Hair) * 1.5) solid rgba(255, 255, 255, 0.35);
}

.Interactive .Item.Locked {
  pointer-events: none;
}
</style>
