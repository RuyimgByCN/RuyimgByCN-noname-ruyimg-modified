<script setup lang="ts">
import { computed } from "vue";
import GeneralPortrait from "./GeneralPortrait.vue";
import type { CardView, SeatView } from "../../types/engine";
import type { DecadeUiRuntimeState } from "../../engine/surfaces/decadeUiSurface";
import type { SkinSwitchRuntimeState } from "../../engine/surfaces/skinSwitchSurface";
import {
  cardKindLabel,
  identityBadgeInfo,
} from "../../engine/surfaces/decadeTabletopLayout";

const props = defineProps<{
  seat: SeatView;
  self: boolean;
  identityLabel: string;
  isTargetMode: boolean;
  isSelectableTarget: boolean;
  isDropTarget: boolean;
  equipCards: CardView[];
  judgeCards?: CardView[];
  relativePosition?: number;
  currentTurn?: boolean;
  decadeUi?: DecadeUiRuntimeState;
  skinSwitch?: SkinSwitchRuntimeState;
  decadeStyle?: string;
  l20Marks?: Record<string, number>;
}>();

const emit = defineEmits<{
  select: [playerId: number];
}>();

const slots = computed(() => props.seat.generals ?? [null, null]);
const hasDeputy = computed(() => slots.value[1] !== null);
const legacyDef = computed(() => !props.seat.generals || (slots.value.every((slot) => slot === null) && props.seat.character_def !== 0)
  ? props.seat.character_def : undefined);
const displayName = computed(() =>
  slots.value
    .map((slot, index) => slot?.name || (index === 0 && legacyDef.value !== undefined ? `#${legacyDef.value}` : "暗将／未配置"))
    .join(" / "),
);

const identityBadge = computed(() => identityBadgeInfo(props.seat.identity));
const decadeBadges = computed(() => props.decadeUi?.activeMarkBadgesBySeat[props.seat.player] ?? []);
const hasJiuBuff = computed(() => props.decadeUi?.jiuBuffSeats.includes(props.seat.player) ?? false);
const isChained = computed(() => Boolean(props.seat.linked));
const chukuangOverlay = computed(() => {
  const act = props.skinSwitch?.lastChukuangAction;
  return act && act.seat === props.seat.player ? act : null;
});

const L20_METAS: Record<string, { label: string; icon: string }> = {
  longmark: { label: "龙印", icon: "🐉" },
  bahu: { label: "跋扈", icon: "⚡" },
  feiyan: { label: "飞扬", icon: "🦅" },
  danlie: { label: "胆裂", icon: "💥" },
  fengyin: { label: "封印", icon: "⛓" },
  fuhun: { label: "父魂", icon: "⚔" },
  guimen: { label: "鬼门", icon: "⛩" },
  hejie: { label: "和解", icon: "🤝" },
  huxiao: { label: "虎啸", icon: "🐯" },
  jianying: { label: "渐营", icon: "🏹" },
  jinfan: { label: "锦帆", icon: "⛵" },
  mohua: { label: "魔化", icon: "👿" },
  niepan: { label: "涅槃", icon: "🪽" },
  biaoji: { label: "标记", icon: "✦" },
};

const activeL20Marks = computed(() => {
  if (!props.l20Marks) return [];
  const list: Array<{ key: string; label: string; icon: string; count: number }> = [];
  for (const [key, count] of Object.entries(props.l20Marks)) {
    if (typeof count === "number" && count > 0 && L20_METAS[key]) {
      list.push({ key, label: L20_METAS[key].label, icon: L20_METAS[key].icon, count });
    }
  }
  return list;
});

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

