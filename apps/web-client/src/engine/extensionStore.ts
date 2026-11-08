/**
 * 扩展包生命周期客户端存储（对齐宿主契约：安装/启停/升级/卸载）。
 *
 * - 发现源：EXTENSION_CATALOG（15 包）；另有正式导入入口 importExtensionPackage
 *   （作者产物完整 npack JSON，可含 VM 模块，无需重编译客户端）
 * - 持久化：localStorage（浏览器）/ 同源键（桌面壳共用）
 * - 在局锁定：active 集快照写入 contentLock；卸载不破坏锁内旧局续玩标记
 * - 正式装载：安装副本（manifest/卡牌/武将/牌堆）经 WorkerRequest.packsJson 交给
 *   NativeEngine.new_with_packs_json / restore_hex_with_packs_json
 */
import type { ContentLock, FrozenPackage } from "../types/engine";
import {
  EXTENSION_CATALOG,
  extensionKey,
  type ExtensionCatalogEntry,
} from "./extensionCatalog";
import {
  bundledFrozenPackage,
  bundledNativePackage,
  nativePackageFingerprint,
  nativePackageKey,
  validateNativePackage,
} from "./bundledNativePackages";

const STORAGE_KEY = "noname-next.extension-store.v1";
const SAVE_SLOTS_KEY = "noname-next.native-saves.v1";

export type ExtensionLifecycle =
  | "available"
  | "installed"
  | "active"
  | "disabled"
  | "revoked";

export interface InstalledExtensionRecord {
  key: string;
  dir: string;
  packageId: string;
  namespace: string;
  /** 当前安装版本 */
  version: string;
  /** 历史版本（升级保留，供旧局续玩/回退） */
  history: string[];
  lifecycle: ExtensionLifecycle;
  installedAt: string;
  updatedAt: string;
  /** 逻辑指纹（防错版） */
  logicFingerprint: string;
  /** 按版本保留安装时的包字节；不会在开局时回读分发源。 */
  nativeVersions?: Record<string, string>;
  /** R05 v4/v5：按版本保留原始 `package.json` 文本、声明资产字节与完整 `LockedPackage` 绑定。 */
  nativeFrozenVersions?: Record<string, FrozenPackage>;
  /**
   * 签名状态（裁决第 12 项）：本地正式导入的包一律 `"unsigned"`——本地验签
   * 暂缓，先定客户端信任模型；内置目录包不带此字段。UI 据此标示「未签名」。
   */
  signature?: "unsigned";
}

export interface ExtensionStoreSnapshot {
  installed: InstalledExtensionRecord[];
  /** 撤销记录独立保留，卸载不能重新授权旧局或重装。 */
  revokedKeys?: string[];
  /** 已撤销的原始包 content_hash 列表（卸载后仍阻止同 content_hash 旧局恢复或重装）。 */
  revokedContentHashes?: string[];
  /** 开局内容锁：启用中的 key 有序列表 */
  contentLock: string[];
  /** 在局冻结：对局中不可卸载改锁（客户端闸） */
  matchFrozen: boolean;
}

function emptySnap(): ExtensionStoreSnapshot {
  return { installed: [], contentLock: [], matchFrozen: false };
}

function loadRaw(): ExtensionStoreSnapshot {
  if (typeof localStorage === "undefined") return emptySnap();
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return emptySnap();
    return { ...emptySnap(), ...JSON.parse(raw) } as ExtensionStoreSnapshot;
  } catch {
    return emptySnap();
  }
}

function saveRaw(snap: ExtensionStoreSnapshot) {
  if (typeof localStorage === "undefined") return;
  localStorage.setItem(STORAGE_KEY, JSON.stringify(snap));
}

function nowIso(): string {
  return new Date().toISOString();
}

function catalogByKey(key: string): ExtensionCatalogEntry | undefined {
  return EXTENSION_CATALOG.find((e) => extensionKey(e) === key);
}

