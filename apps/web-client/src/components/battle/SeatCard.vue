<script setup lang="ts">
import { computed } from "vue";
import GeneralPortrait from "./GeneralPortrait.vue";
import type { CardView, SeatView } from "../../types/engine";
import type { DecadeUiRuntimeState } from "../../engine/surfaces/decadeUiSurface";
import type { SkinSwitchRuntimeState } from "../../engine/surfaces/skinSwitchSurface";

const props = defineProps<{
  seat: SeatView;
  self: boolean;
  identityLabel: string;
  isTargetMode: boolean;
  isSelectableTarget: boolean;
  isDropTarget: boolean;
  equipCards: CardView[];
  currentTurn?: boolean;
  decadeUi?: DecadeUiRuntimeState;
  skinSwitch?: SkinSwitchRuntimeState;
  decadeStyle?: string;
}>();

const emit = defineEmits<{
  select: [playerId: number];
}>();

const slots = computed(() => props.seat.generals ?? [null, null]);
const legacyDef = computed(() => !props.seat.generals || (slots.value.every((slot) => slot === null) && props.seat.character_def !== 0)
  ? props.seat.character_def : undefined);
const displayName = computed(() => slots.value.map((slot, index) => slot?.name || (index === 0 && legacyDef.value !== undefined ? `#${legacyDef.value}` : "暗将／未配置")).join(" / "));

const decadeBadges = computed(() => props.decadeUi?.activeMarkBadgesBySeat[props.seat.player] ?? []);
const hasJiuBuff = computed(() => props.decadeUi?.jiuBuffSeats.includes(props.seat.player) ?? false);
const isChained = computed(() => Boolean(props.seat.linked));
const chukuangOverlay = computed(() => {
  const act = props.skinSwitch?.lastChukuangAction;
  return act && act.seat === props.seat.player ? act : null;
});

function hpIcon(hp: number, _maxHp: number, alive: boolean): string {
  if (!alive) return "✗";
  if (hp <= 0) return "⚠";
  return "✓";
}

function hpColor(hp: number, _maxHp: number, alive: boolean): string {
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
      'current-turn': Boolean(currentTurn),
    }"
    :data-decade-style="decadeStyle ?? decadeUi?.layoutVariant ?? 'shizhounian'"
    role="listitem"
    :aria-label="`座位 ${seat.seat} 玩家 ${seat.player} 武将 ${displayName}`"
    :tabindex="isTargetMode && isSelectableTarget ? 0 : -1"
    @click="isTargetMode && isSelectableTarget && emit('select', seat.player)"
    @keydown.enter="isTargetMode && isSelectableTarget && emit('select', seat.player)"
  >
    <header>
      <div class="seat-tags">
        <span v-if="self" class="tag">你</span>
        <span v-if="seat.turned_over" class="tag" aria-label="武将牌背面">翻面</span>
        <span v-if="isChained" class="tag chain-tag" data-testid="seat-chain-badge">🔗横置</span>
        <span v-if="hasJiuBuff" class="tag jiu-tag" data-testid="seat-jiu-badge">🍶酒态</span>
      </div>
      <span
        v-if="chukuangOverlay"
        class="chukuang-badge"
        data-testid="seat-chukuang-overlay"
        :style="{ transform: chukuangOverlay.flipX ? 'scaleX(-1)' : 'none' }"
      >⚡{{ chukuangOverlay.action }}</span>
    </header>
    <div class="generals" aria-label="双将">
      <GeneralPortrait v-for="(general, index) in slots" :key="index" :general="general" :slot="index" :legacy-def="index === 0 ? legacyDef : undefined" :alive="seat.alive" />
    </div>
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
    <div v-if="decadeBadges.length > 0" class="decade-seat-marks" data-testid="seat-decade-marks">
      <span
        v-for="mk in decadeBadges"
        :key="mk.skillId"
        class="seat-mark-chip"
        :title="mk.upstreamRef"
      >{{ mk.label }}[{{ mk.badgeText }}]</span>
    </div>
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
.generals { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 0.5rem; margin-bottom: 0.5rem; }
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
.seat.current-turn {
  border-color: #f3d27a;
  box-shadow: 0 0 0 2px rgba(243, 210, 122, 0.35);
}
.seat-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 0.25rem;
}
.chain-tag {
  background: #4d78a8;
  color: #fff;
}
.jiu-tag {
  background: #b93829;
  color: #fff;
}
.chukuang-badge {
  display: inline-block;
  padding: 0.1rem 0.4rem;
  border-radius: 4px;
  font-size: 0.7rem;
  font-weight: 700;
  background: rgba(232, 165, 75, 0.22);
  border: 1px solid var(--accent);
  color: #f3d27a;
}
.decade-seat-marks {
  display: flex;
  flex-wrap: wrap;
  gap: 0.2rem;
  margin-top: 0.3rem;
}
.seat-mark-chip {
  padding: 0.08rem 0.35rem;
  border-radius: 4px;
  font-size: 0.68rem;
  background: rgba(232, 165, 75, 0.15);
  border: 1px solid rgba(232, 165, 75, 0.4);
  color: #f3d27a;
}
</style>
