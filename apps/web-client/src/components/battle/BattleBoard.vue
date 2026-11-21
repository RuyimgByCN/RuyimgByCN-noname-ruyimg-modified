<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from "vue";
import type {
  CardView,
  ChoiceRequest,
  LegalAction,
  ObserverSnapshot,
  SeatView,
  SkillView,
} from "../../types/engine";
import SeatCard from "./SeatCard.vue";
import HandStrip from "./HandStrip.vue";
import TriggerOrderChoice from "./TriggerOrderChoice.vue";
import BattleShoushaHud, { type BattleModalId } from "./BattleShoushaHud.vue";
import type { ExtensionSurfacesSnapshot } from "../../engine/extensionSurfaceRuntime";
import { renderCoinCanvasParticles } from "../../engine/extensionSurfaceRuntime";
import { skillDisplayName } from "../../engine/appearanceStore";
import { choiceKindLabel, choicePromptLabel } from "../../engine/observerMetadata";
import {
  cardCategoryOf,
  cardKindLabel,
  computeDecadeSeatAnchor,
  computeRelativeSeatOrder,
} from "../../engine/surfaces/decadeTabletopLayout";

type ResolveChoiceAction = Extract<LegalAction, { ResolveChoice: unknown }>;

const props = defineProps<{
  view: ObserverSnapshot | null;
  selfId: number;
  legalActions: LegalAction[];
  surfaces?: ExtensionSurfacesSnapshot;
  choice?: ChoiceRequest | null;
  choiceActions?: ResolveChoiceAction[];
  choiceSecondsLeft?: number | null;
  canEndPhase?: boolean;
  busy?: boolean;
  connected?: boolean;
  finished?: boolean;
  isSpectator?: boolean;
  isOnline?: boolean;
  roomStarted?: boolean;
}>();

const emit = defineEmits<{
  playCard: [cardId: number, targets: number[]];
  activateSkill: [skillId: number, targets: number[], cardTargets: number[]];
  openModal: [modal: BattleModalId];
  sortHand: [sortedIds: number[]];
  endPhase: [];
  resolveChoice: [action: ResolveChoiceAction];
  resolveTriggerOrder: [token: number];
}>();

// Weather particle canvas & animation loop on the real tabletop
const tableWeatherCanvasRef = ref<HTMLCanvasElement | null>(null);
let weatherRafId: number | null = null;
let weatherFrame = 0;

function startWeatherAnimation() {
  if (weatherRafId != null) return;
  const loop = () => {
    if (tableWeatherCanvasRef.value && props.surfaces?.coin) {
      const ctx = tableWeatherCanvasRef.value.getContext("2d");
      if (ctx) {
        renderCoinCanvasParticles(ctx, props.surfaces.coin, weatherFrame++);
      }
    }
    weatherRafId = requestAnimationFrame(loop);
  };
  weatherRafId = requestAnimationFrame(loop);
}

function stopWeatherAnimation() {
  if (weatherRafId != null) {
    cancelAnimationFrame(weatherRafId);
    weatherRafId = null;
  }
}

onMounted(() => {
  if (props.surfaces?.coin?.snowFall || (props.surfaces?.coin?.activeEffects?.length ?? 0) > 0) {
    startWeatherAnimation();
  }
});

onUnmounted(() => {
  stopWeatherAnimation();
});

watch(
  () => Boolean(props.surfaces?.coin?.snowFall || (props.surfaces?.coin?.activeEffects?.length ?? 0) > 0),
  (active) => {
    if (active) {
      startWeatherAnimation();
    } else {
      stopWeatherAnimation();
    }
  }
);

// Card selection state.
const selectedCard = ref<number | null>(null);
const selectedSkill = ref<SkillView | null>(null);
// Target selection state.
const targetMode = ref(false);
const selectedTargets = ref<number[]>([]);
// Drag-and-drop: currently dragged card id.
const dragCardId = ref<number | null>(null);
// Drop zone highlight state.
const dropTargetId = ref<number | null>(null);

const selfOnRight = computed(() => props.surfaces?.theme?.shoushaLayout === "left");

function seats(): SeatView[] {
  return props.view?.seats ?? [];
}

const orderedSeats = computed(() => computeRelativeSeatOrder(seats(), props.selfId));

function seatAnchorStyle(position: number, total: number): Record<string, string> {
  return computeDecadeSeatAnchor(position, total, { selfOnRight: selfOnRight.value }).cssVars;
}

