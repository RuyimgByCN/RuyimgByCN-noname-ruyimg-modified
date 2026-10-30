import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";
import { assertUniqueAssetPaths, declaredAudioAssets, loadAudioDeclarations, resolveAudioDeclarations } from "./original-audio-resources.mjs";

function fixture(t, skillSource, characterSource = "actor: { skills: ['alias'] }") {
  const root = mkdtempSync(path.join(tmpdir(), "original-audio-"));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  mkdirSync(path.join(root, "character/pack"), { recursive: true });
  writeFileSync(path.join(root, "character/pack/skill.js"), `const skills = { ${skillSource} }; export default skills;`);
  writeFileSync(path.join(root, "character/pack/character.js"), `const characters = { ${characterSource} }; export default characters;`);
  return root;
}
const sourcePaths = (plan, id, variant = "") => plan.mappings.filter((row) => row.id === id && row.variant === variant).map((row) => row.source_path);

test("AST maps true upstream reference/count/list/variant/death forms without executing source", async (t) => {
  const root = fixture(t, `
    direct: { audio: 2, audioname: ['shadow'] }, alias: { audio: 'direct' },
    limited: { audio: ['direct', 1] }, external: { audio: ['missing', 2] },
    mixed: { audio: ['direct1.mp3', ['direct', 1], false, 'direct1.mp3'] },
    exact: { audio: true }, muted: { audio: false },
    voice: { audio: 1 }, voice2: { audio: 1 },
    special: { audio: 2, audioname2: { actor: 'exact' } },
    inherited: { inherit: 'direct' },
    dynamic: { audio: 2, logAudio(event, player) { throw new Error('must not execute'); } },
    custom: { audio: ['skill/custom:2:ogg', 'skill/one.wav'] },
    parent: { audio: 1, subSkill: { child: { audio: 'direct' } } },
  `, `actor: { dieAudios: ['other'] }, other: { dieAudios: ['2'] }`);
  const declarations = await loadAudioDeclarations(root);
  const plan = resolveAudioDeclarations(declarations);
  assert.deepEqual(sourcePaths(plan, 'pack:alias'), ['audio/skill/direct1.mp3', 'audio/skill/direct2.mp3']);
  assert.deepEqual(sourcePaths(plan, 'pack:alias', 'shadow'), ['audio/skill/direct_shadow1.mp3', 'audio/skill/direct_shadow2.mp3']);
  assert.deepEqual(sourcePaths(plan, 'pack:limited'), ['audio/skill/direct1.mp3']);
  assert.deepEqual(sourcePaths(plan, 'pack:external'), ['audio/skill/missing1.mp3', 'audio/skill/missing2.mp3']);
  assert.deepEqual(sourcePaths(plan, 'pack:mixed'), ['audio/skill/direct1.mp3']);
  assert.deepEqual(sourcePaths(plan, 'pack:exact'), ['audio/skill/exact.mp3']);
  assert.deepEqual(sourcePaths(plan, 'pack:muted'), []);
  assert.deepEqual(sourcePaths(plan, 'pack:voice'), ['audio/skill/voice1.mp3']);
  assert.deepEqual(sourcePaths(plan, 'pack:voice2'), ['audio/skill/voice21.mp3']);
  assert.deepEqual(sourcePaths(plan, 'pack:special', 'actor'), ['audio/skill/exact.mp3']);
  assert.deepEqual(sourcePaths(plan, 'pack:inherited'), ['audio/skill/inherited1.mp3', 'audio/skill/inherited2.mp3']);
  assert.deepEqual(sourcePaths(plan, 'pack:custom'), ['audio/skill/custom/custom1.ogg', 'audio/skill/custom/custom2.ogg', 'audio/skill/one.wav']);
  assert.deepEqual(sourcePaths(plan, 'pack:parent_child'), sourcePaths(plan, 'pack:direct'));
  assert.deepEqual(sourcePaths(plan, 'pack:actor'), ['audio/die/other1.mp3', 'audio/die/other2.mp3']);
  assert.equal(plan.dynamic.length, 1);
  assert.match(plan.dynamic[0].expression, /must not execute/);
});

