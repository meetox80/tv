<script setup lang="ts">
import { computed } from "vue"
import Sheet from "./Sheet.vue"
import Captcha from "./Captcha.vue"
import { AnswerCaptcha, CaptchaPending } from "../_state/store"

const Lead = computed(() =>
  CaptchaPending.value?.Reason === "vote"
    ? "Zanim oddasz pierwszy głos, potwierdź, że nie jesteś botem. Na tym urządzeniu robisz to tylko raz."
    : "Zanim wyślesz propozycję, potwierdź, że nie jesteś botem. Na tym urządzeniu robisz to tylko raz."
)
</script>

<template>
  <Sheet :Open="CaptchaPending !== null" Label="Weryfikacja" @Close="AnswerCaptcha(false)">
    <div class="Body">
      <h2 class="Title">Jeszcze tylko jedno</h2>
      <p class="Lead">{{ Lead }}</p>
      <Captcha v-if="CaptchaPending" @Done="AnswerCaptcha(true)" />
    </div>
  </Sheet>
</template>

<style scoped>
.Body {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding-top: 18px;
}

.Title {
  font-family: var(--font-display);
  font-size: 24px;
  font-weight: 600;
  letter-spacing: -0.02em;
}

.Lead {
  font-size: 15px;
  line-height: 1.55;
  color: var(--color-ink-soft);
}
</style>
