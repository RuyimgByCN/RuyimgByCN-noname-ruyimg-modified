/**
 * Batch UI-6 补齐运行时：
 * - L27 `e.css` (选将美化面板框架与倒计时条)
 * - L28 `thunder.css` (雷霆万钧技能提示与玉玺/火焰主题横幅)
 * - L29 (`shengjian`, `main1`, `main1.css`, `_CMeffects`) 龙血玄黄潜龙书法按钮与 CM 光效
 * - L10 (`cardFold`, `cardPhantom`, `shichangshi`, `_emojiRebate1`, `card.css`, `extension.css`) 史诗卡牌手牌折叠、幻影出牌、十常侍徽章与表情回馈
 */

export interface Batch6PatchUiState {
  // L27 e.css
  eCssEnabled: boolean;
  // L28 thunder.css
  thunderCssEnabled: boolean;
  thunderSkillTip: string | null;
  thunderYuqiTheme: "standard" | "golden";
  // L29 main1 + main1.css + shengjian + _CMeffects
  longxueMain1Enabled: boolean;
  longxueCmBurstLabel: string | null;
  // L10 史诗卡牌 6+1 项
  cardFold: boolean;
  cardFoldProportion: 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10;
  cardPhantom: boolean;
  shichangshi: boolean;
  lastPhantomLabel: string | null;
  activeEmojiRebate: {
    fromSeat: number;
    toSeat: number;
    kind: "praise" | "humiliation";
    icon: string;
    label: string;
  } | null;
  activeBatch6Surfaces: ReadonlyArray<
    "e.css" | "thunder.css" | "main1" | "main1.css" | "card.css" | "extension.css"
  >;
}

export function createInitialBatch6PatchUiState(): Batch6PatchUiState {
  return {
    eCssEnabled: true,
    thunderCssEnabled: true,
    thunderSkillTip: null,
    thunderYuqiTheme: "golden",
    longxueMain1Enabled: true,
    longxueCmBurstLabel: null,
    cardFold: true,
    cardFoldProportion: 10,
    cardPhantom: true,
    shichangshi: true,
    lastPhantomLabel: null,
    activeEmojiRebate: null,
    activeBatch6Surfaces: [
      "e.css",
      "thunder.css",
      "main1",
      "main1.css",
      "card.css",
      "extension.css",
    ],
  };
}

export function dispatchLongxueCmEffects(
  state: Batch6PatchUiState,
  opts?: { skillName?: string; daoxin?: number },
): Batch6PatchUiState {
  const skill = opts?.skillName ?? "潜龙 · 剑意迸发";
  const daoxin = opts?.daoxin ?? 99;
  return {
    ...state,
    longxueMain1Enabled: true,
    longxueCmBurstLabel: `${skill} (道心 ${daoxin}/99)`,
  };
}

export function dispatchL10EmojiRebate(
  state: Batch6PatchUiState,
  ev: {
    trigger: "recover" | "dying" | "turnOver" | "gain";
    fromSeat: number;
    toSeat: number;
    isEnemy: boolean;
  },
): Batch6PatchUiState {
  if (ev.fromSeat === ev.toSeat) return state;
  const kind: "praise" | "humiliation" =
    ev.trigger === "dying" || ev.isEnemy ? "humiliation" : "praise";
  const presets = {
    praise: [
      { icon: "💐", label: "献花致敬" },
      { icon: "🥂", label: "敬酒赞赏" },
    ],
    humiliation: [
      { icon: "🥚", label: "砸蛋调侃" },
      { icon: "🩴", label: "拖鞋警告" },
    ],
  }[kind];
  const pick = presets[(ev.fromSeat + ev.toSeat) % presets.length]!;
  return {
    ...state,
    activeEmojiRebate: {
      fromSeat: ev.fromSeat,
      toSeat: ev.toSeat,
      kind,
      icon: pick.icon,
      label: pick.label,
    },
  };
}

export function batch6PatchDatasetAttrs(state: Batch6PatchUiState): Record<string, string> {
  return {
    "data-batch6-surfaces": state.activeBatch6Surfaces.join(" "),
    "data-l28-thunder": state.thunderCssEnabled ? "on" : "off",
    "data-l28-yuqi": state.thunderYuqiTheme,
    "data-l29-main1": state.longxueMain1Enabled ? "on" : "off",
    "data-l10-card-fold": state.cardFold ? "on" : "off",
    "data-l10-card-phantom": state.cardPhantom ? "on" : "off",
    "data-l10-shichangshi": state.shichangshi ? "on" : "off",
  };
}
