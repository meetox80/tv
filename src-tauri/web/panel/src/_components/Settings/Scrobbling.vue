<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, type Ref } from "vue"
import { useI18n } from "../../i18n"
import { Copy } from "../../_api/clipboard"
import * as SpotifyApi from "../../_api/spotify"
import * as SerialApi from "../../_api/serial"
import * as Cache from "../../_api/cache"
import type { ScrobbleMethod } from "../../_api/settings"
import Pane from "./ui/Pane.vue"
import Group from "./ui/Group.vue"
import Row from "./ui/Row.vue"
import Switch from "./ui/Switch.vue"
import Segmented from "./ui/Segmented.vue"
import Glyph from "./ui/Glyph.vue"
import Tile from "./ui/Tile.vue"
import type { GlyphName, Tint } from "./sections"

const { t } = useI18n()

const Props = defineProps<{
  Method: ScrobbleMethod | null
  LastfmApiKey: string
  LastfmUsername: string
  LastfmSaved: boolean
  LastfmError: string
  LastfmStale: string
  LastfmBusy: boolean
  ApiKey: string
  InLessons: boolean
}>()

const Editing = ref(false)

const _CopiedMs = 1600
const _ArmMs = 3000

const Copied: Ref<string> = ref("")
const RegenArmed: Ref<boolean> = ref(false)

let _CopyTimer: ReturnType<typeof setTimeout> | null = null
let _ArmTimer: ReturnType<typeof setTimeout> | null = null

interface MethodOption {
  Id: ScrobbleMethod
  Label: string
  Detail: string
  Icon: GlyphName
  Tint: Tint
}

const Methods = computed<MethodOption[]>(() => [
  { Id: "lastfm", Label: "Last.fm", Detail: t("settings.spotify.lastfm.description"), Icon: "lastfm", Tint: "lastfm" },
  { Id: "spotify", Label: "Spotify", Detail: t("settings.spotify.spotify.description"), Icon: "spotify", Tint: "spotify" },
  { Id: "api", Label: t("settings.spotify.api"), Detail: t("settings.spotify.api.description"), Icon: "code", Tint: "indigo" },
  { Id: "serial", Label: t("settings.serial.method"), Detail: t("settings.spotify.serial.description"), Icon: "plug", Tint: "slate" },
])

function _Flash(Key: string): void {
  Copied.value = Key
  if (_CopyTimer) clearTimeout(_CopyTimer)
  _CopyTimer = setTimeout(() => {
    Copied.value = ""
    _CopyTimer = null
  }, _CopiedMs)
}

function Regenerate(): void {
  if (!RegenArmed.value) {
    RegenArmed.value = true
    if (_ArmTimer) clearTimeout(_ArmTimer)
    _ArmTimer = setTimeout(() => { RegenArmed.value = false }, _ArmMs)
    return
  }

  if (_ArmTimer) clearTimeout(_ArmTimer)
  RegenArmed.value = false
  Emit("RegenerateKey")
}

onUnmounted(() => {
  if (_CopyTimer) clearTimeout(_CopyTimer)
  if (_ArmTimer) clearTimeout(_ArmTimer)
})

const _SpotifyCacheKey = "panel_spotify_status"
const _CachedSpotify = Cache.Read<SpotifyApi.SpotifyStatus>(_SpotifyCacheKey)

const SpotifyClientId = ref(_CachedSpotify?.ClientId ?? "")
const SpotifyClientSecret = ref("")
const SpotifyConnected = ref(_CachedSpotify?.Connected ?? false)
const SpotifyHasSecret = ref(_CachedSpotify?.HasSecret ?? false)
const SpotifyRedirectUri = ref(_CachedSpotify?.RedirectUri ?? "")
const SpotifyBusy = ref(false)
const SpotifyError = ref("")

const CanConnectSpotify = computed(
  () => !!SpotifyClientId.value.trim()
    && (!!SpotifyClientSecret.value.trim() || SpotifyHasSecret.value)
    && !SpotifyBusy.value
)

