import { createApp } from "vue";
import App from "./App.vue";
import { router } from "./router";
import "./styles/global.css";

// NC09 重连：会话内保存过联机意图（房间/服务端/昵称）时，启动后**自动回到该房间**——
// 令牌已在 BattleView 落 localStorage（`reconnectKey(roomId)`），故重连会命中原座位。
// 未保存意图（或解析失败）时保持默认大厅。
function resumeOnlineSession(): void {
  const saved = sessionStorage.getItem("noname-next.online-intent");
  if (!saved) return;
  try {
    const intent = JSON.parse(saved) as {
      action?: string;
      wsUrl?: string;
      displayName?: string;
      password?: string;
      roomId?: number;
      players?: number;
      seed?: number;
    };
    if (intent.action === "spectate") return; // 旁观不自动恢复（避免占用观察位）
    void router.push({
      path: "/battle",
      query: {
        mode: "online",
        action: intent.action === "create" ? "join" : (intent.action ?? "join"),
        room: String(intent.roomId ?? 1),
        players: String(intent.players ?? 4),
        seed: String(intent.seed ?? 42),
      },
    });
  } catch {
    sessionStorage.removeItem("noname-next.online-intent");
  }
}

const app = createApp(App).use(router);
app.mount("#app");
resumeOnlineSession();
