<script setup lang="ts">
export interface StatItem {
  Key: string
  Label: string
  Value: string
  Unit?: string
  Accent?: boolean
  Faint?: boolean
}

defineProps<{ Items: StatItem[] }>()
</script>

<template>
  <div class="Stats" :class="{ Pairs: Items.length === 4 }" :style="{ '--count': Items.length }">
    <div v-for="Item in Items" :key="Item.Key" class="Stat" :class="{ Accent: Item.Accent, Faint: Item.Faint }">
      <span class="StatValue">
        {{ Item.Value }}<span v-if="Item.Unit" class="StatUnit">{{ Item.Unit }}</span>
      </span>
      <span class="StatLabel">{{ Item.Label }}</span>
    </div>
  </div>
</template>

<style scoped>
.Stats {
  display: grid;
  grid-template-columns: repeat(var(--count), minmax(0, 1fr));
}

.Stat {
  position: relative;
  display: flex;
  min-width: 0;
  flex-direction: column;
  gap: 8px;
  padding: 0 20px;
}

.Stat:first-child {
  padding-left: 0;
}

.Stat + .Stat::before {
  content: "";
  position: absolute;
  top: 4px;
  bottom: 4px;
  left: 0;
  border-left: 1px solid var(--line);
}

.StatValue {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-family: var(--display);
  font-size: 44px;
  font-weight: 600;
  line-height: 0.95;
  letter-spacing: -0.04em;
  font-variant-numeric: tabular-nums;
  color: var(--ink);
}

.StatUnit {
  margin-left: 0.18em;
  font-size: 0.42em;
  font-weight: 500;
  letter-spacing: 0;
  color: var(--ink-mute);
}

.Accent .StatValue {
  color: var(--rose);
}

.Faint .StatValue {
  color: var(--ink-faint);
}

.StatLabel {
  font-size: 13px;
  color: var(--ink-mute);
}

@media (max-width: 1023px) {
  .Stats.Pairs {
    grid-template-columns: repeat(2, minmax(0, 1fr));
    row-gap: 24px;
  }

  .Pairs .Stat:nth-child(3) {
    padding-left: 0;
  }

  .Pairs .Stat:nth-child(3)::before {
    display: none;
  }

  .StatValue {
    font-size: 40px;
  }

  .StatLabel {
    font-size: 13.5px;
  }
}

@media (max-width: 639px) {
  .Stats:not(.Pairs) .Stat {
    padding: 0 12px;
  }

  .Stats:not(.Pairs) .Stat:first-child {
    padding-left: 0;
  }

  .Stats:not(.Pairs) .StatValue {
    font-size: 30px;
  }
}
</style>
