<script setup lang="ts">
import { computed, ref, watch } from "vue";
import {
  L20_HUD_MARK_SPECS,
  U01_DDD_GALLERY_GENERALS,
  advanceLongxueDaoxin,
  advanceTaixuNode,
  applyMartialReplacement,
  applyZuanDiyDraft,
  bindTaixuPet,
  buyCoinShopItem,
  buyTaixuBlessing,
  classifyDesktopPointer,
  hiddenCardIdsFromSnapshot,
  incrementL20HudMark,
  normalizeBossHp,
  numToRoma,
  placeCoinBet,
  resolveCoinHonorTitle,
  rotateU01GalleryStandee,
  runOmnisearchQuery,
  selectL27DualGenerals,
  triggerShoushaHudBanner,
  triggerSpineSkinClip,
  unlockLongxueSkills,
  validateSafeExportFilename,
  type ExtensionSurfacesSnapshot,
} from "../../engine/extensionSurfaceRuntime";
import type { ObserverSnapshot } from "../../types/engine";

const props = defineProps<{
  surfaces: ExtensionSurfacesSnapshot;
  view: ObserverSnapshot | null;
  selfId: number;
}>();

const emit = defineEmits<{
  (e: "update:surfaces", next: ExtensionSurfacesSnapshot): void;
}>();

let latestSurfaces: ExtensionSurfacesSnapshot = props.surfaces;
watch(
  () => props.surfaces,
  (v) => {
    latestSurfaces = v;
  },
  { immediate: true },
);

type SurfaceTab = "hud" | "campaign" | "coin" | "omnisearch" | "workbench";
const activeTab = ref<SurfaceTab>("hud");
const searchInput = ref("cangchizhibi");
const exportNameInput = ref("match_seed42.npack.save");
const exportValidationMsg = ref<string>("合法文件名");
const surfaceNotice = ref<string>("");
const spineCanvasRef = ref<HTMLCanvasElement | null>(null);

const mvpSortedEntries = computed(() => {
  const entries = Object.values(props.surfaces.mvp.scores);
  return entries.sort((a, b) => b.total - a.total || a.seat - b.seat);
});

const currentBossStage = computed(() => {
  const stage =
    props.surfaces.boss.stages[props.surfaces.boss.stageIndex] ??
    props.surfaces.boss.stages[0];
  if (!stage) return null;
  return {
    ...stage,
    normalizedHp: normalizeBossHp(stage.hpRaw, props.surfaces.boss.mode === "boss"),
  };
});

const currentTaixuNode = computed(
  () =>
    props.surfaces.taixu.nodes[props.surfaces.taixu.currentNodeIndex] ??
    props.surfaces.taixu.nodes[0],
);

function update(producer: (prev: ExtensionSurfacesSnapshot) => ExtensionSurfacesSnapshot) {
  try {
    const next = producer(latestSurfaces);
    latestSurfaces = next;
    emit("update:surfaces", next);
  } catch (err) {
    surfaceNotice.value = err instanceof Error ? err.message : String(err);
  }
}

function onBuyCoinEffect(item: "fireworks" | "snow" | "nebula" | "twinkle") {
  update((prev) => {
    const coin = buyCoinShopItem(prev.coin, item);
    surfaceNotice.value = `金币商店：已激活 ${item}（余额 ${coin.balance}）`;
    return { ...prev, coin };
  });
}

function onPlaceBet(amount: number) {
  update((prev) => {
    const coin = placeCoinBet(prev.coin, amount);
    surfaceNotice.value = `金币下注：+${amount}（已下注 ${coin.betPlaced}，余额 ${coin.balance}）`;
    return { ...prev, coin };
  });
}

function onAdvanceBoss() {
  update((prev) => {
    const nextIdx = (prev.boss.stageIndex + 1) % Math.max(1, prev.boss.stages.length);
    const boss = { ...prev.boss, stageIndex: nextIdx };
    const stage = boss.stages[nextIdx];
    surfaceNotice.value = `剑阁/BOSS 闯关：推进至 ${stage?.title ?? nextIdx}`;
    return { ...prev, boss };
  });
}

function onCycleBossMode() {
  const modes: Array<"boss" | "versus_jiange"> = ["boss", "versus_jiange"];
  update((prev) => {
    const idx = modes.indexOf(prev.boss.mode);
    const mode = modes[(idx + 1) % modes.length]!;
    surfaceNotice.value = `单机战役模式：${mode}`;
    return { ...prev, boss: { ...prev.boss, mode } };
  });
}