function seatAnchorZone(position: number, total: number): string {
  return computeDecadeSeatAnchor(position, total, { selfOnRight: selfOnRight.value }).zone;
}

function myHand(): CardView[] {
  if (!props.view) return [];
  return props.view.known_cards.filter(
    (c) => c.owner === props.selfId && c.zone === "Hand" && c.def != null
  );
}

function mySkills(): SkillView[] {
  const legalSkillIds = new Set(
    props.legalActions.flatMap((action) =>
      typeof action === "object" && "ActivateSkill" in action
        ? [action.ActivateSkill.skill]
        : []
    )
  );
  return seats().find((seat) => seat.player === props.selfId)?.skills
    .filter(
      (skill) =>
        skill.enabled &&
        (skill.active || skill.view_as) &&
        legalSkillIds.has(skill.id)
    ) ?? [];
}

function identityLabel(id: SeatView["identity"]) {
  switch (id) {
    case "Lord":
      return "主公";
    case "Loyalist":
      return "忠臣";
    case "Rebel":
      return "反贼";
    case "Renegade":
      return "内奸";
    default:
      return "身份未知";
  }
}

function selectableTargets(): Set<number> {
  if (!targetMode.value) return new Set();
  return new Set(currentTargetVectors().flat());
}

function discardCards(): CardView[] {
  if (!props.view) return [];
  return props.view.known_cards.filter(
    (c) => (c.zone === "DiscardPile" || c.zone === "Processing") && c.def != null
  );
}

function judgmentCardsFor(playerId: number): CardView[] {
  if (!props.view) return [];
  return props.view.known_cards.filter(
    (c) => c.owner === playerId && c.zone === "Judge" && c.def != null
  );
}

const selectableSet = computed(() => selectableTargets());

const actionableCardIds = computed(() => {
  if (selectedSkill.value) {
    return Array.from(
      new Set(skillActions(selectedSkill.value.id).flatMap((action) => action.card_targets))
    );
  }
  return Array.from(new Set(playActions().map((action) => action.card)));
});

function playActions(cardId?: number) {
  return props.legalActions.flatMap((action) => {
    if (typeof action !== "object" || !("PlayCard" in action)) return [];
    if (cardId != null && action.PlayCard.card !== cardId) return [];
    return [action.PlayCard];
  });
}

function skillActions(skillId: number) {
  return props.legalActions.flatMap((action) => {
    if (typeof action !== "object" || !("ActivateSkill" in action)) return [];
    return action.ActivateSkill.skill === skillId ? [action.ActivateSkill] : [];
  });
}

function currentTargetVectors(): number[][] {
  if (selectedSkill.value) {
    return skillActions(selectedSkill.value.id)
      .filter((action) =>
        selectedCard.value == null
          ? action.card_targets.length === 0
          : action.card_targets.length === 1 && action.card_targets[0] === selectedCard.value
      )
      .map((action) => action.targets);
  }
  if (selectedCard.value == null) return [];
  return playActions(selectedCard.value).map((action) => action.targets);
}

function sameTargets(left: number[], right: number[]): boolean {
  return left.length === right.length && left.every((id, index) => id === right[index]);
}

const canConfirm = computed(() =>
  currentTargetVectors().some((targets) => sameTargets(targets, selectedTargets.value))
);

const canPlayWithoutTarget = computed(() =>
  currentTargetVectors().some((targets) => targets.length === 0)
);

function equipCardsFor(playerId: number): CardView[] {
  if (!props.view) return [];
  return props.view.known_cards.filter(
    (c) => c.owner === playerId && c.zone === "Equip" && c.def != null
  );
}

function onCardSelect(cardId: number) {
  if (!actionableCardIds.value.includes(cardId)) return;
  if (selectedCard.value === cardId) {
    selectedCard.value = null;
    targetMode.value = false;
    selectedTargets.value = [];
    return;
  }
  selectedCard.value = cardId;
  if (!selectedSkill.value) selectedSkill.value = null;
  targetMode.value = true;
  selectedTargets.value = [];
}

function onSkillSelect(skill: SkillView) {
  if (selectedSkill.value?.id === skill.id) {
    cancelSelection();
    return;
  }
  selectedSkill.value = skill;
  selectedCard.value = null;
  targetMode.value = true;
  selectedTargets.value = [];
}

function onCardDragStart(cardId: number) {
  if (!actionableCardIds.value.includes(cardId)) return;
  dragCardId.value = cardId;
  selectedCard.value = cardId;
  targetMode.value = true;
}

