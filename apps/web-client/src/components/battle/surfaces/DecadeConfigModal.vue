<script setup lang="ts">
import { computed, ref } from "vue";
import {
  DECADE_CONFIG_SPECS,
  DECADE_PLAYER_NODE_KEYS,
  createDefaultDecadeConfig,
  type DecadeConfigCategory,
  type DecadeUiConfig,
} from "../../../engine/surfaces/decadeConfigSchema";
import {
  DECADE_CSS_SURFACE_SPECS,
  DECADE_JS_SURFACE_IDS,
  parseDecadeConfigImportJson,
} from "../../../engine/surfaces/decadeSurfacesRuntime";
import {
  updateDecadePlayerCssVariant,
  type DecadeUiRuntimeState,
} from "../../../engine/surfaces/decadeUiSurface";

const PLAYER_CSS_VARIANTS: ReadonlyArray<DecadeUiRuntimeState["playerCssVariant"]> = [
  "player_new.css",
  "player1_PC.css",
  "player1.css",
  "player2_PC.css",
  "player2.css",
  "player3_PC.css",
  "player3.css",
];

const props = defineProps<{
  state?: DecadeUiRuntimeState;
}>();

const emit = defineEmits<{
  close: [];
  updateConfig: [patch: Partial<DecadeUiConfig>];
  resetConfig: [];
  updateDecadeUi: [next: DecadeUiRuntimeState];
}>();

function onSelectPlayerCssVariant(variant: DecadeUiRuntimeState["playerCssVariant"]) {
  if (!props.state) return;
  emit("updateDecadeUi", updateDecadePlayerCssVariant(props.state, variant));
}

const activeTab = ref<DecadeConfigCategory | "surfaces">("layout");
const importText = ref("");
const importMessage = ref("");

const currentConfig = computed<DecadeUiConfig>(
  () => props.state?.config ?? createDefaultDecadeConfig(),
);

const activeCssList = computed(() => props.state?.activeCssSurfaces ?? []);

const filteredSpecs = computed(() =>
  DECADE_CONFIG_SPECS.filter((item) => item.category === activeTab.value),
);

const categoryTabs: ReadonlyArray<{ id: DecadeConfigCategory | "surfaces"; label: string }> = [
  { id: "layout", label: "界面布局 (10)" },
  { id: "cards", label: "卡牌外观 (8)" },
  { id: "marks", label: "标记身份 (8)" },
  { id: "effects", label: "特效动皮 (14)" },
  { id: "system", label: "系统节点 (4)" },
  { id: "surfaces", label: "43 渲染面清单" },
];

function toggleBool(key: keyof DecadeUiConfig, checked: boolean) {
  emit("updateConfig", { [key]: checked } as Partial<DecadeUiConfig>);
}

function updateSelect(key: keyof DecadeUiConfig, val: string) {
  emit("updateConfig", { [key]: val } as Partial<DecadeUiConfig>);
}

function onExportJson() {
  importText.value = JSON.stringify(currentConfig.value, null, 2);
  importMessage.value = "已导出当前全部 44 项十周年UI 配置 JSON（本地生效，未上传网络）。";
}

function onImportJson() {
  try {
    const res = parseDecadeConfigImportJson(importText.value);
    emit("updateConfig", res.patch);
    importMessage.value = `已导入并校验本地配置（覆盖 ${res.importedKeys} 个键）。`;
  } catch (err) {
    importMessage.value = `导入被拒绝：${err instanceof Error ? err.message : String(err)}`;
  }
}
</script>