function onAdvanceTaixu() {
  update((prev) => {
    const taixu = advanceTaixuNode(prev.taixu);
    const node = taixu.nodes[taixu.currentNodeIndex];
    surfaceNotice.value = `太虚幻境：节点 #${taixu.currentNodeIndex + 1} (${node?.title ?? ""}) · 金币 ${taixu.gold}`;
    return { ...prev, taixu };
  });
}

function onBuyTaixuBlessing(name: string, cost: number) {
  update((prev) => {
    const taixu = buyTaixuBlessing(prev.taixu, name, cost);
    surfaceNotice.value = `太虚赐福：获得「${name}」（金币 ${taixu.gold}）`;
    return { ...prev, taixu };
  });
}

function onBindTaixuPet(petName: string) {
  update((prev) => {
    const taixu = bindTaixuPet(prev.taixu, petName);
    surfaceNotice.value = `太虚灵宠：绑定「${petName}」`;
    return { ...prev, taixu };
  });
}

function onDrawMartialReplacement() {
  update((prev) => {
    const martial = applyMartialReplacement(prev.martial, ["sp_zhaoyun", "shen_guanyu"]);
    surfaceNotice.value = `武皇替补：登场 ${martial.activeReplacement}（补摸 ${martial.replacementDraws} 张）`;
    return { ...prev, martial };
  });
}

function onRecordMartialWin() {
  update((prev) => {
    const recordBook = {
      ...prev.martial.recordBook,
      firstWin: prev.martial.recordBook.firstWin + 1,
    };
    surfaceNotice.value = `武皇战绩：先手 ${recordBook.firstWin} 胜`;
    return { ...prev, martial: { ...prev.martial, recordBook } };
  });
}

function onAdvanceDoomsday() {
  update((prev) => {
    const wave = prev.doomsday.wave + 1;
    surfaceNotice.value = `末日乱斗：波次 #${wave}`;
    return { ...prev, doomsday: { ...prev.doomsday, wave } };
  });
}

function onAdvanceLongxue() {
  update((prev) => {
    const longxue = advanceLongxueDaoxin(prev.longxue, "damage_dealt", 2);
    surfaceNotice.value = `龙血玄黄：道心 ${longxue.daoxin}/99（解锁 ${longxue.unlockedThresholds.join("/") || "无"}）`;
    return { ...prev, longxue };
  });
}

function onIncrementL20Mark(markKey = "longmark") {
  update((prev) => {
    const theme = incrementL20HudMark(prev.theme, markKey, 1);
    const count = theme.l20Marks?.[markKey] ?? 0;
    surfaceNotice.value = `标记补充 (L20)：${markKey} ×${count}（共 ${L20_HUD_MARK_SPECS.length} 项）`;
    return { ...prev, theme };
  });
}

function onToggleThunderMarkPatch() {
  const order: Array<"off" | "basic" | "full"> = ["off", "basic", "full"];
  update((prev) => {
    const idx = order.indexOf(prev.thunder.markPatchState);
    const markPatchState = order[(idx + 1) % order.length]!;
    surfaceNotice.value = `惊雷天牢标记补丁：${markPatchState}`;
    return { ...prev, thunder: { ...prev.thunder, markPatchState } };
  });
}

function onCycleDecade() {
  const order: Array<"shizhounian" | "shousha" | "xinzhan" | "ol" | "baby" | "classic"> = [
    "shizhounian",
    "shousha",
    "xinzhan",
    "ol",
    "baby",
    "classic",
  ];
  update((prev) => {
    const idx = order.indexOf(prev.theme.decadeSkinStyle);
    const decadeSkinStyle = order[(idx + 1) % order.length]!;
    surfaceNotice.value = `十周年 UI：风格切换为 ${decadeSkinStyle}`;
    return { ...prev, theme: { ...prev.theme, decadeSkinStyle } };
  });
}

function onCycleShoushaLayout() {
  update((prev) => {
    const shoushaLayout = prev.theme.shoushaLayout === "right" ? "left" : "right";
    surfaceNotice.value = `手杀 UI：布局切换为 ${shoushaLayout}`;
    return { ...prev, theme: { ...prev.theme, shoushaLayout } };
  });
}

function onCycleRuzhen() {
  update((prev) => {
    const rzshTheme = prev.theme.rzshTheme === "illusion" ? "reality" : "illusion";
    const rzshTierRoman = numToRoma(rzshTheme === "reality" ? 2 : 1);
    surfaceNotice.value = `如真似幻：主题 ${rzshTheme} · 等阶 ${rzshTierRoman}`;
    return { ...prev, theme: { ...prev.theme, rzshTheme, rzshTierRoman } };
  });
}

