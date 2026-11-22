<script setup lang="ts">
import { computed, ref } from "vue";
import {
  L07_CONFIG_SPECS,
  L07_HIGHLIGHT_CSS_IDS,
  buildSimplePaginationView,
  createInitialL07UiState,
  paginateOmnisearchResults,
  runOmnisearchQuery,
  type L07OmnisearchConfig,
  type L07OmnisearchUiState,
  type OmnisearchEntry,
  type OmnisearchSurfaceState,
} from "../../../engine/extensionSurfaceRuntime";
import type { ObserverSnapshot } from "../../../types/engine";

const props = defineProps<{
  state: OmnisearchSurfaceState;
  l07?: L07OmnisearchUiState;
  view: ObserverSnapshot | null;
}>();

const emit = defineEmits<{
  close: [];
  update: [next: OmnisearchSurfaceState];
  "update:l07": [next: L07OmnisearchUiState];
}>();

const l07State = computed(() => props.l07 ?? createInitialL07UiState());
const queryText = ref(props.state.lastError ? "caocao" : props.state.query || "caocao");
const kindFilter = ref<"all" | "general" | "skill" | "card">("all");
const page = ref(1);

function patchL07Config<K extends keyof L07OmnisearchConfig>(key: K, value: L07OmnisearchConfig[K]) {
  emit("update:l07", {
    ...l07State.value,
    config: { ...l07State.value.config, [key]: value },
  });
}

function toggleSearcherStyle(style: "extension.css" | "extension_old.css") {
  emit("update:l07", { ...l07State.value, searcherStyle: style });
}

function toggleUpdateLog() {
  const nextOpen = !l07State.value.updateLogOpen;
  emit("update:l07", {
    ...l07State.value,
    updateLogOpen: nextOpen,
    config: { ...l07State.value.config, loadUpdateContent: nextOpen },
  });
}