test("unknown and absent source voices never borrow another skill's bytes", async (t) => {
  const plan = resolveAudioDeclarations(await loadAudioDeclarations(fixture(t, "alias: { audio: 'unknown' }, other: { audio: 2 }")));
  assert.deepEqual(sourcePaths(plan, 'pack:alias'), []);
  assert.ok(plan.issues.some((row) => row.reference === 'unknown'));
  const result = declaredAudioAssets(plan, []);
  assert.equal(result.assets.length, 0);
  assert.equal(result.missing.length, 3); // other1/2 plus default actor death
});

test("cycles, ambiguous references, duplicate declarations and unsafe paths fail closed", async (t) => {
  for (const source of [
    "a: { audio: 'b' }, b: { audio: 'a' }",
    "a: { inherit: 'b' }, b: { inherit: 'a' }", "a: { audio: 1, audio: 2 }",
    ...['../x.mp3', '/x.mp3', 'skill/../x.mp3', 'skill/x?y.mp3', 'skill/%2e%2e/x.mp3', 'skill/x\\\\y.mp3', 'skill/x#y.mp3'].map((value) => `a: { audio: ${JSON.stringify(value)} }`),
  ]) {
    await assert.rejects(async () => resolveAudioDeclarations(await loadAudioDeclarations(fixture(t, source))), /cycle|duplicate|invalid/);
  }
  const data = await loadAudioDeclarations(fixture(t, "a: { audio: 'foreign' }"));
  for (const pack of ['one', 'two']) data.skills.set(`${pack}:foreign`, { id: `${pack}:foreign`, pack, name: 'foreign', data: { audio: 1 } });
  assert.throws(() => resolveAudioDeclarations(data), /ambiguous/);
  assert.throws(() => assertUniqueAssetPaths([{ path: 'same', sha256: 'a' }, { path: 'same', sha256: 'b' }]), /duplicate canonical/);
});


test("real diy:liangji immediate self reference uses upstream default; a multi-skill cycle rejects", async (t) => {
  const plan = resolveAudioDeclarations(await loadAudioDeclarations(fixture(t, 'liangji: { audio: ["liangji", 2] }', 'actor: { dieAudios: ["actor"] }')));
  assert.deepEqual(sourcePaths(plan, "pack:liangji"), []);
  assert.ok(plan.issues.some((row) => row.reason === "empty_by_upstream_self_reference" && row.declaration.expression === '["liangji", 2]'));
  assert.deepEqual(sourcePaths(plan, "pack:actor"), ["audio/die/actor.mp3"]);
  await assert.rejects(async () => resolveAudioDeclarations(await loadAudioDeclarations(fixture(t, 'a: { audio: "b" }, b: { audio: "a" }'))), /cycle/);
});


test("static card/mode dependencies and unknown character death references follow source declarations", async (t) => {
  const root = fixture(t, 'alias: { audio: "equipment" }, helper: { audio: "mode_voice" }', 'actor: { dieAudios: ["unlisted"] }');
  mkdirSync(path.join(root, "card"));
  writeFileSync(path.join(root, "card/standard.js"), 'export default { skill: { equipment: { audio: 2 } } };');
  mkdirSync(path.join(root, "mode"));
  writeFileSync(path.join(root, "mode/boss.js"), 'export default { skill: { mode_voice: { audio: true } } };');
  const plan = resolveAudioDeclarations(await loadAudioDeclarations(root));
  assert.deepEqual(sourcePaths(plan, "pack:alias"), ["audio/skill/equipment1.mp3", "audio/skill/equipment2.mp3"]);
  assert.deepEqual(sourcePaths(plan, "pack:helper"), ["audio/skill/mode_voice.mp3"]);
  assert.deepEqual(sourcePaths(plan, "pack:actor"), ["audio/die/unlisted.mp3"]);
  assert.ok(plan.mappings.every((row) => row.id.startsWith("pack:")), "dependencies add no roster or parallel resource owners");
});

