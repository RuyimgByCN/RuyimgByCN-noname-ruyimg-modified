import type { PublicPresentationBatch } from "../types/presentation";

function check(ok: unknown): asserts ok { if (!ok) throw new Error("native: public presentation metadata invalid"); }
function object(v: unknown, keys: string[]): Record<string, any> {
  check(v && typeof v === "object" && !Array.isArray(v));
  const o = v as Record<string, any>;
  check(Object.keys(o).length === keys.length && keys.every(k => Object.hasOwn(o, k)));
  return o;
}
function integer(v: unknown, max = 0xffffffff, min = 0): v is number { return typeof v === "number" && Number.isSafeInteger(v) && v >= min && v <= max; }
function token(v: unknown): v is string { return typeof v === "string" && v.length > 0 && new TextEncoder().encode(v).length <= 256 && v !== "." && v !== ".." && !/[\\/%?#:\u0000-\u001f\u007f-\u009f]/u.test(v); }
function path(v: unknown): v is string { return typeof v === "string" && new TextEncoder().encode(v).length <= 256 && v.split("/").every(token); }
function hex(v: unknown, n = 64): boolean { return typeof v === "string" && v.length === n && /^[a-f0-9]+$/.test(v); }
// Exact shared policy: C0/C1 controls, Unicode White_Space, BOM, and backslash.
function repository(v: unknown): boolean { return typeof v === "string" && v.length > 0 && new TextEncoder().encode(v).length <= 256 && !/[\u0000-\u0020\u007f-\u00a0\u1680\u2000-\u200a\u2028-\u2029\u202f\u205f\u3000\ufeff\\]/u.test(v); }
function owner(v: unknown): void {
  const kind = (v as any)?.kind;
  if (kind === "package") {
    const o = object(v, ["kind", "namespace", "package_id", "version", "logic_hash", "content_hash"]);
    check(token(o.namespace) && token(o.package_id) && token(o.version) && hex(o.logic_hash) && hex(o.content_hash));
  } else {
    const o = object(v, ["kind", "repository", "adopted_revision", "catalog_sha256", "asset_manifest_sha256"]);
    check(kind === "original" && repository(o.repository) && hex(o.adopted_revision, 40) && hex(o.catalog_sha256) && hex(o.asset_manifest_sha256));
  }
}
function definition(v: unknown, kind: string): void {
  const o = object(v, ["owner", "kind", "local_id"]); owner(o.owner); check(o.kind === kind);
  if (o.owner.kind === "package") check(integer(o.local_id, 0xffffffff, kind === "card" ? 1 : 0));
  else { const id = object(o.local_id, ["pack", "raw_symbol"]); check(token(id.pack) && token(id.raw_symbol)); }
}
function asset(v: unknown): void {
  const o = object(v, ["source", "reference", "digest_algorithm", "digest", "size"]); owner(o.source);
  check(hex(o.digest) && integer(o.size, Number.MAX_SAFE_INTEGER, 1));
  if (o.source.kind === "package") {
    const ref = object(o.reference, ["kind", "namespace", "package_id", "asset_id"]);
    check(ref.kind === "package" && ref.namespace === o.source.namespace && ref.package_id === o.source.package_id && token(ref.asset_id) && o.digest_algorithm === "blake3");
  } else {
    const ref = object(o.reference, ["kind", "path", "source_category", "source_path"]);
    check(ref.kind === "original" && path(ref.path) && (ref.path.startsWith("audio-declared/") || ref.path.startsWith("audio/") || ref.path.startsWith("audio-compat/")) && path(ref.source_path) && ref.source_path.startsWith("audio/") && ["core", "compat"].includes(ref.source_category) && o.digest_algorithm === "sha256");
  }
}
function voice(v: unknown): void {
  if (v === null) return;
  const o = object(v, ["role", "form", "assets"]); definition(o.role, "general");
  check(o.form === null || token(o.form));
  check(Array.isArray(o.assets) && o.assets.length > 0 && o.assets.length <= 1024);
  o.assets.forEach(asset);
  const keys = o.assets.map((a: any) => JSON.stringify([
    a.source.kind, a.source.namespace, a.source.package_id, a.source.version, a.source.logic_hash, a.source.content_hash,
    a.source.repository, a.source.adopted_revision, a.source.catalog_sha256, a.source.asset_manifest_sha256,
    a.reference.kind, a.reference.namespace, a.reference.package_id, a.reference.asset_id, a.reference.path,
    a.reference.source_category, a.reference.source_path, a.digest_algorithm, a.digest, a.size,
  ]));
  check(new Set(keys).size === o.assets.length);
}
const NATURES = ["normal", "fire", "thunder"];
function effective(v: unknown): void { const o = object(v, ["kind", "key", "nature"]); check(CARD_KINDS.includes(o.kind) && token(o.key) && NATURES.includes(o.nature)); }
function event(v: unknown, players: number): void {
  const kind = (v as any)?.kind;
  const fields: Record<string, string[]> = {
    card_used: ["actor", "targets", "effective", "voice"], card_responded: ["actor", "effective", "voice"],
    skill_activated: ["actor", "skill", "targets", "voice"], damage_dealt: ["source", "target", "amount", "nature"],
    damage_prevented: ["source", "target", "nature"], hp_recovered: ["actor", "amount"], player_died: ["actor", "killer", "voice"],
  };
  check(typeof kind === "string" && Object.hasOwn(fields, kind)); const o = object(v, ["kind", ...fields[kind]]);
  for (const key of ["actor", "target"]) if (Object.hasOwn(o, key)) check(integer(o[key], players, 1));
  for (const key of ["source", "killer"]) if (Object.hasOwn(o, key)) check(o[key] === null || integer(o[key], players, 1));
  if (Object.hasOwn(o, "amount")) check(integer(o.amount, 0xffffffff, 1));
  if (Object.hasOwn(o, "nature")) check(NATURES.includes(o.nature));
  if (Object.hasOwn(o, "voice")) voice(o.voice);
  if (Object.hasOwn(o, "effective")) effective(o.effective);
  if (Object.hasOwn(o, "targets") && o.targets !== null) check(Array.isArray(o.targets) && o.targets.length <= 16 && o.targets.every((p: unknown) => integer(p, players, 1)) && new Set(o.targets).size === o.targets.length);
  if (Object.hasOwn(o, "skill")) definition(object(o.skill, ["definition"]).definition, "skill");
}

/** Absent is legacy compatible; present data is validated before any recipient mutation. */
export function presentationMetadata(value: unknown, revision: number, players: number): PublicPresentationBatch | undefined {
  if (value === undefined) return undefined;
  const o = object(value, ["revision", "status", "unavailable", "occurrences"]);
  check(integer(o.revision, Number.MAX_SAFE_INTEGER) && o.revision === revision && ["complete", "incomplete", "overflow"].includes(o.status));
  check(new TextEncoder().encode(JSON.stringify(value)).length <= 65536);
  check(Array.isArray(o.unavailable) && o.unavailable.length <= 28);
  for (const item of o.unavailable) {
    const scope = object(item, ["domain", "reason"]);
    check(["card_use", "card_response", "skill_activation", "damage", "recovery", "death", "voice_identity"].includes(scope.domain));
    check(["source_no_audio_discriminator", "commit_boundary_unproved", "verified_binding_missing", "public_binding_unavailable"].includes(scope.reason));
  }
  check(new Set(o.unavailable.map((v: any) => `${v.domain}:${v.reason}`)).size === o.unavailable.length);
  check(o.status !== "complete" || o.unavailable.length === 0);
  check(Array.isArray(o.occurrences) && o.occurrences.length <= 1024 && (o.status !== "overflow" || o.occurrences.length === 0));
  o.occurrences.forEach((item: unknown, index: number) => { const entry = object(item, ["ordinal", "event"]); check(entry.ordinal === index); event(entry.event, players); });
  return value as PublicPresentationBatch;
}

/** Validate only actual envelope response locations, including nested chooseGeneral. */
export function validatePresentationResponse(value: unknown): void {
  if (!value || typeof value !== "object") return;
  const o = value as Record<string, any>;
  if (Object.hasOwn(o, "presentation")) {
    check(o.presentation !== undefined && o.view && Array.isArray(o.view.seats));
    check(o.revision === undefined || o.revision === o.view.revision);
    presentationMetadata(o.presentation, o.revision ?? o.view.revision, o.view.seats.length);
  }
  if (Object.hasOwn(o, "envelope")) validatePresentationResponse(o.envelope);
}
const CARD_KINDS: string[] = ["sha", "shan", "tao", "horse_plus", "horse_minus", "weapon", "armor", "huosha", "tiesuo", "nanman", "wanjian", "juedou", "wuxie", "lebu", "bingliang", "shandian", "guohe", "shunshou", "jiu", "leisha", "shoulijian", "wuxingpan", "wangmeizhike", "sanjian", "shuigong", "fudichouxin", "chenhuodajie", "suolianjia", "hongshui", "toulianghuanzhu", "linghunzhihuo", "zhaomingdan", "jihuocard", "youdishenru", "geanguanhuo", "caochuanjiejian", "shezhanqunru", "shujinsan", "mianlijinzhen", "feibiao", "jinlianzhu", "tanhuadong", "yougeng", "qingtuan", "yuanbaorou", "gudonggeng", "mapodoufu", "chunbing", "liyutang", "molicha", "xiajiao", "mizhilianou", "luyugeng", "bingpotong", "qiankunbiao", "longxugou", "shenhuofeiya", "ziyangdan", "yunvyuanshen", "shihuifen", "liufengsan", "shatang", "heilonglinpian", "liutouge", "mutoumianju", "dinvxuanshuang", "gjyuheng", "dunpaigedang", "zhiliaobo", "yuansuhuimie", "xingjiegoutong", "chuansongmen", "tanshezhiren", "shijieshu", "yanziyaoshui", "aozuzhilei", "wenyi", "zhihuanjun", "baoxueyaoshui", "qinpendayu", "birinongwu", "ciguhanshuang", "kunenfayin", "shanbengshu", "poxiao", "zumoshoukao", "monkey", "xuelunyang", "mianju", "jiuwei", "wuzhong", "taoyuan", "wugu", "jiedao", "renwang", "qinggang", "xuanyuanjian", "qilin", "hanbing", "zhuge", "guanshi", "qinglong", "zhangba", "cixiong", "fangtian", "hufu", "liuxinghuoyu", "caoyao", "chilongya", "tianxianjiu", "xixueguizhihuan", "guiyoujie", "shentoumianju", "xingjunyan", "qinglonglingzhu", "zhufangshenshi", "fengxueren", "yufulu", "yangpijuan", "daihuofenglun", "xiayuncailing", "langeguaiyi", "longlushanfeng", "haidao", "shuimomuxue", "feixu", "yixialan", "shamolvzhou", "linzhongjianta", "bingheyaosai", "youlin", "lindixiliu", "dizaizhen", "jiguan", "xuejibingbao", "shenmiguo", "fengyinzhidan", "lianyaohu", "qiankundai", "shennongding", "nvwashi", "kunlunjingc", "donghuangzhong", "kongdongyin", "jingleishan", "chiyuxi", "guisheqi", "xianluhui", "guilingzhitao", "sifeizhenmian", "shuchui", "sadengjinhuan", "haotianta", "yuruyi", "yuchanqian", "yuchankun", "yuchanzhen", "yuchanxun", "yuchankan", "yuchanli", "yuchangen", "yuchandui", "shencaojie", "qinglongzhigui", "zhuquezhizhang", "baishouzhihu", "xuanwuzhihuang", "huanglinzhicong", "hslingjian", "tianfuzhen", "qixingzhen", "niaoxiangzhen", "changshezhen", "pozhenjue"];