async function LoadSpotify() {
  try {
    const Status = await SpotifyApi.LoadStatus()
    SpotifyClientId.value = Status.ClientId
    SpotifyConnected.value = Status.Connected
    SpotifyRedirectUri.value = Status.RedirectUri
    SpotifyHasSecret.value = Status.HasSecret
    Cache.Write(_SpotifyCacheKey, Status)
  } catch {}
}

function ReadSpotifyOutcome() {
  const Params = new URLSearchParams(globalThis.location?.search ?? "")
  const Outcome = Params.get("spotify")
  if (!Outcome) return

  if (Outcome !== "connected") SpotifyError.value = t(`settings.spotify.status.${Outcome}`)

  const Url = new URL(globalThis.location.href)
  Url.searchParams.delete("spotify")
  globalThis.history.replaceState({}, "", Url)
}

onMounted(() => {
  ReadSpotifyOutcome()
  void LoadSpotify()
  void LoadSerial()
})

async function ConnectSpotify() {
  if (!CanConnectSpotify.value) return

  SpotifyBusy.value = true
  SpotifyError.value = ""

  try {
    await SpotifyApi.SaveCredentials(SpotifyClientId.value.trim(), SpotifyClientSecret.value.trim())
    await SpotifyApi.BeginAuthorize()
  } catch {
    SpotifyError.value = t("settings.spotify.status.missing_credentials")
    SpotifyBusy.value = false
  }
}

async function DisconnectSpotify() {
  await SpotifyApi.Disconnect()
  SpotifyConnected.value = false
  SpotifyClientSecret.value = ""
  SpotifyError.value = ""
  void LoadSpotify()
}

function CopyRedirectUri() {
  Copy(SpotifyRedirectUri.value)
  _Flash("redirect")
}

const LastfmConnected = computed(() => Props.LastfmSaved && !Editing.value)

const CanSaveLastfm = computed(
  () => !!Props.LastfmApiKey.trim() && !!Props.LastfmUsername.trim() && !Props.LastfmBusy
)

function _LastfmText(Code: string): string {
  if (!Code) return ""
  const Key = `settings.lastfm.error.${Code}`
  const Text = t(Key)
  return Text === Key ? t("settings.lastfm.error.unknown") : Text
}

const LastfmMessage = computed(() => _LastfmText(Props.LastfmError))
const StaleMessage = computed(() => _LastfmText(Props.LastfmStale))

watch(
  () => Props.LastfmBusy,
  (Now, Before) => {
    if (Before && !Now && !Props.LastfmError) Editing.value = false
  }
)

const Emit = defineEmits<{
  "update:Method": [value: ScrobbleMethod]
  "update:LastfmApiKey": [value: string]
  "update:LastfmUsername": [value: string]
  SaveLastfm: []
  RegenerateKey: []
  "update:InLessons": [Enabled: boolean]
}>()

const _SerialCacheKey = "panel_serial_status"
const _CachedSerial = Cache.Read<SerialApi.SerialStatus>(_SerialCacheKey)
const _Bauds = [9600, 19200, 38400, 57600, 115200, 230400]

const SerialPort = ref(_CachedSerial?.Port ?? "")
const SerialBaud = ref(_CachedSerial?.Baud ?? 115200)
const SerialPorts = ref<string[]>(_CachedSerial?.Ports ?? [])
const SerialPublicKey = ref(_CachedSerial?.PublicKey ?? "")
const SerialFingerprint = ref(_CachedSerial?.Fingerprint ?? "")
const SerialPeerKey = ref(_CachedSerial?.PeerKey ?? "")
const SerialOpen = ref(_CachedSerial?.Open ?? false)
const SerialFault = ref(_CachedSerial?.Error ?? "")
const SerialSaving = ref(false)

function _KeyBytes(Text: string): number {
  const Clean = Text.trim().replace(/-/g, "+").replace(/_/g, "/")
  if (!Clean) return 0

  try {
    return atob(Clean).length
  } catch {
    return -1
  }
}

const PeerKeyState = computed(() => {
  const Size = _KeyBytes(SerialPeerKey.value)
  if (Size === 0) return "empty"
  return Size === 32 ? "ok" : "bad"
})

