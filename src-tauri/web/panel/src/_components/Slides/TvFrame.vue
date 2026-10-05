<script setup lang="ts">
import Backdrop from "@tv/Backdrop.vue"
import TopBar from "@tv/TopBar.vue"
import Clock from "@tv/Clock.vue"
import Player from "@tv/Player.vue"
import { SlideHeight, SlideWidth } from "./model"

export interface TvState {
  Track: { Title: string; Artist: string } | null
  Cover: string | null
  Notice: string
  Pages: string[]
  Active: number
  Now: Date
  Temperature: number | null
  Condition: string | null
}

const _ScreenWidth = 1920
const _ScreenHeight = 1080
const _PageLeft = 72
const _PageTop = 168

defineProps<{ Scale: number; State: TvState }>()
</script>

<template>
  <div class="Tv" :style="{ width: `${_ScreenWidth * Scale}px`, height: `${_ScreenHeight * Scale}px` }">
    <div class="Screen" :style="{ transform: `scale(${Scale})` }" aria-hidden="true">
      <Backdrop :Cover="State.Cover" />
      <TopBar :Notice="State.Notice" :Pages="State.Pages" :Active="State.Active" />
      <div class="Bottom">
        <Clock :Now="State.Now" :Temperature="State.Temperature" :Condition="State.Condition" />
        <Player :Track="State.Track" :Cover="State.Cover" :Muted="false" />
      </div>
    </div>

    <div
      class="Page"
      :style="{
        left: `${_PageLeft * Scale}px`,
        top: `${_PageTop * Scale}px`,
        width: `${SlideWidth * Scale}px`,
        height: `${SlideHeight * Scale}px`,
      }"
    >
      <slot />
    </div>
  </div>
</template>

<style scoped>
.Tv {
  position: relative;
  flex-shrink: 0;
  overflow: hidden;
  border-radius: 10px;
  background: #050507;
  box-shadow: 0 0 0 1px var(--edge), 0 30px 80px -40px rgba(0, 0, 0, 0.9);
}

.Screen {
  position: absolute;
  top: 0;
  left: 0;
  height: 1080px;
  width: 1920px;
  overflow: hidden;
  background: #050507;
  color: #fff;
  transform-origin: 0 0;
  pointer-events: none;
}

.Bottom {
  position: absolute;
  right: 72px;
  bottom: 62px;
  left: 72px;
  z-index: 10;
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 48px;
}

.Page {
  position: absolute;
  z-index: 15;
}
</style>