function checkVersionCopy(versions: Record<string, string> | undefined, version: string, payload: string): void {
  const existing = versions?.[version];
  if (existing === undefined) return;
  const canonical = (json: string) => JSON.stringify(JSON.parse(json), (_key, value: unknown) => {
    if (value === null || typeof value !== "object" || Array.isArray(value)) return value;
    return Object.fromEntries(Object.entries(value).sort(([a], [b]) => a < b ? -1 : a > b ? 1 : 0));
  });
  if (canonical(existing) !== canonical(payload)) throw new Error(`同版本包内容冲突: ${version}`);
}

function checkFrozenVersionCopy(
  frozenVersions: Record<string, FrozenPackage> | undefined,
  version: string,
  frozen: FrozenPackage,
): void {
  const existing = frozenVersions?.[version];
  if (existing === undefined) return;
  if (
    existing.package_json !== frozen.package_json ||
    JSON.stringify(existing.binding) !== JSON.stringify(frozen.binding) ||
    JSON.stringify(existing.assets) !== JSON.stringify(frozen.assets)
  ) {
    throw new Error(`同版本包内容冲突: ${version}`);
  }
}

export function listCatalog(): ExtensionCatalogEntry[] {
  return EXTENSION_CATALOG.slice();
}

export function readExtensionStore(): ExtensionStoreSnapshot {
  return loadRaw();
}

export function setMatchFrozen(frozen: boolean): ExtensionStoreSnapshot {
  const snap = loadRaw();
  snap.matchFrozen = frozen;
  saveRaw(snap);
  return snap;
}

/** 安装：从目录装入本地存储（幂等；同版本重装刷新时间戳）。 */
export function installExtension(key: string): ExtensionStoreSnapshot {
  const cat = catalogByKey(key);
  if (!cat) throw new Error(`未知扩展: ${key}`);
  const snap = loadRaw();
  if (snap.matchFrozen) throw new Error("对局中不可安装或替换包内容");
  if (snap.revokedKeys?.includes(key)) throw new Error(`已撤销包不可重装: ${key}`);
  const payload = bundledNativePackage(cat);
  const frozen = bundledFrozenPackage(cat);
  if (snap.revokedContentHashes?.includes(frozen.binding.content_hash)) {
    throw new Error(`已撤销包不可重装: ${key}`);
  }
  const existing = snap.installed.find((r) => r.key === key);
  const t = nowIso();
  if (existing) {
    if (existing.lifecycle === "revoked") {
      throw new Error(`已撤销包不可重装: ${key}（需清除撤销记录）`);
    }
    checkVersionCopy(existing.nativeVersions, cat.version, payload);
    checkFrozenVersionCopy(existing.nativeFrozenVersions, cat.version, frozen);
    existing.updatedAt = t;
    existing.nativeVersions ??= {};
    if (existing.version === cat.version && !existing.nativeVersions[cat.version]) {
      existing.nativeVersions[cat.version] = payload;
    }
    existing.nativeFrozenVersions ??= {};
    if (existing.version === cat.version && !existing.nativeFrozenVersions[cat.version]) {
      existing.nativeFrozenVersions[cat.version] = frozen;
    }
    if (existing.lifecycle === "available") existing.lifecycle = "installed";
  } else {
    snap.installed.push({
      key,
      dir: cat.dir,
      packageId: cat.packageId,
      namespace: cat.namespace,
      version: cat.version,
      history: [cat.version],
      lifecycle: "installed",
      installedAt: t,
      updatedAt: t,
      logicFingerprint: cat.logicFingerprint,
      nativeVersions: { [cat.version]: payload },
      nativeFrozenVersions: { [cat.version]: frozen },
    });
  }
  saveRaw(snap);
  return snap;
}

