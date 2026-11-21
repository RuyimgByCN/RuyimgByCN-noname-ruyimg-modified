/**
 * L08 十周年UI 真实牌桌环绕式座位布局与身份/牌名映射
 * 上游对照:
 * - `archive/upstream/noname-liuli/noname/extension/十周年UI/decadeLayout.css:81-534` (2-8 人座位坐标)
 * - `archive/upstream/noname-liuli/noname/extension/十周年UI/player_new.css:190-208, 1101-1150` (主视角与身份标记)
 * - `archive/upstream/noname-liuli/noname/extension/十周年UI/layout_new.css:126-188, 589-658` (底部手牌区与操作按钮区)
 */

import type { CardView, SeatView } from "../../types/engine";
import { lookupCardKindByDef, type TabletopCardCategory } from "./cardCatalogLabels";

export type SeatPerimeterZone = "self" | "right" | "top" | "left";

export interface RelativeSeatSlot {
  seat: SeatView;
  position: number;
  total: number;
}

export interface DecadeSeatAnchor {
  position: number;
  total: number;
  zone: SeatPerimeterZone;
  /** CSS variables used by desktop absolute perimeter layout */
  cssVars: Record<string, string>;
}

export interface IdentityBadgeInfo {
  glyph: string;
  label: string;
  tone: "lord" | "loyalist" | "rebel" | "renegade" | "unknown";
}

/**
 * 按座次升序排列并以 `selfId` 为 `position = 0`（逆时针环绕：下家=1 在右，上家=N-1 在左）。
 */
export function computeRelativeSeatOrder(
  seats: ReadonlyArray<SeatView>,
  selfId: number,
): RelativeSeatSlot[] {
  if (seats.length === 0) return [];
  const ordered = [...seats].sort((a, b) => a.seat - b.seat || a.player - b.player);
  const total = ordered.length;
  const rawIdx = ordered.findIndex((s) => s.player === selfId);
  const selfIdx = rawIdx >= 0 ? rawIdx : 0;
  return ordered.map((seat, idx) => ({
    seat,
    position: (idx - selfIdx + total) % total,
    total,
  }));
}

/**
 * 按上游 `decadeLayout.css:241-377` (`data-decade-layout='on'`) 返回 2–8 人环绕式座位锚点，
 * 并保证 `768px` 及以上桌面视口中顶排与左右侧席位不重叠。
 */
export function computeDecadeSeatAnchor(
  position: number,
  total: number,
  opts?: { selfOnRight?: boolean },
): DecadeSeatAnchor {
  const n = Math.max(2, Math.min(8, Math.round(total)));
  const pos = ((position % n) + n) % n;

  if (pos === 0) {
    const selfOnRight = Boolean(opts?.selfOnRight);
    return {
      position: 0,
      total: n,
      zone: "self",
      cssVars: {
        "--seat-left": selfOnRight ? "auto" : "16px",
        "--seat-right": selfOnRight ? "16px" : "auto",
        "--seat-top": "auto",
        "--seat-bottom": "10px",
        "--seat-tx": "0%",
      },
    };
  }

  const makeTop = (leftPct: number, topPx: number): DecadeSeatAnchor => ({
    position: pos,
    total: n,
    zone: "top",
    cssVars: {
      "--seat-left": `${leftPct}%`,
      "--seat-right": "auto",
      "--seat-top": `${topPx}px`,
      "--seat-bottom": "auto",
      "--seat-tx": "-50%",
    },
  });

  const makeRight = (topPct: number, rightPx = 14): DecadeSeatAnchor => ({
    position: pos,
    total: n,
    zone: "right",
    cssVars: {
      "--seat-left": "auto",
      "--seat-right": `${rightPx}px`,
      "--seat-top": `${topPct}%`,
      "--seat-bottom": "auto",
      "--seat-tx": "0%",
    },
  });

  const makeLeft = (topPct: number, leftPx = 14): DecadeSeatAnchor => ({
    position: pos,
    total: n,
    zone: "left",
    cssVars: {
      "--seat-left": `${leftPx}px`,
      "--seat-right": "auto",
      "--seat-top": `${topPct}%`,
      "--seat-bottom": "auto",
      "--seat-tx": "0%",
    },
  });

  switch (n) {
    case 2:
      return makeTop(50, 10);
    case 3:
      return pos === 1 ? makeTop(68, 12) : makeTop(32, 12);
    case 4:
      if (pos === 1) return makeRight(36);
      if (pos === 2) return makeTop(50, 10);
      return makeLeft(36);
    case 5:
      if (pos === 1) return makeRight(36);
      if (pos === 2) return makeTop(67, 12);
      if (pos === 3) return makeTop(33, 12);
      return makeLeft(36);
    case 6:
      if (pos === 1) return makeRight(36);
      if (pos === 2) return makeTop(76, 12);
      if (pos === 3) return makeTop(50, 10);
      if (pos === 4) return makeTop(24, 12);
      return makeLeft(36);
    case 7:
      if (pos === 1) return makeRight(38);
      if (pos === 2) return makeTop(83, 12);
      if (pos === 3) return makeTop(61, 10);
      if (pos === 4) return makeTop(39, 10);
      if (pos === 5) return makeTop(17, 12);
      return makeLeft(38);
    case 8:
    default:
      if (pos === 1) return makeRight(38);
      if (pos === 2) return makeTop(86, 12);
      if (pos === 3) return makeTop(68, 10);
      if (pos === 4) return makeTop(50, 8);
      if (pos === 5) return makeTop(32, 10);
      if (pos === 6) return makeTop(14, 12);
      return makeLeft(38);
  }
}

