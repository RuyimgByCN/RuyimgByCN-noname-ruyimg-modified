/**
 * G1 转换包目录（15 个）——对齐 packages/extensions 与宿主契约装载面。
 * 客户端管理面以本目录为「可发现」源；真实装载入局仍待窄壳扩展 API。
 */
export interface ExtensionCatalogEntry {
  /** 目录名（packages/extensions/<dir>） */
  dir: string;
  packageId: string;
  namespace: string;
  version: string;
  /** 展示名 */
  title: string;
  /** 逻辑指纹前缀（目录识别，非完整哈希） */
  logicFingerprint: string;
}

/** 15 个 G1 转换包（2026-10-16 与仓库目录核对）。 */
export const EXTENSION_CATALOG: ExtensionCatalogEntry[] = [
  {
    dir: "cardpile-upstream-liuli",
    packageId: "cardpile",
    namespace: "upstream-liuli",
    version: "1.0.0",
    title: "牌堆（琉璃）",
    logicFingerprint: "8dd99eb2",
  },
  {
    dir: "cardpile-upstream-noname",
    packageId: "cardpile",
    namespace: "upstream-noname",
    version: "1.0.0",
    title: "牌堆（无名）",
    logicFingerprint: "83aa81c4",
  },
  {
    dir: "d3-upstream-noname",
    packageId: "d3-select",
    namespace: "upstream-noname",
    version: "1.0.0",
    title: "3D精选",
    logicFingerprint: "6f257a96",
  },
  {
    dir: "huanle-upstream-noname",
    packageId: "huanle-cards",
    namespace: "upstream-noname",
    version: "1.0.0",
    title: "欢乐卡牌",
    logicFingerprint: "6ae1907c",
  },
  {
    dir: "shaai-ow-upstream-noname",
    packageId: "shaai-ow",
    namespace: "upstream-noname",
    version: "1.0.0",
    title: "守望先锋",
    logicFingerprint: "6117a13e",
  },
  {
    dir: "shaai-xianjian-upstream-noname",
    packageId: "shaai-xianjian",
    namespace: "upstream-noname",
    version: "1.0.0",
    title: "仙剑奇侠传",
    logicFingerprint: "d9ee1863",
  },
  {
    dir: "shahai-gujian-upstream-noname",
    packageId: "shahai-gujian",
    namespace: "upstream-noname",
    version: "1.0.0",
    title: "古剑奇谭",
    logicFingerprint: "ae13018f",
  },
  {
    dir: "shahai-gwent-upstream-noname",
    packageId: "shahai-gwent",
    namespace: "upstream-noname",
    version: "1.0.0",
    title: "昆特牌",
    logicFingerprint: "0bd2569a",
  },
  {
    dir: "shahai-hearth-upstream-noname",
    packageId: "shahai-hearth",
    namespace: "upstream-noname",
    version: "1.0.0",
    title: "炉石传说",
    logicFingerprint: "deafc31c",
  },
  {
    dir: "shahai-mtg-upstream-noname",
    packageId: "shahai-mtg",
    namespace: "upstream-noname",
    version: "1.0.0",
    title: "万智牌",
    logicFingerprint: "6417be73",
  },
  {
    dir: "shahai-swd-upstream-noname",
    packageId: "shahai-swd",
    namespace: "upstream-noname",
    version: "1.0.0",
    title: "轩辕剑",
    logicFingerprint: "31ccea98",
  },
  {
    dir: "shahai-yunchou-upstream-noname",
    packageId: "shahai-yunchou",
    namespace: "upstream-noname",
    version: "1.0.0",
    title: "运筹帷幄",
    logicFingerprint: "57491089",
  },
  {
    dir: "shahai-zhenfa-upstream-noname",
    packageId: "shahai-zhenfa",
    namespace: "upstream-noname",
    version: "1.0.0",
    title: "阵法",
    logicFingerprint: "53945b47",
  },
  {
    dir: "wudian-upstream-noname",
    packageId: "wudian",
    namespace: "upstream-noname",
    version: "1.0.0",
    title: "玩点论杀",
    logicFingerprint: "e0d488b6",
  },
  {
    dir: "wuxing-upstream-liuli",
    packageId: "wuxing",
    namespace: "upstream-liuli",
    version: "1.0.0",
    title: "五行",
    logicFingerprint: "845ea6fa",
  },
];

/** 稳定键：同 package_id 跨命名空间可并存。 */
export function extensionKey(e: Pick<ExtensionCatalogEntry, "namespace" | "packageId">): string {
  return `${e.namespace}/${e.packageId}`;
}
