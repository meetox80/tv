<template>
  <p class="Signature">
    made with <span class="Heart" aria-label="love">&#9829;</span> by
    <a class="Author" href="https://t.me/kotwicha" target="_blank" rel="noopener noreferrer">kotwicha</a>
  </p>
</template>

<style scoped>
.Signature {
  font-size: 13px;
  color: var(--color-ink-faint);
}

.Heart {
  color: var(--color-rose-deep);
}

.Author {
  color: var(--color-ink-mute);
  text-decoration: none;
  transition: color 160ms ease;
}

.Author:hover {
  color: var(--color-ink);
}
</style>
