/**
 * 扩展包生命周期客户端存储（对齐宿主契约：安装/启停/升级/卸载）。
 *
 * - 发现源：EXTENSION_CATALOG（15 包）
 * - 持久化：localStorage（浏览器）/ 同源键（桌面壳共用）
 * - 在局锁定：active 集快照写入 contentLock；卸载不破坏锁内旧局续玩标记
 * - 正式装载：安装副本（manifest/卡牌/武将/牌堆）经 WorkerRequest.packsJson 交给
 *   NativeEngine.new_with_packs_json / restore_hex_with_packs_json
 */
import {
  EXTENSION_CATALOG,
  extensionKey,
  type ExtensionCatalogEntry,
} from "./extensionCatalog";
import { bundledNativePackage, nativePackageFingerprint, nativePackageKey, validateNativePackage } from "./bundledNativePackages";

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
}

export interface ExtensionStoreSnapshot {
  installed: InstalledExtensionRecord[];
  /** 撤销记录独立保留，卸载不能重新授权旧局或重装。 */
  revokedKeys?: string[];
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
  const existing = snap.installed.find((r) => r.key === key);
  const t = nowIso();
  if (existing) {
    if (existing.lifecycle === "revoked") {
      throw new Error(`已撤销包不可重装: ${key}（需清除撤销记录）`);
    }
    existing.updatedAt = t;
    existing.nativeVersions ??= {};
    if (existing.version === cat.version && !existing.nativeVersions[cat.version]) {
      existing.nativeVersions[cat.version] = payload;
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
  if (!cat) throw new Error(`未知扩展: ${key}`);
  const snap = loadRaw();
  if (snap.matchFrozen) throw new Error("对局中不可升级（避免内容锁漂移）");
  const rec = snap.installed.find((r) => r.key === key);
  if (!rec || rec.lifecycle === "revoked") throw new Error(`未安装或已撤销: ${key}`);
  const next = toVersion ?? cat.version;
  if (next !== cat.version) throw new Error(`无可安装的版本 ${next}`);
  const payload = bundledNativePackage(cat);
  rec.nativeVersions ??= {};
  rec.nativeVersions[next] ??= payload;
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
  if (snap.installed.some((r) => r.key === key && r.lifecycle === "revoked")) {
    snap.revokedKeys ??= [];
    if (!snap.revokedKeys.includes(key)) snap.revokedKeys.push(key);
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
