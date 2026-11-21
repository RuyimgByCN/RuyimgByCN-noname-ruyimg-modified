<script setup lang="ts">
import { computed, ref } from "vue";
import type { ObserverSnapshot } from "../../../types/engine";
import {
  buildPauseHistoryRows,
  clampSliderValue,
  moveGuanXingCardRow,
  resolveGameOverBanner,
  resolveGameStartBanner,
  swapGuanXingCards,
  type DecadeGuanXingState,
  type DecadeLineArcState,
} from "../../../engine/surfaces/decadeSurfacesRuntime";
import { validatePoxiSelection } from "../../../engine/surfaces/decadeHooksRuntime";
import {
  computeDecadeSeatAnchor,
  type RelativeSeatSlot,
} from "../../../engine/surfaces/decadeTabletopLayout";
import type { DecadeUiRuntimeState } from "../../../engine/surfaces/decadeUiSurface";

const props = withDefaults(
  defineProps<{
    decadeUi?: DecadeUiRuntimeState;
    liveLineArc?: DecadeLineArcState | null;
    view: ObserverSnapshot | null;
    selfId?: number;
    orderedSeats?: ReadonlyArray<RelativeSeatSlot>;
    selfOnRight?: boolean;
    finished?: boolean;
    selfAlive?: boolean;
    startBannerDismissed?: boolean;
    zone?: "banners" | "stage" | "all";
  }>(),
  { zone: "all" },
);

const showBanners = computed(() => props.zone === "all" || props.zone === "banners");
const showStage = computed(() => props.zone === "all" || props.zone === "stage");

const emit = defineEmits<{
  updateDecadeUi: [next: DecadeUiRuntimeState];
  dismissStartBanner: [];
}>();

const chatDraft = ref("");

const cfg = computed(() => props.decadeUi?.config);
const animAllowed = computed(() => cfg.value?.gameAnimationEffect !== false);

const startBannerText = computed(() => {
  if (props.startBannerDismissed || !cfg.value || cfg.value.SanGuoShaGameStart === "off") {
    return null;
  }
  return props.decadeUi?.startBanner ?? resolveGameStartBanner(cfg.value.SanGuoShaGameStart);
});

const gameOverBannerText = computed(() => {
  if (!props.finished || !cfg.value || cfg.value.SanGuoShaGameOver === "off") return null;
  return resolveGameOverBanner(cfg.value.SanGuoShaGameOver, Boolean(props.selfAlive));
});

const pauseRows = computed(() => buildPauseHistoryRows(props.view));

const sliderInfo = computed(() =>
  clampSliderValue(10, 100, props.decadeUi?.sliderSpeedPct ?? 50),
);

function findSeatCenterPct(seatOrPlayerId: number): { x: number; y: number } {
  const slots = props.orderedSeats ?? [];
  const slot =
    slots.find((s) => s.seat.player === seatOrPlayerId) ??
    slots.find((s) => s.seat.seat === seatOrPlayerId);
  if (!slot) return { x: 50, y: 50 };
  return computeDecadeSeatAnchor(slot.position, slot.total, {
    selfOnRight: Boolean(props.selfOnRight),
    decadeLayout: cfg.value?.decadeLayout ?? "on",
  }).centerPct;
}

const activeLineArc = computed(() => props.liveLineArc ?? props.decadeUi?.lineArc ?? null);

const lineArcGeom = computed(() => {
  const arc = activeLineArc.value;
  if (!arc) return { x1: 50, y1: 50, x2: 50, y2: 50, stroke: "#fde047" };
  const from = findSeatCenterPct(arc.fromSeat);
  const to = findSeatCenterPct(arc.toSeat);
  const cpzs = cfg.value?.CPZS ?? "shoushaX";
  const stroke =
    cpzs === "shousha"
      ? "#f97316"
      : cpzs === "dajiangjun" || cpzs === "dasima"
        ? "#f43f5e"
        : cpzs === "jiangjun" || cpzs === "weijiangjun"
          ? "#38bdf8"
          : "#fde047";
  return { x1: from.x, y1: from.y, x2: to.x, y2: to.y, stroke };
});

interface PoxiPreviewCard {
  id: number;
  suit: string | null;
  label: string;
  revealed: boolean;
  side: "self" | "target";
}

