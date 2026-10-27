#!/usr/bin/env node

import { createHash } from "node:crypto";
import {
  copyFile,
  lstat,
  link,
  mkdir,
  readFile,
  readdir,
  rm,
  stat,
  writeFile,
} from "node:fs/promises";
import { createReadStream } from "node:fs";
import path from "node:path";
import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";
import ts from "../apps/web-client/node_modules/typescript/lib/typescript.js";
import { compareContentNames, compareFileNames, listFiles } from "./original-content-order.mjs";

const cliArgs = process.argv.slice(2);
if (cliArgs.includes("--help") || cliArgs.includes("-h")) {
  console.log(`Usage: node scripts/sync-original-content.mjs [options]

Options:
  --core-source PATH   Core noname repository or apps/core directory
  --source PATH        Alias of --core-source
  --skin-source PATH   Optional local skin repository
  --verify-only        Verify generated output without changing it
  --ast-self-check     Parse supported data-only JS and print counts
  --help, -h           Show this help`);
  process.exit(0);
}
validateCliArgs(cliArgs);

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const defaultSource = path.resolve(repoRoot, "../archive/upstream/noname");
const legacySourceInput = pathFlag("--source");
const explicitCoreSourceInput = pathFlag("--core-source");
const skinSourceInput = pathFlag("--skin-source");
if (legacySourceInput && explicitCoreSourceInput) {
  throw new Error("--source is an alias of --core-source; use only one");
}
const coreSourceRoot = await resolveContentRoot(
  explicitCoreSourceInput || legacySourceInput || process.env.NONAME_ORIGINAL_ROOT || defaultSource,
);
const skinSourceRoot = skinSourceInput
  ? await resolveContentRoot(skinSourceInput)
  : null;
const verifyOnly = process.argv.includes("--verify-only");
const astSelfCheck = process.argv.includes("--ast-self-check");
let provenanceCache = null;
const outputRoot = path.join(repoRoot, "content/original");
const assetRoot = path.join(outputRoot, "assets");
const legacySourceRoot = path.join(outputRoot, "legacy-source");
const portedRoot = path.join(outputRoot, "ported");
const metadataOnlySkillKeys = new Set(["ai", "audio", "audioname", "audioname2", "charlotte"]);
const presentationOnlySkillKeys = new Set(["mark", "marktext"]);
// Populated lazily by loadEmptyBodySkills during buildCatalog.
const emptyBodySkillCache = new Map();
const reviewedClientOnlySkills = new Set([
  // AI ordering plus cleanup of that client-side global helper; no authority rule effect.
  "diy:nslongyue_ai",
  // Client AI ordering only; no authority rule effect.
  "yingbian:ciwei_ai",
  // Shorthand for legacy client-side friendship/name lookup; authority uses relation queries.
  "mobile:friendgongli",
  // Legacy mark intro cleanup only; logging and client storage deletion have no authority rule effect.
  "newjiang:weifeng2",
]);
const reviewedCompositeSkillContainers = new Set([
  // Legacy group container; authoritative behavior is emitted by the five Jilue branches.
  "extra:jilue",
  // Character-owned limited marker; ABI 381 resolves its awakening inside paired twzhengjian.
  "tw:twzhongchi",
  // Presence marker; ResolvePottuntian action 3 emits Zaoxian's remove-mark rewards.
  "bingshi:potzaoxian",
  // Character-owned shell; ResolvePotweiyan action 2 emits Zhuangshi's costs and helpers.
  "bingshi:potzhuangshi",
]);
const foldedIntoParentSkills = new Map([
  // Legacy global helper; its hand-limit aura is emitted by the authoritative xianghai definition.
  ["shiji:xianghai_g", "shiji:xianghai"],
  // Legacy UI/storage counter; authoritative match-lifetime history is tracked by the engine.
  ["shiji:qinzheng_count", "shiji:qinzheng"],
  // Legacy grouped Duel-resolution helper; authoritative behavior is emitted by noda_xunxin.
  ["key:noda_xunxin2", "key:noda_xunxin"],
  // Legacy final-slot helper; authoritative lifetime trigger is emitted by chihaya_liewu.
  ["key:chihaya_liewu2", "key:chihaya_liewu"],
  // Legacy UI/storage counter; authoritative turn history is emitted by mobilejingce.
  ["mobile:mobilejingce_count", "mobile:mobilejingce"],
  // Legacy damage-draw helper; ResolveBeizhu tracks only direct Beizhu Sha damage per point.
  ["mobile:beizhu_draw", "mobile:beizhu"],
  // Legacy converted-Sha color helper; authoritative behavior is part of hinata_qiulve view-as.
  ["key:hinata_qiulve_clear", "key:hinata_qiulve"],
  // Legacy grouped after-compare helper; authoritative behavior is emitted by hanzhan itself.
  ["refresh:hanzhan_gain", "refresh:hanzhan"],
  // Legacy grouped phase-play helper; authoritative behavior is emitted by sidi itself.
  ["yijiang:sidi2", "yijiang:sidi"],
  // Legacy lord/group helpers; each rule surface is emitted by its authoritative parent.
  ["shenhua:baonue2", "shenhua:baonue"],
  ["shenhua:songwei2", "shenhua:songwei"],
  ["shenhua:bazhen_bagua", "shenhua:bazhen"],
  ["shenhua:wansha2", "shenhua:wansha"],
  // Legacy grouped discard-phase helper; authoritative behavior is emitted by Renjie itself.
  ["extra:renjie2", "extra:renjie"],
  // Qixing/weather helpers are emitted by the parent effects and their source-scoped expiry.
  ["extra:qixing2", "extra:qixing"],
  // Legacy Longhun view-as helpers are emitted as declarative branches on the parent.
  ["extra:longhun1", "extra:longhun"],
  ["extra:longhun2", "extra:longhun"],
  ["extra:longhun3", "extra:longhun"],
  ["extra:longhun4", "extra:longhun"],
  ["extra:dawu3", "extra:dawu"],
  ["extra:kuangfeng3", "extra:kuangfeng"],
  // Duorui cleanup and both global modifiers are emitted by their authoritative parents.
  ["extra:duorui_clear", "extra:drlt_duorui"],
  ["extra:g_drlt_zhiti", "extra:drlt_zhiti"],
  // extra:g_nzry_jieying folded into nzry_jieying until upstream removed the global helper (1.11.5, 2026-08-15 sync).
  // Legacy bookkeeping helpers; their card-use counters/modifiers are emitted by the parents.
  ["sp:shanjia2", "sp:shanjia"],
  ["sp:hanyong3", "sp:hanyong"],
  ["sp:fengpo2", "sp:fengpo"],
  ["sp:qiangwu3", "sp:qiangwu"],
  // Legacy grouped converted-Sha helper; both source-card branches are emitted by Nuzhan.
  ["sp:nuzhan2", "sp:nuzhan"],
  ["sp2:langmie_damage", "sp2:langmie"],
  ["standard:kongcheng1", "standard:kongcheng"],
  ["standard:jijiang1", "standard:jijiang"],
  ["standard:jijiang3", "standard:jijiang"],
  ["standard:rende1", "standard:rende"],
  ["standard:wushuang1", "standard:wushuang"],
  ["standard:wushuang2", "standard:wushuang"],
  ["yijiang:chunlao2", "yijiang:chunlao"],
  ["yijiang:chunlao2_old", "yijiang:chunlao"],
  ["yijiang:lihuo2", "yijiang:lihuo"],
  ["yijiang:lihuo3", "yijiang:lihuo"],
  ["yijiang:reshangshi_2nd", "yijiang:reshangshi"],
  ["yijiang:yonglve2", "yijiang:yonglve"],
  ["yijiang:qiangzhi_draw", "yijiang:qiangzhi"],
  ["yijiang:yanyu2", "yijiang:yanyu"],
]);

// Some characters reference a skill that upstream defines in a different pack, yet the ported
// package publishes its own local copy (a distinct def_id) for content-lock / replay stability.
// Upstream character/<pack>/skill.js does not define such a skill, so buildCatalog would otherwise
// never register it and the character would resolve to the foreign-pack definition — disagreeing
// with the published npack. These entries pin the local copy: during buildCatalog each listed
// local key is synthesized from the published ported definition, and the owning character resolves
// to it instead of the foreign-pack alias. Documented in docs/migration/upstream-1.11.5-delta.md.
const crossPackagePortedSkills = new Map([
  // shenhua:zhangxiu lists twjuxiang (defined upstream in tw), but the shenhua pack publishes a
  // local lord-only twjuxiang copy (1234567907); resolve the character to that local def_id.
  ["shenhua:twjuxiang", { referencedBy: ["shenhua:zhangxiu"], source: "legacy-source/character/shenhua/character.js" }],
]);

