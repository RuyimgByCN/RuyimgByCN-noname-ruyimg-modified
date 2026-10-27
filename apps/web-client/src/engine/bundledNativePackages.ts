// 安装时复制随应用分发的实际包数据；开局和读档只使用固定副本。
import type { ExtensionCatalogEntry } from "./extensionCatalog";
import package0 from "../../../../packages/extensions/cardpile-upstream-liuli/package.npack.json";
import data0_pile from "../../../../packages/extensions/cardpile-upstream-liuli/assets/pile.json";
import package1 from "../../../../packages/extensions/cardpile-upstream-noname/package.npack.json";
import data1_pile from "../../../../packages/extensions/cardpile-upstream-noname/assets/pile.json";
import package2 from "../../../../packages/extensions/d3-upstream-noname/package.npack.json";
import data2_characters from "../../../../packages/extensions/d3-upstream-noname/assets/character.json";
import package3 from "../../../../packages/extensions/huanle-upstream-noname/package.npack.json";
import data3_cards from "../../../../packages/extensions/huanle-upstream-noname/assets/cards.json";
import package4 from "../../../../packages/extensions/shaai-ow-upstream-noname/package.npack.json";
import data4_cards from "../../../../packages/extensions/shaai-ow-upstream-noname/assets/cards.json";
import package5 from "../../../../packages/extensions/shaai-xianjian-upstream-noname/package.npack.json";
import data5_cards from "../../../../packages/extensions/shaai-xianjian-upstream-noname/assets/cards.json";
import package6 from "../../../../packages/extensions/shahai-gujian-upstream-noname/package.npack.json";
import data6_cards from "../../../../packages/extensions/shahai-gujian-upstream-noname/assets/cards.json";
import package7 from "../../../../packages/extensions/shahai-gwent-upstream-noname/package.npack.json";
import data7_cards from "../../../../packages/extensions/shahai-gwent-upstream-noname/assets/cards.json";
import package8 from "../../../../packages/extensions/shahai-hearth-upstream-noname/package.npack.json";
import data8_cards from "../../../../packages/extensions/shahai-hearth-upstream-noname/assets/cards.json";
import package9 from "../../../../packages/extensions/shahai-mtg-upstream-noname/package.npack.json";
import data9_cards from "../../../../packages/extensions/shahai-mtg-upstream-noname/assets/cards.json";
import package10 from "../../../../packages/extensions/shahai-swd-upstream-noname/package.npack.json";
import data10_cards from "../../../../packages/extensions/shahai-swd-upstream-noname/assets/cards.json";
import package11 from "../../../../packages/extensions/shahai-yunchou-upstream-noname/package.npack.json";
import data11_cards from "../../../../packages/extensions/shahai-yunchou-upstream-noname/assets/cards.json";
import data11_characters from "../../../../packages/extensions/shahai-yunchou-upstream-noname/assets/character.json";
import package12 from "../../../../packages/extensions/shahai-zhenfa-upstream-noname/package.npack.json";
import data12_cards from "../../../../packages/extensions/shahai-zhenfa-upstream-noname/assets/cards.json";
import package13 from "../../../../packages/extensions/wudian-upstream-noname/package.npack.json";
import data13_cards from "../../../../packages/extensions/wudian-upstream-noname/assets/cards.json";
import package14 from "../../../../packages/extensions/wuxing-upstream-liuli/package.npack.json";
import data14_cards from "../../../../packages/extensions/wuxing-upstream-liuli/assets/cards.json";

export interface NativePackagePayload {
  manifest: {
    namespace: string;
    package_id: string;
    version: string;
    logic_hash: number[];
  };
  cards?: unknown;
  characters?: unknown;
  pile?: unknown;
}

const bundled: Record<string, { package: { modules: unknown[] }; payload: unknown }> = {
  "cardpile-upstream-liuli": { package: package0, payload: { manifest: package0.manifest, pile: data0_pile } },
  "cardpile-upstream-noname": { package: package1, payload: { manifest: package1.manifest, pile: data1_pile } },
  "d3-upstream-noname": { package: package2, payload: { manifest: package2.manifest, characters: data2_characters } },
  "huanle-upstream-noname": { package: package3, payload: { manifest: package3.manifest, cards: data3_cards } },
  "shaai-ow-upstream-noname": { package: package4, payload: { manifest: package4.manifest, cards: data4_cards } },
  "shaai-xianjian-upstream-noname": { package: package5, payload: { manifest: package5.manifest, cards: data5_cards } },
  "shahai-gujian-upstream-noname": { package: package6, payload: { manifest: package6.manifest, cards: data6_cards } },
  "shahai-gwent-upstream-noname": { package: package7, payload: { manifest: package7.manifest, cards: data7_cards } },
  "shahai-hearth-upstream-noname": { package: package8, payload: { manifest: package8.manifest, cards: data8_cards } },
  "shahai-mtg-upstream-noname": { package: package9, payload: { manifest: package9.manifest, cards: data9_cards } },
  "shahai-swd-upstream-noname": { package: package10, payload: { manifest: package10.manifest, cards: data10_cards } },
  "shahai-yunchou-upstream-noname": { package: package11, payload: { manifest: package11.manifest, cards: data11_cards, characters: data11_characters } },
  "shahai-zhenfa-upstream-noname": { package: package12, payload: { manifest: package12.manifest, cards: data12_cards } },
  "wudian-upstream-noname": { package: package13, payload: { manifest: package13.manifest, cards: data13_cards } },
  "wuxing-upstream-liuli": { package: package14, payload: { manifest: package14.manifest, cards: data14_cards } },
};

export function validateNativePackage(value: unknown): NativePackagePayload {
  const payload = value as NativePackagePayload | null;
  const manifest = payload?.manifest;
  if (!manifest || ![manifest.namespace, manifest.package_id, manifest.version]
    .every((value) => typeof value === "string" && value.trim().length > 0
      && value !== "." && value !== ".." && !value.includes("/") && !value.includes("\\"))
    || !Array.isArray(manifest.logic_hash) || manifest.logic_hash.length !== 32
    || !manifest.logic_hash.every((n) => Number.isInteger(n) && n >= 0 && n <= 255)) {
    throw new Error("扩展包身份或逻辑指纹无效");
  }
  const assets = [payload?.cards, payload?.characters, payload?.pile].filter((v) => v !== undefined);
  if (!assets.length || assets.some((v) => v === null || typeof v !== "object" || Array.isArray(v))) {
    throw new Error("扩展包缺少可装载的卡牌、武将或牌堆数据");
  }
  return payload!;
}

export function nativePackageKey(payload: NativePackagePayload): string {
  return `${payload.manifest.namespace}/${payload.manifest.package_id}`;
}

export function nativePackageFingerprint(payload: NativePackagePayload): string {
  return payload.manifest.logic_hash.map((n) => n.toString(16).padStart(2, "0")).join("");
}

export function bundledNativePackage(entry: ExtensionCatalogEntry): string {
  const source = bundled[entry.dir];
  if (!source) throw new Error(`无可安装包数据: ${entry.dir}`);
  if (!Array.isArray(source.package.modules) || source.package.modules.length) {
    throw new Error(`此客户端尚不支持安装含 VM 模块的包: ${entry.dir}`);
  }
  const payload = validateNativePackage(source.payload);
  if (nativePackageKey(payload) !== `${entry.namespace}/${entry.packageId}`
    || payload.manifest.version !== entry.version
    || !nativePackageFingerprint(payload).startsWith(entry.logicFingerprint)) {
    throw new Error(`包目录与实际安装内容不一致: ${entry.dir}`);
  }
  return JSON.stringify(payload);
}
