<script setup lang="ts">
import { computed, ref } from "vue";
import type { CardView } from "../../types/engine";
import {
  sortShoushaHandCards,
  type ShoushaSortableCard,
} from "../../engine/surfaces/shoushaUiSurface";

const props = defineProps<{ cards: CardView[]; actionableIds: number[] }>();
const emit = defineEmits<{
  select: [id: number];
  dragStart: [id: number];
  sortHand: [sortedIds: number[]];
}>();

const selectedId = ref<number | null>(null);
const draggingId = ref<number | null>(null);
const handSorted = ref(false);

function cardDisplayName(c: CardView): string {
  return ((c as any).name as string | undefined) ?? c.kind ?? (c.def != null ? `#${c.def}` : "");
}

const displayCards = computed(() => {
  if (!handSorted.value) return props.cards;
  const sortables: ShoushaSortableCard[] = props.cards.map((c) => ({
    id: c.id,
    name: cardDisplayName(c),
    suit: (c.suit ? c.suit.toLowerCase() : "none") as any,
    number: c.rank ?? 99,
    muniu: Boolean((c as any).muniu),
  }));
  const sorted = sortShoushaHandCards(sortables);
  const orderMap = new Map(sorted.map((s, idx) => [s.id, idx]));
  return [...props.cards].sort((a, b) => (orderMap.get(a.id) ?? 0) - (orderMap.get(b.id) ?? 0));
});

function toggleHandSort() {
  handSorted.value = !handSorted.value;
  emit(
    "sortHand",
    displayCards.value.map((c) => c.id),
  );
}

function cardCategory(c: CardView): "basic" | "trick" | "equip" {
  const k = c.kind?.toLowerCase() ?? "";
  if (k.includes("equip")) return "equip";
  if (k.includes("trick")) return "trick";
  return "basic";
}

