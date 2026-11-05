import type {
  ContentLock,
  FrozenPackage,
  LockedContentEnvelope,
  LockedPackage,
} from "../types/engine";
import {
  nativePackageFingerprint,
  nativePackageKey,
  validateNativePackage,
  type NativePackagePayload,
} from "./bundledNativePackages";
import {
  readExtensionStore,
  type ExtensionStoreSnapshot,
  type NativeSaveSlot,
} from "./extensionStore";

/** Rust `ContentLock::empty().lock_hash` (`blake3("noname-next/content-lock/v1")`). */
export const EMPTY_CONTENT_LOCK_HASH =
  "a6cd5606ddabd11575f25d991b9410418047ca7c2a91fbf1068c1cd747111415";

export interface NativeMatchContent {
  packsJson?: string;
  contentLock: string[];
  lockSeed: string;
  canonical_locked_content?: ContentLock;
  frozen_packages?: FrozenPackage[];
}

function snapshot(packs: NativePackagePayload[]): NativeMatchContent {
  const contentLock = packs.map(nativePackageKey);
  if (new Set(contentLock).size !== contentLock.length) throw new Error("内容锁包含重复扩展");
  const installed = new Map(packs.map((pack) => [nativePackageKey(pack), pack]));
  for (const pack of packs) for (const dependency of pack.manifest.dependencies ?? []) {
    const key = `${dependency.namespace}/${dependency.package_id}`;
    const target = installed.get(key);
    if (!target) throw new Error(`扩展包缺少依赖: ${key}@${dependency.version}`);
    const expected = dependency.logic_hash;
    if (target.manifest.version !== dependency.version
      || (expected?.some((n) => n !== 0)
        && expected.some((n, i) => n !== target.manifest.logic_hash[i]))) {
      throw new Error(`扩展包依赖版本或指纹不匹配: ${key}@${dependency.version}`);
    }
  }
  return {
    packsJson: packs.length ? JSON.stringify(packs) : undefined,
    contentLock,
    lockSeed: packs.map((p) => `${nativePackageKey(p)}@${p.manifest.version}:${nativePackageFingerprint(p)}`).join("|"),
  };
}

/** 正式开局只读安装副本；旧的纯元数据记录必须先在管理页升级/重新安装。 */
export function prepareNativeMatchContent(store = readExtensionStore()): NativeMatchContent {
  const packs = store.contentLock.map((key) => {
    const record = store.installed.find((entry) => entry.key === key);
    if (!record || record.lifecycle !== "active") throw new Error(`内容锁中的包未启用: ${key}`);
    const json = record.nativeVersions?.[record.version];
    if (!json) throw new Error(`缺少已安装包内容: ${key}，请先在扩展管理页升级或重新安装`);
    const payload = validateNativePackage(JSON.parse(json));
    if (nativePackageKey(payload) !== key || payload.manifest.version !== record.version
      || !/^[a-f0-9]{8,64}$/.test(record.logicFingerprint)
      || !nativePackageFingerprint(payload).startsWith(record.logicFingerprint)) {
      throw new Error(`已安装包版本或指纹不匹配: ${key}`);
    }
    return payload;
  });
  return snapshot(packs);
}

/** 存档携带原局副本；启用集变化/卸载不改写旧局，撤销仍阻止恢复。 */
export function restoreNativeMatchContent(
  save: Partial<
    Pick<
      NativeSaveSlot,
      "packsJson" | "contentLock" | "lockSeed" | "canonical_locked_content" | "frozen_packages"
    >
  >,
  store: ExtensionStoreSnapshot = readExtensionStore(),
): NativeMatchContent {
  if (
    save &&
    typeof save === "object" &&
    ("canonical_locked_content" in save || "frozen_packages" in save)
  ) {
    return restoreLockedNativeMatchContent(save, store);
  }
  const values: unknown = save.packsJson ? JSON.parse(save.packsJson) : [];
  if (!Array.isArray(values)) throw new Error("存档扩展内容必须是数组");
  const content = snapshot(values.map(validateNativePackage));
  if (content.lockSeed !== (save.lockSeed ?? "")
    || JSON.stringify(content.contentLock) !== JSON.stringify(save.contentLock ?? [])) {
    throw new Error("存档内容锁与包副本不匹配；旧扩展存档缺少实际包内容时不能恢复");
  }
  for (const key of content.contentLock) {
    if (store.revokedKeys?.includes(key)
      || store.installed.some((record) => record.key === key && record.lifecycle === "revoked")) {
      throw new Error(`存档包含已撤销包: ${key}`);
    }
  }
  return content;
}