function _ApplySerial(Status: SerialApi.SerialStatus): void {
  SerialPort.value = Status.Port
  SerialBaud.value = Status.Baud
  SerialPorts.value = Status.Ports
  SerialPublicKey.value = Status.PublicKey
  SerialFingerprint.value = Status.Fingerprint
  SerialPeerKey.value = Status.PeerKey
  SerialOpen.value = Status.Open
  SerialFault.value = Status.Error ?? ""
  Cache.Write(_SerialCacheKey, Status)
}

async function LoadSerial(): Promise<void> {
  try {
    _ApplySerial(await SerialApi.LoadStatus())
  } catch {}
}

const SerialMessage = computed(() => {
  if (!SerialFault.value) return ""
  const Key = `settings.serial.error.${SerialFault.value}`
  const Text = t(Key)
  return Text === Key ? t("settings.serial.error.unknown") : Text
})

const CanSaveSerial = computed(
  () => !!SerialPort.value.trim() && PeerKeyState.value === "ok" && !SerialSaving.value
)

async function SaveSerial(): Promise<void> {
  if (!CanSaveSerial.value) return

  SerialSaving.value = true

  try {
    const Result = await SerialApi.SaveConfig(
      SerialPort.value.trim(),
      SerialBaud.value,
      SerialPeerKey.value.trim(),
    )

    SerialFault.value = Result.Ok ? "" : Result.Error || "unknown"
    if (Result.Ok) await LoadSerial()
  } catch {
    SerialFault.value = "unknown"
  } finally {
    SerialSaving.value = false
  }
}

async function RegenerateSerial(): Promise<void> {
  try {
    _ApplySerial(await SerialApi.RegenerateKeys())
  } catch {}
}

function CopySerialKey(): void {
  Copy(SerialPublicKey.value)
  _Flash("serial")
}

type CodeTabId = "curl" | "fetch" | "python"

const CodeTab = ref<CodeTabId>("curl")

const CodeTabs: { Value: CodeTabId; Label: string }[] = [
  { Value: "curl", Label: "curl" },
  { Value: "fetch", Label: "fetch" },
  { Value: "python", Label: "Python" },
]

const Origin = computed(() => globalThis.location?.origin || "http://localhost:9999")

const Endpoint = computed(() => `${Origin.value}/api/playback/poke?key=${Props.ApiKey}`)

const CoverEndpoint = computed(() => `${Origin.value}/api/playback/cover?key=${Props.ApiKey}`)

const Examples = computed<Record<string, string>>(() => ({
  curl: [
    `curl -X POST "${Endpoint.value}" \\`,
    `  -H "Content-Type: application/json" \\`,
    `  -d '{"Title": "Never Gonna Give You Up", "Artist": "Rick Astley",`,
    `       "CoverUrl": "https://example.com/cover.jpg"}'`,
    ``,
    `curl -X POST "${CoverEndpoint.value}" \\`,
    `  -H "Content-Type: image/jpeg" \\`,
    `  --data-binary @cover.jpg`,
  ].join("\n"),

  fetch: [
    `await fetch("${Endpoint.value}", {`,
    `  method: "POST",`,
    `  headers: { "Content-Type": "application/json" },`,
    `  body: JSON.stringify({`,
    `    Title: "Never Gonna Give You Up",`,
    `    Artist: "Rick Astley",`,
    `    Cover: "data:image/jpeg;base64,/9j/4AAQ..."`,
    `  })`,
    `})`,
  ].join("\n"),

  python: [
    `import requests`,
    ``,
    `requests.post("${Endpoint.value}", json={`,
    `    "Title": "Never Gonna Give You Up",`,
    `    "Artist": "Rick Astley",`,
    `    "CoverUrl": "https://example.com/cover.jpg",`,
    `})`,
    ``,
    `requests.post(`,
    `    "${CoverEndpoint.value}",`,
    `    headers={"Content-Type": "image/jpeg"},`,
    `    data=open("cover.jpg", "rb").read(),`,
    `)`,
  ].join("\n"),
}))

