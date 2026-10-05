import { fileURLToPath, URL } from "node:url"
import { defineConfig } from "vite"
import vue from "@vitejs/plugin-vue"
import tailwindcss from "@tailwindcss/vite"

const Tv = fileURLToPath(new URL("../../../src/views/Home/_components", import.meta.url))
const Root = fileURLToPath(new URL("../../..", import.meta.url))

export default defineConfig({
  plugins: [vue(), tailwindcss()],
  base: "./",
  resolve: {
    alias: { "@tv": Tv },
    dedupe: ["vue"],
  },
  server: {
    fs: { allow: [Root] },
  },
})
