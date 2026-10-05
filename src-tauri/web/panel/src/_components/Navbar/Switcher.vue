<script setup lang="ts">
import { useI18n } from "../../i18n"

const { t } = useI18n()

defineProps<{ Active: string }>()
const Emit = defineEmits<{ Navigate: [page: string] }>()

const Pages = ["dashboard", "voting", "tunnel", "settings"] as const
</script>

<template>
  <div class="flex items-center gap-4 lg:gap-8 overflow-x-auto no-scrollbar">
    <button
      v-for="P in Pages"
      :key="P"
      class="text-[12px] lg:text-[13px] whitespace-nowrap transition-colors duration-300"
      :class="Active === P ? 'text-white/80' : 'text-white/40 hover:text-white/80'"
      @click="Emit('Navigate', P)"
    >
      {{ t(`nav.${P}`) }}
    </button>
  </div>
</template>
