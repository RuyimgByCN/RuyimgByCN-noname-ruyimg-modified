<script setup lang="ts">
import { computed, ref, watch } from "vue";
import {
  SKIN_SWITCH_CONFIG_SPECS,
  parseSkinSwitchConfigImportJson,
  type SkinSwitchUiConfig,
} from "../../../engine/surfaces/skinSwitchConfigSchema";
import {
  SKIN_SWITCH_SURFACE_IDS,
  createInitialSkinSwitchRuntimeState,
  dispatchSkinSwitchHook,
  runSkinSwitchActionConfig,
  skinSwitchArenaDatasetAttrs,
  updateSkinSwitchConfigInState,
  type SkinSwitchRuntimeState,
  type SkinSwitchSurfaceId,
  type SkinSwitchUiHookId,
} from "../../../engine/surfaces/skinSwitchSurface";
import Spine40SkinImportBox from "../surfaces/Spine40SkinImportBox.vue";

const props = defineProps<{
  state?: SkinSwitchRuntimeState;
}>();

const emit = defineEmits<{
  close: [];
  update: [next: SkinSwitchRuntimeState];
}>();

const latest = ref<SkinSwitchRuntimeState>(props.state ?? createInitialSkinSwitchRuntimeState());
watch(
  () => props.state,
  (v) => {
    if (v) latest.value = v;
  },
);

const activeTab = ref<"vault" | "configs" | "spine40">("vault");
const standX = ref(12);
const standY = ref(-8);
const scalePct = ref(108);
const seatScreenXPct = ref(28);
const jsonText = ref("");
const jsonMsg = ref("");

const current = computed(() => latest.value);
const surfaceAttrs = computed(() => skinSwitchArenaDatasetAttrs(current.value));

function emitNext(next: SkinSwitchRuntimeState) {
  latest.value = next;
  emit("update", next);
}

function triggerChukuang(hook: SkinSwitchUiHookId) {
  const next = dispatchSkinSwitchHook(current.value, {
    hook,
    seat: 0,
    generalId: "caocao",
    cardName: hook === "_checkDcdShan" ? "shan" : "sha",
    cardNature: "fire",
    seatScreenXPct: seatScreenXPct.value,
    doubleAvatar: false,
    hp: hook === "_pfqh_check_hp_change" ? 1 : 4,
    roundNumber: 2,
    roundThreshold: 2,
    faction: "shen",
  });
  emitNext({
    ...next,
    activeModal: "cang-zhen-ge",
  });
}

function selectSurface(s: SkinSwitchSurfaceId) {
  emitNext({
    ...current.value,
    activeSurface: s,
    toastMessage: `L24 切换样式面: ${s}`,
  });
}

function setBoolConfig(key: keyof SkinSwitchUiConfig, checked: boolean) {
  emitNext(updateSkinSwitchConfigInState(current.value, { [key]: checked } as Partial<SkinSwitchUiConfig>));
}

function triggerActionConfig(key: "previewDynamic" | "resetArchiveDynamicSkin" | "genDynamicSkin" | "genDyTempFile") {
  const next = runSkinSwitchActionConfig(current.value, key);
  emitNext(next);
  if (key === "previewDynamic") activeTab.value = "spine40";
}

function onImportJson() {
  try {
    const res = parseSkinSwitchConfigImportJson(jsonText.value);
    emitNext(updateSkinSwitchConfigInState(current.value, res.patch));
    jsonMsg.value = `已导入并校验 ${res.importedKeys} 项皮肤切换配置。`;
  } catch (err) {
    jsonMsg.value = `导入被拒绝：${err instanceof Error ? err.message : String(err)}`;
  }
}

function onExportJson() {
  jsonText.value = JSON.stringify(current.value.config, null, 2);
  jsonMsg.value = `已导出全部 ${SKIN_SWITCH_CONFIG_SPECS.length} 项皮肤切换配置 JSON`;
}
</script>

