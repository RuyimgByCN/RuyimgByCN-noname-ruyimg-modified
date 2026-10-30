// Original-content previews use the same source manifest served by game-server.
export interface OriginalAudioAsset {
  kind: string;
  path: string;
  size: number;
  source_category: string;
  source_path: string;
  sha256: string;
}

export function originalResourceUrl(base: string, asset: string | null | undefined): string {
  if (typeof asset !== "string" || !asset || !asset.split("/").every((part) =>
    part && part !== "." && part !== ".." && !/[\\%?#:\u0000-\u001f]/.test(part),
  )) return "";
  return `${base}/${asset.split("/").map(encodeURIComponent).join("/")}`;
}

export function originalCharacterAudio(
  manifest: { assets: OriginalAudioAsset[] },
  character: { name: string; skills: string[] },
): OriginalAudioAsset[] {
  return manifest.assets.filter((asset) => {
    if (!asset || typeof asset.path !== "string" || typeof asset.sha256 !== "string" ||
      asset.kind !== "audio" || asset.source_category !== "core" ||
      asset.path !== asset.source_path || !/^[a-f0-9]{64}$/.test(asset.sha256) ||
      !Number.isSafeInteger(asset.size) || asset.size <= 0 ||
      !originalResourceUrl("", asset.path) || !asset.path.endsWith(".mp3")) return false;
    if (asset.path === `audio/die/${character.name}.mp3`) return true;
    return character.skills.some((skill) => {
      const prefix = `audio/skill/${skill}`;
      return asset.path.startsWith(prefix) && /^(?:[1-9][0-9]*)?\.mp3$/.test(asset.path.slice(prefix.length));
    });
  });
}
