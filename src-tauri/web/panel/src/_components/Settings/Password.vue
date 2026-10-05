<script setup lang="ts">
import { ref } from "vue"
import { useI18n } from "../../i18n"

const { t } = useI18n()

const Current = ref("")
const Next = ref("")
</script>

<template>
  <section id="s-password">
    <p class="text-[15px] text-white/70 mb-1">{{ t("settings.password") }}</p>
    <p class="text-[12px] text-[#57534E] mb-4">{{ t("settings.password.description") }}</p>
    <div class="space-y-3 max-w-[360px]">
      <input
        v-model="Current"
        type="password"
        :placeholder="t('settings.password.current')"
        class="w-full bg-white/[0.03] border border-white/[0.06] rounded-[10px] px-4 py-2.5 text-[13px] text-white/70 placeholder:text-[#44403C] focus:outline-none focus:border-white/[0.12] transition-colors duration-150"
      />
      <input
        v-model="Next"
        type="password"
        :placeholder="t('settings.password.new')"
        class="w-full bg-white/[0.03] border border-white/[0.06] rounded-[10px] px-4 py-2.5 text-[13px] text-white/70 placeholder:text-[#44403C] focus:outline-none focus:border-white/[0.12] transition-colors duration-150"
      />
      <button class="px-5 py-2.5 bg-white/[0.06] hover:bg-white/[0.1] border border-white/[0.06] rounded-[10px] text-[13px] text-white/60 hover:text-white/80 transition-all duration-150">
        {{ t("settings.password.change") }}
      </button>
    </div>
  </section>
</template>
