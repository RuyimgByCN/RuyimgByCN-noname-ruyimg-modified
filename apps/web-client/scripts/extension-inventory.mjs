// 转换包清单：由 packages/extensions 的实际包数据推导，供冒烟与回归共用。
// 不写死目录数，避免像旧 `extension_catalog_15` 那样随包增删过期。
import { existsSync, readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";

/**
 * 逐目录读取 `<dir>/package.npack.json` 的 manifest。
 *
 * @param {string} extRoot `packages/extensions` 绝对路径
 * @returns {{
 *   packs: Map<string, {namespace: string, package_id: string, version: string}>,
 *   dirCount: number,
 *   orphanDirs: string[],
 *   problems: string[],
 * }}
 */
export function collectPackManifests(extRoot) {
  const dirCount = existsSync(extRoot)
    ? readdirSync(extRoot, { withFileTypes: true }).filter((e) => e.isDirectory())
        .length
    : 0;
  if (dirCount === 0) {
    return { packs: new Map(), dirCount, orphanDirs: [], problems: ["包目录不存在或为空"] };
  }
  const dirs = readdirSync(extRoot, { withFileTypes: true })
    .filter((e) => e.isDirectory())
    .map((e) => e.name)
    .sort();
  const packs = new Map();
  const keys = new Map();
  const orphanDirs = [];
  const problems = [];
  for (const dir of dirs) {
    const npack = join(extRoot, dir, "package.npack.json");
    if (!existsSync(npack)) {
      orphanDirs.push(dir);
      continue;
    }
    let manifest;
    try {
      manifest = JSON.parse(readFileSync(npack, "utf8")).manifest;
    } catch (error) {
      problems.push(`${dir}: npack 不可解析 (${error})`);
      continue;
    }
    const { namespace, package_id: packageId, version } = manifest ?? {};
    if (!namespace || !packageId || !version) {
      problems.push(`${dir}: manifest 缺 namespace/package_id/version`);
      continue;
    }
    const key = `${namespace}/${packageId}`;
    if (keys.has(key)) {
      problems.push(`${dir}: 包键 ${key} 与 ${keys.get(key)} 重复`);
    }
    keys.set(key, dir);
    packs.set(dir, { namespace, package_id: packageId, version });
  }
  return { packs, dirCount, orphanDirs, problems };
}

/**
 * 客户端可发现目录（EXTENSION_CATALOG）与在盘包的一致性。
 * 未进目录的在盘包只统计、不判失败：管理面覆盖全部在盘包是另一项产品工作。
 *
 * @param {Array<{dir: string, packageId: string, namespace: string, version: string}>} catalog
 * @param {Map<string, {namespace: string, package_id: string, version: string}>} packs
 * @returns {{problems: string[], uncovered: string[]}}
 */
export function checkCatalogConsistency(catalog, packs) {
  const problems = [];
  for (const entry of catalog) {
    const manifest = packs.get(entry.dir);
    if (!manifest) {
      problems.push(`${entry.dir}: 目录项没有对应在盘包`);
      continue;
    }
    const actual = `${manifest.namespace}/${manifest.package_id}@${manifest.version}`;
    const declared = `${entry.namespace}/${entry.packageId}@${entry.version}`;
    if (actual !== declared) {
      problems.push(`${entry.dir}: 目录项 ${declared} ≠ 包 ${actual}`);
    }
  }
  const catalogDirs = new Set(catalog.map((e) => e.dir));
  const uncovered = [...packs.keys()].filter((dir) => !catalogDirs.has(dir));
  return { problems, uncovered };
}
