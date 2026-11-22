<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type { CardView } from "../../types/engine";
import type { DecadeUiConfig } from "../../engine/surfaces/decadeConfigSchema";
import type { DecadeUiRuntimeState } from "../../engine/surfaces/decadeUiSurface";
import { buildCardAuxOverlay } from "../../engine/surfaces/decadeHooksRuntime";
import { sortShoushaHandCards, type ShoushaSortableCard } from "../../engine/surfaces/shoushaUiSurface";
import {
  cardCategoryLabel,
  cardCategoryOf,
  cardKindLabel,
} from "../../engine/surfaces/decadeTabletopLayout";

const props = defineProps<{
  cards: CardView[];
  actionableIds: number[];
  decadeConfig?: DecadeUiConfig;
  decadeUi?: DecadeUiRuntimeState;
  cardFoldEnabled?: boolean;
  cardFoldProportion?: 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10;
  cardPhantomEnabled?: boolean;
}>();
const effectiveCfg = computed(() => props.decadeConfig ?? props.decadeUi?.config);
const specialHelTag = computed(() => {
  const cfg = effectiveCfg.value;
  if (!cfg) return null;
  const preset = cfg._connectMode ? cfg.cardHelasisy2 : cfg.cardHelasisy;
  if (!preset || preset === "off") return null;
  return preset;
});
const animDisabled = computed(() => effectiveCfg.value?.gameAnimationEffect === false);
const emit = defineEmits<{
  select: [id: number];
  dragStart: [id: number];
  sortHand: [sortedIds: number[]];
}>();

const selectedId = ref<number | null>(null);
const draggingId = ref<number | null>(null);
const handSorted = ref(false);

watch(
  () => props.cards,
  (nextCards) => {
    if (selectedId.value != null && !nextCards.some((c) => c.id === selectedId.value)) {
      selectedId.value = null;
    }
  },
);

function cardDisplayName(c: CardView): string {
  const custom = (c as { name?: string }).name;
  if (custom) return custom;
  return cardKindLabel(c.kind, c.def);
}

const displayCards = computed(() => {
  if (!handSorted.value) return props.cards;
  const sortables: ShoushaSortableCard[] = props.cards.map((c) => ({
    id: c.id,
    name: cardDisplayName(c),
    suit: (c.suit ? c.suit.toLowerCase() : "none") as ShoushaSortableCard["suit"],
    number: c.rank ?? 99,
    muniu: Boolean((c as { muniu?: boolean }).muniu),
  }));
  const sorted = sortShoushaHandCards(sortables);
  const orderMap = new Map(sorted.map((s, idx) => [s.id, idx]));
  return [...props.cards].sort((a, b) => (orderMap.get(a.id) ?? 0) - (orderMap.get(b.id) ?? 0));
});

const selectedCardObj = computed(() =>
  selectedId.value == null
    ? null
    : (displayCards.value.find((c) => c.id === selectedId.value) ?? null),
);

const overlapPx = computed(() => {
  if (!props.cardFoldEnabled) {
    const pct = (props.cardFoldProportion ?? 10) / 100;
    return `-${Math.round(92 * pct * 0.22)}px`;
  }
  if (displayCards.value.length <= 5) return "-14px";
  if (displayCards.value.length <= 8) return "-18px";
  return "-24px";
});

function toggleHandSort() {
  handSorted.value = !handSorted.value;
  emit(
    "sortHand",
    displayCards.value.map((c) => c.id),
  );
}

function cardCategory(c: CardView): "basic" | "trick" | "equip" {
  return cardCategoryOf(c.kind, c.def);
}

