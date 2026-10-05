<script setup lang="ts">
import { ref, watch, nextTick, type Ref } from "vue"
import { useI18n } from "../../i18n"
import Sheet from "../Settings/ui/Sheet.vue"

const { t } = useI18n()

const Props = defineProps<{
  Open: boolean
  Title: string
  Body: string
  ConfirmLabel: string
  WithReason?: boolean
  Busy?: boolean
}>()

const Emit = defineEmits<{ Confirm: [reason: string]; Cancel: [] }>()

const Reason: Ref<string> = ref("")
const Field: Ref<HTMLInputElement | null> = ref(null)

watch(() => Props.Open, async (Now) => {
  if (!Now) return
  Reason.value = ""
  await nextTick()
  Field.value?.focus()
})
</script>

<template>
  <Sheet :Open="Open" :Title="Title" :Message="Body" :Layer="80" @Close="Emit('Cancel')">
    <template v-if="WithReason" #default>
      <input
        ref="Field"
        v-model="Reason"
        type="text"
        maxlength="200"
        class="Input"
        :placeholder="t('voting.bans.reason.placeholder')"
        :aria-label="t('voting.bans.reason')"
        @keydown.enter="Emit('Confirm', Reason)"
      />
    </template>

    <template #actions>
      <button type="button" class="Btn" @click="Emit('Cancel')">{{ t("settings.cancel") }}</button>
      <button type="button" class="Btn Armed" :disabled="Busy" @click="Emit('Confirm', Reason)">{{ ConfirmLabel }}</button>
    </template>
  </Sheet>
</template>
