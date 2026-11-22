<script setup lang="ts">
import { computed, ref } from "vue";
import type { GeneralSelectState } from "../../engine/generalSelectModel";

const props = defineProps<{
  state: GeneralSelectState;
  busy: boolean;
  changeWujiangTx?: "off" | "blur" | "shine";
  shoushaChoose?: "shizhounian" | "shousha" | "off";
  shoushaChooseType?: "old" | "new";
  shoushaSingleRow?: boolean;
  shoushaDrawLuck?: boolean;
  shoushaInfinityDraw?: boolean;
  eCssEnabled?: boolean;
  qhlyChooseOrigin?: boolean;
  qhlyChooseOriginGuozhan?: boolean;
  banQh?: boolean;
}>();

const emit = defineEmits<{
  choose: [name: string];
  reload: [];
}>();

const luckUsesCount = ref(0);
const candidateLuckScore = computed(() => {
  if (!props.state.candidates.length) return 0;
  const seed = props.state.candidates.reduce((acc, c, i) => acc + c.def * (i + 3), 0);
  return 58 + (seed % 41);
});

function onReloadClick() {
  if (!props.shoushaInfinityDraw && props.shoushaDrawLuck && luckUsesCount.value >= 7) return;
  luckUsesCount.value++;
  emit("reload");
}

const locked = computed(
  () => props.busy || props.state.selectionStatus === "pending",
);
const selectedName = computed(() => props.state.assignment?.name ?? null);
const viewSkillsLabel = computed(() =>
  props.state.projection.skillNames.length > 0
    ? props.state.projection.skillNames.join(" · ")
    : "视图未暴露技能名",
);
const loadLabel = computed(() => {
  switch (props.state.loadStatus) {
    case "loading":
      return "读取候选…";
    case "error":
      return "候选不可用";
    case "empty":
      return "候选为空";
    default:
      return `${props.state.candidates.length} 项候选`;
  }
});
</script>

<template>
  <section
    class="panel general-select"
    :class="{ 'l27-ecss-styled': eCssEnabled !== false }"
    :data-change-wujiang-tx="changeWujiangTx ?? 'shine'"
    :data-shousha-choose="shoushaChoose ?? 'shousha'"
    :data-shousha-choose-type="shoushaChooseType ?? 'new'"
    :data-shousha-single-row="shoushaSingleRow ? 'on' : 'off'"
    :data-qhly-choose-origin="qhlyChooseOrigin ? 'on' : 'off'"
    :data-qhly-choose-origin-gz="qhlyChooseOriginGuozhan ? 'on' : 'off'"
    :data-ban-qh="banQh ? 'on' : 'off'"
    aria-label="选将面板"
    data-testid="general-select-panel"
  >
    <div class="gs-head">
      <div>
        <h2>
          选将
          <span v-if="eCssEnabled !== false" class="l27-biaotixx" data-testid="l27-ecss-header">· 选将美化</span>
          <span v-if="eCssEnabled !== false" class="l27-self-ribbon">主视点</span>
        </h2>
        <p class="muted tiny">
          NC05 · 目标座位 P{{ state.seat }} · {{ loadLabel }} · 点击候选即按名装配
          （原生正式入口，非 debug）
        </p>
        <div v-if="eCssEnabled !== false" class="l27-jindutiao-track" aria-hidden="true">
          <div class="l27-jindutiao-fill"></div>
        </div>
      </div>
      <button
        type="button"
        :disabled="Boolean(banQh) || locked || state.loadStatus === 'loading' || (!shoushaInfinityDraw && shoushaDrawLuck && luckUsesCount >= 7)"
        aria-label="刷新选将候选"
        data-testid="btn-general-reload"
        @click="onReloadClick"
      >
        {{ banQh ? '已屏蔽换将 (banQh)' : shoushaDrawLuck ? (shoushaInfinityDraw ? '无限换将(∞)' : `手气卡刷新(${Math.max(0, 7 - luckUsesCount)}/7)`) : '刷新候选' }}
      </button>
    </div>

    <div v-if="shoushaDrawLuck && state.candidates.length > 0" class="shousha-luck-bar" data-testid="shousha-choose-luck-bar">
      <span>手气评价：<strong>{{ candidateLuckScore }} 分</strong></span>
      <span>样式：{{ shoushaChoose ?? 'shousha' }} / {{ shoushaChooseType ?? 'new' }}</span>
      <span>排列：{{ shoushaSingleRow ? '单行卷轴' : '多行网格' }}</span>
    </div>

    <p
      v-if="state.loadStatus === 'loading' && state.candidates.length === 0"
      class="muted"
      role="status"
    >
      正在读取原生目录候选…
    </p>
    <p
      v-else-if="state.loadStatus === 'empty'"
      class="muted empty"
      role="status"
    >
      候选为空：内核目录没有返回任何武将（这是合法的空目录，不是加载失败）。
    </p>

    <p v-if="state.error" class="error" role="alert">{{ state.error }}</p>

    <p
      v-if="state.selectionStatus === 'accepted' && state.assignment"
      class="success"
      role="status"
    >
      ✅ 已为 P{{ state.assignment.seat }} 装配
      <strong>{{ state.assignment.name }}</strong>
      <span class="muted">
        · state_hash {{ state.assignment.state_hash.slice(0, 16) }}…
      </span>
      <br />
      <span class="muted">刷新后视图技能：{{ viewSkillsLabel }}</span>
    </p>

    <ul v-if="state.candidates.length > 0" class="gs-grid">
      <li v-for="candidate in state.candidates" :key="candidate.def">
        <button
          type="button"
          class="candidate"
          :class="{ selected: selectedName === candidate.name }"
          :disabled="locked"
          :aria-pressed="selectedName === candidate.name"
          :aria-label="`选择 ${candidate.name}（${candidate.skillLabel}）`"
          @click="emit('choose', candidate.name)"
        >
          <span class="candidate-name">{{ candidate.name }}</span>
          <span class="candidate-skills">{{ candidate.skillLabel }}</span>
          <span class="candidate-def muted">#{{ candidate.def }}</span>
        </button>
      </li>
    </ul>
  </section>
