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
    part && part !== "." && part !== ".." && !/[\\%?#:\u0000-\u001f\u007f]/.test(part),
  )) return "";
  return `${base}/${asset.split("/").map(encodeURIComponent).join("/")}`;
}

export function originalCharacterAudio(
  manifest: { assets: OriginalAudioAsset[] },
  character: { name: string; skills: string[]; pack?: string; resolved_skills?: string[] },
): OriginalAudioAsset[] {
  const validName = (name: string | undefined) => typeof name === "string" && !name.includes("/") && !!originalResourceUrl("", name);
  if (!validName(character.pack) || !validName(character.name)) return [];
  const byPath = new Map<string, OriginalAudioAsset>();
  for (const asset of manifest.assets) {
    if (!asset || typeof asset.path !== "string") continue;
    // A conflicting row must never be selected according to its position in the manifest.
    if (byPath.has(asset.path)) return [];
    byPath.set(asset.path, asset);
  }
  const characterPrefix = `audio-declared/${character.pack}/character/${character.name}/`;
  const prefixes = [
    `audio-declared/${character.pack}/die/${character.name}/`,
    `${characterPrefix}die/`,
    ...character.skills.filter(validName).map((skill) => `${characterPrefix}skill/${skill}/`),
  ];
  const seenSources = new Set<string>();
  return manifest.assets.filter((asset) => {
    if (!asset || typeof asset.path !== "string" || !prefixes.some((prefix) => asset.path.startsWith(prefix)) ||
      typeof asset.source_path !== "string" ||
      typeof asset.sha256 !== "string" || asset.kind !== "audio" || asset.source_category !== "core" ||
      !/^[a-f0-9]{64}$/.test(asset.sha256) || !Number.isSafeInteger(asset.size) || asset.size <= 0 ||
      !originalResourceUrl("", asset.path) || !originalResourceUrl("", asset.source_path) ||
      !asset.source_path.startsWith("audio/") || !/\.(?:mp3|ogg|wav|m4a|aac|flac|opus|mp4)$/.test(asset.path)) return false;
    const source = byPath.get(asset.source_path);
    if (!source || source.kind !== "audio" || source.source_category !== "core" ||
      source.path !== source.source_path || source.sha256 !== asset.sha256 || source.size !== asset.size ||
      asset.path.split("/").at(-1) !== source.path.split("/").at(-1) || seenSources.has(asset.source_path)) return false;
    seenSources.add(asset.source_path);
    return true;
  });
}
