<script setup lang="ts">
import { computed, ref } from "vue";
import {
  QHLY_GROUP_COLORS,
  QHLY_SURFACE_PRESETS,
  canTriggerQhlyVoiceWithNotbb,
  createInitialQhlyRuntimeState,
  dispatchQhlyUiHook,
  highlightSkillKeywords,
  resolveQhlyCharacterSubtitle,
  resolveQhlySkinShare,
  type QhlyRuntimeState,
  type QhlyStylesheetSurfaceId,
} from "../../../engine/surfaces/qhlySurface";
import QhlyConfigPanel from "../surfaces/QhlyConfigPanel.vue";

const props = defineProps<{
  state?: QhlyRuntimeState;
}>();

const emit = defineEmits<{
  close: [];
  update: [next: QhlyRuntimeState];
}>();

const generalInput = ref(props.state?.inspectGeneralId ?? "sb_caopi");
const skillInput = ref("sbxingshang");
const activeViewTab = ref<"inspect" | "configs">("inspect");
const voiceGuardMsg = ref("");

const current = computed(() => props.state ?? createInitialQhlyRuntimeState());
const cfg = computed(() => current.value.config);
const presetEntries = computed(
  () => Object.entries(QHLY_SURFACE_PRESETS) as Array<[QhlyStylesheetSurfaceId, (typeof QHLY_SURFACE_PRESETS)[QhlyStylesheetSurfaceId]]>,
);
const shareInfo = computed(() => resolveQhlySkinShare(generalInput.value, skillInput.value));
const subtitleText = computed(() =>
  resolveQhlyCharacterSubtitle(cfg.value, {
    title: "魏文帝",
    skinName: current.value.replaceSkinByGeneral[generalInput.value] ?? "origin",
    pkgName: "谋攻篇",
  }),
);
const highlightedSkillSegs = computed(() =>
  highlightSkillKeywords(
    "锁定技/限定技 · 出牌阶段开始时，若目标受伤，共享台词并触发专属语音。",
    cfg.value.qhly_keymarkopen,
  ),
);

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
  const res = canTriggerQhlyVoiceWithNotbb(next, skillInput.value, generalInput.value, Date.now());
  voiceGuardMsg.value = res.allowed ? "台词触发成功" : "防啰嗦(qhly_notbb)或静音冷却拦截";
  emit("update", { ...res.nextState, inspectGeneralId: generalInput.value });
}
</script>

<template>
  <div class="modal-backdrop" role="dialog" aria-modal="true" aria-label="千幻聆音武将详情与皮肤窗口" data-testid="modal-qhly-detail">
    <div
      class="modal-card qhly-modal-card"
      :data-qhly-sheet="current.activeStylesheet"
      :data-qhly-fontsize="cfg.qhly_fontsize1"
      :data-qhly-fit-x="cfg.qhly_layoutFitX ? 'on' : 'off'"
      :data-qhly-fit-y="cfg.qhly_layoutFitY ? 'on' : 'off'"
      :data-qhly-v-middle="cfg.qhly_vMiddle ? 'on' : 'off'"
      :data-qhly-shuimo-cover="cfg.qhly_hideShuimoCover ? 'hidden' : 'visible'"
      :style="{
        '--qhly-shuimo-lh': cfg.qhly_hanggaoxiufu,
        '--qhly-lol-lh': cfg.qhly_lolhanggaoxiufu,
      }"
      data-testid="qhly-modal-card"
    >
      <header class="modal-head">
        <div>
          <h3>千幻聆音 · 武将详情与皮肤台 (L09 · 82配置 / 12样式)</h3>
          <p class="muted tiny">
            副标题：<code>{{ subtitleText }}</code> · 小窗：{{ cfg.qhly_smallwindowstyle }}({{ cfg.qhly_dragonsize }}×)
          </p>
        </div>
        <div class="head-tabs">
          <button
            type="button"
            class="mode-tab-btn"
            :class="{ active: activeViewTab === 'inspect' }"
            data-testid="btn-qhly-tab-inspect"
            @click="activeViewTab = 'inspect'"
          >
            套装与资料 (12)
          </button>
          <button
            type="button"
            class="mode-tab-btn"
            :class="{ active: activeViewTab === 'configs' }"
            data-testid="btn-qhly-tab-configs"
            @click="activeViewTab = 'configs'"
          >
            全部 82 项配置
          </button>
          <button type="button" data-testid="btn-close-qhly-modal" @click="emit('close')">关闭 ✕</button>
        </div>
      </header>

      <div v-if="activeViewTab === 'inspect'" class="qhly-grid">
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

          <div
            class="share-box qhly-detail-bio qhly-preview-box"
            :data-qhly-lutou="cfg.qhly_lutou ? 'on' : 'off'"
            :data-qhly-lvjing="cfg.lvjingFixx ? 'on' : 'off'"
            data-testid="qhly-modal-share-result"
          >
            <div class="qhly-ink-veil">墨迹装饰层 (hideShuimoCover={{ cfg.qhly_hideShuimoCover ? "隐藏" : "显示" }})</div>
            <div>皮肤共享决议：<code>{{ shareInfo.resolvedGeneral }}</code> / <code>{{ shareInfo.resolvedSkill }}</code> ({{ shareInfo.shared ? "已共享" : "独立原皮" }})</div>
            <div>当前激活皮肤：<strong>{{ current.replaceSkinByGeneral[generalInput] ?? "origin" }}</strong></div>
            <div>专属 BGM：<code>{{ current.activeBgmTrack ?? "未播放" }}</code></div>
            <div class="skill-preview-line" data-testid="qhly-skill-highlight-line">
              <span
                v-for="(seg, idx) in highlightedSkillSegs"
                :key="idx"
                :style="seg.color ? { color: seg.color, fontWeight: 700 } : undefined"
              >{{ seg.text }}</span>
            </div>
            <div v-if="voiceGuardMsg" class="tiny muted" data-testid="qhly-voice-guard-msg">{{ voiceGuardMsg }}</div>
          </div>

          <div class="faction-swatches">
            <span
              v-for="(hex, fac) in QHLY_GROUP_COLORS"
              :key="fac"
              class="swatch"
              :style="{ borderColor: hex, color: hex, fontSize: `${Number(cfg.qhly_shilizihao) * 0.18}px` }"
            >{{ fac }} {{ hex }}</span>
          </div>

          <div class="actions-row">
            <button type="button" class="primary" data-testid="btn-qhly-modal-apply-kill" @click="triggerKillSkinAndBgm">
              触发击杀换肤与专属 BGM
            </button>
          </div>
        </div>
      </div>

      <QhlyConfigPanel
        v-else
        :state="current"
        @update="(next) => emit('update', next)"
      />
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
.qhly-ink-veil {
  font-size: 0.68rem;
  color: #a1a1aa;
  border-bottom: 1px dashed rgba(255, 255, 255, 0.16);
  padding-bottom: 0.2rem;
}
.head-tabs {
  display: flex;
  gap: 0.35rem;
  align-items: center;
}
.mode-tab-btn {
  padding: 0.25rem 0.55rem;
  border-radius: 6px;
  border: 1px solid rgba(255, 255, 255, 0.2);
  background: rgba(255, 255, 255, 0.06);
  color: #e2e8f0;
  font-size: 0.76rem;
  cursor: pointer;
}
.mode-tab-btn.active {
  border-color: #f59e0b;
  background: rgba(245, 158, 11, 0.2);
  color: #fde68a;
}
.tiny { font-size: 0.75rem; margin: 0.15rem 0 0; }
</style>
