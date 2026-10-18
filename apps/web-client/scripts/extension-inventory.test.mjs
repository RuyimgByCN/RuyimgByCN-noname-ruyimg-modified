// 扩展包清单守护的回归：证明数据推导出的断言**会**失败（非恒真），
// 以及真实仓库当前的自洽性。运行：pnpm --dir apps/web-client test
import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";

import { checkCatalogConsistency, collectPackManifests } from "./extension-inventory.mjs";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const extRoot = join(root, "../../packages/extensions");

/** 在临时目录搭一个包目录 fixture：`{dir: manifest | "bad" | null}`。 */
function fixture(dirs) {
  const base = mkdtempSync(join(tmpdir(), "nc03-inventory-"));
  for (const [dir, manifest] of Object.entries(dirs)) {
    mkdirSync(join(base, dir), { recursive: true });
    if (manifest === null) continue;
    writeFileSync(
      join(base, dir, "package.npack.json"),
      manifest === "bad" ? "{ not json" : JSON.stringify({ manifest }),
    );
  }
  return base;
}

test("实测仓库：包清单由实际数据推导，目录项全部可回查", () => {
  const inventory = collectPackManifests(extRoot);
  assert.deepEqual(inventory.problems, []);
  assert.equal(inventory.packs.size, inventory.dirCount - inventory.orphanDirs.length);
  assert.ok(inventory.packs.size >= 15, `在盘包 ${inventory.packs.size} 少于目录项下限`);

  const catalog = [
    { dir: "huanle-upstream-noname", packageId: "huanle-cards", namespace: "upstream-noname", version: "1.0.0" },
  ];
  const check = checkCatalogConsistency(catalog, inventory.packs);
  assert.deepEqual(check.problems, []);
  assert.ok(check.uncovered.length > 0, "未进客户端目录的在盘包应被统计而不是隐藏");
});

test("目录项与包元数据不一致会被判失败", () => {
  const base = fixture({
    "sample-upstream-noname": {
      namespace: "upstream-noname",
      package_id: "sample",
      version: "1.0.0",
    },
  });
  try {
    const { packs, problems } = collectPackManifests(base);
    assert.deepEqual(problems, []);

    const ok = checkCatalogConsistency(
      [{ dir: "sample-upstream-noname", packageId: "sample", namespace: "upstream-noname", version: "1.0.0" }],
      packs,
    );
    assert.deepEqual(ok.problems, []);

    const wrongNamespace = checkCatalogConsistency(
      [{ dir: "sample-upstream-noname", packageId: "sample", namespace: "upstream-liuli", version: "1.0.0" }],
      packs,
    );
    assert.equal(wrongNamespace.problems.length, 1);
    assert.match(wrongNamespace.problems[0], /目录项 upstream-liuli\/sample@1\.0\.0 ≠ 包 upstream-noname\/sample@1\.0\.0/);

    const missingDir = checkCatalogConsistency(
      [{ dir: "gone-upstream-noname", packageId: "gone", namespace: "upstream-noname", version: "1.0.0" }],
      packs,
    );
    assert.deepEqual(missingDir.problems, ["gone-upstream-noname: 目录项没有对应在盘包"]);
  } finally {
    rmSync(base, { recursive: true, force: true });
  }
});

test("坏 npack、缺字段与重复包键会被判失败", () => {
  const base = fixture({
    "bad-json-upstream-noname": "bad",
    "no-version-upstream-noname": { namespace: "upstream-noname", package_id: "noversion" },
    "dup-a-upstream-noname": { namespace: "upstream-noname", package_id: "dup", version: "1.0.0" },
    "dup-b-upstream-liuli": { namespace: "upstream-noname", package_id: "dup", version: "2.0.0" },
    "orphan-dir": null,
  });
  try {
    const inventory = collectPackManifests(base);
    assert.equal(inventory.dirCount, 5);
    assert.deepEqual(inventory.orphanDirs, ["orphan-dir"]);
    assert.equal(inventory.packs.size, 2);
    assert.equal(inventory.problems.length, 3);
    const problems = inventory.problems.join("\n");
    assert.match(problems, /bad-json-upstream-noname: npack 不可解析/);
    assert.match(problems, /no-version-upstream-noname: manifest 缺/);
    assert.match(problems, /dup-b-upstream-liuli: 包键 upstream-noname\/dup 与 dup-a-upstream-noname 重复/);
  } finally {
    rmSync(base, { recursive: true, force: true });
  }
});