function onCycleQhly() {
  const order: Array<"decade" | "shousha" | "classic"> = ["decade", "shousha", "classic"];
  update((prev) => {
    const idx = order.indexOf(prev.theme.qhlyTheme);
    const qhlyTheme = order[(idx + 1) % order.length]!;
    surfaceNotice.value = `千幻聆音：窗口皮肤 ${qhlyTheme}`;
    return { ...prev, theme: { ...prev.theme, qhlyTheme } };
  });
}

function onSwitchSkin(seat: number) {
  update((prev) => {
    const current = prev.theme.activeSkinBySeat[seat] ?? 1;
    const nextIdx = (current % 4) + 1;
    const activeSkinBySeat = { ...prev.theme.activeSkinBySeat, [seat]: nextIdx };
    surfaceNotice.value = `皮肤切换：P${seat} → 皮肤 #${nextIdx}`;
    return { ...prev, theme: { ...prev.theme, activeSkinBySeat } };
  });
}

function onThrowEmotion(targetSeat: number) {
  update((prev) => {
    const throwEmotionCue = `P${props.selfId}->P${targetSeat}:egg`;
    surfaceNotice.value = `笨蛋插件：${throwEmotionCue}`;
    return { ...prev, theme: { ...prev.theme, throwEmotionCue } };
  });
}

function onRunOmnisearch() {
  update((prev) => {
    const hiddenIds = hiddenCardIdsFromSnapshot(props.view, props.selfId);
    const omnisearch = runOmnisearchQuery(prev.omnisearch, searchInput.value, hiddenIds);
    surfaceNotice.value = omnisearch.lastError
      ? `全能搜索拦截：${omnisearch.lastError}`
      : `全能搜索：命中 ${omnisearch.results.length} 条`;
    return { ...prev, omnisearch };
  });
}

function onValidateExportName() {
  try {
    const safe = validateSafeExportFilename(exportNameInput.value);
    exportValidationMsg.value = `合法：${safe}`;
  } catch (err) {
    exportValidationMsg.value = `拒绝：${err instanceof Error ? err.message : String(err)}`;
  }
}

function onRotateU01Gallery() {
  update((prev) => {
    const theme = rotateU01GalleryStandee(prev.theme);
    surfaceNotice.value = `3D 精选 (U01)：${theme.u01SelectedGeneral} · 偏航 ${theme.u01YawDeg}° · 缩放 ${theme.u01ZoomPct}%（共 ${U01_DDD_GALLERY_GENERALS.length} 将）`;
    return { ...prev, theme };
  });
}

function onTriggerSpineClip() {
  update((prev) => {
    const theme = triggerSpineSkinClip(prev.theme);
    surfaceNotice.value = `Spine 4.0 变换参数 (L08/L24)：${theme.spineActiveClip} · 偏移 (${theme.skinStandX},${theme.skinStandY}) · 缩放 ${theme.skinScalePct}%`;
    return { ...prev, theme };
  });
}

function onTriggerShoushaBanner() {
  update((prev) => {
    const theme = triggerShoushaHudBanner(prev.theme, "无双·千军破(zhefu)");
    surfaceNotice.value = `手杀 UI (L14)：${theme.shoushaKillBanner} · 倒计时 ${theme.shoushaCountdownPct}%`;
    return { ...prev, theme };
  });
}

function onApplyZuanDiyOffline() {
  update((prev) => {
    const workbench = applyZuanDiyDraft(prev.workbench, {
      generalId: "ssr_sunquan",
      maxHp: 5,
      onlineRoom: false,
    });
    surfaceNotice.value = `祖安 DIY (L25)：${workbench.zuanDiyStatus}`;
    return { ...prev, workbench };
  });
}

function onRejectZuanDiyOnline() {
  update((prev) => {
    return {
      ...prev,
      workbench: applyZuanDiyDraft(prev.workbench, {
        generalId: "ssr_online",
        maxHp: 99,
        onlineRoom: true,
      }),
    };
  });
}

function onSelectL27DualGenerals() {
  update((prev) => {
    const workbench = selectL27DualGenerals(prev.workbench, "ddd_caomao", "ddd_liuchong");
    surfaceNotice.value = `选将美化双将 (L27)：已锁定 ${workbench.l27DualPrimary} + ${workbench.l27DualSecondary}`;
    return { ...prev, workbench };
  });
}

function onSurfacePointerMove(ev: MouseEvent) {
  const rect = (ev.currentTarget as HTMLElement | null)?.getBoundingClientRect();
  if (!rect) return;
  const dx = Math.round(ev.clientX - rect.left);
  const dy = Math.round(ev.clientY - rect.top);
  const lastPointerClass = classifyDesktopPointer(props.surfaces.theme.pointerLayout, dx, dy);
  if (lastPointerClass !== props.surfaces.theme.lastPointerClass) {
    emit("update:surfaces", {
      ...props.surfaces,
      theme: { ...props.surfaces.theme, lastPointerClass },
    });
  }
}
</script>