test("an equivalent top-level helper and finishSkill subskill share one audio identity; conflicts reject", async (t) => {
  const equivalent = await loadAudioDeclarations(fixture(t, 'parent: { subSkill: { child: { audio: 1 } } }, parent_child: { audio: 1 }'));
  assert.deepEqual(sourcePaths(resolveAudioDeclarations(equivalent), 'pack:parent_child'), ['audio/skill/parent_child1.mp3']);
  await assert.rejects(() => loadAudioDeclarations(fixture(t, 'parent: { subSkill: { child: { audio: 1 } } }, parent_child: { audio: 2 }')), /conflicting audio skill/);
});

test("character bindings use actual tempname priority and only declared substitute forms", async (t) => {
  const source = await loadAudioDeclarations(fixture(t, `
    shared: { audio: 2, audioname: ['foreign', 'temp', 'shadow'], audioname2: { foreign: 'foreignVoice', skinOverride: 'overrideVoice' } },
    alias: { audio: 'shared' }, foreignVoice: { audio: 1 }, overrideVoice: { audio: 1 },
  `, `actor: { skills: ['alias'], tempname: ['temp'] }, foreign: { skills: ['alias'] }`));
  const substitutes = [
    { pack: 'pack', character: 'actor', substitute: 'shadow', directives: [] },
    { pack: 'pack', character: 'actor', substitute: 'otherSkin', directives: ['tempname:skinOverride', 'die:actor'] },
  ];
  const plan = resolveAudioDeclarations(source, substitutes);
  const voices = (id, variant = '') => plan.mappings.filter((row) => row.id === id && row.type === 'character-skill' && row.variant === variant).map((row) => row.source_path);
  assert.deepEqual(voices('pack:actor'), ['audio/skill/shared_temp1.mp3', 'audio/skill/shared_temp2.mp3']);
  assert.deepEqual(voices('pack:foreign'), ['audio/skill/foreignVoice_foreign1.mp3']);
  assert.deepEqual(voices('pack:actor', 'shadow'), ['audio/skill/shared_shadow1.mp3', 'audio/skill/shared_shadow2.mp3']);
  assert.deepEqual(voices('pack:actor', 'otherSkin'), ['audio/skill/overrideVoice_temp1.mp3']);
  assert.ok(!plan.mappings.some((row) => row.id === 'pack:actor' && row.type === 'character-skill' && row.source_path.includes('foreignVoice')));
  assert.ok(plan.mappings.some((row) => row.id === 'pack:actor' && row.type === 'character-die' && row.variant === 'otherSkin' && row.source_path === 'audio/die/actor.mp3'));
  for (const substitute of ['../escape', 'bad%2Fname', 'bad\\name']) {
    assert.throws(() => resolveAudioDeclarations(source, [{ ...substitutes[0], substitute }]), /invalid/);
  }
  assert.throws(() => resolveAudioDeclarations(source, [substitutes[0], substitutes[0]]), /duplicate canonical/);
});

test("dynamic actor inputs produce no guessed binding and substitute death reference cycles reject", async (t) => {
  const dynamic = resolveAudioDeclarations(await loadAudioDeclarations(fixture(t, 'alias: { audio: 2 }', 'actor: { skills: ["alias"], tempname: player.storage.names }')));
  assert.ok(!dynamic.mappings.some((row) => row.type === 'character-skill'));
  assert.ok(dynamic.issues.some((row) => row.reason === 'dynamic character audio inputs'));
  const source = await loadAudioDeclarations(fixture(t, 'alias: { audio: 1 }', 'actor: { skills: ["alias"] }, other: { dieAudios: ["shadow"] }'));
  assert.throws(() => resolveAudioDeclarations(source, [{ pack: 'pack', character: 'actor', substitute: 'shadow', directives: ['die:other'] }]), /cycle/);
  const self = resolveAudioDeclarations(source, [{ pack: 'pack', character: 'actor', substitute: 'shadow', directives: ['die:shadow'] }]);
  assert.ok(self.mappings.some((row) => row.type === 'character-die' && row.source_path === 'audio/die/shadow.mp3'));
});

