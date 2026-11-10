<script setup lang="ts">
import { computed, ref } from "vue";
import {
  TAIXU_DLC_SEASONS,
  TAIXU_SERVANT_SPECS,
  createInitialTaixuUiRuntimeState,
  purchaseTaixuMerchantItem,
  selectTaixuServantSurface,
  switchTaixuViewport,
  type TaixuUiRuntimeState,
} from "../../../engine/surfaces/taixuSurface";
import type { TaixuSurfaceState } from "../../../engine/extensionSurfaceRuntime";

const props = defineProps<{
  taixu: TaixuSurfaceState;
  taixuUi?: TaixuUiRuntimeState;
}>();

const emit = defineEmits<{
  close: [];
  update: [payload: { taixu: TaixuSurfaceState; taixuUi: TaixuUiRuntimeState }];
}>();

const activeSeason = ref(TAIXU_DLC_SEASONS[0]?.id ?? "GFHuangJinZhiLuan");
const uiState = computed(() => props.taixuUi ?? createInitialTaixuUiRuntimeState());

const UPSTREAM_BLESSINGS = [
  { id: "buff_txhj_aozhan", label: "鏖战", cost: 80 },
  { id: "buff_txhj_yanhuo", label: "焱火", cost: 60 },
  { id: "buff_txhj_xiongziyingfa", label: "雄姿英发", cost: 90 },
  { id: "buff_txhj_juguzhidao", label: "贾估之道", cost: 70 },
  { id: "buff_txhj_tianxiawushuang", label: "天下无双", cost: 100 },
] as const;

function advanceCampaignStep() {
  const nextIdx = Math.min(props.taixu.nodes.length - 1, props.taixu.currentNodeIndex + 1);
  const nextNodes = props.taixu.nodes.map((n, idx) =>
    idx <= nextIdx ? { ...n, cleared: true } : n,
  );
  emit("update", {
    taixu: {
      ...props.taixu,
      seasonId: activeSeason.value,
      currentNodeIndex: nextIdx,
      gold: props.taixu.gold + 120,
      nodes: nextNodes,
    },
    taixuUi: uiState.value,
  });
}

function buyUpstreamBlessing(buffId: string, baseCost: number) {
  if (props.taixu.blessings.includes(buffId)) return;
  const hasJugu = props.taixu.blessings.includes("buff_txhj_juguzhidao");
  const finalCost = hasJugu ? Math.floor(baseCost / 2) : baseCost;
  if (props.taixu.gold < finalCost) return;
  emit("update", {
    taixu: {
      ...props.taixu,
      gold: props.taixu.gold - finalCost,
      blessings: [...props.taixu.blessings, buffId],
    },
    taixuUi: purchaseTaixuMerchantItem({ ...uiState.value, hasJuguzhidaoBuff: hasJugu }, baseCost),
  });
}

function bindServant(servantId: string) {
  const spec = TAIXU_SERVANT_SPECS.find((s) => s.id === servantId);
  emit("update", {
    taixu: {
      ...props.taixu,
      boundPet: spec?.label ?? servantId,
    },
    taixuUi: selectTaixuServantSurface(uiState.value, servantId),
  });
}

function toggleViewport() {
  const nextW = uiState.value.viewportMode === "pc" ? 720 : 1280;
  emit("update", {
    taixu: props.taixu,
    taixuUi: switchTaixuViewport(uiState.value, nextW),
  });
}
</script>

