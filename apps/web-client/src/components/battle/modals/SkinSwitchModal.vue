<script setup lang="ts">
import { computed, ref } from "vue";
import {
  SKIN_SWITCH_SURFACE_IDS,
  createInitialSkinSwitchRuntimeState,
  dispatchSkinSwitchHook,
  type SkinSwitchRuntimeState,
} from "../../../engine/surfaces/skinSwitchSurface";

const props = defineProps<{
  state?: SkinSwitchRuntimeState;
}>();

const emit = defineEmits<{
  close: [];
  update: [next: SkinSwitchRuntimeState];
}>();

const current = computed(() => props.state ?? createInitialSkinSwitchRuntimeState());
const standX = ref(12);
const standY = ref(-8);
const scalePct = ref(108);
const seatScreenXPct = ref(28);

function triggerChukuang(hook: "_gj" | "_ts" | "_checkDcdShan" | "_checkDcdChuChang") {
  let next = dispatchSkinSwitchHook(current.value, {
    hook,
    seat: 0,
    generalId: "caocao",
    cardName: hook === "_checkDcdShan" ? "shan" : "sha",
    cardNature: "fire",
    seatScreenXPct: seatScreenXPct.value,
    doubleAvatar: false,
  });
  next = {
    ...next,
    activeModal: "cang-zhen-ge",
  };
  emit("update", next);
}
</script>

<template>
  <div class="modal-backdrop" role="dialog" aria-modal="true" aria-label="皮肤切换藏珍阁与坐标编辑器" data-testid="modal-skinswitch-vault">
    <div class="modal-card">
      <header class="modal-head">
        <div>
          <h3>皮肤切换 · 藏珍阁与出框参数编辑器 (L24)</h3>
          <p class="muted tiny">上游基准：皮肤切换/extension.js · dynamicSkin.js · saveSkinParams ({{ SKIN_SWITCH_SURFACE_IDS.length }} 表面)</p>
        </div>
        <button type="button" data-testid="btn-close-skinswitch-modal" @click="emit('close')">关闭 ✕</button>
      </header>

      <div class="vault-body">
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
  width: min(820px, 96vw);
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
.vault-body {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1rem;
}
.sliders {
  display: flex;
  flex-direction: column;
  gap: 0.65rem;
  font-size: 0.84rem;
}
.sliders label {
  display: flex;
  flex-direction: column;
  gap: 0.2rem;
}
.preview-stage {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: space-between;
  padding: 0.8rem;
  border-radius: 10px;
  background: var(--bg-elevated);
  border: 1px dashed var(--border);
}
.standee-box {
  width: 140px;
  height: 110px;
  border-radius: 10px;
  border: 2px solid var(--accent);
  background: rgba(232, 165, 75, 0.14);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 0.25rem;
  transition: transform 0.18s ease;
}
.action-tag { font-weight: 700; color: var(--accent); }
.flip-tag { font-family: var(--mono); font-size: 0.74rem; }
.clip-buttons {
  display: flex;
  flex-wrap: wrap;
  gap: 0.35rem;
  margin-top: 0.7rem;
}
.tiny { font-size: 0.75rem; margin: 0.15rem 0 0; }
</style>
