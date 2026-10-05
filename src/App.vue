<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick, type Ref } from "vue"
import { invoke } from "@tauri-apps/api/core"
import StartView from "./views/StartView/StartView.vue"
import SetupView from "./views/SetupView/SetupView.vue"
import HomeView from "./views/Home/Home.vue"
import * as Frames from "./frames"

type Phase = "loading" | "start" | "setup" | "home"

const _FontDeadlineMs: number = 120

const CurrentPhase: Ref<Phase> = ref("loading")
const Paired: Ref<boolean> = ref(false)

function _EnterSetup(): void {
  Paired.value = true
  CurrentPhase.value = "setup"
}

function _LeaveSetup(): void {
  invoke("reset_client_connection").catch(() => {})
  CurrentPhase.value = "start"
}

function _Deadline(Ms: number): Promise<void> {
  return new Promise(Resolve => setTimeout(Resolve, Ms))
}

function _OnKeydown(Event: KeyboardEvent): void {
  if ((Event.ctrlKey || Event.metaKey) && Event.key.toLowerCase() === "q") {
    Event.preventDefault()
    invoke("quit_app").catch(() => {})
  }
}

function _NextPaint(): Promise<void> {
  return new Promise(Resolve => {
    requestAnimationFrame(() => requestAnimationFrame(() => Resolve()))
  })
}

onMounted(async () => {
  window.addEventListener("keydown", _OnKeydown)
  Frames.Start()

  let Done: boolean = false
  try {
    Done = await invoke<boolean>("is_setup_complete")
  } catch {}

  CurrentPhase.value = Done ? "home" : "start"

  await Promise.race([document.fonts.ready, _Deadline(_FontDeadlineMs)])
  await nextTick()
  await _NextPaint()

  invoke("show_window").catch(() => {})
})

onUnmounted(() => {
  window.removeEventListener("keydown", _OnKeydown)
  Frames.Stop()
})
</script>

<template>
  <div class="relative h-screen w-full overflow-hidden bg-[#0B0B0C] antialiased">
    <Transition name="view">
      <StartView v-if="CurrentPhase === 'start'" class="Layer" :Paired="Paired" @Done="_EnterSetup" />
      <SetupView
        v-else-if="CurrentPhase === 'setup'"
        class="Layer"
        @Back="_LeaveSetup"
        @Done="CurrentPhase = 'home'"
      />
      <HomeView v-else-if="CurrentPhase === 'home'" class="Layer" />
    </Transition>
  </div>
</template>

<style scoped>
.Layer {
  position: absolute;
  inset: 0;
}

.view-enter-active {
  transition: opacity 0.3s ease;
  z-index: 1;
}

.view-leave-active {
  transition: opacity 0.3s ease;
  z-index: 0;
}

.view-enter-from,
.view-leave-to {
  opacity: 0;
}
</style>
