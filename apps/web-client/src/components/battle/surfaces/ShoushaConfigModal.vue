<script setup lang="ts">
import { computed, ref } from "vue";
import {
  SHOUSHA_CONFIG_SPECS,
  createDefaultShoushaConfig,
  parseShoushaConfigImportJson,
  type ShoushaConfigCategory,
  type ShoushaUiConfig,
} from "../../../engine/surfaces/shoushaConfigSchema";
import { SHOUSHA_SURFACE_SPECS } from "../../../engine/surfaces/shoushaSurfacesRuntime";
import type { ShoushaUiRuntimeState } from "../../../engine/surfaces/shoushaUiSurface";

const props = defineProps<{
  state?: ShoushaUiRuntimeState;
}>();

const emit = defineEmits<{
  close: [];
  updateConfig: [patch: Partial<ShoushaUiConfig>];
  resetConfig: [];
}>();

const activeTab = ref<ShoushaConfigCategory | "surfaces">("jindutiao");
const expandedInfoKey = ref<string | null>(null);
const importText = ref("");
const importMessage = ref("");

const currentConfig = computed<ShoushaUiConfig>(
  () => props.state?.config ?? createDefaultShoushaConfig(),
);

const activeSurfaceList = computed(() => props.state?.activeSurfaces ?? []);
const filteredSpecs = computed(() =>
  SHOUSHA_CONFIG_SPECS.filter((item) => item.category === activeTab.value),
);

const jsSurfaces = computed(() => SHOUSHA_SURFACE_SPECS.filter((s) => s.kind === "js"));
const cssSurfaces = computed(() => SHOUSHA_SURFACE_SPECS.filter((s) => s.kind === "css"));

const categoryTabs: ReadonlyArray<{ id: ShoushaConfigCategory | "surfaces"; label: string }> = [
  { id: "jindutiao", label: "进度条与将灯 (8)" },
  { id: "prompts", label: "阶段与播报 (7)" },
  { id: "appearance", label: "界面与美化 (11)" },
  { id: "select", label: "选将与手气卡 (5)" },
  { id: "info", label: "说明与鸣谢 (17)" },
  { id: "surfaces", label: "40 渲染面清单" },
];

function toggleBool(key: keyof ShoushaUiConfig, checked: boolean) {
  emit("updateConfig", { [key]: checked } as Partial<ShoushaUiConfig>);
}

function updateSelect(key: keyof ShoushaUiConfig, val: string) {
  emit("updateConfig", { [key]: val } as Partial<ShoushaUiConfig>);
}

function onExportJson() {
  importText.value = JSON.stringify(currentConfig.value, null, 2);
  importMessage.value = "已导出当前全部 48 项手杀UI 配置 JSON（本地生效，未上传网络）。";
}

