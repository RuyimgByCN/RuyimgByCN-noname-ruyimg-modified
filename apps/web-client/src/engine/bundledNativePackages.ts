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
    content_schema_version?: number;
    dependencies?: Array<{ namespace: string; package_id: string; version: string; logic_hash?: number[] }>;
  };
  cards?: unknown;
  characters?: unknown;
  pile?: unknown;
  modules?: unknown;
  skill_index?: unknown;
  trigger_index?: unknown;
  active_index?: unknown;
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

function validIdentity(value: unknown): value is string {
  return typeof value === "string" && value.trim().length > 0
    && value !== "." && value !== ".." && !value.includes("/") && !value.includes("\\");
}

function validHash(value: unknown): value is number[] {
  return Array.isArray(value) && value.length === 32
    && value.every((n) => Number.isInteger(n) && n >= 0 && n <= 255);
}

export function validateNativePackage(value: unknown): NativePackagePayload {
  const payload = value as NativePackagePayload | null;
  const manifest = payload?.manifest;
  if (!manifest || ![manifest.namespace, manifest.package_id, manifest.version].every(validIdentity)
    || !validHash(manifest.logic_hash)) {
    throw new Error("扩展包身份或逻辑指纹无效");
  }
  if (manifest.dependencies !== undefined) {
    if (!Array.isArray(manifest.dependencies)) throw new Error("扩展包依赖格式无效");
    const seen = new Set<string>();
    for (const dependency of manifest.dependencies) {
      if (!dependency || typeof dependency !== "object" || Array.isArray(dependency)
        || ![dependency.namespace, dependency.package_id, dependency.version].every(validIdentity)
        || (dependency.logic_hash !== undefined && !validHash(dependency.logic_hash))) {
        throw new Error("扩展包依赖格式无效");
      }
      const key = `${dependency.namespace}/${dependency.package_id}`;
      if (seen.has(key) || key === `${manifest.namespace}/${manifest.package_id}`) {
        throw new Error(`扩展包依赖重复或指向自身: ${key}`);
      }
      seen.add(key);
    }
  }
  validateNativeMechanismFields(payload);
  if (payload.modules === undefined && [payload.skill_index, payload.trigger_index, payload.active_index,
    (payload as unknown as Record<string, unknown>).modifier_index, (payload as unknown as Record<string, unknown>).view_as_index].some((entry) => entry !== undefined)) {
    throw new Error("扩展包缺少 modules: declaration-bearing partial npack requires modules");
  }
  if (payload.modules === undefined && payload.manifest.content_schema_version === 5) {
    throw new Error("schema5 declaration package requires modules");
  }
  if (payload?.modules !== undefined && !Array.isArray(payload.modules)) {
    throw new Error("扩展包 VM 模块格式无效");
  }
  if (payload?.modules !== undefined) validateVmModules(payload);
  const assets = [payload?.cards, payload?.characters, payload?.pile].filter((v) => v !== undefined);
  if ((!assets.length && payload?.modules === undefined)
    || assets.some((v) => v === null || typeof v !== "object" || Array.isArray(v))) {
    throw new Error("扩展包缺少可装载的卡牌、武将、牌堆或 VM 数据");
  }
  return payload!;
}

function nonNegativeInt(value: unknown): value is number {
  return Number.isInteger(value) && (value as number) >= 0;
}

/**
 * VM 模块与索引的结构校验：镜像 rule-vm 装载期的形状与 `entry_pc` 界限
 * （`verify.rs`：`entry_pc >= code.len()` 拒绝），让坏副本在安装入口即被拒。
 * 字节码语义、模块/包 logic_hash 与 ABI 窗口仍由开局时同一 Rust 校验器裁决，
 * 客户端不复刻规范序列化以免与内核漂移。
 */
