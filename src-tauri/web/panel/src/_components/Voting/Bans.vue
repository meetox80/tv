<script setup lang="ts">
import { ref, onMounted, watch, type Ref } from "vue"
import { useI18n } from "../../i18n"
import * as Api from "../../_api/voting"
import { Relative, Plural } from "./_format"

const { t } = useI18n()

const Props = defineProps<{ Tick: number; Revision: number }>()
const Emit = defineEmits<{ Changed: [count: number]; Error: [error: unknown] }>()

const Bans: Ref<Api.BannedDevice[]> = ref([])
const Loaded: Ref<boolean> = ref(false)
const Handle: Ref<string> = ref("")
const Reason: Ref<string> = ref("")
const Busy: Ref<boolean> = ref(false)
const Unbanning: Ref<string> = ref("")

async function Load(): Promise<void> {
  try {
    Bans.value = await Api.LoadBans()
    Loaded.value = true
    Emit("Changed", Bans.value.length)
  } catch (E) {
    Emit("Error", E)
  }
}

async function Ban(): Promise<void> {
  const Value = Handle.value.trim()
  if (!Value || Busy.value) return

  Busy.value = true

  try {
    await Api.SetBan(Value, true, Reason.value)
    Handle.value = ""
    Reason.value = ""
    await Load()
  } catch (E) {
    Emit("Error", E)
  } finally {
    Busy.value = false
  }
}

async function Unban(Value: string): Promise<void> {
  if (Unbanning.value) return
  Unbanning.value = Value

  try {
    await Api.SetBan(Value, false)
    Bans.value = Bans.value.filter(B => B.Handle !== Value)
    Emit("Changed", Bans.value.length)
  } catch (E) {
    Emit("Error", E)
  } finally {
    Unbanning.value = ""
  }
}

watch(() => Props.Revision, () => {
  void Load()
})

onMounted(() => {
  void Load()
})
</script>

<template>
  <div class="Bans">
    <p class="Intro">{{ t("voting.bans.description") }}</p>

    <form class="Add" @submit.prevent="Ban">
      <input
        v-model="Handle"
        type="text"
        spellcheck="false"
        autocomplete="off"
        class="Input Monospace HandleField"
        :aria-label="t('voting.bans.handle')"
        :placeholder="t('voting.bans.handle.placeholder')"
      />
      <input
        v-model="Reason"
        type="text"
        maxlength="200"
        class="Input Grow"
        :aria-label="t('voting.bans.reason')"
        :placeholder="t('voting.bans.reason.placeholder')"
      />
      <button type="submit" class="Btn Danger Submit" :disabled="!Handle.trim() || Busy">
        {{ t("voting.bans.ban") }}
      </button>
    </form>

    <TransitionGroup tag="div" name="ban" class="Rows">
      <div v-for="Device in Bans" :key="Device.Handle" class="Row">
        <div class="Main">
          <p class="Handle">{{ Device.Handle }}</p>
          <p class="Meta" :data-tick="Tick">
            <span :class="{ Reason: !!Device.Reason }">{{ Device.Reason || t("voting.bans.noreason") }}</span>
            <span class="Sep" />
            <span>{{ t("voting.bans.since") }} {{ Relative(Device.BannedAt) }}</span>
            <span class="Sep" />
            <span class="Count">{{ Device.Requests }} {{ t(`voting.bans.requests.${Plural(Device.Requests)}`) }}</span>
          </p>
        </div>

        <button type="button" class="Unban" :disabled="Unbanning === Device.Handle" @click="Unban(Device.Handle)">
          {{ t("voting.bans.unban") }}
        </button>
      </div>
    </TransitionGroup>

    <p v-if="Loaded && !Bans.length" class="Empty">{{ t("voting.bans.empty") }}</p>
  </div>
</template>

<style scoped>
.Bans {
  width: 100%;
  max-width: 820px;
  margin: 0 auto;
}

.Intro {
  margin: 0 0 18px;
  font-size: 14.5px;
  line-height: 1.5;
  color: var(--ink-mute);
}

.Add {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.HandleField {
  font-size: 14px;
}

.Submit {
  height: 42px;
}

@media (min-width: 768px) {
  .Add {
    flex-direction: row;
  }

  .HandleField {
    width: 230px;
    flex-shrink: 0;
  }

  .Grow {
    flex: 1;
  }
}

.Rows {
  position: relative;
  display: flex;
  flex-direction: column;
  margin-top: 28px;
}

.Row {
  position: relative;
  display: flex;
  align-items: center;
  gap: 14px;
  margin: 0 -14px;
  border-radius: 20px;
  padding: 16px 14px;
  transition: background 200ms ease;
}

.Row + .Row::before {
  content: "";
  position: absolute;
  top: 0;
  right: 14px;
  left: 14px;
  height: 1px;
  background: var(--line);
  transition: opacity 200ms ease;
}

.Row:hover {
  background: rgba(255, 255, 255, 0.035);
}

.Row:hover::before,
.Row:hover + .Row::before {
  opacity: 0;
}

.Main {
  min-width: 0;
  flex: 1;
}

.Handle {
  margin: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-family: var(--mono);
  color: var(--ink);
}

.Meta {
  display: flex;
  min-width: 0;
  flex-wrap: wrap;
  align-items: center;
  gap: 4px 8px;
  margin: 5px 0 0;
  font-size: 13.5px;
  color: var(--ink-mute);
}

.Reason {
  color: var(--ink-soft);
}

.Count {
  font-variant-numeric: tabular-nums;
}

.Sep {
  height: 3px;
  width: 3px;
  flex-shrink: 0;
  border-radius: 999px;
  background: var(--ink-faint);
}

.Unban {
  display: inline-flex;
  height: 34px;
  flex-shrink: 0;
  align-items: center;
  border-radius: 999px;
  background: var(--glass-strong);
  box-shadow: inset 0 0 0 1px var(--edge);
  padding: 0 14px;
  font-size: 13px;
  font-weight: 500;
  color: var(--ink-soft);
  cursor: pointer;
  transition: background 180ms ease, color 180ms ease;
}

.Unban:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.1);
  color: var(--ink);
}

.Unban:disabled {
  opacity: 0.4;
}

.Empty {
  margin: 64px auto 0;
  text-align: center;
  font-size: 14px;
  color: var(--ink-faint);
}

.ban-move {
  transition: transform 260ms cubic-bezier(0.2, 0.9, 0.3, 1);
}

.ban-enter-active,
.ban-leave-active {
  transition: opacity 200ms ease, transform 260ms cubic-bezier(0.2, 0.9, 0.3, 1);
}

.ban-leave-active {
  position: absolute;
  left: 0;
  right: 0;
}

.ban-enter-from {
  opacity: 0;
  transform: translateY(-6px);
}

.ban-leave-to {
  opacity: 0;
  transform: translateX(24px);
}
</style>
