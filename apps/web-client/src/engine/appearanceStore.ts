import { reactive } from "vue";
import type {
  ContentAssetRef,
  ContentCatalogCharacter,
  ContentCatalogPage,
  ContentCatalogSkin,
} from "../types/engine";
import {
  desktopContentCatalogPage,
  isTauriShell,
  resolveDesktopContentAsset,
} from "./desktopHost";

const PAGE_LIMIT = 200;

type CatalogSource =
  | { kind: "desktop"; key: "desktop" }
  | { kind: "http"; key: string; baseUrl: string };

export const appearanceState = reactive({
  contentLockHash: "",
  total: 0,
  pages: new Map<number, ContentCatalogPage>(),
  characters: new Map<number, ContentCatalogCharacter>(),
  skins: new Map<number, ContentCatalogSkin>(),
  translations: {} as Record<string, Record<string, string>>,
  selectedSkins: new Map<number, number | null>(),
  portraitUrls: new Map<number, string>(),
  imageUrls: new Map<number, string>(),
  substituteUrls: new Map<string, string>(),
});

let source: CatalogSource | null = null;
const pageRequests = new Map<number, Promise<ContentCatalogPage | null>>();
const characterRequests = new Map<
  number,
  Promise<ContentCatalogCharacter | undefined>
>();
const assetRequests = new Map<string, Promise<string | null>>();

export function configureLocalAppearanceCatalog(base?: string): void {
  if (isTauriShell()) {
    setSource({ kind: "desktop", key: "desktop" });
    return;
  }
  const baseUrl = base
    ? new URL(base, window.location.href).origin
    : `${window.location.protocol === "https:" ? "https:" : "http:"}//${
        window.location.hostname || "127.0.0.1"
      }:7700`;
  setSource({ kind: "http", key: `http:${baseUrl}`, baseUrl });
}

export function configureOnlineAppearanceCatalog(wsUrl: string): void {
  const url = new URL(wsUrl.trim(), window.location.href);
  url.protocol =
    url.protocol === "wss:" || url.protocol === "https:" ? "https:" : "http:";
  const baseUrl = url.origin;
  setSource({ kind: "http", key: `http:${baseUrl}`, baseUrl });
}

export async function ensureAppearanceCharacter(
  characterId: number
): Promise<ContentCatalogCharacter | undefined> {
  const cached = appearanceState.characters.get(characterId);
  if (cached) {
    if (!appearanceState.imageUrls.has(characterId)) {
      await refreshCharacterImage(characterId);
    }
    return cached;
  }
  if (!source) return undefined;

  const existing = characterRequests.get(characterId);
  if (existing) return existing;

  const sourceKey = source.key;
  const request = (async () => {
    let offset = 0;
    while (source?.key === sourceKey) {
      const page = await loadPage(offset);
      if (!page) break;
      const character = appearanceState.characters.get(characterId);
      if (character) {
        await refreshCharacterImage(characterId);
        return character;
      }
      const next = page.offset + page.characters.length;
      if (next >= page.total || next <= offset) break;
      offset = next;
    }
    return undefined;
  })();
  characterRequests.set(characterId, request);
  try {
    return await request;
  } finally {
    if (characterRequests.get(characterId) === request) {
      characterRequests.delete(characterId);
    }
  }
}

export function characterDisplayName(characterId: number): string | undefined {
  const key = appearanceState.characters.get(characterId)?.definition.name_key;
  return key ? translated(key) : undefined;
}

export function skinsForCharacter(characterId: number): ContentCatalogSkin[] {
  const character = appearanceState.characters.get(characterId);
  return (
    character?.skin_ids.flatMap((id) => {
      const skin = appearanceState.skins.get(id);
      return skin && skin.presentation.kind !== "substitute" ? [skin] : [];
    }) ?? []
  );
}

export function skinDisplayName(skin: ContentCatalogSkin): string {
  return translated(skin.name_key);
}

export function selectedSkinId(characterId: number): number | null {
  const character = appearanceState.characters.get(characterId);
  if (!character || !appearanceState.contentLockHash) return null;
  if (appearanceState.selectedSkins.has(characterId)) {
    return appearanceState.selectedSkins.get(characterId) ?? null;
  }
  const key = selectionStorageKey(characterId);
  let selected: number | null = null;
  if (key) {
    try {
      const value = Number(localStorage.getItem(key));
      if (
        Number.isInteger(value) &&
        selectableSkinIds(character).includes(value)
      ) {
        selected = value;
      }
    } catch {
      // Storage may be unavailable in hardened webviews.
    }
  }
  appearanceState.selectedSkins.set(characterId, selected);
  return selected;
}

