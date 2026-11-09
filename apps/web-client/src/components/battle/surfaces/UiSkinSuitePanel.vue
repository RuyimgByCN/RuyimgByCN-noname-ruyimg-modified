<script setup lang="ts">
import { computed, ref, watch } from "vue";
import {
  DECADE_MARKSKILL_SPECS,
} from "../../../engine/surfaces/decadeMarkSkillsData";
import {
  DECADE_ALL_UI_HOOK_IDS,
  DECADE_SURFACE_MODULES,
  createInitialDecadeUiRuntimeState,
  dispatchDecadeUiHook,
  type DecadeUiRuntimeState,
} from "../../../engine/surfaces/decadeUiSurface";
import {
  QHLY_GROUP_COLORS,
  QHLY_SURFACE_PRESETS,
  QHLY_UI_HOOK_IDS,
  createInitialQhlyRuntimeState,
  dispatchQhlyUiHook,
  resolveQhlySkinShare,
  type QhlyRuntimeState,
  type QhlyStylesheetSurfaceId,
} from "../../../engine/surfaces/qhlySurface";
import {
  SHOUSHA_SURFACE_MODULES,
  SHOUSHA_UI_HOOK_IDS,
  createInitialShoushaUiRuntimeState,
  dispatchShoushaUiHook,
  type ShoushaUiRuntimeState,
} from "../../../engine/surfaces/shoushaUiSurface";
import {
  SKIN_SWITCH_SURFACE_IDS,
  SKIN_SWITCH_UI_HOOK_IDS,
  createInitialSkinSwitchRuntimeState,
  dispatchSkinSwitchHook,
  type SkinSwitchRuntimeState,
} from "../../../engine/surfaces/skinSwitchSurface";
import {
  TAIXU_DLC_SEASONS,
  TAIXU_SERVANT_SPECS,
  TAIXU_SURFACE_STYLESHEETS,
  createInitialTaixuUiRuntimeState,
  purchaseTaixuMerchantItem,
  selectTaixuServantSurface,
  switchTaixuViewport,
  type TaixuUiRuntimeState,
} from "../../../engine/surfaces/taixuSurface";

const props = defineProps<{
  qhly?: QhlyRuntimeState;
  skinSwitch?: SkinSwitchRuntimeState;
  shoushaUi?: ShoushaUiRuntimeState;
  decadeUi?: DecadeUiRuntimeState;
  taixuUi?: TaixuUiRuntimeState;
}>();

const emit = defineEmits<{
  (
    e: "update-suites",
    payload: {
      qhly: QhlyRuntimeState;
      skinSwitch: SkinSwitchRuntimeState;
      shoushaUi: ShoushaUiRuntimeState;
      decadeUi: DecadeUiRuntimeState;
      taixuUi: TaixuUiRuntimeState;
      notice: string;
    },
  ): void;
}>();

const selectedMarkId = ref<string>("xionghuo");

const localSuites = ref({
  qhly: props.qhly ?? createInitialQhlyRuntimeState(),
  skinSwitch: props.skinSwitch ?? createInitialSkinSwitchRuntimeState(),
  shoushaUi: props.shoushaUi ?? createInitialShoushaUiRuntimeState(),
  decadeUi: props.decadeUi ?? createInitialDecadeUiRuntimeState(),
  taixuUi: props.taixuUi ?? createInitialTaixuUiRuntimeState(),
});

watch(
  () => [props.qhly, props.skinSwitch, props.shoushaUi, props.decadeUi, props.taixuUi] as const,
  ([qhly, skinSwitch, shoushaUi, decadeUi, taixuUi]) => {
    localSuites.value = {
      qhly: qhly ?? createInitialQhlyRuntimeState(),
      skinSwitch: skinSwitch ?? createInitialSkinSwitchRuntimeState(),
      shoushaUi: shoushaUi ?? createInitialShoushaUiRuntimeState(),
      decadeUi: decadeUi ?? createInitialDecadeUiRuntimeState(),
      taixuUi: taixuUi ?? createInitialTaixuUiRuntimeState(),
    };
  },
);

const qhlyState = computed(() => localSuites.value.qhly);
const skinState = computed(() => localSuites.value.skinSwitch);
const shoushaState = computed(() => localSuites.value.shoushaUi);
const decadeState = computed(() => localSuites.value.decadeUi);
const taixuState = computed(() => localSuites.value.taixuUi);

const qhlyPreset = computed(() => QHLY_SURFACE_PRESETS[qhlyState.value.activeStylesheet]);
const qhlySharePreview = computed(() =>
  resolveQhlySkinShare(qhlyState.value.inspectGeneralId, "sbxingshang"),
);