<template>
  <div class="modal-backdrop" role="dialog" aria-modal="true" aria-label="皮肤切换藏珍阁与坐标编辑器" data-testid="modal-skinswitch-vault">
    <div
      class="modal-card skin-switch-modal-card"
      v-bind="surfaceAttrs"
      data-testid="skinswitch-modal-card"
    >
      <header class="modal-head">
        <div>
          <h3>皮肤切换 · 藏珍阁与出框参数编辑器 (L24 · 14配置/22钩子/10表面)</h3>
          <p class="muted tiny">
            激活表面：<code>{{ current.activeSurface }}</code> · 形态：<code>{{ current.specialFormBySeat[0] ?? "base" }}</code> · 音频队列：{{ current.audioPlayQueue.length }}
          </p>
        </div>
        <div class="head-tabs">
          <button type="button" class="tab-pill" :class="{ active: activeTab === 'vault' }" data-testid="btn-skinswitch-tab-vault" @click="activeTab = 'vault'">藏珍阁 &amp; 出框动作</button>
          <button type="button" class="tab-pill" :class="{ active: activeTab === 'configs' }" data-testid="btn-skinswitch-tab-configs" @click="activeTab = 'configs'">14 配置 &amp; 10 表面</button>
          <button type="button" class="tab-pill" :class="{ active: activeTab === 'spine40' }" data-testid="btn-skinswitch-tab-spine40" @click="activeTab = 'spine40'">Spine 4.0 自备导入</button>
          <button type="button" class="tab-pill" :class="{ active: current.gxnrDialogOpen }" data-testid="btn-skinswitch-gxnr" @click="emitNext({ ...current, gxnrDialogOpen: !current.gxnrDialogOpen })">更新说明 ({{ current.config.GXNR }})</button>
          <button type="button" data-testid="btn-close-skinswitch-modal" @click="emit('close')">关闭 ✕</button>
        </div>
      </header>

      <div v-if="current.gxnrDialogOpen" class="skinswitch-toast" data-testid="skinswitch-gxnr-dialog" style="margin-bottom: 0.6rem;">
        <span class="skinswitch-icon-glyph">★</span> L24 皮肤切换 (1.24 净室版)：自备骨骼导入 + 坐标存档 + 出框翻转与动态背景裁剪均已原生接入。
      </div>

      <div v-if="activeTab === 'vault'" class="vault-body">
        <div class="sliders">
          <label>
            立绘偏移 X ({{ standX }}px)
            <input v-model.number="standX" type="range" min="-80" max="80" data-testid="slider-skinswitch-x" />
          </label>
          <label>
            立绘偏移 Y ({{ standY }}px)
            <input v-model.number="standY" type="range" min="-80" max="80" data-testid="slider-skinswitch-y" />
          </label>
          <label>
            缩放比例 ({{ scalePct }}%)
            <input v-model.number="scalePct" type="range" min="70" max="150" data-testid="slider-skinswitch-scale" />
          </label>
          <label>
            座位屏幕横坐标 ({{ seatScreenXPct }}% · &lt;50% 攻击自动水平翻转)
            <input v-model.number="seatScreenXPct" type="range" min="5" max="95" data-testid="slider-skinswitch-screenx" />
          </label>
          <div v-if="current.toastMessage" class="skinswitch-toast" data-testid="skinswitch-toast">
            <span class="skinswitch-icon-glyph">✦</span> {{ current.toastMessage }}
          </div>
        </div>

        <div class="preview-stage">
          <div
            class="standee-box"
            data-testid="skinswitch-modal-preview"
            :style="{
              transform: `translate(${standX}px, ${standY}px) scale(${(current.lastChukuangAction?.flipX ? -1 : 1) * (scalePct / 100)}, ${scalePct / 100})`,
            }"
          >
            <span class="action-tag">{{ current.lastChukuangAction?.action ?? "DaiJi" }}</span>
            <span class="flip-tag">flipX={{ Boolean(current.lastChukuangAction?.flipX) }}</span>
          </div>
          <div class="clip-buttons">
            <button type="button" data-testid="btn-skinswitch-modal-gongji" @click="triggerChukuang('_gj')">出框攻击 (_gj)</button>
            <button type="button" data-testid="btn-skinswitch-modal-teshu" @click="triggerChukuang('_ts')">特殊动作 (_ts)</button>
            <button type="button" data-testid="btn-skinswitch-modal-shan" @click="triggerChukuang('_checkDcdShan')">闪避 (_checkDcdShan)</button>
            <button type="button" data-testid="btn-skinswitch-modal-chuchang" @click="triggerChukuang('_checkDcdChuChang')">出场 (_checkDcdChuChang)</button>
            <button type="button" data-testid="btn-skinswitch-modal-group" @click="triggerChukuang('_qhlyChangeGroupSkin')">变阵变皮</button>
            <button type="button" data-testid="btn-skinswitch-modal-audio" @click="triggerChukuang('_playAudioToQueue')">入队元素杀音</button>
          </div>
        </div>
      </div>

      <div v-else-if="activeTab === 'configs'" class="configs-tab">
        <div class="surfaces-strip">
          <button
            v-for="surf in SKIN_SWITCH_SURFACE_IDS"
            :key="surf"
            type="button"
            class="tab-pill"
            :class="{ active: current.activeSurface === surf }"
            :data-testid="`skinswitch-surface-${surf}`"
            @click="selectSurface(surf)"
          >
            {{ surf }}
          </button>
        </div>

        <div class="cfg-grid">
          <div v-for="spec in SKIN_SWITCH_CONFIG_SPECS" :key="spec.key" class="cfg-row" :data-testid="`skinswitch-cfg-row-${spec.key}`">
            <div class="cfg-meta">
              <strong>{{ spec.label }}</strong>
              <code>{{ spec.key }}</code>
            </div>
            <div class="cfg-ctrl">
              <input
                v-if="spec.control === 'boolean'"
                type="checkbox"
                :checked="Boolean(current.config[spec.key])"
                :data-testid="`skinswitch-cfg-input-${spec.key}`"
                @change="setBoolConfig(spec.key, ($event.target as HTMLInputElement).checked)"
              />
              <button
                v-else-if="spec.control === 'action'"
                type="button"
                class="tab-pill"
                :data-testid="`skinswitch-cfg-input-${spec.key}`"
                @click="triggerActionConfig(spec.key as 'previewDynamic' | 'resetArchiveDynamicSkin' | 'genDynamicSkin' | 'genDyTempFile')"
              >
                执行
              </button>
              <span v-else class="tiny muted" :data-testid="`skinswitch-cfg-input-${spec.key}`">1.24 净室更新</span>
            </div>
          </div>
        </div>

        <pre v-if="current.generatedDynamicSnippet" class="snippet-pre" data-testid="skinswitch-generated-snippet">{{ current.generatedDynamicSnippet }}</pre>
        <pre v-if="current.generatedTemplateSnippet" class="snippet-pre" data-testid="skinswitch-generated-template">{{ current.generatedTemplateSnippet }}</pre>
        <div class="json-guard-box">
          <textarea v-model="jsonText" rows="2" placeholder='{"showTopArc":true,"cugDynamicBg":true}' data-testid="skinswitch-cfg-json-textarea"></textarea>
          <div class="json-actions">
            <button type="button" data-testid="btn-skinswitch-export-json" @click="onExportJson">导出 14 配置</button>
            <button type="button" data-testid="btn-skinswitch-import-json" @click="onImportJson">导入校验(裁决38)</button>
            <span v-if="jsonMsg" class="tiny" data-testid="skinswitch-cfg-msg">{{ jsonMsg }}</span>
          </div>
        </div>
      </div>

      <div v-else class="spine-tab">
        <Spine40SkinImportBox
          :active-clip="current.lastChukuangAction?.action ?? 'daiji'"
          :stand-x="standX"
          :stand-y="standY"
          :scale-pct="scalePct"
          @trigger-clip="triggerChukuang('_gj')"
        />
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-backdrop { position: fixed; inset: 0; z-index: 60; background: rgba(8, 10, 14, 0.76); display: flex; align-items: center; justify-content: center; padding: 1rem; }
.modal-card { width: min(860px, 96vw); max-height: 88vh; overflow-y: auto; background: var(--bg-panel, #1b1814); border: 1.5px solid var(--accent, #e8a54b); border-radius: 12px; padding: 1rem 1.2rem; }
.modal-head { display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 0.8rem; gap: 0.5rem; }
.modal-head h3 { margin: 0; font-size: 0.96rem; }
.head-tabs { display: flex; gap: 0.3rem; flex-wrap: wrap; }
.tab-pill { padding: 0.22rem 0.5rem; border-radius: 6px; border: 1px solid rgba(255,255,255,0.2); background: rgba(255,255,255,0.06); color: #e2e8f0; font-size: 0.72rem; cursor: pointer; }
.tab-pill.active { border-color: #f59e0b; background: rgba(245,158,11,0.22); color: #fde68a; }
.vault-body { display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; }
.sliders { display: flex; flex-direction: column; gap: 0.65rem; font-size: 0.84rem; }
.sliders label { display: flex; flex-direction: column; gap: 0.2rem; }
.preview-stage { display: flex; flex-direction: column; align-items: center; justify-content: space-between; padding: 0.8rem; border-radius: 10px; background: var(--bg-elevated); border: 1px dashed var(--border); }
.standee-box { width: 140px; height: 110px; border-radius: 10px; border: 2px solid var(--accent); background: rgba(232, 165, 75, 0.14); display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 0.25rem; transition: transform 0.18s ease; }
.action-tag { font-weight: 700; color: var(--accent); }
.flip-tag { font-family: var(--mono); font-size: 0.74rem; }
.clip-buttons { display: flex; flex-wrap: wrap; gap: 0.35rem; margin-top: 0.7rem; }
.surfaces-strip { display: flex; flex-wrap: wrap; gap: 0.3rem; margin-bottom: 0.6rem; }
.cfg-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(250px, 1fr)); gap: 0.38rem; max-height: 36vh; overflow-y: auto; }
.cfg-row { display: flex; justify-content: space-between; align-items: center; padding: 0.35rem 0.5rem; border-radius: 7px; border: 1px solid rgba(255,255,255,0.1); background: rgba(255,255,255,0.03); }
.cfg-meta { display: flex; flex-direction: column; font-size: 0.72rem; }
.cfg-meta code { color: #fbbf24; font-size: 0.68rem; }
.snippet-pre { margin: 0.5rem 0; padding: 0.45rem; background: #0b0f16; border-radius: 6px; font-size: 0.68rem; max-height: 100px; overflow-y: auto; }
.json-guard-box { margin-top: 0.6rem; display: flex; flex-direction: column; gap: 0.35rem; }
.json-guard-box textarea { width: 100%; background: #0b0f16; color: #f8fafc; border: 1px solid rgba(255,255,255,0.2); border-radius: 6px; padding: 0.35rem; font-size: 0.74rem; }
.json-actions { display: flex; gap: 0.45rem; align-items: center; }
.tiny { font-size: 0.75rem; margin: 0.15rem 0 0; }
</style>