function CopyKey() {
  Copy(Props.ApiKey)
  _Flash("key")
}

function CopyExample() {
  Copy(Examples.value[CodeTab.value])
  _Flash("example")
}
</script>

<template>
  <Pane Id="scrobbling">
    <div class="Sources" role="radiogroup" :aria-label="t('settings.scrobbling.source')">
      <button
        v-for="M in Methods"
        :key="M.Id"
        type="button"
        role="radio"
        class="Source"
        :class="{ On: Method === M.Id }"
        :aria-checked="Method === M.Id"
        @click="Emit('update:Method', M.Id)"
      >
        <span class="SourceTop">
          <Tile :Icon="M.Icon" :Tint="M.Tint" :Size="44" />
          <span class="Tick" aria-hidden="true"><Glyph Name="check" /></span>
        </span>
        <span class="SourceName">{{ M.Label }}</span>
        <span class="SourceDetail">{{ M.Detail }}</span>
      </button>
    </div>

    <Transition name="swap" mode="out-in">
      <div v-if="Method === 'lastfm'" key="lastfm" class="Flow">
        <Group v-if="!LastfmConnected" :Title="t('settings.lastfm.account')">
          <Row :Label="t('settings.lastfm.apikey.label')" Stack>
            <template #end>
              <input
                :value="LastfmApiKey"
                type="text"
                spellcheck="false"
                autocomplete="off"
                :placeholder="t('settings.lastfm.apikey.placeholder')"
                class="Input Monospace Stretch"
                @input="Emit('update:LastfmApiKey', ($event.target as HTMLInputElement).value)"
                @keydown.enter="CanSaveLastfm && Emit('SaveLastfm')"
              />
            </template>
          </Row>
          <Row :Label="t('settings.lastfm.username.label')" Stack>
            <template #end>
              <input
                :value="LastfmUsername"
                type="text"
                spellcheck="false"
                autocomplete="off"
                :placeholder="t('settings.lastfm.username.placeholder')"
                class="Input Stretch"
                @input="Emit('update:LastfmUsername', ($event.target as HTMLInputElement).value)"
                @keydown.enter="CanSaveLastfm && Emit('SaveLastfm')"
              />
            </template>
          </Row>

          <template #footer>
            <span v-if="LastfmMessage" class="Fault">{{ LastfmMessage }}</span>
            <template v-else>
              {{ t("settings.lastfm.hint") }}
              <a href="https://www.last.fm/api/account/create" target="_blank" rel="noopener" class="Link">last.fm/api/account/create</a>
            </template>
          </template>
          <template #action>
            <button v-if="LastfmSaved" type="button" class="Btn" @click="Editing = false">{{ t("settings.cancel") }}</button>
            <button type="button" class="Btn Primary" :disabled="!CanSaveLastfm" @click="Emit('SaveLastfm')">
              {{ LastfmBusy ? t("settings.lastfm.verifying") : t("settings.save") }}
            </button>
          </template>
        </Group>

        <Group v-else :Title="t('settings.lastfm.account')">
          <Row :Label="LastfmUsername" :Detail="t('settings.lastfm.connected.short')" Icon="lastfm" Tint="lastfm">
            <template #end>
              <button type="button" class="Btn" @click="Editing = true">{{ t("settings.lastfm.edit") }}</button>
            </template>
          </Row>

          <template v-if="LastfmStale" #footer>
            <span class="Fault">{{ t("settings.lastfm.stale") }} {{ StaleMessage }}</span>
          </template>
        </Group>
      </div>

      <div v-else-if="Method === 'spotify'" key="spotify" class="Flow">
        <Group :Title="t('settings.spotify.app')">
          <Row v-if="SpotifyConnected" :Label="t('settings.spotify.connected')" Icon="spotify" Tint="spotify">
            <template #end>
              <button type="button" class="Btn" @click="DisconnectSpotify">{{ t("settings.spotify.disconnect") }}</button>
            </template>
          </Row>

          <Row :Label="t('settings.spotify.redirect.label')" Stack>
            <template #end>
              <span class="Value Monospace">{{ SpotifyRedirectUri }}</span>
              <button type="button" class="Btn" @click="CopyRedirectUri">
                {{ Copied === "redirect" ? t("settings.copied") : t("settings.copy") }}
              </button>
            </template>
          </Row>

          <template v-if="!SpotifyConnected">
            <Row Label="Client ID" Stack>
              <template #end>
                <input
                  v-model="SpotifyClientId"
                  type="text"
                  spellcheck="false"
                  autocomplete="off"
                  placeholder="Client ID"
                  class="Input Monospace Stretch"
                />
              </template>
            </Row>
            <Row Label="Client secret" Stack>
              <template #end>
                <input
                  v-model="SpotifyClientSecret"
                  type="password"
                  autocomplete="new-password"
                  :placeholder="SpotifyHasSecret ? '••••••••••••••••' : 'Client secret'"
                  class="Input Monospace Stretch"
                  @keydown.enter="ConnectSpotify"
                />
              </template>
            </Row>
          </template>

          <template #footer>
            <span v-if="SpotifyError" class="Fault">{{ SpotifyError }}</span>
            <template v-else>
              {{ t("settings.spotify.redirect.hint") }}
              <a href="https://developer.spotify.com/dashboard" target="_blank" rel="noopener" class="Link">developer.spotify.com/dashboard</a>
            </template>
          </template>
          <template v-if="!SpotifyConnected" #action>
            <button type="button" class="Btn Primary" :disabled="!CanConnectSpotify" @click="ConnectSpotify">
              {{ t("settings.spotify.connect") }}
            </button>
          </template>
        </Group>
      </div>

      <div v-else-if="Method === 'api'" key="api" class="Flow">
        <Group :Title="t('settings.scrobbling.api.label')">
          <Row Block>
            <div class="KeyLine">
              <span class="KeyValue Monospace">{{ ApiKey }}</span>
              <div class="KeyActions">
                <button type="button" class="Btn" @click="CopyKey">
                  {{ Copied === "key" ? t("settings.copied") : t("settings.copy") }}
                </button>
                <button type="button" class="Btn" :class="{ Armed: RegenArmed }" @click="Regenerate">
                  {{ RegenArmed ? t("settings.regenerate.confirm") : t("settings.regenerate") }}
                </button>
              </div>
            </div>
          </Row>

          <template #footer>
            <span v-if="RegenArmed" class="Fault">{{ t("settings.regenerate.warning") }}</span>
          </template>
        </Group>

        <Group :Title="t('settings.scrobbling.example')">
          <template #aside>
            <Segmented :Options="CodeTabs" :Value="CodeTab" @Change="CodeTab = $event" />
          </template>

          <Row Block>
            <div class="CodeBox">
              <button type="button" class="CodeCopy" :aria-label="t('settings.copy')" @click="CopyExample">
                <Transition name="swap" mode="out-in">
                  <Glyph v-if="Copied === 'example'" key="done" Name="check" />
                  <Glyph v-else key="copy" Name="copy" />
                </Transition>
              </button>
              <pre class="Code Monospace">{{ Examples[CodeTab] }}</pre>
            </div>
          </Row>

          <template #footer>
            {{ t("settings.scrobbling.docs") }}
            <a href="/llms.txt?documentation=scrobbling" target="_blank" rel="noopener" class="Link">llms.txt</a>
          </template>
        </Group>
      </div>

      <div v-else-if="Method === 'serial'" key="serial" class="Flow">
        <Group :Title="t('settings.serial.connection')">
          <template #aside>
            <span>{{ SerialOpen ? t("settings.serial.open") : t("settings.serial.closed") }}</span>
          </template>

          <Row :Label="t('settings.serial.port')" Stack>
            <template #end>
              <input
                v-model="SerialPort"
                type="text"
                list="serial-ports"
                spellcheck="false"
                autocomplete="off"
                :placeholder="t('settings.serial.port.placeholder')"
                class="Input Monospace Stretch"
              />
              <datalist id="serial-ports">
                <option v-for="Name in SerialPorts" :key="Name" :value="Name" />
              </datalist>
            </template>
          </Row>

          <Row :Label="t('settings.serial.baud')">
            <template #end>
              <select v-model.number="SerialBaud" class="Input Select">
                <option v-for="Rate in _Bauds" :key="Rate" :value="Rate">{{ Rate }}</option>
              </select>
            </template>
          </Row>

          <template #footer>
            <template v-if="SerialPorts.length">
              {{ t("settings.serial.found") }} <span class="Monospace">{{ SerialPorts.join(", ") }}</span>
            </template>
            <template v-else>{{ t("settings.serial.none") }}</template>
          </template>
          <template #action>
            <button type="button" class="Btn" @click="LoadSerial">{{ t("settings.serial.rescan") }}</button>
          </template>
        </Group>

        <Group :Title="t('settings.serial.pairing')" Id="scrobbling-serial">
          <Row :Label="t('settings.serial.ourkey')">
            <template #detail>
              {{ t("settings.serial.fingerprint") }} <span class="Monospace">{{ SerialFingerprint }}</span>
            </template>
            <template #end>
              <button type="button" class="Btn" @click="CopySerialKey">
                {{ Copied === "serial" ? t("settings.copied") : t("settings.copy") }}
              </button>
              <button type="button" class="Btn" @click="RegenerateSerial">{{ t("settings.serial.regenerate") }}</button>
            </template>
            <template #below>
              <p class="KeyBox Monospace">{{ SerialPublicKey }}</p>
            </template>
          </Row>

          <Row :Label="t('settings.serial.peerkey')">
            <template #below>
              <div class="PeerField">
                <input
                  v-model="SerialPeerKey"
                  type="text"
                  spellcheck="false"
                  autocomplete="off"
                  :placeholder="t('settings.serial.peerkey.placeholder')"
                  class="Input Monospace"
                  :class="{ Invalid: PeerKeyState === 'bad' }"
                  @keydown.enter="SaveSerial"
                />
                <Transition name="swap">
                  <Glyph v-if="PeerKeyState === 'ok'" Name="check" class="PeerOk" />
                </Transition>
              </div>
            </template>
          </Row>

          <template #footer>
            <span v-if="PeerKeyState === 'bad'" class="Fault">{{ t("settings.serial.peerkey.invalid") }}</span>
            <span v-else-if="SerialMessage" class="Fault">{{ SerialMessage }}</span>
            <template v-else>
              {{ t("settings.serial.pairing.description") }}
              <a href="/llms.txt?documentation=serial" target="_blank" rel="noopener" class="Link">{{ t("settings.serial.docs") }}</a>
            </template>
          </template>
          <template #action>
            <button type="button" class="Btn Primary" :disabled="!CanSaveSerial" @click="SaveSerial">
              {{ SerialSaving ? t("settings.serial.saving") : t("settings.save") }}
            </button>
          </template>
        </Group>
      </div>
    </Transition>

    <Group Id="scrobbling-lessons">
      <Row :Label="t('settings.scrobbling.lessons')" :Detail="t('settings.scrobbling.lessons.description')">
        <template #end>
          <Switch :On="InLessons" :Label="t('settings.scrobbling.lessons')" @Toggle="Emit('update:InLessons', !InLessons)" />
        </template>
      </Row>
    </Group>
  </Pane>