function commitSuites(
  partial: Partial<{
    qhly: QhlyRuntimeState;
    skinSwitch: SkinSwitchRuntimeState;
    shoushaUi: ShoushaUiRuntimeState;
    decadeUi: DecadeUiRuntimeState;
    taixuUi: TaixuUiRuntimeState;
  }>,
  notice: string,
) {
  const merged = {
    qhly: partial.qhly ?? localSuites.value.qhly,
    skinSwitch: partial.skinSwitch ?? localSuites.value.skinSwitch,
    shoushaUi: partial.shoushaUi ?? localSuites.value.shoushaUi,
    decadeUi: partial.decadeUi ?? localSuites.value.decadeUi,
    taixuUi: partial.taixuUi ?? localSuites.value.taixuUi,
  };
  localSuites.value = merged;
  emit("update-suites", {
    ...merged,
    notice,
  });
}

function onCycleQhlyStylesheet() {
  const keys = Object.keys(QHLY_SURFACE_PRESETS) as QhlyStylesheetSurfaceId[];
  const idx = keys.indexOf(qhlyState.value.activeStylesheet);
  const nextSheet = keys[(idx + 1) % keys.length]!;
  const nextQhly = { ...qhlyState.value, activeStylesheet: nextSheet, detailModalOpen: true };
  commitSuites({ qhly: nextQhly }, `千幻聆音 (L09)：已切换套装 ${QHLY_SURFACE_PRESETS[nextSheet].label}`);
}

function onRunQhlyKillAndBgm() {
  let next = dispatchQhlyUiHook(qhlyState.value, {
    hook: "_qhlyChageKillingSkin",
    killerGeneralId: qhlyState.value.inspectGeneralId,
  });
  next = dispatchQhlyUiHook(next, { hook: "_qhly_bgm", generalId: next.inspectGeneralId });
  next = dispatchQhlyUiHook(next, {
    hook: "_qhly_addButton",
    dragOffset: { leftPx: 12, topPx: 8 },
  });
  commitSuites(
    { qhly: next },
    `千幻聆音 (L09)：击杀换肤→${next.replaceSkinByGeneral[next.inspectGeneralId]} · BGM→${next.activeBgmTrack}`,
  );
}

function onRunSkinSwitchChukuang() {
  let next = dispatchSkinSwitchHook(skinState.value, {
    hook: "_gj",
    seat: 0,
    seatScreenXPct: 35,
  });
  next = dispatchSkinSwitchHook(next, {
    hook: "_playAudioToQueue",
    seat: 0,
    generalId: "caocao",
    cardName: "sha",
    cardNature: "fire",
  });
  next = dispatchSkinSwitchHook(next, {
    hook: "_pfqh_check_jisha",
    seat: 0,
  });
  next = {
    ...next,
    activeModal: next.activeModal === "cang-zhen-ge" ? "skeletonEditor" : "cang-zhen-ge",
  };
  commitSuites(
    { skinSwitch: next },
    `皮肤切换 (L24)：出框动作 ${next.lastChukuangAction?.action} (flipX=${next.lastChukuangAction?.flipX}) · 藏珍阁/骨骼编辑器 ${next.activeModal}`,
  );
}

function onRunShoushaPaixuAndPhase() {
  let next = dispatchShoushaUiHook(shoushaState.value, "_jd_cpjd", { seat: 0 });
  next = dispatchShoushaUiHook(next, "_paixu_paixu_paixu", {
    seat: 0,
    handCards: [
      { id: 101, name: "sha", suit: "diamond", number: 9, muniu: false },
      { id: 102, name: "shan", suit: "heart", number: 2, muniu: true },
      { id: 103, name: "sha", suit: "spade", number: 7, muniu: false },
    ],
  });
  next = dispatchShoushaUiHook(next, "_uicardupdate", { seat: 0 });
  commitSuites(
    { shoushaUi: next },
    `手杀 UI (L14)：阶段「${next.phaseRibbonLabel}」· 自动牌序 [${next.sortedHandCardIds.join(",")}] (木牛优先)`,
  );
}

function onAddDecadeMarkBadge() {
  let next = dispatchDecadeUiHook(decadeState.value, selectedMarkId.value, {
    seat: 0,
    count: 2,
    yang: true,
    suits: ["diamond", "spade", "heart"],
  });
  next = dispatchDecadeUiHook(next, "_jiubuff_", { seat: 0 });
  commitSuites(
    { decadeUi: next },
    `十周年 UI (L08)：已挂载标记 ${selectedMarkId.value}（共支持 ${DECADE_ALL_UI_HOOK_IDS.length} 项钩子 / ${DECADE_SURFACE_MODULES.length} 项渲染面）`,
  );
}

