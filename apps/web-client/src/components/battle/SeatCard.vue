<script setup lang="ts">
import { computed, ref } from "vue";
import GeneralPortrait from "./GeneralPortrait.vue";
import type { CardView, SeatView } from "../../types/engine";
import {
  DECADE_PLAYER_NODE_KEYS,
  type DecadePlayerNodeKey,
  type DecadeUiConfig,
} from "../../engine/surfaces/decadeConfigSchema";
import { DECADE_NOSKILL_CONTROL_IDS } from "../../engine/surfaces/decadeMarkSkillsData";
import {
  deriveGeneralRarityTier,
  resolveDecadeSeatBorderTier,
} from "../../engine/surfaces/decadeHooksRuntime";
import type { DecadeUiRuntimeState } from "../../engine/surfaces/decadeUiSurface";
import type { L20RuntimeState } from "../../engine/surfaces/l20MarkSupplementRuntime";
import type { QhlyRuntimeState } from "../../engine/surfaces/qhlySurface";
import { resolveShoushaGuanjie } from "../../engine/surfaces/shoushaSurfacesRuntime";
import type { ShoushaUiRuntimeState } from "../../engine/surfaces/shoushaUiSurface";
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
  decadeConfig?: DecadeUiConfig;
  shoushaUi?: ShoushaUiRuntimeState;
  l20Ui?: L20RuntimeState;
  qhlyUi?: QhlyRuntimeState;
  skinSwitch?: SkinSwitchRuntimeState;
  decadeStyle?: string;
  l20Marks?: Record<string, number>;
}>();