function validateVmModules(payload: NativePackagePayload): void {
  const modules = payload.modules as unknown[];
  const codeLength = new Map<string, number>();
  for (const module of modules) {
    const m = module as Record<string, unknown> | null;
    if (!m || typeof m !== "object" || Array.isArray(m)) throw new Error("扩展包 VM 模块格式无效");
    if (!validIdentity(m.name) || codeLength.has(m.name)) throw new Error("扩展包 VM 模块名缺失或重复");
    const code = m.code;
    if (!Array.isArray(code) || !code.length
      || !code.every((b) => Number.isInteger(b) && (b as number) >= 0 && (b as number) <= 255)) {
      throw new Error(`扩展包 VM 模块字节码无效: ${m.name}`);
    }
    if (![m.local_count, m.max_stack, m.entry_pc, m.caps_mask].every(nonNegativeInt)
      || (m.entry_pc as number) >= code.length) {
      throw new Error(`扩展包 VM 模块入口或预算字段无效: ${m.name}`);
    }
    if (!validHash(m.logic_hash)) throw new Error(`扩展包 VM 模块指纹格式无效: ${m.name}`);
    codeLength.set(m.name, code.length);
  }
  const skills = new Set<number>();
  const skillIndex = payload.skill_index;
  if (skillIndex !== undefined) {
    if (!Array.isArray(skillIndex)) throw new Error("扩展包技能索引格式无效");
    for (const entry of skillIndex) {
      const e = entry as Record<string, unknown> | null;
      if (!e || typeof e !== "object" || !nonNegativeInt(e.skill_def_id) || !validIdentity(e.name)
        || skills.has(e.skill_def_id)) {
        throw new Error("扩展包技能索引条目无效或重复");
      }
      skills.add(e.skill_def_id);
    }
  }
  for (const [index, label] of [[payload.trigger_index, "触发"], [payload.active_index, "主动"]] as const) {
    if (index === undefined) continue;
    if (!Array.isArray(index)) throw new Error(`扩展包${label}索引格式无效`);
    for (const entry of index) {
      const e = entry as Record<string, unknown> | null;
      const native = e?.host_key !== undefined;
      if (native && (payload.manifest.content_schema_version !== 5 || typeof e?.host_key !== "string"
        || !e.host_key.startsWith("host:") || (e.module !== undefined && e.module !== "") || e.entry_pc !== 0)) {
        throw new Error("schema5 native executor conflict or invalid host_key");
      }
      if (!e || typeof e !== "object" || (!native && (typeof e.module !== "string" || !codeLength.has(e.module)))) {
        throw new Error(`扩展包${label}索引引用了不存在的 VM 模块`);
      }
      if (!nonNegativeInt(e.entry_pc) || (!native && e.entry_pc >= codeLength.get(e.module as string)!)
        || !nonNegativeInt(e.skill_def_id) || (skillIndex !== undefined && !skills.has(e.skill_def_id))) {
        throw new Error(`扩展包${label}索引入口或技能引用无效: ${e.module}`);
      }
    }
  }
}


function validateNativeMechanismFields(payload: NativePackagePayload): void {
  const schema = payload.manifest.content_schema_version;
  if (schema !== undefined && ![3, 4, 5].includes(schema)) throw new Error("unsupported content schema");
  const fields: Array<[unknown, string[], string]> = [
    [payload.skill_index, ["charlotte", "persevere", "forever", "temporary", "group_children", "reveal", "onremove"], "技能"],
    [payload.trigger_index, ["host_key", "force_die", "force_out", "repeat", "hidden", "event_skill"], "触发"],
    [payload.active_index, ["host_key"], "主动"],
  ];
  for (const [entries, names, label] of fields) {
    if (entries === undefined) continue;
    if (!Array.isArray(entries)) throw new Error(`扩展包${label}索引格式无效`);
    for (const entry of entries) {
      if (!entry || typeof entry !== "object") continue;
      for (const name of names) {
        if (!Object.hasOwn(entry, name)) continue;
        if (schema !== 5 || entry[name] == null) throw new Error(`schema5 field requires schema5: ${name}`);
        if (["charlotte", "persevere", "forever", "temporary", "force_die", "force_out"].includes(name)
          && typeof entry[name] !== "boolean") throw new Error(`schema5 literal boolean required: ${name}`);
        if (name === "group_children" && (!Array.isArray(entry[name]) || !entry[name].every(nonNegativeInt))) {
          throw new Error("invalid schema5 group_children");
        }
        if (name === "repeat" && !["once", "damage_amount", "damage_amount_else_one"].includes(entry[name])) throw new Error("invalid schema5 repeat");
        if (name === "hidden" && !["public_only", "ask_reveal", "run_hidden"].includes(entry[name])) throw new Error("invalid schema5 hidden");
      }
    }
  }
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
  if (!Array.isArray(source.package.modules)) throw new Error(`扩展包 VM 模块格式无效: ${entry.dir}`);
  // 含 VM 模块的包须整包装入（Worker 按完整 npack 桥接技能声明），并与导入
  // 入口走同一套结构校验；无模块的包维持「manifest + 资产」载荷不变。
  const raw = source.package.modules.length ? { ...source.package, ...source.payload as object } : source.payload;
  const payload = validateNativePackage(raw);
  if (nativePackageKey(payload) !== `${entry.namespace}/${entry.packageId}`
    || payload.manifest.version !== entry.version
    || !nativePackageFingerprint(payload).startsWith(entry.logicFingerprint)) {
    throw new Error(`包目录与实际安装内容不一致: ${entry.dir}`);
  }
  return JSON.stringify(payload);
}