function onRunTaixuConsoleStep() {
  const nextWidth = taixuState.value.viewportMode === "pc" ? 720 : 1280;
  let next = switchTaixuViewport(taixuState.value, nextWidth);
  next = selectTaixuServantSurface(next, "jiuwei");
  next = { ...next, hasJuguzhidaoBuff: true };
  next = purchaseTaixuMerchantItem(next, 50);
  commitSuites(
    { taixuUi: next },
    `太虚幻境 (L11)：响应式 ${next.responsive.activeStylesheet} · 侍灵「九尾」(${next.servantFrameClass}) · 商铺余币 ${next.coinBalance}`,
  );
}
</script>

<template>
  <div class="ui-skin-suite-grid" data-testid="ui-skin-suite-panel">
    <!-- L08 十周年UI 完整牌桌标记/酒火/国战隐匿渲染面 -->
    <div
      class="suite-card decade-suite-card"
      :data-layout="decadeState.layoutVariant"
      :data-player-css="decadeState.playerCssVariant"
      data-testid="l08-decade-full-surface"
    >
      <div class="suite-head">
        <strong>十周年 UI (L08 · {{ DECADE_ALL_UI_HOOK_IDS.length }}钩子 / {{ DECADE_SURFACE_MODULES.length }}表面)</strong>
        <span class="suite-pill">{{ decadeState.layoutVariant }} · {{ decadeState.splashMode }}</span>
      </div>
      <div class="decade-mark-bar" data-testid="l08-decade-mark-bar">
        <span
          v-for="badge in decadeState.activeMarkBadgesBySeat[0] ?? []"
          :key="badge.skillId"
          class="decade-mark-chip"
          :title="badge.upstreamRef"
          :data-mark-id="badge.skillId"
        >
          {{ badge.label }}[{{ badge.badgeText }}]
        </span>
        <span v-if="decadeState.jiuBuffSeats.includes(0)" class="decade-jiu-badge">🍶酒态</span>
      </div>
      <div class="suite-actions">
        <select v-model="selectedMarkId" aria-label="十周年标记选择" data-testid="select-l08-markskill">
          <option v-for="spec in DECADE_MARKSKILL_SPECS.slice(0, 24)" :key="spec.id" :value="spec.id">
            {{ spec.label }} ({{ spec.id }})
          </option>
        </select>
        <button type="button" data-testid="btn-l08-apply-mark" @click="onAddDecadeMarkBadge">
          挂载十周年标记角标
        </button>
      </div>
    </div>

    <!-- L09 千幻聆音 12 套主题资料卡 / 语音共享 / 换肤浮动钮渲染面 -->
    <div
      class="suite-card qhly-suite-card"
      :style="{
        background: qhlyPreset.bgGradient,
        borderColor: qhlyPreset.borderColor,
      }"
      data-testid="l09-qhly-full-surface"
    >
      <div class="suite-head">
        <strong :style="{ color: qhlyPreset.accentColor }">
          千幻聆音 (L09 · {{ QHLY_UI_HOOK_IDS.length }}钩子 / 12套样式)
        </strong>
        <span
          class="qhly-yh-arc"
          :style="{ backgroundColor: QHLY_GROUP_COLORS[qhlyState.yuanhuSyncedSeats[0] ?? 'wei'] }"
        >
          圆弧:{{ qhlyState.yuanhuSyncedSeats[0] ?? "wei" }}
        </span>
      </div>
      <p class="tiny">
        套装：<code>{{ qhlyPreset.stylesheet }}</code> ·
        共享映射：<code>{{ qhlyState.inspectGeneralId }} → {{ qhlySharePreview.resolvedGeneral }}</code>
        (台词 <code>{{ qhlySharePreview.resolvedSkill }}</code>) ·
        BGM：<code>{{ qhlyState.activeBgmTrack ?? "待触发" }}</code>
      </p>
      <div class="suite-actions">
        <button type="button" data-testid="btn-l09-cycle-sheet" @click="onCycleQhlyStylesheet">
          切换千幻套装
        </button>
        <button type="button" data-testid="btn-l09-kill-bgm" @click="onRunQhlyKillAndBgm">
          触发击杀换肤+角色BGM
        </button>
      </div>
    </div>

    <!-- L14 手杀UI 53 钩子 / 阶段横幅 / 木牛自动牌序 / 进度条渲染面 -->
    <div class="suite-card shousha-suite-card" data-testid="l14-shousha-full-surface">
      <div class="suite-head">
        <strong>手杀 UI (L14 · {{ SHOUSHA_UI_HOOK_IDS.length }}钩子 / {{ SHOUSHA_SURFACE_MODULES.length }}表面)</strong>
        <span class="suite-pill">{{ shoushaState.phaseRibbonLabel }} (控件#{{ shoushaState.skillControlRev }})</span>
      </div>
      <p class="tiny">
        提示：<code>{{ shoushaState.playPromptText ?? "无" }}</code> ·
        自动牌序：<code>[{{ shoushaState.sortedHandCardIds.join(",") || "未排" }}]</code> ·
        露头(ZLLT)：<code>{{ shoushaState.zlltOutcropEnabled ? "开" : "关" }}</code>
      </p>
      <div class="suite-actions">
        <button type="button" data-testid="btn-l14-paixu" @click="onRunShoushaPaixuAndPhase">
          执行手杀木牛自动牌序
        </button>
      </div>
    </div>

    <!-- L24 皮肤切换 22 钩子 / 藏珍阁 / 骨骼参数编辑 / 出框语音队列渲染面 -->
    <div class="suite-card skinswitch-suite-card" data-testid="l24-skinswitch-full-surface">
      <div class="suite-head">
        <strong>皮肤切换 (L24 · {{ SKIN_SWITCH_UI_HOOK_IDS.length }}钩子 / {{ SKIN_SWITCH_SURFACE_IDS.length }}表面)</strong>
        <span class="suite-pill">弹窗:{{ skinState.activeModal }} · 变身:{{ skinState.specialFormBySeat[0] ?? "base" }}</span>
      </div>
      <p class="tiny">
        出框：<code>{{ skinState.lastChukuangAction ? `${skinState.lastChukuangAction.action}(flipX=${skinState.lastChukuangAction.flipX})` : "待机" }}</code> ·
        语音队列：<code>{{ skinState.audioPlayQueue.map((a) => a.card).join(",") || "空" }}</code>
      </p>
      <div class="suite-actions">
        <button type="button" data-testid="btn-l24-chukuang" @click="onRunSkinSwitchChukuang">
          触发攻击出框+火杀语音入队
        </button>
      </div>
    </div>

    <!-- L11 太虚幻境 4 套响应式样式 (PcStyle/MobileStyle/servant/style) / 8 DLC 赛季 / 21 侍灵台 -->
    <div class="suite-card taixu-suite-card" data-testid="l11-taixu-full-surface">
      <div class="suite-head">
        <strong>太虚幻境 (L11 · {{ TAIXU_SURFACE_STYLESHEETS.length }}样式 / {{ TAIXU_DLC_SEASONS.length }}赛季 / {{ TAIXU_SERVANT_SPECS.length }}侍灵)</strong>
        <span class="suite-pill">{{ taixuState.responsive.activeStylesheet }} ({{ taixuState.responsive.unit }})</span>
      </div>
      <p class="tiny">
        赛季：<code>{{ taixuState.activeSeasonId }}</code> ·
        侍灵：<code>{{ taixuState.selectedServantId }}</code> (<code>{{ taixuState.servantFrameClass }}</code>) ·
        金币：<code>{{ taixuState.coinBalance }}</code> (巨贾冻结:{{ taixuState.hasJuguzhidaoBuff ? "是" : "否" }})
      </p>
      <div class="suite-actions">
        <button type="button" data-testid="btn-l11-taixu-console" @click="onRunTaixuConsoleStep">
          切换太虚 PC/触屏样式+九尾商铺
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.ui-skin-suite-grid {
  margin-top: 0.55rem;
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 0.5rem;
}
.suite-card {
  padding: 0.45rem 0.6rem;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--bg-panel);
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
}
.suite-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 0.3rem;
  font-size: 0.78rem;
}
.suite-pill,
.qhly-yh-arc {
  font-size: 0.7rem;
  padding: 0.08rem 0.4rem;
  border-radius: 999px;
  border: 1px solid var(--border);
  color: #f8fafc;
  font-family: var(--mono);
}
.decade-mark-bar {
  display: flex;
  flex-wrap: wrap;
  gap: 0.25rem;
}
.decade-mark-chip {
  font-size: 0.72rem;
  padding: 0.08rem 0.38rem;
  border-radius: 4px;
  background: linear-gradient(180deg, #3b2712, #231508);
  border: 1px solid #d97706;
  color: #fde68a;
  font-family: var(--mono);
}
.decade-jiu-badge {
  font-size: 0.72rem;
  padding: 0.08rem 0.38rem;
  border-radius: 4px;
  background: #7f1d1d;
  color: #fecaca;
}
.suite-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 0.35rem;
  align-items: center;
}
</style>