<template>
  <div
    class="modal-backdrop"
    role="dialog"
    aria-modal="true"
    aria-label="十周年UI 44配置与43渲染面"
    data-testid="l08-decade-menu-modal"
    @click.self="emit('close')"
  >
    <div class="modal-card">
      <header class="modal-head">
        <div>
          <h2>十周年UI 设置与渲染面控制台 (L08 · 44配置 / 43表面)</h2>
          <p class="tiny muted">
            实时生效于牌桌；所有配置仅本地保存与渲染，不含商业 Spine/音频二进制素材（裁决 35/38）。
          </p>
        </div>
        <div class="head-actions">
          <button
            type="button"
            class="btn-secondary"
            data-testid="btn-decade-config-reset"
            @click="emit('resetConfig')"
          >
            恢复默认
          </button>
          <button type="button" class="btn-close" data-testid="btn-close-decade-modal" @click="emit('close')">
            ✕ 关闭
          </button>
        </div>
      </header>

      <nav class="tab-bar" role="tablist">
        <button
          v-for="tab in categoryTabs"
          :key="tab.id"
          type="button"
          role="tab"
          class="tab-btn"
          :class="{ active: activeTab === tab.id }"
          :data-testid="`decade-cfg-tab-${tab.id}`"
          @click="activeTab = tab.id"
        >
          {{ tab.label }}
        </button>
      </nav>

      <div v-if="activeTab !== 'surfaces'" class="cfg-grid">
        <div
          v-for="spec in filteredSpecs"
          :key="spec.key"
          class="cfg-row"
          :data-testid="`decade-cfg-row-${spec.key}`"
        >
          <div class="cfg-info">
            <strong>{{ spec.label }}</strong>
            <code>{{ spec.key }}</code>
            <span class="tiny muted">extension.js:{{ spec.upstreamLine }}</span>
            <p v-if="spec.intro" class="tiny muted intro-line">{{ spec.intro }}</p>
          </div>
          <div class="cfg-control">
            <label v-if="spec.control === 'boolean'" class="switch-label">
              <input
                type="checkbox"
                :checked="Boolean(currentConfig[spec.key])"
                :data-testid="`decade-cfg-input-${spec.key}`"
                @change="toggleBool(spec.key, ($event.target as HTMLInputElement).checked)"
              />
              <span>{{ Boolean(currentConfig[spec.key]) ? "开启" : "关闭" }}</span>
            </label>
            <select
              v-else-if="spec.control === 'select'"
              :value="String(currentConfig[spec.key])"
              :aria-label="spec.label"
              :data-testid="`decade-cfg-input-${spec.key}`"
              @change="updateSelect(spec.key, ($event.target as HTMLSelectElement).value)"
            >
              <option v-for="opt in spec.options ?? []" :key="opt.value" :value="opt.value">
                {{ opt.label }}
              </option>
            </select>
            <span v-else class="readonly-pill" :data-testid="`decade-cfg-input-${spec.key}`">
              <template v-if="spec.key === 'node'">
                {{ DECADE_PLAYER_NODE_KEYS.length }} 项槽位已绑定
              </template>
              <template v-else>
                {{ currentConfig._connectMode ? "联机 (_connectMode=true)" : "单机 (_connectMode=false)" }}
              </template>
            </span>
          </div>
        </div>
      </div>

      <div v-else class="surfaces-view" data-testid="decade-43-surfaces-list">
        <div class="player-css-switcher">
          <span class="tiny muted">座位边框 CSS (<code>player1..3[_PC].css</code>)：</span>
          <select
            :value="props.state?.playerCssVariant ?? 'player_new.css'"
            data-testid="decade-player-css-select"
            @change="
              onSelectPlayerCssVariant(
                ($event.target as HTMLSelectElement).value as DecadeUiRuntimeState['playerCssVariant'],
              )
            "
          >
            <option v-for="v in PLAYER_CSS_VARIANTS" :key="v" :value="v">{{ v }}</option>
          </select>
        </div>
        <p class="tiny muted">
          JS 模块（{{ DECADE_JS_SURFACE_IDS.length }}）：{{ DECADE_JS_SURFACE_IDS.join("、") }}
        </p>
        <div class="css-pills">
          <span
            v-for="css in DECADE_CSS_SURFACE_SPECS"
            :key="css.id"
            class="css-pill"
            :class="{ active: activeCssList.includes(css.id) }"
            :title="`${css.role} (${css.upstreamLines})`"
          >
            {{ css.id }}
          </span>
        </div>
        <div class="json-import-box">
          <h4>本地配置 JSON 导出 / 导入（裁决 38 安全校验）</h4>
          <textarea
            v-model="importText"
            rows="4"
            placeholder='{"cardBorder":"night","skill_control_gold":"forever"}'
            data-testid="decade-cfg-json-textarea"
          ></textarea>
          <div class="row-btns">
            <button type="button" data-testid="btn-decade-export-json" @click="onExportJson">
              导出当前配置 JSON
            </button>
            <button type="button" data-testid="btn-decade-import-json" @click="onImportJson">
              导入并校验配置 JSON
            </button>
            <span v-if="importMessage" class="tiny status-msg" data-testid="decade-cfg-import-msg">
              {{ importMessage }}
            </span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-backdrop {
  position: fixed;
  inset: 0;
  z-index: 60;
  background: rgba(6, 8, 12, 0.78);
  display: grid;
  place-items: center;
  padding: 1rem;
}
.modal-card {
  width: min(920px, 100%);
  max-height: 88vh;
  overflow-y: auto;
  background: #141922;
  border: 1.5px solid rgba(232, 165, 75, 0.55);
  border-radius: 12px;
  padding: 1rem 1.2rem;
  color: #f4ebd4;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}
.modal-head {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 0.75rem;
}
.modal-head h2 {
  margin: 0;
  font-size: 1.05rem;
  color: #fde68a;
}
.head-actions {
  display: flex;
  gap: 0.4rem;
}
.tab-bar {
  display: flex;
  flex-wrap: wrap;
  gap: 0.35rem;
}
.tab-btn {
  padding: 0.3rem 0.65rem;
  border-radius: 6px;
  border: 1px solid rgba(255, 255, 255, 0.14);
  background: rgba(255, 255, 255, 0.05);
  color: #d6cbb5;
  font-size: 0.78rem;
  cursor: pointer;
}
.tab-btn.active {
  border-color: #e8a54b;
  background: rgba(232, 165, 75, 0.2);
  color: #fde68a;
}
.cfg-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(360px, 1fr));
  gap: 0.55rem;
}
.cfg-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 0.6rem;
  padding: 0.45rem 0.65rem;
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.08);
}
.cfg-info {
  display: flex;
  flex-direction: column;
  gap: 0.12rem;
  min-width: 0;
}
.cfg-info code {
  font-size: 0.72rem;
  color: #93c5fd;
}
.intro-line {
  margin: 0;
}
.cfg-control select {
  background: #0d1118;
  color: #f4ebd4;
  border: 1px solid rgba(232, 165, 75, 0.4);
  border-radius: 5px;
  padding: 0.22rem 0.4rem;
  font-size: 0.78rem;
}
.switch-label {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  font-size: 0.8rem;
  cursor: pointer;
}
.readonly-pill {
  font-size: 0.72rem;
  padding: 0.15rem 0.4rem;
  border-radius: 4px;
  background: rgba(148, 163, 184, 0.15);
  color: #cbd5e1;
}
.css-pills {
  display: flex;
  flex-wrap: wrap;
  gap: 0.3rem;
  margin-top: 0.4rem;
}
.css-pill {
  font-family: var(--mono);
  font-size: 0.7rem;
  padding: 0.14rem 0.42rem;
  border-radius: 4px;
  border: 1px solid rgba(255, 255, 255, 0.12);
  color: #94a3b8;
}
.css-pill.active {
  border-color: #34d399;
  color: #86efac;
  background: rgba(52, 211, 153, 0.14);
}
.json-import-box {
  margin-top: 0.75rem;
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
}
.json-import-box textarea {
  width: 100%;
  background: #0b0e14;
  color: #e2e8f0;
  border: 1px solid rgba(232, 165, 75, 0.35);
  border-radius: 6px;
  padding: 0.4rem;
  font-family: var(--mono);
  font-size: 0.75rem;
}
.row-btns {
  display: flex;
  flex-wrap: wrap;
  gap: 0.45rem;
  align-items: center;
}
.status-msg {
  color: #86efac;
}
.tiny {
  font-size: 0.72rem;
}
</style>