const P_SUIT_GLYPH: Record<string, string> = { Spade: "♠", Heart: "♥", Club: "♣", Diamond: "♦" };

const poxiCards = computed<{ selfCards: PoxiPreviewCard[]; targetCards: PoxiPreviewCard[]; selfTargeted: boolean }>(() => {
  const dlg = props.decadeUi?.poxiDialog;
  if (!dlg) return { selfCards: [], targetCards: [], selfTargeted: false };
  const selfOwnerId =
    props.selfId ??
    props.orderedSeats?.find((s) => s.position === 0)?.seat.player ??
    0;
  const selfTargeted = dlg.targetSeat === selfOwnerId;
  const known = props.view?.known_cards ?? [];
  const selfHand = known.filter(
    (c) => c.owner === selfOwnerId && c.zone === "Hand" && c.def != null,
  );
  const selfCards: PoxiPreviewCard[] = selfHand.map((c) => {
    const suit = c.suit ?? "Spade";
    return {
      id: c.id,
      suit,
      label: `${P_SUIT_GLYPH[suit] ?? "?"}${c.rank ?? "?"} #${c.id}`,
      revealed: true,
      side: "self",
    };
  });

  if (selfTargeted) {
    return { selfCards, targetCards: [], selfTargeted: true };
  }

  const targetSeat = props.view?.seats.find((s) => s.player === dlg.targetSeat);
  const revealedTarget = known.filter(
    (c) => c.owner === dlg.targetSeat && c.zone === "Hand" && c.def != null,
  );
  const targetCards: PoxiPreviewCard[] = revealedTarget.map((c) => {
    const suit = c.suit ?? "Spade";
    return {
      id: c.id,
      suit,
      label: `${P_SUIT_GLYPH[suit] ?? "?"}${c.rank ?? "?"} #${c.id}`,
      revealed: true,
      side: "target",
    };
  });
  const hiddenCount = Math.max(0, (targetSeat?.hand_count ?? 0) - targetCards.length);
  for (let i = 0; i < hiddenCount; i++) {
    targetCards.push({
      id: -(i + 1),
      suit: null,
      label: `未知手牌 ${i + 1}`,
      revealed: false,
      side: "target",
    });
  }
  return { selfCards, targetCards, selfTargeted: false };
});

const poxiValidation = computed(() => {
  const dlg = props.decadeUi?.poxiDialog;
  if (!dlg) return null;
  if (poxiCards.value.selfTargeted) {
    return { valid: false, reason: "目标不能为自身（请选择其他角色）" };
  }
  const all = [...poxiCards.value.selfCards, ...poxiCards.value.targetCards];
  const selected = dlg.selectedIds
    .map((id) => all.find((c) => c.id === id))
    .filter((c): c is PoxiPreviewCard & { suit: string } => Boolean(c && c.revealed && c.suit));
  return validatePoxiSelection(dlg.mode, selected);
});

function togglePoxiCard(card: PoxiPreviewCard) {
  if (!card.revealed || !card.suit) return;
  const dlg = props.decadeUi?.poxiDialog;
  if (!dlg) return;
  const exists = dlg.selectedIds.includes(card.id);
  const selectedIds = exists
    ? dlg.selectedIds.filter((id) => id !== card.id)
    : [...dlg.selectedIds, card.id];
  updateState({ poxiDialog: { ...dlg, selectedIds } });
}

function togglePoxiMode() {
  const dlg = props.decadeUi?.poxiDialog;
  if (!dlg) return;
  const nextMode = dlg.mode === "drlt_poxi" ? "pro_drlt_poxi" : "drlt_poxi";
  updateState({ poxiDialog: { ...dlg, mode: nextMode, selectedIds: [] } });
}

function updateState(partial: Partial<DecadeUiRuntimeState>) {
  if (!props.decadeUi) return;
  emit("updateDecadeUi", { ...props.decadeUi, ...partial });
}

function togglePauseHistory() {
  updateState({ pauseHistoryOpen: !props.decadeUi?.pauseHistoryOpen });
}

function toggleChatFold() {
  if (!props.decadeUi) return;
  updateState({
    chatBox: {
      ...props.decadeUi.chatBox,
      folded: !props.decadeUi.chatBox.folded,
    },
  });
}

