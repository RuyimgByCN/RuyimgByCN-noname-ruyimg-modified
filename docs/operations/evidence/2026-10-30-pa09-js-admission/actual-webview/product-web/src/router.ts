import { createRouter, createWebHashHistory, createWebHistory } from "vue-router";
import { isTauriShell } from "./engine/desktopHost";
import HomeView from "./views/HomeView.vue";
import BattleView from "./views/BattleView.vue";
import SettingsView from "./views/SettingsView.vue";
import ReplayView from "./views/ReplayView.vue";
import CollectionView from "./views/CollectionView.vue";
import ExtensionsView from "./views/ExtensionsView.vue";

export const router = createRouter({
  history: isTauriShell() ? createWebHashHistory() : createWebHistory(),
  routes: [
    { path: "/", name: "home", component: HomeView },
    { path: "/battle", name: "battle", component: BattleView },
    { path: "/extensions", name: "extensions", component: ExtensionsView },
    { path: "/settings", name: "settings", component: SettingsView },
    { path: "/replay", name: "replay", component: ReplayView },
    { path: "/collection", name: "collection", component: CollectionView },
  ],
  scrollBehavior: () => ({ top: 0 }),
});
