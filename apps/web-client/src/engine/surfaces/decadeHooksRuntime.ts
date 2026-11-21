/**
 * L08 十周年UI 全部 319 项 UI 钩子消费链与规则净室渲染投影 (`33 核心钩子 + 286 markskill.js`)
 * 严守裁决 35/38 净室边界：不读取 `spine.js`，不含闭源商业资产；UI 钩子仅负责视图/动效/交互提示，绝不篡改内核规则。
 */

import type { CardView, EngineEnvelope, SeatView } from "../../types/engine";
import { lookupCardKindByDef } from "./cardCatalogLabels";
import type { DecadeUiConfig } from "./decadeConfigSchema";
import { DECADE_MARKSKILL_SPECS } from "./decadeMarkSkillsData";
import {
  dispatchDecadeUiHook,
  formatDecadeSkillMarkBadge,
  type DecadeMarkBadgeView,
  type DecadeUiRuntimeState,
} from "./decadeUiSurface";

export interface DecadeCoreHookSpec {
  readonly id: string;
  readonly sourceFile: "animation.js" | "animation_new.js" | "extension.js" | "meihua.js" | "splash.js";
  readonly line: number;
  readonly trigger: string;
  readonly role: string;
}

/** 33 项核心 UI 钩子上游契约表 */
export const DECADE_CORE_HOOK_SPECS: ReadonlyArray<DecadeCoreHookSpec> = [
  { id: "_jiuwo_", sourceFile: "animation.js", line: 1402, trigger: "dyingBegin", role: "濒死求救呼吸光效与角标" },
  { id: "_delete_jiuwo_", sourceFile: "animation.js", line: 1416, trigger: "dyingAfter/die", role: "脱离濒死或阵亡时清除救我光效" },
  { id: "_dyjinengtx", sourceFile: "animation.js", line: 1430, trigger: "logSkillBegin/useSkillBegin", role: "武将势力发光与技能特写横幅" },
  { id: "_losetl_", sourceFile: "animation.js", line: 1604, trigger: "loseHpBegin", role: "体力流失震颤提示与数值投影" },
  { id: "_jinnangpai_", sourceFile: "animation.js", line: 1614, trigger: "useCardBegin/respondBegin", role: "桃园/南蛮/万箭全屏锦囊横幅" },
  { id: "_jiubuff_", sourceFile: "animation.js", line: 1632, trigger: "useCard(jiu)", role: "酒态增伤烈焰角标" },
  { id: "_jiubuff_xiaoshi_", sourceFile: "animation.js", line: 1650, trigger: "useCard(sha)/phaseAfter/dieAfter", role: "出杀/回合结束清除酒态角标" },
  { id: "_player_hp", sourceFile: "animation.js", line: 1674, trigger: "damageBegin5/recoverBegin", role: "受伤/回血跳字数值气泡" },
  { id: "_HpRecover", sourceFile: "animation_new.js", line: 1402, trigger: "recoverBegin", role: "EffectWood 绿意治愈光环" },
  { id: "_Hplose", sourceFile: "animation_new.js", line: 1448, trigger: "damageBegin4", role: "受伤红闪与数值记录" },
  { id: "_killFire", sourceFile: "animation_new.js", line: 1620, trigger: "dieBegin/phaseAfter", role: "击杀者燃火边界特效" },
  { id: "_changeEquipExt", sourceFile: "extension.js", line: 54, trigger: "equipAfter/loseBegin", role: "装备区文本透明度与槽位同步" },
  { id: "_ui_auto_cheat", sourceFile: "extension.js", line: 180, trigger: "phaseUse(local-only)", role: "单机本地调试即时检查开关" },
  { id: "_kill_framebg", sourceFile: "extension.js", line: 210, trigger: "gameStart/showCharacterEnd", role: "按武将评级自动匹配边框等阶" },
  { id: "_rarity", sourceFile: "extension.js", line: 321, trigger: "gameStart/showCharacterEnd", role: "稀有度(legend/epic/rare)龙头与框色判定" },
  { id: "_tiesuo_ying", sourceFile: "extension.js", line: 479, trigger: "gameDrawAfter/dieBegin", role: "神刘备结营金锁链光效" },
  { id: "_usecard", sourceFile: "extension.js", line: 7540, trigger: "useCardAfter", role: "出牌阶段光效结算并清理" },
  { id: "_decadeUI_usecardBegin", sourceFile: "extension.js", line: 7556, trigger: "useCardBegin", role: "卡牌使用瞬间框体亮光" },
  { id: "_discard", sourceFile: "extension.js", line: 7569, trigger: "discardAfter/loseAsyncAfter", role: "弃牌完成清理瞬态高光" },
  { id: "_decadeUI_dieKillEffect", sourceFile: "extension.js", line: 7604, trigger: "dieBegin(source)", role: "击杀破敌横幅展示" },
  { id: "_recasting", sourceFile: "meihua.js", line: 3336, trigger: "phaseUse", role: "牌面重铸计数与抛牌提示" },
  { id: "_xunizhuangbei_gua_gua", sourceFile: "meihua.js", line: 3464, trigger: "equipAfter/gameStart", role: "八阵/玲珑虚拟八卦防具槽标" },
  { id: "_gshoupai", sourceFile: "meihua.js", line: 3748, trigger: "gainEnd/loseEnd/gameStart", role: "2v2/窥伺场景公开手牌数徽标" },
  { id: "drlt_poxi", sourceFile: "meihua.js", line: 6772, trigger: "phaseUse", role: "魄袭双区手牌4异花色视图弹窗" },
  { id: "pro_drlt_poxi", sourceFile: "meihua.js", line: 6887, trigger: "phaseUse", role: "界魄袭双区1-4异花色视图弹窗" },
  { id: "_gzyinni", sourceFile: "splash.js", line: 5, trigger: "gameStart/showCharacterEnd", role: "国战未明置主/副将隐匿幕布" },
  { id: "_guozhan", sourceFile: "splash.js", line: 339, trigger: "gameStart/showCharacterEnd", role: "手杀国战暗将与明置势力条" },
  { id: "_gzyinniszn", sourceFile: "splash.js", line: 391, trigger: "gameStart/showCharacterEnd", role: "十周年国战隐匿幕布" },
  { id: "_guozhanszn", sourceFile: "splash.js", line: 504, trigger: "gameStart/showCharacterEnd", role: "十周年国战副将框架标识" },
  { id: "_yinni", sourceFile: "splash.js", line: 556, trigger: "gameStart/showCharacterEnd", role: "晋势力隐匿通用提示" },
  { id: "_shousha_shuangjiang", sourceFile: "splash.js", line: 817, trigger: "gameStart", role: "手杀双将副将框势力条" },
  { id: "_shizhounian_shuangjiang", sourceFile: "splash.js", line: 944, trigger: "gameStart", role: "十周年双将常规露头框条" },
  { id: "_shizhounian_shuangjiang2", sourceFile: "splash.js", line: 993, trigger: "showCharacterAfter/dieBegin", role: "国战亮将后双将框体挂接" },
];

