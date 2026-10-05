<script setup lang="ts">
import { computed, ref, watch, type Ref } from "vue"
import NumField from "./NumField.vue"
import ColorRow from "./ColorRow.vue"
import Icon, { type IconName } from "./Icon.vue"
import { KindName, LayerName, type CornerRadii, type ImageFit, type Slide, type SlideElement, type TextAlign, type TextFont } from "./model"

export type Arrange = "front" | "forward" | "backward" | "back"
export type AlignTo = "left" | "center" | "right" | "top" | "middle" | "bottom"
export type Axis = "horizontal" | "vertical"
export type Dimension = "X" | "Y" | "W" | "H"

const Props = defineProps<{ Slide: Slide; Elements: SlideElement[]; Index: number; Count: number; TextHeight: number }>()

const Emit = defineEmits<{
  Patch: [Patch: Partial<SlideElement>, Final: boolean]
  Move: [Key: Dimension, Value: number, Final: boolean]
  Background: [Value: string | null]
  Arrange: [How: Arrange]
  Align: [How: AlignTo]
  Distribute: [Along: Axis]
  Duplicate: []
  Remove: []
  Replace: []
}>()

const One = computed<SlideElement | null>(() => (Props.Elements.length === 1 ? Props.Elements[0] : null))
const Many = computed(() => Props.Elements.length > 1)

function _Plural(Count: number): string {
  const Ones = Count % 10
  const Tens = Count % 100
  return Ones >= 2 && Ones <= 4 && (Tens < 12 || Tens > 14) ? "elementy" : "elementów"
}

const Title = computed(() => {
  if (One.value) return LayerName(One.value)
  if (Many.value) return `${Props.Elements.length} ${_Plural(Props.Elements.length)}`
  return `Slajd ${Props.Index + 1}`
})

const Subtitle = computed(() => {
  if (One.value) return KindName(One.value.Kind)
  if (Many.value) return "Zaznaczenie"
  return `z ${Props.Count}`
})

const HeadIcon = computed<IconName>(() => (One.value ? One.value.Kind : Many.value ? "layers" : "rect"))

const Rounded = computed(() => One.value?.Kind === "rect" || One.value?.Kind === "image")

const Independent: Ref<boolean> = ref(false)

watch(() => One.value?.Id, () => {
  Independent.value = !!One.value?.Corners
}, { immediate: true })

const CornerValues = computed<CornerRadii>(() => {
  const E = One.value
  if (E?.Corners) return E.Corners
  const R = E?.Radius ?? 0
  return [R, R, R, R]
})

const SharedRadius = computed<number | null>(() => (new Set(CornerValues.value).size === 1 ? CornerValues.value[0] : null))

const CornerFields: { Index: number; Turn: string; Label: string }[] = [
  { Index: 0, Turn: "", Label: "Lewy górny narożnik" },
  { Index: 1, Turn: "Turn1", Label: "Prawy górny narożnik" },
  { Index: 3, Turn: "Turn3", Label: "Lewy dolny narożnik" },
  { Index: 2, Turn: "Turn2", Label: "Prawy dolny narożnik" },
]

function SetCorner(Index: number, Value: number, Final: boolean): void {
  const Next = [...CornerValues.value] as CornerRadii
  Next[Index] = Value
  Emit("Patch", { Corners: Next }, Final)
}

const SharedOpacity = computed<number | null>(() => {
  const Values = new Set(Props.Elements.map(E => Math.round((E.Opacity ?? 1) * 100)))
  return Values.size === 1 ? [...Values][0] : null
})

const Fonts: { Value: TextFont; Label: string }[] = [
  { Value: "display", Label: "Space Grotesk" },
  { Value: "body", Label: "Inter" },
]

const Weights: { Value: number; Label: string }[] = [
  { Value: 400, Label: "Normalna" },
  { Value: 500, Label: "Średnia" },
  { Value: 600, Label: "Półgruba" },
  { Value: 700, Label: "Gruba" },
]

