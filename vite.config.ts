import { defineConfig } from "vite"
import vue from "@vitejs/plugin-vue"
import tailwindcss from "@tailwindcss/vite"

const _Host = "localhost"
const _Port = 1420

export default defineConfig({
  plugins: [
    vue(),
    tailwindcss(),
  ],
  clearScreen: false,
  server: {
    host: _Host,
    port: _Port,
    strictPort: true,
    hmr: {
      protocol: "ws",
      host: _Host,
      port: _Port,
    },
  },
})
