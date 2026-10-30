//! NC07（裁决第 11 项，规则 v327）：[字段审计](../../../../docs/operations/evidence/2026-10-28-nc07-lianyaohu-hash/archive-vs-hash-field-audit.log)
//! 所列 10 个「入档未入摘要」字段的**逐字段反例**（GT-012 口径：只改一个
//! 语义字段，摘要必须区分）。
//!
//! 反例写法：同一引擎的同一快照，仅在恢复前把该字段改为非默认值——两份存档
//! 都合法、可恢复、其余逐字相同；`canonical_hash` 必须给出**不同**摘要。
//! 修复前（v326）这 10 个字段不参与摘要，两摘要相同＝缺陷（先红）。
//! 十个字段全部在规则路径被读取（审计末尾引用计数：目标合法性、摸牌数、
//! 伤害重定向、手牌上限、装备改造），**排除名单为空**；结构守卫见
//! `nc07_archive_hash_guard.rs`。

use engine_native::snap::StateSnapshot;
use engine_native::Engine;

fn base_snapshot() -> StateSnapshot {
    let e = Engine::new(9600, 4);
    e.save()
}

/// 只改一个字段后的恢复摘要；两份快照都必须可恢复（合法值）。
fn hash_after(mutation: impl FnOnce(&mut StateSnapshot)) -> [u8; 32] {
    let mut snap = base_snapshot();
    mutation(&mut snap);
    let restored = Engine::restore(&snap)
        .unwrap_or_else(|e| panic!("single-field mutation must stay restorable: {e:?}"));
    restored.state_hash()
}

fn baseline() -> [u8; 32] {
    hash_after(|_| ())
}

fn assert_distinguishes(name: &str, mutation: impl FnOnce(&mut StateSnapshot)) {
    let base = baseline();
    let mutated = hash_after(mutation);
    assert_ne!(mutated, base, "{name}: 摘要必须区分该字段（入档未入摘要＝GT-012 缺陷）");
}

/// 机关鸢待获得牌（swd 机关鸢：命中后置入、获得时消费）。
#[test]
fn jiguanyuan_cards_enters_hash() {
    assert_distinguishes("jiguanyuan_cards", |s| {
        let card = s.core.deck.last().copied().expect("deck non-empty");
        s.core.player_mut(0).jiguanyuan_cards.push(card);
    });
}

/// 极冷冻（swd 金极冷冻：本回合杀不可被闪避）。
#[test]
fn jileng_active_enters_hash() {
    assert_distinguishes("jileng_active", |s| {
        s.core.player_mut(0).jileng_active = true;
    });
}

/// 嘲讽（swd：判定阶段开始时清除的强制目标旗标）。
#[test]
fn chaofeng_active_enters_hash() {
    assert_distinguishes("chaofeng_active", |s| {
        s.core.player_mut(0).chaofeng_active = true;
    });
}

/// 形体改造层数（swd 形体改造：手牌上限按层削减，判定阶段开始清零）。
#[test]
fn xingtigaizao_count_enters_hash() {
    assert_distinguishes("xingtigaizao_count", |s| {
        s.core.player_mut(0).xingtigaizao_count = 2;
    });
}

/// 隐秘立场（swd：目标合法性判定读取）。
#[test]
fn yinmi_active_enters_hash() {
    assert_distinguishes("yinmi_active", |s| {
        s.core.player_mut(0).yinmi_active = true;
    });
}

/// 雪煌已发动（swd 雪煌：每局一次 limited）。
#[test]
fn xuehuang_used_enters_hash() {
    assert_distinguishes("xuehuang_used", |s| {
        s.core.player_mut(0).xuehuang_used = true;
    });
}

/// 血祭冰瀑剩余次数（swd：摸牌阶段 +1 张，逐次递减）。
#[test]
fn xuejibingbao_enters_hash() {
    assert_distinguishes("xuejibingbao", |s| {
        s.core.player_mut(0).xuejibingbao = 2;
    });
}

/// 拟攻等级（swd：装备命中累计 0–4）。
#[test]
fn nigong_lv_enters_hash() {
    assert_distinguishes("nigong_lv", |s| {
        s.core.player_mut(0).nigong_lv = 2;
    });
}

/// 卡牌实例改造表（ADR 0780：装备名合成/数值升级——目标合法性与攻距读取）。
#[test]
fn card_mods_enters_hash() {
    assert_distinguishes("card_mods", |s| {
        let card = s.core.deck.last().copied().expect("deck non-empty");
        s.core.player_mut(0).card_mods.insert(card, 2);
    });
}

/// 悯婉目标集（回合级：伤害结算收集、回合结束清空——事件分派读取）。
#[test]
fn minwan_targets_enters_hash() {
    assert_distinguishes("minwan_targets", |s| {
        s.core.turn.minwan_targets.push(2);
    });
}

/// v326 及更早存档按版本门禁拒绝（裁决第 11 项）。夹具为 v326 版本号上采集的
/// 普通档（仁德前的出牌阶段静止态，见证据目录 capture-v326）。
#[test]
fn v326_and_earlier_archives_are_rejected_by_rule_version() {
    use engine_native::archive::decode_archive;
    use engine_native::snap::SnapshotError;
    let ev = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/operations/evidence/2026-10-29-nc07-archive-hash-fields");
    let snap = decode_archive(&std::fs::read(ev.join("native-v326.ngsa")).unwrap()).unwrap();
    assert_eq!(snap.ruleset_version, 326);
    assert!(matches!(
        Engine::restore(&snap),
        Err(SnapshotError::RuleVersionMismatch { snapshot: 326, catalog: 330 })
    ));
}
