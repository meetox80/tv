<script setup lang="ts">
import { ref, watch, nextTick, onMounted, onUnmounted } from "vue"
import Login from "./_pages/Login.vue"
import Settings from "./_pages/Settings.vue"
import * as Session from "./_api/session"
import * as Extensions from "./extensions"

const Resumed = Session.Token() !== null

const ShowLogin = ref(!Resumed)
const ShowDashboard = ref(Resumed)
const LoginLeaving = ref(false)
const DashboardEntering = ref(false)
let _ExpiryTimer: ReturnType<typeof setTimeout> | null = null

function _StopExpiryTimer(): void {
  if (!_ExpiryTimer) return
  clearTimeout(_ExpiryTimer)
  _ExpiryTimer = null
}

function _Expire(): void {
  _StopExpiryTimer()
  Session.Clear()
  ShowDashboard.value = false
  DashboardEntering.value = false
  LoginLeaving.value = false
  ShowLogin.value = true
}

function _ArmExpiry(): void {
  _StopExpiryTimer()

  const Remaining = Session.RemainingMs()
  if (Remaining <= 0) {
    if (ShowDashboard.value) _Expire()
    return
  }

  _ExpiryTimer = setTimeout(_Expire, Remaining)
}

function _OnVisibility(): void {
  if (document.hidden || !ShowDashboard.value) return
  _ArmExpiry()
}

async function OnAuthenticated() {
  _ArmExpiry()
  LoginLeaving.value = true

  setTimeout(async () => {
    ShowLogin.value = false
    ShowDashboard.value = true
    DashboardEntering.value = true
    await nextTick()

    setTimeout(() => {
      DashboardEntering.value = false
    }, 450)
  }, 300)
}

watch(ShowDashboard, (On) => {
  if (On) void Extensions.Load()
}, { immediate: true })

onMounted(() => {
  if (Resumed) _ArmExpiry()
  document.addEventListener("visibilitychange", _OnVisibility)
  window.addEventListener("panel-unauthorized", _Expire)
})

onUnmounted(() => {
  _StopExpiryTimer()
  document.removeEventListener("visibilitychange", _OnVisibility)
  window.removeEventListener("panel-unauthorized", _Expire)
})
</script>

<template>
  <div v-if="ShowLogin" class="fixed inset-0 z-10" :class="{ 'login-leave': LoginLeaving }">
    <Login @Authenticated="OnAuthenticated" />
  </div>

  <div
    v-if="ShowDashboard"
    class="Dashboard"
    :class="{ 'dash-enter': DashboardEntering }"
  >
    <div class="relative z-10 h-dvh">
      <Settings />
    </div>
  </div>
</template>

<style>
.Dashboard {
  position: relative;
  height: 100dvh;
  overflow: hidden;
  background: var(--bg);
}

.login-leave {
  animation: loginOut 0.35s cubic-bezier(0.4, 0, 0.2, 1) forwards;
}

.dash-enter {
  animation: dashIn 0.45s cubic-bezier(0.16, 1, 0.3, 1) forwards;
}

@keyframes loginOut {
  0%   { opacity: 1; transform: scale(1); filter: blur(0); }
  100% { opacity: 0; transform: scale(0.96); filter: blur(8px); }
}

@keyframes dashIn {
  0%   { opacity: 0; transform: scale(1.04); filter: blur(6px); }
  100% { opacity: 1; transform: scale(1); filter: blur(0); }
}

</style>
