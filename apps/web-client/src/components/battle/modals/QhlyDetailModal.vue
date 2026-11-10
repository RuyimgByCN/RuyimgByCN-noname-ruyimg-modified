<script setup lang="ts">
import { computed, ref } from "vue";
import {
  QHLY_GROUP_COLORS,
  QHLY_SURFACE_PRESETS,
  createInitialQhlyRuntimeState,
  dispatchQhlyUiHook,
  resolveQhlySkinShare,
  type QhlyRuntimeState,
  type QhlyStylesheetSurfaceId,
} from "../../../engine/surfaces/qhlySurface";

const props = defineProps<{
  state?: QhlyRuntimeState;
}>();

const emit = defineEmits<{
  close: [];
  update: [next: QhlyRuntimeState];
}>();

const generalInput = ref(props.state?.inspectGeneralId ?? "sb_caopi");
const skillInput = ref("sbxingshang");

const current = computed(() => props.state ?? createInitialQhlyRuntimeState());
const presetEntries = computed(
  () => Object.entries(QHLY_SURFACE_PRESETS) as Array<[QhlyStylesheetSurfaceId, (typeof QHLY_SURFACE_PRESETS)[QhlyStylesheetSurfaceId]]>,
);
const shareInfo = computed(() => resolveQhlySkinShare(generalInput.value, skillInput.value));

function selectStylesheet(sheetId: QhlyStylesheetSurfaceId) {
  emit("update", {
    ...current.value,
    inspectGeneralId: generalInput.value,
    activeStylesheet: sheetId,
    detailModalOpen: true,
  });
}

function triggerKillSkinAndBgm() {
  let next = dispatchQhlyUiHook(current.value, {
    hook: "_qhlyChageKillingSkin",
    killerGeneralId: generalInput.value,
  });
  next = dispatchQhlyUiHook(next, {
    hook: "_qhly_bgm",
    generalId: generalInput.value,
  });
  emit("update", { ...next, inspectGeneralId: generalInput.value });
}
</script>

<template>
  <div class="modal-backdrop" role="dialog" aria-modal="true" aria-label="千幻聆音武将详情与皮肤窗口" data-testid="modal-qhly-detail">
    <div class="modal-card">
      <header class="modal-head">
        <div>
          <h3>千幻聆音 · 武将详情与皮肤台 (L09)</h3>
          <p class="muted tiny">上游基准：千幻聆音/extension.js · skinShare.js (12 套样式表)</p>
        </div>
        <button type="button" data-testid="btn-close-qhly-modal" @click="emit('close')">关闭 ✕</button>
      </header>

      <div class="qhly-grid">
        <div class="qhly-presets" role="listbox" aria-label="千幻主题套装">
          <button
            v-for="[id, spec] in presetEntries"
            :key="id"
            type="button"
            class="preset-chip"
            :class="{ active: current.activeStylesheet === id }"
            :data-testid="`qhly-sheet-${id}`"
            @click="selectStylesheet(id)"
          >
            <strong>{{ spec.label }}</strong>
            <span class="tiny muted">{{ spec.stylesheet }}</span>
          </button>
        </div>

        <div class="qhly-inspector">
          <div class="field-row">
            <label>
              武将 ID
              <input v-model="generalInput" type="text" data-testid="qhly-modal-general-input" />
            </label>
            <label>
              技能语音映射
              <input v-model="skillInput" type="text" data-testid="qhly-modal-skill-input" />
            </label>
          </div>

          <div class="share-box" data-testid="qhly-modal-share-result">
            <div>皮肤共享决议：<code>{{ shareInfo.resolvedGeneral }}</code> / <code>{{ shareInfo.resolvedSkill }}</code> ({{ shareInfo.shared ? "已共享" : "独立原皮" }})</div>
            <div>当前激活皮肤：<strong>{{ current.replaceSkinByGeneral[generalInput] ?? "origin" }}</strong></div>
            <div>专属 BGM：<code>{{ current.activeBgmTrack ?? "未播放" }}</code></div>
          </div>

          <div class="faction-swatches">
            <span
              v-for="(hex, fac) in QHLY_GROUP_COLORS"
              :key="fac"
              class="swatch"
              :style="{ borderColor: hex, color: hex }"
            >{{ fac }} {{ hex }}</span>
          </div>

          <div class="actions-row">
            <button type="button" class="primary" data-testid="btn-qhly-modal-apply-kill" @click="triggerKillSkinAndBgm">
              触发击杀换肤与专属 BGM
            </button>
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
  background: rgba(8, 10, 14, 0.76);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 1rem;
}
.modal-card {
  width: min(860px, 96vw);
  max-height: 88vh;
  overflow-y: auto;
  background: var(--bg-panel, #1b1814);
  border: 1px solid var(--accent, #e8a54b);
  border-radius: 12px;
  padding: 1rem 1.2rem;
}
.modal-head {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 0.8rem;
}
.modal-head h3 { margin: 0; }
.qhly-grid {
  display: grid;
  grid-template-columns: 1fr 1.2fr;
  gap: 0.9rem;
}
.qhly-presets {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.4rem;
}
.preset-chip {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  padding: 0.4rem 0.55rem;
  border-radius: 8px;
  border: 1px solid var(--border);
  background: var(--bg-elevated);
  text-align: left;
  cursor: pointer;
}
.preset-chip.active {
  border-color: var(--accent);
  box-shadow: 0 0 0 1px var(--accent);
}
.field-row {
  display: flex;
  gap: 0.6rem;
  margin-bottom: 0.6rem;
}
.field-row label {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  font-size: 0.82rem;
  flex: 1;
}
.field-row input {
  padding: 0.3rem 0.5rem;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--bg);
  color: var(--text);
}
.share-box {
  padding: 0.6rem 0.75rem;
  border-radius: 8px;
  background: var(--bg-elevated);
  font-size: 0.84rem;
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}
.faction-swatches {
  display: flex;
  flex-wrap: wrap;
  gap: 0.35rem;
  margin: 0.6rem 0;
}
.swatch {
  padding: 0.1rem 0.4rem;
  border: 1px solid;
  border-radius: 999px;
  font-size: 0.72rem;
  font-family: var(--mono);
}
.tiny { font-size: 0.75rem; margin: 0.15rem 0 0; }
</style>
