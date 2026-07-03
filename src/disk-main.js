import "@unocss/reset/tailwind.css";
import "uno.css";
import "./page-transition.css";
import { createApp } from "vue";
import DiskPage from "./DiskPage.vue/index.js";
import { installPageTransition } from "./page-transition";

installPageTransition();
createApp(DiskPage).mount("#app");
