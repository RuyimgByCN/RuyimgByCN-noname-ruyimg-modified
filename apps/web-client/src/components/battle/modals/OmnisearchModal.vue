<script setup lang="ts">
import { computed, ref } from "vue";
import {
  paginateOmnisearchResults,
  runOmnisearchQuery,
  type OmnisearchEntry,
  type OmnisearchSurfaceState,
} from "../../../engine/extensionSurfaceRuntime";
import type { ObserverSnapshot } from "../../../types/engine";

const props = defineProps<{
  state: OmnisearchSurfaceState;
  view: ObserverSnapshot | null;
}>();

const emit = defineEmits<{
  close: [];
  update: [next: OmnisearchSurfaceState];
}>();

const queryText = ref(props.state.lastError ? "caocao" : props.state.query || "caocao");
const kindFilter = ref<"all" | "general" | "skill" | "card">("all");
const page = ref(1);

const extraEntries: OmnisearchEntry[] = [
  { packageId: "shahai", kind: "card", id: "cangchizhibi", displayName: "苍螭之璧" },
  { packageId: "taixu", kind: "skill", id: "buff_txhj_aozhan", displayName: "太虚祝福·鏖战" },
  { packageId: "taixu", kind: "general", id: "txhj_zhaozhong", displayName: "太虚赵忠" },
  { packageId: "zuan", kind: "general", id: "ssr_luxun", displayName: "SSR陆逊" },
  { packageId: "d3", kind: "general", id: "ddd_caomao", displayName: "3D曹髦" },
];

const hiddenCardIds = computed(() =>
  (props.view?.known_cards ?? []).filter((c) => c.def == null).map((c) => c.id),
);

const filteredEntries = computed(() => {
  const list = props.state.results;
  if (kindFilter.value === "all") return list;
  return list.filter((e) => e.kind === kindFilter.value);
});

const paged = computed(() => paginateOmnisearchResults(filteredEntries.value, page.value, 8));

function executeSearch() {
  page.value = 1;
  const next = runOmnisearchQuery(props.state, queryText.value, hiddenCardIds.value, extraEntries);
  emit("update", next);
}
</script>

<template>
  <div class="modal-backdrop" role="dialog" aria-modal="true" aria-label="全能搜索牌局百科词典" data-testid="modal-omnisearch-dict">
    <div class="modal-card">
      <header class="modal-head">
        <div>
          <h3>全能搜索 · 牌局实时百科词典 (L07)</h3>
          <p class="muted tiny">上游基准：全能搜索/extension.js · 禁绝暗牌窥探与 eval 注入</p>
        </div>
        <button type="button" data-testid="btn-close-omnisearch-modal" @click="emit('close')">关闭 ✕</button>
      </header>

      <form class="search-bar" @submit.prevent="executeSearch">
        <input
          v-model="queryText"
          type="search"
          placeholder="输入武将 / 技能 / 卡牌 ID 或中文名…"
          data-testid="omnisearch-modal-input"
        />
        <select v-model="kindFilter" aria-label="分类筛选" data-testid="omnisearch-modal-kind">
          <option value="all">全部类别</option>
          <option value="general">武将</option>
          <option value="skill">技能</option>
          <option value="card">卡牌</option>
        </select>
        <button type="submit" class="primary" data-testid="btn-omnisearch-modal-run" @click.prevent="executeSearch">检索</button>
      </form>

      <p v-if="state.lastError" class="search-err" role="alert" data-testid="omnisearch-modal-error">
        安全拦截：{{ state.lastError }}
      </p>

      <div class="results-grid" data-testid="omnisearch-modal-results">
        <article v-for="item in paged.items" :key="`${item.packageId}:${item.id}`" class="result-card">
          <span class="kind-tag" :data-kind="item.kind">{{ item.kind }}</span>
          <strong>{{ item.displayName }}</strong>
          <code class="muted">{{ item.packageId }} · {{ item.id }}</code>
        </article>
        <p v-if="!paged.items.length && !state.lastError" class="muted">暂无匹配词条，点击「检索」查询公开图鉴。</p>
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
.search-bar {
  display: flex;
  gap: 0.45rem;
  margin-bottom: 0.75rem;
}
.search-bar input {
  flex: 1;
  padding: 0.35rem 0.6rem;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--bg);
  color: var(--text);
}
.search-bar select {
  padding: 0.35rem 0.5rem;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--bg);
  color: var(--text);
}
.search-err {
  color: var(--color-negative);
  font-family: var(--mono);
  font-size: 0.84rem;
}
.results-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(210px, 1fr));
  gap: 0.5rem;
}
.result-card {
  display: flex;
  flex-direction: column;
  gap: 0.2rem;
  padding: 0.5rem 0.65rem;
  border-radius: 8px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
}
.kind-tag {
  align-self: flex-start;
  font-size: 0.68rem;
  padding: 0.06rem 0.35rem;
  border-radius: 999px;
  background: rgba(232, 165, 75, 0.2);
  color: var(--accent);
}
.tiny { font-size: 0.75rem; margin: 0.15rem 0 0; }
</style>
