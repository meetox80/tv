<script setup lang="ts">
import { openUrl } from "@tauri-apps/plugin-opener"

const _ProfileUrl: string = "https://t.me/kotwicha"

function _OpenProfile(): void {
  openUrl(_ProfileUrl).catch(() => {})
}
</script>

<template>
  <button type="button" class="Signature" @click="_OpenProfile">
    <span class="Wing WingLeft">made with</span>
    <span class="Heart">&#9829;</span>
    <span class="Wing WingRight">by kotwicha</span>
  </button>
</template>

<style scoped>
.Signature {
  position: relative;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  appearance: none;
  background: none;
  border: 0;
  padding: 0;
  font-family: var(--font-body);
  font-size: 0.8125rem;
  letter-spacing: 0.08em;
  color: #b9b5c6;
  cursor: pointer;
}

.Wing {
  position: absolute;
  top: 50%;
  white-space: nowrap;
  opacity: 0;
  pointer-events: none;
  transition:
    opacity 0.16s ease,
    transform 0.3s var(--ease-smooth);
}

.WingLeft {
  right: 100%;
  padding-right: 0.55em;
  transform: translate(0.75rem, -50%);
}

.WingRight {
  left: 100%;
  padding-left: 0.55em;
  transform: translate(-0.75rem, -50%);
}

.Heart {
  color: #a8908e;
  font-size: 0.875rem;
  line-height: 1;
}

.Signature:hover .Wing {
  opacity: 1;
  pointer-events: auto;
  transition:
    opacity 0.35s ease 0.05s,
    transform 0.55s var(--ease-smooth);
}

.Signature:hover .WingLeft,
.Signature:hover .WingRight {
  transform: translate(0, -50%);
}

.Signature:hover .Heart {
  animation: Beat 1.6s ease-in-out infinite;
}

@keyframes Beat {
  0%, 100% { transform: scale(1) }
  18% { transform: scale(1.22) }
  32% { transform: scale(1.04) }
  48% { transform: scale(1.16) }
}
</style>
