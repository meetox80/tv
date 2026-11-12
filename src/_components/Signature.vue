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
    <svg class="Heart" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
      <path d="M12 20.5c-.3 0-.6-.1-.8-.3C7.4 17 3 13.3 3 8.9 3 6.2 5.1 4 7.7 4c1.7 0 3.2.9 4.3 2.4C13.1 4.9 14.6 4 16.3 4 18.9 4 21 6.2 21 8.9c0 4.4-4.4 8.1-8.2 11.3-.2.2-.5.3-.8.3z" />
    </svg>
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
  display: block;
  width: 1rem;
  height: 1rem;
  color: #f4f2f7;
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
