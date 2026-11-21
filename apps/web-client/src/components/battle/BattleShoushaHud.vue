<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from "vue";
import type { ObserverSnapshot } from "../../types/engine";
import type { ExtensionSurfacesSnapshot } from "../../engine/extensionSurfaceRuntime";
import { createDefaultShoushaConfig } from "../../engine/surfaces/shoushaConfigSchema";
import { resolveShoushaGtbbBanner } from "../../engine/surfaces/shoushaSurfacesRuntime";

export type BattleModalId =
  | "qhly"
  | "skinswitch"
  | "mvp"
  | "omnisearch"
  | "taixu"
  | "decade-menu"
  | "shousha-menu";

const props = defineProps<{
  view: ObserverSnapshot | null;
  surfaces?: ExtensionSurfacesSnapshot;
}>();

const emit = defineEmits<{
  openModal: [modal: BattleModalId];
}>();

const shoushaConfig = computed(
  () => props.surfaces?.uiSuites?.shoushaUi?.config ?? createDefaultShoushaConfig(),
);

const gtbbIndexOffset = ref(0);
const liveTickShift = ref(0);
let stTimer: number | null = null;
let gtbbTimer: number | null = null;

function stopTimers() {
  if (stTimer != null) {
    clearInterval(stTimer);
    stTimer = null;
  }
  if (gtbbTimer != null) {
    clearInterval(gtbbTimer);
    gtbbTimer = null;
  }
}

function syncTimers() {
  stopTimers();
  if (shoushaConfig.value.jindutiao && shoushaConfig.value.jindutiaotuoguan) {
    const rawMs = Number.parseInt(shoushaConfig.value.jindutiaoST || "200", 10) || 200;
    const ms = Math.max(80, Math.min(2000, rawMs));
    stTimer = window.setInterval(() => {
      if (liveTickShift.value < 100) liveTickShift.value += 2;
    }, ms);
  }
  if (shoushaConfig.value.GTBB) {
    const rawGtbb = Number.parseInt(shoushaConfig.value.GTBBTime || "60000", 10);
    const ms = Number.isFinite(rawGtbb) ? Math.max(5000, rawGtbb) : 45000;
    gtbbTimer = window.setInterval(() => {
      gtbbIndexOffset.value++;
    }, ms);
  }
}

watch(
  () => [props.view?.revision, props.view?.phase],
  () => {
    liveTickShift.value = 0;
  },
);
watch(
  () => [shoushaConfig.value.jindutiao, shoushaConfig.value.jindutiaotuoguan, shoushaConfig.value.jindutiaoST, shoushaConfig.value.GTBB, shoushaConfig.value.GTBBTime],
  () => syncTimers(),
  { immediate: true },
);
onUnmounted(() => stopTimers());

const gtbbBannerText = computed(() =>
  resolveShoushaGtbbBanner(shoushaConfig.value, (props.view?.round ?? 0) + gtbbIndexOffset.value),
);

const jindutiaoTrackHeightPx = computed(() => {
  const pct = Number.parseInt(shoushaConfig.value.jindutiaoSet || "28", 10) || 28;
  return `${Math.max(6, Math.min(18, Math.round(pct * 0.42)))}px`;
});

const phaseZh = computed(() => {
  const shoushaRibbon = props.surfaces?.uiSuites?.shoushaUi?.phaseRibbonLabel;
  if (shoushaRibbon && shoushaRibbon !== "等待阶段") return shoushaRibbon;
  switch (props.view?.phase) {
    case "Prepare":
      return "准备阶段";
    case "Judge":
      return "判定阶段";
    case "Draw":
      return "摸牌阶段";
    case "Play":
      return "出牌阶段";
    case "Discard":
      return "弃牌阶段";
    case "End":
      return "结束阶段";
    default:
      return "战局就绪";
  }
});

const countdownPct = computed(() => {
  const base = props.surfaces?.theme?.shoushaCountdownPct ?? 78;
  const raw = shoushaConfig.value.jindutiaotuoguan ? base - liveTickShift.value : base;
  return Math.max(0, Math.min(100, Math.round(raw)));
});

const tuoguanTriggered = computed(
  () => shoushaConfig.value.jindutiaotuoguan && countdownPct.value === 0,
);

const jiangdengTier = computed(() => {
  const alive = props.view?.seats.filter((s) => s.alive).length ?? 0;
  if (alive >= 6) return "将灯·统帅";
  if (alive >= 4) return "将灯·领军";
  return "将灯·枭雄";
});

const rzshCrest = computed(() => {
  const roman = props.surfaces?.theme?.rzshTierRoman ?? "IV";
  const mode = props.surfaces?.theme?.rzshTheme === "reality" ? "似幻" : "如真";
  return `${mode}·${roman}阶`;
});
</script>