function validIdentitySegment(value: unknown): value is string {
  return (
    typeof value === "string" &&
    value.trim().length > 0 &&
    value === value.trim() &&
    value !== "." &&
    value !== ".." &&
    !value.includes("/") &&
    !value.includes("\\")
  );
}

function validHex64(value: unknown): value is string {
  return typeof value === "string" && /^[a-f0-9]{64}$/.test(value);
}

function validateLockedBinding(binding: unknown): LockedPackage {
  if (!binding || typeof binding !== "object" || Array.isArray(binding)) {
    throw new Error("锁定包 binding 必须是对象");
  }
  const b = binding as LockedPackage;
  if (
    !validIdentitySegment(b.namespace) ||
    !validIdentitySegment(b.package_id) ||
    !validIdentitySegment(b.version) ||
    !validHex64(b.logic_hash) ||
    !validHex64(b.content_hash)
  ) {
    throw new Error("锁定包身份或 hash 字段无效");
  }
  return b;
}

export function validateFrozenPackageCopy(value: unknown): FrozenPackage {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new Error("frozen_packages 元素必须是对象");
  }
  const copy = value as FrozenPackage;
  const binding = validateLockedBinding(copy.binding);
  if (typeof copy.package_json !== "string" || copy.package_json.length === 0) {
    throw new Error("frozen_packages.package_json 必须是非空字符串");
  }
  if (copy.package_json.startsWith("\uFEFF")) {
    throw new Error("frozen_packages.package_json 不接受 UTF-8 BOM");
  }
  if (!copy.assets || typeof copy.assets !== "object" || Array.isArray(copy.assets)) {
    throw new Error("frozen_packages.assets 必须是对象");
  }
  for (const [relPath, bytes] of Object.entries(copy.assets)) {
    if (
      !relPath ||
      relPath.startsWith("/") ||
      relPath.includes("\\") ||
      relPath.split("/").some((seg) => !seg || seg === "." || seg === "..")
    ) {
      throw new Error(`原局包资产路径越界: ${relPath}`);
    }
    if (
      !Array.isArray(bytes) ||
      !bytes.every((b) => Number.isInteger(b) && b >= 0 && b <= 255)
    ) {
      throw new Error(`原局包资产字节无效: ${relPath}`);
    }
  }
  let parsed: Record<string, unknown>;
  try {
    parsed = JSON.parse(copy.package_json) as Record<string, unknown>;
  } catch {
    throw new Error("frozen_packages.package_json 不是合法 JSON");
  }
  const manifest = parsed?.manifest as NativePackagePayload["manifest"] | undefined;
  if (
    !manifest ||
    manifest.namespace !== binding.namespace ||
    manifest.package_id !== binding.package_id ||
    manifest.version !== binding.version ||
    !Array.isArray(manifest.logic_hash) ||
    manifest.logic_hash.map((n) => n.toString(16).padStart(2, "0")).join("") !==
      binding.logic_hash
  ) {
    throw new Error("frozen_packages.package_json 与 binding 不匹配");
  }
  return copy;
}

/**
 * R05 v4 §2/§3：从当前启用集选出已验证的 `FrozenPackage[]`（只读 `nativeFrozenVersions`；
 * 旧记录仅有 `nativeVersions` 时明确拒绝并提示重装原包产物）。
 */
