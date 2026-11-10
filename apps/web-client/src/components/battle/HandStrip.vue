<script setup lang="ts">
import { computed, ref } from "vue";
import type { CardView } from "../../types/engine";

const props = defineProps<{ cards: CardView[]; actionableIds: number[] }>();
const emit = defineEmits<{
  select: [id: number];
  dragStart: [id: number];
  sortHand: [sortedIds: number[]];
}>();

const selectedId = ref<number | null>(null);
const draggingId = ref<number | null>(null);
const handSorted = ref(false);

const SUIT_WEIGHT: Record<string, number> = {
  Spade: 1,
  Heart: 2,
  Club: 3,
  Diamond: 4,
};

const displayCards = computed(() => {
  if (!handSorted.value) return props.cards;
  return [...props.cards].sort((a, b) => {
    const ka = a.kind ?? "Other";
    const kb = b.kind ?? "Other";
    if (ka !== kb) return ka.localeCompare(kb);
    const sa = SUIT_WEIGHT[a.suit ?? ""] ?? 9;
    const sb = SUIT_WEIGHT[b.suit ?? ""] ?? 9;
    if (sa !== sb) return sa - sb;
    const ra = a.rank ?? 99;
    const rb = b.rank ?? 99;
    if (ra !== rb) return ra - rb;
    return a.id - b.id;
  });
});

function toggleHandSort() {
  handSorted.value = !handSorted.value;
  emit(
    "sortHand",
    displayCards.value.map((c) => c.id),
  );
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
        :class="{ selected: c.id === selectedId, dragging: c.id === draggingId }"
        :disabled="!actionableIds.includes(c.id)"
        role="option"
        :aria-selected="c.id === selectedId"
        :aria-label="`卡牌 ${c.id}${c.def != null ? ' 已知' : ' 未知'}`"
        draggable="true"
        @click="onSelect(c.id)"
        @dragstart="onDragStart($event, c.id)"
        @dragend="onDragEnd"
      >
        <span class="suit" :data-suit="c.suit">{{ suitGlyph(c.suit) }}</span>
        <span class="rank">{{ rankLabel(c.rank) }}</span>
        <span class="id muted">#{{ c.id }}</span>
        <span v-if="c.def != null" class="card-known">▸</span>
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
  background: rgba(232, 165, 75, 0.12);
  color: var(--text);
  cursor: pointer;
}
.strip { display: flex; flex-wrap: wrap; gap: 0.45rem; }
.card {
  width: 64px; height: 88px;
  display: flex; flex-direction: column; align-items: center; justify-content: center;
  gap: 0.15rem;
  background: #f4efe6; color: #1a1208;
  border: 1px solid #c9b89a; border-radius: 8px;
  padding: 0.25rem;
  transition: transform 0.1s, box-shadow 0.1s, opacity 0.15s;
  position: relative; cursor: grab;
}
.card:active { cursor: grabbing; }
.card:disabled { cursor: not-allowed; opacity: 0.45; transform: none; }
.card:hover { transform: translateY(-3px); }
.card.selected {
  transform: translateY(-6px);
  border-color: var(--color-accent);
  box-shadow: 0 3px 8px rgba(232, 165, 75, 0.4);
}
.card.dragging {
  opacity: 0.4;
  transform: scale(0.95);
}
.card.selected::after {
  content: "✓"; position: absolute; top: -6px; right: -6px;
  background: var(--color-positive); color: #fff; border-radius: 50%;
  width: 16px; height: 16px; font-size: 10px;
  display: flex; align-items: center; justify-content: center; font-weight: bold;
}
.suit[data-suit="Heart"], .suit[data-suit="Diamond"] { color: #c0392b; }
.rank { font-weight: 700; font-family: var(--mono); }
.id { font-size: 0.65rem; }
.card-known { font-size: 0.7rem; color: var(--color-accent); }
.empty { margin: 0.25rem 0; }
.hint { font-size: 0.8rem; color: var(--color-positive); margin: 0.4rem 0 0; }
.drag-hint { color: var(--color-accent); }
@media (prefers-reduced-motion: reduce) {
  .card:hover, .card.selected, .card.dragging { transform: none; }
}
</style>
