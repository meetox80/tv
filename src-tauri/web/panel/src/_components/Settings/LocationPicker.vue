<script setup lang="ts">
import { ref, computed, watch, nextTick, onBeforeUnmount, type Ref } from "vue"
import L from "leaflet"
import "leaflet/dist/leaflet.css"
import { useI18n } from "../../i18n"
import Sheet from "./ui/Sheet.vue"

const { t } = useI18n()

const Props = defineProps<{
  Open: boolean
  Latitude: number
  Longitude: number
}>()

const Emit = defineEmits<{
  Close: []
  Pick: [value: { Latitude: number; Longitude: number }]
}>()

const MapEl: Ref<HTMLElement | null> = ref(null)
const Lat: Ref<number> = ref(Props.Latitude)
const Lon: Ref<number> = ref(Props.Longitude)

let _Map: L.Map | null = null
let _Marker: L.Marker | null = null

const _Icon = L.divIcon({
  className: "PickerPin",
  iconSize: [16, 16],
  iconAnchor: [8, 8],
})

function _Degrees(Value: number, Positive: string, Negative: string): string {
  return `${Math.abs(Value).toFixed(4).replace(".", ",")}° ${Value < 0 ? Negative : Positive}`
}

const Coords = computed(() => `${_Degrees(Lat.value, "N", "S")}  ·  ${_Degrees(Lon.value, "E", "W")}`)

function _Place(At: L.LatLng): void {
  Lat.value = Number(At.lat.toFixed(5))
  Lon.value = Number(At.lng.toFixed(5))
  _Marker?.setLatLng(At)
}

async function _Build(): Promise<void> {
  await nextTick()
  if (!MapEl.value || _Map) return

  _Map = L.map(MapEl.value, { attributionControl: false, zoomControl: true })
  _Map.setView([Lat.value, Lon.value], 12)

  L.tileLayer("https://tile.openstreetmap.org/{z}/{x}/{y}.png", {
    maxZoom: 19,
    crossOrigin: true,
  }).addTo(_Map)

  _Marker = L.marker([Lat.value, Lon.value], { icon: _Icon }).addTo(_Map)
  _Map.on("click", (E: L.LeafletMouseEvent) => _Place(E.latlng))

  setTimeout(() => _Map?.invalidateSize(), 60)
}

function _Destroy(): void {
  _Map?.remove()
  _Map = null
  _Marker = null
}

watch(() => Props.Open, (Open) => {
  if (!Open) return

  Lat.value = Props.Latitude
  Lon.value = Props.Longitude
  void _Build()
})

onBeforeUnmount(_Destroy)

function Confirm(): void {
  Emit("Pick", { Latitude: Lat.value, Longitude: Lon.value })
}
</script>

<template>
  <Sheet :Open="Open" :Title="t('settings.weather.pick')" :Width="720" @Close="Emit('Close')" @Closed="_Destroy">
    <div ref="MapEl" class="MapCanvas" />
    <p class="Coords">{{ Coords }}</p>

    <template #actions>
      <button type="button" class="Btn" @click="Emit('Close')">{{ t("settings.cancel") }}</button>
      <button type="button" class="Btn Primary" @click="Confirm">{{ t("settings.save") }}</button>
    </template>
  </Sheet>
</template>

<style scoped>
.MapCanvas {
  height: min(50vh, 420px);
  width: 100%;
  overflow: hidden;
  border-radius: 16px;
  background: #0e0d11;
  box-shadow: inset 0 0 0 1px var(--edge);
}

.Coords {
  margin: 4px 0 0;
  font-size: 13px;
  font-variant-numeric: tabular-nums;
  white-space: pre;
  color: var(--ink-mute);
}
</style>

<style>
.PickerPin {
  border: 2px solid #0c0b0f;
  border-radius: 999px;
  background: var(--ink);
}

.MapCanvas.leaflet-container {
  background: #0e0d11;
  cursor: crosshair;
  font-family: inherit;
}

.MapCanvas .leaflet-control-zoom {
  overflow: hidden;
  border: 0;
  border-radius: 10px;
  box-shadow: inset 0 0 0 1px var(--edge);
}

.MapCanvas .leaflet-control-zoom a {
  height: 32px;
  width: 32px;
  border: 0;
  background: #16151a;
  line-height: 32px;
  color: var(--ink-soft);
}

.MapCanvas .leaflet-control-zoom a + a {
  border-top: 1px solid var(--line);
}

.MapCanvas .leaflet-control-zoom a:hover {
  background: #201e25;
  color: var(--ink);
}
</style>
