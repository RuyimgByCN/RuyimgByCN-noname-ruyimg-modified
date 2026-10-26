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

export interface NativeMatchContent {
  packsJson?: string;
  contentLock: string[];
  lockSeed: string;
}

function snapshot(packs: NativePackagePayload[]): NativeMatchContent {
  const contentLock = packs.map(nativePackageKey);
  if (new Set(contentLock).size !== contentLock.length) throw new Error("内容锁包含重复扩展");
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
  save: Pick<NativeSaveSlot, "packsJson" | "contentLock" | "lockSeed">,
  store: ExtensionStoreSnapshot = readExtensionStore(),
): NativeMatchContent {
  const values: unknown = save.packsJson ? JSON.parse(save.packsJson) : [];
  if (!Array.isArray(values)) throw new Error("存档扩展内容必须是数组");
  const content = snapshot(values.map(validateNativePackage));
  if (content.lockSeed !== save.lockSeed
    || JSON.stringify(content.contentLock) !== JSON.stringify(save.contentLock)) {
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