export function selectFrozenPackagesForMatch(
  store: ExtensionStoreSnapshot = readExtensionStore(),
): FrozenPackage[] {
  if (new Set(store.contentLock).size !== store.contentLock.length) {
    throw new Error("内容锁包含重复扩展");
  }
  const copies = store.contentLock.map((key) => {
    const record = store.installed.find((entry) => entry.key === key);
    if (!record || record.lifecycle !== "active") {
      throw new Error(`内容锁中的包未启用: ${key}`);
    }
    const frozen = record.nativeFrozenVersions?.[record.version];
    if (!frozen) {
      if (record.nativeVersions?.[record.version]) {
        throw new Error(
          `扩展包缺少原始字节锁副本 (nativeFrozenVersions): ${key}，请重新安装或重新导入原包产物`,
        );
      }
      throw new Error(`缺少已安装包内容: ${key}，请先在扩展管理页升级或重新安装`);
    }
    const validated = validateFrozenPackageCopy(frozen);
    if (
      `${validated.binding.namespace}/${validated.binding.package_id}` !== key ||
      validated.binding.version !== record.version ||
      !validated.binding.logic_hash.startsWith(record.logicFingerprint)
    ) {
      throw new Error(`已安装包版本或指纹不匹配: ${key}`);
    }
    return validated;
  });
  const bySlot = new Map(
    copies.map((c) => [`${c.binding.namespace}/${c.binding.package_id}`, c]),
  );
  for (const copy of copies) {
    const parsed = JSON.parse(copy.package_json) as NativePackagePayload;
    for (const dep of parsed.manifest.dependencies ?? []) {
      const depKey = `${dep.namespace}/${dep.package_id}`;
      const target = bySlot.get(depKey);
      if (!target) {
        throw new Error(`扩展包缺少依赖: ${depKey}@${dep.version}`);
      }
      const depHash = dep.logic_hash?.some((n) => n !== 0)
        ? dep.logic_hash.map((n) => n.toString(16).padStart(2, "0")).join("")
        : undefined;
      if (
        target.binding.version !== dep.version ||
        (depHash !== undefined && target.binding.logic_hash !== depHash)
      ) {
        throw new Error(`扩展包依赖版本或指纹不匹配: ${depKey}@${dep.version}`);
      }
    }
  }
  return copies;
}

/**
 * R05 v4 §2/§3：通过 Worker/IPC `prepareLockedContent` 生成本局规范 `ContentLock` 与 `FrozenPackage[]`。
 */
export async function prepareLockedNativeMatchContent(
  prepareLockedContentFn: (frozenPackagesJson: string) => Promise<LockedContentEnvelope>,
  store: ExtensionStoreSnapshot = readExtensionStore(),
): Promise<NativeMatchContent> {
  const copies = selectFrozenPackagesForMatch(store);
  const legacy = prepareNativeMatchContent(store);
  const envelope = await prepareLockedContentFn(JSON.stringify(copies));
  const validated = validateLockedEnvelope(
    envelope?.canonical_locked_content,
    envelope?.frozen_packages,
    store,
  );
  return {
    ...legacy,
    canonical_locked_content: validated.canonical_locked_content,
    frozen_packages: validated.frozen_packages,
  };
}