function onTargetSelect(playerId: number) {
  if (!targetMode.value || !selectableSet.value.has(playerId)) return;
  if (selectedTargets.value.includes(playerId)) {
    selectedTargets.value = selectedTargets.value.filter((id) => id !== playerId);
  } else {
    const maxTargets = Math.max(0, ...currentTargetVectors().map((targets) => targets.length));
    selectedTargets.value = [...selectedTargets.value, playerId].slice(0, maxTargets);
  }
}

function onSeatDragOver(e: DragEvent, playerId: number) {
  if (dragCardId.value != null && selectableSet.value.has(playerId)) {
    e.preventDefault();
    e.dataTransfer!.dropEffect = "move";
    dropTargetId.value = playerId;
  }
}

function onSeatDragLeave(_playerId: number) {
  if (dropTargetId.value === _playerId) {
    dropTargetId.value = null;
  }
}

function onSeatDrop(e: DragEvent, playerId: number) {
  e.preventDefault();
  const cardId = dragCardId.value;
  dropTargetId.value = null;
  dragCardId.value = null;
  if (cardId == null) return;
  if (!selectableSet.value.has(playerId)) return;
  if (selectedSkill.value?.view_as) {
    const action = skillActions(selectedSkill.value.id).find(
      (candidate) =>
        sameTargets(candidate.targets, [playerId]) &&
        candidate.card_targets.length === 1 &&
        candidate.card_targets[0] === cardId
    );
    if (!action) return;
    emit("activateSkill", action.skill, action.targets, action.card_targets);
  } else {
    const action = playActions(cardId).find((candidate) =>
      sameTargets(candidate.targets, [playerId])
    );
    if (!action) return;
    emit("playCard", action.card, action.targets);
  }
  selectedCard.value = null;
  selectedSkill.value = null;
  targetMode.value = false;
  selectedTargets.value = [];
}

function confirmAction() {
  if (!canConfirm.value) return;
  if (selectedSkill.value) {
    const action = skillActions(selectedSkill.value.id).find(
      (candidate) =>
        sameTargets(candidate.targets, selectedTargets.value) &&
        (selectedCard.value == null
          ? candidate.card_targets.length === 0
          : candidate.card_targets.length === 1 &&
            candidate.card_targets[0] === selectedCard.value)
    );
    if (!action) return;
    emit("activateSkill", action.skill, action.targets, action.card_targets);
  } else {
    if (selectedCard.value == null) return;
    const action = playActions(selectedCard.value).find((candidate) =>
      sameTargets(candidate.targets, selectedTargets.value)
    );
    if (!action) return;
    emit("playCard", action.card, action.targets);
  }
  selectedCard.value = null;
  selectedSkill.value = null;
  targetMode.value = false;
  selectedTargets.value = [];
}

function playWithoutTarget() {
  selectedTargets.value = [];
  confirmAction();
}

function cancelSelection() {
  selectedCard.value = null;
  selectedSkill.value = null;
  targetMode.value = false;
  selectedTargets.value = [];
}

function skillLabel(skill: SkillView): string {
  return skill.name ? skillDisplayName(skill.name) : `#${skill.def}`;
}

function suitGlyph(suit: CardView["suit"]): string {
  switch (suit) {
    case "Spade":
      return "♠";
    case "Heart":
      return "♥";
    case "Club":
      return "♣";
    case "Diamond":
      return "♦";
    default:
      return "?";
  }
}

function rankLabel(rank: number | null): string {
  if (rank == null) return "";
  if (rank === 1) return "A";
  if (rank === 11) return "J";
  if (rank === 12) return "Q";
  if (rank === 13) return "K";
  return String(rank);
}

function isPassOrCancelChoice(choice: ChoiceRequest, action: ResolveChoiceAction): boolean {
  if (action.ResolveChoice.cancelled) return true;
  if (action.ResolveChoice.selections.length !== 1) return false;
  const opt = choice.options?.find((item) => item.token === action.ResolveChoice.selections[0]);
  return opt?.label_key === "pass" || opt?.label_key === "no" || opt?.label_key === "cancel";
}

