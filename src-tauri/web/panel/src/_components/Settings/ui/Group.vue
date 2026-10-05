<script setup lang="ts">
defineProps<{ Title?: string; Footer?: string; Id?: string }>()
</script>

<template>
  <div class="Group" :class="{ Bare: !Title && !$slots.aside }" :id="Id ? `s-${Id}` : undefined" :data-anchor="Id">
    <div v-if="Title || $slots.aside" class="Head">
      <h3 v-if="Title" class="HeadTitle">{{ Title }}</h3>
      <div v-if="$slots.aside" class="Aside"><slot name="aside" /></div>
    </div>

    <div class="List">
      <slot />
    </div>

    <div v-if="Footer || $slots.footer || $slots.action" class="Foot">
      <div class="FootText">
        <slot name="footer">{{ Footer }}</slot>
      </div>
      <div v-if="$slots.action" class="FootAction"><slot name="action" /></div>
    </div>
  </div>
</template>

<style scoped>
.Group {
  display: flex;
  flex-direction: column;
  border-radius: 6px;
  scroll-margin-top: 28px;
}

.Group.Flash {
  animation: flash 1800ms var(--ease);
}

@keyframes flash {
  0%, 30% {
    background: rgba(224, 168, 168, 0.07);
    box-shadow: 0 0 0 18px rgba(224, 168, 168, 0.07);
  }
  100% {
    background: rgba(224, 168, 168, 0);
    box-shadow: 0 0 0 18px rgba(224, 168, 168, 0);
  }
}

.Head {
  display: flex;
  min-height: 28px;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding-bottom: 12px;
}

.HeadTitle {
  margin: 0;
  font-family: var(--display);
  font-size: 19px;
  font-weight: 600;
  letter-spacing: -0.025em;
  color: var(--ink);
}

.Aside {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 13px;
  color: var(--ink-mute);
}

.List {
  display: flex;
  flex-direction: column;
  border-bottom: 1px solid var(--line);
}

.Bare > .List > :deep(.Row:first-child::before) {
  display: none;
}

.Foot {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 20px;
  padding-top: 14px;
}

.FootText {
  min-width: 0;
  flex: 1;
  padding-top: 2px;
  font-size: 13px;
  line-height: 1.55;
  color: var(--ink-mute);
  text-wrap: pretty;
}

.FootText:empty {
  display: none;
}

.FootAction {
  display: flex;
  flex-shrink: 0;
  gap: 10px;
  margin-left: auto;
}

@media (max-width: 1023px) {
  .HeadTitle {
    font-size: 20px;
  }

  .Foot {
    flex-wrap: wrap;
  }

  .FootText {
    flex-basis: 100%;
    font-size: 13.5px;
  }
}
</style>
