<script setup lang="ts">
import { computed } from "vue"
import { useI18n } from "../../../i18n"

interface Post {
  Id: number
  Title: string
  Date: string
  Image: string | null
  Body: string[]
}

const Props = defineProps<{ Post: Post | null }>()

const { t } = useI18n()

const _Months = [
  "stycznia", "lutego", "marca", "kwietnia", "maja", "czerwca",
  "lipca", "sierpnia", "września", "października", "listopada", "grudnia",
]

const Day = computed(() => {
  const Raw = Props.Post?.Date
  if (!Raw) return ""

  const At = new Date(Raw)
  if (Number.isNaN(At.getTime())) return ""

  return `${At.getDate()} ${_Months[At.getMonth()]}`
})

const Lead = computed(() => Props.Post?.Body[0] ?? "")
</script>

<template>
  <div v-if="Post" class="News" :class="{ Solo: !Post.Image }">
    <div v-if="Post.Image" class="Frame">
      <img :key="Post.Image" :src="Post.Image" alt="" class="Shot" />
    </div>

    <div class="Column">
      <p class="Eyebrow">
        <span>{{ t("news.title") }}</span>
        <span v-if="Day" class="Separator" />
        <span v-if="Day">{{ Day }}</span>
      </p>

      <h1 class="Headline">{{ Post.Title }}</h1>

      <p v-if="Lead" class="Lead">{{ Lead }}</p>
    </div>
  </div>
</template>

<style scoped>
.News {
  display: flex;
  width: 100%;
  height: 100%;
  align-items: center;
  gap: 88px;
}

.Frame {
  position: relative;
  height: 100%;
  width: 50%;
  flex-shrink: 0;
  overflow: hidden;
  border-radius: 36px;
  background: rgba(255, 255, 255, 0.03);
  box-shadow: 0 50px 100px -40px rgba(0, 0, 0, 0.95);
  isolation: isolate;
}

.Shot {
  height: 100%;
  width: 100%;
  object-fit: cover;
  object-position: center 40%;
  transform-origin: 60% 40%;
  animation: drift 28s cubic-bezier(0.45, 0, 0.55, 1) infinite alternate;
}

@keyframes drift {
  from {
    transform: scale(1);
  }
  to {
    transform: scale(1.06);
  }
}

.Column {
  display: flex;
  min-width: 0;
  flex: 1;
  flex-direction: column;
  align-items: flex-start;
}

.Solo {
  justify-content: center;
}

.Solo .Column {
  width: 1300px;
  flex: none;
}

.Eyebrow {
  display: flex;
  align-items: center;
  gap: 16px;
  border-radius: 999px;
  padding: 13px 28px;
  background: rgba(255, 255, 255, 0.06);
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.08);
  backdrop-filter: blur(16px);
  font-family: var(--font-body);
  font-size: 27px;
  font-weight: 500;
  letter-spacing: -0.01em;
  line-height: 1;
  color: rgba(255, 255, 255, 0.72);
}

.Separator {
  height: 5px;
  width: 5px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.3);
}

.Headline {
  display: -webkit-box;
  width: 100%;
  margin-top: 34px;
  overflow: hidden;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 3;
  font-family: var(--font-clock);
  font-size: 84px;
  font-weight: 600;
  letter-spacing: -0.04em;
  line-height: 1.04;
  color: rgba(255, 255, 255, 0.98);
}

.Lead {
  display: -webkit-box;
  width: 100%;
  margin-top: 34px;
  overflow: hidden;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 5;
  font-family: var(--font-clock);
  font-size: 36px;
  font-weight: 400;
  line-height: 1.42;
  letter-spacing: -0.015em;
  color: rgba(255, 255, 255, 0.64);
}

@media (prefers-reduced-motion: reduce) {
  .Shot {
    animation: none;
  }
}
</style>