function judgeGlyph(card: CardView): string {
  if (card.kind === "TrickLe" || card.def === 13) return "乐";
  if (card.kind === "TrickBingliang" || card.def === 14) return "兵";
  if (card.kind === "TrickShandian" || card.def === 15) return "⚡";
  if (card.def === 28) return "洪";
  if (card.def === 112) return "界";
  return cardKindLabel(card.kind, card.def).slice(0, 1);
}

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
    class="seat has-portrait decade-seat-frame"
    :class="{
      self,
      dead: !seat.alive,
      dying: seat.alive && seat.hp <= 0,
      'turned-over': seat.turned_over,
      'dual-general': hasDeputy,
      'target-mode': isTargetMode,
      'target-selectable': isTargetMode && isSelectableTarget,
      'drop-target': isDropTarget,
      'current-turn': Boolean(currentTurn),
      'has-jiu-buff': hasJiuBuff,
      'is-chained': isChained,
    }"
    :data-decade-style="decadeStyle ?? decadeUi?.layoutVariant ?? 'shizhounian'"
    :data-position="relativePosition ?? (self ? 0 : seat.seat)"
    role="listitem"
    :aria-label="`座位 ${seat.seat} 玩家 ${seat.player} 武将 ${displayName} ${identityLabel}`"
    :tabindex="isTargetMode && isSelectableTarget ? 0 : -1"
    @click="isTargetMode && isSelectableTarget && emit('select', seat.player)"
    @keydown.enter="isTargetMode && isSelectableTarget && emit('select', seat.player)"
  >
    <!-- Top-right DecadeUI identity corner stamp (主/忠/反/内/猜) -->
    <div
      class="seat-identity-stamp"
      :data-identity-tone="identityBadge.tone"
      :title="identityLabel"
      data-testid="seat-identity-stamp"
    >
      {{ identityBadge.glyph }}
    </div>

    <!-- Diagonal chain bar when linked -->
    <div v-if="isChained" class="seat-chain-bar" aria-hidden="true">
      <span class="chain-link">⛓</span>
      <span class="chain-link">⛓</span>
      <span class="chain-link">⛓</span>
    </div>

    <!-- Turned-over face-down veil -->
    <div v-if="seat.turned_over" class="seat-turned-veil" aria-hidden="true"></div>

    <div class="seat-portrait-outer">
      <!-- Bottom-left hand count plaque anchored to portrait frame corner -->
      <div
        class="seat-card-count"
        :title="`手牌 ${seat.hand_count} 张`"
        data-testid="seat-card-count"
      >
        {{ seat.hand_count }}
      </div>

      <div class="seat-portrait-stage">
        <div class="generals" :class="{ 'has-deputy': hasDeputy }" aria-label="双将">
          <GeneralPortrait
            :general="slots[0]"
            :slot="0"
            :legacy-def="legacyDef"
            :alive="seat.alive"
            :self="self"
          />
          <GeneralPortrait
            :class="{ 'deputy-hidden-slot': !hasDeputy }"
            :general="slots[1]"
            :slot="1"
            :alive="seat.alive"
            :self="self"
          />
        </div>

        <!-- Status tags & chukuang overlay -->
        <header class="seat-overlay-header">
          <div class="seat-tags">
            <span v-if="self" class="tag self-tag">你</span>
            <span v-if="currentTurn" class="tag turn-tag">回合</span>
            <span v-if="seat.turned_over" class="tag" aria-label="武将牌背面">翻面</span>
            <span v-if="isChained" class="tag chain-tag" data-testid="seat-chain-badge">🔗横置</span>
            <span v-if="hasJiuBuff" class="tag jiu-tag" data-testid="seat-jiu-badge">🍶酒态</span>
          </div>
          <span
            v-if="chukuangOverlay"
            class="chukuang-badge"
            data-testid="seat-chukuang-overlay"
            :class="{ 'flip-x': chukuangOverlay.flipX }"
            :style="{ transform: chukuangOverlay.flipX ? 'scaleX(-1) translateY(-6px)' : 'translateY(-6px)' }"
          >⚡{{ chukuangOverlay.action }}</span>
        </header>

        <!-- Per-seat delayed tricks / judgment diamonds -->
        <div
          v-if="judgeCards && judgeCards.length > 0"
          class="seat-judge-marks"
          data-testid="seat-judge-marks"
          aria-label="判定区"
        >
          <span
            v-for="jc in judgeCards"
            :key="jc.id"
            class="seat-judge-diamond"
            :title="`${cardKindLabel(jc.kind, jc.def)} #${jc.id}`"
          >
            <span class="judge-glyph">{{ judgeGlyph(jc) }}</span>
          </span>
        </div>

        <!-- Equip strips overlaid cleanly at bottom of portrait -->
        <div v-if="equipCards.length > 0" class="equip-zone" aria-label="装备区">
          <div
            v-for="card in equipCards"
            :key="card.id"
            class="equip-card"
            :title="`${cardKindLabel(card.kind, card.def)} #${card.id}${card.def != null ? ' (def:' + card.def + ')' : ''}`"
          >
            <span class="equip-suit" :data-suit="card.suit">{{ suitGlyph(card.suit) }}</span>
            <span class="equip-name">{{ cardKindLabel(card.kind, card.def) }}</span>
          </div>
        </div>

        <div v-if="!seat.alive" class="dead-badge">✗ 阵亡 · {{ identityBadge.glyph }}</div>
        <div v-else-if="seat.hp <= 0" class="dying-badge">⚠ 濒死</div>
      </div>
    </div>

    <!-- Bottom DecadeUI HP & meta footer -->
    <div class="seat-footer">
      <div class="hp-row">
        <span
          class="state-icon"
          :style="{ color: hpColor(seat.hp, seat.max_hp, seat.alive) }"
          :aria-label="seat.alive ? (seat.hp <= 0 ? '濒死' : '存活') : '阵亡'"
        >{{ hpIcon(seat.hp, seat.max_hp, seat.alive) }}</span>
        <div
          class="hp-bar"
          role="progressbar"
          :aria-valuenow="Math.max(seat.hp, 0)"
          :aria-valuemax="seat.max_hp"
          :aria-label="`体力 ${seat.hp}/${seat.max_hp}`"
        >
          <div
            class="hp-bar-fill"
            :style="{
              width: Math.max(Math.min((seat.hp / Math.max(1, seat.max_hp)) * 100, 100), 0) + '%',
              background: !seat.alive ? 'var(--color-negative)' : seat.hp <= 0 ? 'var(--color-warning)' : 'var(--color-positive)'
            }"
          ></div>
        </div>
        <span class="hp" aria-label="体力">{{ seat.hp }}/{{ seat.max_hp }}</span>
      </div>

      <div class="meta muted">
        <span>P{{ seat.player }} · 手牌 {{ seat.hand_count }}</span>
        <span>{{ identityLabel }}</span>
      </div>

      <!-- L08 Decade mark badges -->
      <div v-if="decadeBadges.length > 0" class="decade-seat-marks" data-testid="seat-decade-marks">
        <span
          v-for="mk in decadeBadges"
          :key="mk.skillId"
          class="seat-mark-chip"
          :class="{
            'mark-yinyang-yang': mk.badgeText.includes('阳'),
            'mark-yinyang-yin': mk.badgeText.includes('阴'),
            'mark-limited': mk.badgeText.includes('限') || mk.badgeText.includes('✕'),
          }"
          :title="mk.upstreamRef"
        >{{ mk.label }}[{{ mk.badgeText }}]</span>
      </div>

      <!-- L20 14 Mark badges -->
      <div v-if="activeL20Marks.length > 0" class="seat-l20-marks" data-testid="seat-l20-mark-badges">
        <span
          v-for="mk in activeL20Marks"
          :key="mk.key"
          class="l20-mark-badge"
          :class="`mark-${mk.key}`"
          :title="`${mk.label} ×${mk.count}`"
        >
          <span class="mark-icon">{{ mk.icon }}</span>
          <span class="mark-name">{{ mk.label }}</span>
          <span v-if="mk.count > 1" class="mark-count">×{{ mk.count }}</span>
        </span>
      </div>
    </div>

    <div v-if="isTargetMode && isSelectableTarget" class="target-hint">
      ▸ 点击选择目标
    </div>
  </article>