function suitGlyph(s: CardView["suit"]) {
  switch (s) {
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

function rankLabel(r: number | null) {
  if (r == null) return "?";
  if (r === 1) return "A";
  if (r === 11) return "J";
  if (r === 12) return "Q";
  if (r === 13) return "K";
  return String(r);
}

function onSelect(id: number) {
  if (!props.actionableIds.includes(id)) return;
  selectedId.value = selectedId.value === id ? null : id;
  emit("select", id);
}

function onDragStart(e: DragEvent, id: number) {
  if (!props.actionableIds.includes(id)) {
    e.preventDefault();
    return;
  }
  draggingId.value = id;
  if (e.dataTransfer) {
    e.dataTransfer.setData("text/plain", String(id));
    e.dataTransfer.effectAllowed = "move";
  }
  emit("dragStart", id);
}

function onDragEnd() {
  draggingId.value = null;
}
</script>

<template>
  <section
    class="hand decade-hand-wrap"
    :data-l10-card-fold="cardFoldEnabled ? 'on' : 'off'"
    :data-l10-fold-proportion="cardFoldProportion ?? 10"
    aria-label="手牌"
  >
    <div class="hand-header">
      <div class="hand-meta">
        <span class="title">
          手牌 <strong>{{ cards.length }}</strong> 张
        </span>
        <span v-if="selectedId != null" class="hint-chip">
          ✓ 已选中 #{{ selectedId }}（选目标或直接打出）
        </span>
        <span v-else-if="draggingId != null" class="hint-chip drag-hint">
          🂠 拖拽 #{{ draggingId }} 到目标座位释放
        </span>
        <span v-else class="muted subtle-tip">点击或拖拽出牌</span>
      </div>
      <button
        type="button"
        class="lipai-btn"
        data-testid="btn-handstrip-lipai"
        @click="toggleHandSort"
      >
        {{ handSorted ? "✓ 手杀已理牌" : "⇄ 理牌" }}
      </button>
    </div>

    <div
      class="strip"
      :class="{ 'no-card-anims': animDisabled }"
      role="listbox"
      aria-label="手牌列表"
      :style="{ '--card-overlap': overlapPx }"
    >
      <button
        v-for="c in displayCards"
        :key="c.id"
        type="button"
        class="card"
        :class="[
          `card-cat-${cardCategory(c)}`,
          {
            selected: c.id === selectedId,
            dragging: c.id === draggingId,
            'is-virtual': c.def == null && c.suit != null,
          },
        ]"
        :disabled="!actionableIds.includes(c.id)"
        :data-kind="c.kind"
        role="option"
        :aria-selected="c.id === selectedId"
        :aria-label="`卡牌 ${cardDisplayName(c)} #${c.id}${c.def != null ? ' 已知' : ' 未知'}`"
        draggable="true"
        @click="onSelect(c.id)"
        @dragstart="onDragStart($event, c.id)"
        @dragend="onDragEnd"
      >
        <div class="card-corner top-left">
          <span class="suit" :data-suit="c.suit">{{ suitGlyph(c.suit) }}</span>
          <span class="rank">{{ rankLabel(c.rank) }}</span>
        </div>

        <div class="card-body">
          <span
            v-if="cardDisplayName(c)"
            class="card-name"
            :class="{ 'card-name-ascii': cardDisplayName(c).startsWith('#') }"
          >{{ cardDisplayName(c) }}</span>
          <span v-else-if="c.def != null" class="card-known">▸</span>
          <span class="card-cat-label">{{ cardCategoryLabel(c.kind, c.def) }}</span>
          <span
            v-if="effectiveCfg?.cardAlternateNameVisible"
            class="card-aux-overlay"
            data-testid="card-aux-overlay"
          >{{ buildCardAuxOverlay(c.kind, c.def) }}</span>
          <span v-if="specialHelTag" class="card-hel-tag" data-testid="card-helasisy-tag">{{ specialHelTag }}</span>
          <span class="id muted">#{{ c.id }}</span>
        </div>

        <div class="card-corner bottom-right">
          <span class="suit" :data-suit="c.suit">{{ suitGlyph(c.suit) }}</span>
          <span class="rank">{{ rankLabel(c.rank) }}</span>
        </div>
      </button>
      <div
        v-if="cardPhantomEnabled && selectedCardObj"
        class="card phantom l10-card-phantom"
        :class="`card-cat-${cardCategory(selectedCardObj)}`"
        style="margin-left: 14px; margin-right: 0"
        data-testid="l10-card-phantom"
        aria-hidden="true"
      >
        <div class="card-corner top-left">
          <span class="suit" :data-suit="selectedCardObj.suit">{{ suitGlyph(selectedCardObj.suit) }}</span>
          <span class="rank">{{ rankLabel(selectedCardObj.rank) }}</span>
        </div>
        <div class="card-body">
          <span class="card-name">{{ cardDisplayName(selectedCardObj) }}</span>
          <span class="l10-phantom-tag">幻影出牌 #{{ selectedCardObj.id }}</span>
        </div>
      </div>
      <p v-if="!cards.length" class="muted empty">无可见手牌</p>
    </div>
  </section>
</template>

<style scoped>
.decade-hand-wrap {
  display: flex;
  flex-direction: column;
  justify-content: flex-end;
  min-width: 0;
  width: 100%;
  padding: 0.4rem 0.65rem 0.45rem;
  border-radius: 10px;
  border: 1px solid rgba(232, 165, 75, 0.22);
  background: linear-gradient(180deg, rgba(16, 18, 20, 0.42) 0%, rgba(10, 11, 13, 0.78) 100%);
  box-shadow: inset 0 1px 0 rgba(255, 224, 150, 0.08);
}
.hand-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
  margin-bottom: 0.15rem;
}
.hand-meta {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 0.45rem;
  min-width: 0;
}
.title {
  font-size: 0.8rem;
  color: #e6dac0;
  font-weight: 600;
}
.title strong {
  color: #f3d27a;
  font-family: var(--mono);
}
.subtle-tip {
  font-size: 0.72rem;
}
.hint-chip {
  font-size: 0.72rem;
  color: #86efac;
  font-weight: 600;
}
.drag-hint {
  color: #93c5fd;
}
.lipai-btn {
  padding: 0.22rem 0.65rem;
  font-size: 0.75rem;
  font-weight: 600;
  border-radius: 6px;
  border: 1px solid rgba(232, 165, 75, 0.55);
  background: linear-gradient(180deg, rgba(46, 34, 22, 0.92), rgba(28, 20, 13, 0.92));
  color: #f3d27a;
  cursor: pointer;
  white-space: nowrap;
}
.lipai-btn:hover {
  border-color: #fde68a;
  background: linear-gradient(180deg, rgba(66, 48, 30, 0.96), rgba(40, 28, 18, 0.96));
}
.strip {
  display: flex;
  align-items: flex-end;
  padding: 26px 0.9rem 6px 0.25rem;
  overflow-x: auto;
  overflow-y: hidden;
  min-height: 162px;
}
.card {
  position: relative;
  width: 92px;
  height: 128px;
  margin-right: var(--card-overlap, -12px);
  border-radius: 7px;
  padding: 0.32rem 0.35rem;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  background: #fdfbf7;
  color: #1a1a1a;
  border: 1.5px solid #c8baa6;
  box-shadow: -3px 2px 10px rgba(0, 0, 0, 0.55);
  cursor: pointer;
  transition: transform 0.15s ease, margin 0.15s ease, z-index 0.15s, box-shadow 0.15s ease;
  user-select: none;
  flex-shrink: 0;
}
.card:last-of-type {
  margin-right: 0;
}
.card:hover:not(:disabled) {
  transform: translateY(-16px) scale(1.04);
  z-index: 10;
  margin-right: 4px;
  box-shadow: 0 8px 18px rgba(0, 0, 0, 0.65);
}
.card.selected {
  transform: translateY(-22px) scale(1.06);
  box-shadow: 0 0 16px rgba(232, 165, 75, 0.95), -3px 2px 12px rgba(0, 0, 0, 0.7);
  border-color: #e8a54b;
  z-index: 15;
}
.card.dragging {
  opacity: 0.5;
  transform: scale(0.95);
}
.strip.no-card-anims .card {
  transition: none !important;
}
.card-hel-tag {
  font-size: 0.56rem;
  padding: 0 0.22rem;
  border-radius: 3px;
  background: rgba(147, 51, 234, 0.16);
  color: #7e22ce;
}
.card-aux-overlay {
  font-size: 0.55rem;
  font-weight: 700;
  padding: 0 0.22rem;
  border-radius: 3px;
  background: rgba(180, 83, 9, 0.16);
  color: #78350f;
}
.card:disabled {
  cursor: not-allowed;
  filter: brightness(0.68) saturate(0.72) grayscale(0.28);
  box-shadow: -2px 1px 6px rgba(0, 0, 0, 0.45);
}
.card-cat-basic {
  border-color: #b89370;
  background: linear-gradient(180deg, #fffcf6 0%, #f1e6d5 100%);
}
.card-cat-trick {
  border-color: #4a8f63;
  background: linear-gradient(180deg, #f7fdf9 0%, #dff0e6 100%);
}
.card-cat-equip {
  border-color: #5b7398;
  background: linear-gradient(180deg, #f7f9fd 0%, #e1eaf6 100%);
}
.card-corner {
  display: flex;
  flex-direction: column;
  align-items: center;
  line-height: 0.95;
}
.card-corner.top-left {
  align-self: flex-start;
}
.card-corner.bottom-right {
  align-self: flex-end;
  transform: rotate(180deg);
}
.suit {
  font-size: 0.92rem;
  font-weight: 700;
}
.suit[data-suit="Heart"],
.suit[data-suit="Diamond"] {
  color: #c0392b;
}
.suit[data-suit="Spade"],
.suit[data-suit="Club"] {
  color: #1a1a1a;
}
.rank {
  font-family: var(--mono);
  font-size: 0.84rem;
  font-weight: 800;
}
.card-body {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 0.14rem;
  flex: 1;
}
.card-name {
  font-size: 0.88rem;
  font-weight: 800;
  writing-mode: vertical-rl;
  text-orientation: upright;
  letter-spacing: 0.08em;
  color: #24190f;
  line-height: 1.05;
  max-height: 76px;
}
.card-name.card-name-ascii {
  writing-mode: horizontal-tb;
  font-family: var(--mono);
  font-size: 0.78rem;
  letter-spacing: 0.02em;
}
.card-cat-label {
  font-size: 0.58rem;
  font-weight: 700;
  padding: 0.04rem 0.24rem;
  border-radius: 3px;
  background: rgba(0, 0, 0, 0.08);
  color: #4a4238;
}
.id {
  font-size: 0.58rem;
  opacity: 0.72;
}
.card-known {
  font-size: 0.75rem;
  color: var(--accent);
}
.empty {
  padding: 1rem 0;
  font-size: 0.82rem;
}

@media (max-width: 767px) {
  .decade-hand-wrap {
    padding: 0.3rem 0.42rem;
  }
  .strip {
    min-height: 118px;
    padding: 18px 0.45rem 4px 0.12rem;
    overflow-y: hidden;
  }
  .card {
    width: 68px;
    height: 94px;
    padding: 0.22rem 0.26rem;
    margin-right: -12px;
  }
  .card-name {
    font-size: 0.68rem;
    max-height: 48px;
    letter-spacing: 0.03em;
  }
  .id,
  .subtle-tip {
    display: none;
  }
}

@media (prefers-reduced-motion: reduce) {
  .card:hover,
  .card.selected,
  .card.dragging {
    transform: none;
  }
}
</style>