const Aligns: { Value: TextAlign; Icon: IconName; Label: string }[] = [
  { Value: "left", Icon: "text-left", Label: "Do lewej" },
  { Value: "center", Icon: "text-center", Label: "Do środka" },
  { Value: "right", Icon: "text-right", Label: "Do prawej" },
]

const Fits: { Value: ImageFit; Label: string; Title: string }[] = [
  { Value: "cover", Label: "Wypełnij", Title: "Wypełnia ramkę i przycina nadmiar" },
  { Value: "contain", Label: "Dopasuj", Title: "Cały obraz mieści się w ramce" },
  { Value: "fill", Label: "Rozciągnij", Title: "Rozciąga obraz do kształtu ramki" },
]

const Orders: { Value: Arrange; Icon: IconName; Label: string }[] = [
  { Value: "front", Icon: "front", Label: "Na wierzch" },
  { Value: "forward", Icon: "forward", Label: "Wyżej" },
  { Value: "backward", Icon: "backward", Label: "Niżej" },
  { Value: "back", Icon: "back", Label: "Na spód" },
]

const AlignTools: { Value: AlignTo; Icon: IconName; Label: string }[] = [
  { Value: "left", Icon: "align-left", Label: "Wyrównaj do lewej" },
  { Value: "center", Icon: "align-center", Label: "Wyśrodkuj w poziomie" },
  { Value: "right", Icon: "align-right", Label: "Wyrównaj do prawej" },
  { Value: "top", Icon: "align-top", Label: "Wyrównaj do góry" },
  { Value: "middle", Icon: "align-middle", Label: "Wyśrodkuj w pionie" },
  { Value: "bottom", Icon: "align-bottom", Label: "Wyrównaj do dołu" },
]

function Picked(Event: Event): string {
  return (Event.target as HTMLSelectElement).value
}
</script>

