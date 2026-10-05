<script setup lang="ts">
import { computed } from "vue"

interface Track {
  Title: string
  Artist: string
}

const Props = defineProps<{
  Track: Track | null
  Cover: string | null
  Muted: boolean
}>()

const Visible = computed(() => !Props.Muted && Props.Track !== null)

const Key = computed(() => (Props.Track ? `${Props.Track.Title}|${Props.Track.Artist}` : ""))
</script>

<template>
  <Transition name="player">
    <div v-if="Visible" class="Player">
      <div class="Stack">
        <Transition name="swap">
          <div :key="Key" class="Meta">
            <span class="Title">{{ Track?.Title }}</span>
            <span class="Artist">{{ Track?.Artist }}</span>
          </div>
        </Transition>
      </div>

      <div class="Sleeve">
        <Transition name="art">
          <img v-if="Cover" :key="Cover" :src="Cover" alt="" class="Cover" />
        </Transition>

        <div v-if="!Cover" class="Empty">
          <span class="Bar" />
          <span class="Bar" />
          <span class="Bar" />
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.Player {
  display: flex;
  min-width: 0;
  flex-shrink: 1;
  align-items: center;
  gap: 26px;
}

.Stack {
  position: relative;
  display: flex;
  min-width: 0;
  max-width: 760px;
  flex-direction: column;
  align-items: flex-end;
  overflow: hidden;
}

.Meta {
  display: flex;
  width: 100%;
  flex-direction: column;
  align-items: flex-end;
  gap: 7px;
}

.Title {
  display: block;
  max-width: 760px;
  overflow: hidden;
  font-family: var(--font-clock, "Inter", system-ui, sans-serif);
  font-size: 31px;
  font-weight: 600;
  line-height: 1.15;
  letter-spacing: -0.025em;
  color: rgba(255, 255, 255, 0.97);
  text-align: right;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.Artist {
  display: block;
  max-width: 760px;
  overflow: hidden;
  font-family: var(--font-clock, "Inter", system-ui, sans-serif);
  font-size: 21px;
  font-weight: 500;
  line-height: 1.2;
  letter-spacing: -0.01em;
  color: rgba(255, 255, 255, 0.52);
  text-align: right;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.Sleeve {
  position: relative;
  height: 92px;
  width: 92px;
  flex-shrink: 0;
  overflow: hidden;
  border-radius: 14px;
  background: rgba(255, 255, 255, 0.05);
  box-shadow:
    inset 0 0 0 1px rgba(255, 255, 255, 0.1),
    0 18px 44px -14px rgba(0, 0, 0, 0.9);
}

.Cover {
  position: absolute;
  inset: 0;
  height: 100%;
  width: 100%;
  object-fit: cover;
}

.Empty {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 5px;
}

.Empty .Bar {
  height: 20px;
  width: 4px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.25);
}

.Empty .Bar:nth-child(2) {
  height: 32px;
}

.player-enter-active {
  transition: opacity 420ms ease, transform 620ms cubic-bezier(0.2, 0.9, 0.3, 1);
}

.player-leave-active {
  transition: opacity 360ms ease, transform 560ms cubic-bezier(0.4, 0, 0.7, 0.2);
}

.player-enter-from,
.player-leave-to {
  opacity: 0;
  transform: translateX(calc(100% + 72px));
}

.swap-enter-active,
.swap-leave-active {
  transition: transform 560ms cubic-bezier(0.22, 0.61, 0.36, 1), opacity 340ms ease;
}

.swap-leave-active {
  position: absolute;
  top: 0;
  right: 0;
}

.swap-enter-from {
  opacity: 0;
  transform: translateY(100%);
}

.swap-leave-to {
  opacity: 0;
  transform: translateY(-100%);
}

.art-enter-active,
.art-leave-active {
  transition: opacity 600ms ease;
}

.art-leave-active {
  position: absolute;
}

.art-enter-from,
.art-leave-to {
  opacity: 0;
}
</style>