function onImportJson() {
  try {
    const res = parseShoushaConfigImportJson(importText.value);
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
    aria-label="手杀UI 48配置与40渲染面"
    data-testid="l14-shousha-menu-modal"
    @click.self="emit('close')"
  >
    <div
      class="modal-card shousha-modal-card"
      :data-kgmh="currentConfig.KGMH"
      :data-bg="currentConfig.yangshi_bg"
    >
      <header class="modal-head">
        <div>
          <h2>手杀UI 控制台 (L14 · {{ SHOUSHA_CONFIG_SPECS.length }}配置 / {{ SHOUSHA_SURFACE_SPECS.length }}表面)</h2>
          <p class="tiny muted">
            实时生效于牌桌与选将区；所有配置仅本地保存与渲染，不含商业音视频素材（裁决 35/38）。
          </p>
        </div>
        <div class="head-actions">
          <button
            type="button"
            class="btn-secondary"
            data-testid="btn-shousha-config-reset"
            @click="emit('resetConfig')"
          >
            恢复默认
          </button>
          <button type="button" class="btn-close" data-testid="btn-close-shousha-modal" @click="emit('close')">
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
          :data-testid="`shousha-cfg-tab-${tab.id}`"
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
          :data-testid="`shousha-cfg-row-${spec.key}`"
        >
          <div class="cfg-info">
            <strong>{{ spec.label }}</strong>
            <code>{{ spec.key }}</code>
            <span class="tiny muted">extension.js:{{ spec.upstreamLine }}</span>
            <p
              v-if="spec.intro && (spec.control !== 'info' || expandedInfoKey === spec.key)"
              class="tiny muted intro-line"
            >
              {{ spec.intro }}
            </p>
          </div>
          <div class="cfg-control">
            <label v-if="spec.control === 'boolean'" class="switch-label shousha-switch-label">
              <input
                type="checkbox"
                :checked="Boolean(currentConfig[spec.key])"
                :data-testid="`shousha-cfg-input-${spec.key}`"
                @change="toggleBool(spec.key, ($event.target as HTMLInputElement).checked)"
              />
              <span>{{ Boolean(currentConfig[spec.key]) ? "开启" : "关闭" }}</span>
            </label>
            <select
              v-else-if="spec.control === 'select'"
              :value="String(currentConfig[spec.key])"
              :aria-label="spec.label"
              :data-testid="`shousha-cfg-input-${spec.key}`"
              @change="updateSelect(spec.key, ($event.target as HTMLSelectElement).value)"
            >
              <option v-for="opt in spec.options ?? []" :key="opt.value" :value="opt.value">
                {{ opt.label }}
              </option>
            </select>
            <button
              v-else-if="spec.intro"
              type="button"
              class="btn-info-toggle"
              :data-testid="`shousha-cfg-input-${spec.key}`"
              @click="expandedInfoKey = expandedInfoKey === spec.key ? null : spec.key"
            >
              {{ expandedInfoKey === spec.key ? "收起说明" : "展开说明" }}
            </button>
            <span v-else class="readonly-pill" :data-testid="`shousha-cfg-input-${spec.key}`">
              分隔条目
            </span>
          </div>
        </div>
      </div>

      <div v-else class="surfaces-view" data-testid="shousha-40-surfaces-list">
        <p class="tiny muted">
          JS 模块（{{ jsSurfaces.length }}）：{{ jsSurfaces.map((s) => s.id).join("、") }}
        </p>
        <div class="css-pills">
          <span
            v-for="surf in SHOUSHA_SURFACE_SPECS"
            :key="surf.id"
            class="css-pill"
            :class="{ active: activeSurfaceList.includes(surf.id) }"
            :title="`${surf.role} (${surf.upstreamFile}:${surf.upstreamLines})`"
            :data-testid="`shousha-surface-pill-${surf.id}`"
          >
            {{ surf.id }}
          </span>
        </div>
        <div class="json-import-box">
          <h4>本地配置 JSON 导出 / 导入（裁决 38 安全校验）</h4>
          <textarea
            v-model="importText"
            rows="4"
            placeholder='{"yangshi":"off","jindutiaoYangshi":"2","GTBB":true}'
            data-testid="shousha-cfg-json-textarea"
          ></textarea>
          <div class="row-btns">
            <button type="button" data-testid="btn-shousha-export-json" @click="onExportJson">
              导出当前配置 JSON
            </button>
            <button type="button" data-testid="btn-shousha-import-json" @click="onImportJson">
              导入并校验配置 JSON
            </button>
            <span v-if="importMessage" class="tiny status-msg" data-testid="shousha-cfg-import-msg">
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
.btn-info-toggle,
.readonly-pill {
  font-size: 0.72rem;
  padding: 0.15rem 0.4rem;
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.08);
  border: 1px solid rgba(255, 255, 255, 0.14);
  color: #e2e8f0;
}
.css-pills {
  display: flex;
  flex-wrap: wrap;
  gap: 0.3rem;
}
.css-pill {
  font-size: 0.72rem;
  padding: 0.15rem 0.42rem;
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.12);
  color: #94a3b8;
}
.css-pill.active {
  background: rgba(16, 185, 129, 0.18);
  border-color: #34d399;
  color: #a7f3d0;
}
.json-import-box {
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
  margin-top: 0.5rem;
}
.json-import-box h4 {
  margin: 0;
  font-size: 0.85rem;
  color: #fde68a;
}
.json-import-box textarea {
  width: 100%;
  background: #0b0f16;
  color: #e2e8f0;
  border: 1px solid rgba(255, 255, 255, 0.15);
  border-radius: 6px;
  padding: 0.45rem;
  font-family: var(--mono);
  font-size: 0.75rem;
}
.row-btns {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.45rem;
}
.row-btns button,
.btn-secondary,
.btn-close {
  padding: 0.28rem 0.65rem;
  border-radius: 6px;
  border: 1px solid rgba(232, 165, 75, 0.42);
  background: rgba(232, 165, 75, 0.14);
  color: #fef3c7;
  font-size: 0.78rem;
  cursor: pointer;
}
.status-msg {
  color: #86efac;
}
</style>