const trees = [
  {
    name: "legacy-source",
    sourceCategory: "core",
    sourceRoot: coreSourceRoot,
    source: path.join(coreSourceRoot, "character"),
    target: path.join(legacySourceRoot, "character"),
    hardlink: false,
    required: true,
  },
  {
    name: "portraits",
    sourceCategory: "core",
    sourceRoot: coreSourceRoot,
    source: path.join(coreSourceRoot, "image/character"),
    target: path.join(assetRoot, "portraits"),
    hardlink: true,
    required: true,
  },
  {
    name: "static-skins",
    sourceCategory: "skin",
    sourceRoot: skinSourceRoot,
    source: skinSourceRoot && path.join(skinSourceRoot, "extension/千幻聆音/sanguoskin"),
    target: path.join(assetRoot, "static-skins"),
    hardlink: true,
    required: false,
  },
  {
    name: "static-skins-decade",
    sourceCategory: "skin",
    sourceRoot: skinSourceRoot,
    source: skinSourceRoot &&
      path.join(skinSourceRoot, "extension/十周年UI/assets/千幻聆音/sanguoskin"),
    target: path.join(assetRoot, "static-skins-decade"),
    hardlink: true,
    required: false,
  },
  {
    name: "dynamic-skins",
    sourceCategory: "skin",
    sourceRoot: skinSourceRoot,
    source: skinSourceRoot && path.join(skinSourceRoot, "extension/十周年UI/assets/dynamic"),
    target: path.join(assetRoot, "dynamic-skins"),
    hardlink: true,
    required: false,
  },
];

const managedSkinFiles = [
  {
    source: "extension/千幻聆音/LICENSE",
    target: "skins/千幻聆音-LICENSE",
    trees: ["static-skins"],
  },
  {
    source: "extension/千幻聆音/readme.md",
    target: "skins/千幻聆音-README.md",
    trees: ["static-skins"],
  },
  {
    source: "extension/千幻聆音/skinShare.js",
    target: "skins/skinShare.js",
    trees: ["static-skins", "static-skins-decade"],
  },
  {
    source: "extension/十周年UI/LICENSE",
    target: "skins/十周年UI-LICENSE",
    trees: ["static-skins-decade", "dynamic-skins"],
  },
  {
    source: "extension/十周年UI/src/skins/dynamicSkin.js",
    target: "skins/dynamicSkin.js",
    trees: ["dynamic-skins"],
  },
];

await requireDirectory(coreSourceRoot);
if (skinSourceRoot) await requireDirectory(skinSourceRoot);
const availableTrees = [];
for (const tree of trees) {
  if (tree.source && await fileExists(tree.source)) availableTrees.push(tree);
  else if (tree.required) await requireDirectory(tree.source);
}
const packNames = (await readdir(path.join(coreSourceRoot, "character"), { withFileTypes: true }))
  .filter((entry) => entry.isDirectory())
  .map((entry) => entry.name)
  .sort();

if (astSelfCheck) {
  await runAstSelfCheck(packNames);
  process.exit(0);
}

if (verifyOnly) {
  const results = [];
  for (const tree of availableTrees) results.push(await verifyTree(tree));
  await verifyCatalog(results);
  printResults("verified", results);
  process.exit(0);
}

await mkdir(outputRoot, { recursive: true });
await mkdir(legacySourceRoot, { recursive: true });
await removeStaleOptionalTargets();

// Resolve skill sources per pack: classic `skill.js`, or split `skill/*.js` (excluding
// aggregator `skill/index.js`, as used by offline since upstream 1.11.5).
const skillFileEntries = [];
for (const pack of packNames) {
  for (const file of await listPackSkillFiles(pack)) {
    skillFileEntries.push({ pack, file });
  }
}

await run("cargo", [
  "run",
  "-q",
  "-p",
  "legacy-analyzer",
  "--",
  "matrix",
  ...skillFileEntries.map(({ pack, file }) => `${pack}=${file}`),
  "--json",
  path.join(outputRoot, "skill-capability-matrix.json"),
  "--md",
  path.join(outputRoot, "skill-capability-matrix.md"),
]);

const synced = [];
for (const tree of availableTrees) synced.push(await syncTree(tree));

for (const file of ["LICENSE", "package.json"]) {
  const source = path.join(coreSourceRoot, file);
  if (await fileExists(source)) await copyFile(source, path.join(legacySourceRoot, file));
}
await rm(path.join(legacySourceRoot, "skins"), { recursive: true, force: true });
for (const managedFile of managedSkinFiles) {
  if (!managedSkinFileEnabled(managedFile)) continue;
  const sourceFile = path.join(skinSourceRoot, managedFile.source);
  if (!(await fileExists(sourceFile))) continue;
  const targetFile = path.join(legacySourceRoot, managedFile.target);
  await mkdir(path.dirname(targetFile), { recursive: true });
  await copyFile(sourceFile, targetFile);
}

const catalog = await buildCatalog(packNames, synced);
await writeJson(path.join(outputRoot, "catalog.json"), catalog);
await syncWebStandardAssets(catalog);
const provenance = await contentProvenance();
await writeJson(path.join(outputRoot, "asset-manifest.json"), {
  schema_version: 2,
  source: sourceLabel(),
  sources: provenance,
  trees: synced.map(({ name, sourceCategory, files, bytes, hardlinked, copied }) => ({
    name,
    source_category: sourceCategory,
    files,
    bytes,
    hardlinked,
    copied,
  })),
  assets: await assetEntries(),
});

const verified = [];
for (const tree of availableTrees) verified.push(await verifyTree(tree));
await verifyCatalog(verified);
printResults("synced and verified", verified);

