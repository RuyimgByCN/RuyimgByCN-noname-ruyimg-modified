import { createHash } from "node:crypto";
import { copyFile, link, mkdir, readFile, readdir, realpath, rm, stat } from "node:fs/promises";
import path from "node:path";
import ts from "../apps/web-client/node_modules/typescript/lib/typescript.js";
import { compareFileNames, listFiles } from "./original-content-order.mjs";
import { parseDefaultObjectFile, parseObjectFile, propertyName, safeLiteral } from "./original-source-ast.mjs";

const audioKeys = new Set(["audio", "audioname", "audioname2", "logAudio", "logAudio2", "inherit", "dieAudios", "tempname"]);
const own = (object, key) => Object.hasOwn(object, key);
const segment = (value) => typeof value === "string" && value && value !== "." && value !== ".." && !/[\/\\%?#:\u0000-\u001f\u007f]/.test(value);

export function assertAudioPath(value) {
  if (typeof value !== "string" || !value || !value.split("/").every((part) =>
    part && part !== "." && part !== ".." && !/[\\%?#:\u0000-\u001f\u007f]/.test(part))) {
    throw new Error(`invalid audio path: ${value}`);
  }
  return value;
}

// Pins the reviewed five-file input, including recipe, tool digest and original source SHAs.
const compatProvenanceSHA256 = "1bd98a3e5d7ab22f6749b132d041f409e95a0dbf2f7ed9a603ec256422cc5313";
const compatSizes = [18716, 114531, 10188, 39091, 81053];

export async function validateAudioCompat(coreRoot, compatRoot, sha256) {
  const roots = await Promise.all([realpath(coreRoot), realpath(compatRoot)]);
  if (roots[1] !== path.join(await realpath(path.dirname(compatRoot)), path.basename(compatRoot))) {
    throw new Error("audio compat input root must be checked in, not a symlink");
  }
  async function boundedFile(root, relative) {
    assertAudioPath(relative);
    const file = await realpath(path.join(root, relative));
    if (!file.startsWith(`${root}${path.sep}`) || !(await stat(file)).isFile()) {
      throw new Error(`audio compat realpath escapes root: ${relative}`);
    }
    return file;
  }
  const provenanceFile = await boundedFile(roots[1], "provenance.json");
  const bytes = await readFile(provenanceFile);
  const provenance = JSON.parse(bytes);
  const bindings = provenance.bindings;
  if (!Array.isArray(bindings) || bindings.length !== 5) throw new Error("invalid audio compat bindings");
  const sources = new Set(), derivatives = new Set();
  for (const [index, row] of bindings.entries()) {
    if (Object.keys(row).sort().join() !== "derivative_path,derivative_sha256,source_path,source_sha256" ||
      !row.source_path.startsWith("audio/") || !row.derivative_path.startsWith("audio/") ||
      !/^[a-f0-9]{64}$/.test(row.source_sha256) || !/^[a-f0-9]{64}$/.test(row.derivative_sha256)) {
      throw new Error("invalid audio compat binding");
    }
    if (sources.has(row.source_path) || derivatives.has(row.derivative_path)) throw new Error("duplicate audio compat binding");
    sources.add(row.source_path); derivatives.add(row.derivative_path);
    const source = await boundedFile(roots[0], row.source_path);
    const derivative = await boundedFile(roots[1], row.derivative_path);
    if (await sha256(source) !== row.source_sha256) throw new Error(`${row.source_path}: audio compat source SHA differs`);
    if ((await stat(derivative)).size !== compatSizes[index] || await sha256(derivative) !== row.derivative_sha256) {
      throw new Error(`${row.derivative_path}: audio compat derivative size/SHA differs`);
    }
  }
  if (createHash("sha256").update(bytes).digest("hex") !== compatProvenanceSHA256) {
    throw new Error("audio compat fixed provenance/recipe/tool binding differs");
  }
  const expected = ["provenance.json", ...derivatives].sort(compareFileNames);
  const actual = [];
  async function visit(directory, prefix = "") {
    for (const entry of await readdir(directory, { withFileTypes: true })) {
      const relative = `${prefix}${entry.name}`;
      if (entry.isDirectory()) await visit(path.join(directory, entry.name), `${relative}/`);
      else if (entry.isFile()) actual.push(relative);
      else throw new Error(`${relative}: audio compat non-regular input`);
    }
  }
  await visit(roots[1]);
  actual.sort(compareFileNames);
  if (JSON.stringify(actual) !== JSON.stringify(expected)) throw new Error("audio compat input file list differs");
  return bindings;
}

export function audioBackingIndex(assets) {
  assertUniqueAssetPaths(assets);
  const backing = new Map();
  for (const asset of assets) {
    if (asset.kind !== "audio" || asset.path.startsWith("audio-declared/")) continue;
    const key = `${asset.source_category}:${asset.source_path}`;
    if (backing.has(key)) throw new Error(`duplicate audio backing: ${key}`);
    const expected = asset.source_category === "core" ? asset.source_path :
      asset.source_category === "compat" ? asset.source_path.replace(/^audio\//, "audio-compat/") : null;
    if (asset.path !== expected || !asset.source_path.startsWith("audio/")) throw new Error("invalid audio backing identity");
    backing.set(key, asset);
  }
  return backing;
}

// The same data-only TS parser is used by catalog generation; source JS is never executed.
export async function loadAudioDeclarations(sourceRoot) {
  const skills = new Map(), characters = new Map(), declarations = [], dynamic = [];
  const packs = (await readdir(path.join(sourceRoot, "character"), { withFileTypes: true }))
    .filter((entry) => entry.isDirectory()).map((entry) => entry.name).sort(compareFileNames);
  function readRecord(property, pack, file, name, character = false) {
    if (!segment(pack) || !segment(name)) throw new Error(`${file}: invalid audio identity ${pack}:${name}`);
    const data = Object.create(null);
    if (!ts.isPropertyAssignment(property) || !ts.isObjectLiteralExpression(property.initializer)) {
      throw new Error(`${file}: unsupported declaration ${name}`);
    }
    for (const field of property.initializer.properties) {
      const key = propertyName(field);
      if (!audioKeys.has(key) && !(character && key === "skills")) continue;
      if (own(data, key)) throw new Error(`${file}: duplicate ${name}.${key}`);
      const node = ts.isPropertyAssignment(field) ? field.initializer : field;
      const parsed = safeLiteral(node);
      const detail = { id: `${pack}:${name}`, field: key, source: file,
        line: node.getSourceFile().getLineAndCharacterOfPosition(node.getStart()).line + 1,
        expression: node.getText(), static: parsed.ok && parsed.omitted.length === 0 };
      declarations.push(detail);
      if (detail.static) data[key] = parsed.value;
      else {
        data[key] = undefined;
        const parameters = new Set(), references = new Set();
        function visit(current) {
          if (ts.isFunctionLike(current)) current.parameters.forEach((parameter) => parameters.add(parameter.getText()));
          if (ts.isPropertyAccessExpression(current) || ts.isElementAccessExpression(current)) references.add(current.getText());
          ts.forEachChild(current, visit);
        }
        visit(node);
        dynamic.push({ ...detail, parameters: [...parameters], references: [...references] });
      }
    }
    return { id: `${pack}:${name}`, pack, name, data, node: property.initializer, source: file };
  }
  function insertSkill(property, pack, file, name, subskill = false, emit = true) {
    const record = { ...readRecord(property, pack, file, name), subskill, emit };
    const existing = skills.get(record.id);
    if (existing && (existing.subskill === subskill || JSON.stringify(existing.data) !== JSON.stringify(record.data))) {
      throw new Error(`conflicting audio skill declaration: ${record.id}`);
    }
    // Upstream can publish both a top-level helper and an equivalent finishSkill subskill.
    if (!existing) skills.set(record.id, record);
    if (subskill) return;
    for (const field of record.node.properties) {
      if (propertyName(field) !== "subSkill" || !ts.isPropertyAssignment(field) || !ts.isObjectLiteralExpression(field.initializer)) continue;
      for (const child of field.initializer.properties) insertSkill(child, pack, file, `${name}_${propertyName(child)}`, true, emit);
    }
  }
  for (const pack of packs) {
    const base = path.join(sourceRoot, "character", pack);
    const files = await readdir(base, { withFileTypes: true });
    let skillFiles = [];
    if (files.some((entry) => entry.name === "skill.js")) skillFiles = ["skill.js"];
    else if (files.some((entry) => entry.name === "skill" && entry.isDirectory())) {
      skillFiles = (await readdir(path.join(base, "skill"))).filter((name) => name.endsWith(".js") && name !== "index.js")
        .sort(compareFileNames).map((name) => `skill/${name}`);
    }
    for (const file of skillFiles) {
      const relative = `character/${pack}/${file}`;
      const parsed = await parseObjectFile(path.join(sourceRoot, relative), "skills");
      for (const property of parsed.properties) insertSkill(property, pack, relative, propertyName(property));
    }
    if (files.some((entry) => entry.name === "character.js")) {
      const relative = `character/${pack}/character.js`;
      const parsed = await parseObjectFile(path.join(sourceRoot, relative), "characters");
      for (const property of parsed.properties) {
        const record = readRecord(property, pack, relative, propertyName(property), true);
        if (characters.has(record.id)) throw new Error(`conflicting audio character declaration: ${record.id}`);
        characters.set(record.id, record);
      }
    }
  }
  // Resolve character-declared references to static core/card/mode skill definitions too.
  const supportFiles = ["noname/library/skill.js"];
  for (const dir of ["card", "mode"]) {
    const entries = await readdir(path.join(sourceRoot, dir), { withFileTypes: true }).catch((error) => {
      if (error.code !== "ENOENT") throw error;
      return [];
    });
    supportFiles.push(...entries.filter((entry) => entry.isFile() && entry.name.endsWith(".js")).map((entry) => `${dir}/${entry.name}`));
  }
  const modeSkills = path.join(sourceRoot, "mode/guozhan/src/skill");
  const modules = await listFiles(modeSkills).catch((error) => {
    if (error.code !== "ENOENT") throw error;
    return [];
  });
  supportFiles.push(...modules.filter((file) => file.endsWith(".js") && path.basename(file) !== "index.js").map((file) => `mode/guozhan/src/skill/${file.split(path.sep).join("/")}`));
  for (const file of supportFiles.sort(compareFileNames)) {
    const parsed = await parseDefaultObjectFile(path.join(sourceRoot, file)).catch((error) => {
      if (error.code !== "ENOENT") throw error;
      return null;
    });
    if (!parsed) continue;
    const rootSkill = file === "noname/library/skill.js" || file.startsWith("mode/guozhan/src/skill/");
    const container = rootSkill ? parsed : parsed.properties.find((property) => propertyName(property) === "skill")?.initializer;
    if (!container || (!rootSkill && !ts.isObjectLiteralExpression(container))) continue;
    const pack = `support-${file.replace(/\.js$/, "").replaceAll("/", "-")}`;
    for (const property of container.properties) {
      if (ts.isPropertyAssignment(property) && ts.isObjectLiteralExpression(property.initializer)) insertSkill(property, pack, file, propertyName(property), false, false);
    }
  }
  return { skills, characters, declarations, dynamic };
}

export function resolveAudioDeclarations({ skills, characters, dynamic = [], declarations = [] }, substitutes = []) {
  const issues = [], mappings = [];
  const namesByRecords = new Map([skills, characters].map((records) => {
    const names = new Map();
    for (const record of records.values()) names.set(record.name, [...(names.get(record.name) ?? []), record]);
    return [records, names];
  }));
  const byName = (records, name, pack) => {
    if (!segment(name)) throw new Error(`invalid audio reference: ${name}`);
    const local = records.get(`${pack}:${name}`);
    if (local) return local;
    const candidates = namesByRecords.get(records).get(name) ?? [];
    const main = candidates.filter((candidate) => candidate.emit !== false);
    const matches = main.length ? main : candidates;
    const equivalentLeaves = matches.every((candidate) => candidate.emit === false &&
      JSON.stringify(candidate.data) === JSON.stringify(matches[0].data) && !candidate.data.inherit &&
      references(candidate.data.audio).length === 0 && Object.values(candidate.data.audioname2 ?? {}).every((value) => references(value).length === 0));
    if (matches.length > 1 && !equivalentLeaves) throw new Error(`ambiguous audio reference: ${name}`);
    return matches[0];
  };
  const effective = (record, history = []) => {
    if (history.includes(record.id)) throw new Error(`audio inheritance cycle: ${[...history, record.id].join(" -> ")}`);
    if (!record.data.inherit) return record.data;
    const parent = byName(skills, record.data.inherit, record.pack);
    if (!parent) {
      issues.push({ id: record.id, reason: "unknown inherit", reference: record.data.inherit, declarations: declarations.filter((item) => item.id === record.id && item.field === "inherit") });
      return record.data;
    }
    const data = { ...effective(parent, [...history, record.id]) };
    // Upstream finishSkill copies missing/null properties, including inherited numeric audio.
    for (const [key, value] of Object.entries(record.data)) if (value != null) data[key] = value;
    return data;
  };
  const references = (info) => {
    if (Array.isArray(info)) {
      if (info.length === 2 && typeof info[0] === "string" && typeof info[1] === "number") return [info[0]];
      return info.flatMap(references);
    }
    return typeof info === "string" && !/[\/.:]/.test(info) && !/^(?:true|false|\d+)$/.test(info) ? [info] : [];
  };
  const variants = (record, history = []) => {
    if (history.includes(record.id)) throw new Error(`audio reference cycle: ${[...history, record.id].join(" -> ")}`);
    const data = effective(record);
    const names = [...(data.audioname ?? []), ...Object.keys(data.audioname2 ?? {})];
    if (!names.every(segment)) throw new Error(`${record.id}: invalid audioname`);
    for (const ref of references(data.audio).concat(...Object.values(data.audioname2 ?? {}).map(references))) {
      const target = byName(skills, ref, record.pack);
      if (target && target.id !== record.id) names.push(...variants(target, [...history, record.id]));
    }
    return [...new Set(names)];
  };
  function parse(info, { record, type, player = "", audionames = [], history = [] }) {
    const context = { record, type, player, audionames, history };
    const defaultPath = `${type}/`;
    const reference = (name, override) => {
      const records = type === "skill" ? skills : characters;
      const target = byName(records, name, record.pack);
      if (!target && override === undefined && type === "skill") {
        issues.push({ id: record.id, reason: "unknown reference", reference: name, declarations: declarations.filter((item) => item.id === record.id && ["audio", "audioname2"].includes(item.field)) });
        return [];
      }
      if (name === record.name) {
        // Upstream checkHistory uses this type's default for an immediate self reference.
        issues.push({ id: record.id, reason: type === "skill" ? "empty_by_upstream_self_reference" : "default_by_upstream_self_reference",
          reference: name, declaration: declarations.find((item) => item.id === record.id && item.field === (type === "skill" ? "audio" : "dieAudios")) });
        return type === "skill" ? [] : parse(true, context);
      }
      if (history.includes(`${type}:${name}`)) throw new Error(`audio reference cycle: ${[...history, name].join(" -> ")}`);
      return resolve(target ?? { id: name, pack: record.pack, name, data: {} }, type, player, audionames, history, override);
    };
    if (Array.isArray(info)) {
      if (type === "skill" && info.length === 2 && typeof info[0] === "string" && typeof info[1] === "number") {
        const [name, count] = info;
        if (!Number.isSafeInteger(count) || count < 0) throw new Error(`${record.id}: invalid audio count`);
        return byName(skills, name, record.pack) ? reference(name).slice(0, count) : reference(name, count);
      }
      return [...new Set(info.flatMap((item) => parse(item, context)))];
    }
    if (info == null || info === false || info === "false") return [];
    if (!["string", "number", "boolean"].includes(typeof info)) throw new Error(`${record.id}: invalid audio info`);
    if (typeof info === "number" && (!Number.isSafeInteger(info) || info < 0)) throw new Error(`${record.id}: invalid audio count`);
    const value = String(info);
    const list = /^(?:(.*):)?(true|\d+)(?::([^:]+))?$/.exec(value);
    if (list) {
      const dir = list[1] === undefined ? defaultPath : `${list[1]}/`;
      const ext = list[3] === undefined ? ".mp3" : `.${list[3]}`;
      const audioName = (Array.isArray(player) ? player : [player]).find((name) => audionames.includes(name));
      const name = record.name + (type === "skill" && audioName ? `_${audioName}` : "");
      if (list[2] === "true") return [assertAudioPath(`audio/${dir}${name}${ext}`)];
      const count = Number(list[2]);
      if (!Number.isSafeInteger(count) || count > 10000) throw new Error(`${record.id}: invalid audio count`);
      return Array.from({ length: count }, (_, index) => assertAudioPath(`audio/${dir}${name}${index + 1}${ext}`));
    }
    if (/[\\%?#:\u0000-\u001f\u007f]/.test(value)) throw new Error(`invalid audio path: ${value}`);
    const slash = value.lastIndexOf("/"), dot = value.lastIndexOf(".");
    if (slash === -1 && dot === -1) return reference(value);
    return [assertAudioPath(`audio/${slash === -1 ? defaultPath : ""}${value}${dot > slash ? "" : ".mp3"}`)];
  }
  function resolve(record, type, player = "", audionames = [], history = [], override) {
    const key = `${type}:${record.name}`;
    if (history.includes(key)) throw new Error(`audio reference cycle: ${[...history, key].join(" -> ")}`);
    const data = type === "skill" ? effective(record) : record.data;
    const names = [...new Set([...audionames, ...(data.audioname ?? [])])];
    let info = override;
    if (info === undefined) {
      if (type === "die") info = data.dieAudios == null || (Array.isArray(data.dieAudios) && !data.dieAudios.length) ? true : data.dieAudios;
      else {
        const name = (Array.isArray(player) ? player : [player]).find((name) => data.audioname2?.[name]);
        info = name ? data.audioname2[name] : data.audio ?? false;
      }
    }
    return parse(info, { record, type, player, audionames: names, history: [...history, key] });
  }
  for (const record of [...skills.values()].filter((record) => record.emit !== false).sort((a, b) => compareFileNames(a.id, b.id))) {
    for (const variant of ["", ...variants(record)]) {
      const paths = resolve(record, "skill", variant);
      paths.forEach((source_path, index) => mappings.push({ id: record.id, type: "skill", variant,
        path: `audio-declared/${record.pack}/skill/${record.name}/${variant ? `name-${variant}` : "base"}/${String(index + 1).padStart(4, "0")}/${path.posix.basename(source_path)}`, source_path }));
    }
  }
  for (const record of [...characters.values()].sort((a, b) => compareFileNames(a.id, b.id))) {
    resolve(record, "die").forEach((source_path, index) => mappings.push({ id: record.id, type: "die", variant: "",
      path: `audio-declared/${record.pack}/die/${record.name}/${String(index + 1).padStart(4, "0")}/${path.posix.basename(source_path)}`, source_path }));
  }
  // Substitute declarations are keyed by global character name, not their declaring pack.
  for (const substitute of substitutes) {
    const owners = namesByRecords.get(characters).get(substitute.character) ?? [];
    if (owners.length !== 1) throw new Error(`${substitute.character}: ${owners.length ? "ambiguous" : "unknown"} substitute owner`);
  }
  // Character previews use real actor inputs, never the skill's entire unrelated voice pool.
  for (const character of [...characters.values()].sort((a, b) => compareFileNames(a.id, b.id))) {
    if (["skills", "tempname"].some((key) => own(character.data, key) && character.data[key] === undefined)) {
      issues.push({ id: character.id, reason: "dynamic character audio inputs", declarations: declarations.filter((item) => item.id === character.id && ["skills", "tempname"].includes(item.field)) });
      continue;
    }
    const skillNames = character.data.skills ?? [];
    const tempnames = character.data.tempname ?? [];
    if (!Array.isArray(skillNames) || !skillNames.every(segment) || !Array.isArray(tempnames) || !tempnames.every(segment)) {
      throw new Error(`${character.id}: invalid character audio inputs`);
    }
    const forms = [{ name: "", playerNames: [character.name, ...tempnames] }];
    for (const substitute of substitutes.filter((row) => row.character === character.name)) {
      if (!segment(substitute.substitute) || !Array.isArray(substitute.directives) || !substitute.directives.every((value) => typeof value === "string")) {
        throw new Error(`${character.id}: invalid character substitute`);
      }
      let skinTempnames = [], dieAudios = [];
      for (const directive of substitute.directives) {
        if (directive.startsWith("tempname:")) skinTempnames = directive.slice(9).split(":");
        if (directive.startsWith("die:")) dieAudios.push(directive.slice(4));
        if (directive.startsWith("die_audio:")) dieAudios.push(...directive.slice(10).split(":"));
      }
      if (!skinTempnames.every(segment)) throw new Error(`${character.id}: invalid substitute tempname`);
      // changeSkin adds the form to player.tempname; explicit skin tempname overrides it.
      forms.push({ name: substitute.substitute,
        playerNames: [...(skinTempnames.length ? skinTempnames : [substitute.substitute]), character.name, ...tempnames], dieAudios });
    }
    for (const form of forms) {
      const variant = form.name ? `form-${form.name}` : "base";
      for (const name of skillNames.filter((name) => name !== "dualside")) {
        const skill = byName(skills, name, character.pack);
        if (!skill) {
          issues.push({ id: character.id, reason: "unknown character skill", reference: name,
            declarations: declarations.filter((item) => item.id === character.id && item.field === "skills") });
          continue;
        }
        resolve(skill, "skill", form.playerNames).forEach((source_path, index) => mappings.push({
          id: character.id, type: "character-skill", skill_id: skill.id, variant: form.name, player_names: form.playerNames,
          path: `audio-declared/${character.pack}/character/${character.name}/skill/${name}/${variant}/${String(index + 1).padStart(4, "0")}/${path.posix.basename(source_path)}`, source_path,
        }));
      }
      if (form.name) {
        const skin = { id: `${character.pack}:${form.name}`, pack: character.pack, name: form.name, data: { dieAudios: form.dieAudios } };
        resolve(skin, "die").forEach((source_path, index) => mappings.push({
          id: character.id, type: "character-die", variant: form.name,
          path: `audio-declared/${character.pack}/character/${character.name}/die/${variant}/${String(index + 1).padStart(4, "0")}/${path.posix.basename(source_path)}`, source_path,
        }));
      }
    }
  }
  assertUniqueAssetPaths(mappings);
  return { mappings, issues, dynamic, declarations };
}

export function declaredAudioAssets(result, directAssets, bindings = []) {
  const backing = audioBackingIndex(directAssets);
  const bySource = new Map();
  for (const row of bindings) {
    if (bySource.has(row.source_path)) throw new Error("duplicate audio compat binding");
    bySource.set(row.source_path, row);
  }
  const assets = [], missing = [];
  for (const mapping of result.mappings) {
    assertAudioPath(mapping.path);
    let source = backing.get(`core:${mapping.source_path}`);
    const binding = bySource.get(mapping.source_path);
    if (binding) {
      if (!source || source.sha256 !== binding.source_sha256) throw new Error("audio compat original binding differs");
      source = backing.get(`compat:${binding.derivative_path}`);
      if (!source || source.sha256 !== binding.derivative_sha256) throw new Error("audio compat derivative binding differs");
    }
    if (!source) { missing.push(mapping); continue; }
    assets.push({ ...source, path: `${path.posix.dirname(mapping.path)}/${path.posix.basename(source.path)}` });
  }
  assertUniqueAssetPaths(assets);
  return { ...result, assets, missing };
}

export async function syncDeclaredAudio(assetRoot, assets, directAssets, verifyOnly, sha256) {
  assertUniqueAssetPaths(assets);
  const backing = audioBackingIndex(directAssets);
  for (const asset of assets) {
    assertAudioPath(asset.path);
    assertAudioPath(asset.source_path);
    const source = backing.get(`${asset.source_category}:${asset.source_path}`);
    if (!source || source.sha256 !== asset.sha256 || source.size !== asset.size) throw new Error("declared audio backing differs");
    if (!asset.path.startsWith("audio-declared/") || !asset.source_path.startsWith("audio/")) throw new Error("invalid declared audio namespace");
  }
  const targetRoot = path.join(assetRoot, "audio-declared");
  if (verifyOnly) {
    const actual = await readdir(assetRoot);
    const files = actual.includes("audio-declared") ? await listFiles(targetRoot) : [];
    if (JSON.stringify(files.map((file) => `audio-declared/${file.split(path.sep).join("/")}`)) !==
      JSON.stringify(assets.map((asset) => asset.path).sort(compareFileNames))) {
      throw new Error("declared audio source/target file lists differ");
    }
  } else {
    await rm(targetRoot, { recursive: true, force: true });
  }
  for (const asset of assets) {
    const source = path.join(assetRoot, assertAudioPath(backing.get(`${asset.source_category}:${asset.source_path}`).path));
    const target = path.join(assetRoot, assertAudioPath(asset.path));
    if (!verifyOnly) {
      await mkdir(path.dirname(target), { recursive: true });
      try { await link(source, target); }
      catch (error) {
        if (!["EXDEV", "EPERM", "EACCES", "EMLINK"].includes(error.code)) throw error;
        await copyFile(source, target);
      }
    }
    if ((await stat(target)).size !== asset.size || await sha256(target) !== asset.sha256) {
      throw new Error(`${asset.path}: declared audio content differs`);
    }
  }
}

export function assertUniqueAssetPaths(assets) {
  const paths = new Set();
  for (const asset of assets) {
    if (paths.has(asset.path)) throw new Error(`duplicate canonical asset path: ${asset.path}`);
    paths.add(asset.path);
  }
}