function choiceOptionText(choice: ChoiceRequest, token: number): string {
  const opt = choice.options?.find((item) => item.token === token);
  if (!opt) {
    return choice.native_kind === "confirm" && token === 0 ? "确定" : `选择 ${token}`;
  }
  const key = opt.label_key;
  if (key === "confirm" || key === "yes") return "确定";
  if (key === "no" || key === "cancel") return "取消";
  if (key === "pass") return "不出 / 跳过";
  if (key === "hand") return "手牌";
  if (key.startsWith("card:")) {
    const id = Number(key.slice(5));
    const card = props.view?.known_cards.find((c) => c.id === id);
    if (card) {
      const face = `${suitGlyph(card.suit)}${rankLabel(card.rank)}`;
      return `${face} ${cardKindLabel(card.kind, card.def)}`;
    }
    return Number.isFinite(id) ? `卡牌 #${id}` : "卡牌";
  }
  if (key.startsWith("player:")) {
    return `P${key.slice(7)}`;
  }
  if (key.startsWith("suit:")) {
    const raw = key.slice(5).toLowerCase();
    if (raw === "spade") return "♠ 黑桃";
    if (raw === "heart") return "♥ 红桃";
    if (raw === "club") return "♣ 梅花";
    if (raw === "diamond") return "♦ 方片";
  }
  if (key.startsWith("skill:")) {
    return skillDisplayName(key.slice(6));
  }
  if (key.startsWith("kind:")) {
    return `牌型 #${key.slice(5)}`;
  }
  if (key.startsWith("tag:")) {
    const raw = key.slice(4);
    if (raw === "plus") return "手牌上限 +1";
    if (raw === "minus") return "手牌上限 -1";
    return `选项 #${raw}`;
  }
  return `选项 ${token}`;
}

function choiceButtonLabel(choice: ChoiceRequest, action: ResolveChoiceAction): string {
  if (action.ResolveChoice.cancelled) return "取消";
  const selections = action.ResolveChoice.selections;
  if (selections.length === 0) return "跳过";
  if (selections.length === 1) return choiceOptionText(choice, selections[0]!);
  return selections.map((tok) => choiceOptionText(choice, tok)).join(" + ");
}

const phaseLabelZh = computed(() => {
  switch (props.view?.phase) {
    case "Prepare":
      return "准备阶段";
    case "Judge":
      return "判定阶段";
    case "Draw":
      return "摸牌阶段";
    case "Play":
      return "出牌阶段";
    case "Discard":
      return "弃牌阶段";
    case "End":
      return "结束阶段";
    default:
      return "战局等待";
  }
});

const tabletopTipText = computed(() => {
  if (props.choice) return choicePromptLabel(props.choice);
  if (selectedSkill.value) {
    return `发动【${skillLabel(selectedSkill.value)}】：请选择目标或素材`;
  }
  if (selectedCard.value != null) {
    return canPlayWithoutTarget.value
      ? `已选中手牌 #${selectedCard.value}，点击「直接打出」或选择目标`
      : `已选中手牌 #${selectedCard.value}，请在牌桌上选择目标角色`;
  }
  if (props.view) {
    if (props.view.current_player === props.selfId && props.view.phase === "Play") {
      return props.surfaces?.uiSuites?.shoushaUi?.playPromptText || "出牌阶段，请选择一张牌或结束阶段";
    }
    return `第 ${props.view.round} 轮 · P${props.view.current_player} ${phaseLabelZh.value}`;
  }
  return "等待对局初始化…";
});

const endPhaseDisabled = computed(
  () =>
    Boolean(props.busy) ||
    Boolean(props.finished) ||
    Boolean(props.isSpectator) ||
    (Boolean(props.isOnline) && !props.roomStarted) ||
    !props.canEndPhase,
);
</script>