/**
 * 正式导入：作者产物（完整 npack JSON，可含 VM 模块与卡牌/武将/牌堆资产）经
 * 校验后装入本地存储，不依赖内置目录、不要求重编译客户端。
 * - 同包同版本重复导入须内容一致，否则拒绝（不覆盖已安装副本）；
 * - 同包其他版本按升级/切换处理：导入版本成为当前版本，先前版本进 history，
 *   其安装副本保留供回退与旧局恢复；
 * - 任何校验失败都不改写存储；已撤销的包不可重装；对局冻结期间不可导入。
 * - 本地导入的包一律记为**未签名**（`signature: "unsigned"`，裁决第 12 项：
 *   本地验签暂缓，先定客户端信任模型；UI 据此标示）。
 *
 * 两层校验：本函数只做**结构层**（JSON/身份字段/模块形状/索引引用）；
 * 字节码语义、包/模块 logic_hash 与宿主 ABI 窗口由 Rust 校验器裁决——产品
 * 路径用 `importExtensionPackageValidated` 在**导入期**经 Worker
 * `validatePackage` 请求完成（GT-014，与服务端导入期拒绝一致），开局装载
 * 时仍会再校验一次。
 */
export function importExtensionPackage(json: string): ExtensionStoreSnapshot {
  return commitImport(prepareImport(json));
}

/**
 * 导入期深层校验入口（GT-014，裁决第 12 项）：结构层通过后，把原始 JSON 文本交
 * `deepValidate`（产品路径为 Worker 内的 Rust 校验器）裁决；拒绝即抛出且
 * **不改写存储**；通过才提交。`deepValidate` 抛出的错误原样透传。
 */
export async function importExtensionPackageValidated(
  json: string,
  deepValidate: (rawPackJson: string) => Promise<FrozenPackage | void>,
): Promise<ExtensionStoreSnapshot> {
  const prepared = prepareImport(json);
  const validated = await deepValidate(json);
  if (validated) {
    const fullLogicHash = nativePackageFingerprint(prepared.payload);
    if (
      validated.package_json !== prepared.originalPackageJson ||
      validated.binding.namespace !== prepared.payload.manifest.namespace ||
      validated.binding.package_id !== prepared.payload.manifest.package_id ||
      validated.binding.version !== prepared.version ||
      validated.binding.logic_hash !== fullLogicHash ||
      !/^[a-f0-9]{64}$/.test(validated.binding.content_hash)
    ) {
      throw new Error("深层校验返回的 FrozenPackage 与原始包原文或身份不匹配");
    }
    prepared.frozen = validated;
  }
  return commitImport(prepared);
}

interface PreparedImport {
  payload: ReturnType<typeof validateNativePackage>;
  key: string;
  version: string;
  canonical: string;
  originalPackageJson: string;
  fingerprint: string;
  frozen?: FrozenPackage;
}

/** 结构层解析 + 存储前置条件（不写存储）。 */
function prepareImport(json: string): PreparedImport {
  const snap = loadRaw();
  if (snap.matchFrozen) throw new Error("对局中不可安装或替换包内容");
  if (typeof json !== "string" || json.startsWith("\uFEFF")) {
    throw new Error("扩展包不是合法 JSON（不接受 UTF-8 BOM）");
  }
  const originalPackageJson = json;
  let parsed: unknown;
  try {
    parsed = JSON.parse(originalPackageJson);
  } catch {
    throw new Error("扩展包不是合法 JSON");
  }
  const payload = validateNativePackage(parsed);
  const key = nativePackageKey(payload);
  const version = payload.manifest.version;
  const canonical = JSON.stringify(payload);
  checkImportPreconditions(snap, key, version, canonical, originalPackageJson);
  return {
    payload,
    key,
    version,
    canonical,
    originalPackageJson,
    fingerprint: nativePackageFingerprint(payload).slice(0, 8),
  };
}

