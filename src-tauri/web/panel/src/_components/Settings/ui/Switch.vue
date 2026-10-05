<script setup lang="ts">
withDefaults(defineProps<{ On: boolean; Label: string; Disabled?: boolean; Dim?: boolean }>(), {
  Disabled: false,
  Dim: false,
})

const Emit = defineEmits<{ Toggle: [] }>()
</script>

<template>
  <button
    type="button"
    role="switch"
    class="Switch"
    :class="{ On, Dim }"
    :aria-checked="On"
    :aria-label="Label"
    :disabled="Disabled"
    @click.stop="Emit('Toggle')"
  >
    <span class="Knob" />
  </button>
</template>

<style scoped>
.Switch {
  --w: 46px;
  --h: 28px;
  --k: 22px;
  --pad: 3px;
  position: relative;
  height: var(--h);
  width: var(--w);
  flex-shrink: 0;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.07);
  box-shadow: inset 0 0 0 1px var(--edge);
  cursor: pointer;
  transition: background 300ms ease, box-shadow 300ms ease, opacity 200ms ease;
}

.Switch:hover {
  box-shadow: inset 0 0 0 1px var(--edge-strong);
}

.Switch.On {
  background: var(--ink);
  box-shadow: 0 8px 24px -12px rgba(255, 255, 255, 0.45);
}

.Switch.Dim {
  opacity: 0.4;
}

.Switch:disabled {
  pointer-events: none;
}

.Knob {
  position: absolute;
  top: var(--pad);
  left: var(--pad);
  height: var(--k);
  width: var(--k);
  border-radius: 999px;
  background: rgba(244, 242, 247, 0.5);
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.35);
  transition: left 420ms var(--spring), width 240ms var(--ease), background 300ms ease;
}

.On .Knob {
  left: calc(100% - var(--k) - var(--pad));
  background: #0c0b0f;
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.3);
}

.Switch:active .Knob {
  width: calc(var(--k) + 6px);
}

.On:active .Knob {
  left: calc(100% - var(--k) - 6px - var(--pad));
}

.Switch:focus-visible {
  outline: 2px solid var(--rose);
  outline-offset: 3px;
}

@media (max-width: 1023px) {
  .Switch {
    --w: 52px;
    --h: 32px;
    --k: 26px;
  }
}
</style>
