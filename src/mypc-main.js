import "@unocss/reset/tailwind.css";
import "uno.css";
import "./page-transition.css";
import { createApp } from "vue";
import MypcPage from "./MypcPage.vue/index.js";
import { installPageTransition } from "./page-transition.js";

installPageTransition();
createApp(MypcPage).mount("#app");