export function selectSkin(characterId: number, skinId: number | null): void {
  const character = appearanceState.characters.get(characterId);
  if (skinId != null && (!character || !selectableSkinIds(character).includes(skinId))) {
    return;
  }
  appearanceState.selectedSkins.set(characterId, skinId);
  const key = selectionStorageKey(characterId);
  if (key) {
    try {
      if (skinId == null) localStorage.removeItem(key);
      else localStorage.setItem(key, String(skinId));
    } catch {
      // The in-memory selection still works.
    }
  }
  void refreshCharacterImage(characterId);
}

export function characterImageUrl(characterId: number): string {
  return (
    appearanceState.imageUrls.get(characterId) ??
    fallbackPortraitUrl(characterId)
  );
}

export function characterPortraitUrl(characterId: number): string {
  return (
    appearanceState.portraitUrls.get(characterId) ??
    fallbackPortraitUrl(characterId)
  );
}

export function substituteImageUrl(
  characterId: number,
  rule: "dead"
): string | null {
  return appearanceState.substituteUrls.get(`${characterId}:${rule}`) ?? null;
}

export function fallbackPortraitUrl(characterId: number): string {
  return `/original/portraits/${characterId}.jpg`;
}

async function loadPage(offset: number): Promise<ContentCatalogPage | null> {
  const cached = appearanceState.pages.get(offset);
  if (cached) return cached;
  const existing = pageRequests.get(offset);
  if (existing) return existing;
  const activeSource = source;
  if (!activeSource) return null;

  const request = (async () => {
    let page: unknown;
    if (activeSource.kind === "desktop") {
      page = await desktopContentCatalogPage(offset, PAGE_LIMIT);
    } else {
      const url = new URL("/content-catalog", activeSource.baseUrl);
      url.searchParams.set("offset", String(offset));
      url.searchParams.set("limit", String(PAGE_LIMIT));
      const response = await fetch(url);
      if (!response.ok) {
        throw new Error(`内容目录加载失败 (${response.status})`);
      }
      page = await response.json();
    }
    if (source !== activeSource || page == null) return null;
    assertCatalogPage(page);
    if (page.offset !== Math.min(offset, page.total)) {
      throw new Error("内容目录分页偏移不一致");
    }
    if (
      offset !== 0 &&
      appearanceState.contentLockHash &&
      appearanceState.contentLockHash !== page.content_lock_hash
    ) {
      clearCatalog();
      return loadPage(0);
    }
    ingestPage(page);
    return page;
  })();
  pageRequests.set(offset, request);
  try {
    return await request;
  } finally {
    if (pageRequests.get(offset) === request) pageRequests.delete(offset);
  }
}

function ingestPage(page: ContentCatalogPage): void {
  if (
    appearanceState.contentLockHash &&
    appearanceState.contentLockHash !== page.content_lock_hash
  ) {
    clearCatalog();
  }
  appearanceState.contentLockHash = page.content_lock_hash;
  appearanceState.total = page.total;
  appearanceState.pages.set(page.offset, page);
  for (const character of page.characters) {
    appearanceState.characters.set(character.definition.id, character);
  }
  for (const skin of page.skins) appearanceState.skins.set(skin.id, skin);
  for (const [locale, entries] of Object.entries(page.translations)) {
    appearanceState.translations[locale] = {
      ...appearanceState.translations[locale],
      ...entries,
    };
  }
}

async function refreshCharacterImage(characterId: number): Promise<void> {
  const character = appearanceState.characters.get(characterId);
  if (!character) return;
  const selected = selectedSkinId(characterId);
  const skin = selected == null ? undefined : appearanceState.skins.get(selected);
  const skinAsset =
    skin?.presentation.kind === "static" ? skin.presentation.asset : null;
  const fallback = fallbackPortraitUrl(characterId);
  appearanceState.portraitUrls.set(characterId, fallback);
  appearanceState.imageUrls.set(characterId, fallback);

  const selection = `${appearanceState.contentLockHash}:${selected ?? "base"}`;
  const portrait = character.portrait_asset
    ? await resolveAsset(character.portrait_asset).catch(() => null)
    : null;
  const resolvedSkin = skinAsset
    ? await resolveAsset(skinAsset).catch(() => null)
    : null;
  const deadSubstitute = character.skin_ids
    .map((id) => appearanceState.skins.get(id))
    .find(
      (candidate) =>
        candidate?.presentation.kind === "substitute" &&
        candidate.presentation.rule === "dead"
    );
  const deadAsset =
    deadSubstitute?.presentation.kind === "substitute"
      ? deadSubstitute.presentation.asset
      : null;
  const resolvedDead = deadAsset
    ? await resolveAsset(deadAsset).catch(() => null)
    : null;
  if (
    selection ===
      `${appearanceState.contentLockHash}:${selectedSkinId(characterId) ?? "base"}`
  ) {
    const portraitUrl = portrait ?? fallback;
    appearanceState.portraitUrls.set(characterId, portraitUrl);
    appearanceState.imageUrls.set(characterId, resolvedSkin ?? portraitUrl);
    if (resolvedDead) {
      appearanceState.substituteUrls.set(`${characterId}:dead`, resolvedDead);
    }
  }
}