const MARKSKILL_ID_SET: ReadonlySet<string> = new Set(DECADE_MARKSKILL_SPECS.map((s) => s.id));
const RARITY_BORDER_MAP: Readonly<Record<string, string>> = {
  junk: "one",
  common: "two",
  rare: "three",
  epic: "four",
  legend: "five",
};

/** `_kill_framebg` (`extension.js:210-260`)：按配置或武将评级返回最终边框等阶 `one..five` */
export function resolveDecadeSeatBorderTier(
  config: Pick<DecadeUiConfig, "borderLevel">,
  rarityTier?: "junk" | "common" | "rare" | "epic" | "legend",
): string {
  if (config.borderLevel !== "kill") return config.borderLevel;
  return RARITY_BORDER_MAP[rarityTier ?? "rare"] ?? "three";
}

/** `_rarity` (`extension.js:321-355`)：确定性推导座位武将评级（主公或高def倾向更高评级） */
export function deriveGeneralRarityTier(
  seat: Pick<SeatView, "character_def" | "identity" | "generals">,
): "junk" | "common" | "rare" | "epic" | "legend" {
  const def = seat.generals?.[0]?.def ?? seat.character_def ?? 0;
  if (seat.identity === "Lord" || seat.generals?.[0]?.faction === "shen") return "legend";
  if (def >= 200) return "epic";
  if (def >= 60) return "rare";
  if (def > 0) return "common";
  return "junk";
}

