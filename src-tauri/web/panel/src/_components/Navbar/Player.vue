<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted, onUnmounted, type Ref } from "vue"
import { useI18n } from "../../i18n"
import { Fetch } from "../../_api/http"

const { t } = useI18n()
const Emit = defineEmits<{ Navigate: [page: string] }>()

interface Track {
  Title: string
  Artist: string
  Cover: string | null
}

const NowPlaying: Ref<Track | null> = ref(null)
const SourceReady: Ref<boolean> = ref(false)
const ShownCover: Ref<string | null> = ref(null)
const MeasureEl: Ref<HTMLElement | null> = ref(null)
const TextWidth: Ref<string> = ref("auto")

let _PollTimer: ReturnType<typeof setInterval> | null = null
let _CoverToken = 0

async function _FetchTrack(): Promise<void> {
  try {
    const Res = await fetch("/api/playback/current")
    if (!Res.ok) return
    NowPlaying.value = await Res.json()
  } catch {}
}

async function _FetchSource(): Promise<void> {
  try {
    const Res = await Fetch("/api/playback/source")
    if (!Res.ok) return
    const Data = await Res.json()
    SourceReady.value = Data.Ready === true
  } catch {}
}

function Truncate(S: string, Max = 16) {
  return S.length > Max ? S.slice(0, Max) + "..." : S
}

const Display = computed(() => {
  if (!NowPlaying.value) return null
  return {
    Title: Truncate(NowPlaying.value.Title),
    Artist: Truncate(NowPlaying.value.Artist),
  }
})

const TrackKey = computed(() => {
  if (Display.value) return `${Display.value.Title}|${Display.value.Artist}`
  return SourceReady.value ? "idle" : "connect"
})

const CoverUrl = computed(() => NowPlaying.value?.Cover ?? null)

async function _Preload(Url: string): Promise<boolean> {
  const Img = new Image()
  Img.src = Url

  try {
    if (Img.decode) await Img.decode()
    else await new Promise((Resolve, Reject) => {
      Img.onload = Resolve
      Img.onerror = Reject
    })
    return true
  } catch {
    return false
  }
}

watch(CoverUrl, async (Next) => {
  const Token = ++_CoverToken

  if (!Next) {
    ShownCover.value = null
    return
  }

  if (Next === ShownCover.value) return

  const Ready = await _Preload(Next)
  if (Token !== _CoverToken) return

  ShownCover.value = Ready ? Next : null
})

watch(TrackKey, async () => {
  await nextTick()
  const El = MeasureEl.value
  if (El) TextWidth.value = `${El.offsetWidth}px`
}, { immediate: true })

onMounted(() => {
  _FetchTrack()
  _FetchSource()

  _PollTimer = setInterval(() => {
    _FetchTrack()
    _FetchSource()
  }, 3000)
})

onUnmounted(() => {
  if (_PollTimer) clearInterval(_PollTimer)
})
</script>

<template>
  <div class="Pill">
    <div class="Art">
      <Transition name="art">
        <img v-if="ShownCover" :key="ShownCover" :src="ShownCover" alt="" class="ArtImg" />
        <span v-else key="idle" class="ArtIdle">
          <svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 3v10.55c-.59-.34-1.27-.55-2-.55C7.79 13 6 14.79 6 17s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z"/></svg>
        </span>
      </Transition>
    </div>

    <div class="Text" :style="{ width: TextWidth }">
      <Transition name="meta">
        <div :key="TrackKey" class="Meta">
          <template v-if="Display">
            <span class="Title">{{ Display.Title }}</span>
            <span class="Dot">·</span>
            <span class="Artist">{{ Display.Artist }}</span>
          </template>
          <span v-else-if="SourceReady" class="Idle">{{ t("player.idle") }}</span>
          <button v-else class="Connect" @click="Emit('Navigate', 'settings')">
            {{ t("player.connect") }}
          </button>
        </div>
      </Transition>

      <div ref="MeasureEl" class="Meta Measure" aria-hidden="true">
        <template v-if="Display">
          <span class="Title">{{ Display.Title }}</span>
          <span class="Dot">·</span>
          <span class="Artist">{{ Display.Artist }}</span>
        </template>
        <span v-else-if="SourceReady" class="Idle">{{ t("player.idle") }}</span>
        <span v-else class="Connect">{{ t("player.connect") }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.Pill {
  display: flex;
  align-items: center;
  gap: 8px;
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.04);
  padding: 4px 14px 4px 4px;
  font-size: 12px;
  line-height: 1;
}

.Art {
  position: relative;
  height: 24px;
  width: 24px;
  flex-shrink: 0;
}

.ArtImg,
.ArtIdle {
  position: absolute;
  inset: 0;
  height: 24px;
  width: 24px;
  border-radius: 999px;
  object-fit: cover;
  box-shadow: 0 0 0 1px rgba(255, 255, 255, 0.1);
}

.ArtIdle {
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.04);
  box-shadow: 0 0 0 1px rgba(255, 255, 255, 0.06);
}

.ArtIdle svg {
  height: 12px;
  width: 12px;
  color: rgba(255, 255, 255, 0.25);
}

.Text {
  position: relative;
  height: 14px;
  overflow: hidden;
  transition: width 320ms cubic-bezier(0.2, 0.9, 0.3, 1);
}

.Meta {
  position: absolute;
  top: 0;
  left: 0;
  display: flex;
  height: 14px;
  align-items: center;
  gap: 8px;
  white-space: nowrap;
}

.Measure {
  visibility: hidden;
  pointer-events: none;
}

.Title {
  color: rgba(255, 255, 255, 0.6);
}

.Dot {
  color: rgba(255, 255, 255, 0.2);
}

.Artist {
  color: rgba(255, 255, 255, 0.35);
}

.Idle {
  color: rgba(255, 255, 255, 0.25);
}

.Connect {
  color: rgba(255, 255, 255, 0.25);
  transition: color 200ms ease;
}

.Connect:hover {
  color: rgba(255, 255, 255, 0.5);
}

.art-enter-active {
  transition: opacity 280ms ease, transform 320ms cubic-bezier(0.2, 0.9, 0.3, 1);
}

.art-leave-active {
  transition: opacity 220ms ease, transform 320ms cubic-bezier(0.2, 0.9, 0.3, 1);
}

.art-enter-from {
  opacity: 0;
  transform: scale(0.82);
}

.art-leave-to {
  opacity: 0;
  transform: scale(1.12);
}

.meta-enter-active {
  transition: opacity 240ms ease 70ms, transform 300ms cubic-bezier(0.2, 0.9, 0.3, 1) 70ms;
}

.meta-leave-active {
  transition: opacity 160ms ease, transform 220ms ease;
}

.meta-enter-from {
  opacity: 0;
  transform: translateY(7px);
}

.meta-leave-to {
  opacity: 0;
  transform: translateY(-7px);
}
</style>
