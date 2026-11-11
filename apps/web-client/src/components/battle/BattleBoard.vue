<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from "vue";
import type {
  CardView,
  LegalAction,
  ObserverSnapshot,
  SeatView,
  SkillView,
} from "../../types/engine";
import SeatCard from "./SeatCard.vue";
import HandStrip from "./HandStrip.vue";
import BattleShoushaHud, { type BattleModalId } from "./BattleShoushaHud.vue";
import type { ExtensionSurfacesSnapshot } from "../../engine/extensionSurfaceRuntime";
import { renderCoinCanvasParticles } from "../../engine/extensionSurfaceRuntime";

const props = defineProps<{
  view: ObserverSnapshot | null;
  selfId: number;
  legalActions: LegalAction[];
  surfaces?: ExtensionSurfacesSnapshot;
}>();

const emit = defineEmits<{
  playCard: [cardId: number, targets: number[]];
  activateSkill: [skillId: number, targets: number[], cardTargets: number[]];
  openModal: [modal: BattleModalId];
  sortHand: [sortedIds: number[]];
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

function seats(): SeatView[] {
  return props.view?.seats ?? [];
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
    (c) => c.zone === "DiscardPile" && c.def != null
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

// Drag-and-drop: drop a card onto a seat.
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
  // Reset selection.
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
</script>

<template>
  <div
    class="board battle-table battle-felt-surface"
    :class="[
      `theme-decade-${surfaces?.theme?.decadeSkinStyle ?? 'shizhounian'}`,
      `layout-shousha-${surfaces?.theme?.shoushaLayout ?? 'left'}`,
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

    <!-- Kill / broadcast ribbon on the tabletop -->
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

    <BattleShoushaHud
      :view="view"
      :surfaces="surfaces"
      @open-modal="(m) => emit('openModal', m)"
    />
    <!-- Action bar when a card is selected -->
    <div v-if="selectedCard != null || selectedSkill" class="action-bar panel" role="toolbar" aria-label="行动操作">
      <span class="muted">
        ▸
        <template v-if="selectedSkill">技能 #{{ selectedSkill.def }}</template>
        <template v-else>手牌 #{{ selectedCard }}</template>
        <span v-if="selectedSkill?.view_as && selectedCard != null"> · 素材 #{{ selectedCard }}</span>
        <span v-if="targetMode && selectedTargets.length > 0">
          → 目标 P{{ selectedTargets[0] }}
        </span>
      </span>
      <div class="row">
        <button
          class="primary"
          type="button"
          @click="confirmAction"
          :disabled="!canConfirm"
          aria-label="确认行动"
        >
          {{ selectedSkill ? "发动技能" : "确认出牌" }}
        </button>
        <button type="button" :disabled="!canPlayWithoutTarget" @click="playWithoutTarget" aria-label="无目标直接打出">
          {{ selectedSkill ? "无目标发动" : "直接打出" }}
        </button>
        <button type="button" @click="cancelSelection" aria-label="取消选择">
          取消
        </button>
      </div>
    </div>

    <section v-if="mySkills().length" class="skill-bar panel" aria-label="可发动技能">
      <span class="muted">技能</span>
      <button
        v-for="skill in mySkills()"
        :key="skill.id"
        type="button"
        :class="{ primary: selectedSkill?.id === skill.id }"
        @click="onSkillSelect(skill)"
      >
        #{{ skill.def }}{{ skill.view_as ? " · 视为" : "" }}
      </button>
    </section>

    <div class="seats" role="list" aria-label="座位列表">
      <div
        v-for="s in seats()"
        :key="s.player"
        class="seat-wrapper"
        @dragover="onSeatDragOver($event, s.player)"
        @dragleave="onSeatDragLeave(s.player)"
        @drop="onSeatDrop($event, s.player)"
      >
        <SeatCard
          :seat="s"
          :self="s.player === selfId"
          :identity-label="identityLabel(s.identity)"
          :is-target-mode="targetMode"
          :is-selectable-target="selectableSet.has(s.player)"
          :is-drop-target="dropTargetId === s.player"
          :equip-cards="equipCardsFor(s.player)"
          :current-turn="view?.current_player === s.player"
          :decade-ui="surfaces?.uiSuites?.decadeUi"
          :skin-switch="surfaces?.uiSuites?.skinSwitch"
          :decade-style="surfaces?.theme?.decadeSkinStyle"
          :l20-marks="surfaces?.theme?.l20Marks"
          @select="onTargetSelect"
        />
      </div>
    </div>
    <HandStrip
      :cards="myHand()"
      :actionable-ids="actionableCardIds"
      @select="onCardSelect"
      @drag-start="onCardDragStart"
      @sort-hand="(ids) => emit('sortHand', ids)"
    />

    <!-- Judgment zone (delayed tricks: Le/Bingliang/Shandian) -->
    <section v-if="seats().some(s => judgmentCardsFor(s.player).length > 0)" class="judgment-zone" aria-label="判定区">
      <h3 class="muted jz-title">⚖ 判定区</h3>
      <div class="jz-entries">
        <div v-for="s in seats()" :key="'jz-' + s.player" class="jz-entry">
          <template v-for="c in judgmentCardsFor(s.player)" :key="c.id">
            <span class="jz-card" :title="`P${s.player} #${c.id}`">
              <span class="jz-suit">{{
                c.suit === 'Spade' ? '♠' : c.suit === 'Heart' ? '♥' :
                c.suit === 'Club' ? '♣' : c.suit === 'Diamond' ? '♦' : '?'
              }}</span>
              <span class="jz-player">P{{ s.player }}</span>
            </span>
          </template>
        </div>
      </div>
    </section>

    <!-- Discard pile -->
    <section v-if="discardCards().length > 0" class="discard-pile" aria-label="弃牌堆">
      <h3 class="muted discard-title">弃牌堆（{{ discardCards().length }}）</h3>
      <div class="discard-strip">
        <div
          v-for="c in discardCards().slice(-10)"
          :key="c.id"
          class="discard-card"
          :title="`#${c.id} def:${c.def}`"
        >
          <span class="d-suit">{{
            c.suit === 'Spade' ? '♠' :
            c.suit === 'Heart' ? '♥' :
            c.suit === 'Club' ? '♣' :
            c.suit === 'Diamond' ? '♦' : '?'
          }}</span>
        </div>
        <span v-if="discardCards().length > 10" class="muted discard-more">+{{ discardCards().length - 10 }}</span>
      </div>
    </section>
  </div>
</template>

<style scoped>
.board {
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}
/* Real mahogany wood border & dark green felt surface */
.battle-table {
  min-height: 720px;
  border-radius: 16px;
  padding: 1.1rem 1.3rem;
  background: radial-gradient(ellipse at 50% 45%, #183e2b 0%, #0d261a 65%, #08160f 100%);
  border: 8px solid #3c2415;
  box-shadow: inset 0 0 50px rgba(0, 0, 0, 0.85), 0 8px 32px rgba(0, 0, 0, 0.65);
}
.battle-table::before {
  content: "";
  position: absolute;
  inset: 2px;
  border: 1px solid rgba(232, 165, 75, 0.25);
  border-radius: 10px;
  pointer-events: none;
}
/* Handkill / Shousha theme variant */
.battle-table.layout-shousha-right,
.battle-table.theme-decade-shousha {
  background: radial-gradient(ellipse at 50% 50%, #2a2017 0%, #17110b 70%, #0c0805 100%);
  border-color: #2b1a11;
}
/* Full-tabletop dynamic weather & particles canvas */
.tabletop-weather-layer {
  position: absolute;
  inset: 0;
  pointer-events: none;
  z-index: 1;
  overflow: hidden;
  border-radius: 12px;
}
.tabletop-weather-canvas {
  width: 100%;
  height: 100%;
  display: block;
}
/* Center kill / broadcast banner */
.tabletop-kill-banner {
  position: absolute;
  top: 64px;
  left: 50%;
  transform: translateX(-50%);
  z-index: 20;
  display: flex;
  align-items: center;
  gap: 0.6rem;
  padding: 0.4rem 1.6rem;
  background: linear-gradient(135deg, rgba(184, 59, 59, 0.95), rgba(92, 20, 20, 0.95));
  border: 2px solid #e8a54b;
  border-radius: 30px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.6), 0 0 15px rgba(232, 165, 75, 0.5);
  color: #fff9e6;
  font-weight: 800;
  font-size: 1.05rem;
  letter-spacing: 0.08em;
  text-shadow: 0 2px 4px rgba(0, 0, 0, 0.9);
  animation: banner-glide 0.35s ease-out;
}
.banner-wing {
  color: #f3d27a;
  font-size: 1.15rem;
}
@keyframes banner-glide {
  0% { opacity: 0; transform: translate(-50%, -18px) scale(0.92); }
  100% { opacity: 1; transform: translate(-50%, 0) scale(1); }
}
.action-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.6rem 1rem;
  border: 1px solid var(--color-accent);
  background: var(--bg-elevated);
}
.skill-bar {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex-wrap: wrap;
  padding: 0.6rem 0.75rem;
}
.seats {
  position: relative;
  z-index: 2;
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
  gap: 0.65rem;
}
.discard-pile {
  position: relative;
  z-index: 2;
  padding: 0.5rem 0.75rem;
  background: rgba(0, 0, 0, 0.45);
  border: 1px dashed rgba(232, 165, 75, 0.35);
  border-radius: 8px;
  box-shadow: inset 0 2px 8px rgba(0, 0, 0, 0.6);
}
.judgment-zone {
  position: relative;
  z-index: 2;
  padding: 0.5rem 0.75rem;
  background: rgba(14, 40, 60, 0.45);
  border: 1px dashed rgba(91, 159, 212, 0.4);
  border-radius: 8px;
  box-shadow: inset 0 2px 8px rgba(0, 0, 0, 0.6);
}
.jz-title { font-size: 0.82rem; margin: 0 0 0.3rem; }
.jz-entries { display: flex; gap: 0.4rem; flex-wrap: wrap; }
.jz-entry { display: inline-flex; gap: 0.15rem; }
.jz-card {
  display: inline-flex; align-items: center; gap: 0.2rem;
  padding: 0.15rem 0.4rem;
  background: var(--bg-elevated);
  border: 1px solid rgba(91, 159, 212, 0.3);
  border-radius: 4px;
  font-size: 0.72rem;
}
.jz-suit { font-weight: bold; color: var(--accent-2); }
.jz-player { color: var(--muted); font-family: var(--mono); }
.discard-title { font-size: 0.82rem; margin: 0 0 0.3rem; }
.discard-strip { display: flex; gap: 0.2rem; align-items: center; }
.discard-card {
  width: 28px; height: 36px;
  display: flex; align-items: center; justify-content: center;
  background: rgba(255,255,255,0.05);
  border: 1px solid var(--border);
  border-radius: 4px;
  font-size: 0.8rem;
  opacity: 0.7;
}
.discard-more { font-size: 0.75rem; margin-left: 0.2rem; }
</style>