const extraEntries: OmnisearchEntry[] = [
  { packageId: "shahai", kind: "card", id: "cangchizhibi", displayName: "苍螭之璧" },
  { packageId: "taixu", kind: "skill", id: "buff_txhj_aozhan", displayName: "太虚祝福·鏖战" },
  { packageId: "taixu", kind: "general", id: "txhj_zhaozhong", displayName: "太虚赵忠" },
  { packageId: "zuan", kind: "general", id: "ssr_luxun", displayName: "SSR陆逊" },
  { packageId: "d3", kind: "general", id: "ddd_caomao", displayName: "3D曹髦" },
  { packageId: "standard", kind: "card", id: "sha_basic", displayName: "杀（基本牌）" },
  { packageId: "standard", kind: "card", id: "shan_basic", displayName: "闪（基本牌）" },
  { packageId: "standard", kind: "card", id: "tao_basic", displayName: "桃（基本牌）" },
  { packageId: "ex", kind: "card", id: "sha_fire", displayName: "火杀（军争）" },
  { packageId: "ex", kind: "card", id: "sha_thunder", displayName: "雷杀（军争）" },
  { packageId: "standard", kind: "general", id: "zhangfei", displayName: "张飞（standard）" },
  { packageId: "standard", kind: "general", id: "zhaoyun", displayName: "赵云（standard）" },
  { packageId: "standard", kind: "general", id: "machao", displayName: "马超（standard）" },
  { packageId: "shenhua", kind: "general", id: "huangzhong", displayName: "黄忠（shenhua）" },
  { packageId: "standard", kind: "skill", id: "paoxiao", displayName: "咆哮" },
  { packageId: "standard", kind: "skill", id: "liegong", displayName: "烈弓" },
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
const paginationView = computed(() =>
  buildSimplePaginationView(Math.max(1, paged.value.totalPages), page.value, 5),
);

function executeSearch() {
  page.value = 1;
  const next = runOmnisearchQuery(props.state, queryText.value, hiddenCardIds.value, extraEntries);
  emit("update", next);
}
</script>

<template>
  <div class="modal-backdrop" role="dialog" aria-modal="true" aria-label="全能搜索牌局百科词典" data-testid="modal-omnisearch-dict">
    <div
      class="modal-card Searcher"
      :data-l07-highlight="l07State.config.css"
      :data-l07-searcher-style="l07State.searcherStyle"
      :data-l07-bg="l07State.config.background"
    >
      <header class="modal-head">
        <div>
          <h3>全能搜索 · 牌局实时百科词典 (L07 · {{ l07State.config.version }})</h3>
          <p class="muted tiny">上游基准：全能搜索/extension.js · {{ L07_CONFIG_SPECS.length }}配置 / 9表面 (SimplePagination + 6代码高亮)</p>
        </div>
        <button type="button" data-testid="btn-close-omnisearch-modal" @click="emit('close')">关闭 ✕</button>
      </header>

      <div class="l07-config-strip" data-testid="l07-config-strip">
        <label class="tiny">
          高亮:
          <select
            :value="l07State.config.css"
            data-testid="l07-select-highlight"
            @change="patchL07Config('css', ($event.target as HTMLSelectElement).value as L07OmnisearchConfig['css'])"
          >
            <option v-for="id in L07_HIGHLIGHT_CSS_IDS" :key="id" :value="id">{{ id }}</option>
          </select>
        </label>
        <label class="tiny">
          背景:
          <select
            :value="l07State.config.background"
            data-testid="l07-select-bg"
            @change="patchL07Config('background', ($event.target as HTMLSelectElement).value as L07OmnisearchConfig['background'])"
          >
            <option value="none">默认</option>
            <option value="相爱相杀">相爱相杀</option>
            <option value="才颜双绝">才颜双绝</option>
            <option value="新杀春日">新杀春日</option>
            <option value="青铜龙纹">青铜龙纹</option>
            <option value="随机">随机</option>
          </select>
        </label>
        <button
          type="button"
          data-testid="btn-l07-toggle-style"
          @click="toggleSearcherStyle(l07State.searcherStyle === 'extension.css' ? 'extension_old.css' : 'extension.css')"
        >
          {{ l07State.searcherStyle }}
        </button>
        <button type="button" data-testid="btn-l07-changelog" @click="toggleUpdateLog">
          更新日志 ({{ l07State.updateLogOpen ? "收起" : "展开" }})
        </button>
      </div>

      <div v-if="l07State.updateLogOpen" class="l07-changelog-box tiny" data-testid="l07-changelog-box">
        <strong>全能搜索 v3.2 更新说明：</strong> 集成 SimplePagination.js 分页器、6 套 highlight.js 暗色配色、禁绝暗牌窥探与 zero-eval 查询。
      </div>

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

      <pre class="l07-code-preview" data-testid="l07-highlight-preview"><code class="hljs"><span class="hljs-keyword">const</span> skill = <span class="hljs-string">"{{ paged.items[0]?.id ?? 'jianxiong' }}"</span>; <span class="hljs-comment">// {{ l07State.config.css }} · {{ l07State.config.background }}</span></code></pre>

      <nav class="simple-pagination" aria-label="SimplePagination" data-testid="l07-simple-pagination">
        <button type="button" class="page-li" :disabled="!paginationView.canPrev" @click="page = Math.max(1, page - 1)">‹</button>
        <button
          v-for="p in paginationView.pages"
          :key="p"
          type="button"
          class="page-li"
          :class="{ 'page-active': p === paginationView.pageNumber }"
          @click="page = p"
        >
          {{ p }}
        </button>
        <button type="button" class="page-li" :disabled="!paginationView.canNext" @click="page = Math.min(paginationView.totalPageCount, page + 1)">›</button>
        <span class="tiny muted">第 {{ paginationView.pageNumber }}/{{ paginationView.totalPageCount }} 页</span>
      </nav>
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
