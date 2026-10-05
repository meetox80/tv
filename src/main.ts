import { createApp } from "vue"
import App from "./App.vue"
import "./assets/css/style.css"
import { BlockZoom } from "./zoom"

BlockZoom()

createApp(App).mount("#app")
