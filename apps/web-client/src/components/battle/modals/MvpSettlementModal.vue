<script setup lang="ts">
import { computed, ref } from "vue";
import type { ObserverSnapshot } from "../../../types/engine";
import {
  L13_CONFIG_SPECS,
  L13_SURFACE_IDS,
  createInitialL13UiState,
  type L13MvpConfig,
  type L13MvpUiState,
  type MvpScoreRow,
  type MvpSurfaceState,
} from "../../../engine/extensionSurfaceRuntime";

const props = defineProps<{
  mvp: MvpSurfaceState;
  l13?: L13MvpUiState;
  view: ObserverSnapshot | null;
}>();

const emit = defineEmits<{
  close: [];
  "update:l13": [next: L13MvpUiState];
  recastCue: [seat: number];
  oneMore: [];
}>();

const praiseCountBySeat = ref<Record<number, number>>({});
const l13State = computed(() => props.l13 ?? createInitialL13UiState());
const activeSurfaceCss = computed(() =>
  l13State.value.config.yangshi === "on" ? "extension_new.css" : "extension.css",
);

const rankedRows = computed<MvpScoreRow[]>(() => {
  const existing = Object.values(props.mvp.scores);
  if (existing.length > 0) {
    return [...existing].sort((a, b) => b.total - a.total || a.seat - b.seat);
  }
  const seats = props.view?.seats ?? [];
  return seats
    .map((s) => {
      const attack = Math.max(1, (s.max_hp - Math.max(0, s.hp)) * 12 + s.skills.length * 10);
      const heal = s.alive ? 10 : 0;
      const assist = s.hand_count * 4;
      const base = 20;
      return {
        seat: s.player,
        attack,
        heal,
        assist,
        base,
        penalty: s.alive ? 0 : 5,
        total: base + attack + heal + assist - (s.alive ? 0 : 5),
        kills: 0,
        damageDealt: Math.max(0, s.max_hp - s.hp),
      };
    })
    .sort((a, b) => b.total - a.total || a.seat - b.seat);
});

const topSeat = computed(() => props.mvp.mvpSeat ?? rankedRows.value[0]?.seat ?? 0);

function praiseSeat(seat: number) {
  praiseCountBySeat.value = {
    ...praiseCountBySeat.value,
    [seat]: (praiseCountBySeat.value[seat] ?? 0) + 1,
  };
}

function patchL13Config<K extends keyof L13MvpConfig>(key: K, value: L13MvpConfig[K]) {
  emit("update:l13", {
    ...l13State.value,
    config: { ...l13State.value.config, [key]: value },
  });
}

function onTriggerRecastCue() {
  emit("recastCue", topSeat.value);
}

function onClickOneMore() {
  emit("update:l13", {
    ...l13State.value,
    oneMoreClicks: l13State.value.oneMoreClicks + 1,
  });
  emit("oneMore");
}
</script>