test("equivalent dependency leaves are provable; identical indirect metadata cannot hide different source contexts", async (t) => {
  const root = fixture(t, 'alias: { audio: "shared" }');
  mkdirSync(path.join(root, 'mode'));
  for (const file of ['one', 'two']) writeFileSync(path.join(root, `mode/${file}.js`), 'export default { skill: { shared: { audio: 2 } } };');
  const leaves = resolveAudioDeclarations(await loadAudioDeclarations(root));
  assert.deepEqual(leaves.mappings.filter(row => row.type === 'character-skill').map(row => row.source_path), ['audio/skill/shared1.mp3', 'audio/skill/shared2.mp3']);
  for (const [index, file] of ['one', 'two'].entries()) writeFileSync(path.join(root, `mode/${file}.js`), `export default { skill: { shared: { audio: "local" }, local: { audio: ${index + 1} } } };`);
  await assert.rejects(async () => resolveAudioDeclarations(await loadAudioDeclarations(root)), /ambiguous audio reference/);
});

test("substitutes resolve one global exact owner across declaration packs and reject ambiguity", async (t) => {
  const source = await loadAudioDeclarations(fixture(t, 'alias: { audio: 2, audioname: ["shadow"] }'));
  const declaration = { pack: 'different-declaring-pack', character: 'actor', substitute: 'shadow', directives: [] };
  const plan = resolveAudioDeclarations(source, [declaration]);
  const variants = plan.mappings.filter(row => row.type.startsWith('character') && row.variant === 'shadow');
  assert.ok(variants.some(row => row.source_path === 'audio/skill/alias_shadow1.mp3'));
  assert.ok(variants.some(row => row.source_path === 'audio/die/shadow.mp3'));
  assert.ok(variants.every(row => row.path.startsWith('audio-declared/pack/character/actor/')));
  assert.equal(declaration.pack, 'different-declaring-pack', 'declaration provenance stays unchanged');
  assert.throws(() => resolveAudioDeclarations(source, [{ ...declaration, character: 'missing' }]), /unknown substitute owner/);
  source.characters.set('different-declaring-pack:actor', { id: 'different-declaring-pack:actor', pack: 'different-declaring-pack', name: 'actor', data: {} });
  assert.throws(() => resolveAudioDeclarations(source, [declaration]), /ambiguous substitute owner/);
});

test("five static bindings select ten AST aliases without changing global owners", async (t) => {
  const { readFileSync } = await import('node:fs');
  const bindings = JSON.parse(readFileSync(new URL('../content/original-audio-compat/provenance.json', import.meta.url))).bindings;
  const baseline = JSON.parse(readFileSync(new URL('../content/original/asset-manifest.json', import.meta.url)));
  const core = bindings.map(row => baseline.assets.find(asset => asset.path === row.source_path));
  const sizes = [18716, 114531, 10188, 39091, 81053];
  const backing = bindings.map((row, i) => ({ kind: 'audio', path: row.derivative_path.replace('audio/', 'audio-compat/'),
    source_category: 'compat', source_path: row.derivative_path, sha256: row.derivative_sha256, size: sizes[i] }));
  const root = fixture(t, 'tsumugi_mugyu: { audio: 5 }', 'actor: { skills: ["tsumugi_mugyu"], dieAudios: ["actor"] }');
  const plan = resolveAudioDeclarations(await loadAudioDeclarations(root));
  const result = declaredAudioAssets(plan, [...core, ...backing], bindings);
  assert.equal(result.assets.length, 10);
  assert.ok(result.assets.every(row => row.source_category === 'compat' && row.path.endsWith('.m4a')));
  assert.equal(result.assets.filter(row => row.path.includes('/character/actor/')).length, 5);
  for (const row of result.assets) {
    const source = backing.find(source => source.source_path === row.source_path);
    assert.deepEqual({ ...row, path: source.path }, source);
  }
  assert.throws(() => declaredAudioAssets(plan, [{ ...core[0], sha256: 'a'.repeat(64) }, ...core.slice(1), ...backing], bindings), /binding/);
  assert.throws(() => declaredAudioAssets(plan, [...core, ...backing, { ...backing[0], path: 'audio-compat/other.m4a' }], bindings), /duplicate.*backing/);
});
