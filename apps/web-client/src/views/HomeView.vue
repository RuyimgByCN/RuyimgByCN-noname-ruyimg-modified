<script setup lang="ts">
import { useRouter } from "vue-router";
import { computed, onMounted, ref } from "vue";
import {
  desktopPlatformInfo,
  type DesktopPlatformInfo,
} from "../engine/desktopHost";

const router = useRouter();
const players = ref(4);
const seed = ref(42);
const displayName = ref(`player-${Math.floor(Math.random() * 900 + 100)}`);
const password = ref("");
const roomId = ref(1);
const wsUrl = ref(defaultWsUrl());
const onlineError = ref("");
const desktopPlatform = ref<DesktopPlatformInfo | null>(null);

onMounted(async () => {
  desktopPlatform.value = await desktopPlatformInfo();
});

function goBattle() {
  router.push({
    path: "/battle",
    query: { players: String(players.value), seed: String(seed.value) },
  });
}

function goOnline(action: "create" | "join" | "spectate") {
  onlineError.value = "";
  if (!wsUrl.value.trim()) {
    onlineError.value = "请填写 WebSocket 地址";
    return;
  }
  if (action !== "create" && roomId.value < 1) {
    onlineError.value = "房间号必须大于 0";
    return;
  }
  sessionStorage.setItem(
    "noname-next.online-intent",
    JSON.stringify({
      action,
      wsUrl: wsUrl.value,
      displayName: displayName.value,
      password: password.value,
      roomId: roomId.value,
      players: players.value,
      seed: seed.value,
    })
  );
  void router.push({
    path: "/battle",
    query: {
      mode: "online",
      action,
      ...(action === "create" ? {} : { room: String(roomId.value) }),
    },
  });
}

function defaultWsUrl(): string {
  if (typeof window === "undefined") return "ws://127.0.0.1:7700/ws";
  const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
  const host = window.location.hostname || "127.0.0.1";
  return `${protocol}//${host}:7700/ws`;
}

const roomLabel = computed(() => (roomId.value > 0 ? `#${roomId.value}` : "—"));
</script>

<template>
  <div class="stack">
    <section class="panel hero">
      <h1>确定性身份局</h1>
      <p class="muted">
        默认原生引擎后端（engine-native-wasm 窄壳，Worker 内运行）。界面只消费观察者投影，不持有完整权威状态。可用
        <code>VITE_ENGINE_BACKEND=legacy</code> 回退旧线。
      </p>
      <p v-if="desktopPlatform" class="shell-badge">
        Tauri 2 · {{ desktopPlatform.os }} · {{ desktopPlatform.arch }}
      </p>
      <div class="row form">
        <label>
          人数
          <select v-model.number="players" aria-label="玩家人数">
            <option v-for="n in [2, 3, 4, 5, 6, 7, 8]" :key="n" :value="n">
              {{ n }}
            </option>
          </select>
        </label>
        <label>
          种子
          <input
            v-model.number="seed"
            type="number"
            min="0"
            aria-label="随机种子"
          />
        </label>
        <button class="primary" type="button" @click="goBattle">
          本地开局
        </button>
      </div>
    </section>

    <section class="panel">
      <h2>联机大厅</h2>
      <p class="muted">
        通过 game-server WebSocket 加入权威房间。浏览器仅提交命令并接收自己的观察者投影。
      </p>
      <div class="online-grid">
        <label class="wide">
          服务端
          <input v-model.trim="wsUrl" type="url" autocomplete="url" />
        </label>
        <label>
          昵称
          <input v-model.trim="displayName" maxlength="24" autocomplete="nickname" />
        </label>
        <label>
          房间密码
          <input v-model="password" type="password" autocomplete="current-password" />
        </label>
        <label>
          房间号 {{ roomLabel }}
          <input v-model.number="roomId" type="number" min="1" inputmode="numeric" />
        </label>
      </div>
      <div class="row online-actions">
        <button class="primary" type="button" @click="goOnline('create')">创建并入座</button>
        <button type="button" @click="goOnline('join')">加入房间</button>
        <button type="button" @click="goOnline('spectate')">旁观</button>
      </div>
      <p v-if="onlineError" class="error" role="alert">{{ onlineError }}</p>
    </section>

    <section class="panel">
      <h2>架构要点</h2>
      <ul class="muted list">
        <li>主线程：Vue Shell + DOM 战斗层 + 动画时间线</li>
        <li>本地：Worker + 原生窄壳（默认）/ 旧线 WASM；联机：WebSocket + 权威服务器</li>
        <li>扩展包管理：安装 / 启停 / 升级 / 卸载（导航「扩展」）</li>
        <li>支持自动步进、存读档、倍速、跳过动画、减少动态效果</li>
      </ul>
    </section>
  </div>
</template>

<style scoped>
.hero h1 {
  font-size: 1.6rem;
}
.shell-badge {
  display: inline-flex;
  margin: 0 0 0.8rem;
  padding: 0.25rem 0.55rem;
  border: 1px solid var(--accent);
  border-radius: 999px;
  color: var(--accent);
  font: 0.78rem var(--mono);
}
.form label {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  font-size: 0.85rem;
  color: var(--muted);
}
.form input,
.form select,
.online-grid input {
  background: var(--bg);
  border: 1px solid var(--border);
  color: var(--text);
  border-radius: 8px;
  padding: 0.4rem 0.55rem;
  min-width: 6rem;
}
.online-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 0.7rem;
  margin: 0.8rem 0;
}
.online-grid label {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  color: var(--muted);
  font-size: 0.85rem;
}
.online-grid .wide {
  grid-column: span 2;
}
.online-actions {
  flex-wrap: wrap;
}
.error {
  color: var(--danger);
  margin-bottom: 0;
}
@media (max-width: 640px) {
  .online-grid .wide { grid-column: span 1; }
}
.list {
  margin: 0;
  padding-left: 1.2rem;
  line-height: 1.7;
}
</style>