async function buildCatalog(packs, treeResults) {
  const matrix = JSON.parse(
    await readFile(path.join(outputRoot, "skill-capability-matrix.json"), "utf8"),
  );
  const profileByKey = new Map(
    matrix.skills.map((skill) => [`${skill.source}:${skill.name}`, skill]),
  );
  const migrated = await portedDefinitions();
  const characters = [];
  const skills = [];
  const skillByPackAndName = new Map();
  const skillByName = new Map();

  for (const pack of packs) {
    const packRoot = path.join(coreSourceRoot, "character", pack);
    const characterFile = path.join(packRoot, "character.js");
    const packSkillFiles = await listPackSkillFiles(pack);
    const translateFile = path.join(packRoot, "translate.js");
    if (!(await fileExists(characterFile)) || packSkillFiles.length === 0) continue;

    const translations = (await fileExists(translateFile))
      ? simpleObjectValues(await parseObjectFile(translateFile, "translates", true))
      : new Map();
    const skillRows = new Map();
    for (const skillFile of packSkillFiles) {
      const skillObject = await parseObjectFile(skillFile, "skills");
      const relativeSource = path.relative(coreSourceRoot, skillFile).split(path.sep).join("/");
      for (const property of skillObject.properties) {
        const name = propertyName(property);
        if (!name) continue;
        if (skillRows.has(name)) {
          throw new Error(`${pack}:${name}: duplicate skill definition across ${skillRows.get(name).source} and legacy-source/character/${relativeSource}`);
        }
        const line = skillObject.sourceFile.getLineAndCharacterOfPosition(
          property.getStart(skillObject.sourceFile),
        ).line + 1;
        const profile = profileByKey.get(`${pack}:${name}`);
        const skillKey = `${pack}:${name}`;
        const migratedDefId = migrated.skills.get(skillKey);
        const coveredBy = foldedIntoParentSkills.get(skillKey);
        const placeholderBehaviorful =
          migratedDefId !== undefined &&
          migrated.behaviorlessSkills.has(skillKey) &&
          !reviewedClientOnlySkills.has(skillKey) &&
          !reviewedCompositeSkillContainers.has(skillKey) &&
          !foldedIntoParentSkills.has(skillKey) &&
          (profileHasRuleSurface(profile) || !legacySkillIsMetadataOnly(property)) &&
          legacySkillHasAuthoritativeRuleSurface(property);
        let placeholderGap = false;
        if (placeholderBehaviorful) {
          if (await isEmptyBodySkill(pack, name)) {
            // ADR 0597: empty-body placeholder is a known fidelity gap, not a build error.
            placeholderGap = true;
          } else {
            throw new Error(`${skillKey}: behaviorful legacy skill cannot be ported as an empty DSL skill`);
          }
        }
        skillRows.set(name, {
          id: skillKey,
          def_id: migratedDefId ?? stableDefId("skill", `${pack}:${name}`),
          pack,
          name,
          display_name: translations.get(name) || name,
          description: translations.get(`${name}_info`) || null,
          source: `legacy-source/character/${relativeSource}`,
          line,
          migration_status: placeholderGap
            ? "source_synced"
            : migratedDefId === undefined && coveredBy === undefined
              ? "source_synced"
              : "ported",
          ...(placeholderGap ? { placeholder: true } : {}),
          ...(coveredBy === undefined ? {} : { covered_by: coveredBy }),
          profile: profile
            ? {
                grade: profile.grade,
                triggers: profile.triggers,
                active_windows: profile.active_windows,
                locked: profile.locked,
                optional_review: profile.optional_review,
                view_as: profile.view_as,
                modifier: profile.modifier,
                mapped_effects: profile.mapped_effects,
                required_surfaces: profile.required_surfaces,
                gaps: profile.gaps,
                capability_hits: profile.capability_hits,
              }
            : null,
          referenced_by: [],
        });
      }
    }
    for (const skill of skillRows.values()) {
      skills.push(skill);
      skillByPackAndName.set(`${pack}:${skill.name}`, skill);
      const matches = skillByName.get(skill.name) || [];
      matches.push(skill);
      skillByName.set(skill.name, matches);
    }

    const characterObject = await parseObjectFile(characterFile, "characters");
    for (const property of characterObject.properties) {
      const name = propertyName(property);
      if (!name || !ts.isPropertyAssignment(property) || !ts.isObjectLiteralExpression(property.initializer)) {
        continue;
      }
      const data = objectValue(property.initializer);
      const portraitSource = typeof data.img === "string"
        ? data.img
        : `image/character/${name}.jpg`;
      const portraitFile = path.join(coreSourceRoot, portraitSource);
      const portrait = (await fileExists(portraitFile))
        ? `assets/portraits/${path.basename(portraitSource)}`
        : null;
      const characterKey = `${pack}:${name}`;
      const migratedDefId = portedCharacterDefId(migrated, characterKey);
      characters.push({
        id: characterKey,
        def_id: migratedDefId ?? stableDefId("character", characterKey),
        migration_status: migratedDefId === undefined ? "source_synced" : "ported",
        pack,
        name,
        display_name: translations.get(name) || name,
        sex: data.sex ?? null,
        group: data.group ?? null,
        hp: data.hp ?? null,
        max_hp: data.maxHp ?? data.hp ?? null,
        // ADR 0542: dualside is a dual-face character marker, not a rule skill.
        skills: Array.isArray(data.skills)
          ? data.skills.filter((name) => typeof name === "string" && name && name !== "dualside")
          : [],
        resolved_skills: [],
        names: data.names ?? null,
        clans: data.clans ?? [],
        is_lord: data.isZhugong === true,
        portrait,
        source: `legacy-source/character/${pack}/character.js`,
      });
    }
  }

  // Materialize cross-pack ported-skill local copies (see crossPackagePortedSkills). These skills
  // are not defined upstream in the owning pack, so the loop above never registered them; synthesize
  // a ported catalog entry from the published npack def_id so referenced characters resolve locally
  // and agree with the published character_index instead of a foreign-pack alias.
  for (const [skillKey, meta] of crossPackagePortedSkills) {
    if (skillByPackAndName.has(skillKey)) continue;
    const defId = migrated.skills.get(skillKey);
    if (defId === undefined) {
      throw new Error(`${skillKey}: crossPackagePortedSkills entry has no published ported def_id`);
    }
    const [pack, name] = skillKey.split(":");
    const entry = {
      id: skillKey,
      def_id: defId,
      pack,
      name,
      display_name: name,
      description: null,
      source: meta.source,
      line: null,
      // Cross-pack local copies are published in the owning pack's npack, hence ported.
      migration_status: "ported",
      profile: null,
      referenced_by: [],
    };
    skills.push(entry);
    skillByPackAndName.set(skillKey, entry);
    const matches = skillByName.get(name) || [];
    matches.push(entry);
    skillByName.set(name, matches);
  }

  const unresolvedSkillRefs = [];
  for (const character of characters) {
    for (const name of character.skills) {
      const local = skillByPackAndName.get(`${character.pack}:${name}`);
      const matches = local ? [local] : skillByName.get(name) || [];
      if (matches.length === 1) {
        character.resolved_skills.push(matches[0].id);
        matches[0].referenced_by.push(character.id);
      } else {
        unresolvedSkillRefs.push({
          character: character.id,
          skill: name,
          reason: matches.length === 0 ? "definition_not_in_character_packs" : "ambiguous_definition",
        });
      }
    }
  }
  const skillsById = new Map(skills.map((skill) => [skill.id, skill]));
  for (const character of characters) {
    const publishedSkills = portedCharacterSkills(migrated, character.id);
    const resolved = character.resolved_skills.map((id) => skillsById.get(id));
    const complete = publishedSkills !== undefined &&
      resolved.length === character.skills.length &&
      resolved.every((skill) => skill?.migration_status === "ported") &&
      publishedSkills.length === resolved.length &&
      publishedSkills.every((defId, index) => defId === resolved[index].def_id);
    character.migration_status = complete ? "ported" : "source_synced";
  }

  for (const [skillKey, defId] of migrated.skills) {
    // The bootstrap standard package predates stable original-content IDs.
    if (skillKey.startsWith("standard:")) continue;
    if (skillByPackAndName.has(skillKey)) continue;
    // Package-only helpers have no upstream catalog identity. Their already-published def_id is
    // the compatibility anchor for replays and content locks, so do not retroactively renumber it.
    // New helpers should still be allocated with stableDefId before first publication.
  }

  characters.sort((a, b) => compareContentNames(a.id, b.id));
  skills.sort((a, b) => compareContentNames(a.id, b.id));
  assertUniqueDefIds("character", characters);
  assertUniqueDefIds("skill", skills);
  for (const skill of skills) skill.referenced_by.sort();

  const staticSkins = await staticSkinCatalog();
  const substitutes = await coreCharacterSubstitutes(packs);
  const aliases = await staticSkinAliases();
  const dynamicCatalog = await dynamicSkinCatalog();
  const dynamicSkins = dynamicCatalog.skins;
  const dynamicAliases = dynamicCatalog.aliases;
  const provenance = await contentProvenance();
  const characterIds = new Set(characters.map((character) => character.name));
  const missingPortraits = characters
    .filter((character) => !character.portrait)
    .map((character) => character.id);

  return {
    schema_version: 4,
    source: {
      repository: sourceLabel(),
      version: JSON.parse(await readFile(path.join(coreSourceRoot, "package.json"), "utf8")).version,
      code_license: "GPL-3.0-only",
      asset_license: skinSourceRoot
        ? "local_unreviewed_non_redistributable"
        : "preserved_from_core_source; redistribution requires source-license review",
      skin_asset_policy: skinSourceRoot
        ? "local_unreviewed_non_redistributable"
        : "not_configured",
      provenance,
    },
    counts: {
      packs: new Set(characters.map((character) => character.pack)).size,
      characters: characters.length,
      migrated_characters: characters.filter(
        (character) => character.migration_status === "ported",
      ).length,
      skills: skills.length,
      migrated_skills: skills.filter((skill) => skill.migration_status === "ported").length,
      profiled_skills: skills.filter((skill) => skill.profile).length,
      portraits: treeResults.find((tree) => tree.name === "portraits")?.files ?? 0,
      missing_character_portraits: missingPortraits.length,
      static_skins: staticSkins.length,
      dynamic_skins: dynamicSkins.length,
      substitutes: substitutes.length,
      skin_aliases: aliases.length,
      dynamic_skin_aliases: dynamicAliases.length,
      unresolved_skill_references: unresolvedSkillRefs.length,
    },
    packages: [...new Set(characters.map((character) => character.pack))],
    characters,
    skills,
    skins: {
      static: staticSkins,
      dynamic: dynamicSkins,
      substitutes,
      aliases,
      dynamic_aliases: dynamicAliases,
      character_keys_without_roster_match: [
        ...new Set(
          [...staticSkins, ...dynamicSkins]
            .map((skin) => skin.character)
            .filter((name) => !characterIds.has(name)),
        ),
      ].sort(),
    },
    unresolved_skill_references: unresolvedSkillRefs,
    missing_portraits: missingPortraits,
  };
}

async function portedDefinitions() {
  const files = [path.join(repoRoot, "content/standard/standard.npack.json")];
  if (await fileExists(portedRoot)) {
    for (const relative of await listFiles(portedRoot)) {
      if (relative.endsWith(".npack.json")) files.push(path.join(portedRoot, relative));
    }
  }

  const skills = new Map();
  const behaviorlessSkills = new Set();
  const characters = new Map();
  const characterIds = new Set();
  const characterSkills = new Map();
  const characterSkillsById = new Map();
  for (const file of files.sort()) {
    const pkg = JSON.parse(await readFile(file, "utf8"));
    const pack = pkg.manifest.package_id;
    const behaviorfulSkillIds = new Set(
      [
        ...(pkg.trigger_index || []),
        ...(pkg.modifier_index || []),
        ...(pkg.active_index || []),
        ...(pkg.view_as_index || []),
      ].map((entry) => entry.skill_def_id),
    );
    for (const skill of pkg.skill_index) {
      const key = `${pack}:${skill.name}`;
      insertPortedDefinition(skills, key, skill.skill_def_id, file);
      if (!behaviorfulSkillIds.has(skill.skill_def_id)) behaviorlessSkills.add(key);
    }
    for (const character of pkg.character_index) {
      if (!character.sex) throw new Error(`${pack}:${character.name_key}: missing character sex`);
      if (!character.group) throw new Error(`${pack}:${character.name_key}: missing character group`);
      const key = `${pack}:${character.name_key}`;
      insertPortedDefinition(characters, key, character.id, file);
      characterSkills.set(key, character.skills);
      characterSkillsById.set(character.id, character.skills);
      characterIds.add(character.id);
    }
  }
  return {
    skills,
    behaviorlessSkills,
    characters,
    characterIds,
    characterSkills,
    characterSkillsById,
  };
}

function portedCharacterDefId(migrated, key) {
  const exact = migrated.characters.get(key);
  if (exact !== undefined) return exact;
  const stable = stableDefId("character", key);
  return migrated.characterIds.has(stable) ? stable : undefined;
}

function portedCharacterSkills(migrated, key) {
  return migrated.characterSkills.get(key) ??
    migrated.characterSkillsById.get(stableDefId("character", key));
}

function profileHasRuleSurface(profile) {
  return Boolean(
    profile &&
      ((profile.triggers || []).length ||
        (profile.active_windows || []).length ||
        profile.view_as ||
        profile.modifier ||
        (profile.mapped_effects || []).length ||
        (profile.required_surfaces || []).length),
  );
}