<template>
  <aside class="Inspector">
    <header class="Head">
      <span class="HeadIcon"><Icon :Name="HeadIcon" /></span>
      <div class="HeadText">
        <p class="Title">{{ Title }}</p>
        <p class="Subtitle">{{ Subtitle }}</p>
      </div>
      <div v-if="Elements.length" class="HeadTools">
        <button type="button" class="Tool" title="Duplikuj (Ctrl+D)" aria-label="Duplikuj" @click="Emit('Duplicate')"><Icon Name="duplicate" /></button>
        <button type="button" class="Tool Danger" title="Usuń (Delete)" aria-label="Usuń" @click="Emit('Remove')"><Icon Name="trash" /></button>
      </div>
    </header>

    <template v-if="!Elements.length">
      <section class="Sec">
        <p class="SecTitle">Tło</p>
        <ColorRow :Value="Slide.Background ?? null" AllowNone NoneLabel="Przezroczyste" @Pick="Emit('Background', $event)" />
        <p class="Hint">Przezroczyste tło pokazuje tło telewizora.</p>
      </section>
      <section class="Sec">
        <p class="Hint">Kliknij element, aby go zmienić. Z wciśniętym klawiszem Shift zaznaczysz kilka naraz. Obraz przeciągnij albo wklej na slajd.</p>
      </section>
    </template>

    <template v-else-if="Many">
      <section class="Sec">
        <p class="SecTitle">Wyrównanie</p>
        <div class="Icons Six">
          <button v-for="Tool in AlignTools" :key="Tool.Value" type="button" class="IconButton" :title="Tool.Label" :aria-label="Tool.Label" @click="Emit('Align', Tool.Value)">
            <Icon :Name="Tool.Icon" />
          </button>
        </div>
        <div v-if="Elements.length > 2" class="Icons Two">
          <button type="button" class="IconButton Labelled" title="Równe odstępy w poziomie" @click="Emit('Distribute', 'horizontal')">
            <Icon Name="distribute-h" /> W poziomie
          </button>
          <button type="button" class="IconButton Labelled" title="Równe odstępy w pionie" @click="Emit('Distribute', 'vertical')">
            <Icon Name="distribute-v" /> W pionie
          </button>
        </div>
      </section>
      <section class="Sec">
        <p class="SecTitle">Warstwa</p>
        <div class="Pair">
          <NumField :Value="SharedOpacity" Placeholder="Mieszane" Suffix="%" :Min="0" :Max="100" Title="Krycie" @Change="(V, F) => Emit('Patch', { Opacity: V / 100 }, F)">
            <Icon Name="opacity" />
          </NumField>
          <div class="Icons Four">
            <button v-for="Order in Orders" :key="Order.Value" type="button" class="IconButton" :title="Order.Label" :aria-label="Order.Label" @click="Emit('Arrange', Order.Value)">
              <Icon :Name="Order.Icon" />
            </button>
          </div>
        </div>
      </section>
    </template>

    <template v-else-if="One">
      <section class="Sec">
        <p class="SecTitle">Układ</p>
        <div class="Pair">
          <NumField Label="X" :Value="One.X" :Min="-3552" :Max="5328" @Change="(V, F) => Emit('Move', 'X', V, F)" />
          <NumField Label="Y" :Value="One.Y" :Min="-1396" :Max="2094" @Change="(V, F) => Emit('Move', 'Y', V, F)" />
        </div>
        <div class="Pair">
          <NumField Label="W" :Value="One.W" :Min="8" :Max="7104" Title="Szerokość" @Change="(V, F) => Emit('Move', 'W', V, F)" />
          <NumField
            v-if="One.Kind !== 'text'"
            Label="H"
            :Value="One.H"
            :Min="8"
            :Max="2792"
            Title="Wysokość"
            @Change="(V, F) => Emit('Move', 'H', V, F)"
          />
          <NumField v-else Label="H" :Value="Math.round(TextHeight)" Disabled Title="Wysokość tekstu dopasowuje się sama" />
        </div>
        <div class="Trio">
          <NumField :Value="One.Rotation ?? 0" Suffix="°" :Min="-180" :Max="180" Title="Obrót" @Change="(V, F) => Emit('Patch', { Rotation: V }, F)">
            <Icon Name="rotate" />
          </NumField>
          <template v-if="Rounded">
            <NumField
              :Value="SharedRadius"
              Placeholder="Mieszane"
              :Min="0"
              :Max="2000"
              Title="Zaokrąglenie narożników"
              @Change="(V, F) => Emit('Patch', { Radius: V, Corners: undefined }, F)"
            >
              <Icon Name="radius" />
            </NumField>
            <button
              type="button"
              class="Toggle"
              :class="{ On: Independent }"
              title="Osobne narożniki"
              aria-label="Osobne narożniki"
              :aria-pressed="Independent"
              @click="Independent = !Independent"
            >
              <Icon Name="corners" />
            </button>
          </template>
        </div>
        <div v-if="Rounded && Independent" class="Pair">
          <NumField
            v-for="Corner in CornerFields"
            :key="Corner.Index"
            :Value="CornerValues[Corner.Index]"
            :Min="0"
            :Max="2000"
            :Title="Corner.Label"
            @Change="(V, F) => SetCorner(Corner.Index, V, F)"
          >
            <Icon Name="radius" :class="Corner.Turn" />
          </NumField>
        </div>
      </section>

      <section class="Sec">
        <p class="SecTitle">Warstwa</p>
        <div class="Pair">
          <NumField :Value="Math.round((One.Opacity ?? 1) * 100)" Suffix="%" :Min="0" :Max="100" Title="Krycie" @Change="(V, F) => Emit('Patch', { Opacity: V / 100 }, F)">
            <Icon Name="opacity" />
          </NumField>
          <div class="Icons Four">
            <button v-for="Order in Orders" :key="Order.Value" type="button" class="IconButton" :title="Order.Label" :aria-label="Order.Label" @click="Emit('Arrange', Order.Value)">
              <Icon :Name="Order.Icon" />
            </button>
          </div>
        </div>
      </section>

      <section v-if="One.Kind === 'text'" class="Sec">
        <p class="SecTitle">Tekst</p>
        <div class="SelectBox">
          <select class="Select" :value="One.Font ?? 'display'" aria-label="Krój pisma" @change="Emit('Patch', { Font: Picked($event) as TextFont }, true)">
            <option v-for="Option in Fonts" :key="Option.Value" :value="Option.Value">{{ Option.Label }}</option>
          </select>
          <Icon Name="chevron" />
        </div>
        <div class="Pair">
          <div class="SelectBox">
            <select class="Select" :value="One.Weight ?? 500" aria-label="Grubość" @change="Emit('Patch', { Weight: Number(Picked($event)) }, true)">
              <option v-for="Option in Weights" :key="Option.Value" :value="Option.Value">{{ Option.Label }}</option>
            </select>
            <Icon Name="chevron" />
          </div>
          <NumField :Value="One.Size ?? 48" :Min="8" :Max="400" Title="Rozmiar" @Change="(V, F) => Emit('Patch', { Size: V }, F)">
            <Icon Name="text" />
          </NumField>
        </div>
        <div class="Icons Three">
          <button
            v-for="Option in Aligns"
            :key="Option.Value"
            type="button"
            class="IconButton"
            :class="{ On: (One.Align ?? 'left') === Option.Value }"
            :title="Option.Label"
            :aria-label="Option.Label"
            :aria-pressed="(One.Align ?? 'left') === Option.Value"
            @click="Emit('Patch', { Align: Option.Value }, true)"
          >
            <Icon :Name="Option.Icon" />
          </button>
        </div>
      </section>

      <section v-if="One.Kind === 'text'" class="Sec">
        <p class="SecTitle">Kolor</p>
        <ColorRow :Value="One.Color ?? '#f4f2f7'" @Pick="Emit('Patch', { Color: $event ?? '#f4f2f7' }, true)" />
      </section>

      <section v-else-if="One.Kind !== 'image'" class="Sec">
        <p class="SecTitle">Wypełnienie</p>
        <ColorRow :Value="One.Fill ?? '#e0a8a8'" @Pick="Emit('Patch', { Fill: $event ?? '#e0a8a8' }, true)" />
      </section>

      <section v-if="One.Kind === 'image'" class="Sec">
        <p class="SecTitle">Obraz</p>
        <div class="Icons Three">
          <button
            v-for="Option in Fits"
            :key="Option.Value"
            type="button"
            class="IconButton Labelled"
            :title="Option.Title"
            :class="{ On: (One.Fit ?? 'cover') === Option.Value }"
            :aria-pressed="(One.Fit ?? 'cover') === Option.Value"
            @click="Emit('Patch', { Fit: Option.Value }, true)"
          >
            {{ Option.Label }}
          </button>
        </div>
        <button type="button" class="Wide" @click="Emit('Replace')">Zamień obraz</button>
      </section>
    </template>
  </aside>
