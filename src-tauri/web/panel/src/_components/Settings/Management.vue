<script setup lang="ts">
import { ref, computed, onUnmounted, type Ref } from "vue"
import { useI18n } from "../../i18n"
import { ChangePassword, NukeDatabase } from "../../_api/settings"
import Pane from "./ui/Pane.vue"
import Group from "./ui/Group.vue"
import Row from "./ui/Row.vue"
import Sheet from "./ui/Sheet.vue"

const { t } = useI18n()

const _DoneMs = 6000

const PasswordOpen: Ref<boolean> = ref(false)
const NukeOpen: Ref<boolean> = ref(false)
const Current: Ref<string> = ref("")
const Next: Ref<string> = ref("")
const Repeat: Ref<string> = ref("")
const Busy: Ref<boolean> = ref(false)
const Fault: Ref<string> = ref("")
const Done: Ref<boolean> = ref(false)
const Nuking: Ref<boolean> = ref(false)

let _DoneTimer: ReturnType<typeof setTimeout> | null = null

const Mismatch = computed(() => Repeat.value.length > 0 && Next.value !== Repeat.value)

const CanSubmit = computed(
  () => !!Current.value && !!Next.value.trim() && Next.value === Repeat.value && !Busy.value
)

function OpenPassword(): void {
  Current.value = ""
  Next.value = ""
  Repeat.value = ""
  Fault.value = ""
  PasswordOpen.value = true
}

function _FaultText(Code: string): string {
  const Key = `settings.password.error.${Code}`
  const Text = t(Key)
  return Text === Key ? t("settings.password.error.unknown") : Text
}

async function SubmitPassword(): Promise<void> {
  if (!CanSubmit.value) return

  Busy.value = true
  Fault.value = ""

  try {
    const Result = await ChangePassword(Current.value, Next.value)
    if (!Result.Ok) {
      Fault.value = _FaultText(Result.Error ?? "unknown")
      return
    }

    PasswordOpen.value = false
    Done.value = true
    if (_DoneTimer) clearTimeout(_DoneTimer)
    _DoneTimer = setTimeout(() => { Done.value = false }, _DoneMs)
  } catch {
    Fault.value = _FaultText("unknown")
  } finally {
    Busy.value = false
  }
}

async function ConfirmNuke(): Promise<void> {
  if (Nuking.value) return
  Nuking.value = true

  try {
    await NukeDatabase()
  } catch {
    Nuking.value = false
    return
  }

  NukeOpen.value = false
  const Message = t("settings.nuke.done")
  document.body.innerHTML = `<div style="display:flex;align-items:center;justify-content:center;height:100vh;background:#0b0a0a;color:rgba(235,235,245,0.6);font-family:-apple-system,BlinkMacSystemFont,Geist,sans-serif;font-size:15px">${Message}</div>`
}

onUnmounted(() => {
  if (_DoneTimer) clearTimeout(_DoneTimer)
})
</script>

<template>
  <Pane Id="management">
    <Group>
      <Row :Label="t('settings.password.change')" Chevron @Press="OpenPassword" />
      <Row :Label="t('settings.nuke')" Danger Chevron @Press="NukeOpen = true" />

      <template v-if="Done" #footer>{{ t("settings.password.done") }}</template>
    </Group>

    <Sheet :Open="PasswordOpen" :Title="t('settings.password.change')" @Close="PasswordOpen = false">
      <input
        v-model="Current"
        type="password"
        autocomplete="current-password"
        :placeholder="t('settings.password.current')"
        class="Input"
        @keydown.enter="SubmitPassword"
      />
      <input
        v-model="Next"
        type="password"
        autocomplete="new-password"
        :placeholder="t('settings.password.new')"
        class="Input"
        @keydown.enter="SubmitPassword"
      />
      <input
        v-model="Repeat"
        type="password"
        autocomplete="new-password"
        :placeholder="t('settings.password.repeat')"
        class="Input"
        :class="{ Invalid: Mismatch }"
        @keydown.enter="SubmitPassword"
      />
      <p v-if="Fault || Mismatch" class="SheetFault Fault">{{ Fault || t("settings.password.error.mismatch") }}</p>

      <template #actions>
        <button type="button" class="Btn" @click="PasswordOpen = false">{{ t("settings.cancel") }}</button>
        <button type="button" class="Btn Primary" :disabled="!CanSubmit" @click="SubmitPassword">
          {{ Busy ? t("settings.password.saving") : t("settings.password.submit") }}
        </button>
      </template>
    </Sheet>

    <Sheet
      :Open="NukeOpen"
      :Title="t('settings.nuke.title')"
      :Message="t('settings.nuke.warning')"
      @Close="NukeOpen = false"
    >
      <template #actions>
        <button type="button" class="Btn" @click="NukeOpen = false">{{ t("settings.cancel") }}</button>
        <button type="button" class="Btn Armed" :disabled="Nuking" @click="ConfirmNuke">{{ t("settings.nuke.confirm") }}</button>
      </template>
    </Sheet>
  </Pane>
</template>

<style scoped>
.SheetFault {
  margin: 2px 2px 0;
  font-size: 12px;
  line-height: 1.4;
}
</style>
