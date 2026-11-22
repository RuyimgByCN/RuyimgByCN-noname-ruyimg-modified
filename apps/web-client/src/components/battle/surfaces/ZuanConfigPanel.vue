<script setup lang="ts">
import { computed, ref, watch } from "vue";
import {
  ZUAN_CONFIG_SPECS,
  createDefaultZuanConfig,
  parseZuanConfigImportJson,
  type ZuanConfigGroup,
} from "../../../engine/surfaces/zuanConfigSchema";
import {
  ZUAN_UI_HOOK_IDS,
  createInitialZuanUiRuntimeState,
  dispatchZuanUiHook,
  updateZuanConfigInState,
  type ZuanUiRuntimeState,
} from "../../../engine/surfaces/zuanSurface";
import type { Batch6PatchUiState } from "../../../engine/surfaces/batch6PatchSurfacesRuntime";

const props = defineProps<{
  zuanUi?: ZuanUiRuntimeState;
  batch6?: Batch6PatchUiState;
}>();

const emit = defineEmits<{
  close: [];
  updateZuan: [next: ZuanUiRuntimeState];
  updateBatch6: [next: Batch6PatchUiState];
}>();

const latestZuan = ref<ZuanUiRuntimeState>(props.zuanUi ?? createInitialZuanUiRuntimeState());
watch(
  () => props.zuanUi,
  (v) => {
    if (v) latestZuan.value = v;
  },
);

const activeGroup = ref<ZuanConfigGroup | "hooks_patch">("info_dividers");
const jsonText = ref("");
const jsonMsg = ref("");

const tabs: ReadonlyArray<{ id: ZuanConfigGroup | "hooks_patch"; label: string }> = [
  { id: "info_dividers", label: "介绍/彩蛋/分割线 (23)" },
  { id: "gameplay_modes", label: "模式与玩法 (10)" },
  { id: "visual_fx", label: "特效与指示线 (20)" },
  { id: "interaction_audio", label: "对话音效/DIY (15)" },
  { id: "hooks_patch", label: "40 钩子 & L10/L27-29 补丁" },
];

const filteredSpecs = computed(() =>
  activeGroup.value === "hooks_patch"
    ? []
    : ZUAN_CONFIG_SPECS.filter((s) => s.group === activeGroup.value),
);

function setZuanField(key: string, val: boolean | string) {
  const next = updateZuanConfigInState(latestZuan.value, { [key]: val });
  latestZuan.value = next;
  emit("updateZuan", next);
}

function fireZuanHook(hook: (typeof ZUAN_UI_HOOK_IDS)[number]) {
  const next = dispatchZuanUiHook(latestZuan.value, hook, { seat: 0, reviveTurns: 2 });
  latestZuan.value = next;
  emit("updateZuan", next);
}

function toggleBatch6Flag(key: "eCssEnabled" | "thunderCssEnabled" | "longxueMain1Enabled" | "cardFold" | "cardPhantom" | "shichangshi") {
  if (!props.batch6) return;
  emit("updateBatch6", { ...props.batch6, [key]: !props.batch6[key] });
}

function onImportJson() {
  try {
    const res = parseZuanConfigImportJson(jsonText.value);
    const next = updateZuanConfigInState(latestZuan.value, res.patch);
    latestZuan.value = next;
    emit("updateZuan", next);
    jsonMsg.value = `已导入并校验 ${res.importedKeys} 项祖安配置。`;
  } catch (err) {
    jsonMsg.value = `导入被拒绝：${err instanceof Error ? err.message : String(err)}`;
  }
}

function onExportJson() {
  jsonText.value = JSON.stringify(latestZuan.value.config, null, 2);
  jsonMsg.value = `已导出全部 ${ZUAN_CONFIG_SPECS.length} 项祖安配置 JSON`;
}

function onReset() {
  const next = updateZuanConfigInState(latestZuan.value, createDefaultZuanConfig());
  latestZuan.value = next;
  emit("updateZuan", next);
  jsonMsg.value = "已恢复祖安 68 项默认配置。";
}
</script>