/** 前置条件在准备与提交时各查一次（深层校验为异步，期间存储可能变化）。 */
function checkImportPreconditions(
  snap: ExtensionStoreSnapshot,
  key: string,
  version: string,
  canonical: string,
  originalPackageJson: string,
  frozen?: FrozenPackage,
): void {
  if (snap.matchFrozen) throw new Error("对局中不可安装或替换包内容");
  if (snap.revokedKeys?.includes(key)) throw new Error(`已撤销包不可重装: ${key}`);
  if (frozen && snap.revokedContentHashes?.includes(frozen.binding.content_hash)) {
    throw new Error(`已撤销包不可重装: ${key}`);
  }
  const existing = snap.installed.find((r) => r.key === key);
  if (existing?.lifecycle === "revoked") {
    throw new Error(`已撤销包不可重装: ${key}（需清除撤销记录）`);
  }
  if (existing) {
    checkVersionCopy(existing.nativeVersions, version, canonical);
    if (frozen) {
      checkFrozenVersionCopy(existing.nativeFrozenVersions, version, frozen);
    } else if (
      existing.nativeFrozenVersions?.[version] &&
      existing.nativeFrozenVersions[version].package_json !== originalPackageJson
    ) {
      throw new Error(`同版本包内容冲突: ${version}`);
    }
  }
}

/** 提交：重读存储并复核前置条件后写入记录。 */
function commitImport(p: PreparedImport): ExtensionStoreSnapshot {
  const snap = loadRaw();
  const { payload, key, version, canonical, originalPackageJson, fingerprint, frozen } = p;
  checkImportPreconditions(snap, key, version, canonical, originalPackageJson, frozen);
  const existing = snap.installed.find((r) => r.key === key);
  const t = nowIso();
  if (existing) {
    existing.nativeVersions ??= {};
    existing.nativeVersions[version] ??= canonical;
    if (frozen) {
      existing.nativeFrozenVersions ??= {};
      existing.nativeFrozenVersions[version] ??= frozen;
    }
    if (existing.version !== version) {
      if (!existing.history.includes(existing.version)) existing.history.push(existing.version);
      existing.version = version;
      if (!existing.history.includes(version)) existing.history.push(version);
    }
    existing.logicFingerprint = fingerprint;
    existing.updatedAt = t;
    existing.signature = "unsigned";
    if (existing.lifecycle === "available") existing.lifecycle = "installed";
  } else {
    snap.installed.push({
      key,
      dir: "imported",
      packageId: payload.manifest.package_id,
      namespace: payload.manifest.namespace,
      version,
      history: [version],
      lifecycle: "installed",
      installedAt: t,
      updatedAt: t,
      logicFingerprint: fingerprint,
      nativeVersions: { [version]: canonical },
      ...(frozen ? { nativeFrozenVersions: { [version]: frozen } } : {}),
      signature: "unsigned",
    });
  }
  saveRaw(snap);
  return snap;
}

/** 启用：进入 contentLock（依赖图简化为单包；冲突同 packageId 异命名空间并存允许）。 */
export function activateExtension(key: string): ExtensionStoreSnapshot {
  const snap = loadRaw();
  if (snap.matchFrozen) throw new Error("对局中不可变更内容锁");
  const rec = snap.installed.find((r) => r.key === key);
  if (!rec || rec.lifecycle === "revoked") throw new Error(`未安装或已撤销: ${key}`);
  rec.lifecycle = "active";
  rec.updatedAt = nowIso();
  if (!snap.contentLock.includes(key)) snap.contentLock.push(key);
  snap.contentLock.sort();
  saveRaw(snap);
  return snap;
}

/** 停用：移出内容锁，保留安装记录。 */
export function deactivateExtension(key: string): ExtensionStoreSnapshot {
  const snap = loadRaw();
  if (snap.matchFrozen) throw new Error("对局中不可变更内容锁");
  const rec = snap.installed.find((r) => r.key === key);
  if (!rec) throw new Error(`未安装: ${key}`);
  if (rec.lifecycle === "revoked") throw new Error(`已撤销: ${key}`);
  rec.lifecycle = "disabled";
  rec.updatedAt = nowIso();
  snap.contentLock = snap.contentLock.filter((k) => k !== key);
  saveRaw(snap);
  return snap;
}

/**
 * 升级：目录版本写入当前；旧版本进 history（旧局续玩可回指）。
 * 同版本视为刷新；若目录版本已在 history 且等于当前则幂等。
 */
