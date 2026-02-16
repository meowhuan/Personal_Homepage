import "@unocss/reset/tailwind.css";
import "uno.css";
import "./page-transition.css";
import { createApp } from "vue";
import BlogPage from "./BlogPage.vue";
import { installPageTransition } from "./page-transition";

installPageTransition();
createApp(BlogPage).mount("#app");