function suitGlyph(s: CardView["suit"]) {
  switch (s) {
    case "Spade": return "♠";
    case "Heart": return "♥";
    case "Club": return "♣";
    case "Diamond": return "♦";
    default: return "?";
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
  selectedId.value = id;
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
  <section class="hand" aria-label="手牌">
    <div class="hand-header">
      <h2 class="muted title">
        手牌（{{ cards.length }} 张{{ selectedId != null ? " · 已选中 #" + selectedId : " · 点击或拖拽" }}{{ handSorted ? " · 已理牌" : "" }}）
      </h2>
      <button
        type="button"
        class="lipai-btn"
        data-testid="btn-handstrip-lipai"
        @click="toggleHandSort"
      >
        {{ handSorted ? "✓ 手杀已理牌" : "⇄ 理牌" }}
      </button>
    </div>
    <div class="strip" role="listbox" aria-label="手牌列表">
      <button
        v-for="c in displayCards"
        :key="c.id"
        type="button"
        class="card"
        :class="[
          `card-cat-${cardCategory(c)}`,
          { selected: c.id === selectedId, dragging: c.id === draggingId },
        ]"
        :disabled="!actionableIds.includes(c.id)"
        role="option"
        :aria-selected="c.id === selectedId"
        :aria-label="`卡牌 ${c.id}${c.def != null ? ' 已知' : ' 未知'}`"
        draggable="true"
        @click="onSelect(c.id)"
        @dragstart="onDragStart($event, c.id)"
        @dragend="onDragEnd"
      >
        <!-- Top-left corner suit & rank -->
        <div class="card-corner top-left">
          <span class="suit" :data-suit="c.suit">{{ suitGlyph(c.suit) }}</span>
          <span class="rank">{{ rankLabel(c.rank) }}</span>
        </div>

        <div class="card-body">
          <span class="id muted">#{{ c.id }}</span>
          <span v-if="cardDisplayName(c)" class="card-name">{{ cardDisplayName(c) }}</span>
          <span v-else-if="c.def != null" class="card-known">▸</span>
          <span class="card-cat-label">{{ cardCategory(c) === "equip" ? "装备" : cardCategory(c) === "trick" ? "锦囊" : "基本" }}</span>
        </div>

        <!-- Bottom-right corner suit & rank (inverted) -->
        <div class="card-corner bottom-right">
          <span class="suit" :data-suit="c.suit">{{ suitGlyph(c.suit) }}</span>
          <span class="rank">{{ rankLabel(c.rank) }}</span>
        </div>
      </button>
      <p v-if="!cards.length" class="muted empty">无可见手牌</p>
    </div>
    <p v-if="selectedId != null" class="hint">
      ✓ 已选中 #{{ selectedId }} — 在棋盘上选择目标，或点击「结束阶段」直接打出
    </p>
    <p v-if="draggingId != null" class="hint drag-hint">
      🂠 拖拽中 #{{ draggingId }} — 拖到目标玩家释放出牌
    </p>
  </section>
</template>

<style scoped>
.hand-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
  margin-bottom: 0.4rem;
}
.title { font-size: 0.9rem; margin: 0; font-weight: 500; }
.lipai-btn {
  padding: 0.18rem 0.55rem;
  font-size: 0.76rem;
  border-radius: 6px;
  border: 1px solid rgba(232, 165, 75, 0.45);
  background: var(--bg-elevated);
  color: var(--accent);
  cursor: pointer;
}
.lipai-btn:hover {
  border-color: var(--accent-2);
}
.strip {
  display: flex;
  align-items: flex-end;
  padding: 1.2rem 0.5rem 0.5rem;
  overflow-x: auto;
  min-height: 120px;
}
.card {
  position: relative;
  width: 76px;
  height: 108px;
  margin-right: -18px; /* Cascading fan overlap */
  border-radius: 6px;
  padding: 0.35rem;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  background: #fdfbf7;
  color: #1a1a1a;
  border: 1.5px solid #c8baa6;
  box-shadow: -3px 0 8px rgba(0, 0, 0, 0.45);
  cursor: pointer;
  transition: transform 0.15s ease, margin 0.15s ease, z-index 0.15s;
  user-select: none;
  flex-shrink: 0;
}
.card:hover:not(:disabled) {
  transform: translateY(-16px) scale(1.05);
  z-index: 10;
  margin-right: -8px;
}
.card.selected {
  transform: translateY(-24px) scale(1.08);
  box-shadow: 0 0 14px rgba(232, 165, 75, 0.9), -3px 0 10px rgba(0, 0, 0, 0.6);
  border-color: #e8a54b;
  z-index: 15;
}
.card.dragging {
  opacity: 0.5;
  transform: scale(0.95);
}
.card:disabled {
  opacity: 0.45;
  cursor: not-allowed;
  filter: grayscale(0.5);
}
/* Category frames: basic, trick, equip */
.card-cat-basic {
  border-color: #b89370;
  background: linear-gradient(180deg, #fffcf8 0%, #f4ede2 100%);
}
.card-cat-trick {
  border-color: #4a8f63;
  background: linear-gradient(180deg, #f7fdf9 0%, #e5f3eb 100%);
}
.card-cat-equip {
  border-color: #687998;
  background: linear-gradient(180deg, #f7f9fd 0%, #e6edf7 100%);
}
.card-corner {
  display: flex;
  flex-direction: column;
  align-items: center;
  line-height: 1;
}
.card-corner.top-left {
  align-self: flex-start;
}
.card-corner.bottom-right {
  align-self: flex-end;
  transform: rotate(180deg);
}
.suit {
  font-size: 0.95rem;
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
  font-size: 0.85rem;
  font-weight: 800;
}
.card-body {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 0.15rem;
  flex: 1;
}
.card-name {
  font-size: 0.82rem;
  font-weight: 700;
  writing-mode: vertical-rl;
  letter-spacing: 0.1em;
  color: #2b1f14;
}
.card-cat-label {
  font-size: 0.62rem;
  font-weight: 600;
  padding: 0.05rem 0.25rem;
  border-radius: 3px;
  background: rgba(0, 0, 0, 0.08);
  color: #555;
}
.id { font-size: 0.65rem; }
.card-known {
  font-size: 0.75rem;
  color: var(--accent);
}
.empty { padding: 1rem 0; }
.hint {
  font-size: 0.78rem;
  color: var(--color-positive);
  margin-top: 0.4rem;
}
.drag-hint {
  color: var(--accent-2);
}
@media (prefers-reduced-motion: reduce) {
  .card:hover, .card.selected, .card.dragging { transform: none; }
}
</style>