<template>
  <section
    class="battle-shousha-hud"
    aria-label="手杀牌桌顶栏与扩展弹窗入口"
    data-testid="battle-shousha-table-hud"
  >
    <div class="hud-left">
      <span
        v-if="shoushaConfig.JDTS"
        class="phase-ribbon hud-phase-pill"
        data-testid="battle-hud-phase-ribbon"
      >
        ⚑ {{ phaseZh }}
      </span>
      <span v-if="view" class="round-pill" data-testid="battle-hud-round">
        第 {{ view.round }} 轮 · P{{ view.current_player }}
      </span>
      <span
        v-if="shoushaConfig.jiangdeng"
        class="lantern-badge"
        data-testid="battle-hud-jiangdeng"
      >
        <span class="candle-flame">🏮</span> {{ jiangdengTier }}
      </span>
      <span class="rzsh-crest" data-testid="battle-hud-rzsh-crest">
        ✦ {{ rzshCrest }}
      </span>
      <span
        v-if="gtbbBannerText"
        class="shousha-gtbb-banner"
        :data-gtbb-interval="shoushaConfig.GTBBTime"
        data-testid="shousha-gtbb-banner"
      >
        📢 {{ gtbbBannerText }}
      </span>
      <span
        v-if="tuoguanTriggered"
        class="shousha-tuoguan-indicator"
        data-testid="shousha-tuoguan-indicator"
      >
        ⏱ 读秒托管触发
      </span>
    </div>

    <div
      v-if="shoushaConfig.jindutiao"
      class="hud-timebar"
      role="progressbar"
      :aria-valuenow="countdownPct"
      aria-valuemin="0"
      aria-valuemax="100"
      data-testid="battle-hud-jindutiao"
    >
      <span class="timebar-label">读秒 {{ Math.ceil(countdownPct * 0.15) }}s</span>
      <div
        class="timebar-track"
        :class="{ 'is-urgent': countdownPct < 25 }"
        :style="{ height: jindutiaoTrackHeightPx }"
        data-testid="battle-hud-timebar-track"
      >
        <div
          class="timebar-fill shousha-hud-progress-fill"
          data-testid="battle-hud-countdown-fill"
          :class="{
            'fill-safe': countdownPct >= 50,
            'fill-warn': countdownPct < 50 && countdownPct >= 25,
            'fill-danger': countdownPct < 25,
          }"
          :style="{ width: `${countdownPct}%` }"
        >
          <span class="spark-cursor" aria-hidden="true">✦</span>
        </div>
      </div>
    </div>

    <nav class="hud-modal-actions" aria-label="扩展全屏界面入口">
      <button
        type="button"
        class="hud-btn"
        data-testid="btn-open-qhly-modal"
        @click="emit('openModal', 'qhly')"
      >
        千幻详情
      </button>
      <button
        type="button"
        class="hud-btn"
        data-testid="btn-open-skinswitch-modal"
        @click="emit('openModal', 'skinswitch')"
      >
        藏珍阁·动皮
      </button>
      <button
        type="button"
        class="hud-btn"
        data-testid="btn-open-mvp-modal"
        @click="emit('openModal', 'mvp')"
      >
        MVP结算
      </button>
      <button
        type="button"
        class="hud-btn"
        data-testid="btn-open-omnisearch-modal"
        @click="emit('openModal', 'omnisearch')"
      >
        全能搜索
      </button>
      <button
        type="button"
        class="hud-btn"
        data-testid="btn-open-taixu-modal"
        @click="emit('openModal', 'taixu')"
      >
        太虚战役图
      </button>
      <button
        type="button"
        class="hud-btn"
        data-testid="btn-open-decade-menu-modal"
        @click="emit('openModal', 'decade-menu')"
      >
        十周年配置
      </button>
      <button
        type="button"
        class="hud-btn"
        data-testid="btn-open-shousha-menu-modal"
        @click="emit('openModal', 'shousha-menu')"
      >
        手杀配置
      </button>
    </nav>
  </section>
</template>

