<script setup lang="ts">
import { RouterLink, RouterView } from "vue-router";
</script>

<template>
  <div class="app-shell">
    <header class="topbar" role="banner">
      <RouterLink class="brand" to="/" aria-label="无名杀 Next 首页">
        无名杀 <span class="accent">Next</span>
      </RouterLink>
      <nav class="nav" aria-label="主导航">
        <RouterLink to="/">大厅</RouterLink>
        <RouterLink to="/battle">对局</RouterLink>
        <RouterLink to="/settings">设置</RouterLink>
        <RouterLink to="/replay">录像</RouterLink>
        <RouterLink to="/collection">武将库</RouterLink>
      </nav>
    </header>
    <main class="main" id="main" tabindex="-1">
      <RouterView />
    </main>
  </div>
</template>