export function upgradeExtension(key: string, toVersion?: string): ExtensionStoreSnapshot {
  const cat = catalogByKey(key);
  const snap = loadRaw();
  if (!cat) {
    if (snap.installed.some((r) => r.key === key)) {
      throw new Error(`导入包没有内置目录版本，请导入新版本包以升级: ${key}`);
    }
    throw new Error(`未知扩展: ${key}`);
  }
  if (snap.matchFrozen) throw new Error("对局中不可升级（避免内容锁漂移）");
  const rec = snap.installed.find((r) => r.key === key);
  if (!rec || rec.lifecycle === "revoked") throw new Error(`未安装或已撤销: ${key}`);
  const next = toVersion ?? cat.version;
  if (next !== cat.version) throw new Error(`无可安装的版本 ${next}`);
  const payload = bundledNativePackage(cat);
  const frozen = bundledFrozenPackage(cat);
  checkVersionCopy(rec.nativeVersions, next, payload);
  checkFrozenVersionCopy(rec.nativeFrozenVersions, next, frozen);
  rec.nativeVersions ??= {};
  rec.nativeVersions[next] ??= payload;
  rec.nativeFrozenVersions ??= {};
  rec.nativeFrozenVersions[next] ??= frozen;
  if (rec.version !== next) {
    if (!rec.history.includes(rec.version)) rec.history.push(rec.version);
    rec.version = next;
    if (!rec.history.includes(next)) rec.history.push(next);
  }
  rec.logicFingerprint = cat.logicFingerprint;
  rec.updatedAt = nowIso();
  if (rec.lifecycle === "installed" || rec.lifecycle === "disabled") {
    /* 保持启停态 */
  }
  saveRaw(snap);
  return snap;
}

/** 回退到历史版本（须已在 history）。 */
export function rollbackExtension(key: string, version: string): ExtensionStoreSnapshot {
  const snap = loadRaw();
  if (snap.matchFrozen) throw new Error("对局中不可回退");
  const rec = snap.installed.find((r) => r.key === key);
  if (!rec || rec.lifecycle === "revoked") throw new Error(`未安装或已撤销: ${key}`);
  if (!rec.history.includes(version)) throw new Error(`无历史版本 ${version}`);
  if (!rec.nativeVersions?.[version]) throw new Error(`缺少已安装的版本内容 ${version}`);
  const payload = validateNativePackage(JSON.parse(rec.nativeVersions[version]));
  if (nativePackageKey(payload) !== key || payload.manifest.version !== version) {
    throw new Error(`历史包身份或版本不匹配: ${key}`);
  }
  const frozen = rec.nativeFrozenVersions?.[version];
  if (
    frozen &&
    (`${frozen.binding.namespace}/${frozen.binding.package_id}` !== key ||
      frozen.binding.version !== version)
  ) {
    throw new Error(`历史包身份或版本不匹配: ${key}`);
  }
  rec.version = version;
  rec.logicFingerprint = nativePackageFingerprint(payload).slice(0, 8);
  rec.updatedAt = nowIso();
  saveRaw(snap);
  return snap;
}

/** 卸载：移出锁与安装表；若曾启用则标记需重新 import（不静默）。 */
export function uninstallExtension(key: string): ExtensionStoreSnapshot {
  const snap = loadRaw();
  if (snap.matchFrozen) throw new Error("对局中不可卸载");
  const revokedRec = snap.installed.find((r) => r.key === key && r.lifecycle === "revoked");
  if (revokedRec) {
    snap.revokedKeys ??= [];
    if (!snap.revokedKeys.includes(key)) snap.revokedKeys.push(key);
    if (revokedRec.nativeFrozenVersions) {
      snap.revokedContentHashes ??= [];
      for (const f of Object.values(revokedRec.nativeFrozenVersions)) {
        if (!snap.revokedContentHashes.includes(f.binding.content_hash)) {
          snap.revokedContentHashes.push(f.binding.content_hash);
        }
      }
    }
  }
  snap.contentLock = snap.contentLock.filter((k) => k !== key);
  snap.installed = snap.installed.filter((r) => r.key !== key);
  saveRaw(snap);
  return snap;
}