// A legacy skill with no authoritative rule surface (no trigger/filter/content/mod/viewAs/enable
// or other rule-bearing keys) carries only AI/UI/metadata. Per the project fidelity policy (AI/UI
// is not authoritative), such a skill is correctly an empty DSL body — it is ported, not a
// placeholder. The metadata-key allowlist above only covers simple shapes; this catches the broader
// set of skills whose JS carries only `ai:`, `init:`-without-effect, `subSkill:` containers, etc.
const authoritativeRuleKeys = new Set([
  "trigger", "filter", "content", "mod", "viewAs", "enable", "usable", "filterTarget",
  "filterCard", "selectCard", "selectTarget", "check", "cost", "onUse", "onLose",
  " popup ", // placeholder to avoid false matches; real check below
]);
function legacySkillHasAuthoritativeRuleSurface(property) {
  if (!ts.isPropertyAssignment(property) || !ts.isObjectLiteralExpression(property.initializer)) {
    return false;
  }
  for (const item of property.initializer.properties) {
    const name = propertyName(item);
    if (name === null) continue;
    // Rule-bearing keys: presence means the skill has authoritative game behavior to port.
    if (
      name === "trigger" || name === "content" || name === "filter" || name === "mod" ||
      name === "viewAs" || name === "enable" || name === "filterTarget" || name === "filterCard" ||
      name === "selectCard" || name === "selectTarget" || name === "cost" || name === "onUse" ||
      name === "onLose" || name === "usable" || name === "group" || name === "global" ||
      name === " derivation " // placeholder
    ) {
      return true;
    }
  }
  return false;
}

function legacySkillIsMetadataOnly(property) {
  return ts.isPropertyAssignment(property) &&
    ts.isObjectLiteralExpression(property.initializer) &&
    property.initializer.properties.every((item) => {
      const name = propertyName(item);
      if (name === null) return false;
      if (metadataOnlySkillKeys.has(name) || presentationOnlySkillKeys.has(name)) return true;
      if (["locked", "nopop", "equipSkill", "zhuSkill"].includes(name)) {
        return ts.isPropertyAssignment(item) &&
          [ts.SyntaxKind.TrueKeyword, ts.SyntaxKind.FalseKeyword].includes(item.initializer.kind);
      }
      if (name === "onremove") {
        return ts.isPropertyAssignment(item) && item.initializer.kind === ts.SyntaxKind.TrueKeyword;
      }
      if (name === "intro") {
        return ts.isPropertyAssignment(item) &&
          ts.isObjectLiteralExpression(item.initializer) &&
          item.initializer.properties.every((entry) => propertyName(entry) !== "onunmark");
      }
      return false;
    });
}

async function metadataOnlyLegacySkills(packs) {
  const result = new Set(reviewedClientOnlySkills);
  for (const pack of packs) {
    for (const file of await listPackSkillFiles(pack)) {
      const object = await parseObjectFile(file, "skills");
      for (const property of object.properties) {
        const name = propertyName(property);
        if (name && legacySkillIsMetadataOnly(property)) result.add(`${pack}:${name}`);
      }
    }
  }
  return result;
}

// Set of legacy skills that DO have an authoritative rule surface (trigger/content/filter/mod/
// viewAs/enable/etc.). Skills NOT in this set are pure AI/UI/metadata and an empty DSL body is
// correct for them. Used by verifyCatalog, which iterates catalog skills (no JS property in hand).
async function authoritativeSurfaceLegacySkills(packs) {
  const result = new Set();
  for (const pack of packs) {
    for (const file of await listPackSkillFiles(pack)) {
      const object = await parseObjectFile(file, "skills");
      for (const property of object.properties) {
        const name = propertyName(property);
        if (name && legacySkillHasAuthoritativeRuleSurface(property)) result.add(`${pack}:${name}`);
      }
    }
  }
  return result;
}

// List authoritative skill definition files for a character pack.
// Supports classic single-file `skill.js` and split `skill/<module>.js` layouts
// (aggregator `skill/index.js` is skipped — it only re-exports module objects).
async function listPackSkillFiles(pack) {
  const packRoot = path.join(coreSourceRoot, "character", pack);
  const single = path.join(packRoot, "skill.js");
  if (await fileExists(single)) return [single];
  const skillDir = path.join(packRoot, "skill");
  if (!(await fileExists(skillDir))) return [];
  const entries = await readdir(skillDir, { withFileTypes: true });
  return entries
    .filter((entry) => entry.isFile() && entry.name.endsWith(".js") && entry.name !== "index.js")
    .map((entry) => path.join(skillDir, entry.name))
    .sort();
}
function insertPortedDefinition(map, key, defId, file) {
  const existing = map.get(key);
  if (existing !== undefined && existing !== defId) {
    throw new Error(`${key}: conflicting def ids ${existing}/${defId} in ${file}`);
  }
  map.set(key, defId);
}

// A ported skill whose .skill source body is `skill <name>(<id>) {}` (empty body, comments
// ignored, optional locked/lord_only/conversion/non_gainable modifiers) is an intentional
// placeholder per ADR 0597's fidelity ledger (TODO/approximate skeletons). Such skills carry a
// def_id in the npack but no behavior module; buildCatalog records them as source_synced rather
// than throwing, so the catalog honestly reflects the fidelity gap instead of blocking regen.
// Skills may live in per-skill files (`<name>.skill`) or batch files (`*_meta.skill`, etc.).
async function loadEmptyBodySkills(pack) {
  if (emptyBodySkillCache.has(pack)) return emptyBodySkillCache.get(pack);
  const set = new Set();
  const emptyBodyRegex = /skill\s+(\w+)\s*\(\s*\d+\s*\)\s*(?:locked\s+|lord_only\s+|conversion\s+|non_gainable\s+)*\{\s*\}/g;
  const packDir = path.join(portedRoot, pack);
  if (await fileExists(packDir)) {
    for (const relative of await listFiles(packDir)) {
      if (!relative.endsWith(".skill")) continue;
      const text = await readFile(path.join(packDir, relative), "utf8");
      const stripped = text.replace(/\/\/.*/g, "").replace(/\/\*.*?\*/gs, "");
      for (const match of stripped.matchAll(emptyBodyRegex)) {
        set.add(match[1]);
      }
    }
  }
  emptyBodySkillCache.set(pack, set);
  return set;
}
async function isEmptyBodySkill(pack, skillName) {
  return (await loadEmptyBodySkills(pack)).has(skillName);
}

function stableDefId(kind, id) {
  const bytes = createHash("sha256").update(`noname-next/original/${kind}/${id}`).digest();
  return (bytes.readUInt32BE(0) | 0x80000000) >>> 0;
}

function assertUniqueDefIds(kind, rows) {
  const byId = new Map();
  for (const row of rows) {
    const existing = byId.get(row.def_id);
    if (existing && existing !== row.id) {
      throw new Error(`${kind} def id collision ${row.def_id}: ${existing}/${row.id}`);
    }
    byId.set(row.def_id, row.id);
  }
}

async function staticSkinCatalog() {
  if (!skinSourceRoot) return [];
  const roots = [
    ["static", "extension/千幻聆音/sanguoskin", "static-skins"],
    ["static-decade", "extension/十周年UI/assets/千幻聆音/sanguoskin", "static-skins-decade"],
  ];
  const skins = [];
  for (const [idPrefix, source, target] of roots) {
    const sourcePath = path.join(skinSourceRoot, source);
    if (!(await fileExists(sourcePath))) continue;
    const files = await listFiles(sourcePath);
    for (const relative of files.filter((file) => /\.(?:avif|jpe?g|png|webp)$/i.test(file))) {
      const [character] = relative.split(path.sep);
      skins.push({
        id: `${idPrefix}:${relative.split(path.sep).join(":")}`,
        character,
        name: path.basename(relative, path.extname(relative)),
        asset: `assets/${target}/${relative.split(path.sep).join("/")}`,
        source_category: "skin",
        redistribution: "local_unreviewed_non_redistributable",
      });
    }
  }
  return skins;
}

async function coreCharacterSubstitutes(packs) {
  const rows = [];
  for (const pack of packs) {
    const file = path.join(coreSourceRoot, "character", pack, "index.js");
    if (!(await fileExists(file))) continue;
    const sourceFile = await parseJavaScript(file);
    const substituteObject = findNamedObjectProperty(sourceFile, "characterSubstitute");
    if (!substituteObject) continue;
    for (const characterProperty of substituteObject.properties) {
      const character = propertyName(characterProperty);
      if (!character || !ts.isPropertyAssignment(characterProperty) ||
        !ts.isArrayLiteralExpression(characterProperty.initializer)) continue;
      for (const entry of characterProperty.initializer.elements) {
        const parsed = safeLiteral(entry, "$");
        if (!parsed.ok || !Array.isArray(parsed.value) || typeof parsed.value[0] !== "string") {
          continue;
        }
        const substitute = parsed.value[0];
        const line = sourceFile.getLineAndCharacterOfPosition(entry.getStart(sourceFile)).line + 1;
        rows.push({
          id: `substitute:${pack}:${character}:${substitute}`,
          pack,
          character,
          substitute,
          directives: Array.isArray(parsed.value[1]) ? parsed.value[1] : [],
          asset: await portraitAssetFor(substitute),
          source_category: "core",
          source: `legacy-source/character/${pack}/index.js`,
          line,
          ...(parsed.omitted.length ? { omitted_non_literal_fields: parsed.omitted } : {}),
        });
      }
    }
  }
  rows.sort((a, b) => compareContentNames(a.id, b.id));
  return rows;
}