/** `cardAlternateNameVisible` (`extension.js:12330`)：卡牌辅助名称与攻击距离/效果摘要 */
export function buildCardAuxOverlay(kind: CardView["kind"], def?: number | null): string {
  const entry = lookupCardKindByDef(def);
  const label = entry?.label ?? "";
  if (kind === "HorseDef" || label === "+1马") return "防御 +1 距";
  if (kind === "HorseOff" || label === "-1马") return "进攻 -1 距";
  if (kind === "Weapon" || label.includes("武器") || entry?.category === "equip") {
    if (label.includes("连弩")) return "攻距 1 · 连发";
    if (label.includes("青釭") || label.includes("雌雄") || label.includes("古锭")) return "攻距 2 · 特效";
    if (label.includes("青龙") || label.includes("丈八") || label.includes("贯石")) return "攻距 3";
    if (label.includes("方天") || label.includes("朱雀")) return "攻距 4";
    if (label.includes("麒麟")) return "攻距 5";
    if (kind === "Armor" || label.includes("防具") || label.includes("盾") || label.includes("甲")) {
      return "防具护体";
    }
    return "武器 / 装备";
  }
  if (kind === "Armor") return "防具护体";
  if (kind === "ShaFire" || kind === "ShaThunder" || label === "火杀" || label === "雷杀") {
    return "属性杀 · 传伤";
  }
  if (kind === "TrickNanman" || kind === "TrickWanjian" || kind === "TrickTaoyuan" || kind === "TrickWugu" || label.includes("南蛮") || label.includes("万箭") || label.includes("桃园") || label.includes("五谷")) {
    return "全体锦囊";
  }
  if (label === "顺手牵羊" || label === "兵粮寸断" || kind === "TrickShunshou" || kind === "TrickBingliang") {
    return "距 1 · 限距";
  }
  if (kind === "TrickTiesuo" || label === "铁索连环") return "连环 · 可重铸";
  if (kind.startsWith("Trick") || entry?.category === "trick") return "锦囊牌效";
  if (kind === "Tao" || label === "桃") return "回复 +1";
  if (kind === "Jiu" || label === "酒") return "增伤 · 救濒";
  if (kind === "Shan" || label === "闪") return "响应闪避";
  return "攻距 · 出伤";
}

/** `drlt_poxi` / `pro_drlt_poxi` (`meihua.js:6828-6833, 6918`) 四异花色检验 */
export function validatePoxiSelection(
  mode: "drlt_poxi" | "pro_drlt_poxi",
  selectedCards: ReadonlyArray<{ id: number; suit: string }>,
): { valid: boolean; reason: string } {
  const suits = new Set<string>();
  for (const c of selectedCards) {
    if (suits.has(c.suit)) {
      return { valid: false, reason: `存在重复花色 (${c.suit})，魄袭须选不同花色` };
    }
    suits.add(c.suit);
  }
  if (mode === "drlt_poxi") {
    return selectedCards.length === 4
      ? { valid: true, reason: "已选满 4 张不同花色卡牌" }
      : { valid: false, reason: `已选 ${selectedCards.length}/4 张（标准魄袭须恰好 4 张异花色）` };
  }
  return selectedCards.length >= 1 && selectedCards.length <= 4
    ? { valid: true, reason: `已选 ${selectedCards.length} 张异花色卡牌 (界魄袭 1-4 张)` }
    : { valid: false, reason: "界魄袭至少选择 1 张卡牌（至多 4 张异花色）" };
}

/** 从原生事件条目提取显式目标/来源座位编号 */
function extractNumericSeatField(rawEv: unknown, keys: ReadonlyArray<string>): number | null {
  if (!rawEv || typeof rawEv !== "object") return null;
  const obj = rawEv as Record<string, unknown>;
  for (const k of keys) {
    if (typeof obj[k] === "number" && Number.isFinite(obj[k])) return obj[k] as number;
  }
  const pub = obj.PublicEvent;
  if (pub && typeof pub === "object") {
    const pubObj = pub as Record<string, unknown>;
    for (const k of keys) {
      if (typeof pubObj[k] === "number" && Number.isFinite(pubObj[k])) return pubObj[k] as number;
    }
  }
  return null;
}