<template>
  <div
    class="board battle-table battle-felt-surface"
    :class="[
      `theme-decade-${surfaces?.theme?.decadeSkinStyle ?? 'shizhounian'}`,
      `layout-shousha-${surfaces?.theme?.shoushaLayout ?? 'right'}`,
      `theme-rzsh-${surfaces?.theme?.rzshTheme ?? 'reality'}`,
      `theme-qhly-${surfaces?.theme?.qhlyTheme ?? 'decade'}`,
      { 'longxue-shengjian-active': surfaces?.longxue?.shengjian },
    ]"
    role="region"
    aria-label="三国杀真实牌桌战场"
  >
    <!-- Tabletop weather / particles canvas layer -->
    <div
      v-if="surfaces?.coin?.snowFall || (surfaces?.coin?.activeEffects?.length ?? 0) > 0"
      class="tabletop-weather-layer"
      data-testid="tabletop-weather-overlay"
      aria-hidden="true"
    >
      <canvas
        ref="tableWeatherCanvasRef"
        class="tabletop-weather-canvas"
        width="1200"
        height="680"
        data-testid="tabletop-weather-canvas"
      ></canvas>
    </div>

    <BattleShoushaHud
      :view="view"
      :surfaces="surfaces"
      @open-modal="(m) => emit('openModal', m)"
    />

    <!-- Upstream Ten-Year UI #arena stage: perimeter seats + central play/prompt zone + bottom hand dock -->
    <div
      class="arena-stage"
      data-decade-layout="on"
      :data-player-count="orderedSeats.length"
      :data-right-layout="selfOnRight ? 'on' : 'off'"
    >
      <div class="seats" role="list" aria-label="座位列表">
        <div
          v-for="slot in orderedSeats"
          :key="slot.seat.player"
          class="seat-wrapper"
          :data-position="slot.position"
          :data-zone="seatAnchorZone(slot.position, slot.total)"
          :style="seatAnchorStyle(slot.position, slot.total)"
          @dragover="onSeatDragOver($event, slot.seat.player)"
          @dragleave="onSeatDragLeave(slot.seat.player)"
          @drop="onSeatDrop($event, slot.seat.player)"
        >
          <SeatCard
            :seat="slot.seat"
            :self="slot.seat.player === selfId"
            :relative-position="slot.position"
            :identity-label="identityLabel(slot.seat.identity)"
            :is-target-mode="targetMode"
            :is-selectable-target="selectableSet.has(slot.seat.player)"
            :is-drop-target="dropTargetId === slot.seat.player"
            :equip-cards="equipCardsFor(slot.seat.player)"
            :judge-cards="judgmentCardsFor(slot.seat.player)"
            :current-turn="view?.current_player === slot.seat.player"
            :decade-ui="surfaces?.uiSuites?.decadeUi"
            :skin-switch="surfaces?.uiSuites?.skinSwitch"
            :decade-style="surfaces?.theme?.decadeSkinStyle"
            :l20-marks="surfaces?.theme?.l20Marks"
            @select="onTargetSelect"
          />
        </div>
      </div>

      <!-- Central Tabletop Area: kill/broadcast banner + played/discard pile + central prompt & action buttons above hand -->
      <div class="tabletop-center-zone" data-testid="tabletop-center-zone">
        <div class="center-table-cards">
          <transition name="banner-fade">
            <div
              v-if="surfaces?.theme?.shoushaKillBanner"
              class="tabletop-kill-banner"
              data-testid="l14-shousha-banner-text"
            >
              <span class="banner-wing left">⚔</span>
              <span class="banner-text">{{ surfaces.theme.shoushaKillBanner }}</span>
              <span class="banner-wing right">⚔</span>
            </div>
          </transition>

          <section v-if="discardCards().length > 0" class="discard-pile" aria-label="弃牌堆">
            <span class="muted discard-title">弃牌堆 · {{ discardCards().length }}</span>
            <div class="discard-strip">
              <div
                v-for="c in discardCards().slice(-6)"
                :key="c.id"
                class="discard-card"
                :class="`discard-cat-${cardCategoryOf(c.kind, c.def)}`"
                :title="`${cardKindLabel(c.kind, c.def)} #${c.id}`"
              >
                <span class="d-suit" :data-suit="c.suit">{{ suitGlyph(c.suit) }}{{ rankLabel(c.rank) }}</span>
                <span class="d-name">{{ cardKindLabel(c.kind, c.def) }}</span>
              </div>
              <span v-if="discardCards().length > 6" class="muted discard-more">+{{ discardCards().length - 6 }}</span>
            </div>
          </section>
        </div>

        <div class="prompt-and-controls">
          <section v-if="mySkills().length" class="skill-bar" aria-label="可发动技能">
            <span class="skill-bar-label">技能</span>
            <button
              v-for="skill in mySkills()"
              :key="skill.id"
              type="button"
              class="decade-skill-btn"
              :class="{ primary: selectedSkill?.id === skill.id }"
              @click="onSkillSelect(skill)"
            >
              {{ skillLabel(skill) }}{{ skill.view_as ? " · 视为" : "" }}
            </button>
          </section>

          <!-- Pending engine ChoiceRequest prompt & buttons right above hand area -->
          <section
            v-if="choice"
            class="choice tabletop-prompt-box"
            aria-label="待选择"
            data-testid="tabletop-choice-zone"
          >
            <div class="prompt-banner">
              <h2>{{ choicePromptLabel(choice) }}</h2>
              <span class="prompt-meta muted">
                choice #{{ choice.choice_id }} · 玩家 {{ choice.observer }} · {{ choiceKindLabel(choice) }}
              </span>
              <p v-if="choice.deadline_unix_ms != null" class="prompt-timer" role="timer">
                {{ choiceSecondsLeft === null ? "正在校准服务器时间…" : `剩余 ${choiceSecondsLeft} 秒 · 到时托管` }}
              </p>
            </div>
            <TriggerOrderChoice
              v-if="choice.prompt === 'trigger_order'"
              :choice="choice"
              :busy="Boolean(busy)"
              :enabled="Boolean(connected) && !isSpectator && choice.observer === selfId"
              @select="(token) => emit('resolveTriggerOrder', token)"
            />
            <div v-else-if="choiceActions && choiceActions.length" class="dui-controls-row">
              <button
                v-for="(action, index) in choiceActions"
                :key="index"
                type="button"
                class="decade-ctrl-btn"
                :class="{
                  primary: !isPassOrCancelChoice(choice, action),
                  cancel: isPassOrCancelChoice(choice, action),
                }"
                :disabled="Boolean(busy) || !connected || Boolean(isSpectator)"
                @click="emit('resolveChoice', action)"
              >
                {{ choiceButtonLabel(choice, action) }}
              </button>
            </div>
            <p v-else class="muted prompt-wait">当前观察者无权处理此选择。</p>
          </section>

          <!-- Card/skill selection confirmation bar above hand area -->
          <div
            v-else-if="selectedCard != null || selectedSkill"
            class="action-bar tabletop-prompt-box"
            role="toolbar"
            aria-label="行动操作"
          >
            <div class="prompt-banner">
              <span class="action-summary">
                ▸
                <template v-if="selectedSkill">技能【{{ skillLabel(selectedSkill) }}】</template>
                <template v-else>手牌 #{{ selectedCard }}</template>
                <span v-if="selectedSkill?.view_as && selectedCard != null"> · 素材 #{{ selectedCard }}</span>
                <span v-if="targetMode && selectedTargets.length > 0">
                  → 目标 P{{ selectedTargets.join(", P") }}
                </span>
              </span>
            </div>
            <div class="dui-controls-row">
              <button
                class="primary decade-ctrl-btn"
                type="button"
                @click="confirmAction"
                :disabled="!canConfirm"
                aria-label="确认行动"
              >
                {{ selectedSkill ? "发动技能" : "确认出牌" }}
              </button>
              <button
                type="button"
                class="decade-ctrl-btn"
                :disabled="!canPlayWithoutTarget"
                @click="playWithoutTarget"
                aria-label="无目标直接打出"
              >
                {{ selectedSkill ? "无目标发动" : "直接打出" }}
              </button>
              <button
                type="button"
                class="decade-ctrl-btn cancel"
                @click="cancelSelection"
                aria-label="取消选择"
              >
                取消
              </button>
            </div>
          </div>

          <!-- DecadeUI .hand-tip & #dui-controls end-phase row -->
          <div class="hand-tip-bar" data-testid="tabletop-hand-tip-bar">
            <span class="hand-tip-text">{{ tabletopTipText }}</span>
            <button
              v-if="view && connected !== false"
              type="button"
              class="decade-ctrl-btn end-phase-btn"
              data-testid="btn-tabletop-end-phase"
              :disabled="endPhaseDisabled"
              @click="emit('endPhase')"
            >
              结束阶段
            </button>
          </div>
        </div>
      </div>

      <!-- Bottom Hand Card Strip aligned alongside Self seat -->
      <div class="hand-dock" :class="{ 'dock-left': selfOnRight }">
        <HandStrip
          :cards="myHand()"
          :actionable-ids="actionableCardIds"
          @select="onCardSelect"
          @drag-start="onCardDragStart"
          @sort-hand="(ids) => emit('sortHand', ids)"
        />
      </div>
    </div>
  </div>
