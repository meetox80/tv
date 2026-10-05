<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick, type Ref } from "vue"
import Signature from "./_components/Signature.vue"
import StatusScreen from "./_components/StatusScreen.vue"
import TabBar from "./_components/TabBar.vue"
import CaptchaSheet from "./_components/CaptchaSheet.vue"
import Quota from "./_components/Quota.vue"
import RequestView from "./_views/RequestView.vue"
import BoardView from "./_views/BoardView.vue"
import MineView from "./_views/MineView.vue"
import PrivacyView from "./_views/PrivacyView.vue"
import { Describe } from "./_api/messages"
import {
  ActiveTab, Banned, Board, Device, Open, RefreshSongs, RefreshState, State, StateError, Tabs, Watch, type Tab,
} from "./_state/store"

type Route = "app" | "privacy"

const _PrivacyHash = "#/prywatnosc"

const Route: Ref<Route> = ref(_Read())
const Main: Ref<HTMLElement | null> = ref(null)
const Scrolled: Ref<boolean> = ref(false)
const Docked: Ref<boolean> = ref(false)

let _Unwatch: (() => void) | null = null

function _Read(): Route {
  return location.hash === _PrivacyHash ? "privacy" : "app"
}

function _OnScroll(): void {
  Scrolled.value = window.scrollY > 4
}

function _OnHash(): void {
  Route.value = _Read()
  window.scrollTo({ top: 0 })
}

function LeavePrivacy(): void {
  if (history.length > 1 && document.referrer.startsWith(location.origin)) history.back()
  else location.hash = ""
  Route.value = "app"
}

function Go(Id: Tab): void {
  ActiveTab.value = Id
  if (Route.value === "privacy") LeavePrivacy()
}

const Phase = computed(() => {
  if (!State.value) return StateError.value ? "offline" : "loading"
  if (Banned.value) return "banned"
  if (!Open.value) return "closed"
  return "open"
})

async function Retry(): Promise<void> {
  await RefreshState()
  await RefreshSongs()
}

watch(ActiveTab, async () => {
  await nextTick()
  window.scrollTo({ top: 0, behavior: "smooth" })
  Main.value?.focus({ preventScroll: true })
})

onMounted(async () => {
  window.addEventListener("hashchange", _OnHash)
  window.addEventListener("scroll", _OnScroll, { passive: true })
  _OnScroll()
  await Retry()
  _Unwatch = Watch()
})

onUnmounted(() => {
  window.removeEventListener("hashchange", _OnHash)
  window.removeEventListener("scroll", _OnScroll)
  _Unwatch?.()
})
</script>

<template>
  <header class="Top" :class="{ Scrolled, Docked }">
    <div class="TopInner">
      <a href="#" class="Brand" aria-label="zstio-tv, strona główna" @click.prevent="LeavePrivacy">
        <img src="/zstio.png" alt="" class="Mark" width="38" height="38" />
        <span class="BrandName">zstio-tv</span>
      </a>

      <nav v-if="Phase === 'open'" class="Nav" aria-label="Sekcje">
        <button
          v-for="I in Tabs"
          :key="I.Id"
          type="button"
          class="NavItem"
          :class="{ On: Route === 'app' && ActiveTab === I.Id }"
          :aria-current="Route === 'app' && ActiveTab === I.Id ? 'page' : undefined"
          @click="Go(I.Id)"
        >
          {{ I.Label }}
        </button>
      </nav>

      <Transition name="fade">
        <Quota v-if="Phase === 'open'" class="Allowance" />
      </Transition>
    </div>
  </header>

  <div class="Shell" :class="{ HasTabs: Phase === 'open' }">
    <main ref="Main" class="Main" tabindex="-1">
      <PrivacyView v-if="Route === 'privacy'" class="Page" @Back="LeavePrivacy" />

      <div v-show="Route === 'app'" class="Page">
        <Transition name="view" mode="out-in">
          <StatusScreen v-if="Phase === 'loading'" key="loading" Tone="loading" Title="Chwileczkę…">
            Wczytujemy głosowanie.
          </StatusScreen>

          <StatusScreen v-else-if="Phase === 'offline'" key="offline" Tone="offline" Title="Brak połączenia" @Retry="Retry">
            {{ Describe(StateError) }}
          </StatusScreen>

          <StatusScreen
            v-else-if="Phase === 'banned'"
            key="banned"
            Tone="banned"
            Title="To urządzenie jest zablokowane"
            :Handle="Device?.Handle ?? null"
          >
            Z tego urządzenia nie można już proponować utworów ani głosować. Jeśli to pomyłka,
            napisz do nas i podaj poniższy identyfikator.
          </StatusScreen>

          <div v-else-if="Phase === 'closed'" key="closed" class="Closed">
            <StatusScreen Tone="closed" Title="Głosowanie jest wyłączone" class="ClosedNote" @Retry="Retry">
              Teraz nie można proponować utworów ani głosować. Zajrzyj później.
            </StatusScreen>
            <BoardView v-if="Board.length" />
          </div>

          <div v-else key="open">
            <Transition name="tab" mode="out-in">
              <KeepAlive>
                <BoardView v-if="ActiveTab === 'board'" key="board" />
                <RequestView v-else-if="ActiveTab === 'request'" key="request" @Docked="Docked = $event" />
                <MineView v-else key="mine" />
              </KeepAlive>
            </Transition>
          </div>
        </Transition>
      </div>
    </main>

    <footer class="Foot">
      <a v-if="Route !== 'privacy'" :href="_PrivacyHash" class="PolicyLink">Polityka prywatności</a>
      <Signature class="Credit" />
    </footer>
  </div>

  <TabBar v-if="Phase === 'open'" :Away="Route === 'privacy'" @Pick="Go" />
  <CaptchaSheet />