</template>

<style scoped>
.Inspector {
  display: flex;
  min-height: 0;
  flex-direction: column;
  overflow-y: auto;
  border-left: 1px solid var(--line);
  background: #09080b;
  scrollbar-width: thin;
  scrollbar-color: rgba(255, 255, 255, 0.08) transparent;
}

.Head {
  position: sticky;
  top: 0;
  z-index: 2;
  display: flex;
  align-items: center;
  gap: 10px;
  border-bottom: 1px solid var(--line);
  background: #09080b;
  padding: 12px 12px 12px 16px;
}

.HeadIcon {
  display: flex;
  height: 28px;
  width: 28px;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  border-radius: 7px;
  background: rgba(255, 255, 255, 0.05);
  color: var(--ink-soft);
}

.HeadText {
  min-width: 0;
  flex: 1;
}

.Title {
  margin: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 13px;
  font-weight: 600;
  color: var(--ink);
}

.Subtitle {
  margin: 1px 0 0;
  font-size: 11.5px;
  color: var(--ink-faint);
}

.HeadTools {
  display: flex;
  gap: 2px;
}

.Tool {
  display: flex;
  height: 28px;
  width: 28px;
  align-items: center;
  justify-content: center;
  border-radius: 7px;
  color: var(--ink-mute);
  cursor: pointer;
  transition: background 140ms ease, color 140ms ease;
}

