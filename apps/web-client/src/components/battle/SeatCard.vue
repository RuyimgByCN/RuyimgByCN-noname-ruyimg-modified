<script setup lang="ts">
import { computed, ref, watch } from "vue";
import {
  characterDisplayName,
  characterImageUrl,
  characterPortraitUrl,
  ensureAppearanceCharacter,
  fallbackPortraitUrl,
  selectedSkinId,
  selectSkin,
  skinDisplayName,
  skinsForCharacter,
  substituteImageUrl,
} from "../../engine/appearanceStore";
import type { CardView, SeatView } from "../../types/engine";

const props = defineProps<{
  seat: SeatView;
  self: boolean;
  identityLabel: string;
  isTargetMode: boolean;
  isSelectableTarget: boolean;
  isDropTarget: boolean;
  equipCards: CardView[];
}>();

const emit = defineEmits<{
  select: [playerId: number];
}>();

const displayName = computed(
  () =>
    characterDisplayName(props.seat.character_def) ??
    `#${props.seat.character_def}`
);
const skins = computed(() => skinsForCharacter(props.seat.character_def));
const selectedSkin = computed(() => selectedSkinId(props.seat.character_def));
const selectedPortrait = computed(() =>
  characterImageUrl(props.seat.character_def)
);
const basePortrait = computed(() =>
  characterPortraitUrl(props.seat.character_def)
);
const originalPortrait = computed(() =>
  fallbackPortraitUrl(props.seat.character_def)
);
const portraitCandidates = computed(() =>
  Array.from(
    new Set([
      ...(props.seat.alive
        ? []
        : [substituteImageUrl(props.seat.character_def, "dead")]),
      selectedPortrait.value,
      basePortrait.value,
      originalPortrait.value,
    ])
  ).filter((url): url is string => !!url)
);
const failedAssets = ref<string[]>([]);
const portraitSrc = computed(
  () =>
    portraitCandidates.value.find(
      (url) => !failedAssets.value.includes(url)
    ) ?? ""
);

watch(
  () => props.seat.character_def,
  (characterId) => {
    void ensureAppearanceCharacter(characterId).catch(() => undefined);
  },
  { immediate: true }
);

watch(portraitCandidates, () => {
  failedAssets.value = [];
});

function onPortraitError(): void {
  if (portraitSrc.value) {
    failedAssets.value = [...failedAssets.value, portraitSrc.value];
  }
}

function onSkinChange(event: Event): void {
  const value = (event.target as HTMLSelectElement).value;
  selectSkin(props.seat.character_def, value === "" ? null : Number(value));
}

function hpIcon(hp: number, maxHp: number, alive: boolean): string {
  if (!alive) return "✗";
  if (hp <= 0) return "⚠";
  return "✓";
}

function hpColor(hp: number, maxHp: number, alive: boolean): string {
  if (!alive) return "var(--color-negative)";
  if (hp <= 0) return "var(--color-warning)";
  return "var(--color-positive)";
}
</script>

<template>
  <article
    class="seat has-portrait"
    :class="{
      self,
      dead: !seat.alive,
      dying: seat.alive && seat.hp <= 0,
      'target-mode': isTargetMode,
      'target-selectable': isTargetMode && isSelectableTarget,
      'drop-target': isDropTarget,
      'current-turn': false,
    }"
    role="listitem"
    :aria-label="`座位 ${seat.seat} 玩家 ${seat.player} 武将 ${displayName}`"
    :tabindex="isTargetMode && isSelectableTarget ? 0 : -1"
    @click="isTargetMode && isSelectableTarget && emit('select', seat.player)"
    @keydown.enter="isTargetMode && isSelectableTarget && emit('select', seat.player)"
  >
    <img
      v-if="portraitSrc"
      class="portrait"
      :src="portraitSrc"
      :alt="`${displayName}武将头像`"
      @error="onPortraitError"
    />
    <div v-else class="portrait no-image" role="img" :aria-label="`${displayName}暂无头像`">
      无图
    </div>
    <header>
      <span class="name">{{ displayName }}</span>
      <span v-if="self" class="tag">你</span>
      <span v-if="seat.turned_over" class="tag" aria-label="武将牌背面">翻面</span>
    </header>
    <div class="hp-row">
      <span
        class="state-icon"
        :style="{ color: hpColor(seat.hp, seat.max_hp, seat.alive) }"
        :aria-label="seat.alive ? (seat.hp <= 0 ? '濒死' : '存活') : '阵亡'"
      >{{ hpIcon(seat.hp, seat.max_hp, seat.alive) }}</span>
      <span class="hp" aria-label="体力">
        {{ seat.hp }} / {{ seat.max_hp }}
      </span>
    </div>
    <!-- HP bar -->
    <div class="hp-bar" role="progressbar" :aria-valuenow="Math.max(seat.hp, 0)" :aria-valuemax="seat.max_hp" :aria-label="`体力 ${seat.hp}/${seat.max_hp}`">
      <div
        class="hp-bar-fill"
        :style="{
          width: Math.max(Math.min(seat.hp / seat.max_hp * 100, 100), 0) + '%',
          background: !seat.alive ? 'var(--color-negative)' : seat.hp <= 0 ? 'var(--color-warning)' : 'var(--color-positive)'
        }"
      ></div>
    </div>
    <div class="meta muted">
      <span>手牌 {{ seat.hand_count }}</span>
      <span>{{ identityLabel }}</span>
    </div>
    <select
      v-if="skins.length > 0"
      class="skin-select"
      :value="selectedSkin ?? ''"
      :aria-label="`${displayName}皮肤`"
      @click.stop
      @keydown.stop
      @change="onSkinChange"
    >
      <option value="">默认皮肤</option>
      <option v-for="skin in skins" :key="skin.id" :value="skin.id">
        {{ skinDisplayName(skin) }}{{ skin.presentation.kind === "spine" ? "（静态回退）" : "" }}
      </option>
    </select>
    <div v-if="equipCards.length > 0" class="equip-zone" aria-label="装备区">
      <div
        v-for="card in equipCards"
        :key="card.id"
        class="equip-card"
        :title="`卡牌 #${card.id}${card.def != null ? ' (def:' + card.def + ')' : ''}`"
      >
        <span class="equip-suit">{{
          card.suit === 'Spade' ? '♠' :
          card.suit === 'Heart' ? '♥' :
          card.suit === 'Club' ? '♣' :
          card.suit === 'Diamond' ? '♦' : '?'
        }}</span>
        <span class="equip-id">#{{ card.id }}</span>
      </div>
    </div>
    <div v-if="!seat.alive" class="dead-badge">✗ 阵亡</div>
    <div v-else-if="seat.hp <= 0" class="dying-badge">⚠ 濒死</div>
    <div v-if="isTargetMode && isSelectableTarget" class="target-hint">
      ▸ 点击选择为目标
    </div>
  </article>