<style scoped>
.battle-shousha-hud {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 0.6rem 0.9rem;
  padding: 0.5rem 0.8rem;
  border-radius: 10px;
  border: 1px solid rgba(232, 165, 75, 0.35);
  background: linear-gradient(90deg, rgba(28, 22, 16, 0.92), rgba(44, 31, 19, 0.88));
}
.hud-left {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.45rem;
}
/* Embroidered silk banner for phase ribbon */
.phase-ribbon {
  position: relative;
  padding: 0.22rem 0.95rem;
  border-radius: 4px;
  background: linear-gradient(135deg, #b83b3b 0%, #852222 50%, #5c1414 100%);
  color: #fff9e6;
  font-weight: 700;
  font-size: 0.84rem;
  letter-spacing: 0.05em;
  border: 1px solid #e8a54b;
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.5), inset 0 1px 0 rgba(255, 255, 255, 0.25);
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.8);
  animation: ribbon-slide-in 0.3s ease-out;
}
@keyframes ribbon-slide-in {
  0% { transform: translateY(-4px); opacity: 0.7; }
  100% { transform: translateY(0); opacity: 1; }
}
.round-pill {
  font-family: var(--mono);
  font-size: 0.78rem;
  color: var(--muted);
}
/* Lantern & Candle flicker styling */
.lantern-badge {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  padding: 0.14rem 0.55rem;
  border-radius: 6px;
  font-size: 0.76rem;
  font-weight: 600;
  border: 1px solid rgba(232, 197, 75, 0.4);
  background: rgba(232, 197, 75, 0.12);
  color: #f3d27a;
  animation: lantern-glow 2.2s infinite ease-in-out alternate;
}
@keyframes lantern-glow {
  0% { box-shadow: 0 0 4px rgba(232, 197, 75, 0.2); }
  100% { box-shadow: 0 0 10px rgba(232, 197, 75, 0.55); }
}
.candle-flame {
  display: inline-block;
  animation: flame-flicker 1.8s infinite ease-in-out alternate;
}
@keyframes flame-flicker {
  0% { transform: scale(0.95) rotate(-2deg); }
  100% { transform: scale(1.08) rotate(3deg); }
}
.rzsh-crest {
  padding: 0.14rem 0.5rem;
  border-radius: 6px;
  font-size: 0.76rem;
  font-weight: 600;
  border: 1px solid rgba(232, 197, 75, 0.35);
  background: rgba(232, 197, 75, 0.1);
  color: #f3d27a;
}
.hud-timebar {
  display: flex;
  align-items: center;
  gap: 0.45rem;
  min-width: 170px;
  flex: 1;
  max-width: 260px;
}
.timebar-label {
  font-family: var(--mono);
  font-size: 0.74rem;
  color: #f3d27a;
  white-space: nowrap;
}
.timebar-track {
  position: relative;
  flex: 1;
  height: 9px;
  border-radius: 999px;
  background: rgba(0, 0, 0, 0.4);
  border: 1px solid rgba(255, 255, 255, 0.12);
  overflow: visible;
}
.timebar-track.is-urgent {
  animation: bar-urgent-pulse 0.5s ease-in-out infinite alternate;
}
@keyframes bar-urgent-pulse {
  0% { box-shadow: 0 0 4px rgba(224, 90, 90, 0.4); }
  100% { box-shadow: 0 0 12px rgba(224, 90, 90, 0.9); }
}
.timebar-fill {
  position: relative;
  height: 100%;
  border-radius: 999px;
  transition: width 0.2s ease, background 0.3s ease;
}
.timebar-fill.fill-safe {
  background: linear-gradient(90deg, #3db868, #5fbf7a);
  box-shadow: 0 0 8px rgba(95, 191, 122, 0.5);
}
.timebar-fill.fill-warn {
  background: linear-gradient(90deg, #d89638, #f59e0b);
  box-shadow: 0 0 8px rgba(245, 158, 11, 0.5);
}
.timebar-fill.fill-danger {
  background: linear-gradient(90deg, #b83b3b, #e05a5a);
  box-shadow: 0 0 12px rgba(224, 90, 90, 0.8);
  animation: danger-flash 0.4s ease-in-out infinite alternate;
}
@keyframes danger-flash {
  0% { filter: brightness(1); }
  100% { filter: brightness(1.4); }
}
.spark-cursor {
  position: absolute;
  right: -5px;
  top: -6px;
  font-size: 0.8rem;
  color: #fff4bd;
  text-shadow: 0 0 8px #ffd700, 0 0 12px #ff8c00;
  animation: spark-twinkle 0.3s infinite alternate;
  pointer-events: none;
}
@keyframes spark-twinkle {
  0% { transform: scale(0.9) rotate(0deg); opacity: 0.8; }
  100% { transform: scale(1.2) rotate(15deg); opacity: 1; }
}
.hud-modal-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 0.35rem;
}
.hud-btn {
  padding: 0.22rem 0.55rem;
  font-size: 0.76rem;
  border-radius: 6px;
  border: 1px solid rgba(232, 165, 75, 0.4);
  background: rgba(255, 255, 255, 0.06);
  color: var(--text);
  cursor: pointer;
}
.hud-btn:hover {
  border-color: var(--accent);
  background: rgba(232, 165, 75, 0.18);
}
@media (max-width: 767px) {
  .battle-shousha-hud {
    padding: 0.34rem 0.48rem;
    gap: 0.28rem 0.45rem;
  }
  .phase-ribbon {
    padding: 0.14rem 0.55rem;
    font-size: 0.74rem;
  }
  .round-pill,
  .lantern-badge,
  .rzsh-crest {
    font-size: 0.68rem;
    padding: 0.08rem 0.36rem;
  }
  .hud-timebar {
    display: none;
  }
  .shousha-gtbb-banner {
    display: none;
  }
  .hud-modal-actions {
    flex-wrap: wrap;
    max-width: 100%;
    gap: 0.22rem;
    overflow: visible;
  }
  .hud-btn {
    flex-shrink: 0;
    padding: 0.1rem 0.34rem;
    font-size: 0.64rem;
    line-height: 1.2;
  }
}
</style>