</template>

<style scoped>
.general-select {
  scroll-margin-top: 72px;
}
.general-select h2 {
  font-size: 1.05rem;
  margin: 0;
}
.gs-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 0.75rem;
}
.tiny {
  font-size: 0.8rem;
  margin: 0.2rem 0 0;
}
.gs-grid {
  list-style: none;
  margin: 0.65rem 0 0;
  padding: 0;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
  gap: 0.4rem;
  max-height: 18rem;
  overflow-y: auto;
}
.candidate {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 0.1rem;
  width: 100%;
  padding: 0.4rem 0.55rem;
  text-align: left;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 6px;
  transition: transform 0.22s ease, filter 0.22s ease, box-shadow 0.22s ease;
}
.general-select[data-change-wujiang-tx="shine"] .candidate:hover:not(:disabled),
.general-select[data-change-wujiang-tx="shine"] .candidate.selected {
  box-shadow: 0 0 12px rgba(250, 204, 21, 0.45);
}
.general-select[data-change-wujiang-tx="blur"] .candidate:not(.selected):hover:not(:disabled) {
  filter: blur(0.4px) brightness(1.08);
}
.general-select[data-change-wujiang-tx="off"] .candidate {
  transition: none;
}
.candidate:hover:not(:disabled) {
  border-color: var(--accent);
}
.candidate.selected {
  border-color: var(--accent);
  box-shadow: inset 0 0 0 1px var(--accent);
}
.candidate-name {
  font-weight: 600;
}
.candidate-skills {
  font-size: 0.75rem;
  color: var(--muted);
}
.candidate-def {
  font-family: var(--mono);
  font-size: 0.65rem;
}
.error {
  color: var(--danger);
  margin: 0.35rem 0 0;
}
.success {
  color: var(--color-positive);
  border-left: 3px solid var(--color-positive);
  padding-left: 0.55rem;
  margin: 0.45rem 0 0;
  font-size: 0.88rem;
}
.empty {
  font-style: italic;
}
</style>