/** 撤销：保留记录但禁止激活（对齐签名信任链 Revoked 守卫语义的客户端闸）。 */
export function revokeExtension(key: string): ExtensionStoreSnapshot {
  const snap = loadRaw();
  if (snap.matchFrozen) throw new Error("对局中不可撤销");
  const rec = snap.installed.find((r) => r.key === key);
  if (!rec) throw new Error(`未安装: ${key}`);
  rec.lifecycle = "revoked";
  snap.revokedKeys ??= [];
  if (!snap.revokedKeys.includes(key)) snap.revokedKeys.push(key);
  if (rec.nativeFrozenVersions) {
    snap.revokedContentHashes ??= [];
    for (const f of Object.values(rec.nativeFrozenVersions)) {
      if (!snap.revokedContentHashes.includes(f.binding.content_hash)) {
        snap.revokedContentHashes.push(f.binding.content_hash);
      }
    }
  }
  rec.updatedAt = nowIso();
  snap.contentLock = snap.contentLock.filter((k) => k !== key);
  saveRaw(snap);
  return snap;
}

/** 内容锁摘要（联机/开局展示）。 */
export function contentLockSummary(snap = loadRaw()): {
  packages: Array<{ key: string; version: string; fingerprint: string }>;
  hashSeed: string;
} {
  const packages = snap.contentLock
    .map((key) => {
      const rec = snap.installed.find((r) => r.key === key);
      if (!rec) return null;
      return {
        key,
        version: rec.version,
        fingerprint: rec.logicFingerprint,
      };
    })
    .filter(Boolean) as Array<{ key: string; version: string; fingerprint: string }>;
  const hashSeed = packages.map((p) => `${p.key}@${p.version}:${p.fingerprint}`).join("|");
  return { packages, hashSeed };
}

/** 一键安装并启用全部 15 包（开发/验收用）。 */
export function installAndActivateAll(): ExtensionStoreSnapshot {
  let snap = loadRaw();
  for (const e of EXTENSION_CATALOG) {
    const key = extensionKey(e);
    snap = installExtension(key);
    snap = activateExtension(key);
  }
  return snap;
}

/** 原生存档槽（localStorage；与扩展内容锁摘要一并保存，供 G5 旧版会话面）。 */
export interface NativeSaveSlot {
  id: string;
  label: string;
  hex: string;
  seed: number;
  players: number;
  contentLock: string[];
  lockSeed: string;
  savedAt: string;
  revision: number;
  /** 完整的固定包副本：当前启用集变化或卸载后仍可恢复旧局。 */
  packsJson?: string;
  /** R05 v4/v5：规范聚合内容锁。 */
  canonical_locked_content?: ContentLock;
  /** R05 v4/v5：原始字节冻结包列表。 */
  frozen_packages?: FrozenPackage[];
  /** 裁决 34：扩展渲染面与单机战役脚本（金币、太虚幻境、手杀 MVP、剑阁闯关、龙血道心等）完整快照 JSON。 */
  extensionSurfacesJson?: string;
}

export function listNativeSaves(): NativeSaveSlot[] {
  if (typeof localStorage === "undefined") return [];
  try {
    const raw = localStorage.getItem(SAVE_SLOTS_KEY);
    return raw ? (JSON.parse(raw) as NativeSaveSlot[]) : [];
  } catch {
    return [];
  }
}

export function putNativeSave(slot: NativeSaveSlot): NativeSaveSlot[] {
  const all = listNativeSaves().filter((s) => s.id !== slot.id);
  all.unshift(slot);
  localStorage.setItem(SAVE_SLOTS_KEY, JSON.stringify(all.slice(0, 20)));
  return all;
}

export function removeNativeSave(id: string): NativeSaveSlot[] {
  const all = listNativeSaves().filter((s) => s.id !== id);
  localStorage.setItem(SAVE_SLOTS_KEY, JSON.stringify(all));
  return all;
}
