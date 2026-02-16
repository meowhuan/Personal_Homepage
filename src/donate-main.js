import "@unocss/reset/tailwind.css";
import "uno.css";
import "./page-transition.css";
import { createApp } from "vue";
import DonatePage from "./DonatePage.vue";
import { installPageTransition } from "./page-transition";

installPageTransition();
createApp(DonatePage).mount("#app");