<template>
  <div class="modal-backdrop" role="dialog" aria-modal="true" aria-label="太虚幻境赛季节点图与侍灵商铺" data-testid="modal-taixu-campaign">
    <div class="modal-card">
      <header class="modal-head">
        <div>
          <h3>太虚幻境 · 赛季战役路线图与侍灵商铺 (L11)</h3>
          <p class="muted tiny">
            上游基准：dlc/*/extension_season.js · extension_buff.js · extension_servant.js
          </p>
        </div>
        <button type="button" data-testid="btn-close-taixu-modal" @click="emit('close')">关闭 ✕</button>
      </header>

      <div class="taixu-top-bar">
        <label>
          赛季选择
          <select v-model="activeSeason" data-testid="taixu-modal-season-select">
            <option v-for="s in TAIXU_DLC_SEASONS" :key="s.id" :value="s.id">
              {{ s.title }} ({{ s.id }})
            </option>
          </select>
        </label>
        <span class="gold-pill" data-testid="taixu-modal-gold">🪙 {{ taixu.gold }} 钱币</span>
        <span class="pet-pill" data-testid="taixu-modal-pet">侍灵：{{ taixu.boundPet ?? "未同行" }}</span>
        <button type="button" data-testid="btn-taixu-modal-viewport" @click="toggleViewport">
          样式：{{ uiState.responsive.activeStylesheet }}
        </button>
      </div>

      <div class="node-graph" data-testid="taixu-modal-node-graph">
        <div
          v-for="(node, idx) in taixu.nodes"
          :key="node.id"
          class="node-step"
          :class="{ cleared: node.cleared, active: idx === taixu.currentNodeIndex }"
        >
          <span class="node-idx">#{{ idx + 1 }}</span>
          <strong>{{ node.title }}</strong>
          <span class="tiny muted">{{ node.kind }}</span>
        </div>
        <button type="button" class="primary" data-testid="btn-taixu-modal-advance" @click="advanceCampaignStep">
          推进下一关卡节点 ▸
        </button>
      </div>

      <div class="taixu-columns">
        <section class="shop-box">
          <h4>奇遇商铺 (真实上游 buff_txhj_* 祝福)</h4>
          <div class="blessing-list">
            <button
              v-for="b in UPSTREAM_BLESSINGS"
              :key="b.id"
              type="button"
              class="blessing-btn"
              :disabled="taixu.blessings.includes(b.id)"
              :data-testid="`btn-taixu-buy-${b.id}`"
              @click="buyUpstreamBlessing(b.id, b.cost)"
            >
              <span>{{ b.label }} (<code>{{ b.id }}</code>)</span>
              <span>{{ taixu.blessings.includes(b.id) ? "已习得" : `${b.cost}币` }}</span>
            </button>
          </div>
          <p class="tiny muted" data-testid="taixu-modal-owned-blessings">
            已持有祝福：{{ taixu.blessings.join(", ") || "无" }}
          </p>
        </section>

        <section class="servant-box">
          <h4>同行侍灵殿 ({{ TAIXU_SERVANT_SPECS.length }} 尊)</h4>
          <div class="servant-grid">
            <button
              v-for="sv in TAIXU_SERVANT_SPECS.slice(0, 8)"
              :key="sv.id"
              type="button"
              class="servant-btn"
              :data-testid="`btn-taixu-servant-${sv.id}`"
              @click="bindServant(sv.id)"
            >
              {{ sv.label }} ({{ sv.id }})
            </button>
          </div>
        </section>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-backdrop {
  position: fixed;
  inset: 0;
  z-index: 60;
  background: rgba(8, 10, 14, 0.78);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 1rem;
}
.modal-card {
  width: min(880px, 96vw);
  max-height: 90vh;
  overflow-y: auto;
  background: var(--bg-panel, #1b1814);
  border: 1px solid var(--accent, #e8a54b);
  border-radius: 12px;
  padding: 1rem 1.2rem;
}
.modal-head {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 0.7rem;
}
.modal-head h3 { margin: 0; }
.taixu-top-bar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.65rem;
  margin-bottom: 0.8rem;
}
.gold-pill,
.pet-pill {
  padding: 0.2rem 0.55rem;
  border-radius: 999px;
  background: rgba(232, 165, 75, 0.16);
  border: 1px solid rgba(232, 165, 75, 0.4);
  font-size: 0.82rem;
}
.node-graph {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.5rem;
  padding: 0.65rem;
  border-radius: 10px;
  background: var(--bg-elevated);
  margin-bottom: 0.85rem;
}
.node-step {
  display: flex;
  flex-direction: column;
  padding: 0.4rem 0.6rem;
  border-radius: 8px;
  border: 1px solid var(--border);
  font-size: 0.82rem;
}
.node-step.cleared { border-color: var(--color-positive); }
.node-step.active {
  border-color: var(--accent);
  box-shadow: 0 0 0 1px var(--accent);
}
.taixu-columns {
  display: grid;
  grid-template-columns: 1.2fr 1fr;
  gap: 0.85rem;
}
.shop-box,
.servant-box {
  padding: 0.65rem;
  border-radius: 10px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
}
.shop-box h4,
.servant-box h4 { margin: 0 0 0.5rem; font-size: 0.9rem; }
.blessing-list {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
}
.blessing-btn {
  display: flex;
  justify-content: space-between;
  font-size: 0.8rem;
}
.servant-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.35rem;
}
.servant-btn { font-size: 0.8rem; }
.tiny { font-size: 0.74rem; margin: 0.25rem 0 0; }
</style>
