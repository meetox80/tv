<script setup lang="ts">
import { useI18n } from "../../i18n"

const { t } = useI18n()

defineProps<{ Domain: string }>()
const Emit = defineEmits<{ "update:Domain": [value: string]; Save: [] }>()
</script>

<template>
  <section id="s-tunnel">
    <p class="text-[15px] text-white/70 mb-1">{{ t("settings.tunnel") }}</p>
    <p class="text-[12px] text-[#57534E] mb-4">{{ t("settings.tunnel.description") }}</p>
    <div class="flex flex-col lg:flex-row gap-3">
      <input
        :value="Domain"
        @input="Emit('update:Domain', ($event.target as HTMLInputElement).value)"
        type="text"
        placeholder="example.com"
        class="flex-1 bg-white/[0.03] border border-white/[0.06] rounded-[10px] px-4 py-2.5 text-[13px] text-white/70 font-mono placeholder:text-[#44403C] focus:outline-none focus:border-white/[0.12] transition-colors duration-150"
      />
      <button
        class="px-5 py-2.5 bg-white/[0.06] hover:bg-white/[0.1] border border-white/[0.06] rounded-[10px] text-[13px] text-white/60 hover:text-white/80 transition-all duration-150 shrink-0"
        @click="Emit('Save')"
      >
        {{ t("settings.save") }}
      </button>
    </div>
  </section>
</template>