function onSendChat() {
  if (!props.decadeUi || !chatDraft.value.trim()) return;
  const text = chatDraft.value.trim();
  const nextId = (props.decadeUi.chatBox.messages.at(-1)?.id ?? 0) + 1;
  updateState({
    chatBox: {
      ...props.decadeUi.chatBox,
      messages: [
        ...props.decadeUi.chatBox.messages.slice(-24),
        { id: nextId, sender: "我", text },
      ],
    },
  });
  chatDraft.value = "";
}

function onSliderInput(ev: Event) {
  const val = Number((ev.target as HTMLInputElement).value);
  updateState({ sliderSpeedPct: clampSliderValue(10, 100, val).value });
}

function onGuanXingCardClick(state: DecadeGuanXingState, cardId: number) {
  if (state.selectedCardId == null) {
    updateState({ guanXing: { ...state, selectedCardId: cardId } });
    return;
  }
  if (state.selectedCardId === cardId) {
    updateState({ guanXing: moveGuanXingCardRow(state, cardId) });
    return;
  }
  updateState({ guanXing: swapGuanXingCards(state, state.selectedCardId, cardId) });
}
</script>

<template>
  <!-- Center-zone banners (`splash.js` + `effect.js`) inside .center-table-cards to prevent seat overlaps -->
  <div v-if="showBanners" class="decade-center-banners">
    <!-- Game-start splash banner (`SanGuoShaGameStart`, `splash.js`) -->
    <div
      v-if="startBannerText && animAllowed"
      class="decade-splash-banner"
      data-testid="decade-start-banner"
      @click="emit('dismissStartBanner')"
    >
      <span class="splash-title">{{ startBannerText }}</span>
      <span class="splash-sub">点击收起</span>
    </div>

    <!-- Game-over settlement banner (`SanGuoShaGameOver`) -->
    <div
      v-if="gameOverBannerText"
      class="decade-gameover-banner"
      :class="`mode-${cfg?.SanGuoShaGameOver}`"
      data-testid="decade-gameover-banner"
    >
      {{ gameOverBannerText }}
    </div>

    <!-- `_jinnangpai_` (`animation.js:1614`): AOE trick fullscreen ribbon inside center banner zone -->
    <div
      v-if="decadeUi?.trickEffectBanner && animAllowed"
      class="decade-trick-banner"
      :data-trick-card="decadeUi.trickEffectBanner.card"
      data-testid="decade-trick-banner"
      @click="updateState({ trickEffectBanner: null })"
    >
      ✨ {{ decadeUi.trickEffectBanner.title }}
    </div>

    <!-- `effect.js`: kill overlay, compare face-off, skill activation popup -->
    <div
      v-if="decadeUi?.killOverlay && animAllowed"
      class="decade-effect-toast kill"
      data-testid="decade-effect-kill"
      @click="updateState({ killOverlay: null })"
    >
      <strong>破敌击杀</strong> · {{ decadeUi.killOverlay.killerName }} 击败 {{ decadeUi.killOverlay.victimName }}
    </div>

    <div
      v-if="decadeUi?.skillPopup && animAllowed"
      class="decade-effect-toast skill"
      data-testid="decade-effect-skill"
      @click="updateState({ skillPopup: null })"
    >
      <span class="skill-general">{{ decadeUi.skillPopup.generalName }}</span>
      <strong>【{{ decadeUi.skillPopup.skillName }}】</strong>
    </div>

    <div
      v-if="decadeUi?.compareEffect"
      class="decade-compare-overlay"
      data-testid="decade-effect-compare"
    >
      <div class="compare-side">
        <span class="muted">{{ decadeUi.compareEffect.sourceName }}</span>
        <strong>{{ decadeUi.compareEffect.sourceCardLabel }}</strong>
      </div>
      <span class="compare-vs">VS</span>
      <div class="compare-side">
        <span class="muted">{{ decadeUi.compareEffect.targetName }}</span>
        <strong>{{ decadeUi.compareEffect.targetCardLabel }}</strong>
      </div>
      <button type="button" class="mini-close" @click="updateState({ compareEffect: null })">关闭</button>
    </div>
  </div>

  <!-- Stage-level layers (`effect.line` SVG, `content.chooseGuanXing`, `pause.js`, `component.chatBox`) -->
  <template v-if="showStage">
    <!-- SVG target arc line (`effect.line` + live target selection, styled by `CPZS`) -->
    <svg
      v-if="activeLineArc && cfg?.CPZS !== 'off' && animAllowed"
      class="decade-line-svg"
      viewBox="0 0 100 100"
      preserveAspectRatio="none"
      aria-hidden="true"
      data-testid="decade-effect-line"
    >
      <line
        :x1="lineArcGeom.x1"
        :y1="lineArcGeom.y1"
        :x2="lineArcGeom.x2"
        :y2="lineArcGeom.y2"
        :stroke="lineArcGeom.stroke"
        stroke-width="0.9"
        stroke-dasharray="2.5 1.5"
      />
      <circle :cx="lineArcGeom.x1" :cy="lineArcGeom.y1" r="1.2" :fill="lineArcGeom.stroke" />
      <circle :cx="lineArcGeom.x2" :cy="lineArcGeom.y2" r="1.4" :fill="lineArcGeom.stroke" />
    </svg>

    <!-- `drlt_poxi` / `pro_drlt_poxi` (`meihua.js:6770-6935`): honest 4-suit dual hand inspection modal -->
    <div
      v-if="decadeUi?.poxiDialog"
      class="decade-overlay-modal"
      data-testid="decade-poxi-dialog"
    >
      <div class="overlay-card poxi-card">
        <header class="overlay-head">
          <strong>{{ decadeUi.poxiDialog.mode === 'pro_drlt_poxi' ? '界·魄袭（1~4异花色）' : '魄袭（4异花色）' }}</strong>
          <div class="poxi-head-actions">
            <button
              type="button"
              class="mini-close"
              data-testid="btn-poxi-switch-mode"
              @click="togglePoxiMode"
            >
              切换{{ decadeUi.poxiDialog.mode === 'pro_drlt_poxi' ? '标准魄袭' : '界魄袭' }}
            </button>
            <button type="button" class="mini-close" data-testid="btn-poxi-close" @click="updateState({ poxiDialog: null })">关闭</button>
          </div>
        </header>
        <div class="poxi-pane">
          <span class="tiny muted">己方手牌 ({{ poxiCards.selfCards.length }})</span>
          <div class="gx-cards">
            <button
              v-for="c in poxiCards.selfCards"
              :key="`ps-${c.id}`"
              type="button"
              class="gx-card"
              :class="{ selected: decadeUi.poxiDialog.selectedIds.includes(c.id) }"
              :data-suit="c.suit ?? ''"
              :data-revealed="String(c.revealed)"
              data-testid="poxi-card-btn"
              @click="togglePoxiCard(c)"
            >
              {{ c.label }}
            </button>
          </div>
        </div>
        <div class="poxi-pane">
          <span class="tiny muted">目标 P{{ decadeUi.poxiDialog.targetSeat }} 手牌 ({{ poxiCards.targetCards.length }})</span>
          <div class="gx-cards">
            <button
              v-for="c in poxiCards.targetCards"
              :key="`pt-${c.id}`"
              type="button"
              class="gx-card"
              :class="{ selected: decadeUi.poxiDialog.selectedIds.includes(c.id), 'is-hidden-card': !c.revealed }"
              :disabled="!c.revealed"
              :data-suit="c.suit ?? ''"
              :data-revealed="String(c.revealed)"
              :data-testid="c.revealed ? 'poxi-card-btn' : 'poxi-hidden-card-btn'"
              @click="togglePoxiCard(c)"
            >
              {{ c.label }}
            </button>
          </div>
        </div>
        <div
          v-if="poxiValidation"
          class="poxi-validation-bar"
          :class="poxiValidation.valid ? 'is-valid' : 'is-invalid'"
          data-testid="poxi-validation"
        >
          {{ poxiValidation.reason }}
        </div>
      </div>
    </div>

    <!-- `content.js`: chooseGuanXing top/bottom card sorter -->
    <div
      v-if="decadeUi?.guanXing?.open"
      class="decade-overlay-modal"
      data-testid="decade-guanxing-dialog"
    >
      <div class="overlay-card">
        <header class="overlay-head">
          <strong>观星排列 (content.chooseGuanXing)</strong>
          <button type="button" class="mini-close" @click="updateState({ guanXing: null })">完成</button>
        </header>
        <div class="gx-row">
          <span class="tiny muted">牌堆顶</span>
          <div class="gx-cards">
            <button
              v-for="c in decadeUi.guanXing.topCards"
              :key="c.id"
              type="button"
              class="gx-card"
              :class="{ selected: decadeUi.guanXing.selectedCardId === c.id }"
              @click="onGuanXingCardClick(decadeUi.guanXing, c.id)"
            >
              {{ c.suit }}{{ c.rank }} {{ c.label }}
            </button>
          </div>
        </div>
        <div class="gx-row">
          <span class="tiny muted">牌堆底</span>
          <div class="gx-cards">
            <button
              v-for="c in decadeUi.guanXing.bottomCards"
              :key="c.id"
              type="button"
              class="gx-card"
              :class="{ selected: decadeUi.guanXing.selectedCardId === c.id }"
              @click="onGuanXingCardClick(decadeUi.guanXing, c.id)"
            >
              {{ c.suit }}{{ c.rank }} {{ c.label }}
            </button>
            <span v-if="!decadeUi.guanXing.bottomCards.length" class="tiny muted">（双击卡牌移至牌堆底）</span>
          </div>
        </div>
      </div>
    </div>

    <!-- `pause.js`: pause history drawer -->
    <div
      v-if="decadeUi?.pauseHistoryOpen"
      class="decade-overlay-modal"
      data-testid="decade-pause-history"
    >
      <div class="overlay-card">
        <header class="overlay-head">
          <strong>对局暂停历史 (pause.js)</strong>
          <button type="button" class="mini-close" @click="togglePauseHistory">恢复牌局</button>
        </header>
        <ul class="pause-list">
          <li v-for="row in pauseRows" :key="row.seat" :class="{ dead: !row.alive }">
            <span>#{{ row.seat + 1 }} {{ row.name }}</span>
            <span>体力 {{ row.hpText }} · 手牌 {{ row.handCount }}</span>
          </li>
        </ul>
      </div>
    </div>

    <!-- `component.js`: compact collapsible tabletop chatBox & pacing slider -->
    <div
      v-if="decadeUi && !decadeUi.chatBox.folded"
      class="decade-chat-popover"
      data-testid="decade-chat-box"
    >
      <div class="chat-head">
        <strong>牌桌快捷聊天</strong>
        <button type="button" class="mini-close" @click="toggleChatFold">收起</button>
      </div>
      <div class="chat-slider-row">
        <span class="tiny muted">动效速率 {{ sliderInfo.value }}%</span>
        <input
          type="range"
          min="10"
          max="100"
          :value="sliderInfo.value"
          class="decade-slider"
          aria-label="动画速率滑块"
          @input="onSliderInput"
        />
      </div>
      <ul class="chat-messages">
        <li v-for="msg in decadeUi.chatBox.messages.slice(-5)" :key="msg.id">
          <strong>{{ msg.sender }}:</strong> {{ msg.text }}
        </li>
      </ul>
      <div class="chat-input-row">
        <input
          v-model="chatDraft"
          type="text"
          placeholder="输入快捷语..."
          data-testid="decade-chat-input"
          @keydown.enter.prevent="onSendChat"
        />
        <button type="button" data-testid="btn-decade-chat-send" @click="onSendChat">发送</button>
      </div>
    </div>
  </template>