/**
 * 上游 `十周年UI/extension.js:7068-7116` 身份角标（主/忠/反/内/猜）。
 */
export function identityBadgeInfo(identity: SeatView["identity"]): IdentityBadgeInfo {
  switch (identity) {
    case "Lord":
      return { glyph: "主", label: "主公", tone: "lord" };
    case "Loyalist":
      return { glyph: "忠", label: "忠臣", tone: "loyalist" };
    case "Rebel":
      return { glyph: "反", label: "反贼", tone: "rebel" };
    case "Renegade":
      return { glyph: "内", label: "内奸", tone: "renegade" };
    default:
      return { glyph: "猜", label: "身份未知", tone: "unknown" };
  }
}

const CARD_KIND_ZH: Readonly<Record<string, string>> = {
  Sha: "杀",
  Shan: "闪",
  Tao: "桃",
  Jiu: "酒",
  ShaFire: "火杀",
  ShaThunder: "雷杀",
  Weapon: "武器",
  Armor: "防具",
  HorseOff: "-1马",
  HorseDef: "+1马",
  TrickWuzhong: "无中生有",
  TrickGuohe: "过河拆桥",
  TrickShunshou: "顺手牵羊",
  TrickJuedou: "决斗",
  TrickNanman: "南蛮入侵",
  TrickWanjian: "万箭齐发",
  TrickTaoyuan: "桃园结义",
  TrickWugu: "五谷丰登",
  TrickLe: "乐不思蜀",
  TrickBingliang: "兵粮寸断",
  TrickShandian: "闪电",
  TrickJiedao: "借刀杀人",
  TrickTiesuo: "铁索连环",
  TrickHuogong: "火攻",
  TrickWuxie: "无懈可击",
};

export function cardKindLabel(kind: CardView["kind"] | undefined, def?: number | null): string {
  if (kind && Object.hasOwn(CARD_KIND_ZH, kind)) {
    return CARD_KIND_ZH[kind]!;
  }
  const entry = lookupCardKindByDef(def);
  if (entry) return entry.label;
  if (def != null) return `#${def}`;
  return kind && kind !== "Other" ? kind : "未知牌";
}

export function cardCategoryOf(
  kind: CardView["kind"] | undefined,
  def?: number | null,
): TabletopCardCategory {
  const k = kind ?? "";
  if (k === "Weapon" || k === "Armor" || k === "HorseOff" || k === "HorseDef") {
    return "equip";
  }
  if (k.startsWith("Trick")) return "trick";
  const entry = lookupCardKindByDef(def);
  if (entry) return entry.category;
  return "basic";
}

export function cardCategoryLabel(kind: CardView["kind"] | undefined, def?: number | null): string {
  if (kind && kind !== "Other" && Object.hasOwn(CARD_KIND_ZH, kind)) {
    const cat = cardCategoryOf(kind, def);
    return cat === "equip" ? "装备" : cat === "trick" ? "锦囊" : "基本";
  }
  const entry = lookupCardKindByDef(def);
  if (entry) return entry.categoryTag;
  return "功能";
}