async function staticSkinAliases() {
  if (!skinSourceRoot ||
    (!treeAvailable("static-skins") && !treeAvailable("static-skins-decade"))) return [];
  const file = path.join(skinSourceRoot, "extension/千幻聆音/skinShare.js");
  if (!(await fileExists(file))) return [];
  const sourceFile = await parseJavaScript(file, true);
  const aliasObject = findAssignedObject(sourceFile, ["lib", "qhly_skinShare"]);
  if (!aliasObject) {
    throw new Error(`${file}: lib.qhly_skinShare = { ... } not found`);
  }
  const byCharacter = new Map();
  for (const property of aliasObject.properties) {
    const character = propertyName(property);
    if (!character || !ts.isPropertyAssignment(property)) continue;
    const parsed = safeLiteral(property.initializer, "$");
    if (!parsed.ok || !isPlainObject(parsed.value) || typeof parsed.value.name !== "string") continue;
    const line = sourceFile.getLineAndCharacterOfPosition(property.getStart(sourceFile)).line + 1;
    const skills = isPlainObject(parsed.value.skills) ? parsed.value.skills : {};
    byCharacter.set(character, {
      id: `skin-alias:${character}`,
      character,
      target_character: parsed.value.name,
      skill_audio_map: skills,
      config: parsed.value,
      source_category: "skin",
      redistribution: "local_unreviewed_non_redistributable",
      source: "legacy-source/skins/skinShare.js",
      line,
      ...(parsed.omitted.length ? { omitted_non_literal_fields: parsed.omitted } : {}),
    });
  }
  return [...byCharacter.values()].sort((a, b) => compareContentNames(a.id, b.id));
}

async function dynamicSkinCatalog() {
  if (!skinSourceRoot || !treeAvailable("dynamic-skins")) return { skins: [], aliases: [] };
  const configFile = path.join(skinSourceRoot, "extension/十周年UI/src/skins/dynamicSkin.js");
  if (!(await fileExists(configFile))) return { skins: [], aliases: [] };
  const sourceFile = await parseJavaScript(configFile);
  const configObject = findVariableObject(sourceFile, "dynamicSkinConfig");
  if (!configObject) throw new Error(`${configFile}: dynamicSkinConfig object not found`);
  const skins = [];
  for (const characterProperty of configObject.properties) {
    const character = propertyName(characterProperty);
    if (
      !character ||
      !ts.isPropertyAssignment(characterProperty) ||
      !ts.isObjectLiteralExpression(characterProperty.initializer)
    ) continue;
    for (const skinProperty of characterProperty.initializer.properties) {
      const name = propertyName(skinProperty);
      if (!name || !ts.isPropertyAssignment(skinProperty)) continue;
      const parsed = safeLiteral(skinProperty.initializer, "$");
      const data = parsed.ok && isPlainObject(parsed.value) ? parsed.value : {};
      const assetBase = typeof data.name === "string" ? data.name : null;
      const line = sourceFile.getLineAndCharacterOfPosition(
        skinProperty.getStart(sourceFile),
      ).line + 1;
      skins.push({
        id: `dynamic:${character}:${name}`,
        character,
        name,
        asset_base: assetBase ? `assets/dynamic-skins/${assetBase}` : null,
        background: typeof data.background === "string"
          ? `assets/dynamic-skins/${data.background}`
          : null,
        version: data.version ?? null,
        config: data,
        resource_references: collectResourceReferences(data),
        source_category: "skin",
        redistribution: "local_unreviewed_non_redistributable",
        source: "legacy-source/skins/dynamicSkin.js",
        line,
        ...(!parsed.ok || parsed.omitted.length
          ? { omitted_non_literal_fields: parsed.omitted.length ? parsed.omitted : ["$"] }
          : {}),
      });
    }
  }
  skins.sort((a, b) => compareContentNames(a.id, b.id));

  const aliasesByCharacter = new Map();
  const extendObject = findVariableObject(sourceFile, "dynamicSkinExtend");
  if (extendObject) {
    for (const property of extendObject.properties) {
      const character = propertyName(property);
      if (!character || !ts.isPropertyAssignment(property)) continue;
      const expression = expressionPath(property.initializer);
      const normalized = expression?.[0] === "window" ? expression.slice(1) : expression;
      if (
        !normalized ||
        normalized.length !== 3 ||
        normalized[0] !== "decadeUI" ||
        normalized[1] !== "dynamicSkin"
      ) continue;
      const targetCharacter = normalized[2];
      const line = sourceFile.getLineAndCharacterOfPosition(
        property.getStart(sourceFile),
      ).line + 1;
      aliasesByCharacter.set(character, {
        id: `dynamic-alias:${character}`,
        character,
        target_character: targetCharacter,
        target_present: skins.some((skin) => skin.character === targetCharacter),
        source_category: "skin",
        redistribution: "local_unreviewed_non_redistributable",
        source: "legacy-source/skins/dynamicSkin.js",
        line,
      });
    }
  }
  return {
    skins,
    aliases: [...aliasesByCharacter.values()].sort((a, b) => compareContentNames(a.id, b.id)),
  };
}

async function assetEntries() {
  const entries = [];
  for (const tree of availableTrees.filter((item) => item.name !== "legacy-source")) {
    const files = await listFiles(tree.target);
    for (let offset = 0; offset < files.length; offset += 24) {
      const batch = files.slice(offset, offset + 24);
      entries.push(...await Promise.all(batch.map(async (relative) => {
        const targetFile = path.join(tree.target, relative);
        const info = await stat(targetFile);
        return {
          kind: tree.name,
          path: `${tree.name}/${relative.split(path.sep).join("/")}`,
          size: info.size,
          source_category: tree.sourceCategory,
          source_path: path.relative(
            tree.sourceRoot,
            path.join(tree.source, relative),
          ).split(path.sep).join("/"),
          sha256: await sha256(targetFile),
        };
      })));
    }
  }
  return entries;
}

async function syncWebStandardAssets(catalog) {
  const standard = JSON.parse(
    await readFile(path.join(repoRoot, "content/standard/characters.json"), "utf8"),
  );
  const byName = new Map(
    catalog.characters
      .filter((character) => character.pack === "standard")
      .map((character) => [character.name, character]),
  );
  const publicRoot = path.join(repoRoot, "apps/web-client/public/original/portraits");
  await rm(publicRoot, { recursive: true, force: true });
  await mkdir(publicRoot, { recursive: true });
  const rows = [];
  for (const entry of standard) {
    const character = byName.get(entry.name_key);
    if (!character?.portrait) throw new Error(`standard portrait not found: ${entry.name_key}`);
    const source = path.join(outputRoot, character.portrait);
    const extension = path.extname(source).toLowerCase();
    const filename = `${entry.id}${extension}`;
    const target = path.join(publicRoot, filename);
    try {
      await link(source, target);
    } catch (error) {
      if (!["EXDEV", "EPERM", "EACCES", "EMLINK"].includes(error.code)) throw error;
      await copyFile(source, target);
    }
    rows.push({
      id: entry.id,
      name: character.name,
      display_name: character.display_name,
      portrait: `/original/portraits/${filename}`,
    });
  }
  const generated = path.join(repoRoot, "apps/web-client/src/generated");
  await mkdir(generated, { recursive: true });
  await writeJson(path.join(generated, "standard-characters.json"), rows);
}

async function parseObjectFile(file, variableName, allowDiagnostics = false) {
  const sourceFile = await parseJavaScript(file, allowDiagnostics);
  for (const statement of sourceFile.statements) {
    if (!ts.isVariableStatement(statement)) continue;
    for (const declaration of statement.declarationList.declarations) {
      if (
        declaration.name.getText(sourceFile) === variableName &&
        ts.isObjectLiteralExpression(declaration.initializer)
      ) {
        return { sourceFile, properties: declaration.initializer.properties };
      }
    }
  }
  throw new Error(`${file}: const ${variableName} = { ... } not found`);
}

async function parseJavaScript(file, allowDiagnostics = false) {
  const source = await readFile(file, "utf8");
  const sourceFile = ts.createSourceFile(file, source, ts.ScriptTarget.Latest, true, ts.ScriptKind.JS);
  if (!allowDiagnostics && sourceFile.parseDiagnostics.length) {
    throw new Error(`${file}: TypeScript parser reported ${sourceFile.parseDiagnostics.length} errors`);
  }
  return sourceFile;
}

function findNamedObjectProperty(sourceFile, name) {
  let found = null;
  visit(sourceFile);
  return found;

  function visit(node) {
    if (found) return;
    if (
      ts.isPropertyAssignment(node) &&
      propertyName(node) === name &&
      ts.isObjectLiteralExpression(node.initializer)
    ) {
      found = node.initializer;
      return;
    }
    ts.forEachChild(node, visit);
  }
}

function findVariableObject(sourceFile, variableName) {
  let found = null;
  visit(sourceFile);
  return found;

  function visit(node) {
    if (found) return;
    if (ts.isVariableDeclaration(node) &&
      ts.isIdentifier(node.name) &&
      node.name.text === variableName &&
      ts.isObjectLiteralExpression(node.initializer)) {
      found = node.initializer;
      return;
    }
    ts.forEachChild(node, visit);
  }
}

