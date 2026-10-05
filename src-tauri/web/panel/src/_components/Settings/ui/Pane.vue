<script setup lang="ts">
import { computed } from "vue"
import { useI18n } from "../../../i18n"
import { SectionOf, type SectionId } from "../sections"

const { t } = useI18n()

const Props = defineProps<{ Id: SectionId }>()

const Meta = computed(() => SectionOf(Props.Id))
</script>

<template>
  <section class="Pane" :id="`s-${Id}`" :data-section="Id">
    <header v-if="Meta" class="Head">
      <h2 class="Title">{{ t(Meta.Label) }}</h2>
      <p class="Lead">{{ t(Meta.Description) }}</p>
    </header>

    <slot />
  </section>
</template>

<style scoped>
.Pane {
  display: flex;
  flex-direction: column;
  gap: 48px;
  scroll-margin-top: 28px;
}

.Head {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.Title {
  margin: 0;
  font-family: var(--display);
  font-size: 44px;
  font-weight: 600;
  line-height: 1.05;
  letter-spacing: -0.035em;
  color: var(--ink);
}

.Lead {
  margin: 0;
  max-width: 520px;
  font-size: 15px;
  line-height: 1.5;
  color: var(--ink-mute);
  text-wrap: pretty;
}

@media (max-width: 1023px) {
  .Pane {
    gap: 36px;
    scroll-margin-top: 128px;
  }

  .Title {
    font-size: 34px;
  }
}
</style>