</template>

<style scoped>
.board {
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 0.65rem;
}
/* Real mahogany wood border & dark felt surface (Ten-Year UI arena) */
.battle-table {
  min-height: 840px;
  scroll-margin-top: 72px;
  border-radius: 14px;
  padding: 0.75rem 0.9rem;
  background: radial-gradient(ellipse at 50% 45%, #23382d 0%, #13221a 60%, #0a120d 100%);
  border: 6px solid #3c2415;
  box-shadow: inset 0 0 56px rgba(0, 0, 0, 0.85), 0 8px 32px rgba(0, 0, 0, 0.65);
}
.battle-table::before {
  content: "";
  position: absolute;
  inset: 2px;
  border: 1px solid rgba(232, 165, 75, 0.25);
  border-radius: 9px;
  pointer-events: none;
}
.battle-table.theme-decade-shousha,
.battle-table.layout-shousha-left {
  background: radial-gradient(ellipse at 50% 48%, #2e251d 0%, #1a140f 65%, #0d0907 100%);
  border-color: #2d1c12;
}
.tabletop-weather-layer {
  position: absolute;
  inset: 0;
  pointer-events: none;
  z-index: 1;
  overflow: hidden;
  border-radius: 10px;
}
.tabletop-weather-canvas {
  width: 100%;
  height: 100%;
  display: block;
}
.tabletop-kill-banner {
  position: relative;
  z-index: 15;
  display: inline-flex;
  align-items: center;
  gap: 0.45rem;
  padding: 0.26rem 1.15rem;
  background: linear-gradient(135deg, rgba(184, 59, 59, 0.95), rgba(92, 20, 20, 0.95));
  border: 1.5px solid #e8a54b;
  border-radius: 26px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.6), 0 0 12px rgba(232, 165, 75, 0.4);
  color: #fff9e6;
  font-weight: 800;
  font-size: 0.88rem;
  letter-spacing: 0.08em;
  text-shadow: 0 2px 4px rgba(0, 0, 0, 0.9);
  pointer-events: none;
}
.banner-wing {
  color: #f3d27a;
}

.arena-stage {
  position: relative;
  flex: 1;
  min-height: 780px;
  width: 100%;
  z-index: 2;
}

/* Desktop & tablet wrap-around perimeter seating (2-8 players) */
@media (min-width: 768px) {
  .seats {
    position: absolute;
    inset: 0;
    pointer-events: none;
    z-index: 5;
  }
  .seat-wrapper {
    position: absolute;
    left: var(--seat-left, auto);
    right: var(--seat-right, auto);
    top: var(--seat-top, auto);
    bottom: var(--seat-bottom, auto);
    transform: translateX(var(--seat-tx, 0%));
    width: clamp(92px, 10.6vw, 134px);
    pointer-events: auto;
    z-index: 5;
  }
  .seat-wrapper[data-position="0"] {
    width: clamp(104px, 11.4vw, 146px);
    z-index: 8;
  }
  .hand-dock {
    position: absolute;
    bottom: 10px;
    left: clamp(128px, 13vw, 174px);
    right: 16px;
    z-index: 7;
    display: flex;
    align-items: flex-end;
  }
  .hand-dock.dock-left {
    left: 16px;
    right: clamp(128px, 13vw, 174px);
  }
  .tabletop-center-zone {
    position: absolute;
    left: clamp(114px, 13vw, 174px);
    right: clamp(114px, 13vw, 174px);
    top: 222px;
    bottom: 226px;
    z-index: 6;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    align-items: center;
    overflow-y: auto;
    pointer-events: none;
  }
  .tabletop-center-zone > * {
    pointer-events: auto;
  }
}

.center-table-cards {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  margin: auto 0;
  gap: 0.45rem;
  max-width: 100%;
}
.discard-pile {
  padding: 0.4rem 0.7rem;
  background: rgba(0, 0, 0, 0.36);
  border: 1px dashed rgba(232, 165, 75, 0.3);
  border-radius: 10px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.3rem;
}
.discard-title {
  font-size: 0.72rem;
}
.discard-strip {
  display: flex;
  gap: 0.3rem;
  align-items: center;
  flex-wrap: wrap;
  justify-content: center;
}
.discard-card {
  min-width: 44px;
  height: 58px;
  padding: 0.18rem 0.25rem;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 0.12rem;
  background: rgba(248, 244, 234, 0.9);
  color: #1c1917;
  border: 1px solid #b89f78;
  border-radius: 5px;
  font-size: 0.68rem;
  font-weight: 700;
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.45);
  opacity: 0.88;
}
.discard-card.discard-cat-trick {
  border-color: #4a8f63;
  background: rgba(235, 248, 240, 0.94);
}
.discard-card.discard-cat-equip {
  border-color: #5b7398;
  background: rgba(235, 242, 252, 0.94);
}
.d-suit[data-suit="Heart"],
.d-suit[data-suit="Diamond"] {
  color: #b91c1c;
}
.d-name {
  font-size: 0.64rem;
}
.discard-more {
  font-size: 0.72rem;
}