function findAssignedObject(sourceFile, targetPath) {
  let found = null;
  visit(sourceFile);
  return found;

  function visit(node) {
    if (
      ts.isBinaryExpression(node) &&
      node.operatorToken.kind === ts.SyntaxKind.EqualsToken &&
      expressionPath(node.left)?.join(".") === targetPath.join(".") &&
      ts.isObjectLiteralExpression(node.right)
    ) {
      found = node.right;
    }
    ts.forEachChild(node, visit);
  }
}

function expressionPath(node) {
  if (ts.isIdentifier(node)) return [node.text];
  if (ts.isPropertyAccessExpression(node)) {
    const parent = expressionPath(node.expression);
    return parent ? [...parent, node.name.text] : null;
  }
  if (ts.isElementAccessExpression(node) &&
    node.argumentExpression &&
    (ts.isStringLiteral(node.argumentExpression) ||
      ts.isNoSubstitutionTemplateLiteral(node.argumentExpression))) {
    const parent = expressionPath(node.expression);
    return parent ? [...parent, node.argumentExpression.text] : null;
  }
  return null;
}

function safeLiteral(node, location = "$") {
  if (ts.isStringLiteral(node) || ts.isNoSubstitutionTemplateLiteral(node)) {
    return { ok: true, value: node.text, omitted: [] };
  }
  if (ts.isNumericLiteral(node)) {
    const value = Number(node.text);
    return Number.isFinite(value)
      ? { ok: true, value, omitted: [] }
      : { ok: false, omitted: [location] };
  }
  if (node.kind === ts.SyntaxKind.TrueKeyword) {
    return { ok: true, value: true, omitted: [] };
  }
  if (node.kind === ts.SyntaxKind.FalseKeyword) {
    return { ok: true, value: false, omitted: [] };
  }
  if (node.kind === ts.SyntaxKind.NullKeyword) {
    return { ok: true, value: null, omitted: [] };
  }
  if (
    ts.isPrefixUnaryExpression(node) &&
    ts.isNumericLiteral(node.operand) &&
    [ts.SyntaxKind.PlusToken, ts.SyntaxKind.MinusToken].includes(node.operator)
  ) {
    const unsigned = Number(node.operand.text);
    const value = node.operator === ts.SyntaxKind.MinusToken ? -unsigned : unsigned;
    return Number.isFinite(value)
      ? { ok: true, value, omitted: [] }
      : { ok: false, omitted: [location] };
  }
  if (ts.isArrayLiteralExpression(node)) {
    const value = [];
    const omitted = [];
    for (const [index, element] of node.elements.entries()) {
      if (ts.isSpreadElement(element)) return { ok: false, omitted: [`${location}[${index}]`] };
      const parsed = safeLiteral(element, `${location}[${index}]`);
      if (!parsed.ok) return { ok: false, omitted: parsed.omitted };
      value.push(parsed.value);
      omitted.push(...parsed.omitted);
    }
    return { ok: true, value, omitted };
  }
  if (ts.isObjectLiteralExpression(node)) {
    const value = Object.create(null);
    const omitted = [];
    for (const property of node.properties) {
      const name = propertyName(property);
      if (!name || !ts.isPropertyAssignment(property) ||
        ["__proto__", "prototype", "constructor"].includes(name)) {
        omitted.push(`${location}.*`);
        continue;
      }
      const parsed = safeLiteral(property.initializer, `${location}.${name}`);
      if (parsed.ok) value[name] = parsed.value;
      omitted.push(...parsed.omitted);
    }
    return { ok: true, value, omitted };
  }
  return { ok: false, omitted: [location] };
}

