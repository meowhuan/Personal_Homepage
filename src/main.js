import "@unocss/reset/tailwind.css";
import "uno.css";
import "./page-transition.css";
import { createApp } from "vue";
import App from "./App.vue";
import { installPageTransition } from "./page-transition";

installPageTransition();
createApp(App).mount("#app");
