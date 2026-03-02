import "@unocss/reset/tailwind.css";
import "uno.css";
import "./page-transition.css";
import { createApp } from "vue";
import FriendsPage from "./FriendsPage.vue";
import { installPageTransition } from "./page-transition";

installPageTransition();
createApp(FriendsPage).mount("#app");