function isPlainObject(value) {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

async function portraitAssetFor(character) {
  for (const extension of [".jpg", ".jpeg", ".png", ".webp", ".avif"]) {
    const file = path.join(coreSourceRoot, "image/character", `${character}${extension}`);
    if (await fileExists(file)) return `assets/portraits/${character}${extension}`;
  }
  return null;
}

function collectResourceReferences(value) {
  const references = new Map();
  visit(value, []);
  return [...references.values()].sort((a, b) => compareContentNames(a.field, b.field));

  function visit(current, keys) {
    if (Array.isArray(current)) {
      current.forEach((item, index) => visit(item, [...keys, String(index)]));
      return;
    }
    if (!isPlainObject(current)) return;
    for (const [key, child] of Object.entries(current)) {
      const next = [...keys, key];
      const resourceField = ["name", "background", "image"].includes(key) ||
        next.includes("audio");
      if (resourceField && typeof child === "string") {
        const field = next.join(".");
        references.set(`${field}\0${child}`, {
          field,
          asset: `assets/dynamic-skins/${child}`,
        });
      } else {
        visit(child, next);
      }
    }
  }
}

function simpleObjectValues(parsed) {
  const values = new Map();
  for (const property of parsed.properties) {
    const name = propertyName(property);
    if (!name || !ts.isPropertyAssignment(property)) continue;
    const value = literalValue(property.initializer);
    if (typeof value === "string") values.set(name, value);
  }
  return values;
}

function objectValue(node) {
  const value = {};
  for (const property of node.properties) {
    const name = propertyName(property);
    if (!name || !ts.isPropertyAssignment(property)) continue;
    const parsed = literalValue(property.initializer);
    if (parsed !== undefined) value[name] = parsed;
  }
  return value;
}

function literalValue(node) {
  if (ts.isStringLiteral(node) || ts.isNoSubstitutionTemplateLiteral(node)) return node.text;
  if (ts.isNumericLiteral(node)) return Number(node.text);
  if (node.kind === ts.SyntaxKind.TrueKeyword) return true;
  if (node.kind === ts.SyntaxKind.FalseKeyword) return false;
  if (node.kind === ts.SyntaxKind.NullKeyword) return null;
  if (ts.isPrefixUnaryExpression(node) && ts.isNumericLiteral(node.operand)) {
    const number = Number(node.operand.text);
    return node.operator === ts.SyntaxKind.MinusToken ? -number : number;
  }
  if (ts.isArrayLiteralExpression(node)) {
    const values = node.elements.map(literalValue);
    return values.some((value) => value === undefined) ? undefined : values;
  }
  if (ts.isObjectLiteralExpression(node)) return objectValue(node);
  return undefined;
}

function propertyName(property) {
  const name = property.name;
  if (!name) return null;
  if (ts.isIdentifier(name) || ts.isStringLiteral(name) || ts.isNumericLiteral(name)) return name.text;
  return null;
}

async function removeStaleOptionalTargets() {
  const availableNames = new Set(availableTrees.map((tree) => tree.name));
  for (const tree of trees.filter((item) => !item.required)) {
    if (!availableNames.has(tree.name)) {
      await rm(tree.target, { recursive: true, force: true });
    }
  }
}

async function verifyNoStaleOptionalTargets() {
  const availableNames = new Set(availableTrees.map((tree) => tree.name));
  for (const tree of trees.filter((item) => !item.required)) {
    if (!availableNames.has(tree.name) && await fileExists(tree.target)) {
      throw new Error(`${tree.name}: stale target exists without a configured source`);
    }
  }
}

async function verifyManagedSkinFiles() {
  const expected = [];
  if (skinSourceRoot) {
    for (const managedFile of managedSkinFiles) {
      if (!managedSkinFileEnabled(managedFile)) continue;
      const sourceFile = path.join(skinSourceRoot, managedFile.source);
      if (!(await fileExists(sourceFile))) continue;
      const targetFile = path.join(legacySourceRoot, managedFile.target);
      if (!(await fileExists(targetFile))) {
        throw new Error(`${managedFile.target}: managed skin file is missing`);
      }
      const [sourceHash, targetHash] = await Promise.all([sha256(sourceFile), sha256(targetFile)]);
      if (sourceHash !== targetHash) {
        throw new Error(`${managedFile.target}: managed skin file differs`);
      }
      expected.push(managedFile.target.slice("skins/".length));
    }
  }
  expected.sort(compareFileNames);
  const skinTargetRoot = path.join(legacySourceRoot, "skins");
  const actual = (await fileExists(skinTargetRoot)) ? await listFiles(skinTargetRoot) : [];
  if (expected.join("\0") !== actual.join("\0")) {
    throw new Error("managed skin files contain missing or stale entries");
  }
}

function treeAvailable(name) {
  return availableTrees.some((tree) => tree.name === name);
}

function managedSkinFileEnabled(managedFile) {
  return Boolean(
    skinSourceRoot && managedFile.trees.some((treeName) => treeAvailable(treeName)),
  );
}

async function contentProvenance() {
  if (!provenanceCache) {
    provenanceCache = {
      core: await sourceProvenance("core", coreSourceRoot),
      skin: skinSourceRoot ? await sourceProvenance("skin", skinSourceRoot) : null,
    };
  }
  return provenanceCache;
}

async function sourceProvenance(category, root) {
  const redistributionStatus = category === "skin"
    ? "local_unreviewed_non_redistributable"
    : "preserved_from_source_requires_review";
  const repositoryRoot = await runCapture(
    "git",
    ["-C", root, "rev-parse", "--show-toplevel"],
  ).then((value) => value.trim()).catch(() => null);
  if (!repositoryRoot) {
    return {
      category,
      root: sourceLabel(root),
      repository: null,
      repository_root: null,
      head: null,
      dirty: true,
      tracked_changes: [],
      tracked_change_count: 0,
      untracked_count: (await listFiles(root)).length,
      redistribution_status: redistributionStatus,
    };
  }

  const [head, remote, statusOutput] = await Promise.all([
    runCapture("git", ["-C", repositoryRoot, "rev-parse", "HEAD"]).then((value) => value.trim()),
    runCapture("git", ["-C", repositoryRoot, "remote", "get-url", "origin"])
      .then((value) => value.trim())
      .catch(() => null),
    runCapture("git", [
      "-C",
      repositoryRoot,
      "status",
      "--porcelain=v1",
      "-z",
      "--untracked-files=all",
      "--",
      path.relative(repositoryRoot, root) || ".",
    ]),
  ]);
  const status = parseGitStatus(statusOutput);
  return {
    category,
    root: sourceLabel(root),
    repository: remote ? sanitizeRepository(remote) : sourceLabel(repositoryRoot),
    repository_root: sourceLabel(repositoryRoot),
    head,
    dirty: status.trackedChanges.length > 0 || status.untrackedCount > 0,
    tracked_changes: status.trackedChanges,
    tracked_change_count: status.trackedChanges.length,
    untracked_count: status.untrackedCount,
    redistribution_status: redistributionStatus,
  };
}

function sanitizeRepository(value) {
  try {
    const url = new URL(value);
    url.username = "";
    url.password = "";
    return url.toString();
  } catch {
    return value.replace(/^(https?:\/\/)[^/@]+@/i, "$1");
  }
}

function parseGitStatus(output) {
  const records = output.split("\0");
  const trackedChanges = [];
  let untrackedCount = 0;
  for (let index = 0; index < records.length; index++) {
    const record = records[index];
    if (!record) continue;
    const status = record.slice(0, 2);
    const file = record.slice(3).split(path.sep).join("/");
    if (status === "??") untrackedCount++;
    else if (status !== "!!") trackedChanges.push({ status, path: file });
    if (status.includes("R") || status.includes("C")) index++;
  }
  return { trackedChanges, untrackedCount };
}

async function syncTree(tree) {
  await requireDirectory(tree.source);
  await rm(tree.target, { recursive: true, force: true });
  const files = await listFiles(tree.source);
  let bytes = 0;
  let hardlinked = 0;
  let copied = 0;
  for (const relative of files) {
    const source = path.join(tree.source, relative);
    const target = path.join(tree.target, relative);
    await mkdir(path.dirname(target), { recursive: true });
    const info = await stat(source);
    bytes += info.size;
    if (tree.hardlink) {
      try {
        await link(source, target);
        hardlinked++;
        continue;
      } catch (error) {
        if (!["EXDEV", "EPERM", "EACCES", "EMLINK"].includes(error.code)) throw error;
      }
    }
    await copyFile(source, target);
    copied++;
  }
  return { ...tree, files: files.length, bytes, hardlinked, copied };
}

async function verifyTree(tree) {
  await requireDirectory(tree.source);
  await requireDirectory(tree.target);
  const sourceFiles = await listFiles(tree.source);
  const targetFiles = await listFiles(tree.target);
  if (sourceFiles.join("\0") !== targetFiles.join("\0")) {
    throw new Error(`${tree.name}: source/target file lists differ`);
  }
  let bytes = 0;
  let hardlinked = 0;
  let copied = 0;
  for (const relative of sourceFiles) {
    const source = path.join(tree.source, relative);
    const target = path.join(tree.target, relative);
    const [sourceInfo, targetInfo] = await Promise.all([stat(source), stat(target)]);
    if (sourceInfo.size !== targetInfo.size) throw new Error(`${tree.name}/${relative}: size differs`);
    bytes += sourceInfo.size;
    if (sourceInfo.dev === targetInfo.dev && sourceInfo.ino === targetInfo.ino) {
      hardlinked++;
    } else {
      const [sourceHash, targetHash] = await Promise.all([sha256(source), sha256(target)]);
      if (sourceHash !== targetHash) throw new Error(`${tree.name}/${relative}: content differs`);
      copied++;
    }
  }
  return { ...tree, files: sourceFiles.length, bytes, hardlinked, copied };
}

async function verifyCatalog(results) {
  await verifyNoStaleOptionalTargets();
  await verifyManagedSkinFiles();
  const catalog = JSON.parse(await readFile(path.join(outputRoot, "catalog.json"), "utf8"));
  const manifest = JSON.parse(await readFile(path.join(outputRoot, "asset-manifest.json"), "utf8"));
  if (catalog.schema_version !== 4) throw new Error("catalog schema version mismatch");
  if (manifest.schema_version !== 2) throw new Error("asset manifest schema version mismatch");
  if (catalog.source.repository !== sourceLabel()) throw new Error("catalog core source mismatch");
  if (manifest.source !== sourceLabel()) throw new Error("asset manifest core source mismatch");
  const sourceVersion = JSON.parse(
    await readFile(path.join(coreSourceRoot, "package.json"), "utf8"),
  ).version;
  if (catalog.source.version !== sourceVersion) throw new Error("catalog core version mismatch");
  const expectedSkinPolicy = skinSourceRoot
    ? "local_unreviewed_non_redistributable"
    : "not_configured";
  if (catalog.source.skin_asset_policy !== expectedSkinPolicy) {
    throw new Error("catalog skin asset policy mismatch");
  }
  const provenance = await contentProvenance();
  assertJsonEqual("catalog provenance", catalog.source.provenance, provenance);
  assertJsonEqual("asset manifest provenance", manifest.sources, provenance);
  if (catalog.counts.characters !== catalog.characters.length) throw new Error("catalog character count mismatch");
  if (catalog.counts.skills !== catalog.skills.length) throw new Error("catalog skill count mismatch");
  if (catalog.counts.migrated_characters !== catalog.characters.filter((entry) => entry.migration_status === "ported").length) throw new Error("catalog migrated character count mismatch");
  if (catalog.counts.migrated_skills !== catalog.skills.filter((entry) => entry.migration_status === "ported").length) throw new Error("catalog migrated skill count mismatch");
  assertUniqueDefIds("character", catalog.characters);
  assertUniqueDefIds("skill", catalog.skills);
  const migrated = await portedDefinitions();
  const metadataOnly = await metadataOnlyLegacySkills(catalog.packages);
  const authoritativeSurface = await authoritativeSurfaceLegacySkills(catalog.packages);
  const skillsById = new Map(catalog.skills.map((skill) => [skill.id, skill]));
  for (const character of catalog.characters) {
    const ported = portedCharacterDefId(migrated, character.id);
    const expected = ported ?? stableDefId("character", character.id);
    if (character.def_id !== expected) throw new Error(`${character.id}: character def id mismatch`);
    const publishedSkills = portedCharacterSkills(migrated, character.id);
    const resolved = character.resolved_skills.map((id) => skillsById.get(id));
    const complete = publishedSkills !== undefined &&
      resolved.length === character.skills.length &&
      resolved.every((skill) => skill?.migration_status === "ported") &&
      publishedSkills.length === resolved.length &&
      publishedSkills.every((defId, index) => defId === resolved[index].def_id);
    const status = complete ? "ported" : "source_synced";
    if (character.migration_status !== status) {
      throw new Error(`${character.id}: character migration status mismatch`);
    }
  }
  for (const skill of catalog.skills) {
    const ported = migrated.skills.get(skill.id);
    const coveredBy = foldedIntoParentSkills.get(skill.id);
    const expected = ported ?? stableDefId("skill", skill.id);
    if (skill.def_id !== expected) throw new Error(`${skill.id}: skill def id mismatch`);
    const behaviorfulPlaceholder =
      ported !== undefined &&
      migrated.behaviorlessSkills.has(skill.id) &&
      !reviewedClientOnlySkills.has(skill.id) &&
      !reviewedCompositeSkillContainers.has(skill.id) &&
      !foldedIntoParentSkills.has(skill.id) &&
      (profileHasRuleSurface(skill.profile) || !metadataOnly.has(skill.id)) &&
      authoritativeSurface.has(skill.id) &&
      await isEmptyBodySkill(skill.pack, skill.name);
    const status = behaviorfulPlaceholder || (ported === undefined && coveredBy === undefined)
      ? "source_synced"
      : "ported";
    if (skill.migration_status !== status) throw new Error(`${skill.id}: migration status mismatch`);
    if (skill.covered_by !== coveredBy) throw new Error(`${skill.id}: folded parent mismatch`);
    if (
      ported !== undefined &&
      migrated.behaviorlessSkills.has(skill.id) &&
      !reviewedClientOnlySkills.has(skill.id) &&
      !reviewedCompositeSkillContainers.has(skill.id) &&
      !foldedIntoParentSkills.has(skill.id) &&
      (profileHasRuleSurface(skill.profile) || !metadataOnly.has(skill.id)) &&
      authoritativeSurface.has(skill.id) &&
      !behaviorfulPlaceholder
    ) {
      throw new Error(`${skill.id}: behaviorful legacy skill cannot be ported as an empty DSL skill`);
    }
  }
  for (const [skillId, parentId] of foldedIntoParentSkills) {
    const skill = skillsById.get(skillId);
    const parent = skillsById.get(parentId);
    if (!skill) throw new Error(`${skillId}: folded legacy skill missing from catalog`);
    if (!parent) throw new Error(`${skillId}: folded parent ${parentId} missing from catalog`);
    if (parent.migration_status !== "ported") {
      throw new Error(`${skillId}: folded parent ${parentId} is not ported`);
    }
    if (skill.referenced_by.length) {
      throw new Error(`${skillId}: folded helper is directly referenced by a character`);
    }
  }
  if (catalog.counts.static_skins !== catalog.skins.static.length) throw new Error("static skin count mismatch");
  if (catalog.counts.dynamic_skins !== catalog.skins.dynamic.length) throw new Error("dynamic skin count mismatch");
  if (catalog.counts.substitutes !== catalog.skins.substitutes.length) {
    throw new Error("character substitute count mismatch");
  }
  if (catalog.counts.skin_aliases !== catalog.skins.aliases.length) {
    throw new Error("skin alias count mismatch");
  }
  if (catalog.counts.dynamic_skin_aliases !== catalog.skins.dynamic_aliases.length) {
    throw new Error("dynamic skin alias count mismatch");
  }
  const [staticSkins, substitutes, aliases, dynamicCatalog] = await Promise.all([
    staticSkinCatalog(),
    coreCharacterSubstitutes(packNames),
    staticSkinAliases(),
    dynamicSkinCatalog(),
  ]);
  assertJsonEqual("static skin index", catalog.skins.static, staticSkins);
  assertJsonEqual("character substitute index", catalog.skins.substitutes, substitutes);
  assertJsonEqual("skin alias index", catalog.skins.aliases, aliases);
  assertJsonEqual("dynamic skin index", catalog.skins.dynamic, dynamicCatalog.skins);
  assertJsonEqual("dynamic skin alias index", catalog.skins.dynamic_aliases, dynamicCatalog.aliases);
  if (!skinSourceRoot) {
    for (const [name, rows] of [
      ["static skins", catalog.skins.static],
      ["dynamic skins", catalog.skins.dynamic],
      ["skin aliases", catalog.skins.aliases],
      ["dynamic skin aliases", catalog.skins.dynamic_aliases],
    ]) {
      if (rows.length) throw new Error(`${name}: present without --skin-source`);
    }
    if (manifest.trees.some((tree) => tree.source_category === "skin") ||
      manifest.assets.some((asset) => asset.source_category === "skin")) {
      throw new Error("skin manifest entries present without --skin-source");
    }
  }
  const expectedTrees = results.map(
    ({ name, sourceCategory, files, bytes, hardlinked, copied }) => ({
      name,
      source_category: sourceCategory,
      files,
      bytes,
      hardlinked,
      copied,
    }),
  );
  assertJsonEqual("asset manifest tree summaries", manifest.trees, expectedTrees);
  const expectedAssetEntries = await assetEntries();
  assertJsonEqual("asset manifest entries", manifest.assets, expectedAssetEntries);
  const expectedAssets = results
    .filter((tree) => tree.name !== "legacy-source")
    .reduce((sum, tree) => sum + tree.files, 0);
  if (manifest.assets.length !== expectedAssets) throw new Error("asset manifest count mismatch");
  const webCharacters = JSON.parse(
    await readFile(path.join(repoRoot, "apps/web-client/src/generated/standard-characters.json"), "utf8"),
  );
  const standardCharacters = JSON.parse(
    await readFile(path.join(repoRoot, "content/standard/characters.json"), "utf8"),
  );
  if (webCharacters.length !== standardCharacters.length) {
    throw new Error("web standard character catalog count mismatch");
  }
  for (const [index, character] of webCharacters.entries()) {
    const expected = standardCharacters[index];
    if (character.id !== expected.id || character.name !== expected.name_key) {
      throw new Error(`web standard character mismatch at index ${index}`);
    }
    await stat(path.join(repoRoot, "apps/web-client/public", character.portrait));
  }
}

async function sha256(file) {
  const hash = createHash("sha256");
  for await (const chunk of createReadStream(file)) hash.update(chunk);
  return hash.digest("hex");
}

async function requireDirectory(directory) {
  const info = await lstat(directory).catch(() => null);
  if (!info?.isDirectory()) throw new Error(`directory not found: ${directory}`);
}

async function fileExists(file) {
  return Boolean(await lstat(file).catch(() => null));
}

function pathFlag(name) {
  const indexes = [];
  for (let index = 2; index < process.argv.length; index++) {
    if (process.argv[index] === name) indexes.push(index);
  }
  if (indexes.length > 1) throw new Error(`${name} may only be specified once`);
  if (!indexes.length) return null;
  const value = process.argv[indexes[0] + 1];
  if (!value || value.startsWith("--")) throw new Error(`${name} requires a path`);
  return path.resolve(value);
}

function validateCliArgs(args) {
  const pathOptions = new Set(["--source", "--core-source", "--skin-source"]);
  const switches = new Set(["--verify-only", "--ast-self-check"]);
  for (let index = 0; index < args.length; index++) {
    const arg = args[index];
    if (pathOptions.has(arg)) {
      index++;
      if (index >= args.length || args[index].startsWith("--")) {
        throw new Error(`${arg} requires a path`);
      }
    } else if (!switches.has(arg)) {
      throw new Error(`unknown option ${arg}`);
    }
  }
  for (const flag of switches) {
    if (args.filter((arg) => arg === flag).length > 1) {
      throw new Error(`${flag} may only be specified once`);
    }
  }
}

async function resolveContentRoot(input) {
  const root = path.resolve(input);
  if (
    (await fileExists(path.join(root, "character"))) ||
    (await fileExists(path.join(root, "extension")))
  ) {
    return root;
  }
  return path.join(root, "apps/core");
}

async function writeJson(file, value) {
  await writeFile(file, `${JSON.stringify(value, null, 2)}\n`);
}

async function runAstSelfCheck(packs) {
  const [substitutes, aliases, dynamicCatalog] = await Promise.all([
    coreCharacterSubstitutes(packs),
    staticSkinAliases(),
    dynamicSkinCatalog(),
  ]);
  if (!substitutes.length) throw new Error("AST self-check found no core character substitutes");
  if (skinSourceRoot) {
    const shareFile = path.join(skinSourceRoot, "extension/千幻聆音/skinShare.js");
    const dynamicFile = path.join(
      skinSourceRoot,
      "extension/十周年UI/src/skins/dynamicSkin.js",
    );
    if (
      (treeAvailable("static-skins") || treeAvailable("static-skins-decade")) &&
      await fileExists(shareFile) &&
      !aliases.length
    ) {
      throw new Error("AST self-check found no qhly_skinShare aliases");
    }
    if (
      treeAvailable("dynamic-skins") &&
      await fileExists(dynamicFile) &&
      !dynamicCatalog.skins.length
    ) {
      throw new Error("AST self-check found no dynamicSkinConfig entries");
    }
  }
  const result = {
    core_source: sourceLabel(),
    skin_source: skinSourceRoot ? sourceLabel(skinSourceRoot) : null,
    substitutes: substitutes.length,
    aliases: aliases.length,
    dynamic_skins: dynamicCatalog.skins.length,
    dynamic_aliases: dynamicCatalog.aliases.length,
  };
  assertDataOnly(result);
  assertDataOnly(substitutes);
  assertDataOnly(aliases);
  assertDataOnly(dynamicCatalog);
  console.log(JSON.stringify(result, null, 2));
}

function assertDataOnly(value, location = "$", seen = new Set()) {
  if (value === null || ["string", "boolean"].includes(typeof value)) return;
  if (typeof value === "number" && Number.isFinite(value)) return;
  if (typeof value !== "object") throw new Error(`${location}: non-data AST output`);
  if (seen.has(value)) throw new Error(`${location}: cyclic AST output`);
  seen.add(value);
  if (Array.isArray(value)) {
    value.forEach((item, index) => assertDataOnly(item, `${location}[${index}]`, seen));
  } else {
    for (const [key, child] of Object.entries(value)) {
      assertDataOnly(child, `${location}.${key}`, seen);
    }
  }
  seen.delete(value);
}

function assertJsonEqual(label, actual, expected) {
  if (JSON.stringify(actual) !== JSON.stringify(expected)) {
    throw new Error(`${label} mismatch`);
  }
}

async function run(command, args) {
  await new Promise((resolve, reject) => {
    const child = spawn(command, args, { cwd: repoRoot, stdio: "inherit" });
    child.once("error", reject);
    child.once("exit", (code) => code === 0 ? resolve() : reject(new Error(`${command} exited ${code}`)));
  });
}

async function runCapture(command, args) {
  return await new Promise((resolve, reject) => {
    const stdout = [];
    const stderr = [];
    const child = spawn(command, args, { cwd: repoRoot, stdio: ["ignore", "pipe", "pipe"] });
    child.stdout.on("data", (chunk) => stdout.push(chunk));
    child.stderr.on("data", (chunk) => stderr.push(chunk));
    child.once("error", reject);
    child.once("close", (code) => {
      if (code === 0) resolve(Buffer.concat(stdout).toString("utf8"));
      else reject(new Error(
        `${command} exited ${code}: ${Buffer.concat(stderr).toString("utf8").trim()}`,
      ));
    });
  });
}

function sourceLabel(root = coreSourceRoot) {
  return path.relative(path.dirname(repoRoot), root).split(path.sep).join("/");
}

function printResults(action, results) {
  console.log(`original content ${action}:`);
  for (const result of results) {
    console.log(
      `  ${result.name}: ${result.files} files, ${formatBytes(result.bytes)}, ` +
      `${result.hardlinked} linked, ${result.copied} copied`,
    );
  }
}

function formatBytes(bytes) {
  const units = ["B", "KiB", "MiB", "GiB"];
  let value = bytes;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit++;
  }
  return `${value.toFixed(unit ? 1 : 0)} ${units[unit]}`;
}
