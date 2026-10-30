<script setup lang="ts">
import { computed } from "vue";
import type { GeneralSelectState } from "../../engine/generalSelectModel";

const props = defineProps<{
  state: GeneralSelectState;
  busy: boolean;
}>();

const emit = defineEmits<{
  choose: [name: string];
  reload: [];
}>();

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
  <section class="panel general-select" aria-label="选将面板">
    <div class="gs-head">
      <div>
        <h2>选将</h2>
        <p class="muted tiny">
          NC05 · 目标座位 P{{ state.seat }} · {{ loadLabel }} · 点击候选即按名装配
          （原生正式入口，非 debug）
        </p>
      </div>
      <button
        type="button"
        :disabled="locked || state.loadStatus === 'loading'"
        aria-label="刷新选将候选"
        @click="emit('reload')"
      >
        刷新候选
      </button>
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