<template>
  <div class="modal-backdrop" role="dialog" aria-modal="true" aria-label="手杀MVP战功结算榜" data-testid="modal-mvp-settlement">
    <div
      class="modal-card mvp-modal-card"
      :data-l13-surface="activeSurfaceCss"
      :data-l13-lutou="l13State.config.mvplutou"
    >
      <header class="modal-head">
        <div>
          <h3>手杀 MVP · 全场战功结算 (L13 · {{ L13_CONFIG_SPECS.length }}配置/{{ L13_SURFACE_IDS.length }}样式)</h3>
          <p class="muted tiny">上游基准：手杀MVP/extension.js · 样式({{ activeSurfaceCss }}) / 露头({{ l13State.config.mvplutou }}) / _qy_chongzhu</p>
        </div>
        <button type="button" data-testid="btn-close-mvp-modal" @click="emit('close')">关闭 ✕</button>
      </header>

      <div class="l13-config-strip" data-testid="l13-config-strip">
        <label class="tiny">
          样式:
          <select
            :value="l13State.config.yangshi"
            data-testid="l13-select-yangshi"
            @change="patchL13Config('yangshi', ($event.target as HTMLSelectElement).value as 'on' | 'off')"
          >
            <option value="on">样式1 (extension_new.css)</option>
            <option value="off">样式2 (extension.css)</option>
          </select>
        </label>
        <label class="tiny">
          露头:
          <select
            :value="l13State.config.mvplutou"
            data-testid="l13-select-mvplutou"
            @change="patchL13Config('mvplutou', ($event.target as HTMLSelectElement).value as 'shousha' | 'shizhounian' | 'off')"
          >
            <option value="shousha">手杀</option>
            <option value="shizhounian">十周年</option>
            <option value="off">无</option>
          </select>
        </label>
        <label class="tiny"><input type="checkbox" :checked="l13State.config.qingyao_shoushaMVP" @change="patchL13Config('qingyao_shoushaMVP', ($event.target as HTMLInputElement).checked)" /> MVP动画</label>
        <label class="tiny"><input type="checkbox" :checked="l13State.config.qingyao_shoushapeiyin" @change="patchL13Config('qingyao_shoushapeiyin', ($event.target as HTMLInputElement).checked)" /> 手杀配音</label>
        <label class="tiny"><input type="checkbox" :checked="l13State.config.shoushajiesuan" @change="patchL13Config('shoushajiesuan', ($event.target as HTMLInputElement).checked)" /> 结算面板</label>
        <label class="tiny"><input type="checkbox" :checked="l13State.config.jiesuanfix" @change="patchL13Config('jiesuanfix', ($event.target as HTMLInputElement).checked)" /> 罕见胜负优化</label>
        <label class="tiny"><input type="checkbox" :checked="l13State.config.oneMore" @change="patchL13Config('oneMore', ($event.target as HTMLInputElement).checked)" /> 再来一局</label>
        <button type="button" data-testid="btn-mvp-qy-chongzhu" @click="onTriggerRecastCue">
          触发 _qy_chongzhu
        </button>
      </div>

      <div v-if="l13State.config.yangshi === 'on'" class="qy-mvp-container" data-testid="l13-qy-mvp-container">
        <div class="qy-mvp-avatarbox">
          <strong>P{{ topSeat }} · {{ l13State.config.mvplutou }}露头</strong>
        </div>
        <div>
          <div class="qy-mvp-xing">
            <span class="item on">★</span><span class="item on">★</span><span class="item on">★</span><span class="item on">★</span><span class="item off">★</span>
          </div>
          <p class="tiny muted">{{ l13State.lastRecastCue ? `重铸音效: ${l13State.lastRecastCue}` : "新版倾斜战功框 (extension_new.css)" }}</p>
        </div>
      </div>

      <div class="mvp-crown-banner" data-testid="mvp-modal-crown-seat">
        👑 本局全场最佳：<strong>P{{ topSeat }}</strong>
      </div>

      <table v-if="l13State.config.shoushajiesuan" class="mvp-table">
        <thead>
          <tr>
            <th>排名</th>
            <th>座位</th>
            <th>攻击分</th>
            <th>治疗分</th>
            <th>辅助分</th>
            <th>总战功</th>
            <th>互动</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(row, idx) in rankedRows" :key="row.seat" :class="{ top: row.seat === topSeat }">
            <td>#{{ idx + 1 }}</td>
            <td>P{{ row.seat }}</td>
            <td>{{ row.attack }}</td>
            <td>{{ row.heal }}</td>
            <td>{{ row.assist }}</td>
            <td class="mono">{{ row.total }}</td>
            <td>
              <button
                type="button"
                class="praise-btn"
                :data-testid="`btn-mvp-praise-${row.seat}`"
                @click="praiseSeat(row.seat)"
              >
                👍 献花 ({{ praiseCountBySeat[row.seat] ?? 0 }})
              </button>
            </td>
          </tr>
        </tbody>
      </table>

      <div class="mvp-footer-actions">
        <span class="tiny muted" data-testid="l13-recast-status">
          {{ l13State.lastRecastCue ?? "重铸待机" }} · 再来一局点击 {{ l13State.oneMoreClicks }} 次
        </span>
        <button
          v-if="l13State.config.oneMore"
          type="button"
          class="primary"
          data-testid="btn-mvp-one-more"
          @click="onClickOneMore"
        >
          再来一局
        </button>
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
  width: min(760px, 96vw);
  background: var(--bg-panel, #1b1814);
  border: 1px solid var(--accent, #e8a54b);
  border-radius: 12px;
  padding: 1rem 1.2rem;
}
.modal-head {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 0.75rem;
}
.modal-head h3 { margin: 0; }
.mvp-crown-banner {
  padding: 0.55rem 0.85rem;
  border-radius: 8px;
  background: linear-gradient(90deg, rgba(232, 165, 75, 0.25), rgba(232, 197, 75, 0.08));
  border: 1px solid rgba(232, 165, 75, 0.5);
  margin-bottom: 0.75rem;
  font-size: 0.95rem;
}
.mvp-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.85rem;
}
.mvp-table th,
.mvp-table td {
  padding: 0.45rem 0.5rem;
  border-bottom: 1px solid var(--border);
  text-align: left;
}
.mvp-table tr.top {
  background: rgba(232, 165, 75, 0.12);
  font-weight: 600;
}
.praise-btn {
  font-size: 0.76rem;
  padding: 0.16rem 0.45rem;
}
.mono { font-family: var(--mono); }
.tiny { font-size: 0.75rem; margin: 0.15rem 0 0; }
</style>