const emit = defineEmits<{
  select: [playerId: number];
  openDetail: [seat: SeatView];
  cycleQhlySkin: [seat: SeatView];
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
const seatGuanjie = computed(() => {
  const totalSeats = Math.max(2, Object.keys(props.decadeUi?.handCountBadgeBySeat ?? {}).length || 4);
  return resolveShoushaGuanjie(props.seat, totalSeats);
});
const seatPromptText = computed(
  () =>
    props.shoushaUi?.seatPromptBySeat?.[props.seat.player] ??
    props.shoushaUi?.waitingSeats?.[props.seat.player] ??
    null,
);
const seatSayBubble = computed(
  () => props.shoushaUi?.chatBubbleBySeat?.[props.seat.player] ?? null,
);
const seatPauseGuess = computed(() =>
  !props.seat.identity ? (props.decadeUi?.pauseGuessBySeat?.[props.seat.player] ?? null) : null,
);
const isL20PlayerJiu = computed(() => props.l20Ui?.jiuAuraSeats?.includes(props.seat.player) ?? false);
const l20GroupHalo = computed(() => props.l20Ui?.guozhanFormationsBySeat?.[props.seat.player] ?? null);
const l20WeaponStrike = computed(() => props.l20Ui?.weaponStrikeBySeat?.[props.seat.player] ?? null);
const l20BaojiLabel = computed(() => props.l20Ui?.baojiPopupBySeat?.[props.seat.player] ?? null);
const l20CardFx = computed(() => props.l20Ui?.cardFxCueBySeat?.[props.seat.player] ?? null);
const l20RankSweep = computed(() => (props.l20Ui?.rankSweepBySeat?.[props.seat.player] ?? 0) > 0);
const qhlySkinBtnVisible = computed(() => Boolean(props.qhlyUi?.config?.qhly_skinButton));
const qhlySkinBtnPos = computed(() => {
  const gen = props.seat.generals?.[0]?.name ?? `seat_${props.seat.player}`;
  const skin = props.qhlyUi?.replaceSkinByGeneral?.[gen] ?? "origin";
  const key = `qhly_dragButtonPositionOf_${gen}_${skin}`;
  const pos =
    props.qhlyUi?.config?.qhly_dragButtonPosition === "no"
      ? (props.qhlyUi?.dragButtonPositionByKey?.[key] ?? props.qhlyUi?.dragButtonPositionAll)
      : props.qhlyUi?.dragButtonPositionAll;
  return {
    left: `${pos?.leftPx ?? 6}px`,
    top: `${pos?.topPx ?? 6}px`,
  };
});
const decadeBadges = computed(() => props.decadeUi?.activeMarkBadgesBySeat[props.seat.player] ?? []);
const hasJiuBuff = computed(() => props.decadeUi?.jiuBuffSeats.includes(props.seat.player) ?? false);
const isChained = computed(() => Boolean(props.seat.linked));
const chukuangOverlay = computed(() => {
  const act = props.skinSwitch?.lastChukuangAction;
  return act && act.seat === props.seat.player ? act : null;
});
const effectiveConfig = computed(() => props.decadeConfig ?? props.decadeUi?.config);
const campTextMode = computed(() => effectiveConfig.value?.campIdentityImageMode === false);
const rarityLongMode = computed<"on" | "off" | "xinon" | null>(() => {
  if (effectiveConfig.value?.newDecadeStyle !== "off") return null;
  const r = effectiveConfig.value?.rarityLong;
  if (r === "on" || r === "off" || r === "xinon") return r;
  return null;
});
const noDieGrayscale = computed(() => effectiveConfig.value?.playerDieEffect === false);
const hasHealAura = computed(
  () =>
    Boolean(effectiveConfig.value?.EffectWood) &&
    Boolean(props.decadeUi?.healAuraSeats?.includes(props.seat.player)),
);
const cardUseCue = computed(() =>
  effectiveConfig.value?.cardUseEffect
    ? props.decadeUi?.cardUseCueBySeat?.[props.seat.player]
    : undefined,
);
const detachSelfEquips = computed(
  () =>
    props.self &&
    effectiveConfig.value?.newDecadeStyle === "on" &&
    Boolean(effectiveConfig.value?.aloneEquip),
);
const visibleEquips = computed(() => (detachSelfEquips.value ? [] : props.equipCards));
const NOSKILL_CONTROL_SET: ReadonlySet<string> = new Set(DECADE_NOSKILL_CONTROL_IDS);
const extraGainedSkills = computed(() => {
  if (!effectiveConfig.value?.gainSkillsShown) return [];
  return props.seat.skills
    .slice(2)
    .filter((s) => Boolean(s.name) && !NOSKILL_CONTROL_SET.has(s.name ?? ""))
    .slice(0, 3);
});
const enabledNodeKeys = computed(
  () => new Set<DecadePlayerNodeKey>(effectiveConfig.value?.node ?? DECADE_PLAYER_NODE_KEYS),
);
function isNodeEnabled(key: DecadePlayerNodeKey): boolean {
  return enabledNodeKeys.value.has(key);
}
const gainStickerSkill = computed(() => {
  const mode = effectiveConfig.value?.gainSkillsVisible ?? "off";
  if (mode === "off") return null;
  if (mode === "othersOn" && props.self) return null;
  const candidate = extraGainedSkills.value[0] ?? props.seat.skills.find((s) => Boolean(s.name));
  return candidate?.name ?? null;
});

const seatBorderTier = computed(() => {
  if (!effectiveConfig.value) return undefined;
  const rarity = props.decadeUi?.rarityTierBySeat?.[props.seat.player] ?? deriveGeneralRarityTier(props.seat);
  return resolveDecadeSeatBorderTier(effectiveConfig.value, rarity);
});
const hasJiuwo = computed(() => props.decadeUi?.jiuwoDyingSeats?.includes(props.seat.player) ?? false);
const hasLoseHp = computed(() => props.decadeUi?.loseHpSeats?.includes(props.seat.player) ?? false);
const hasDamageFlash = computed(() => props.decadeUi?.damageFlashSeats?.includes(props.seat.player) ?? false);
const hasKillFire = computed(() => props.decadeUi?.killFireSeats?.includes(props.seat.player) ?? false);
const hasTiesuoYing = computed(() => props.decadeUi?.tiesuoSeats?.includes(props.seat.player) ?? false);
const isHiddenVeil = computed(() => props.decadeUi?.hiddenSeats?.includes(props.seat.player) ?? false);
const isPublicHand = computed(() => props.decadeUi?.publicHandSeats?.includes(props.seat.player) ?? false);
const isEquipMuted = computed(() => Boolean(props.decadeUi?.equipHiddenBySeat?.[props.seat.player]));
const factionGlow = computed(() => props.decadeUi?.factionGlowBySeat?.[props.seat.player] ?? null);
const hpDelta = computed(() => props.decadeUi?.hpDeltaBySeat?.[props.seat.player] ?? null);
const virtualEquips = computed(() => props.decadeUi?.virtualEquipsBySeat?.[props.seat.player] ?? []);
const recastCount = computed(() => props.decadeUi?.recastCountBySeat?.[props.seat.player] ?? 0);
const publicHandDisplay = computed(() => props.decadeUi?.handCountBadgeBySeat?.[props.seat.player] ?? props.seat.hand_count);
const selectedMarkId = ref<string | null>(null);
const selectedMarkDetail = computed(
  () => decadeBadges.value.find((m) => m.skillId === selectedMarkId.value) ?? null,
);
function toggleMarkTooltip(skillId: string): void {
  selectedMarkId.value = selectedMarkId.value === skillId ? null : skillId;
}

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
      'has-heal-aura': hasHealAura,
      'has-card-use-cue': Boolean(cardUseCue),
      'no-die-gray': noDieGrayscale,
      'has-jiuwo-sos': hasJiuwo,
      'has-lose-hp': hasLoseHp,
      'has-damage-flash': hasDamageFlash,
      'has-kill-fire': hasKillFire,
      'has-faction-glow': Boolean(factionGlow),
      'is-hidden-veil': isHiddenVeil,
      'equip-muted': isEquipMuted,
      'has-l20-playerjiu': isL20PlayerJiu,
      'has-l20-guozhan-halo': Boolean(l20GroupHalo),
      'has-l20-rank-sweep': l20RankSweep,
    }"
    :data-l20-group="l20GroupHalo ?? undefined"
    :data-decade-style="decadeStyle ?? decadeUi?.layoutVariant ?? 'shizhounian'"
    :data-position="relativePosition ?? (self ? 0 : seat.seat)"
    :data-border-level="seatBorderTier"
    :data-glow-faction="factionGlow ?? undefined"
    :data-decade-node-keys="enabledNodeKeys.size"
    role="listitem"
    :aria-label="`座位 ${seat.seat} 玩家 ${seat.player} 武将 ${displayName} ${identityLabel}`"
    :tabindex="isTargetMode && isSelectableTarget ? 0 : -1"
    @click="isTargetMode && isSelectableTarget && emit('select', seat.player)"
    @keydown.enter="isTargetMode && isSelectableTarget && emit('select', seat.player)"
  >
    <div
      class="seat-framebg-layer"
      data-node-key="framebg"
      :data-node-enabled="isNodeEnabled('framebg') ? 'on' : 'off'"
      v-show="isNodeEnabled('framebg')"
      aria-hidden="true"
    ></div>

    <!-- Top-right DecadeUI identity corner stamp (主/忠/反/内/猜, respects campIdentityImageMode) -->
    <div
      v-show="isNodeEnabled('identity')"
      class="seat-identity-stamp"
      :class="{ 'camp-text-only': campTextMode }"
      :data-identity-tone="identityBadge.tone"
      :data-camp-mode="campTextMode ? 'text' : 'image'"
      :title="identityLabel"
      data-node-key="identity"
      data-testid="seat-identity-stamp"
    >
      {{ identityBadge.glyph }}
    </div>

    <!-- Diagonal chain bar when linked -->
    <div v-if="isChained && isNodeEnabled('chain')" class="seat-chain-bar" data-node-key="chain" aria-hidden="true">
      <span class="chain-link">⛓</span>
      <span class="chain-link">⛓</span>
      <span class="chain-link">⛓</span>
    </div>
    <span v-else v-show="isNodeEnabled('chain')" class="seat-node-anchor" data-node-key="chain" aria-hidden="true"></span>

    <!-- Turned-over or guozhan hidden veil (`_gzyinni`) -->
    <div
      v-if="(seat.turned_over || isHiddenVeil) && isNodeEnabled('turnedover')"
      class="seat-turned-veil"
      :class="{ 'is-guozhan-hidden': isHiddenVeil }"
      data-node-key="turnedover"
      aria-hidden="true"
    ></div>
    <span v-else v-show="isNodeEnabled('turnedover')" class="seat-node-anchor" data-node-key="turnedover" aria-hidden="true"></span>

    <div class="seat-portrait-outer">
      <!-- Bottom-left hand count plaque anchored to portrait frame corner (`_gshoupai`) -->
      <div
        v-show="isNodeEnabled('count')"
        class="seat-card-count"
        :data-public-hand="isPublicHand ? 'on' : 'off'"
        data-node-key="count"
        :title="`手牌 ${publicHandDisplay} 张`"
        data-testid="seat-card-count"
      >
        {{ publicHandDisplay }}
      </div>

      <!-- `_player_hp` floating delta popup (anchored absolutely, zero flow impact) -->
      <span
        v-if="hpDelta"
        class="hp-delta-pop"
        :class="hpDelta.kind === 'recover' ? 'is-recover' : 'is-damage'"
        data-testid="seat-hp-delta"
      >{{ hpDelta.delta > 0 ? `+${hpDelta.delta}` : String(hpDelta.delta) }}</span>

      <div class="seat-portrait-stage">
        <div class="generals" :class="{ 'has-deputy': hasDeputy }" aria-label="双将">
          <GeneralPortrait
            v-show="isNodeEnabled('avatar')"
            data-node-key="avatar"
            :general="slots[0]"
            :slot="0"
            :legacy-def="legacyDef"
            :alive="seat.alive"
            :self="self"
            :dynamic-skin-enabled="Boolean(effectiveConfig?.dynamicSkin)"
            :dynamic-skin-outcrop="Boolean(effectiveConfig?.dynamicSkinOutcrop)"
            :outcrop-skin="effectiveConfig?.outcropSkin"
          />
          <GeneralPortrait
            v-show="isNodeEnabled('avatar2')"
            data-node-key="avatar2"
            :class="{ 'deputy-hidden-slot': !hasDeputy }"
            :general="slots[1]"
            :slot="1"
            :alive="seat.alive"
            :self="self"
            :dynamic-skin-enabled="Boolean(effectiveConfig?.dynamicSkin)"
            :dynamic-skin-outcrop="Boolean(effectiveConfig?.dynamicSkinOutcrop)"
            :outcrop-skin="effectiveConfig?.outcropSkin"
          />
        </div>

        <!-- Status tags & chukuang overlay -->
        <header class="seat-overlay-header">
          <div class="seat-tags">
            <span v-if="self" class="tag self-tag">你</span>
            <span v-if="effectiveConfig?.JBDDZ" class="tag jbddz-seat-num" data-testid="seat-jbddz-num">[{{ seat.seat + 1 }}号]</span>
            <button
              type="button"
              class="shousha-guanjie-badge"
              data-testid="seat-guanjie-badge"
              title="点击查看手杀武将详情与ZLLT露头"
              @click.stop="emit('openDetail', seat)"
            >
              {{ seatGuanjie.label }}
            </button>
            <span v-if="l20BaojiLabel" class="seat-l20-baoji-popup" data-testid="seat-l20-baoji">{{ l20BaojiLabel }}</span>
            <span v-if="l20WeaponStrike && l20WeaponStrike !== 'no'" class="seat-l20-weapon-badge" data-testid="seat-l20-weapon">⚔{{ l20WeaponStrike }}</span>
            <span v-if="l20CardFx" class="seat-l20-card-fx" data-testid="seat-l20-card-fx">{{ l20CardFx }}</span>
            <span v-if="seatPauseGuess" class="seat-pause-guess-pill" data-testid="seat-pause-guess-pill">猜:{{ seatPauseGuess }}</span>
            <span v-if="seatPromptText" class="seat-prompt-pill" data-testid="seat-prompt-pill">{{ seatPromptText }}</span>
            <span v-if="currentTurn" class="tag turn-tag">回合</span>
            <span v-if="seat.turned_over" class="tag" aria-label="武将牌背面">翻面</span>
            <span v-if="isChained" class="tag chain-tag" data-testid="seat-chain-badge">🔗横置</span>
            <span v-if="hasJiuBuff" class="tag jiu-tag" data-testid="seat-jiu-badge">🍶酒态</span>
            <span v-if="hasJiuwo" class="tag jiuwo-tag" data-testid="seat-jiuwo-badge">🆘救我</span>
            <span v-if="hasKillFire" class="tag killfire-tag" data-testid="seat-killfire-badge">🔥破敌</span>
            <span v-if="hasTiesuoYing" class="tag tiesuo-tag" data-testid="seat-tiesuo-badge">⛓结营</span>
            <span v-if="recastCount > 0" class="tag recast-tag" data-testid="seat-recast-badge">♻重铸×{{ recastCount }}</span>
            <span v-if="isHiddenVeil" class="tag yinni-tag" data-testid="seat-yinni-badge">暗置</span>
            <span v-if="isPublicHand" class="tag pubhand-tag" data-testid="seat-pubhand-badge">明牌{{ publicHandDisplay }}</span>
            <span
              v-if="rarityLongMode && isNodeEnabled('long')"
              class="tag dragon-crest"
              :class="`dragon-${rarityLongMode}`"
              :data-rarity-long="rarityLongMode"
              :title="`龙头美化 (${rarityLongMode})`"
              data-node-key="long"
              data-testid="seat-dragon-crest"
            >{{ rarityLongMode === 'xinon' ? '✨🐉' : rarityLongMode === 'off' ? '🐲' : '🐉' }}</span>
            <span v-else v-show="isNodeEnabled('long')" class="seat-node-anchor" data-node-key="long" aria-hidden="true"></span>
            <span v-if="cardUseCue" class="tag cue-tag" data-testid="seat-card-cue">{{ cardUseCue }}</span>
            <span v-if="gainStickerSkill" class="tag seat-gain-skill-sticker" data-testid="seat-gain-skill-sticker">✦{{ gainStickerSkill }}</span>
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
          v-if="judgeCards && judgeCards.length > 0 && isNodeEnabled('judges')"
          class="seat-judge-marks"
          data-node-key="judges"
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
        <span v-else v-show="isNodeEnabled('judges')" class="seat-node-anchor" data-node-key="judges" aria-hidden="true"></span>

        <!-- Equip strips + `_xunizhuangbei_gua_gua` virtual equips + `_changeEquipExt` dim state -->
        <div
          v-if="(visibleEquips.length > 0 || virtualEquips.length > 0) && isNodeEnabled('equips')"
          class="equip-zone"
          :class="{ 'is-equip-muted': isEquipMuted }"
          data-node-key="equips"
          aria-label="装备区"
        >
          <div
            v-for="card in visibleEquips"
            :key="card.id"
            class="equip-card"
            :title="`${cardKindLabel(card.kind, card.def)} #${card.id}${card.def != null ? ' (def:' + card.def + ')' : ''}`"
          >
            <span class="equip-suit" :data-suit="card.suit">{{ suitGlyph(card.suit) }}</span>
            <span class="equip-name">{{ cardKindLabel(card.kind, card.def) }}</span>
          </div>
          <div
            v-for="ve in virtualEquips"
            :key="`ve-${ve}`"
            class="equip-card is-virtual-equip"
            data-virtual-equip="true"
            data-testid="seat-virtual-equip"
            :title="`虚拟防具 ${ve}`"
          >
            <span class="equip-name">☯ {{ ve }}</span>
          </div>
        </div>
        <span v-else v-show="isNodeEnabled('equips')" class="seat-node-anchor" data-node-key="equips" aria-hidden="true"></span>

        <button
          v-if="qhlySkinBtnVisible"
          type="button"
          class="seat-qhly-skin-btn"
          :style="qhlySkinBtnPos"
          data-testid="btn-seat-qhly-skin"
          @click.stop="emit('cycleQhlySkin', seat)"
        >
          换肤
        </button>
        <div v-if="seatSayBubble" class="seat-say-bubble" data-testid="seat-say-bubble">
          💬 {{ seatSayBubble }}
        </div>
        <div v-if="!seat.alive" class="dead-badge">✗ 阵亡 · {{ identityBadge.glyph }}</div>
        <div v-else-if="seat.hp <= 0" class="dying-badge">⚠ 濒死</div>
      </div>
    </div>

    <!-- Bottom DecadeUI HP & meta footer -->
    <div class="seat-footer">
      <div v-show="isNodeEnabled('hp')" class="hp-row" data-node-key="hp">
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
        <span data-node-key="name" v-show="isNodeEnabled('name')">P{{ seat.player }} · 手牌 {{ seat.hand_count }}</span>
        <span v-show="hasDeputy && isNodeEnabled('name2')" data-node-key="name2">{{ slots[1]?.name ?? "" }}</span>
        <span data-node-key="nameol" v-show="isNodeEnabled('nameol')">{{ identityLabel }}</span>
      </div>

      <!-- L08 Decade mark & gained skill badges (with clickable markskill intro popover) -->
      <div
        v-if="(decadeBadges.length > 0 || extraGainedSkills.length > 0) && isNodeEnabled('marks')"
        class="decade-seat-marks"
        data-node-key="marks"
        data-testid="seat-decade-marks"
      >
        <span
          v-for="sk in extraGainedSkills"
          :key="`gain-${sk.id}`"
          class="seat-mark-chip seat-gain-skill-badge"
          data-testid="seat-gain-skill-badge"
        >{{ sk.name }}</span>
        <span
          v-for="mk in decadeBadges"
          :key="mk.skillId"
          class="seat-mark-chip is-clickable"
          :class="{
            'mark-yinyang-yang': mk.badgeText.includes('阳'),
            'mark-yinyang-yin': mk.badgeText.includes('阴'),
            'mark-limited': mk.badgeText.includes('限') || mk.badgeText.includes('✕'),
            'is-selected': selectedMarkId === mk.skillId,
          }"
          :data-nonumber="mk.nonumberMark ? 'on' : 'off'"
          :data-noskill-control="mk.noskillControl ? 'on' : 'off'"
          :title="mk.tooltip"
          data-testid="seat-mark-badge"
          @click.stop="toggleMarkTooltip(mk.skillId)"
        >{{ mk.label }}[{{ mk.badgeText }}]</span>
      </div>
      <span v-else v-show="isNodeEnabled('marks')" class="seat-node-anchor" data-node-key="marks" aria-hidden="true"></span>
      <div
        v-if="selectedMarkDetail && isNodeEnabled('intro')"
        class="seat-mark-intro-popover"
        data-node-key="intro"
        data-testid="seat-mark-intro-popover"
        @click.stop="selectedMarkId = null"
      >
        <strong>{{ selectedMarkDetail.label }}</strong> · {{ selectedMarkDetail.tooltip }}
      </div>
      <span v-else v-show="isNodeEnabled('intro')" class="seat-node-anchor" data-node-key="intro" aria-hidden="true"></span>
      <span class="seat-node-anchor" data-node-key="wei" :data-faction="slots[0]?.faction ?? 'unknown'" v-show="isNodeEnabled('wei')" aria-hidden="true"></span>
      <span class="seat-node-anchor" data-node-key="handcards1" v-show="isNodeEnabled('handcards1')" aria-hidden="true"></span>
      <span class="seat-node-anchor" data-node-key="handcards2" v-show="isNodeEnabled('handcards2')" aria-hidden="true"></span>
      <span class="seat-node-anchor" data-node-key="expansions" v-show="isNodeEnabled('expansions')" aria-hidden="true"></span>

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
.seat-framebg-layer {
  position: absolute;
  inset: 0;
  border-radius: inherit;
  pointer-events: none;
  z-index: 0;
}
.seat-node-anchor {
  position: absolute;
  width: 0;
  height: 0;
  overflow: hidden;
  pointer-events: none;
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
.seat.dead.no-die-gray {
  filter: none;
}
.seat-identity-stamp.camp-text-only {
  width: auto;
  height: auto;
  padding: 0.08rem 0.28rem;
  border-radius: 3px;
  font-size: 0.72rem;
  background: rgba(15, 18, 22, 0.92) !important;
  background-image: none !important;
  box-shadow: none !important;
  border-style: dashed;
}
.dragon-crest {
  background: rgba(120, 53, 15, 0.88);
  color: #fde68a;
}
.dragon-crest.dragon-off {
  background: rgba(55, 65, 81, 0.88);
  color: #d1d5db;
}
.dragon-crest.dragon-xinon {
  background: linear-gradient(135deg, #b45309, #7c2d12);
  border: 1px solid #fde047;
  color: #fef08a;
}
.cue-tag {
  background: rgba(161, 98, 7, 0.9);
  color: #fef08a;
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
.jiuwo-tag {
  background: #dc2626;
  color: #fff;
}
.killfire-tag {
  background: #ea580c;
  color: #fff7ed;
}
.tiesuo-tag {
  background: #854d0e;
  color: #fef08a;
}
.recast-tag {
  background: #0369a1;
  color: #e0f2fe;
}
.yinni-tag {
  background: #334155;
  color: #e2e8f0;
}
.pubhand-tag {
  background: #0f766e;
  color: #ccfbf1;
}
.is-virtual-equip {
  border-style: dashed !important;
  border-color: rgba(56, 189, 248, 0.72) !important;
  color: #bae6fd !important;
}
.equip-zone.is-equip-muted {
  opacity: 0.48;
}
.seat-mark-chip.is-clickable {
  cursor: pointer;
}
.seat-mark-intro-popover {
  margin-top: 0.16rem;
  padding: 0.2rem 0.34rem;
  border-radius: 5px;
  font-size: 0.62rem;
  line-height: 1.3;
  background: rgba(12, 16, 22, 0.96);
  border: 1px solid rgba(232, 165, 75, 0.6);
  color: #f3ead3;
  cursor: pointer;
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
  left: 24px;
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
