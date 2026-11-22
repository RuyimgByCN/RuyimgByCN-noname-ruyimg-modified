<script setup lang="ts">
import { computed, ref, watch } from "vue";
import {
  QHLY_GROUP_COLORS,
  QHLY_SURFACE_PRESETS,
  canTriggerQhlyVoiceWithNotbb,
  createInitialQhlyRuntimeState,
  dispatchQhlyUiHook,
  highlightSkillKeywords,
  resolveQhlyCharacterSubtitle,
  resolveQhlySkinAssetView,
  resolveQhlySkinShare,
  resolveQhlyVoiceRoute,
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
const detailSubPage = ref<"introduce" | "skill" | "skin" | "config">(
  current.value.config.qhly_listdefaultpage,
);
watch(
  () => current.value.config.qhly_listdefaultpage,
  (next) => {
    detailSubPage.value = next;
  },
);
const quoteDraft = ref(
  current.value.editedLinesByGeneral?.[generalInput.value] ?? "建安风骨，文武兼资。",
);
const resolvedAssetInfo = computed(() =>
  resolveQhlySkinAssetView(
    cfg.value,
    generalInput.value,
    current.value.replaceSkinByGeneral[generalInput.value] ?? "origin",
    { isGuozhan: cfg.value.qhly_guozhan },
  ),
);
const voiceRouteInfo = computed(() => resolveQhlyVoiceRoute(cfg.value, cfg.value.qhly_mvp ? "mvp" : "skill"));
const winRecord = computed(
  () => current.value.winRecordByGeneral?.[generalInput.value] ?? { wins: 5, games: 10 },
);
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

function captureCleanroomDomSnapshot() {
  if (typeof document === "undefined") return;
  const c = document.createElement("canvas");
  c.width = 160;
  c.height = 60;
  const ctx = c.getContext("2d");
  if (ctx) {
    ctx.fillStyle = "#141922";
    ctx.fillRect(0, 0, 160, 60);
    ctx.fillStyle = "#fbbf24";
    ctx.font = "12px sans-serif";
    ctx.fillText(`${generalInput.value}静皮快照`, 10, 34);
  }
  const url = c.toDataURL("image/png");
  emit("update", {
    ...current.value,
    domSnapshotInfo: url,
    lastOperationNotice: `dom2image 已生成静皮快照 (${url.length} B)`,
  });
}

function saveEditedLine() {
  emit("update", {
    ...current.value,
    editedLinesByGeneral: {
      ...current.value.editedLinesByGeneral,
      [generalInput.value]: quoteDraft.value,
    },
    lastOperationNotice: `已保存 ${generalInput.value} 自定义台词`,
  });
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
      :data-qhly-forbid-ext="cfg.qhly_forbidExtPage ? 'on' : 'off'"
      :data-qhly-lihui="cfg.qhly_lihuiSupport ? 'on' : 'off'"
      :data-qhly-preload="cfg.qhly_funcLoadInPrecontent ? 'on' : 'off'"
      :data-qhly-replace-card="cfg.qhly_replaceCharacterCard2"
      :style="{
        '--qhly-shuimo-lh': cfg.qhly_hanggaoxiufu,
        '--qhly-shuimo-lh2': cfg.qhly_hanggaoxiufu2,
        '--qhly-lol-lh': cfg.qhly_lolhanggaoxiufu,
        '--qhly-lol-lh2': cfg.qhly_lolhanggaoxiufu2,
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
            :data-qhly-lutou-fix="cfg.lutouFixx"
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

          <div class="qhly-subpages-row" data-testid="qhly-detail-subpages">
            <button type="button" class="mode-tab-btn" :class="{ active: detailSubPage === 'introduce' }" data-testid="btn-qhly-subpage-introduce" @click="detailSubPage = 'introduce'">简介</button>
            <button type="button" class="mode-tab-btn" :class="{ active: detailSubPage === 'skill' }" data-testid="btn-qhly-subpage-skill" @click="detailSubPage = 'skill'">技能</button>
            <button type="button" class="mode-tab-btn qhly-ext-only-tab" :class="{ active: detailSubPage === 'skin' }" data-testid="btn-qhly-subpage-skin" @click="detailSubPage = 'skin'">皮肤</button>
            <button type="button" class="mode-tab-btn qhly-ext-only-tab" :class="{ active: detailSubPage === 'config' }" data-testid="btn-qhly-subpage-config" @click="detailSubPage = 'config'">配置</button>
            <button type="button" class="mode-tab-btn" data-testid="btn-qhly-double-page" @dblclick="detailSubPage = cfg.qhly_doubledefaultpage">双击页:{{ cfg.qhly_doubledefaultpage }}</button>
          </div>

          <div class="share-box" :data-qhly-subpage="detailSubPage" data-testid="qhly-subpage-body">
            <div v-if="detailSubPage === 'introduce'" class="tiny">
              <strong>[简介页]</strong> 武将 <code>{{ generalInput }}</code> · {{ subtitleText }} · 自定义台词「{{ quoteDraft }}」
            </div>
            <div v-else-if="detailSubPage === 'skill'" class="tiny">
              <strong>[技能页]</strong> 关联技能 <code>{{ skillInput }}</code> · {{ shareInfo.shared ? "已绑定跨将共享台词" : "原技能独立台词" }} · 音轨 {{ voiceRouteInfo.trackLabel }}
            </div>
            <div v-else-if="detailSubPage === 'skin'" class="tiny">
              <strong>[皮肤页]</strong> 当前 {{ current.replaceSkinByGeneral[generalInput] ?? "origin" }} · {{ resolvedAssetInfo.fallbackPolicyLabel }} · 动皮槽={{ resolvedAssetInfo.dynamicSlotCount }}
            </div>
            <div v-else class="tiny">
              <strong>[配置页]</strong> 行高={{ cfg.qhly_hanggaoxiufu }}/{{ cfg.qhly_hanggaoxiufu2 }} · 阵营字号={{ cfg.qhly_shilizihao }} · 拖拽按钮={{ cfg.qhly_dragButton ? "开" : "关" }}
            </div>
          </div>

          <div class="share-box qhly-bio-secondary" data-testid="qhly-asset-resolver-box">
            <div class="tiny">
              资产根目录：<code>{{ resolvedAssetInfo.assetRoot }}</code> · {{ resolvedAssetInfo.fallbackPolicyLabel }}
            </div>
            <div class="tiny">
              动皮槽位={{ resolvedAssetInfo.dynamicSlotCount }} (允许={{ resolvedAssetInfo.dynamicPermitted ? "开" : "关" }}) · 等阶={{ resolvedAssetInfo.rarityMode }} · 轮换={{ resolvedAssetInfo.autoIntervalSec }}s
            </div>
            <div class="tiny muted">{{ resolvedAssetInfo.effectsSummary }}</div>
            <div class="tiny">音轨路由：<code>{{ voiceRouteInfo.trackLabel }}</code></div>
            <div v-if="cfg.qhly_recordWin" class="tiny" data-testid="qhly-record-win-box">
              战绩统计：{{ winRecord.wins }} 胜 / {{ winRecord.games }} 场 ({{ Math.round((winRecord.wins / Math.max(1, winRecord.games)) * 100) }}%)
            </div>
            <div v-if="cfg.qhly_skinconfig" class="tiny" data-testid="qhly-skinconfig-box">
              高级配置模式：已开启皮肤层级参数锁定与独立槽位映射
            </div>
            <div v-if="cfg.qhly_editmode" class="editline-row" data-testid="qhly-editmode-box">
              <input v-model="quoteDraft" type="text" data-testid="qhly-editmode-input" />
              <button type="button" data-testid="btn-qhly-save-line" @click="saveEditedLine">保存台词</button>
            </div>
          </div>

          <div class="faction-swatches">
            <span
              v-for="(hex, fac) in QHLY_GROUP_COLORS"
              :key="fac"
              class="swatch"
              :style="{
                borderColor: hex,
                color: hex,
                fontSize: `${Number(cfg.qhly_currentViewSkin === 'lolbig' ? cfg.qhly_lolshilizihao : cfg.qhly_shilizihao) * 0.18}px`,
              }"
            >{{ fac }} {{ hex }}</span>
          </div>

          <div class="actions-row">
            <button type="button" class="primary" data-testid="btn-qhly-modal-apply-kill" @click="triggerKillSkinAndBgm">
              触发击杀换肤与专属 BGM
            </button>
            <button
              v-if="cfg.qhly_dom2image"
              type="button"
              class="mode-tab-btn"
              data-testid="btn-qhly-dom2image"
              @click="captureCleanroomDomSnapshot"
            >
              生成静皮快照
            </button>
            <span v-if="current.domSnapshotInfo" class="tiny" data-testid="qhly-dom2image-status">✓ 已生成静皮</span>
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