<template>
  <div class="modal-backdrop" role="dialog" aria-modal="true" aria-label="祖安武将 68 配置与 L10/L27-29 面板" data-testid="modal-zuan-panel">
    <div class="modal-card zuan-modal-card">
      <header class="modal-head">
        <div>
          <h3>祖安武将 (L25 · 68配置 / 40钩子) &amp; L10/L27-L29 样式开关</h3>
          <p class="tiny muted">指示线={{ latestZuan.attackLineStyle }} · 音效={{ latestZuan.lastAudioCue ?? "待触发" }} · 最近钩子={{ latestZuan.lastControlHook ?? "无" }}</p>
        </div>
        <button type="button" data-testid="btn-close-zuan-panel" @click="emit('close')">关闭 ✕</button>
      </header>

      <nav class="zuan-tabs">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          type="button"
          class="tab-pill"
          :class="{ active: activeGroup === tab.id }"
          :data-testid="`zuan-tab-${tab.id}`"
          @click="activeGroup = tab.id"
        >
          {{ tab.label }}
        </button>
        <button type="button" class="tab-pill" data-testid="btn-zuan-reset" @click="onReset">恢复默认</button>
      </nav>

      <div v-if="activeGroup !== 'hooks_patch'" class="zuan-cfg-grid">
        <div
          v-for="spec in filteredSpecs"
          :key="spec.key"
          class="zuan-cfg-row"
          :title="spec.tooltip"
          :data-testid="`zuan-cfg-row-${spec.key}`"
        >
          <div class="cfg-meta">
            <strong>{{ spec.label }}</strong>
            <code>{{ spec.key }}</code>
            <span class="tiny muted">{{ spec.tooltip }}</span>
          </div>
          <div class="cfg-ctrl">
            <input
              v-if="spec.control === 'boolean'"
              type="checkbox"
              :checked="Boolean(latestZuan.config[spec.key])"
              :data-testid="`zuan-cfg-input-${spec.key}`"
              @change="setZuanField(spec.key, ($event.target as HTMLInputElement).checked)"
            />
            <select
              v-else-if="spec.control === 'select'"
              :value="String(latestZuan.config[spec.key])"
              :data-testid="`zuan-cfg-input-${spec.key}`"
              @change="setZuanField(spec.key, ($event.target as HTMLSelectElement).value)"
            >
              <option v-for="o in spec.options ?? []" :key="o.value" :value="o.value">{{ o.label }}</option>
            </select>
            <button
              v-else
              type="button"
              class="mini-info-btn"
              :data-testid="`zuan-cfg-input-${spec.key}`"
              @click="jsonMsg = `${spec.key}: ${spec.tooltip}`"
            >
              查看详情
            </button>
          </div>
        </div>
      </div>

      <div v-else class="hooks-patch-pane">
        <div v-if="batch6" class="patch-row" data-testid="batch6-patch-toggles">
          <label><input type="checkbox" :checked="batch6.eCssEnabled" data-testid="toggle-l27-ecss" @change="toggleBatch6Flag('eCssEnabled')" /> L27 e.css 选将美化</label>
          <label><input type="checkbox" :checked="batch6.thunderCssEnabled" data-testid="toggle-l28-thundercss" @change="toggleBatch6Flag('thunderCssEnabled')" /> L28 thunder.css 雷霆提示</label>
          <label><input type="checkbox" :checked="batch6.longxueMain1Enabled" data-testid="toggle-l29-main1" @change="toggleBatch6Flag('longxueMain1Enabled')" /> L29 main1/main1.css 潜龙书法</label>
          <label><input type="checkbox" :checked="batch6.cardFold" data-testid="toggle-l10-cardfold" @change="toggleBatch6Flag('cardFold')" /> L10 cardFold 手牌折叠</label>
          <label>
            折叠比例
            <select
              :value="String(batch6.cardFoldProportion)"
              data-testid="select-l10-cardfold-proportion"
              @change="emit('updateBatch6', { ...batch6, cardFoldProportion: Number(($event.target as HTMLSelectElement).value) as 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 })"
            >
              <option v-for="p in [3, 4, 5, 6, 7, 8, 9, 10]" :key="p" :value="String(p)">{{ p * 10 }}%</option>
            </select>
          </label>
          <label><input type="checkbox" :checked="batch6.cardPhantom" data-testid="toggle-l10-cardphantom" @change="toggleBatch6Flag('cardPhantom')" /> L10 cardPhantom 幻影出牌</label>
          <label><input type="checkbox" :checked="batch6.shichangshi" data-testid="toggle-l10-shichangshi" @change="toggleBatch6Flag('shichangshi')" /> L10 shichangshi 十常侍还原</label>
        </div>
        <div class="hook-grid">
          <button
            v-for="hk in ZUAN_UI_HOOK_IDS"
            :key="hk"
            type="button"
            class="hook-btn"
            :data-testid="`btn-zuan-hook-${hk}`"
            @click="fireZuanHook(hk)"
          >
            {{ hk }}
          </button>
        </div>
      </div>

      <div class="json-guard-box">
        <textarea v-model="jsonText" rows="2" placeholder='{"zhishixian":"laser","audio_draw":true}' data-testid="zuan-cfg-json-textarea"></textarea>
        <div class="json-actions">
          <button type="button" data-testid="btn-zuan-export-json" @click="onExportJson">导出 68 配置</button>
          <button type="button" data-testid="btn-zuan-import-json" @click="onImportJson">导入校验(裁决38)</button>
          <span v-if="jsonMsg" class="tiny" data-testid="zuan-cfg-msg">{{ jsonMsg }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-backdrop { position: fixed; inset: 0; z-index: 64; background: rgba(6, 8, 12, 0.78); display: flex; align-items: center; justify-content: center; padding: 1rem; }
.modal-card { width: min(880px, 96vw); max-height: 88vh; overflow-y: auto; background: #141922; border: 1.5px solid #f59e0b; border-radius: 12px; padding: 1rem 1.2rem; color: #f8fafc; }
.modal-head { display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 0.7rem; }
.modal-head h3 { margin: 0; font-size: 0.98rem; }
.zuan-tabs { display: flex; flex-wrap: wrap; gap: 0.3rem; margin-bottom: 0.65rem; }
.tab-pill, .hook-btn, .mini-info-btn { padding: 0.22rem 0.5rem; border-radius: 6px; border: 1px solid rgba(255,255,255,0.18); background: rgba(255,255,255,0.06); color: #e2e8f0; font-size: 0.72rem; cursor: pointer; }
.tab-pill.active { border-color: #f59e0b; background: rgba(245,158,11,0.22); color: #fef3c7; }
.zuan-cfg-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(260px, 1fr)); gap: 0.4rem; max-height: 44vh; overflow-y: auto; }
.zuan-cfg-row { display: flex; justify-content: space-between; gap: 0.45rem; padding: 0.38rem 0.5rem; border-radius: 7px; border: 1px solid rgba(255,255,255,0.1); background: rgba(255,255,255,0.03); }
.cfg-meta { display: flex; flex-direction: column; gap: 0.1rem; font-size: 0.72rem; }
.cfg-meta code { color: #fbbf24; font-size: 0.68rem; }
.patch-row { display: flex; flex-wrap: wrap; gap: 0.65rem; margin-bottom: 0.6rem; font-size: 0.76rem; }
.hook-grid { display: flex; flex-wrap: wrap; gap: 0.28rem; max-height: 36vh; overflow-y: auto; }
.json-guard-box { margin-top: 0.65rem; display: flex; flex-direction: column; gap: 0.35rem; }
.json-guard-box textarea { width: 100%; background: #0b0f16; color: #f8fafc; border: 1px solid rgba(255,255,255,0.2); border-radius: 6px; padding: 0.35rem; font-size: 0.74rem; }
.json-actions { display: flex; gap: 0.45rem; align-items: center; }
.tiny { font-size: 0.72rem; margin: 0.15rem 0 0; }
</style>