</template>

<style scoped>
.Shell {
  position: relative;
  z-index: 1;
  display: flex;
  min-height: calc(100dvh - var(--head));
  width: 100%;
  max-width: 640px;
  flex-direction: column;
  margin: 0 auto;
  padding: 22px var(--gutter) calc(20px + var(--foot));
}

.Shell.HasTabs {
  padding-bottom: calc(96px + var(--foot));
}

.Top {
  position: sticky;
  top: 0;
  z-index: 20;
  padding-top: env(safe-area-inset-top);
}

.Top::before {
  content: "";
  position: absolute;
  inset: 0;
  background: rgba(8, 7, 10, 0.8);
  box-shadow: inset 0 -1px 0 rgba(255, 255, 255, 0.06);
  backdrop-filter: blur(24px) saturate(1.5);
  -webkit-backdrop-filter: blur(24px) saturate(1.5);
  opacity: 0;
  transition: opacity 260ms ease;
  pointer-events: none;
}

.Top.Scrolled::before {
  opacity: 1;
}

.Top.Docked::before {
  box-shadow: none;
}

.TopInner {
  position: relative;
  display: flex;
  height: 64px;
  width: 100%;
  max-width: 640px;
  align-items: center;
  margin: 0 auto;
  padding: 0 var(--gutter);
}

.Brand {
  display: inline-flex;
  align-items: center;
  gap: 11px;
  color: inherit;
  text-decoration: none;
}

.Mark {
  height: 38px;
  width: 38px;
  object-fit: contain;
}

.BrandName {
  font-family: var(--font-display);
  font-size: 17px;
  font-weight: 600;
  letter-spacing: -0.02em;
}

.Nav {
  display: none;
}

.Allowance {
  margin-left: auto;
}

.Main {
  flex: 1;
  outline: none;
}

.HasTabs .Main {
  min-height: calc(100dvh - var(--head) - var(--foot) - 118px);
}

.Closed {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.ClosedNote {
  min-height: 0;
  padding-top: 20px;
  padding-bottom: 28px;
}

.Foot {
  position: fixed;
  right: 0;
  bottom: 0;
  left: 0;
  z-index: 20;
  display: flex;
  height: var(--foot);
  align-items: center;
  gap: 12px;
  padding: 0 var(--gutter) env(safe-area-inset-bottom);
  white-space: nowrap;
}

.Foot::before {
  content: "";
  position: absolute;
  inset: 0;
  z-index: -1;
  background: rgba(8, 7, 10, 0.8);
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.06);
  backdrop-filter: blur(24px) saturate(1.5);
  -webkit-backdrop-filter: blur(24px) saturate(1.5);
}

.Credit {
  margin-left: auto;
}

.PolicyLink {
  display: inline-flex;
  min-height: 40px;
  align-items: center;
  font-size: 13px;
  color: var(--color-ink-mute);
  text-decoration: none;
  transition: color 160ms ease;
}

.PolicyLink:hover {
  color: var(--color-ink);
}

@media (min-width: 900px) {
  .Shell {
    max-width: 1160px;
    padding: 40px 40px calc(28px + var(--foot));
  }

  .Shell.HasTabs {
    padding-bottom: calc(28px + var(--foot));
  }

  .HasTabs .Main {
    min-height: calc(100dvh - var(--head) - var(--foot) - 68px);
  }

  .TopInner {
    display: grid;
    height: 84px;
    max-width: 1160px;
    grid-template-columns: 1fr auto 1fr;
    padding: 0 40px;
  }

  .Nav {
    display: flex;
    gap: 2px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.04);
    box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.07);
    backdrop-filter: blur(20px) saturate(1.3);
    -webkit-backdrop-filter: blur(20px) saturate(1.3);
    padding: 4px;
  }

  .Allowance {
    grid-column: 3;
    justify-self: end;
  }

  .NavItem {
    height: 38px;
    border-radius: 999px;
    padding: 0 20px;
    font-size: 14px;
    font-weight: 500;
    color: var(--color-ink-mute);
    transition: color 180ms ease, background 220ms var(--ease-smooth);
  }

  .NavItem:hover {
    color: var(--color-ink);
  }

  .NavItem.On {
    background: rgba(255, 255, 255, 0.09);
    color: var(--color-ink);
  }

  .Foot {
    padding: 0 max(40px, calc((100% - 1080px) / 2));
  }
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 240ms ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.Page {
  animation: PageIn 420ms var(--ease-smooth);
}

@keyframes PageIn {
  from {
    opacity: 0;
    transform: translateY(12px);
  }
}

.view-enter-active,
.view-leave-active,
.tab-enter-active,
.tab-leave-active {
  transition: opacity 240ms ease, transform 420ms var(--ease-smooth);
}

.view-enter-from,
.tab-enter-from {
  opacity: 0;
  transform: translateY(12px);
}

.view-leave-to,
.tab-leave-to {
  opacity: 0;
  transform: translateY(-8px);
}
</style>
