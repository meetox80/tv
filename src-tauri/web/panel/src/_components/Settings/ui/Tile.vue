<script setup lang="ts">
import { computed } from "vue"
import Glyph from "./Glyph.vue"
import { Tints, type GlyphName, type Tint } from "../sections"

const Props = withDefaults(defineProps<{ Icon: GlyphName; Tint: Tint; Size?: number }>(), { Size: 36 })

const Style = computed(() => {
  const [Top, Bottom] = Tints[Props.Tint]
  return {
    width: `${Props.Size}px`,
    height: `${Props.Size}px`,
    borderRadius: `${Math.round(Props.Size * 0.3)}px`,
    fontSize: `${Math.round(Props.Size * 0.54)}px`,
    background: `linear-gradient(180deg, ${Top}, ${Bottom})`,
  }
})
</script>

<template>
  <span class="Tile" :style="Style" aria-hidden="true">
    <Glyph :Name="Icon" />
  </span>
</template>

<style scoped>
.Tile {
  position: relative;
  display: inline-flex;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  color: #fff;
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.3),
    inset 0 -1px 0 rgba(0, 0, 0, 0.15),
    0 10px 24px -12px rgba(0, 0, 0, 0.9);
}

.Tile :deep(.Glyph) {
  filter: drop-shadow(0 1px 1px rgba(0, 0, 0, 0.2));
}
</style>