</template>

<style scoped>
.seat {
  background: var(--seat);
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 0.7rem 0.8rem;
  position: relative;
  min-height: 96px;
  transition: border-color 0.15s, box-shadow 0.15s;
}
.seat.has-portrait {
  min-height: 112px;
  padding-left: 5.4rem;
}
.portrait {
  position: absolute;
  inset: 0 auto 0 0;
  width: 4.8rem;
  height: 100%;
  object-fit: cover;
  object-position: top center;
  border-radius: 9px 0 0 9px;
  border-right: 1px solid var(--border);
}
.no-image {
  display: grid;
  place-items: center;
  background: var(--bg-elevated);
  color: var(--muted);
  font-size: 0.75rem;
}
.skin-select {
  width: 100%;
  margin-top: 0.35rem;
  padding: 0.12rem 0.25rem;
  color: var(--muted);
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 4px;
  font-size: 0.7rem;
}
.seat.self {
  border-color: var(--accent);
  box-shadow: 0 0 0 1px rgba(232, 165, 75, 0.25);
}
.seat.dead {
  opacity: 0.55;
  filter: grayscale(0.4);
}
.seat.dying {
  border-color: var(--color-warning);
  animation: pulse-warn 1.5s ease-in-out infinite;
}
.equip-zone {
  display: flex;
  flex-wrap: wrap;
  gap: 0.2rem;
  margin-top: 0.35rem;
}
.equip-card {
  display: inline-flex;
  align-items: center;
  gap: 0.15rem;
  padding: 0.1rem 0.3rem;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 4px;
  font-size: 0.7rem;
}
.equip-suit {
  font-weight: bold;
  color: var(--accent-2);
}
.equip-suit:has(+ .equip-id):nth-child(1) {
  color: inherit;
}
.equip-id {
  font-family: var(--mono);
  font-size: 0.65rem;
  color: var(--muted);
}

.seat.target-mode {
  cursor: default;
}
.seat.target-selectable {
  cursor: pointer;
  border-color: var(--color-accent);
  box-shadow: 0 0 0 2px rgba(232, 165, 75, 0.4);
}
.seat.drop-target {
  border-color: var(--color-positive) !important;
  box-shadow: 0 0 0 3px rgba(95, 191, 122, 0.6) !important;
  transform: scale(1.03);
}
.seat.target-selectable:hover {
  border-color: var(--accent);
  transform: translateY(-1px);
}
@keyframes pulse-warn {
  0%, 100% { box-shadow: 0 0 0 1px rgba(232, 197, 75, 0.3); }
  50% { box-shadow: 0 0 0 3px rgba(232, 197, 75, 0.5); }
}
header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.35rem;
}
.name {
  font-weight: 650;
}
.tag {
  font-size: 0.75rem;
  background: var(--accent);
  color: #1a1208;
  border-radius: 999px;
  padding: 0.1rem 0.45rem;
  font-weight: 700;
}
.hp-row {
  display: flex;
  align-items: center;
  gap: 0.4rem;
}
.state-icon {
  font-family: var(--mono);
  font-weight: bold;
  font-size: 1.1rem;
  min-width: 1.2em;
  text-align: center;
}
.hp {
  font-family: var(--mono);
  font-size: 1.05rem;
}
.hp-bar {
  margin-top: 0.3rem;
  height: 5px;
  background: var(--bg-elevated);
  border-radius: 3px;
  overflow: hidden;
}
.hp-bar-fill {
  height: 100%;
  transition: width 0.3s ease, background 0.3s;
  border-radius: 3px;
}
.meta {
  display: flex;
  justify-content: space-between;
  font-size: 0.8rem;
  margin-top: 0.35rem;
}
.dead-badge {
  position: absolute;
  inset: auto 0.5rem 0.45rem auto;
  color: var(--color-negative);
  font-size: 0.75rem;
  font-weight: 700;
}
.dying-badge {
  position: absolute;
  inset: auto 0.5rem 0.45rem auto;
  color: var(--color-warning);
  font-size: 0.75rem;
  font-weight: 700;
}
.target-hint {
  font-size: 0.7rem;
  color: var(--color-accent);
  margin-top: 0.3rem;
  font-weight: 600;
}
@media (prefers-reduced-motion: reduce) {
  .seat.dying {
    animation: none;
  }
  .seat.target-selectable:hover {
    transform: none;
  }
}
</style>
