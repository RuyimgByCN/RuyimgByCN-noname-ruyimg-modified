<script setup lang="ts">
import { computed, ref, watch } from "vue";
import {
  QHLY_CONFIG_SPECS,
  createDefaultQhlyConfig,
  parseQhlyConfigImportJson,
  type QhlyConfigCategory,
  type QhlyUiConfig,
} from "../../../engine/surfaces/qhlyConfigSchema";
import {
  clearQhlySkinSettings,
  restoreOfficialSkins,
  updateQhlyConfigInState,
  type QhlyRuntimeState,
} from "../../../engine/surfaces/qhlySurface";

const props = defineProps<{
  state: QhlyRuntimeState;
}>();

const emit = defineEmits<{
  update: [next: QhlyRuntimeState];
}>();

const latestState = ref<QhlyRuntimeState>(props.state);
watch(
  () => props.state,
  (next) => {
    latestState.value = next;
  },
);

function emitNext(next: QhlyRuntimeState) {
  latestState.value = next;
  emit("update", next);
}

const activeCat = ref<QhlyConfigCategory>("ui");
const jsonText = ref("");
const jsonMsg = ref("");

const categoryTabs: ReadonlyArray<{ id: QhlyConfigCategory; label: string }> = [
  { id: "ui", label: "UI设置 (7)" },
  { id: "window", label: "窗口与立绘 (34)" },
  { id: "decade", label: "十周年联动 (14)" },
  { id: "audio", label: "音效与防啰嗦 (8)" },
  { id: "theme_tune", label: "水墨/海克斯 (11)" },
  { id: "actions", label: "工具与清理 (8)" },
];

const filteredSpecs = computed(() =>
  QHLY_CONFIG_SPECS.filter((s) => s.category === activeCat.value),
);

function setBool(key: keyof QhlyUiConfig, checked: boolean) {
  emitNext(updateQhlyConfigInState(latestState.value, { [key]: checked } as Partial<QhlyUiConfig>));
}

function setSelect(key: keyof QhlyUiConfig, val: string) {
  emitNext(updateQhlyConfigInState(latestState.value, { [key]: val } as Partial<QhlyUiConfig>));
}

function onAction(key: keyof QhlyUiConfig) {
  if (key === "qhly_clear") {
    emitNext(clearQhlySkinSettings(latestState.value));
    return;
  }
  if (key === "qhly_restore") {
    emitNext(restoreOfficialSkins(latestState.value));
    return;
  }
  emitNext(
    updateQhlyConfigInState(latestState.value, { qhly_plugin: !latestState.value.config.qhly_plugin }),
  );
}

function onExport() {
  jsonText.value = JSON.stringify(latestState.value.config, null, 2);
  jsonMsg.value = `已导出全部 ${QHLY_CONFIG_SPECS.length} 项千幻配置 JSON（仅本地）`;
}

function onImport() {
  try {
    const res = parseQhlyConfigImportJson(jsonText.value);
    emitNext(updateQhlyConfigInState(latestState.value, res.patch));
    jsonMsg.value = `已导入并校验 ${res.importedKeys} 项千幻配置。`;
  } catch (err) {
    jsonMsg.value = `导入被拒绝：${err instanceof Error ? err.message : String(err)}`;
  }
}

function onReset() {
  emitNext(updateQhlyConfigInState(latestState.value, createDefaultQhlyConfig()));
  jsonMsg.value = "已恢复千幻 82 项默认配置。";
}
</script>