function validateLockedEnvelope(
  rawLock: unknown,
  rawFrozen: unknown,
  store: ExtensionStoreSnapshot,
): LockedContentEnvelope {
  if (!rawLock || typeof rawLock !== "object" || Array.isArray(rawLock)) {
    throw new Error("canonical_locked_content 必须是非空 ContentLock 对象");
  }
  const lock = rawLock as ContentLock;
  if (!Array.isArray(lock.packages) || !validHex64(lock.lock_hash)) {
    throw new Error("canonical_locked_content 缺少有效 packages 数组或 64 位 lock_hash");
  }
  if (!Array.isArray(rawFrozen)) {
    throw new Error("frozen_packages 必须是数组");
  }
  if (lock.packages.length !== rawFrozen.length) {
    throw new Error("canonical_locked_content.packages 与 frozen_packages 数量不一致");
  }
  if (lock.packages.length === 0 && lock.lock_hash !== EMPTY_CONTENT_LOCK_HASH) {
    throw new Error("标准空内容锁 lock_hash 不匹配 ContentLock::empty()");
  }
  const seenSlots = new Set<string>();
  const frozenPackages = rawFrozen.map(validateFrozenPackageCopy);
  for (let i = 0; i < lock.packages.length; i++) {
    const lp = validateLockedBinding(lock.packages[i]);
    const fp = frozenPackages[i]!;
    const key = `${lp.namespace}/${lp.package_id}`;
    if (seenSlots.has(key)) {
      throw new Error(`内容锁包含重复扩展槽位: ${key}`);
    }
    seenSlots.add(key);
    if (i > 0) {
      const prev = lock.packages[i - 1]!;
      const prevTuple = `${prev.namespace}\0${prev.package_id}\0${prev.version}`;
      const currTuple = `${lp.namespace}\0${lp.package_id}\0${lp.version}`;
      if (prevTuple >= currTuple) {
        throw new Error("canonical_locked_content.packages 不满足规范排序");
      }
    }
    if (
      lp.namespace !== fp.binding.namespace ||
      lp.package_id !== fp.binding.package_id ||
      lp.version !== fp.binding.version ||
      lp.logic_hash !== fp.binding.logic_hash ||
      lp.content_hash !== fp.binding.content_hash
    ) {
      throw new Error(`内容锁与 frozen_packages[${i}] binding 不匹配: ${key}`);
    }
    if (
      store.revokedKeys?.includes(key) ||
      store.revokedContentHashes?.includes(lp.content_hash) ||
      store.installed.some((r) => r.key === key && r.lifecycle === "revoked")
    ) {
      throw new Error(`存档包含已撤销包: ${key}`);
    }
  }
  return {
    canonical_locked_content: {
      packages: lock.packages.map((p) => ({ ...p })),
      lock_hash: lock.lock_hash,
    },
    frozen_packages: frozenPackages,
  };
}

/**
 * R05 v4 §4：严格按空值/新旧上下文政策表校验并恢复 `NativeSaveSlot`。
 */
export function restoreLockedNativeMatchContent(
  save: Partial<
    Pick<
      NativeSaveSlot,
      "packsJson" | "contentLock" | "lockSeed" | "canonical_locked_content" | "frozen_packages"
    >
  >,
  store: ExtensionStoreSnapshot = readExtensionStore(),
): NativeMatchContent {
  if (!save || typeof save !== "object") {
    throw new Error("存档槽位无效");
  }
  const hasCanonicalField = "canonical_locked_content" in save;
  const hasFrozenField = "frozen_packages" in save;
  if (!hasCanonicalField && !hasFrozenField) {
    // §4 第一行 vs 第七行：缺新字段时仅允许无任何扩展证据的 legacy 标准候选。
    const hasExtensionEvidence =
      (Array.isArray(save.contentLock) && save.contentLock.length > 0) ||
      (typeof save.lockSeed === "string" && save.lockSeed.length > 0) ||
      (save.packsJson !== undefined && save.packsJson !== "[]");
    if (hasExtensionEvidence) {
      throw new Error(
        "旧扩展存档缺少 canonical_locked_content 与 frozen_packages 原字节上下文，拒绝恢复",
      );
    }
    return {
      contentLock: [],
      lockSeed: "",
      packsJson: undefined,
      canonical_locked_content: {
        packages: [],
        lock_hash: EMPTY_CONTENT_LOCK_HASH,
      },
      frozen_packages: [],
    };
  }
  if (!hasCanonicalField || !hasFrozenField) {
    throw new Error("存档必须同时提供 canonical_locked_content 与 frozen_packages");
  }
  const validated = validateLockedEnvelope(
    save.canonical_locked_content,
    save.frozen_packages,
    store,
  );
  const contentLock = validated.canonical_locked_content.packages.map(
    (p) => `${p.namespace}/${p.package_id}`,
  );
  const lockSeed = validated.canonical_locked_content.packages
    .map((p) => `${p.namespace}/${p.package_id}@${p.version}:${p.logic_hash}`)
    .join("|");
  return {
    contentLock,
    lockSeed,
    packsJson: save.packsJson,
    canonical_locked_content: validated.canonical_locked_content,
    frozen_packages: validated.frozen_packages,
  };
}