</template>

<style scoped>
.KeyLine {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 10px 14px;
}

.KeyValue {
  min-width: 0;
  flex: 1 1 260px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 13.5px;
  color: var(--ink);
  user-select: all;
}

.KeyActions {
  display: flex;
  gap: 8px;
}

.KeyBox {
  margin: 0;
  overflow-wrap: anywhere;
  border-radius: 8px;
  background: rgba(0, 0, 0, 0.22);
  box-shadow: inset 0 0 0 0.5px var(--edge);
  padding: 9px 11px;
  font-size: 12.5px;
  line-height: 1.5;
  color: var(--ink);
  user-select: all;
}

.PeerField {
  position: relative;
}

.PeerField .Input {
  padding-right: 36px;
}

.PeerOk {
  position: absolute;
  top: 50%;
  right: 11px;
  margin-top: -8px;
  font-size: 16px;
  color: var(--good);
  stroke-width: 2.6;
}

.Select {
  width: auto;
  min-width: 120px;
  appearance: none;
  -webkit-appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='rgba(235,235,245,0.6)' stroke-width='2.4' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpath d='m8 10 4-4 4 4M8 14l4 4 4-4'/%3E%3C/svg%3E");
  background-position: right 8px center;
  background-repeat: no-repeat;
  background-size: 14px;
  padding-right: 30px;
  font-variant-numeric: tabular-nums;
}