/** 根据对局 `EngineEnvelope` 同步 33 核心钩子 + 286 `markskill` 实时表现状态 */
export function syncDecadeUiFromEnvelope(
  prev: DecadeUiRuntimeState,
  env: EngineEnvelope,
): DecadeUiRuntimeState {
  const view = env.view;
  if (!view) return prev;

  const revAdvanced =
    prev.lastSyncedRevision === undefined ||
    env.revision !== prev.lastSyncedRevision ||
    view.round !== prev.lastSyncedRound ||
    view.phase !== prev.lastSyncedPhase ||
    view.current_player !== prev.lastSyncedPlayer ||
    (env.events?.length ?? 0) > 0;

  const turnOrPhaseChanged =
    prev.lastSyncedRound !== undefined &&
    (view.round !== prev.lastSyncedRound ||
      view.phase !== prev.lastSyncedPhase ||
      view.current_player !== prev.lastSyncedPlayer);

  // Expire per-action / per-phase transient cues cleanly before projecting current envelope
  let next: DecadeUiRuntimeState = revAdvanced
    ? {
        ...prev,
        hpDeltaBySeat: {},
        damageFlashSeats: [],
        loseHpSeats: [],
        healAuraSeats: [],
        trickEffectBanner: null,
        skillPopup: null,
        factionGlowBySeat: {},
        cardUseCueBySeat: {},
        killOverlay: null,
        jiuBuffSeats:
          turnOrPhaseChanged && view.phase !== "Play" ? [] : prev.jiuBuffSeats,
        killFireSeats:
          turnOrPhaseChanged && (view.phase === "Discard" || view.phase === "End" || view.current_player !== prev.lastSyncedPlayer)
            ? []
            : prev.killFireSeats,
      }
    : prev;

  const nextPrevHp: Record<number, number> = { ...prev.prevHpBySeat };
  const nextPrevMaxHp: Record<number, number> = { ...(prev.prevSeatMaxHpBySeat ?? {}) };
  const nextPrevHadGeneral: Record<number, boolean> = { ...(prev.prevSeatHadGeneralBySeat ?? {}) };
  const hpSyncedSeats = new Set<number>();
  const damagedSeatsFromHp: number[] = [];
  const recoveredSeatsFromHp: number[] = [];
  const diedSeatsInStep: number[] = [];

  for (const seat of view.seats) {
    const pid = seat.player;
    next = dispatchDecadeUiHook(next, "_gshoupai", {
      seat: pid,
      handCount: seat.hand_count,
    });

    // _rarity + _kill_framebg
    const tier = deriveGeneralRarityTier(seat);
    next = dispatchDecadeUiHook(next, "_rarity", { seat: pid, rarity: tier });

    // _jiuwo_ / _delete_jiuwo_
    if (seat.alive && seat.hp <= 0) {
      next = dispatchDecadeUiHook(next, "_jiuwo_", { seat: pid });
    } else {
      next = dispatchDecadeUiHook(next, "_delete_jiuwo_", { seat: pid });
    }
    if (!seat.alive) {
      diedSeatsInStep.push(pid);
    }

    // _player_hp delta tracking: ignore max_hp / initial general initialization shifts
    const oldHp = prev.prevHpBySeat[pid];
    const oldMaxHp = prev.prevSeatMaxHpBySeat?.[pid];
    const hadGeneralBefore = prev.prevSeatHadGeneralBySeat?.[pid] ?? false;
    const hasGeneralNow = Boolean(seat.generals?.[0] || seat.character_def !== 0);
    const isGeneralInit =
      oldMaxHp !== undefined && (oldMaxHp !== seat.max_hp || (!hadGeneralBefore && hasGeneralNow));

    if (!isGeneralInit && typeof oldHp === "number" && oldHp !== seat.hp) {
      const diff = seat.hp - oldHp;
      hpSyncedSeats.add(pid);
      if (diff < 0) damagedSeatsFromHp.push(pid);
      else recoveredSeatsFromHp.push(pid);
      next = dispatchDecadeUiHook(next, "_player_hp", {
        seat: pid,
        delta: Math.abs(diff),
        hpKind: diff > 0 ? "recover" : "damage",
      });
      if (diff < 0) {
        next = dispatchDecadeUiHook(next, "_losetl_", { seat: pid, delta: Math.abs(diff) });
      }
    }
    nextPrevHp[pid] = seat.hp;
    nextPrevMaxHp[pid] = seat.max_hp;
    nextPrevHadGeneral[pid] = hasGeneralNow;

    // _gzyinni / _shizhounian_shuangjiang hidden & dual general slots
    const generals = seat.generals ?? [null, null];
    const hasUnrevealed = generals.some((g) => g !== null && g.revealed === false);
    if (hasUnrevealed) {
      next = dispatchDecadeUiHook(next, "_gzyinni", { seat: pid, revealed: false });
    } else if (next.hiddenSeats.includes(pid)) {
      next = dispatchDecadeUiHook(next, "_gzyinni", { seat: pid, revealed: true });
    }
    if (generals[1] !== null && next.splashMode === "standard") {
      next = dispatchDecadeUiHook(next, "_shizhounian_shuangjiang", { seat: pid });
    }

    // Sync live seat skills -> markskill.js (286) + _tiesuo_ying + _xunizhuangbei_gua_gua
    const badges: DecadeMarkBadgeView[] = [...(next.activeMarkBadgesBySeat[pid] ?? [])];
    const badgeMap = new Map<string, DecadeMarkBadgeView>(badges.map((b) => [b.skillId, b]));
    for (const sk of seat.skills ?? []) {
      const sname = sk.name ?? "";
      if (!sname) continue;
      if (sname === "nzry_jieying" || sname === "pro_nzry_jieying") {
        next = dispatchDecadeUiHook(next, "_tiesuo_ying", { seat: pid });
      }
      if (sname === "bazhen" || sname === "bazhen_bagua" || sname === "linglong_bagua") {
        next = dispatchDecadeUiHook(next, "_xunizhuangbei_gua_gua", { seat: pid, equipName: "八卦阵" });
      }
      if (MARKSKILL_ID_SET.has(sname) && sk.use_count > 0) {
        const badge = formatDecadeSkillMarkBadge(sname, {
          count: Math.max(1, sk.use_count),
          usedLimited: sk.use_count > 0,
          yang: sk.use_count % 2 === 1,
        });
        if (badge) badgeMap.set(sname, badge);
      }
    }
    if (seat.r05_effects && seat.r05_effects.length > 0) {
      for (const ef of seat.r05_effects) {
        if (typeof ef.baoshu_combs === "number" && ef.baoshu_combs > 0) {
          const b = formatDecadeSkillMarkBadge("rebotu", { count: ef.baoshu_combs });
          if (b) badgeMap.set(b.skillId, b);
        }
      }
    }
    next = {
      ...next,
      activeMarkBadgesBySeat: {
        ...next.activeMarkBadgesBySeat,
        [pid]: Array.from(badgeMap.values()),
      },
    };
  }
  next = {
    ...next,
    prevHpBySeat: nextPrevHp,
    prevSeatMaxHpBySeat: nextPrevMaxHp,
    prevSeatHadGeneralBySeat: nextPrevHadGeneral,
    lastSyncedRevision: env.revision,
    lastSyncedRound: view.round,
    lastSyncedPhase: view.phase,
    lastSyncedPlayer: view.current_player,
  };

  const curPlayer = view.current_player ?? 0;
  const curSeat = view.seats.find((s) => s.player === curPlayer);
  const curName = curSeat?.generals?.[0]?.name ?? `P${curPlayer}`;
  const curFaction = curSeat?.generals?.[0]?.faction ?? "qun";

  for (const rawEv of env.events ?? []) {
    const pubCode = (rawEv as { PublicEvent?: { code?: number } })?.PublicEvent?.code;
    const evType = (rawEv as { type?: string })?.type;
    const cardKind = (rawEv as { card?: string; kind?: string })?.card ?? (rawEv as { kind?: string })?.kind;
    const explicitTarget = extractNumericSeatField(rawEv, ["target", "victim", "player", "seat"]);
    const explicitSource = extractNumericSeatField(rawEv, ["source", "killer", "actor"]);

    if (pubCode === 4 || evType === "DamageDealt") {
      const targets =
        explicitTarget != null
          ? [explicitTarget]
          : damagedSeatsFromHp;
      for (const t of targets) {
        const preservedDelta = hpSyncedSeats.has(t)
          ? Math.abs(next.hpDeltaBySeat[t]?.delta ?? 1)
          : 1;
        next = dispatchDecadeUiHook(next, "_Hplose", { seat: t, delta: preservedDelta });
      }
    } else if (pubCode === 5 || evType === "HpRecovered") {
      const targets =
        explicitTarget != null
          ? [explicitTarget]
          : recoveredSeatsFromHp;
      for (const t of targets) {
        const preservedDelta = hpSyncedSeats.has(t)
          ? Math.abs(next.hpDeltaBySeat[t]?.delta ?? 1)
          : 1;
        next = dispatchDecadeUiHook(next, "_HpRecover", { seat: t, delta: preservedDelta });
      }
    } else if (pubCode === 2 || pubCode === 3 || evType === "CardPlayed") {
      const actor = explicitSource ?? explicitTarget ?? curPlayer;
      next = dispatchDecadeUiHook(next, "_decadeUI_usecardBegin", { seat: actor });
      next = dispatchDecadeUiHook(next, "_usecard", { seat: actor });
      if (cardKind === "Jiu") {
        next = dispatchDecadeUiHook(next, "_jiubuff_", { seat: actor });
      } else if (cardKind === "Sha" || cardKind === "ShaFire" || cardKind === "ShaThunder") {
        next = dispatchDecadeUiHook(next, "_jiubuff_xiaoshi_", { seat: actor });
      } else if (cardKind === "TrickTaoyuan") {
        next = dispatchDecadeUiHook(next, "_jinnangpai_", { seat: actor, cardName: "taoyuan" });
      } else if (cardKind === "TrickNanman") {
        next = dispatchDecadeUiHook(next, "_jinnangpai_", { seat: actor, cardName: "nanman" });
      } else if (cardKind === "TrickWanjian") {
        next = dispatchDecadeUiHook(next, "_jinnangpai_", { seat: actor, cardName: "wanjian" });
      }
    } else if (pubCode === 7 || evType === "PlayerDied") {
      const victimId = explicitTarget ?? diedSeatsInStep[0];
      if (victimId != null) {
        const victimSeat = view.seats.find((s) => s.player === victimId);
        const victimName = victimSeat?.generals?.[0]?.name ?? `P${victimId}`;
        const killerId = explicitSource ?? curPlayer;
        const killerSeat = view.seats.find((s) => s.player === killerId);
        const killerName = killerSeat?.generals?.[0]?.name ?? curName;
        next = dispatchDecadeUiHook(next, "_decadeUI_dieKillEffect", {
          seat: victimId,
          killerName,
          victimName,
        });
        next = dispatchDecadeUiHook(next, "_killFire", { seat: killerId });
        next = dispatchDecadeUiHook(next, "_tiesuo_ying", { seat: victimId, clear: true });
      }
    } else if (evType === "SkillActivated") {
      const actor = explicitSource ?? explicitTarget ?? curPlayer;
      const actorSeat = view.seats.find((s) => s.player === actor);
      const sName = (rawEv as { skill?: string })?.skill ?? "技能发动";
      next = dispatchDecadeUiHook(next, "_dyjinengtx", {
        seat: actor,
        generalName: actorSeat?.generals?.[0]?.name ?? curName,
        skillName: sName,
        faction: actorSeat?.generals?.[0]?.faction ?? curFaction,
      });
    } else if (pubCode === 8 && view.phase === "Discard") {
      next = dispatchDecadeUiHook(next, "_jiubuff_xiaoshi_", { seat: curPlayer });
      next = dispatchDecadeUiHook(next, "_killFire", { seat: curPlayer, clear: true });
    }
  }
  return next;
}
