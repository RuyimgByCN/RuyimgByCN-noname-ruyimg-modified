import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { copyFileSync, cpSync, mkdirSync, mkdtempSync, readFileSync, renameSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const repo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
function fixture(t) {
  const scratch = path.join(repo, ".scratch");
  mkdirSync(scratch, { recursive: true });
  const fixtureRoot = mkdtempSync(path.join(scratch, "resource-sync-"));
  t.after(() => rmSync(fixtureRoot, { recursive: true, force: true }));
  const root = path.join(fixtureRoot, "worktree"), source = path.join(fixtureRoot, "source");
  for (const dir of ["scripts", "apps/web-client", "content/original/ported", "content/original/legacy-source", "packages"]) {
    mkdirSync(path.join(root, dir), { recursive: true });
  }
  for (const dir of ["character/pack", "image/character", "audio/skill"]) mkdirSync(path.join(source, dir), { recursive: true });
  for (const name of ["sync-original-content.mjs", "original-content-order.mjs", "original-source-ast.mjs", "original-audio-resources.mjs"]) {
    copyFileSync(path.join(repo, "scripts", name), path.join(root, "scripts", name));
  }
  symlinkSync(path.join(repo, "apps/web-client/node_modules"), path.join(root, "apps/web-client/node_modules"));
  cpSync(path.join(repo, "content/original-audio-compat"), path.join(root, "content/original-audio-compat"), { recursive: true });
  const sidecar = path.join(root, "content/original-audio-compat/provenance.json");
  const proof = JSON.parse(readFileSync(sidecar));
  const canonical = process.env.NONAME_ORIGINAL_ROOT || path.resolve(repo, "../archive/upstream/noname/apps/core");
  for (const row of proof.bindings) copyFileSync(path.join(canonical, row.source_path), path.join(source, row.source_path));
  writeFileSync(path.join(source, "image/character/actor.jpg"), "portrait bytes");
  writeFileSync(path.join(source, "audio/skill/voice1.mp3"), "voice bytes");
  writeFileSync(path.join(source, "character/pack/skill.js"), 'const skills = { voice: { audio: 1 }, alias: { audio: "voice" }, tsumugi_mugyu: { audio: 5 } };');
  writeFileSync(path.join(source, "character/pack/character.js"), 'const characters = { actor: { skills: ["alias"] }, actor2: { skills: ["tsumugi_mugyu"] } };');
  const provenance = { core: {
    category: "core", root: "source", repository: null, repository_root: null,
    head: null, dirty: true, tracked_changes: [], tracked_change_count: 0,
    untracked_count: 9, redistribution_status: "preserved_from_source_requires_review",
  }, skin: null };
  const protectedFiles = {
    "content/original/catalog.json": JSON.stringify({ schema_version: 4, source: { provenance }, characters: [{ id: "sp:tangtang", def_id: 2295081372 }] }),
    "content/original/ported/frozen.npack.json": "immutable old package bytes",
    "content/original/legacy-source/sentinel.js": "manual legacy source",
    "packages/old-lock.json": "old package lock bytes",
  };
  for (const [name, bytes] of Object.entries(protectedFiles)) writeFileSync(path.join(root, name), bytes);
  const manifestFile = path.join(root, "content/original/asset-manifest.json");
  writeFileSync(manifestFile, JSON.stringify({ schema_version: 2, source: "source", sources: provenance,
    trees: [{ name: "legacy-source", files: 1 }], assets: [] }));
  const run = (...args) => spawnSync(process.execPath, [path.join(root, "scripts/sync-original-content.mjs"),
    "--core-source", source, "--resources-only", ...args], {
      cwd: root, encoding: "utf8", env: { ...process.env, GIT_CEILING_DIRECTORIES: fixtureRoot },
    });
  return { root, source, sidecar, proof, manifestFile, run, protectedFiles };
}

function replace(file, bytes) {
  rmSync(file);
  writeFileSync(file, bytes);
}

test("resources-only refresh preserves catalog, legacy source, immutable packages and old locks and verifies bytes", (t) => {
  const { root, source, manifestFile, run, protectedFiles } = fixture(t);
  const result = run();
  assert.equal(result.status, 0, result.stderr);
  for (const [name, bytes] of Object.entries(protectedFiles)) assert.equal(readFileSync(path.join(root, name), "utf8"), bytes);
  const manifest = JSON.parse(readFileSync(manifestFile));
  assert.deepEqual(manifest.assets.filter(row => row.source_path.includes('voice1')).map(row => row.path), [
    "audio/skill/voice1.mp3",
    "audio-declared/pack/skill/alias/base/0001/voice1.mp3",
    "audio-declared/pack/skill/voice/base/0001/voice1.mp3",
    "audio-declared/pack/character/actor/skill/alias/base/0001/voice1.mp3",
  ]);
  const compat = manifest.assets.filter(row => row.source_category === 'compat');
  assert.equal(compat.length, 15, 'five backing plus ten aliases');
  assert.ok(compat.every(row => row.kind === 'audio' && row.path.endsWith('.m4a')));
  assert.equal(manifest.sources.compat.head, null);
  assert.equal(manifest.sources.compat.root, 'content/original-audio-compat');
  assert.equal(run("--verify-only").status, 0);
  const alias = path.join(root, "content/original/assets/audio-declared/pack/character/actor/skill/alias/base/0001/voice1.mp3");
  assert.equal(readFileSync(alias, "utf8"), "voice bytes");
  const derivedAlias = compat.find(row => row.path.includes('/character/actor2/'));
  assert.deepEqual(readFileSync(path.join(root, 'content/original/assets', derivedAlias.path)),
    readFileSync(path.join(root, 'content/original-audio-compat', derivedAlias.source_path)), 'alias copies real compat backing');
  replace(alias, "bad alias");
  const badAlias = run("--verify-only");
  assert.notEqual(badAlias.status, 0);
  assert.match(badAlias.stderr, /declared audio content differs/);
  assert.equal(run().status, 0);
  const validManifest = readFileSync(manifestFile, "utf8");
  const duplicate = JSON.parse(validManifest);
  duplicate.assets.push({ ...duplicate.assets[1], sha256: "a".repeat(64) });
  writeFileSync(manifestFile, JSON.stringify(duplicate));
  const duplicateRefresh = run();
  assert.notEqual(duplicateRefresh.status, 0);
  assert.match(duplicateRefresh.stderr, /duplicate canonical asset path/);
  assert.equal(readFileSync(alias, "utf8"), "voice bytes", "rejection preserves adopted alias bytes");
  writeFileSync(manifestFile, validManifest);
  const target = path.join(root, "content/original/assets/audio/skill/voice1.mp3");
  replace(target, "wrong bytes");
  const corrupted = run("--verify-only");
  assert.notEqual(corrupted.status, 0);
  assert.match(corrupted.stderr, /content differs/);
  assert.equal(readFileSync(path.join(source, "audio/skill/voice1.mp3"), "utf8"), "voice bytes");
  for (const category of ['core', 'compat']) {
    const wrongSource = JSON.parse(validManifest);
    wrongSource.sources[category].head = "unadopted";
    writeFileSync(manifestFile, JSON.stringify(wrongSource));
    const rejected = run();
    assert.notEqual(rejected.status, 0);
    assert.match(rejected.stderr, /resource source provenance mismatch/);
    assert.equal(readFileSync(target, "utf8"), "wrong bytes", "unadopted source fails before syncing assets");
  }
});

test("static compat drift, paths, duplicate bindings, size, SHA, recipe and tool fail before any resource write", (t) => {
  const { root, source, sidecar, proof, manifestFile, run, protectedFiles } = fixture(t);
  assert.equal(run().status, 0, 'positive valid fixture must first sync');
  assert.equal(run('--verify-only').status, 0, 'positive valid fixture must first verify');
  const originalSidecar = readFileSync(sidecar);
  const row = proof.bindings[0];
  const raw = path.join(source, row.source_path), derived = path.join(root, 'content/original-audio-compat', row.derivative_path);
  const rawBytes = readFileSync(raw), derivedBytes = readFileSync(derived);
  const adopted = JSON.parse(readFileSync(manifestFile));
  const protectedBytes = new Map([manifestFile, ...adopted.assets.map(row => path.join(root, 'content/original/assets', row.path)),
    ...Object.keys(protectedFiles).map(file => path.join(root, file))].map(file => [file, readFileSync(file)]));
  const reject = (label) => {
    for (const args of [[], ['--verify-only']]) {
      const result = run(...args);
      assert.notEqual(result.status, 0, label);
      assert.match(result.stderr, /audio compat|invalid audio path/, `${label}: ${result.stderr}`);
      for (const [file, bytes] of protectedBytes) assert.deepEqual(readFileSync(file), bytes, `${label}: preserved ${file}`);
    }
  };
  for (const [label, mutate] of [
    ['source SHA', d => { d.bindings[0].source_sha256 = 'a'.repeat(64); }],
    ['derived SHA', d => { d.bindings[0].derivative_sha256 = 'a'.repeat(64); }],
    ['traversal', d => { d.bindings[0].derivative_path = 'audio/../escape.m4a'; }],
    ['absolute source', d => { d.bindings[0].source_path = raw; }],
    ['duplicate source', d => { d.bindings[1].source_path = row.source_path; }],
    ['duplicate derivative', d => { d.bindings[1].derivative_path = row.derivative_path; }],
    ['recipe', d => { d.recipe.summary += ' changed'; }],
    ['tool', d => { d.tool.ffmpeg = 'unreviewed'; }],
  ]) {
    const changed = structuredClone(proof); mutate(changed); writeFileSync(sidecar, JSON.stringify(changed));
    reject(label); writeFileSync(sidecar, originalSidecar);
  }
  replace(raw, Buffer.from('changed canonical source')); reject('source drift'); replace(raw, rawBytes);
  replace(derived, Buffer.concat([derivedBytes, Buffer.from('!')])); reject('derived size'); replace(derived, derivedBytes);
  const wrongBytes = Buffer.from(derivedBytes); wrongBytes[100] ^= 1;
  replace(derived, wrongBytes); reject('derived same-size SHA'); replace(derived, derivedBytes);
  const outside = path.join(root, 'outside.m4a'); writeFileSync(outside, derivedBytes);
  rmSync(derived); symlinkSync(outside, derived); reject('derivative symlink escape'); replace(derived, derivedBytes);
  writeFileSync(outside, rawBytes); rmSync(raw); symlinkSync(outside, raw); reject('source symlink escape'); replace(raw, rawBytes);
  const extra = path.join(root, 'content/original-audio-compat/audio/skill/extra.m4a'); writeFileSync(extra, derivedBytes);
  reject('unbound derivative'); rmSync(extra);
  const compatRoot = path.join(root, 'content/original-audio-compat'), movedRoot = `${compatRoot}-outside`;
  renameSync(compatRoot, movedRoot); symlinkSync(movedRoot, compatRoot); reject('symlink input root');
  rmSync(compatRoot); renameSync(movedRoot, compatRoot);
  assert.equal(run().status, 0, 'refresh restores hardlink accounting after fixture replacements');
  assert.equal(run('--verify-only').status, 0, 'restored positive fixture still verifies');
});

test("unbound compat symlinks fail before writes after a valid sync and verify", async (t) => {
  for (const type of ['file', 'dir']) await t.test(type, (t) => {
    const { root, manifestFile, run, protectedFiles } = fixture(t);
    const outside = path.join(root, 'outside');
    if (type === 'dir') mkdirSync(outside);
    else writeFileSync(outside, 'outside file bytes');
    const synced = run();
    assert.equal(synced.status, 0, synced.stderr);
    const verified = run('--verify-only');
    assert.equal(verified.status, 0, verified.stderr);
    const adopted = JSON.parse(readFileSync(manifestFile));
    const protectedBytes = new Map([manifestFile, ...adopted.assets.map(row => path.join(root, 'content/original/assets', row.path)),
      ...Object.keys(protectedFiles).map(file => path.join(root, file))].map(file => [file, readFileSync(file)]));
    symlinkSync(outside, path.join(root, 'content/original-audio-compat/audio/skill/extra.m4a'), type);
    const attempts = [[], ['--verify-only']].map(args => ({ args, result: run(...args) }));
    for (const { args, result } of attempts) {
      assert.notEqual(result.status, 0, `${type} symlink accepted by ${args.length ? 'verify' : 'sync'}; statuses=${JSON.stringify(attempts.map(({ args, result }) => ({ args, status: result.status })))}: ${result.stdout}`);
      assert.match(result.stderr, /audio compat non-regular input/);
      for (const [file, bytes] of protectedBytes) assert.deepEqual(readFileSync(file), bytes, `preserved ${file}`);
    }
  });
});