<template>
  <section
    class="panel extension-surface-panel"
    data-testid="extension-surface-panel"
    aria-label="扩展渲染面与单机战役台"
    @mousemove="onSurfacePointerMove"
  >
    <!-- 天气粒子层（U04/L05 金币下雪/烟花/星云/闪烁 CSS 粒子层） -->
    <div
      v-if="surfaces.coin.snowFall || surfaces.coin.activeEffects.length > 0"
      class="weather-particle-layer"
      :data-weather="surfaces.coin.snowFall ? 'snow' : surfaces.coin.activeEffects[0]"
      data-testid="coin-weather-overlay"
      aria-hidden="true"
    >
      <span v-for="n in 8" :key="n" class="weather-particle" :style="{ '--idx': n }">
        {{ surfaces.coin.snowFall ? "❄" : "✦" }}
      </span>
    </div>

    <div class="surface-header">
      <div class="surface-title-group">
        <h2>扩展渲染面与战役台（裁决 34 · 37 来源原生落点）</h2>
        <span class="surface-badges">
          <span class="badge" data-testid="badge-decade">十周年:{{ surfaces.theme.decadeSkinStyle }}</span>
          <span class="badge" data-testid="badge-shousha">手杀:{{ surfaces.theme.shoushaLayout }}</span>
          <span class="badge" data-testid="badge-ruzhen">如真:{{ surfaces.theme.rzshTheme }}({{ surfaces.theme.rzshTierRoman }})</span>
          <span class="badge" data-testid="badge-qhly">千幻:{{ surfaces.theme.qhlyTheme }}</span>
          <span class="badge" data-testid="badge-coin">🪙 {{ surfaces.coin.balance }}</span>
          <span class="badge" data-testid="badge-daoxin">道心 {{ surfaces.longxue.daoxin }}/99</span>
        </span>
      </div>
      <div class="surface-tabs" role="tablist">
        <button
          type="button"
          role="tab"
          data-testid="tab-hud"
          :class="{ active: activeTab === 'hud' }"
          @click="activeTab = 'hud'"
        >
          战局 HUD (L08/L09/L12/L13/L14/L28/L29)
        </button>
        <button
          type="button"
          role="tab"
          data-testid="tab-campaign"
          :class="{ active: activeTab === 'campaign' }"
          @click="activeTab = 'campaign'"
        >
          单机战役 (U02/L03/L11/L19/L21)
        </button>
        <button
          type="button"
          role="tab"
          data-testid="tab-coin"
          :class="{ active: activeTab === 'coin' }"
          @click="activeTab = 'coin'"
        >
          金币假人 (U04/L05)
        </button>
        <button
          type="button"
          role="tab"
          data-testid="tab-omnisearch"
          :class="{ active: activeTab === 'omnisearch' }"
          @click="activeTab = 'omnisearch'"
        >
          全能搜索 (L07)
        </button>
        <button
          type="button"
          role="tab"
          data-testid="tab-workbench"
          :class="{ active: activeTab === 'workbench' }"
          @click="activeTab = 'workbench'"
        >
          工作台与适配 (U01/L01/L02/L15-L27)
        </button>
      </div>
    </div>

    <p v-if="surfaceNotice" class="surface-notice" data-testid="surface-notice" role="status">
      {{ surfaceNotice }}
    </p>

    <!-- Tab 1: 战局 HUD -->
    <div v-if="activeTab === 'hud'" class="surface-body" data-testid="panel-hud">
      <div class="hud-grid">
        <div class="hud-card" data-testid="hud-mvp-card">
          <h3>手杀 MVP 计分板 (L13) · 当前领先 P{{ surfaces.mvp.mvpSeat ?? 0 }}</h3>
          <ul class="mvp-list">
            <li v-for="entry in mvpSortedEntries" :key="entry.seat" :data-testid="`mvp-seat-${entry.seat}`">
              <strong>P{{ entry.seat }}</strong>
              <span>总分 {{ entry.total }}</span>
              <span class="muted">攻{{ entry.attack }} / 疗{{ entry.heal }} / 辅{{ entry.assist }} / 杀{{ entry.kills }}</span>
            </li>
          </ul>
        </div>

        <div class="hud-card" data-testid="hud-longxue-card">
          <h3>龙血玄黄 · 道心进度条 (L29) &amp; 标记补充 (L20)</h3>
          <div class="daoxin-bar">
            <div class="daoxin-fill" :style="{ width: `${surfaces.longxue.daoxin}%` }"></div>
          </div>
          <p class="tiny">
            当前 <strong data-testid="longxue-daoxin">{{ surfaces.longxue.daoxin }}</strong>/99 · 已解锁阈值：
            <strong data-testid="longxue-thresholds">{{ surfaces.longxue.unlockedThresholds.join(", ") || "无" }}</strong> ·
            潜龙技能：<strong data-testid="longxue-unlocked-skills">{{ unlockLongxueSkills(surfaces.longxue.daoxin).join("/") || "待解锁" }}</strong>
          </p>
          <p class="tiny" data-testid="l20-mark-badges">
            L20 标记补充 ({{ L20_HUD_MARK_SPECS.length }}项)：龙印 ×{{ surfaces.theme.l20Marks?.longmark ?? 0 }}
          </p>
          <div class="row wrap">
            <button type="button" data-testid="btn-longxue-plus" @click="onAdvanceLongxue">道心 +30</button>
            <button type="button" data-testid="btn-l20-mark-plus" @click="onIncrementL20Mark('longmark')">
              L20 龙印 +1
            </button>
            <button type="button" data-testid="btn-thunder-toggle" @click="onToggleThunderMarkPatch">
              惊雷标记补丁 (L28): {{ surfaces.thunder.markPatchState }}
            </button>
          </div>
        </div>

        <div class="hud-card" data-testid="hud-skin-theme-card">
          <h3>主题皮肤与镜头切换 (U01/L08/L09/L12/L14/L24/L26)</h3>
          <div class="row wrap">
            <button type="button" data-testid="btn-cycle-decade" @click="onCycleDecade">
              十周年风格 ({{ surfaces.theme.decadeSkinStyle }})
            </button>
            <button type="button" data-testid="btn-cycle-shousha" @click="onCycleShoushaLayout">
              手杀布局 ({{ surfaces.theme.shoushaLayout }})
            </button>
            <button type="button" data-testid="btn-cycle-ruzhen" @click="onCycleRuzhen">
              如真似幻 ({{ surfaces.theme.rzshTheme }})
            </button>
            <button type="button" data-testid="btn-cycle-qhly" @click="onCycleQhly">
              千幻皮肤 ({{ surfaces.theme.qhlyTheme }})
            </button>
            <button type="button" data-testid="btn-switch-skin" @click="onSwitchSkin(0)">
              切换 P0 皮肤 (#{{ surfaces.theme.activeSkinBySeat[0] ?? 1 }})
            </button>
            <button type="button" data-testid="btn-fool-emotion" @click="onThrowEmotion((selfId + 1) % 4)">
              投掷互动表情 ({{ surfaces.theme.throwEmotionCue ?? "无" }})
            </button>
          </div>

          <div class="sub-surface-box" data-testid="u01-3d-select-gallery">
            <span
              class="standee-pill"
              data-testid="u01-3d-standee"
              :style="{
                transform: `perspective(600px) rotateY(${surfaces.theme.u01YawDeg ?? 15}deg) scale(${(surfaces.theme.u01ZoomPct ?? 110) / 100})`,
              }"
            >
              3D立绘:{{ surfaces.theme.u01SelectedGeneral ?? "ddd_caomao" }} ({{ surfaces.theme.u01YawDeg ?? 15 }}°/{{ surfaces.theme.u01ZoomPct ?? 110 }}%)
            </span>
            <button type="button" data-testid="btn-u01-3d-rotate" @click="onRotateU01Gallery">
              切换 3D 精选立绘 (31将/64技)
            </button>
          </div>

          <div class="sub-surface-box" data-testid="l24-skin-transform-panel">
            <canvas
              ref="spineCanvasRef"
              width="220"
              height="52"
              class="spine-canvas"
              data-testid="spine-skeletal-canvas"
            ></canvas>
            <span class="tiny" data-testid="l24-skin-transform-meta">
              动作:{{ surfaces.theme.spineActiveClip ?? "daiji" }} · 偏移({{ surfaces.theme.skinStandX ?? 4 }},{{ surfaces.theme.skinStandY ?? 0 }}) · 缩放 {{ surfaces.theme.skinScalePct ?? 100 }}%
            </span>
            <button type="button" data-testid="btn-l24-trigger-clip" @click="onTriggerSpineClip">
              触发 Spine 骨骼动作 (L08/L24)
            </button>
          </div>

          <div class="sub-surface-box" data-testid="l14-shousha-hud">
            <div class="daoxin-bar">
              <div class="daoxin-fill" :style="{ width: `${surfaces.theme.shoushaCountdownPct ?? 85}%` }"></div>
            </div>
            <span class="tiny" data-testid="l14-shousha-banner-text">
              手杀播报:{{ surfaces.theme.shoushaKillBanner ?? "连破·一血(spshidi)" }} · 倒计时 {{ surfaces.theme.shoushaCountdownPct ?? 85 }}%
            </span>
            <button type="button" data-testid="btn-l14-shousha-banner" @click="onTriggerShoushaBanner">
              触发手杀连破横幅 (L14·48技)
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Tab 2: 单机战役脚本 -->
    <div v-else-if="activeTab === 'campaign'" class="surface-body" data-testid="panel-campaign">
      <div class="hud-grid">
        <div class="hud-card" data-testid="campaign-boss-card">
          <h3>挑战 BOSS / 守卫剑阁 (U02 / L03)</h3>
          <p v-if="currentBossStage">
            当前首领：<strong data-testid="boss-name">{{ currentBossStage.title }}</strong> ·
            阶段 {{ surfaces.boss.stageIndex + 1 }}/{{ surfaces.boss.stages.length }} ·
            体力 <code data-testid="boss-hp">{{ currentBossStage.normalizedHp.label }}</code> ·
            模式 <code data-testid="boss-mode">{{ surfaces.boss.mode }}</code>
          </p>
          <div class="row wrap">
            <button type="button" data-testid="btn-boss-advance" @click="onAdvanceBoss">推进剑阁阶段</button>
            <button type="button" data-testid="btn-boss-mode" @click="onCycleBossMode">切换战役模式</button>
          </div>
        </div>

        <div class="hud-card" data-testid="campaign-taixu-card">
          <h3>太虚幻境 · 战役节点图与赐福商店 (L11)</h3>
          <p>
            赛季：{{ surfaces.taixu.seasonId }} ·
            节点 <strong data-testid="taixu-node">#{{ surfaces.taixu.currentNodeIndex + 1 }} ({{ currentTaixuNode?.title }})</strong> ·
            金币 <strong data-testid="taixu-gold">{{ surfaces.taixu.gold }}</strong> ·
            灵宠 <strong data-testid="taixu-pet">{{ surfaces.taixu.boundPet }}</strong>
          </p>
          <p class="tiny">
            已选赐福 (<span data-testid="taixu-blessings-count">{{ surfaces.taixu.blessings.length }}</span>/{{ surfaces.taixu.maxBlessings }})：
            {{ surfaces.taixu.blessings.join("、") || "暂无" }}
          </p>
          <div class="row wrap">
            <button type="button" data-testid="btn-taixu-advance" @click="onAdvanceTaixu">推进太虚节点 (+30金)</button>
            <button type="button" data-testid="btn-taixu-buy-blessing" @click="onBuyTaixuBlessing('破军神威', 40)">
              购买赐福「破军神威」(40金)
            </button>
            <button type="button" data-testid="btn-taixu-bind-pet" @click="onBindTaixuPet('苍雷幼麟')">
              绑定灵宠「苍雷幼麟」
            </button>
          </div>
        </div>

        <div class="hud-card" data-testid="campaign-martial-doomsday-card">
          <h3>武皇 5v5 替补战绩簿 (L21) &amp; 末日乱斗 (L19)</h3>
          <p>
            5v5 友方：{{ surfaces.martial.friends.join("/") }} ·
            替补登场 <strong data-testid="martial-replacement">{{ surfaces.martial.activeReplacement ?? "无" }}</strong> ·
            先手战绩 {{ surfaces.martial.recordBook.firstWin }}胜/{{ surfaces.martial.recordBook.firstLose }}负
          </p>
          <p>
            末日波次：<strong data-testid="doomsday-wave">#{{ surfaces.doomsday.wave }}</strong> ·
            身份池 {{ surfaces.doomsday.identities.join(" / ") }}
          </p>
          <div class="row wrap">
            <button type="button" data-testid="btn-martial-draw" @click="onDrawMartialReplacement">抽取 5v5 替补</button>
            <button type="button" data-testid="btn-martial-win" @click="onRecordMartialWin">登记 5v5 胜场</button>
            <button type="button" data-testid="btn-doomsday-advance" @click="onAdvanceDoomsday">推进末日波次</button>
          </div>
        </div>
      </div>
    </div>

    <!-- Tab 3: 金币假人 -->
    <div v-else-if="activeTab === 'coin'" class="surface-body" data-testid="panel-coin">
      <div class="hud-grid">
        <div class="hud-card" data-testid="coin-shop-card">
          <h3>金币商店与天气特效 (U04 / L05)</h3>
          <p>
            当前金币：<strong data-testid="coin-balance">{{ surfaces.coin.balance }}</strong> ·
            荣誉称号：<strong data-testid="coin-honor-title">{{ resolveCoinHonorTitle(surfaces.coin.balance) }}</strong> ·
            赛前下注：<strong data-testid="coin-active-bet">{{ surfaces.coin.betPlaced }}</strong> ·
            下雪状态：<strong data-testid="coin-snow">{{ surfaces.coin.snowFall ? "snow" : "clear" }}</strong>
          </p>
          <div class="row wrap">
            <button type="button" data-testid="btn-coin-snow" @click="onBuyCoinEffect('snow')">下雪 (20金)</button>
            <button type="button" data-testid="btn-coin-firework" @click="onBuyCoinEffect('fireworks')">烟花 (50金)</button>
            <button type="button" data-testid="btn-coin-nebula" @click="onBuyCoinEffect('nebula')">星云 (10金)</button>
            <button type="button" data-testid="btn-coin-twinkle" @click="onBuyCoinEffect('twinkle')">闪烁 (10金)</button>
            <button type="button" data-testid="btn-coin-bet20" @click="onPlaceBet(20)">赛前下注 +20</button>
          </div>
        </div>
        <div class="hud-card">
          <h3>对局荣誉结算记录</h3>
          <p data-testid="coin-last-settlement">
            {{
              surfaces.coin.lastSettlement
                ? `模式 ${surfaces.coin.lastSettlement.mode} · 奖励 +${surfaces.coin.lastSettlement.netDelta} 金币`
                : "本局尚未终局结算（终局时自动发奖，录像回放不重复入账）"
            }}
          </p>
        </div>
      </div>
    </div>

    <!-- Tab 4: 全能搜索 -->
    <div v-else-if="activeTab === 'omnisearch'" class="surface-body" data-testid="panel-omnisearch">
      <div class="hud-card">
        <h3>全能搜索百科面板 (L07 · 含暗牌隔离与零 eval 门禁)</h3>
        <div class="row">
          <input
            v-model="searchInput"
            type="text"
            data-testid="input-omnisearch"
            aria-label="全能搜索关键词"
            placeholder="输入卡牌/武将/技能或 #99"
          />
          <button type="button" data-testid="btn-omnisearch-run" @click="onRunOmnisearch">检索</button>
        </div>
        <p
          v-if="surfaces.omnisearch.lastError"
          class="error"
          data-testid="omnisearch-blocked"
        >
          安全门禁拦截：{{ surfaces.omnisearch.lastError }}
        </p>
        <ul v-else class="omnisearch-results" data-testid="omnisearch-results">
          <li v-for="item in surfaces.omnisearch.results" :key="item.id">
            <span class="badge">{{ item.kind }}</span>
            <strong>{{ item.displayName }}</strong> ({{ item.id }}) — {{ item.packageId }}
          </li>
        </ul>
      </div>
    </div>

    <!-- Tab 5: 工作台与适配 -->
    <div v-else class="surface-body" data-testid="panel-workbench">
      <div class="hud-grid">
        <div class="hud-card">
          <h3>电脑端指针适配与导出文件名白名单 (L23)</h3>
          <p class="tiny" data-testid="desktop-pointer-coords">
            指针布局：{{ surfaces.theme.pointerLayout }} · 最近手势：{{ surfaces.theme.lastPointerClass }}
          </p>
          <div class="row">
            <input v-model="exportNameInput" type="text" data-testid="input-export-filename" aria-label="导出文件名" />
            <button type="button" data-testid="btn-validate-export" @click="onValidateExportName">校验文件名</button>
          </div>
          <p class="tiny" data-testid="export-validation-msg">{{ exportValidationMsg }}</p>
        </div>
        <div class="hud-card">
          <h3>偏好、天梯、编辑器与更新门禁 (L01/L02/L15-L20/L22/L25/L27)</h3>
          <p class="tiny">
            AI 变体 #{{ surfaces.workbench.aiTuningVariant }} ·
            版本比对 {{ surfaces.workbench.sjVersionCompare }} ·
            手杀全扩 {{ surfaces.workbench.shoushaFilteredPacks.length }} 包 ·
            拖动阅读 {{ surfaces.workbench.dragLastKind }} ·
            斗转星移「{{ surfaces.workbench.douzhuanRewrittenText }}」·
            更新镜像 {{ surfaces.workbench.updateGuardUrl }} ·
            段位强度 {{ surfaces.workbench.eloDifficulty.strength }} (延迟 {{ surfaces.workbench.eloDifficulty.thinkDelaySec }}s) ·
            笨蛋提示「{{ surfaces.workbench.foolHintLabel }}」
          </p>
          <div class="sub-surface-box" data-testid="l25-zuan-diy-editor">
            <span class="tiny" data-testid="l25-diy-status">
              祖安 DIY (L25): {{ surfaces.workbench.zuanDiyStatus ?? "离线沙箱就绪 (60将/122技)" }} · 武将 {{ surfaces.workbench.zuanDiyGeneralId ?? "ssr_luxun" }} ({{ surfaces.workbench.zuanDiyMaxHp ?? 4 }}血)
            </span>
            <button type="button" data-testid="btn-l25-diy-apply" @click="onApplyZuanDiyOffline">
              应用单机 DIY 武将
            </button>
            <button type="button" data-testid="btn-l25-diy-online-guard" @click="onRejectZuanDiyOnline">
              校验联机禁用门禁
            </button>
          </div>
          <div class="sub-surface-box" data-testid="l27-dual-general-picker">
            <span class="tiny" data-testid="l27-dual-status">
              双将选将框 (L27 chooseCharacterTwo): {{ surfaces.workbench.l27DualPrimary ?? "caocao" }} + {{ surfaces.workbench.l27DualSecondary ?? "simayi" }} ({{ surfaces.workbench.l27DualLocked ? "已锁定" : "待选" }} · 37技)
            </span>
            <button type="button" data-testid="btn-l27-dual-pick" @click="onSelectL27DualGenerals">
              锁定双将组合 (ddd_caomao+ddd_liuchong)
            </button>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.extension-surface-panel {
  position: relative;
  overflow: hidden;
  border: 1px solid var(--border);
}
.surface-header {
  display: flex;
  flex-direction: column;
  gap: 0.45rem;
}
.surface-title-group {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 0.4rem;
}
.surface-title-group h2 {
  margin: 0;
  font-size: 1rem;
}
.surface-badges {
  display: flex;
  flex-wrap: wrap;
  gap: 0.3rem;
}
.badge {
  font-size: 0.74rem;
  padding: 0.1rem 0.45rem;
  border-radius: 999px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  font-family: var(--mono);
}
.surface-tabs {
  display: flex;
  flex-wrap: wrap;
  gap: 0.35rem;
}
.surface-tabs button.active {
  border-color: var(--accent);
  color: var(--accent);
}
.surface-notice {
  margin: 0.4rem 0 0;
  font-size: 0.82rem;
  color: var(--accent);
  font-family: var(--mono);
}
.surface-body {
  margin-top: 0.55rem;
}
.hud-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
  gap: 0.6rem;
}
.hud-card {
  padding: 0.55rem 0.7rem;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 8px;
}
.hud-card h3 {
  margin: 0 0 0.35rem;
  font-size: 0.88rem;
}
.mvp-list,
.omnisearch-results {
  margin: 0.3rem 0 0;
  padding-left: 1rem;
  font-size: 0.82rem;
}
.mvp-list li {
  display: flex;
  gap: 0.5rem;
  align-items: center;
}
.daoxin-bar {
  height: 6px;
  background: var(--bg-panel);
  border-radius: 3px;
  overflow: hidden;
  margin: 0.3rem 0;
}
.daoxin-fill {
  height: 100%;
  background: linear-gradient(90deg, var(--accent-2), var(--accent));
  transition: width 0.25s ease;
}
.wrap {
  flex-wrap: wrap;
}
.sub-surface-box {
  margin-top: 0.45rem;
  padding: 0.35rem 0.5rem;
  border: 1px dashed var(--border);
  border-radius: 6px;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.4rem;
}
.standee-pill {
  display: inline-block;
  padding: 0.15rem 0.5rem;
  border-radius: 4px;
  background: var(--bg-panel);
  border: 1px solid var(--accent);
  font-family: var(--mono);
  font-size: 0.76rem;
  transition: transform 0.2s ease;
}
.spine-canvas {
  border-radius: 4px;
  border: 1px solid var(--border);
  background: #121a26;
}
.weather-particle-layer {
  position: absolute;
  inset: 0;
  pointer-events: none;
  display: flex;
  justify-content: space-around;
  align-items: flex-start;
  opacity: 0.35;
  font-size: 1rem;
  z-index: 1;
}
.weather-particle {
  animation: weather-fall 2.6s linear infinite;
}
@keyframes weather-fall {
  0% { transform: translateY(-6px); opacity: 0.2; }
  50% { opacity: 0.85; }
  100% { transform: translateY(48px); opacity: 0.1; }
}
@media (prefers-reduced-motion: reduce) {
  .weather-particle { animation: none; }
}
</style>
