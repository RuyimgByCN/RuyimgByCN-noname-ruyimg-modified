import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import type {
  ContentAssetRef,
  ContentBundle,
  ContentCatalogPage,
} from "../types/engine";

export interface DesktopPlatformInfo {
  os: string;
  arch: string;
  shell: string;
}

export function isTauriShell(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export function desktopPlatformInfo(): Promise<DesktopPlatformInfo | null> {
  if (!isTauriShell()) return Promise.resolve(null);
  return invoke<DesktopPlatformInfo>("platform_info");
}

export function setDesktopMatchFrozen(frozen: boolean): Promise<void> {
  if (!isTauriShell()) return Promise.resolve();
  return invoke<void>("set_match_frozen", { frozen });
}

export function desktopContentBundle(): Promise<ContentBundle | null> {
  if (!isTauriShell()) return Promise.resolve(null);
  return invoke<ContentBundle>("content_bundle");
}

export function desktopContentCatalogPage(
  offset: number,
  limit: number
): Promise<ContentCatalogPage | null> {
  if (!isTauriShell()) return Promise.resolve(null);
  return invoke<ContentCatalogPage>("content_catalog_page", { offset, limit });
}

export async function resolveDesktopContentAsset(
  asset: ContentAssetRef
): Promise<string | null> {
  if (!isTauriShell()) return null;
  const path = await invoke<string>("resolve_content_asset", {
    namespace: asset.namespace,
    packageId: asset.package_id,
    assetId: asset.asset_id,
  });
  return convertFileSrc(path);
}