.Select option {
  background: #1c1c1e;
  color: #fff;
}

.Sources {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 12px;
}

.Source {
  position: relative;
  isolation: isolate;
  display: flex;
  min-height: 168px;
  flex-direction: column;
  align-items: flex-start;
  overflow: hidden;
  border-radius: 22px;
  background: rgba(255, 255, 255, 0.03);
  box-shadow: inset 0 0 0 1px var(--edge);
  padding: 16px 16px 18px;
  text-align: left;
  cursor: pointer;
  transition: box-shadow 260ms ease, background 260ms ease, transform 200ms ease;
}

.Source:hover {
  background: rgba(255, 255, 255, 0.045);
  box-shadow: inset 0 0 0 1px var(--edge-strong);
}

.Source:active {
  transform: scale(0.985);
}

.Source.On {
  background: rgba(255, 255, 255, 0.05);
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.28);
}

.SourceTop {
  display: flex;
  width: 100%;
  align-items: flex-start;
  justify-content: space-between;
  margin-bottom: auto;
}

.Tick {
  display: flex;
  height: 24px;
  width: 24px;
  align-items: center;
  justify-content: center;
  border-radius: 999px;
  box-shadow: inset 0 0 0 1.5px rgba(255, 255, 255, 0.16);
  font-size: 13px;
  color: #0c0b0f;
  stroke-width: 3;
  transition: background 260ms ease, box-shadow 260ms ease;
}

