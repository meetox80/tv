<script setup lang="ts">
import { ref, onMounted, type Ref } from "vue"
import CodeEntry from "./views/CodeEntry.vue"
import Configurator from "./views/Configurator.vue"
import { Authed, ClearToken, GetToken } from "./session"

type Phase = "code" | "configurator"

const CurrentPhase: Ref<Phase> = ref("code")

function OnCodeAccepted(): void {
  CurrentPhase.value = "configurator"
}

async function _RestoreSession(): Promise<void> {
  if (!GetToken()) return

  try {
    const Res = await Authed("/api/auth/check", { method: "POST" })

    if (Res.ok) {
      CurrentPhase.value = "configurator"
    } else {
      ClearToken()
    }
  } catch {
    ClearToken()
  }
}

function OnUnauthorized(): void {
  ClearToken()
  CurrentPhase.value = "code"
}

onMounted(() => {
  fetch("/api/connect").catch(() => {})
  _RestoreSession()
})
</script>

<template>
  <div class="relative min-h-screen bg-[#0B0B0C] text-text-primary antialiased overflow-hidden">
    <div class="relative min-h-screen">
      <Transition name="phase" mode="out-in" appear>
        <CodeEntry v-if="CurrentPhase === 'code'" key="code" @Accepted="OnCodeAccepted" />
        <Configurator v-else key="configurator" @Unauthorized="OnUnauthorized" />
      </Transition>
    </div>
  </div>
</template>

<style>
.phase-enter-active,
.phase-leave-active {
  transition: opacity 550ms cubic-bezier(0.16, 1, 0.3, 1), transform 550ms cubic-bezier(0.16, 1, 0.3, 1);
}

.phase-enter-from {
  opacity: 0;
  transform: translateY(16px);
}

.phase-leave-to {
  opacity: 0;
  transform: translateY(-12px);
}
</style>