function resolveAsset(asset: ContentAssetRef): Promise<string | null> {
  if (!validAssetRef(asset) || !source) return Promise.resolve(null);
  const activeSource = source;
  const key = [
    activeSource.key,
    asset.namespace,
    asset.package_id,
    asset.asset_id,
  ].join("\0");
  const cached = assetRequests.get(key);
  if (cached) return cached;
  const request =
    activeSource.kind === "desktop"
      ? resolveDesktopContentAsset(asset)
      : Promise.resolve(
          `${activeSource.baseUrl}/package-assets/${[
            asset.namespace,
            asset.package_id,
          ]
            .map(encodeURIComponent)
            .join("/")}/${asset.asset_id
            .split("/")
            .map(encodeURIComponent)
            .join("/")}?v=${encodeURIComponent(
            appearanceState.contentLockHash
          )}`
        );
  assetRequests.set(key, request);
  return request;
}

export function skillDisplayName(name: string): string {
  return translated(name);
}

export function currentAppearanceBaseUrl(): string | null {
  if (!source) return null;
  if (source.kind === "http") return source.baseUrl;
  return "http://127.0.0.1:7700";
}

export function currentAppearanceSourceKey(): string {
  return source?.key ?? "";
}

function translated(key: string): string {
  const locale =
    typeof navigator === "undefined" ? "" : navigator.language;
  const candidates = [
    locale,
    locale.replace("-", "_"),
    "zh-CN",
    "zh_CN",
    "zh",
    "en",
  ];
  for (const candidate of candidates) {
    const value = appearanceState.translations[candidate]?.[key];
    if (value) return value;
  }
  return key;
}

function selectionStorageKey(characterId: number): string | null {
  return appearanceState.contentLockHash
    ? `noname-next.skin.${appearanceState.contentLockHash}.${characterId}`
    : null;
}

function setSource(next: CatalogSource | null): void {
  source = next;
  clearCatalog();
}

function clearCatalog(): void {
  appearanceState.contentLockHash = "";
  appearanceState.total = 0;
  appearanceState.pages.clear();
  appearanceState.characters.clear();
  appearanceState.skins.clear();
  appearanceState.translations = {};
  appearanceState.selectedSkins.clear();
  appearanceState.portraitUrls.clear();
  appearanceState.imageUrls.clear();
  appearanceState.substituteUrls.clear();
  pageRequests.clear();
  characterRequests.clear();
  assetRequests.clear();
}

function assertCatalogPage(value: unknown): asserts value is ContentCatalogPage {
  const page = value as Partial<ContentCatalogPage> | null;
  if (
    !page ||
    typeof page.content_lock_hash !== "string" ||
    !unsignedInteger(page.offset) ||
    !unsignedInteger(page.limit) ||
    !unsignedInteger(page.total) ||
    page.limit < 1 ||
    page.limit > PAGE_LIMIT ||
    page.offset > page.total ||
    !Array.isArray(page.characters) ||
    page.characters.length > page.limit ||
    !Array.isArray(page.skins) ||
    !Array.isArray(page.aliases) ||
    !page.translations ||
    typeof page.translations !== "object" ||
    !page.characters.every(
      (character) =>
        unsignedInteger(character?.definition?.id) &&
        character.definition.id > 0 &&
        typeof character?.definition?.name_key === "string" &&
        Array.isArray(character?.skin_ids) &&
        character.skin_ids.every(unsignedInteger)
    ) ||
    !page.skins.every(
      (skin) =>
        unsignedInteger(skin?.id) &&
        skin.id > 0 &&
        typeof skin?.name_key === "string" &&
        ["static", "spine", "substitute"].includes(
          skin?.presentation?.kind
        )
    ) ||
    !page.aliases.every(
      (alias) =>
        unsignedInteger(alias?.character_id) &&
        unsignedInteger(alias?.source_character_id) &&
        stringRecord(alias?.voice_mappings)
    ) ||
    !Object.values(page.translations).every(stringRecord)
  ) {
    throw new Error("内容目录响应无效");
  }
}

function unsignedInteger(value: unknown): value is number {
  return Number.isSafeInteger(value) && Number(value) >= 0;
}

function selectableSkinIds(character: ContentCatalogCharacter): number[] {
  return character.skin_ids.filter(
    (id) => appearanceState.skins.get(id)?.presentation.kind !== "substitute"
  );
}

function stringRecord(value: unknown): value is Record<string, string> {
  return (
    !!value &&
    typeof value === "object" &&
    !Array.isArray(value) &&
    Object.values(value).every((entry) => typeof entry === "string")
  );
}

function validAssetRef(asset: ContentAssetRef): boolean {
  return (
    typeof asset.namespace === "string" &&
    asset.namespace.length > 0 &&
    typeof asset.package_id === "string" &&
    asset.package_id.length > 0 &&
    typeof asset.asset_id === "string" &&
    asset.asset_id.length > 0
  );
}