.prompt-and-controls {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.45rem;
  width: 100%;
  max-width: 760px;
}

.skill-bar {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.4rem;
  flex-wrap: wrap;
  padding: 0.32rem 0.65rem;
  border-radius: 999px;
  background: rgba(15, 18, 22, 0.76);
  border: 1px solid rgba(232, 165, 75, 0.38);
}
.skill-bar-label {
  font-size: 0.74rem;
  color: #f3d27a;
  font-weight: 700;
}
.decade-skill-btn {
  padding: 0.2rem 0.65rem;
  font-size: 0.78rem;
  border-radius: 999px;
  background: rgba(34, 40, 50, 0.92);
  border: 1px solid rgba(232, 165, 75, 0.45);
  color: #f5ebd4;
}

.tabletop-prompt-box {
  width: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.45rem;
}
.prompt-banner {
  width: 100%;
  padding: 0.48rem 1rem;
  border-radius: 8px;
  background: linear-gradient(90deg, transparent 0%, rgba(12, 14, 18, 0.78) 14%, rgba(12, 14, 18, 0.78) 86%, transparent 100%);
  text-align: center;
  color: #fff6d6;
  text-shadow: 0 1px 4px #000;
}
.prompt-banner h2 {
  margin: 0;
  font-size: 1.05rem;
  font-weight: 700;
  color: #fff7dd;
}
.prompt-meta {
  display: block;
  margin-top: 0.15rem;
  font-size: 0.72rem;
}
.prompt-timer {
  margin: 0.18rem 0 0;
  font-size: 0.76rem;
  color: #fbbf24;
  font-weight: 600;
}
.prompt-wait {
  margin: 0;
  font-size: 0.78rem;
}
.action-summary {
  font-size: 0.88rem;
  font-weight: 600;
  color: #fde68a;
}

