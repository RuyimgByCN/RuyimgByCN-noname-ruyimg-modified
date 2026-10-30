import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { copyFileSync, mkdirSync, mkdtempSync, readFileSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const repo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
test("resources-only refresh preserves catalog, legacy source and immutable packages and verifies bytes", (t) => {
  const scratch = path.join(repo, ".scratch");
  mkdirSync(scratch, { recursive: true });
  const fixture = mkdtempSync(path.join(scratch, "resource-sync-"));
  t.after(() => rmSync(fixture, { recursive: true, force: true }));
  const root = path.join(fixture, "worktree");
  const source = path.join(fixture, "source");
  for (const dir of ["scripts", "apps/web-client", "content/original/ported", "content/original/legacy-source"]) {
    mkdirSync(path.join(root, dir), { recursive: true });
  }
  for (const dir of ["character", "image/character", "audio/skill"]) mkdirSync(path.join(source, dir), { recursive: true });
  for (const name of ["sync-original-content.mjs", "original-content-order.mjs"]) {
    copyFileSync(path.join(repo, "scripts", name), path.join(root, "scripts", name));
  }
  symlinkSync(path.join(repo, "apps/web-client/node_modules"), path.join(root, "apps/web-client/node_modules"));
  writeFileSync(path.join(source, "image/character/actor.jpg"), "portrait bytes");
  writeFileSync(path.join(source, "audio/skill/voice1.mp3"), "voice bytes");
  const provenance = { core: {
    category: "core", root: "source", repository: null, repository_root: null,
    head: null, dirty: true, tracked_changes: [], tracked_change_count: 0,
    untracked_count: 2, redistribution_status: "preserved_from_source_requires_review",
  }, skin: null };
  const protectedFiles = {
    "content/original/catalog.json": JSON.stringify({ schema_version: 4, source: { provenance }, characters: [{ id: "sp:tangtang", def_id: 2295081372 }] }),
    "content/original/ported/frozen.npack.json": "immutable old package bytes",
    "content/original/legacy-source/sentinel.js": "manual legacy source",
  };
  for (const [name, bytes] of Object.entries(protectedFiles)) writeFileSync(path.join(root, name), bytes);
  const manifestFile = path.join(root, "content/original/asset-manifest.json");
  writeFileSync(manifestFile, JSON.stringify({ schema_version: 2, source: "source", sources: provenance,
    trees: [{ name: "legacy-source", files: 1 }], assets: [] }));
  const run = (...args) => spawnSync(process.execPath, [path.join(root, "scripts/sync-original-content.mjs"),
    "--core-source", source, "--resources-only", ...args], {
      cwd: root, encoding: "utf8", env: { ...process.env, GIT_CEILING_DIRECTORIES: fixture },
    });
  const result = run();
  assert.equal(result.status, 0, result.stderr);
  for (const [name, bytes] of Object.entries(protectedFiles)) assert.equal(readFileSync(path.join(root, name), "utf8"), bytes);
  const manifest = JSON.parse(readFileSync(manifestFile));
  assert.deepEqual(manifest.assets.map((asset) => asset.path), ["portraits/actor.jpg", "audio/skill/voice1.mp3"]);
  assert.equal(manifest.assets[1].source_path, "audio/skill/voice1.mp3");
  assert.equal(run("--verify-only").status, 0);
  const target = path.join(root, "content/original/assets/audio/skill/voice1.mp3");
  // Break the hardlink before corrupting the destination; the source stays read-only.
  rmSync(target);
  writeFileSync(target, "wrong bytes");
  const corrupted = run("--verify-only");
  assert.notEqual(corrupted.status, 0);
  assert.match(corrupted.stderr, /content differs/);
  assert.equal(readFileSync(path.join(source, "audio/skill/voice1.mp3"), "utf8"), "voice bytes");
  const beforeRejectedRefresh = readFileSync(manifestFile, "utf8");
  const wrongSource = JSON.parse(beforeRejectedRefresh);
  wrongSource.sources.core.head = "unadopted";
  writeFileSync(manifestFile, JSON.stringify(wrongSource));
  const rejected = run();
  assert.notEqual(rejected.status, 0);
  assert.match(rejected.stderr, /resource source provenance mismatch/);
  assert.equal(readFileSync(target, "utf8"), "wrong bytes", "unadopted source must fail before syncing assets");
});