<template>
  <div class="qhly-cfg-root" data-testid="qhly-82-configs-panel">
    <nav class="qhly-cat-tabs" role="tablist">
      <button
        v-for="tab in categoryTabs"
        :key="tab.id"
        type="button"
        class="tab-pill"
        :class="{ active: activeCat === tab.id }"
        :data-testid="`qhly-cfg-tab-${tab.id}`"
        @click="activeCat = tab.id"
      >
        {{ tab.label }}
      </button>
      <button type="button" class="tab-pill" data-testid="btn-qhly-cfg-reset" @click="onReset">
        恢复默认
      </button>
    </nav>

    <div class="qhly-cfg-grid">
      <div
        v-for="spec in filteredSpecs"
        :key="spec.key"
        class="qhly-cfg-row"
        :data-testid="`qhly-cfg-row-${spec.key}`"
      >
        <div class="cfg-meta">
          <strong>{{ spec.label }}</strong>
          <code>{{ spec.key }}</code>
          <span class="tiny muted">extension.js:{{ spec.upstreamLine }}</span>
        </div>
        <div class="cfg-ctrl">
          <label v-if="spec.control === 'boolean'" class="bool-toggle">
            <input
              type="checkbox"
              :checked="Boolean(state.config[spec.key])"
              :data-testid="`qhly-cfg-input-${spec.key}`"
              @change="setBool(spec.key, ($event.target as HTMLInputElement).checked)"
            />
            <span>{{ state.config[spec.key] ? "开" : "关" }}</span>
          </label>
          <select
            v-else-if="spec.control === 'select'"
            :value="String(state.config[spec.key])"
            :data-testid="`qhly-cfg-input-${spec.key}`"
            @change="setSelect(spec.key, ($event.target as HTMLSelectElement).value)"
          >
            <option v-for="opt in spec.options ?? []" :key="opt.value" :value="opt.value">
              {{ opt.label }}
            </option>
          </select>
          <button
            v-else-if="spec.control === 'action'"
            type="button"
            class="action-pill"
            :data-testid="`qhly-cfg-input-${spec.key}`"
            @click="onAction(spec.key)"
          >
            {{ spec.key === "qhly_clear" ? "执行清空" : spec.key === "qhly_restore" ? "恢复官方" : "切换插件视图" }}
          </button>
          <span v-else class="tiny muted" :data-testid="`qhly-cfg-input-${spec.key}`">
            {{ spec.key === "skinAudioList" || spec.key === "audioReplace" ? "JSON槽位" : "分组" }}
          </span>
        </div>
      </div>
    </div>

    <div v-if="state.lastOperationNotice" class="tiny op-notice" data-testid="qhly-op-notice">
      {{ state.lastOperationNotice }} · 插件：{{ state.activePlugins.join(" / ") }}
    </div>

    <div class="json-guard-box">
      <textarea
        v-model="jsonText"
        rows="3"
        placeholder='{"qhly_fontsize1":"7","qhly_notbb":"5","qhly_skinButton":true}'
        data-testid="qhly-cfg-json-textarea"
      ></textarea>
      <div class="json-actions">
        <button type="button" data-testid="btn-qhly-export-json" @click="onExport">导出 82 配置</button>
        <button type="button" data-testid="btn-qhly-import-json" @click="onImport">导入校验(裁决38)</button>
        <span v-if="jsonMsg" class="tiny" data-testid="qhly-cfg-import-msg">{{ jsonMsg }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.qhly-cfg-root {
  display: flex;
  flex-direction: column;
  gap: 0.55rem;
}
.qhly-cat-tabs {
  display: flex;
  flex-wrap: wrap;
  gap: 0.3rem;
}
.tab-pill {
  padding: 0.22rem 0.5rem;
  border-radius: 6px;
  font-size: 0.72rem;
  border: 1px solid rgba(255, 255, 255, 0.16);
  background: rgba(255, 255, 255, 0.05);
  color: #e2e8f0;
  cursor: pointer;
}
.tab-pill.active {
  border-color: #f59e0b;
  background: rgba(245, 158, 11, 0.22);
  color: #fde68a;
}
.qhly-cfg-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
  gap: 0.4rem;
  max-height: 290px;
  overflow-y: auto;
}
.qhly-cfg-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 0.45rem;
  padding: 0.35rem 0.5rem;
  border-radius: 6px;
  border: 1px solid rgba(255, 255, 255, 0.09);
  background: rgba(0, 0, 0, 0.24);
}
.cfg-meta {
  display: flex;
  flex-direction: column;
  min-width: 0;
  font-size: 0.74rem;
}
.cfg-meta code {
  color: #93c5fd;
  font-size: 0.68rem;
}
.cfg-ctrl select,
.action-pill {
  font-size: 0.72rem;
  padding: 0.15rem 0.35rem;
  border-radius: 4px;
  border: 1px solid rgba(232, 165, 75, 0.42);
  background: #0f172a;
  color: #fde68a;
}
.bool-toggle {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  font-size: 0.74rem;
}
.json-guard-box {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
}
.json-guard-box textarea {
  width: 100%;
  background: #0b0f16;
  color: #e2e8f0;
  border: 1px solid rgba(255, 255, 255, 0.14);
  border-radius: 6px;
  padding: 0.35rem;
  font-family: var(--mono);
  font-size: 0.72rem;
}
.json-actions {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.4rem;
}
.json-actions button {
  padding: 0.2rem 0.5rem;
  border-radius: 5px;
  border: 1px solid rgba(232, 165, 75, 0.42);
  background: rgba(232, 165, 75, 0.15);
  color: #fef3c7;
  font-size: 0.74rem;
  cursor: pointer;
}
.op-notice {
  color: #86efac;
}
.tiny {
  font-size: 0.7rem;
}
</style>
