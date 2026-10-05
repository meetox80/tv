<script setup lang="ts">
import { computed, useSlots } from "vue"
import Tile from "./Tile.vue"
import Glyph from "./Glyph.vue"
import type { GlyphName, Tint } from "../sections"

const Props = withDefaults(defineProps<{
  Label?: string
  Detail?: string
  Icon?: GlyphName
  Tint?: Tint
  Chevron?: boolean
  Action?: boolean
  Danger?: boolean
  Stack?: boolean
  Block?: boolean
}>(), {
  Tint: "slate",
  Chevron: false,
  Action: false,
  Danger: false,
  Stack: false,
  Block: false,
})

const Emit = defineEmits<{ Press: [] }>()

const Slots = useSlots()

const Pressable = computed(() => Props.Action || Props.Chevron)

const Classes = computed(() => ({
  Pressable: Pressable.value,
  Danger: Props.Danger,
  Stack: Props.Stack,
  Block: Props.Block,
  Iconed: !!Props.Icon,
}))

const HasText = computed(() => !!Props.Label || !!Props.Detail || !!Slots.label)
</script>

<template>
  <component
    :is="Pressable ? 'button' : 'div'"
    :type="Pressable ? 'button' : undefined"
    class="Row"
    :class="Classes"
    @click="Pressable && Emit('Press')"
  >
    <slot v-if="Block" />

    <template v-else>
      <Tile v-if="Icon" :Icon="Icon" :Tint="Tint" :Size="40" class="Icon" />

      <div v-if="HasText" class="Text">
        <slot name="label">
          <p v-if="Label" class="Label">{{ Label }}</p>
        </slot>
        <p v-if="Detail || $slots.detail" class="Detail"><slot name="detail">{{ Detail }}</slot></p>
      </div>

      <div v-if="$slots.end || Chevron" class="End">
        <slot name="end" />
        <Glyph v-if="Chevron" Name="chevron" class="Chevron" />
      </div>

      <div v-if="$slots.below" class="Below">
        <slot name="below" />
      </div>
    </template>
  </component>
</template>

<style scoped>
.Row {
  position: relative;
  display: flex;
  width: 100%;
  min-height: 72px;
  flex-wrap: wrap;
  align-items: center;
  column-gap: 24px;
  row-gap: 14px;
  padding: 16px 0;
  text-align: left;
  color: var(--ink);
}

.Row::before {
  content: "";
  position: absolute;
  top: 0;
  right: 0;
  left: 0;
  border-top: 1px solid var(--line);
}

.Row.Block {
  display: block;
  padding: 22px 0;
}

.Row.Pressable {
  width: calc(100% + 28px);
  margin: 0 -14px;
  border-radius: 16px;
  padding: 16px 14px;
  cursor: pointer;
  transition: background 220ms ease;
}

.Row.Pressable::before {
  right: 14px;
  left: 14px;
}

.Row.Pressable:hover {
  background: rgba(255, 255, 255, 0.03);
}

.Row.Pressable:active {
  background: rgba(255, 255, 255, 0.05);
}

.Row.Pressable:hover::before,
.Row.Pressable:hover + .Row::before {
  opacity: 0;
}

.Row.Iconed {
  column-gap: 16px;
}

.Text {
  min-width: 0;
  flex: 1 1 220px;
}

.Label {
  margin: 0;
  font-size: 15px;
  font-weight: 500;
  line-height: 1.35;
  letter-spacing: -0.01em;
  color: var(--ink);
}

.Danger .Label {
  color: var(--danger);
}

.Detail {
  margin: 4px 0 0;
  max-width: 460px;
  font-size: 13.5px;
  line-height: 1.5;
  color: var(--ink-mute);
}

.End {
  display: flex;
  max-width: 100%;
  min-width: 0;
  flex: 0 1 auto;
  align-items: center;
  justify-content: flex-end;
  gap: 10px;
  margin-left: auto;
  font-size: 14px;
  color: var(--ink-soft);
}

.Chevron {
  font-size: 17px;
  color: var(--ink-faint);
  stroke-width: 2;
  transition: transform 280ms var(--ease), color 200ms ease;
}

.Pressable:hover .Chevron {
  color: var(--ink-soft);
  transform: translateX(3px);
}

.Below {
  width: 100%;
}

.Stack .End {
  flex: 1 1 100%;
  justify-content: flex-start;
}

@media (min-width: 1024px) {
  .Stack .End {
    flex: 1 1 340px;
    justify-content: flex-end;
  }

  .Iconed .Below {
    padding-left: 56px;
  }
}

@media (max-width: 1023px) {
  .Row {
    min-height: 68px;
  }

  .Label {
    font-size: 16px;
  }

  .Detail {
    font-size: 14px;
  }

  .End {
    font-size: 15px;
  }

  .Row.Pressable:hover {
    background: none;
  }

  .Row.Pressable:hover::before,
  .Row.Pressable:hover + .Row::before {
    opacity: 1;
  }
}
</style>