.Tick :deep(.Glyph) {
  opacity: 0;
  transform: scale(0.4);
  transition: opacity 180ms ease, transform 420ms var(--spring);
}

.On .Tick {
  background: var(--ink);
  box-shadow: 0 6px 16px -6px rgba(0, 0, 0, 0.6);
}

.On .Tick :deep(.Glyph) {
  opacity: 1;
  transform: none;
}

.SourceName {
  margin-top: 22px;
  font-family: var(--display);
  font-size: 18px;
  font-weight: 600;
  letter-spacing: -0.02em;
  color: var(--ink);
}

.SourceDetail {
  margin-top: 4px;
  font-size: 12.5px;
  line-height: 1.45;
  color: var(--ink-mute);
}

.On .SourceDetail {
  color: var(--ink-soft);
}

.CodeBox {
  position: relative;
  overflow: hidden;
  border-radius: 18px;
  background: rgba(0, 0, 0, 0.28);
  box-shadow: inset 0 0 0 1px var(--edge);
}

.Code {
  margin: 0;
  overflow-x: auto;
  padding: 18px 56px 18px 20px;
  font-size: 12.5px;
  line-height: 1.65;
  color: rgba(235, 235, 245, 0.78);
  user-select: text;
  scrollbar-width: thin;
}

.CodeCopy {
  position: absolute;
  top: 10px;
  right: 10px;
  z-index: 1;
  display: flex;
  height: 30px;
  width: 30px;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
  background: rgba(40, 40, 44, 0.85);
  box-shadow: inset 0 0 0 0.5px var(--edge);
  font-size: 15px;
  color: var(--ink-mute);
  transition: color 150ms ease, background 150ms ease;
}

.CodeCopy:hover {
  background: rgba(58, 58, 62, 0.95);
  color: var(--ink);
}
@media (max-width: 1023px) {
  .Sources {
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 10px;
  }

  .Source {
    min-height: 156px;
    border-radius: 20px;
  }

  .SourceDetail {
    font-size: 13px;
  }
}
</style>