</template>

<style scoped>
.seat.decade-seat-frame {
  position: relative;
  display: flex;
  flex-direction: column;
  width: 100%;
  min-height: 168px;
  padding: 4px;
  border-radius: 9px;
  border: 1.5px solid rgba(182, 146, 86, 0.55);
  background: linear-gradient(180deg, #2b221a 0%, #1b1510 70%, #120e0b 100%);
  box-shadow: 0 4px 14px rgba(0, 0, 0, 0.65), inset 0 0 0 1px rgba(255, 224, 150, 0.08);
  transition: border-color 0.16s ease, box-shadow 0.16s ease, transform 0.16s ease;
  overflow: visible;
}
.seat.self {
  border-color: rgba(232, 165, 75, 0.9);
  box-shadow: 0 0 14px rgba(232, 165, 75, 0.28), 0 6px 16px rgba(0, 0, 0, 0.72);
}
.seat.current-turn {
  border-color: #6ee7a0;
  box-shadow: 0 0 0 2px rgba(95, 220, 140, 0.45), 0 0 18px rgba(95, 220, 140, 0.35);
}
.seat.dead {
  opacity: 0.62;
  filter: grayscale(0.75);
}
.seat.dying {
  border-color: var(--color-warning);
  animation: pulse-warn 1.5s ease-in-out infinite;
}
.seat.target-mode {
  cursor: default;
}
.seat.target-selectable {
  cursor: pointer;
  border-color: #ef4444;
  box-shadow: 0 0 0 2px rgba(239, 68, 68, 0.65), 0 0 18px rgba(239, 68, 68, 0.45);
}
.seat.target-selectable:hover {
  transform: translateY(-2px);
  border-color: #f87171;
}
.seat.drop-target {
  border-color: var(--color-positive) !important;
  box-shadow: 0 0 0 3px rgba(95, 191, 122, 0.7) !important;
  transform: scale(1.03);
}
@keyframes pulse-warn {
  0%, 100% { box-shadow: 0 0 0 1px rgba(232, 197, 75, 0.35); }
  50% { box-shadow: 0 0 0 3px rgba(232, 197, 75, 0.65); }
}

/* Upstream Ten-Year UI top-right identity badge */
.seat-identity-stamp {
  position: absolute;
  top: -9px;
  right: -9px;
  z-index: 12;
  width: 28px;
  height: 28px;
  border-radius: 7px;
  display: grid;
  place-items: center;
  font-weight: 900;
  font-size: 0.96rem;
  line-height: 1;
  border: 1.5px solid rgba(255, 255, 255, 0.45);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.75);
  user-select: none;
}
.seat-identity-stamp[data-identity-tone="lord"] {
  background: radial-gradient(circle at 35% 30%, #ef4444, #8b1a1a 80%);
  color: #fff6d6;
  border-color: #fbbf24;
  box-shadow: 0 0 12px rgba(239, 68, 68, 0.75);
}
.seat-identity-stamp[data-identity-tone="loyalist"] {
  background: radial-gradient(circle at 35% 30%, #f59e0b, #92400e 80%);
  color: #fffbeb;
  border-color: #fde68a;
  box-shadow: 0 0 10px rgba(245, 158, 11, 0.65);
}
.seat-identity-stamp[data-identity-tone="rebel"] {
  background: radial-gradient(circle at 35% 30%, #22c55e, #14532d 80%);
  color: #ecfdf5;
  border-color: #86efac;
  box-shadow: 0 0 10px rgba(34, 197, 94, 0.65);
}
.seat-identity-stamp[data-identity-tone="renegade"] {
  background: radial-gradient(circle at 35% 30%, #6366f1, #312e81 80%);
  color: #eef2ff;
  border-color: #a5b4fc;
  box-shadow: 0 0 10px rgba(99, 102, 241, 0.65);
}
.seat-identity-stamp[data-identity-tone="unknown"] {
  background: radial-gradient(circle at 35% 30%, #38bdf8, #0c4a6e 80%);
  color: #f0f9ff;
  border-color: #7dd3fc;
  box-shadow: 0 0 10px rgba(56, 189, 248, 0.6);
}

.seat-portrait-outer {
  position: relative;
}

/* Upstream Ten-Year UI bottom-left hand count badge */
.seat-card-count {
  position: absolute;
  left: -8px;
  bottom: 4px;
  z-index: 12;
  min-width: 24px;
  height: 22px;
  padding: 0 5px;
  border-radius: 4px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(180deg, #355e2b, #1d3617);
  border: 1px solid #d4af5a;
  color: #fffbeb;
  font-family: var(--mono);
  font-size: 0.78rem;
  font-weight: 800;
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.8);
  pointer-events: none;
}
/* Match upstream DecadeUI `player_new.css:1079`: hide self hand-count plaque since self cards are shown in HandStrip */
.seat.self .seat-card-count {
  visibility: hidden;
}

.seat-portrait-stage {
  position: relative;
  height: 118px;
  border-radius: 6px;
  overflow: hidden;
  background: #14110f;
}
.generals {
  display: grid;
  grid-template-columns: 1fr;
  width: 100%;
  height: 100%;
}
.generals.has-deputy {
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 2px;
}
/* Keeps deputy slot in DOM for dual-general contract while presenting single-general frames like DecadeUI */
.deputy-hidden-slot {
  position: absolute !important;
  width: 1px !important;
  height: 1px !important;
  min-height: 0 !important;
  overflow: hidden !important;
  opacity: 0 !important;
  pointer-events: none !important;
}

.seat-overlay-header {
  position: absolute;
  bottom: 4px;
  right: 4px;
  left: 24px;
  z-index: 4;
  display: flex;
  justify-content: flex-end;
  align-items: flex-end;
  gap: 0.2rem;
  pointer-events: none;
}
.seat-tags {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 0.18rem;
}
.tag {
  font-size: 0.62rem;
  background: rgba(232, 165, 75, 0.92);
  color: #1a1208;
  border-radius: 999px;
  padding: 0.05rem 0.34rem;
  font-weight: 700;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.7);
}
.turn-tag {
  background: #34d399;
  color: #052e16;
}
.chain-tag {
  background: #4d78a8;
  color: #fff;
}
.jiu-tag {
  background: #b93829;
  color: #fff;
}
.seat.has-jiu-buff {
  border-color: #d93829 !important;
  animation: jiu-glow 1.4s ease-in-out infinite alternate;
}
@keyframes jiu-glow {
  0% { box-shadow: inset 0 0 10px rgba(220, 20, 60, 0.4), 0 0 6px rgba(185, 40, 25, 0.3); }
  100% { box-shadow: inset 0 0 20px rgba(220, 20, 60, 0.75), 0 0 16px rgba(220, 40, 40, 0.6); }
}

.seat-turned-veil {
  position: absolute;
  inset: 0;
  z-index: 6;
  border-radius: 8px;
  background: repeating-linear-gradient(
    -45deg,
    rgba(20, 24, 34, 0.55),
    rgba(20, 24, 34, 0.55) 6px,
    rgba(55, 65, 85, 0.35) 6px,
    rgba(55, 65, 85, 0.35) 12px
  );
  pointer-events: none;
}

.seat-chain-bar {
  position: absolute;
  top: 42%;
  left: -4%;
  right: -4%;
  height: 18px;
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 6px;
  transform: rotate(-12deg);
  background: linear-gradient(90deg, rgba(77, 120, 168, 0.1), rgba(77, 120, 168, 0.85) 50%, rgba(77, 120, 168, 0.1));
  border-top: 1px solid #7eb6ff;
  border-bottom: 1px solid #3a5c88;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.6);
  pointer-events: none;
  z-index: 8;
}
.chain-link {
  font-size: 0.88rem;
  color: #d8e8ff;
  text-shadow: 0 0 4px #4d78a8;
}
.chukuang-badge {
  display: inline-block;
  padding: 0.1rem 0.36rem;
  border-radius: 4px;
  font-size: 0.66rem;
  font-weight: 800;
  background: linear-gradient(135deg, rgba(232, 165, 75, 0.85), rgba(200, 100, 30, 0.85));
  border: 1px solid #f3d27a;
  color: #fff9e6;
  box-shadow: 0 2px 8px rgba(232, 165, 75, 0.6);
  animation: chukuang-leap 0.4s ease-out;
  pointer-events: none;
}
@keyframes chukuang-leap {
  0% { transform: scale(0.8) translateY(0); opacity: 0; }
  50% { transform: scale(1.15) translateY(-8px); opacity: 1; }
  100% { transform: translateY(-6px); opacity: 1; }
}

.seat-judge-marks {
  position: absolute;
  right: 4px;
  bottom: 24px;
  display: flex;
  gap: 4px;
  z-index: 9;
}
.seat-judge-diamond {
  width: 20px;
  height: 20px;
  display: grid;
  place-items: center;
  background: radial-gradient(circle, #f3c146, #b67b14);
  border: 1px solid #fff3c4;
  border-radius: 4px;
  transform: rotate(45deg);
  box-shadow: 0 2px 5px rgba(0, 0, 0, 0.85);
}
.judge-glyph {
  transform: rotate(-45deg);
  font-size: 0.68rem;
  font-weight: 800;
  color: #1f1407;
  line-height: 1;
}

.equip-zone {
  position: absolute;
  left: 3px;
  right: 3px;
  bottom: 3px;
  display: flex;
  flex-direction: column;
  gap: 2px;
  z-index: 5;
  pointer-events: none;
}
.equip-card {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  padding: 0.08rem 0.3rem;
  background: rgba(14, 12, 10, 0.82);
  border-left: 2px solid #d4af5a;
  border-radius: 3px;
  font-size: 0.66rem;
  color: #f3ead8;
  line-height: 1.15;
}
.equip-suit {
  font-weight: 700;
}
.equip-suit[data-suit="Heart"],
.equip-suit[data-suit="Diamond"] {
  color: #f87171;
}
.equip-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.dead-badge {
  position: absolute;
  inset: 0;
  display: grid;
  place-items: center;
  background: rgba(10, 10, 12, 0.62);
  color: #fca5a5;
  font-size: 0.86rem;
  font-weight: 800;
  letter-spacing: 0.06em;
  text-shadow: 0 2px 6px #000;
  z-index: 10;
}
.dying-badge {
  position: absolute;
  inset: auto 4px 4px auto;
  padding: 0.08rem 0.34rem;
  border-radius: 4px;
  background: rgba(120, 40, 10, 0.88);
  color: #fde68a;
  font-size: 0.68rem;
  font-weight: 700;
  z-index: 10;
}

.seat-footer {
  display: flex;
  flex-direction: column;
  gap: 0.18rem;
  padding: 0.28rem 0.24rem 0.1rem;
}
.hp-row {
  display: flex;
  align-items: center;
  gap: 0.28rem;
}
.state-icon {
  font-family: var(--mono);
  font-weight: bold;
  font-size: 0.78rem;
  min-width: 1em;
  text-align: center;
}
.hp-bar {
  flex: 1;
  height: 5px;
  background: rgba(0, 0, 0, 0.55);
  border-radius: 3px;
  overflow: hidden;
}
.hp-bar-fill {
  height: 100%;
  transition: width 0.3s ease, background 0.3s;
  border-radius: 3px;
}
.hp {
  font-family: var(--mono);
  font-size: 0.74rem;
  font-weight: 700;
  color: #efe4ca;
}
.meta {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 0.28rem;
  font-size: 0.68rem;
  color: #b7ad98;
  white-space: nowrap;
}
.target-hint {
  font-size: 0.66rem;
  color: #fca5a5;
  text-align: center;
  font-weight: 700;
  padding-top: 0.12rem;
}

.decade-seat-marks {
  display: flex;
  flex-wrap: wrap;
  gap: 0.16rem;
}
.seat-mark-chip {
  padding: 0.05rem 0.28rem;
  border-radius: 4px;
  font-size: 0.62rem;
  background: rgba(232, 165, 75, 0.18);
  border: 1px solid rgba(232, 165, 75, 0.45);
  color: #f3d27a;
}
.seat-mark-chip.mark-yinyang-yang {
  background: rgba(232, 165, 75, 0.28);
  border-color: #f3d27a;
  color: #ffe6a0;
}
.seat-mark-chip.mark-yinyang-yin {
  background: rgba(140, 60, 180, 0.28);
  border-color: #c078f0;
  color: #e8c8ff;
}
.seat-mark-chip.mark-limited {
  background: rgba(185, 40, 25, 0.28);
  border-color: #e05a5a;
  color: #ffb0b0;
  font-weight: 700;
}
.seat-l20-marks {
  display: flex;
  flex-wrap: wrap;
  gap: 0.16rem;
}
.l20-mark-badge {
  display: inline-flex;
  align-items: center;
  gap: 0.12rem;
  padding: 0.05rem 0.26rem;
  border-radius: 4px;
  font-size: 0.62rem;
  font-weight: 600;
  background: rgba(40, 100, 160, 0.28);
  border: 1px solid rgba(91, 159, 212, 0.45);
  color: #b0d8ff;
}
.mark-icon { font-size: 0.7rem; }
.mark-count {
  font-size: 0.6rem;
  font-family: var(--mono);
  color: #f3d27a;
}

@media (min-width: 768px) and (max-width: 1120px) {
  .seat.decade-seat-frame {
    min-height: 146px;
    padding: 4px;
  }
  .seat-portrait-stage {
    height: 102px;
  }
  .meta {
    font-size: 0.62rem;
    gap: 0.2rem;
  }
}

@media (max-width: 767px) {
  .seat.decade-seat-frame {
    min-height: 102px;
    padding: 3px;
  }
  .seat-portrait-stage,
  .seat.self .seat-portrait-stage {
    height: 72px;
  }
  .seat-footer {
    gap: 0.14rem;
    padding: 0.16rem 0.14rem 0.06rem;
  }
  .meta {
    font-size: 0.64rem;
    gap: 0.25rem;
  }
  .seat-identity-stamp {
    width: 22px;
    height: 22px;
    top: -6px;
    right: -6px;
    font-size: 0.78rem;
  }
  .seat-card-count {
    left: -5px;
    bottom: 2px;
    min-width: 20px;
    height: 18px;
    font-size: 0.68rem;
  }
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
