<script setup lang="ts">
import { ref, watch, type Ref } from "vue"

const Props = defineProps<{ Src: string | null }>()

const Broken: Ref<boolean> = ref(false)

watch(() => Props.Src, () => { Broken.value = false })
</script>

<template>
  <div class="Cover">
    <img
      v-if="Src && !Broken"
      :src="Src"
      alt=""
      class="Image"
      draggable="false"
      loading="lazy"
      @error="Broken = true"
    />
    <div v-else class="Fallback">?</div>
  </div>
</template>

<style scoped>
.Cover {
  position: relative;
  overflow: hidden;
  background: #1c1917;
}

.Image {
  display: block;
  height: 100%;
  width: 100%;
  object-fit: cover;
  pointer-events: none;
}

.Fallback {
  display: flex;
  height: 100%;
  width: 100%;
  align-items: center;
  justify-content: center;
  background: #121114;
  font-family: var(--font-display);
  font-size: 0.8em;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.22);
}
</style>