.dui-controls-row {
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  align-items: center;
  gap: 0.6rem;
}
.decade-ctrl-btn {
  min-width: 84px;
  padding: 0.4rem 1.05rem;
  border-radius: 999px;
  font-size: 0.88rem;
  font-weight: 700;
  background: linear-gradient(180deg, rgba(46, 52, 62, 0.94), rgba(24, 28, 36, 0.96));
  border: 1.5px solid rgba(232, 165, 75, 0.55);
  color: #fdf6e2;
  box-shadow: 0 3px 10px rgba(0, 0, 0, 0.55);
}
.decade-ctrl-btn.primary {
  background: linear-gradient(180deg, #d89b38, #9c6514);
  border-color: #fde68a;
  color: #fffbeb;
}
.decade-ctrl-btn.cancel {
  background: linear-gradient(180deg, rgba(64, 34, 34, 0.92), rgba(38, 18, 18, 0.95));
  border-color: rgba(248, 113, 113, 0.55);
}

.hand-tip-bar {
  display: flex;
  align-items: center;
  justify-content: center;
  flex-wrap: wrap;
  gap: 0.65rem;
  padding: 0.28rem 0.9rem;
  border-radius: 999px;
  background: linear-gradient(90deg, transparent 0%, rgba(0, 0, 0, 0.48) 15%, rgba(0, 0, 0, 0.48) 85%, transparent 100%);
}
.hand-tip-text {
  font-size: 0.84rem;
  font-weight: 600;
  color: #f8f4d9;
  letter-spacing: 0.04em;
  text-shadow: 0 1px 3px #000;
}
.end-phase-btn {
  padding: 0.28rem 0.9rem;
  font-size: 0.8rem;
  background: linear-gradient(180deg, #7c4a19, #4e2d0d);
  border-color: rgba(232, 165, 75, 0.72);
}

/* Mobile / narrow viewport wrap-around perimeter grid */
@media (max-width: 767px) {
  .board {
    gap: 0.35rem;
  }
  .battle-table {
    min-height: 0;
    scroll-margin-top: 12px;
    padding: 0.42rem;
    border-width: 4px;
  }
  .arena-stage {
    min-height: 0;
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 0.34rem;
    align-items: start;
  }
  .seats {
    display: contents;
  }
  .seat-wrapper {
    order: 1;
    min-width: 0;
  }
  .tabletop-center-zone {
    grid-column: 1 / -1;
    order: 2;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.28rem;
    padding: 0.14rem 0;
  }
  .seat-wrapper[data-position="0"] {
    grid-column: 1 / -1;
    order: 3;
  }
  .hand-dock {
    grid-column: 1 / -1;
    order: 4;
  }
}
</style>