</template>

<style scoped>
.decade-center-banners {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.28rem;
}
.decade-splash-banner,
.decade-gameover-banner {
  position: relative;
  z-index: 16;
  padding: 0.22rem 0.85rem;
  border-radius: 999px;
  background: linear-gradient(90deg, rgba(18, 24, 34, 0.92), rgba(76, 48, 16, 0.92), rgba(18, 24, 34, 0.92));
  border: 1px solid #f59e0b;
  color: #fef3c7;
  font-size: 0.76rem;
  font-weight: 700;
  display: inline-flex;
  gap: 0.45rem;
  align-items: center;
  cursor: pointer;
}
.splash-sub {
  font-size: 0.64rem;
  opacity: 0.7;
}
.decade-effect-toast {
  position: relative;
  z-index: 18;
  padding: 0.3rem 0.85rem;
  border-radius: 8px;
  background: rgba(20, 14, 14, 0.92);
  border: 1.5px solid #f87171;
  color: #fee2e2;
  font-size: 0.8rem;
  cursor: pointer;
}
.decade-effect-toast.skill {
  border-color: #facc15;
  color: #fef9c3;
}
.decade-compare-overlay {
  position: relative;
  z-index: 19;
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.42rem 0.8rem;
  border-radius: 10px;
  background: rgba(16, 20, 28, 0.94);
  border: 1.5px solid #f59e0b;
  color: #fef3c7;
}
.compare-side {
  display: flex;
  flex-direction: column;
  align-items: center;
  font-size: 0.78rem;
}
.compare-vs {
  font-weight: 900;
  color: #f87171;
}
.decade-line-svg {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
  z-index: 4;
}
.decade-overlay-modal {
  position: absolute;
  inset: 0;
  z-index: 25;
  background: rgba(6, 10, 14, 0.72);
  display: grid;
  place-items: center;
  padding: 1rem;
}
.overlay-card {
  width: min(460px, 100%);
  background: #141a22;
  border: 1.5px solid #e8a54b;
  border-radius: 10px;
  padding: 0.8rem;
  color: #f3ead3;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}
.overlay-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.gx-row {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}
.gx-cards {
  display: flex;
  flex-wrap: wrap;
  gap: 0.35rem;
}
.gx-card {
  padding: 0.35rem 0.55rem;
  border-radius: 6px;
  border: 1px solid #d4af5a;
  background: #fdfbf7;
  color: #1c1917;
  font-size: 0.76rem;
  font-weight: 700;
}
.gx-card.selected {
  border-color: #f59e0b;
  box-shadow: 0 0 8px #f59e0b;
}
.pause-list {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
}
.pause-list li {
  display: flex;
  justify-content: space-between;
  font-size: 0.78rem;
  padding: 0.24rem 0.45rem;
  border-radius: 5px;
  background: rgba(255, 255, 255, 0.04);
}
.pause-list li.dead {
  opacity: 0.55;
}
.decade-chat-popover {
  position: absolute;
  left: clamp(118px, 14.5vw, 168px);
  bottom: 266px;
  top: auto;
  right: auto;
  transform: none;
  z-index: 22;
  width: min(244px, calc(100% - 28px));
  padding: 0.55rem;
  border-radius: 8px;
  background: rgba(14, 18, 24, 0.94);
  border: 1px solid rgba(232, 165, 75, 0.5);
  color: #f3ead3;
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
  font-size: 0.74rem;
}
@media (max-width: 767px) {
  .decade-chat-popover {
    position: fixed;
    left: 50%;
    top: auto;
    bottom: 14px;
    transform: translateX(-50%);
    z-index: 45;
  }
}
.chat-head,
.chat-slider-row,
.chat-input-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 0.35rem;
}
.decade-slider {
  flex: 1;
  min-width: 0;
}
.chat-input-row input {
  flex: 1;
  min-width: 0;
  background: #0b0e14;
  color: #f4ebd4;
  border: 1px solid rgba(255, 255, 255, 0.2);
  border-radius: 4px;
  padding: 0.2rem 0.35rem;
}
.chat-messages {
  list-style: none;
  margin: 0;
  padding: 0;
  max-height: 90px;
  overflow-y: auto;
}
.mini-close {
  font-size: 0.72rem;
  padding: 0.14rem 0.42rem;
  white-space: nowrap;
  flex-shrink: 0;
}
.tiny {
  font-size: 0.7rem;
}
.poxi-card {
  gap: 0.45rem;
}
.poxi-head-actions {
  display: inline-flex;
  gap: 0.3rem;
  align-items: center;
}
.poxi-pane {
  display: flex;
  flex-direction: column;
  gap: 0.2rem;
}
.gx-card.is-hidden-card {
  opacity: 0.55;
  border-style: dashed;
  cursor: not-allowed;
}
.poxi-validation-bar {
  padding: 0.22rem 0.45rem;
  border-radius: 5px;
  font-size: 0.72rem;
  font-weight: 700;
}
.poxi-validation-bar.is-valid {
  background: rgba(20, 83, 45, 0.45);
  border: 1px solid #4ade80;
  color: #dcfce7;
}
.poxi-validation-bar.is-invalid {
  background: rgba(127, 29, 29, 0.45);
  border: 1px solid #f87171;
  color: #fecaca;
}
</style>