.Tool:hover {
  background: rgba(255, 255, 255, 0.06);
  color: var(--ink);
}

.Tool.Danger:hover {
  background: rgba(239, 138, 138, 0.1);
  color: var(--danger);
}

.Sec {
  display: flex;
  flex-direction: column;
  gap: 8px;
  border-bottom: 1px solid var(--line);
  padding: 12px 12px 14px 16px;
}

.SecTitle {
  margin: 0 0 2px;
  font-size: 11.5px;
  font-weight: 600;
  color: var(--ink-soft);
}

.Hint {
  margin: 0;
  font-size: 12px;
  line-height: 1.5;
  color: var(--ink-faint);
}

.Pair {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  gap: 8px;
}

.Trio {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr) 30px;
  gap: 8px;
}

.Toggle {
  display: flex;
  height: 30px;
  width: 30px;
  align-items: center;
  justify-content: center;
  border-radius: 7px;
  background: rgba(255, 255, 255, 0.035);
  color: var(--ink-mute);
  cursor: pointer;
  transition: background 140ms ease, color 140ms ease;
}

.Toggle:hover {
  background: rgba(255, 255, 255, 0.06);
  color: var(--ink);
}

.Toggle.On {
  background: rgba(255, 255, 255, 0.11);
  color: var(--ink);
}

.Toggle :deep(.Icon) {
  height: 15px;
  width: 15px;
}

.Turn1 { transform: rotate(90deg); }
.Turn2 { transform: rotate(180deg); }
.Turn3 { transform: rotate(270deg); }

.Icons {
  display: grid;
  gap: 2px;
  border-radius: 7px;
  background: rgba(255, 255, 255, 0.035);
  padding: 2px;
}

.Icons.Two { grid-template-columns: repeat(2, minmax(0, 1fr)); }
.Icons.Three { grid-template-columns: repeat(3, minmax(0, 1fr)); }
.Icons.Four { grid-template-columns: repeat(4, minmax(0, 1fr)); }
.Icons.Six { grid-template-columns: repeat(6, minmax(0, 1fr)); }

.IconButton {
  display: flex;
  height: 26px;
  min-width: 0;
  align-items: center;
  justify-content: center;
  gap: 6px;
  border-radius: 5px;
  font-size: 11.5px;
  white-space: nowrap;
  color: var(--ink-mute);
  cursor: pointer;
  transition: background 140ms ease, color 140ms ease;
}

.IconButton:hover {
  background: rgba(255, 255, 255, 0.06);
  color: var(--ink);
}

.IconButton.On {
  background: rgba(255, 255, 255, 0.11);
  color: var(--ink);
}

.IconButton :deep(.Icon) {
  height: 15px;
  width: 15px;
}

.Labelled {
  overflow: hidden;
  padding: 0 6px;
}

.SelectBox {
  position: relative;
  display: flex;
  height: 30px;
  min-width: 0;
  align-items: center;
  border-radius: 7px;
  background: rgba(255, 255, 255, 0.035);
  transition: box-shadow 140ms ease;
}

.SelectBox:hover,
.SelectBox:focus-within {
  box-shadow: inset 0 0 0 1px var(--edge);
}

.SelectBox :deep(.Icon) {
  position: absolute;
  right: 7px;
  height: 13px;
  width: 13px;
  color: var(--ink-faint);
  pointer-events: none;
}

.Select {
  height: 100%;
  width: 100%;
  min-width: 0;
  appearance: none;
  border: 0;
  background: transparent;
  padding: 0 26px 0 10px;
  font: inherit;
  font-size: 12px;
  color: var(--ink);
  outline: none;
  cursor: pointer;
}

.Select option {
  background: #16151a;
  color: var(--ink);
}

.Wide {
  height: 30px;
  border-radius: 7px;
  background: rgba(255, 255, 255, 0.05);
  font-size: 12px;
  font-weight: 500;
  color: var(--ink);
  cursor: pointer;
  transition: background 140ms ease;
}

.Wide:hover {
  background: rgba(255, 255, 255, 0.09);
}
</style>
