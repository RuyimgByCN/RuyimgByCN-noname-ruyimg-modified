<script setup lang="ts">
import { computed } from "vue";
import type { ObserverSnapshot } from "../../types/engine";
import type { ExtensionSurfacesSnapshot } from "../../engine/extensionSurfaceRuntime";

export type BattleModalId = "qhly" | "skinswitch" | "mvp" | "omnisearch" | "taixu";

const props = defineProps<{
  view: ObserverSnapshot | null;
  surfaces?: ExtensionSurfacesSnapshot;
}>();

const emit = defineEmits<{
  openModal: [modal: BattleModalId];
}>();

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
  const raw = props.surfaces?.theme?.shoushaCountdownPct ?? 78;
  return Math.max(0, Math.min(100, Math.round(raw)));
});

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
      <span class="phase-ribbon" data-testid="battle-hud-phase-ribbon">
        ⚑ {{ phaseZh }}
      </span>
      <span v-if="view" class="round-pill" data-testid="battle-hud-round">
        第 {{ view.round }} 轮 · P{{ view.current_player }}
      </span>
      <span class="lantern-badge" data-testid="battle-hud-jiangdeng">
        🏮 {{ jiangdengTier }}
      </span>
      <span class="rzsh-crest" data-testid="battle-hud-rzsh-crest">
        ✦ {{ rzshCrest }}
      </span>
    </div>

    <div class="hud-timebar" role="progressbar" :aria-valuenow="countdownPct" aria-valuemin="0" aria-valuemax="100">
      <span class="timebar-label">读秒 {{ Math.ceil(countdownPct * 0.15) }}s</span>
      <div class="timebar-track">
        <div
          class="timebar-fill"
          data-testid="battle-hud-countdown-fill"
          :style="{ width: `${countdownPct}%` }"
        ></div>
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
.phase-ribbon {
  padding: 0.18rem 0.65rem;
  border-radius: 999px;
  background: linear-gradient(135deg, #d89638, #9c5b16);
  color: #fff8eb;
  font-weight: 700;
  font-size: 0.82rem;
}
.round-pill {
  font-family: var(--mono);
  font-size: 0.78rem;
  color: var(--muted);
}
.lantern-badge,
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
  flex: 1;
  height: 7px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.12);
  overflow: hidden;
}
.timebar-fill {
  height: 100%;
  border-radius: 999px;
  background: linear-gradient(90deg, #5fbf7a, #e8a54b, #e05a5a);
  transition: width 0.2s ease;
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
</style>
