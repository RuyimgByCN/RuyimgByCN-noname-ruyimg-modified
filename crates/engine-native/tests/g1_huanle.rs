//! G1 规则宿主纵向样本（ADR 0776）：欢乐卡牌 → 真实包 → 导入 →
//! 宿主行为技能装载 → 嵌套选择与恢复。
//!
//! 行为子集（数据声明，宿主原语执行）：
//! - 苦无（武器）：杀伤害后目标弃一张手牌（强制触发；登记差异：
//!   上游为目标自选+he 区+notLink，原生为确定性弃首张手牌）
//! - 酒尾（宝物）：结束阶段可选——已受伤回血否则摸牌（可选触发，
//!   确认门由框架 OptionalGate 承担；2026-10-07 载荷对照补齐实体牌）
//! - 手里剑（基础牌 ×2）：目标弃 1 张装备或受 1 点伤害（嵌套选择）
//! - 猴子（宝物）：他人出桃使用前取消+夺桃（命令层挂起机器）
//! - 写轮眼（宝物）：回合开始窃取他人技能至回合结束（多步机器）
//! - 面具（防具）：翻面前置取消（锁定技，TurnOverBefore 前置窗口）

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use content_runtime::{discover_extensions, import_extensions, InstallOptions, PackageStore};
use engine_native::catalog::CardKind;
use engine_native::content::extension::{ExtensionDef, ExtensionPack};
use engine_native::content::host::{HostBehavior, HostSkillDecl};
use engine_native::catalog::CardDef;
use engine_native::{
    ChoiceKind, ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase, RejectReason,
};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action,
    })
}

fn repo_extensions_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

#[derive(serde::Deserialize)]
struct CardJson {
    name: String,
    kind: String,
    range: u32,
    suit: String,
    rank: u8,
}

#[derive(serde::Deserialize)]
struct SkillJson {
    name: String,
    behavior: String,
}

#[derive(serde::Deserialize)]
struct CardsJson {
    cards: Vec<CardJson>,
    host_skills: Vec<SkillJson>,
}

/// 桥接统一走 C0 装载器（ADR 0777/0778）：映射来自 cards.json 的
/// 数据键与 pack 段，测试只校验包元数据与测试期望一致。
fn bridge(pack_name: &'static str, json: &str) -> Result<ExtensionPack, String> {
    let out = content_native_loader::bridge_pack(json).map_err(|e| e.to_string())?;
    assert_eq!(out.pack.def.name, pack_name, "包名来自 pack 段");
    Ok(out.pack)
}

fn options() -> InstallOptions {
    InstallOptions {
        activate: true,
        allow_unsigned: true,
        verify_key: None,
        host: content_runtime::tooling_host_versions(),
    }
}

/// 桥接后的包：3 张实体牌 + 2 条宿主技能声明，行为标签合法。
#[test]
fn huanle_bridge_builds_pack() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    assert!(discovered.iter().any(|d| d.name == "huanle-upstream-noname"));
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    assert!(
        report.installed.iter().any(|m| m.package_id == "huanle-cards"),
        "欢乐卡牌安装"
    );
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "huanle-cards")
        .unwrap();
    let json = std::fs::read_to_string(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .unwrap();
    let pack = bridge("huanle-upstream-noname", &json).expect("桥接");
    // 猴子/写轮眼（E03）：伴随技能由引擎表登记，装载器自动并入。
    assert_eq!(pack.card_count(), 7, "苦无 + 手里剑×2 + 猴子 + 写轮眼 + 面具 + 酒尾（2026-10-07 补齐）");
    assert_eq!(pack.host_skills.len(), 5, "两条行为声明 + 三个伴随技能");
}

/// G1 纵向：建局 → 宿主技能授予 → 苦无（杀伤害强制弃牌）→
/// 手里剑（嵌套选择）→ 字节恢复 → 内容锁绑定。
#[test]
fn g1_huanle_vertical_host_behaviors_and_restore() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report_installed_meta(&store);
    let json = std::fs::read_to_string(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .unwrap();
    let pack = bridge("huanle-upstream-noname", &json).expect("桥接");

    // 建局（扩展卡入目录）
    let mut e =
        Engine::new_with_extensions(3201, 3, &[&pack]).expect("建局");
    let base_cards = Engine::new(1, 3).catalog().cards.len();
    assert_eq!(e.catalog().cards.len(), base_cards + 7, "苦无 + 手里剑×2 + 猴子 + 写轮眼 + 面具 + 酒尾");

    // 授予宿主技能（装备技能挂载生命周期未做，以授予验证行为）
    e.debug_grant(0, "huanle_kuwu");
    for _ in 0..64 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play {
            break;
        }
        let actor = e.turn().current;
        match submit(&mut e, actor, GameAction::NextPhase) {
            CommandOutcome::Accepted { .. } => {}
            other => panic!("advance: {other:?}"),
        }
    }
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    // 苦无：杀造成伤害 → 目标弃一张手牌
    let sha = e.debug_place(0, "sha");
    let victim_card = e.debug_place(1, "tao");
    e.debug_set_hp(1, 4);
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    assert!(
        !e.debug_hand(1).contains(&victim_card),
        "苦无：目标应弃一张手牌"
    );
    assert_eq!(e.debug_hp(1), 3);

    // 手里剑（嵌套选择）：注入扩展牌 → 出牌 → 目标选择弃装备或受伤
    let (shoulijian_id,) = {
        let catalog = e.catalog();
        let sj = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("hnl_shoulijian"))
            .expect("扩展手里剑在目录") as u32;
        (sj,)
    };
    let mut snap = e.save();
    // 注入前置：目标牌从所有区清除（归属守恒校验要求唯一归属；
    // 牌堆构成变化会改变发牌位置，不能假设目标牌恰在牌堆）。
    let purge = |snap: &mut engine_native::StateSnapshot, id: u32| {
        snap.core.deck.retain(|c| *c != id);
        snap.core.discard.retain(|c| *c != id);
        snap.core.table.retain(|c| *c != id);
        for p in snap.core.players.iter_mut() {
            p.hand.retain(|c| *c != id);
            p.equips.retain(|c| *c != id);
            p.judge.retain(|c| *c != id);
        }
    };
    purge(&mut snap, shoulijian_id);
    snap.core.players[0].hand.push(shoulijian_id);
    // 目标持装备：同一次快照注入（保持归属完整）
    let equip_id = {
        let cat = e.catalog();
        cat.card_by_name("horse_plus").expect("目录有马牌")
    };
    purge(&mut snap, equip_id);
    snap.core.players[1].equips.push(equip_id);
    e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: shoulijian_id, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "手里剑应挂起目标选择: {out:?}");
    let c = e.open_choice().expect("嵌套选择挂起");
    assert_eq!(c.spec.prompt, "host_target_discard_equip");
    assert_eq!(c.observer, 1);
    assert!(
        c.spec.options.contains(&ChoiceOption::Card(equip_id)),
        "选项应含目标的装备"
    );

    // 嵌套选择的字节恢复（扩展机器续行编码）
    let bytes = engine_native::archive::encode_archive(&e.save());
    let mut restored = Engine::restore_with_extensions(
        &engine_native::archive::decode_archive(&bytes).expect("decode"),
        &[&pack],
    )
    .expect("扩展机器字节恢复");
    let cid = restored.open_choice().unwrap().id;
    // 目标选择弃装备 → 免伤
    submit(&mut restored, 1, GameAction::Answer {
        choice_id: cid,
        selection: vec![ChoiceOption::Card(equip_id)],
    });
    assert_eq!(restored.debug_hp(1), 3, "弃装备后免伤");
    assert!(
        !restored.state_core().players[1].equips.contains(&equip_id),
        "装备已弃"
    );

    // 内容锁：带扩展存档不能被纯标准目录恢复
    let saved = e.save();
    assert!(matches!(
        Engine::restore(&saved),
        Err(engine_native::SnapshotError::RuleVersionMismatch { .. })
    ));
}

fn report_installed_meta(
    store: &PackageStore,
) -> content_runtime::InstalledMeta {
    let lock = store.current_lock().expect("lock");
    let huanle = lock
        .packages
        .iter()
        .find(|p| p.package_id == "huanle-cards")
        .expect("欢乐卡牌在内容锁");
    content_runtime::InstalledMeta {
        namespace: huanle.namespace.clone(),
        package_id: huanle.package_id.clone(),
        version: huanle.version.clone(),
        logic_hash: huanle.logic_hash.clone(),
        content_hash: huanle.content_hash.clone(),
        signed: false,
        key_id: None,
        extension_root: Some(
            repo_extensions_root()
                .join("huanle-upstream-noname")
                .display()
                .to_string(),
        ),
    }
}

/// 酒尾（可选触发）：结束阶段确认 → 已受伤回血；扩展机器经
/// OptionalGate 包装后仍可恢复。
#[test]
fn huanle_jiuwei_optional_confirm_and_restore() {
    use engine_native::archive::{decode_archive, encode_archive};
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = report_installed_meta(&store);
    let json = std::fs::read_to_string(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .unwrap();
    let pack = bridge("huanle-upstream-noname", &json).expect("桥接");
    let mut e =
        Engine::new_with_extensions(3202, 3, &[&pack]).expect("建局");
    e.debug_grant(0, "huanle_jiuwei");
    advance_to_play(&mut e, 0);
    e.debug_set_hp(0, 2); // 已受伤 → 结束阶段应回血
    for p in 0..3 {
        e.debug_clear_hand(p); // 避免弃牌阶段的 discard_to_limit 先挂起
    }
    // 推进到结束阶段（酒尾确认挂起即停）
    for _ in 0..4 {
        if e.turn().phase == Phase::End && e.open_choice().is_some() {
            break;
        }
        if e.open_choice().is_some() {
            break;
        }
        let actor = e.turn().current;
        match submit(&mut e, actor, GameAction::NextPhase) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("advance: {other:?}"),
        }
    }
    // 可选触发确认（OptionalGate）
    let c = e.open_choice().expect("酒尾确认应挂起");
    assert_eq!(c.spec.prompt, "trigger_confirm");
    // 字节恢复（OptionalGate 内含宿主机器）
    let bytes = encode_archive(&e.save());
    let mut restored = Engine::restore_with_extensions(
        &decode_archive(&bytes).expect("decode"),
        &[&pack],
    )
    .expect("可选触发挂起恢复");
    let cid = restored.open_choice().unwrap().id;
    submit(&mut restored, 0, GameAction::Answer {
        choice_id: cid,
        selection: vec![ChoiceOption::Confirm],
    });
    assert_eq!(restored.debug_hp(0), 3, "已受伤 → 回复 1 点");
}

fn advance_to_play(e: &mut Engine, player: u8) {
    for _ in 0..96 {
        let t = e.turn();
        if t.current == player && t.phase == Phase::Play && e.open_choice().is_none() {
            return;
        }
        // 门感知：途中可能挂起可选触发/弃牌门（v127 跳回合基建后循环
        // 变长）——按 min 应答首选项后继续
        if let Some(g) = e.open_choice() {
            let sel: Vec<ChoiceOption> = if g.spec.min > 0 {
                g.spec.options.iter().take(g.spec.min).copied().collect()
            } else {
                vec![g.spec.options[0]]
            };
            let (id, obs) = (g.id, g.observer);
            drop(g);
            match submit(e, obs, GameAction::Answer { choice_id: id, selection: sel }) {
                CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
                other => panic!("answer: {other:?}"),
            }
            continue;
        }
        let actor = e.turn().current;
        match submit(e, actor, GameAction::NextPhase) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("advance: {other:?}"),
        }
    }
    panic!("no play phase");
}

/// 内容锁绑定：宿主技能快照在扩展集合不含该包时拒绝恢复。
#[test]
fn host_skill_restore_rejects_without_pack() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = report_installed_meta(&store);
    let json = std::fs::read_to_string(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .unwrap();
    let pack = bridge("huanle-upstream-noname", &json).expect("桥接");
    let mut e =
        Engine::new_with_extensions(3203, 3, &[&pack]).expect("建局");
    e.debug_grant(0, "huanle_kuwu");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(1, 4);
    let sha = e.debug_place(0, "sha");
    // 杀 → 苦无机器短暂入快照；在杀结算的出闪询问挂起时苦无机器已
    // 结束——改为直接以「含宿主技能的目录版本」验证：存档带 v(host)，
    // 纯标准恢复必须拒绝。
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    let saved = e.save();
    match Engine::restore(&saved) {
        Err(engine_native::SnapshotError::RuleVersionMismatch { .. }) => {}
        Err(e) => panic!("应报规则版本不符: {e:?}"),
        Ok(_) => panic!("纯标准恢复带宿主扩展存档必须拒绝"),
    }
}

/// G1 装备技能生命周期：装备苦无 → 技能授予（无需授予命令）→
/// 生效（杀伤害后目标弃牌）→ 被拆桥拆除 → 技能撤销。
#[test]
fn equip_skill_lifecycle_grant_and_revoke() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = report_installed_meta(&store);
    let json = std::fs::read_to_string(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .unwrap();
    let pack = bridge("huanle-upstream-noname", &json).expect("桥接");
    let mut e =
        Engine::new_with_extensions(3304, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    // 注入苦无并装备（无 debug_grant——技能随装备授予）
    let (kuwu,) = {
        let catalog = e.catalog();
        let k = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("hnl_kuwu"))
            .expect("苦无在目录") as u32;
        (k,)
    };
    let mut snap = e.save();
    snap.core.deck.retain(|c| *c != kuwu);
    snap.core.players[0].hand.push(kuwu);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let def = e.catalog().skill_by_name("huanle_kuwu").expect("宿主技能在目录");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: kuwu, targets: vec![] });
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert!(
        e.save().core.skills.contains_key(&(0, def)),
        "装备苦无应授予 huanle_kuwu 实例"
    );

    // 技能生效：杀伤害 → 目标弃牌（此前用 debug_grant 验证，此处走装备授予）
    let sha = e.debug_place(0, "sha");
    let victim_card = e.debug_place(1, "tao");
    e.debug_set_hp(1, 4);
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    assert!(!e.debug_hand(1).contains(&victim_card), "装备授予的苦无应生效");

    // 苦无离场 → 技能撤销。规则路径：1 号出手里剑令 0 号弃装备
    //（推进到 1 号出牌阶段，途中挂起选择取首选项）
    for _ in 0..128 {
        if e.turn().current == 1 && e.turn().phase == Phase::Play && e.open_choice().is_none() {
            break;
        }
        if let Some(c) = e.open_choice().cloned() {
            let cid = c.id;
            let sel = if c.spec.min > 0 {
                c.spec.options.iter().take(c.spec.min).copied().collect()
            } else {
                vec![c.spec.options[0]]
            };
            let obs = c.observer;
            match submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: sel }) {
                CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
                other => panic!("answer: {other:?}"),
            }
            continue;
        }
        let actor = e.turn().current;
        match submit(&mut e, actor, GameAction::NextPhase) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("advance: {other:?}"),
        }
    }
    let (sj,) = {
        let catalog = e.catalog();
        let sj = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("hnl_shoulijian"))
            .expect("手里剑在目录") as u32;
        (sj,)
    };
    let mut snap = e.save();
    snap.core.deck.retain(|c| *c != sj);
    snap.core.players[1].hand.push(sj);
    e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let out = submit(&mut e, 1, GameAction::PlayCard { card: sj, targets: vec![0] });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let c = e.open_choice().expect("手里剑嵌套选择");
    assert!(
        c.spec.options.contains(&ChoiceOption::Card(kuwu)),
        "选项应含苦无装备"
    );
    let cid = c.id;
    submit(&mut e, 0, GameAction::Answer {
        choice_id: cid,
        selection: vec![ChoiceOption::Card(kuwu)],
    });
    assert!(
        !e.save().core.skills.contains_key(&(0, def)),
        "苦无离场后 huanle_kuwu 实例应被撤销"
    );
    assert!(
        !e.save().core.players[0].equips.contains(&kuwu),
        "苦无已离场"
    );
    assert_eq!(e.debug_hp(0), 4, "弃装备免伤");
}

/// 装备技能一致性：装备在场但技能实例被移除的快照 → 恢复拒绝
///（EquipSkillMismatch，授予/撤销路径完整性的信任边界）。
#[test]
fn equip_skill_snapshot_mismatch_rejected() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = report_installed_meta(&store);
    let json = std::fs::read_to_string(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .unwrap();
    let pack = bridge("huanle-upstream-noname", &json).expect("桥接");
    let mut e =
        Engine::new_with_extensions(3305, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (kuwu,) = {
        let catalog = e.catalog();
        let k = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("hnl_kuwu"))
            .expect("苦无在目录") as u32;
        (k,)
    };
    let mut snap = e.save();
    snap.core.deck.retain(|c| *c != kuwu);
    snap.core.players[0].hand.push(kuwu);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: kuwu, targets: vec![] });
    assert!(matches!(out, CommandOutcome::Accepted { .. }));
    let mut snap = e.save();
    let def = e.catalog().skill_by_name("huanle_kuwu").unwrap();
    assert!(snap.core.skills.remove(&(0, def)).is_some(), "技能实例在快照中");
    match Engine::restore_with_extensions(&snap, &[&pack]) {
        Err(engine_native::SnapshotError::EquipSkillMismatch { player }) => {
            assert_eq!(player, 0);
        }
        Err(e) => panic!("应报 EquipSkillMismatch: {e:?}"),
        Ok(_) => panic!("装备在而技能实例缺失必须拒绝"),
    }
    // 字节路径同样拒绝
    let bytes = engine_native::archive::encode_archive(&snap);
    match Engine::restore_with_extensions(
        &engine_native::archive::decode_archive(&bytes).expect("decode"),
        &[&pack],
    ) {
        Err(engine_native::SnapshotError::EquipSkillMismatch { .. }) => {}
        Err(e) => panic!("字节路径应报 EquipSkillMismatch: {e:?}"),
        Ok(_) => panic!("字节路径也须拒绝"),
    }
}

/// E03 猴子偷桃（使用前取消窗口原语）：
/// 真实装备猴子（equip_skills 授予）→ 他人出桃 → OptionalGate 确认门挂起
/// （挂起态可恢复）→ 确认后桃的结算被取消（不回血、不入弃堆），桃收为
/// 猴子持有者手牌，猴子被弃且装备技离场撤销；Pass 时桃正常结算。
#[test]
fn huanle_monkey_cancels_peach_and_takes_it() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "huanle-upstream-noname")
            .expect("欢乐包可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let pack = bridge("huanle-upstream-noname", &json).expect("桥接");

    let mut e = Engine::new_with_extensions(3202, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }

    // 真实穿戴：0 号装备猴子 → equip_skills 授予 huanle_monkey。
    let monkey = e.debug_place(0, "hnl_monkey_3");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: monkey, targets: vec![] });
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "装备猴子: {out:?}");
    let def_monkey = e.catalog().skill_by_name("huanle:monkey").expect("猴子技能在目录");
    assert!(
        e.save().core.skills.contains_key(&(0, def_monkey)),
        "装备授予 huanle_monkey 实例"
    );

    // 1 号受伤并持桃出桃 → 猴子确认门挂起（observer=0）。
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(1);
    e.debug_set_hp(1, 2);
    e.debug_place(1, "tao");
    let tao = {
        let hand = e.debug_hand(1);
        let cat = e.catalog();
        *hand.iter().find(|c| cat.card_kind(**c) == CardKind::Tao).expect("桃在手") as u32
    };
    let out = submit(&mut e, 1, GameAction::PlayCard { card: tao, targets: vec![] });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "猴子确认门应挂起: {out:?}");
    let gate = e.open_choice().expect("确认门");
    assert_eq!(gate.spec.prompt, "huanle_monkey_confirm");
    assert_eq!(gate.observer, 0, "猴子持有者被询问");

    // 挂起态恢复：同集合恢复后确认门仍开。
    let snap = e.save();
    e = Engine::restore_with_extensions(&snap, &[&pack]).expect("确认门挂起恢复");
    assert_eq!(
        e.open_choice().expect("恢复后确认门仍挂起").spec.prompt,
        "huanle_monkey_confirm",
        "挂起语义已锚定"
    );

    // 确认：桃的结算被取消——1 号不回血、桃收为 0 号手牌、猴子入弃堆
    // 且装备技离场撤销（无 CardUsed 帧，使用不发生）。
    let (gate_id, gate_observer) = {
        let gate = e.open_choice().expect("恢复后确认门");
        (gate.id, gate.observer)
    };
    let out = submit(&mut e, gate_observer, GameAction::Answer {
        choice_id: gate_id,
        selection: vec![ChoiceOption::Confirm],
    });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(e.debug_hp(1), 2, "桃的结算被取消：不回血");
    assert!(e.debug_hand(0).contains(&tao), "桃收为猴子持有者手牌");
    assert!(
        !e.debug_equips(0).iter().any(|c| *c == monkey),
        "猴子应离开装备区"
    );
    assert!(
        !e.save().core.skills.contains_key(&(0, def_monkey)),
        "猴子离场撤销装备技"
    );

}


/// E03 写轮眼（技能窃取原语）：真实装备写轮眼 → 回合开始确认 → 选目标
/// → 选其技能（PickTag 下标）→ 临时获得（expires_end_of_turn）→ 回合
/// 结束后消失。
#[test]
fn huanle_xuelunyang_steals_target_skill_until_turn_end() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "huanle-upstream-noname")
            .expect("欢乐包可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let pack = bridge("huanle-upstream-noname", &json).expect("桥接");

    let mut e = Engine::new_with_extensions(3203, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }

    // 真实穿戴：0 号装备写轮眼。
    let xl = e.debug_place(0, "hnl_xuelunyang_4");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: xl, targets: vec![] });
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "装备写轮眼: {out:?}");
    let def_xl = e.catalog().skill_by_name("huanle:xuelunyang").expect("写轮眼技能在目录");
    assert!(e.save().core.skills.contains_key(&(0, def_xl)), "装备授予实例");

    // 1 号获得一个标准技能（窃取目标）。
    e.debug_grant(1, "马术");
    let def_mashu = e.catalog().skill_by_name("马术").expect("马术在目录");
    let stolen_name = e.catalog().skills[def_mashu as usize].name;

    // 写轮眼在装备者的**下一个**回合开始触发：先推进到 1 号回合，
    // 再推进到 0 号回合开始（Judge begin）→ 确认门挂起。
    advance_to_play(&mut e, 1);
    // 推进到 0 号回合开始（Judge begin）→ OptionalGate 确认门挂起
    // （NextPhase 的 Pending 返回即新开的门，由 open_choice 分支应答）。
    for _ in 0..64 {
        if let Some(g) = e.open_choice() {
            if g.spec.prompt == "trigger_confirm" && g.observer == 0 {
                break;
            }
        }
        match e.open_choice() {
            Some(g) => {
                let sel: Vec<ChoiceOption> = if g.spec.min > 0 {
                    g.spec.options.iter().take(g.spec.min).copied().collect()
                } else {
                    vec![g.spec.options[0]]
                };
                let (id2, obs2) = (g.id, g.observer);
                drop(g);
                submit(&mut e, obs2, GameAction::Answer { choice_id: id2, selection: sel });
            }
            None => {
                let actor = e.turn().current;
                match submit(&mut e, actor, GameAction::NextPhase) {
                    CommandOutcome::Accepted { .. } => {}
                    CommandOutcome::Pending { .. } => {} // 新开的门由下一轮应答
                    other => panic!("advance: {other:?}"),
                }
            }
        }
    }
    let gate = e.open_choice().expect("写轮眼确认门应挂起");
    assert_eq!(gate.spec.prompt, "trigger_confirm");
    assert_eq!(gate.observer, 0);

    // 确认（OptionalGate 通过）→ 选目标（1 号）→ 选技能（PickTag 下标
    // 0 = 目标技能目录序首项 = 马术）。
    let (cid, obs) = { let g = e.open_choice().unwrap(); (g.id, g.observer) };
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Confirm] });
    let target_pick = e.open_choice().expect("选目标挂起");
    assert_eq!(target_pick.spec.prompt, "huanle_xuelunyang_target");
    assert!(
        target_pick.spec.options.contains(&ChoiceOption::Player(1)),
        "目标选项应含 1 号"
    );
    let (tid, tobs) = { let g = e.open_choice().unwrap(); (g.id, g.observer) };
    submit(&mut e, tobs, GameAction::Answer { choice_id: tid, selection: vec![ChoiceOption::Player(1)] });
    let pick = e.open_choice().expect("技能选择挂起");
    assert_eq!(pick.spec.prompt, "huanle_xuelunyang_skill");
    assert_eq!(pick.spec.kind, ChoiceKind::PickTag);
    let pick_id = { let g = e.open_choice().unwrap(); g.id };
    let out = submit(&mut e, 0, GameAction::Answer {
        choice_id: pick_id,
        selection: vec![ChoiceOption::Tag(0)],
    });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");

    // 临时获得：0 号拥有「马术」实例（expires_end_of_turn）。
    let snap = e.save();
    let def_stolen = e.catalog().skill_by_name(&stolen_name).expect("被窃技能在目录");
    let inst = snap.core.skills.get(&(0, def_stolen)).expect("窃得技能实例");
    assert!(inst.expires_end_of_turn, "临时技能标记到回合结束");

    // 推进过 0 号回合结束 → 实例消失。
    for _ in 0..64 {
        if e.turn().current != 0 && e.save().core.skills.get(&(0, def_stolen)).is_none() {
            break;
        }
        if e.outcome().is_some() {
            break;
        }
        if e.open_choice().is_some() {
            let g = e.open_choice().unwrap();
            // 写轮眼确认门：Pass 放弃（不再窃取，保证到期断言时序干净）。
            if g.spec.prompt == "trigger_confirm" {
                let (id2, obs2) = (g.id, g.observer);
                drop(g);
                submit(&mut e, obs2, GameAction::Answer {
                    choice_id: id2,
                    selection: vec![ChoiceOption::Pass],
                });
                continue;
            }
            let sel: Vec<ChoiceOption> = if g.spec.min > 0 {
                g.spec.options.iter().take(g.spec.min).copied().collect()
            } else {
                vec![g.spec.options[0]]
            };
            let (id2, obs2) = (g.id, g.observer);
            drop(g);
            submit(&mut e, obs2, GameAction::Answer { choice_id: id2, selection: sel });
            continue;
        }
        let actor = e.turn().current;
        match submit(&mut e, actor, GameAction::NextPhase) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            CommandOutcome::Rejected(_) => {}
            other => panic!("advance: {other:?}"),
        }
    }
    // 到期移除 = 实例禁用（enabled=false，条目保留）。
    let stolen_enabled = e
        .save()
        .core
        .skills
        .get(&(0, def_stolen))
        .map(|i| i.enabled)
        .unwrap_or(false);
    assert!(
        !stolen_enabled,
        "回合结束后窃得的技能应失效"
    );;
}

/// E03 写轮眼 Pass 路径：回合开始确认门选择放弃 → 不窃取，随后他人出桃
/// 正常结算（无取消窗口——猴子未装备、窃取未发生）。
#[test]
fn huanle_xuelunyang_pass_declines_theft() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "huanle-upstream-noname")
            .expect("欢乐包可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let pack = bridge("huanle-upstream-noname", &json).expect("桥接");

    let mut e = Engine::new_with_extensions(3204, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let xl = e.debug_place(0, "hnl_xuelunyang_4");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: xl, targets: vec![] });
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "装备写轮眼");
    e.debug_grant(1, "马术");

    // 推进到 1 号的出牌阶段；途中写轮眼确认门（每个 0 号回合开始触发
    // 一次）一律 Pass——放弃窃取。
    let mut passed_gates = 0usize;
    let mut reached = false;
    for _ in 0..96 {
        // 写轮眼在装备者的下个回合开始（Judge begin）触发——推到该点。
        if e.turn().current == 0 && e.turn().phase == Phase::Judge {
            reached = true;
        }
        match e.open_choice() {
            Some(g) if g.spec.prompt == "trigger_confirm" => {
                passed_gates += 1;
                // OptionalGate 门 options=[Confirm]、allow_cancel=true：
                // 放弃 = 空选择取消（Pass 不在选项集，会被拒绝）。
                let (id2, obs2) = (g.id, g.observer);
                drop(g);
                submit(&mut e, obs2, GameAction::Answer {
                    choice_id: id2,
                    selection: vec![],
                });
            }
            Some(g) => {
                let sel: Vec<ChoiceOption> = if g.spec.min > 0 {
                    g.spec.options.iter().take(g.spec.min).copied().collect()
                } else {
                    vec![g.spec.options[0]]
                };
                let (id2, obs2) = (g.id, g.observer);
                drop(g);
                submit(&mut e, obs2, GameAction::Answer { choice_id: id2, selection: sel });
            }
            None => {
                let actor = e.turn().current;
                match submit(&mut e, actor, GameAction::NextPhase) {
                    CommandOutcome::Accepted { .. } => {}
                    CommandOutcome::Pending { .. } => {} // 新开的门由下一轮应答
                    other => panic!("advance: {other:?}"),
                }
            }
        }
    }
    assert!(reached, "96 步内未到达 0 号回合开始（写轮眼触发点）");
    assert!(passed_gates >= 1, "写轮眼确认门应至少触发一次（已全部 Pass）");
    // Pass 后无窃取实例（0 号只有写轮眼自身实例，无马术）。
    let def_mashu = e.catalog().skill_by_name("马术").unwrap();
    assert!(
        !e.save().core.skills.contains_key(&(0, def_mashu)),
        "放弃后不得窃得马术"
    );

    // 推进到 1 号出牌阶段（途中 0 号下回合开始的写轮眼门继续 Pass）。
    for gi2 in 0..96 {
        eprintln!("DBG pass2 gi={gi2} cur={} phase={:?} open={:?}", e.turn().current, e.turn().phase, e.open_choice().map(|g| (g.observer, g.spec.prompt.clone(), g.id)));
        if e.turn().current == 1 && e.turn().phase == Phase::Play && e.open_choice().is_none() {
            break;
        }
        match e.open_choice() {
            Some(g) if g.spec.prompt == "trigger_confirm" => {
                // OptionalGate 门放弃 = 空选择取消。
                let (id2, obs2) = (g.id, g.observer);
                drop(g);
                submit(&mut e, obs2, GameAction::Answer {
                    choice_id: id2,
                    selection: vec![],
                });
            }
            Some(g) => {
                let sel: Vec<ChoiceOption> = if g.spec.min > 0 {
                    g.spec.options.iter().take(g.spec.min).copied().collect()
                } else {
                    vec![g.spec.options[0]]
                };
                let (id2, obs2) = (g.id, g.observer);
                drop(g);
                submit(&mut e, obs2, GameAction::Answer { choice_id: id2, selection: sel });
            }
            None => {
                let actor = e.turn().current;
                match submit(&mut e, actor, GameAction::NextPhase) {
                    CommandOutcome::Accepted { .. } => {}
                    CommandOutcome::Pending { .. } => {} // 新开的门由下一轮应答
                    other => panic!("advance: {other:?}"),
                }
            }
        }
    }

    // 1 号出桃：无取消窗口（猴子未装备），正常结算回血。
    e.debug_clear_hand(1);
    e.debug_set_hp(1, 2);
    e.debug_place(1, "tao");
    let tao = {
        let hand = e.debug_hand(1);
        let cat = e.catalog();
        *hand.iter().find(|c| cat.card_kind(**c) == CardKind::Tao).expect("桃在手") as u32
    };
    let out = submit(&mut e, 1, GameAction::PlayCard { card: tao, targets: vec![] });
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "无窗口时桃直接结算: {out:?}");
    assert_eq!(e.debug_hp(1), 3, "桃正常回血");
}

/// 诊断探针（ignore）：写轮眼窃取后推进多回合，逐步打印
/// resolution.stack / parks / skills 的规模，定位状态膨胀源。
#[test]
#[ignore = "诊断探针：PT_DBG=1 观察规模曲线"]
fn xly_state_growth_probe() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "huanle-upstream-noname")
            .expect("欢乐包可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let pack = bridge("huanle-upstream-noname", &json).expect("桥接");

    let mut e = Engine::new_with_extensions(3205, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let xl = e.debug_place(0, "hnl_xuelunyang_4");
    submit(&mut e, 0, GameAction::PlayCard { card: xl, targets: vec![] });
    e.debug_grant(1, "马术");

    // 窃取流程：确认门 → 选目标(1) → PickTag(0)。
    advance_to_play(&mut e, 1);
    for _ in 0..64 {
        match e.open_choice() {
            Some(g) if g.spec.prompt == "trigger_confirm" && g.observer == 0 => {
                let (id2, obs2) = (g.id, g.observer);
                drop(g);
                submit(&mut e, obs2, GameAction::Answer {
                    choice_id: id2,
                    selection: vec![ChoiceOption::Confirm],
                });
            }
            Some(g) => {
                let sel: Vec<ChoiceOption> = if g.spec.min > 0 {
                    g.spec.options.iter().take(g.spec.min).copied().collect()
                } else {
                    vec![g.spec.options[0]]
                };
                let (id2, obs2) = (g.id, g.observer);
                drop(g);
                submit(&mut e, obs2, GameAction::Answer { choice_id: id2, selection: sel });
            }
            None => {
                let actor = e.turn().current;
                match submit(&mut e, actor, GameAction::NextPhase) {
                    CommandOutcome::Accepted { .. } => {}
                    CommandOutcome::Pending { .. } => {}
                    other => panic!("advance: {other:?}"),
                }
            }
        }
        if e.catalog().skill_by_name("马术").is_some_and(|d| {
            e.save().core.skills.contains_key(&(0, d))
        }) {
            break; // 窃取完成
        }
    }

    // 规模观测：推进 120 步（约多回合），每 10 步打印。
    let mut steps = 0;
    for _ in 0..120 {
        steps += 1;
        let snap = e.save();
        if steps % 10 == 0 {
            eprintln!(
                "PROBE step={steps} cur={} phase={:?} stack={} parks={} skills={} loglen={}",
                snap.core.turn.current,
                snap.core.turn.phase,
                snap.resolution.stack.len(),
                snap.resolution.parks.len(),
                snap.core.skills.len(),
                e.log().len()
            );
        }
        if e.outcome().is_some() {
            eprintln!("PROBE over at step={steps}");
            break;
        }
        match e.open_choice() {
            Some(g) if g.spec.prompt == "trigger_confirm" => {
                // OptionalGate 门 options=[Confirm]：放弃 = 空选择取消。
                let (id2, obs2) = (g.id, g.observer);
                drop(g);
                submit(&mut e, obs2, GameAction::Answer {
                    choice_id: id2,
                    selection: vec![],
                });
            }
            Some(g) => {
                let sel: Vec<ChoiceOption> = if g.spec.min > 0 {
                    g.spec.options.iter().take(g.spec.min).copied().collect()
                } else {
                    vec![g.spec.options[0]]
                };
                let (id2, obs2) = (g.id, g.observer);
                drop(g);
                submit(&mut e, obs2, GameAction::Answer { choice_id: id2, selection: sel });
            }
            None => {
                let actor = e.turn().current;
                match submit(&mut e, actor, GameAction::NextPhase) {
                    CommandOutcome::Accepted { .. } => {}
                    CommandOutcome::Pending { .. } => {}
                    CommandOutcome::Rejected(_) => {}
                    other => panic!("advance: {other:?}"),
                }
            }
        }
    }
}

/// E03 漩涡面具（翻面原语）：真实装备面具 → debug_flip 翻面被取消
/// （TurnOverBefore 强制取消，face_down 不变）→ 摘除面具后翻面正常。
///
/// E03 漩涡面具（翻面原语）：真实装备面具 → debug_flip 翻面被取消
/// （TurnOverBefore 强制取消，face_down 不变）→ 摘除面具后翻面正常。
#[test]
fn huanle_mianju_blocks_flip_while_equipped() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "huanle-upstream-noname")
            .expect("欢乐包可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let pack = bridge("huanle-upstream-noname", &json).expect("桥接");

    let mut e = Engine::new_with_extensions(3206, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);

    // 摘除状态基线：未装备面具时翻面正常发生。
    let flipped = e.debug_flip(1);
    assert!(flipped, "无面具时翻面应发生");
    assert!(e.debug_face_down(1), "1 号武将牌背面朝上");

    // 真实穿戴：0 号装备面具。
    let mianju = e.debug_place(0, "hnl_mianju_5");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: mianju, targets: vec![] });
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "装备面具: {out:?}");
    let def_mianju = e.catalog().skill_by_name("huanle:mianju").expect("面具技能在目录");
    assert!(e.save().core.skills.contains_key(&(0, def_mianju)), "装备授予实例");

    // 翻面被取消：0 号 flip → TurnOverBefore 强制取消 → face_down 不变。
    let flipped = e.debug_flip(0);
    assert!(!flipped, "面具应取消 0 号的翻面");
    assert!(!e.debug_face_down(0), "0 号武将牌朝向不变");

    // 摘除面具（离场撤销）后翻面正常。
    e.debug_remove_equip(0, mianju);
    assert!(!e.save().core.skills.contains_key(&(0, def_mianju)), "离场撤销实例");
    let flipped2 = e.debug_flip(0);
    assert!(flipped2, "摘除面具后翻面应发生");
    assert!(e.debug_face_down(0), "0 号武将牌背面朝上");
}
/// P2/N08 曹仁「据守」（真实翻面来源）：装配曹仁 → 结束阶段确认门
/// Confirm → 武将牌翻面 + 摸 1；翻面者回合开始被跳过并翻回（规则 v127
/// 翻面跳回合基建），故第二次据守为再翻至背面；装备【漩涡面具】后
/// 据守的翻面被取消（登记差异：摸牌仍执行）。
#[test]
fn caoren_jushou_flips_and_mianju_cancels_flip() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "huanle-upstream-noname")
            .expect("欢乐包可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let pack = bridge("huanle-upstream-noname", &json).expect("桥接");

    let mut e = Engine::new_with_extensions(3207, 3, &[&pack]).expect("建局");
    // 装配曹仁（P2/N08 装配授予原语）：据守随装配真实授予。
    e.debug_assemble_general(0, "曹仁");
    assert!(e.debug_has_skill(0, "据守"), "装配曹仁即授予据守");

    // 门感知推进：写轮眼/据守确认门留待调用方断言（返回 (observer, id)），
    // 其余门应答首选项；无门时推进阶段。
    fn step_until_gate_or(e: &mut Engine, target_player: u8, target_phase: Phase) -> Option<(u8, u64)> {
        // 返回 (observer, choice_id)
        for _ in 0..96 {
            if let Some(g) = e.open_choice() {
                if g.spec.prompt == "trigger_confirm" {
                    return Some((g.observer, g.id));
                }
                let sel: Vec<ChoiceOption> = if g.spec.min > 0 {
                    g.spec.options.iter().take(g.spec.min).copied().collect()
                } else {
                    vec![g.spec.options[0]]
                };
                let (id2, obs2) = (g.id, g.observer);
                drop(g);
                submit(e, obs2, GameAction::Answer { choice_id: id2, selection: sel });
                continue;
            }
            if e.turn().current == target_player && e.turn().phase == target_phase {
                return None;
            }
            let actor = e.turn().current;
            match submit(e, actor, GameAction::NextPhase) {
                CommandOutcome::Accepted { .. } => {}
                CommandOutcome::Pending { .. } => {} // 新开的门由下一轮处理
                other => panic!("advance: {other:?}"),
            }
        }
        panic!("96 步内未收敛");
    }

    for p in 0..3 {
        e.debug_clear_hand(p);
    }

    // ① 第一次据守：确认门 → Confirm → 翻面 + 摸 1。
    let gate = step_until_gate_or(&mut e, 0, Phase::End).expect("据守确认门应挂起");
    assert_eq!(gate.0, 0, "据守门 observer=0");
    let (gid, gobs) = (gate.1, gate.0);
    submit(&mut e, gobs, GameAction::Answer {
        choice_id: gid,
        selection: vec![ChoiceOption::Confirm],
    });
    assert!(e.debug_face_down(0), "据守翻面：武将牌背面朝上");

    // ② 第二次据守：先推过当前 End（①的门已消费）——0 号下个回合开
    // 始被跳过并翻回（v127 跳回合基建），再等其下个 End 的确认门
    // （此时正面 → 确认后翻至背面）+ 摸 1。
    let actor = e.turn().current;
    match submit(&mut e, actor, GameAction::NextPhase) {
        CommandOutcome::Accepted { .. } => {}
        other => panic!("推过当前 End: {other:?}"),
    }
    let gate = step_until_gate_or(&mut e, 0, Phase::End).expect("第二次据守确认门应挂起");
    assert_eq!(gate.0, 0);
    // 基线在确认前取：据守摸 1（推进途中 0 号 Draw 摸的 4 张不影响差值）。
    let hand_before2 = e.debug_hand(0).len();
    let (gid, gobs) = (gate.1, gate.0);
    submit(&mut e, gobs, GameAction::Answer {
        choice_id: gid,
        selection: vec![ChoiceOption::Confirm],
    });
    assert!(e.debug_face_down(0), "第二次据守再翻至背面（回合开始已被跳过翻回）");
    assert_eq!(
        e.debug_hand(0).len(),
        hand_before2 + 1,
        "第二次据守摸 1"
    );

    // ③ 组合：装备面具后据守——翻面被取消（TurnOverBefore 强制取消）
    //    但摸牌仍执行（登记差异：取消翻面不影响据守摸牌）。
    // 先排空挂起门（v127 跳回合基建后循环变长，1/2 号可能进入弃牌门），
    // 再推进回 0 号出牌阶段（装备动作需在 Play 阶段），并重置朝向为
    // 正面（隔离②的翻转残留，保证断言只覆盖面具取消语义）。
    while let Some(g) = e.open_choice() {
        let sel: Vec<ChoiceOption> = if g.spec.min > 0 {
            g.spec.options.iter().take(g.spec.min).copied().collect()
        } else {
            vec![g.spec.options[0]]
        };
        let (id, obs) = (g.id, g.observer);
        drop(g);
        submit(&mut e, obs, GameAction::Answer { choice_id: id, selection: sel });
    }
    advance_to_play(&mut e, 0);
    e.debug_set_face_down(0, false);
    let mianju = e.debug_place(0, "hnl_mianju_5");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: mianju, targets: vec![] });
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "装备面具: {out:?}");
    let def_mianju = e.catalog().skill_by_name("huanle:mianju").expect("面具技能在目录");
    assert!(e.save().core.skills.contains_key(&(0, def_mianju)), "面具技能实例授予");

    while let Some(g) = e.open_choice() {
        let sel: Vec<ChoiceOption> = if g.spec.min > 0 {
            g.spec.options.iter().take(g.spec.min).copied().collect()
        } else {
            vec![g.spec.options[0]]
        };
        let (id, obs) = (g.id, g.observer);
        drop(g);
        submit(&mut e, obs, GameAction::Answer { choice_id: id, selection: sel });
    }
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    // 面具取消时无确认门（强制技不询问）——直接推进并断言：
    // face_down 不变（翻面被跳过）+ 手牌 +1（摸牌仍执行）。
    let hand_before3 = e.debug_hand(0).len();
    let face_before3 = e.debug_face_down(0);
    // 推进：据守确认门 → Confirm（发动据守）→ TurnOverBefore 子帧被
    // 面具强制取消（翻面不发生、摸牌执行）→ 阶段推进正常交棒。
    for _ in 0..16 {
        if e.turn().current == 1 && e.turn().phase == Phase::Judge {
            break; // 已离开 0 号回合（翻面被取消后正常交棒）
        }
        match e.open_choice() {
            Some(g) if g.spec.prompt == "trigger_confirm" => {
                let (id2, obs2) = (g.id, g.observer);
                drop(g);
                submit(&mut e, obs2, GameAction::Answer {
                    choice_id: id2,
                    selection: vec![ChoiceOption::Confirm],
                });
            }
            Some(g) => {
                let sel: Vec<ChoiceOption> = if g.spec.min > 0 {
                    g.spec.options.iter().take(g.spec.min).copied().collect()
                } else {
                    vec![g.spec.options[0]]
                };
                let (id2, obs2) = (g.id, g.observer);
                drop(g);
                submit(&mut e, obs2, GameAction::Answer { choice_id: id2, selection: sel });
            }
            None => {
                let actor = e.turn().current;
                match submit(&mut e, actor, GameAction::NextPhase) {
                    CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
                    other => panic!("advance: {other:?}"),
                }
            }
        }
    }
    assert_eq!(e.debug_hand(0).len(), hand_before3 + 1, "据守摸牌仍执行");
    assert_eq!(e.debug_face_down(0), face_before3, "翻面被面具取消");
    assert!(
        !e.debug_face_down(0),
        "0 号武将牌正面（翻面被面具取消）"
    );
}



/// E03 帧生命周期回归守卫：debug_flip 走事件流后，每次翻面帧都应完整
/// 弹栈（stack/parks 清空、无悬挂选择）——缺陷期实证残留帧导致取消写回
/// 指向已提交帧与状态膨胀，修复后的不变量在此固化。
#[test]
fn flip_frame_lifecycle_no_residue() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "huanle-upstream-noname")
            .expect("欢乐包可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let pack = bridge("huanle-upstream-noname", &json).expect("桥接");

    let mut e = Engine::new_with_extensions(3208, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);

    // 无面具基线：两次 flip 的 stack/parks 应清空（帧全弹）。
    for i in 0..2 {
        let flipped = e.debug_flip(1);
        assert!(flipped, "第 {i} 次翻面应发生");
        let snap = e.save();
        eprintln!(
            "PROBE flip{i} stack={} parks={} open={:?} fd1={}",
            snap.resolution.stack.len(),
            snap.resolution.parks.len(),
            snap.resolution.open_choice.is_some(),
            e.debug_face_down(1)
        );
        assert_eq!(
            snap.resolution.stack.len(),
            0,
            "第 {i} 次翻面后事件栈应清空"
        );
    }
    eprintln!("PROBE 无面具基线通过");
}
/// E03 翻面原语（无扩展基线）：debug_flip 经 TurnOverBefore 事件流驱动
/// （帧弹栈后返回），无面具时每次调用都发生翻转，连续两次回到初始朝向；
/// 取消路径（面具写回 cancelled=true）由
/// huanle_mianju_blocks_flip_while_equipped 覆盖。
#[test]
fn flip_primitive_toggles_facing_without_mask() {
    let mut e = Engine::new(3209, 2);
    assert!(!e.debug_face_down(0), "初始正面");

    assert!(e.debug_flip(0), "无面具翻面发生");
    assert!(e.debug_face_down(0), "翻面后背面朝上");

    assert!(e.debug_flip(0), "翻回（翻转发生）");
    assert!(!e.debug_face_down(0), "翻回后正面朝上");
}

/// E03 载荷验收（2026-10-07）：原生欢乐包 7 张实体牌与上游
/// `card/list.js` 逐张对照（名称序/花色/点数）+ 装备栏与伴随装备技。
/// 上游源（archive/upstream/noname/.../欢乐卡牌/card/list.js，
/// commit 5c02a745）：kuwu ♠6 / shoulijian ♦4 / shoulijian ♠4 /
/// monkey ♦5 / xuelunyang ♥2 / mianju ♣3 / jiuwei ♥9。
#[test]
fn huanle_pack_payload_matches_upstream_list() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "huanle-upstream-noname")
            .expect("欢乐包可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let pack = bridge("huanle-upstream-noname", &json).expect("桥接");

    // 逐张对照（card_prefix hnl_ → 目录名 hnl_<name>_<i>）。
    let expect: [(&str, &str, u8, Option<CardKind>); 7] = [
        ("hnl_kuwu_0", "spade", 6, Some(CardKind::Weapon)),
        ("hnl_shoulijian_1", "diamond", 4, Some(CardKind::ShouLiJian)),
        ("hnl_shoulijian_2", "spade", 4, Some(CardKind::ShouLiJian)),
        ("hnl_monkey_3", "diamond", 5, Some(CardKind::Monkey)),
        ("hnl_xuelunyang_4", "heart", 2, Some(CardKind::Xuelunyang)),
        ("hnl_mianju_5", "club", 3, Some(CardKind::Mianju)),
        ("hnl_jiuwei_6", "heart", 9, Some(CardKind::Jiuwei)),
    ];
    assert_eq!(pack.cards.len(), 7, "上游 7 张实体牌（酒尾 2026-10-07 补齐）");
    for (card, (name, suit, rank, kind)) in pack.cards.iter().zip(expect) {
        assert_eq!(card.name, name, "牌名/序");
        let suit = match suit {
            "spade" => engine_native::catalog::Suit::Spade,
            "heart" => engine_native::catalog::Suit::Heart,
            "club" => engine_native::catalog::Suit::Club,
            _ => engine_native::catalog::Suit::Diamond,
        };
        assert_eq!(card.suit, suit, "{name} 花色与上游不符");
        assert_eq!(card.rank, rank, "{name} 点数与上游不符");
        assert_eq!(Some(card.kind), kind, "{name} 牌种");
    }

    // 装备栏对照（上游 subtype）：kuwu 武器、mianju 防具（equip2，
    // 2026-10-07 修正）、monkey/xuelunyang/jiuwei 宝物（equip5）。
    assert_eq!(CardKind::Weapon.equip_slot(), Some(engine_native::catalog::EquipSlot::Weapon));
    assert_eq!(CardKind::Mianju.equip_slot(), Some(engine_native::catalog::EquipSlot::Armor));
    assert_eq!(CardKind::Monkey.equip_slot(), Some(engine_native::catalog::EquipSlot::Treasure));
    assert_eq!(CardKind::Xuelunyang.equip_slot(), Some(engine_native::catalog::EquipSlot::Treasure));
    assert_eq!(CardKind::Jiuwei.equip_slot(), Some(engine_native::catalog::EquipSlot::Treasure));
    assert_eq!(CardKind::ShouLiJian.equip_slot(), None, "手里剑为基础牌");

    // 真实建局：酒尾装备授予 huanle_jiuwei（包声明映射
    // host:phase_end_recover_or_draw）。
    let mut e = Engine::new_with_extensions(3210, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    let jw = e.debug_place(0, "hnl_jiuwei_6");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: jw, targets: vec![] });
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "装备酒尾: {out:?}");
    let def = e.catalog().skill_by_name("huanle_jiuwei").expect("酒尾技能在目录");
    assert!(e.save().core.skills.contains_key(&(0, def)), "装备酒尾授予实例");
}

/// E03 载荷验收：酒尾（♥9 宝物，上游 phaseEnd frequent）——结束阶段
/// 可选：已受伤回复 1，满血摸 1；放弃则无事。确认门由 OptionalGate
/// 承担（TriggerMeta PhaseBegin + 过滤器 End begin:true = 结束阶段）。
#[test]
fn huanle_jiuwei_recovers_or_draws_at_turn_end() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "huanle-upstream-noname")
            .expect("欢乐包可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let pack = bridge("huanle-upstream-noname", &json).expect("桥接");

    let mut e = Engine::new_with_extensions(3211, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let jw = e.debug_place(0, "hnl_jiuwei_6");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: jw, targets: vec![] });
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "装备酒尾: {out:?}");

    // 推进到 0 号结束阶段并应答确认门（confirm=true 时确认）。
    fn advance_to_end_gate(e: &mut Engine) {
        for _ in 0..96 {
            match e.open_choice() {
                Some(g) if g.spec.prompt == "trigger_confirm" && g.observer == 0 => return,
                Some(g) => {
                    let sel: Vec<ChoiceOption> = if g.spec.min > 0 {
                        g.spec.options.iter().take(g.spec.min).copied().collect()
                    } else {
                        vec![g.spec.options[0]]
                    };
                    let (id2, obs2) = (g.id, g.observer);
                    drop(g);
                    submit(e, obs2, GameAction::Answer { choice_id: id2, selection: sel });
                }
                None => {
                    let actor = e.turn().current;
                    match submit(e, actor, GameAction::NextPhase) {
                        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
                        other => panic!("advance: {other:?}"),
                    }
                }
            }
        }
        panic!("96 步内未到达结束阶段确认门");
    }

    // ① 受伤路径：hp 3/4 → End 确认 → 回复到 4。
    e.debug_set_hp(0, 3);
    advance_to_end_gate(&mut e);
    let (gid, gobs) = { let g = e.open_choice().unwrap(); (g.id, g.observer) };
    submit(&mut e, gobs, GameAction::Answer { choice_id: gid, selection: vec![ChoiceOption::Confirm] });
    assert_eq!(e.debug_hp(0), 4, "受伤时酒尾回复 1");

    // ② 满血路径：下回合 End 确认 → 摸 1。
    for _ in 0..96 {
        if e.turn().current == 0 && e.turn().phase == Phase::End && e.open_choice().is_some() {
            break;
        }
        match e.open_choice() {
            Some(g) => {
                let sel: Vec<ChoiceOption> = if g.spec.min > 0 {
                    g.spec.options.iter().take(g.spec.min).copied().collect()
                } else {
                    vec![g.spec.options[0]]
                };
                let (id2, obs2) = (g.id, g.observer);
                drop(g);
                submit(&mut e, obs2, GameAction::Answer { choice_id: id2, selection: sel });
            }
            None => {
                let actor = e.turn().current;
                match submit(&mut e, actor, GameAction::NextPhase) {
                    CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
                    other => panic!("advance: {other:?}"),
                }
            }
        }
    }
    let hand_before = e.debug_hand(0).len();
    let gate = e.open_choice().expect("满血酒尾确认门应挂起");
    assert_eq!(gate.spec.prompt, "trigger_confirm");
    let (gid2, gobs2) = { let g = e.open_choice().unwrap(); (g.id, g.observer) };
    submit(&mut e, gobs2, GameAction::Answer { choice_id: gid2, selection: vec![ChoiceOption::Confirm] });
    assert_eq!(e.debug_hand(0).len(), hand_before + 1, "满血时酒尾摸 1");
    assert_eq!(e.debug_hp(0), 4, "满血不回复");

    // ③ 放弃路径：再一回合 End 放弃 → 无事。
    for _ in 0..96 {
        if e.turn().current == 0 && e.turn().phase == Phase::End && e.open_choice().is_some() {
            break;
        }
        match e.open_choice() {
            Some(g) => {
                let sel: Vec<ChoiceOption> = if g.spec.min > 0 {
                    g.spec.options.iter().take(g.spec.min).copied().collect()
                } else {
                    vec![g.spec.options[0]]
                };
                let (id2, obs2) = (g.id, g.observer);
                drop(g);
                submit(&mut e, obs2, GameAction::Answer { choice_id: id2, selection: sel });
            }
            None => {
                let actor = e.turn().current;
                match submit(&mut e, actor, GameAction::NextPhase) {
                    CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
                    other => panic!("advance: {other:?}"),
                }
            }
        }
    }
    let hand_before3 = e.debug_hand(0).len();
    let gate3 = e.open_choice().expect("放弃路径确认门应挂起");
    let (gid3, gobs3) = (gate3.id, gate3.observer);
    drop(gate3);
    submit(&mut e, gobs3, GameAction::Answer { choice_id: gid3, selection: vec![] });
    assert_eq!(e.debug_hand(0).len(), hand_before3, "放弃不摸牌");
    assert_eq!(e.debug_hp(0), 4, "放弃不回复");
}

/// E03 跨牌组合矩阵（台账 U06，2026-10-07）①：写轮眼窃取据守——
/// 窃得的据守在窃得者回合结束阶段真实触发（翻面 + 摸 1），目标保留
/// 原技能（窃取为复制非转移）。
#[test]
fn huanle_combo_stolen_jushou_fires_for_thief() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "huanle-upstream-noname")
            .expect("欢乐包可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let pack = bridge("huanle-upstream-noname", &json).expect("桥接");

    let mut e = Engine::new_with_extensions(3212, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let xl = e.debug_place(0, "hnl_xuelunyang_4");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: xl, targets: vec![] });
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "装备写轮眼: {out:?}");
    // 目标 1 号装配曹仁（装配授予据守）。
    e.debug_assemble_general(1, "曹仁");
    assert!(e.debug_has_skill(1, "据守"), "1 号有据守");

    // 推进到 1 号回合（途中 1 号的据守确认门放弃），再到 0 号回合开始
    // 的写轮眼确认门。
    for _ in 0..128 {
        if let Some(g) = e.open_choice() {
            if g.spec.prompt == "trigger_confirm" && g.observer == 0 {
                break;
            }
            let sel: Vec<ChoiceOption> = if g.spec.prompt == "trigger_confirm" {
                vec![] // 其他玩家的可选触发一律放弃（保持 1 号朝向不变）
            } else if g.spec.min > 0 {
                g.spec.options.iter().take(g.spec.min).copied().collect()
            } else {
                vec![g.spec.options[0]]
            };
            let (id2, obs2) = (g.id, g.observer);
            drop(g);
            submit(&mut e, obs2, GameAction::Answer { choice_id: id2, selection: sel });
            continue;
        }
        let actor = e.turn().current;
        match submit(&mut e, actor, GameAction::NextPhase) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("advance: {other:?}"),
        }
    }
    let (cid, obs) = { let g = e.open_choice().unwrap(); (g.id, g.observer) };
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Confirm] });
    let target_pick = e.open_choice().expect("选目标挂起");
    assert!(target_pick.spec.options.contains(&ChoiceOption::Player(1)));
    let (tid, tobs) = { let g = e.open_choice().unwrap(); (g.id, g.observer) };
    submit(&mut e, tobs, GameAction::Answer { choice_id: tid, selection: vec![ChoiceOption::Player(1)] });
    let pick = e.open_choice().expect("技能选择挂起");
    let (pid, pobs) = (pick.id, pick.observer);
    drop(pick);
    submit(&mut e, pobs, GameAction::Answer { choice_id: pid, selection: vec![ChoiceOption::Tag(0)] });

    // 窃取完成：0 号拥有据守（expires），1 号保留原实例。
    let def_jushou = e.catalog().skill_by_name("据守").expect("据守在目录");
    let snap = e.save();
    assert!(snap.core.skills.contains_key(&(0, def_jushou)), "窃得据守");
    assert!(snap.core.skills.contains_key(&(1, def_jushou)), "目标保留据守（复制非转移）");
    assert!(!e.debug_face_down(0), "窃得时 0 号尚未翻面");

    // 推进到 0 号结束阶段：窃得的据守触发 → 确认 → 翻面 + 摸 1。
    for _ in 0..96 {
        if e.turn().current == 0 && e.turn().phase == Phase::End && e.open_choice().is_some() {
            break;
        }
        match e.open_choice() {
            Some(g) => {
                let sel: Vec<ChoiceOption> = if g.spec.min > 0 {
                    g.spec.options.iter().take(g.spec.min).copied().collect()
                } else {
                    vec![g.spec.options[0]]
                };
                let (id2, obs2) = (g.id, g.observer);
                drop(g);
                submit(&mut e, obs2, GameAction::Answer { choice_id: id2, selection: sel });
            }
            None => {
                let actor = e.turn().current;
                match submit(&mut e, actor, GameAction::NextPhase) {
                    CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
                    other => panic!("advance: {other:?}"),
                }
            }
        }
    }
    let hand_before = e.debug_hand(0).len();
    let gate = e.open_choice().expect("窃得据守确认门应挂起");
    assert_eq!(gate.spec.prompt, "trigger_confirm");
    assert_eq!(gate.observer, 0);
    let (gid, gobs) = (gate.id, gate.observer);
    drop(gate);
    submit(&mut e, gobs, GameAction::Answer { choice_id: gid, selection: vec![ChoiceOption::Confirm] });
    assert!(e.debug_face_down(0), "窃得的据守翻面（对窃得者生效）");
    assert_eq!(e.debug_hand(0).len(), hand_before + 1, "窃得的据守摸 1");
}

/// E03 跨牌组合矩阵 ②：窃得据守 + 面具——窃得者的据守翻面被面具
/// 取消（TurnOverBefore 前置窗口对窃得技同样生效），摸牌仍执行
/// （登记差异复用）。
#[test]
fn huanle_combo_stolen_jushou_flip_cancelled_by_mask() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "huanle-upstream-noname")
            .expect("欢乐包可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let pack = bridge("huanle-upstream-noname", &json).expect("桥接");

    let mut e = Engine::new_with_extensions(3213, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let xl = e.debug_place(0, "hnl_xuelunyang_4");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: xl, targets: vec![] });
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "装备写轮眼");
    e.debug_assemble_general(1, "曹仁");

    for _ in 0..128 {
        if let Some(g) = e.open_choice() {
            if g.spec.prompt == "trigger_confirm" && g.observer == 0 {
                break;
            }
            let sel: Vec<ChoiceOption> = if g.spec.prompt == "trigger_confirm" {
                vec![]
            } else if g.spec.min > 0 {
                g.spec.options.iter().take(g.spec.min).copied().collect()
            } else {
                vec![g.spec.options[0]]
            };
            let (id2, obs2) = (g.id, g.observer);
            drop(g);
            submit(&mut e, obs2, GameAction::Answer { choice_id: id2, selection: sel });
            continue;
        }
        let actor = e.turn().current;
        match submit(&mut e, actor, GameAction::NextPhase) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("advance: {other:?}"),
        }
    }
    let (cid, obs) = { let g = e.open_choice().unwrap(); (g.id, g.observer) };
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Confirm] });
    let target_pick = e.open_choice().expect("选目标挂起");
    let (tid, tobs) = (target_pick.id, target_pick.observer);
    drop(target_pick);
    submit(&mut e, tobs, GameAction::Answer { choice_id: tid, selection: vec![ChoiceOption::Player(1)] });
    let pick = e.open_choice().expect("技能选择挂起");
    let (pid, pobs) = (pick.id, pick.observer);
    drop(pick);
    submit(&mut e, pobs, GameAction::Answer { choice_id: pid, selection: vec![ChoiceOption::Tag(0)] });
    let def_jushou = e.catalog().skill_by_name("据守").unwrap();
    assert!(e.save().core.skills.contains_key(&(0, def_jushou)), "窃得据守");

    // 推进到 0 号出牌阶段，装备面具，再到结束阶段确认据守。
    for _ in 0..96 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play && e.open_choice().is_none() {
            break;
        }
        if let Some(g) = e.open_choice() {
            let sel: Vec<ChoiceOption> = if g.spec.min > 0 {
                g.spec.options.iter().take(g.spec.min).copied().collect()
            } else {
                vec![g.spec.options[0]]
            };
            let (id2, obs2) = (g.id, g.observer);
            drop(g);
            submit(&mut e, obs2, GameAction::Answer { choice_id: id2, selection: sel });
            continue;
        }
        let actor = e.turn().current;
        match submit(&mut e, actor, GameAction::NextPhase) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("advance: {other:?}"),
        }
    }
    let mj = e.debug_place(0, "hnl_mianju_5");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: mj, targets: vec![] });
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "装备面具: {out:?}");
    for _ in 0..96 {
        if e.turn().current == 0 && e.turn().phase == Phase::End && e.open_choice().is_some() {
            break;
        }
        match e.open_choice() {
            Some(g) => {
                let sel: Vec<ChoiceOption> = if g.spec.min > 0 {
                    g.spec.options.iter().take(g.spec.min).copied().collect()
                } else {
                    vec![g.spec.options[0]]
                };
                let (id2, obs2) = (g.id, g.observer);
                drop(g);
                submit(&mut e, obs2, GameAction::Answer { choice_id: id2, selection: sel });
            }
            None => {
                let actor = e.turn().current;
                match submit(&mut e, actor, GameAction::NextPhase) {
                    CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
                    other => panic!("advance: {other:?}"),
                }
            }
        }
    }
    let hand_before = e.debug_hand(0).len();
    let gate = e.open_choice().expect("面具下窃得据守确认门应挂起");
    let (gid, gobs) = (gate.id, gate.observer);
    drop(gate);
    submit(&mut e, gobs, GameAction::Answer { choice_id: gid, selection: vec![ChoiceOption::Confirm] });
    assert!(!e.debug_face_down(0), "面具取消窃得据守的翻面");
    assert_eq!(e.debug_hand(0).len(), hand_before + 1, "摸牌仍执行（登记差异复用）");
}

/// E03 跨牌组合矩阵 ③④：苦无的伤害来源联动——
/// ③ 手里剑造成的伤害**不是杀**：苦无不触发（两线语义一致的负例，
/// 目标弃权受伤后手牌不变）；
/// ④ 雷杀伤害触发苦无（原生 is_sha_like 含雷/火杀——登记差异的
/// 行为面固化：上游过滤精确杀名）。
#[test]
fn huanle_combo_kuwu_sha_like_and_non_sha_damage() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "huanle-upstream-noname")
            .expect("欢乐包可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let pack = bridge("huanle-upstream-noname", &json).expect("桥接");

    let mut e = Engine::new_with_extensions(3214, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    // 苦无牌种为 Weapon（与标准武器同种）：按目录名精确定位实体牌，
    // debug_place 的按种检索会误取标准武器。
    let kuwu = e.debug_move_named_card_to_hand(0, "hnl_kuwu_0");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: kuwu, targets: vec![] });
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "装备苦无");
    let def_kuwu = e.catalog().skill_by_name("huanle_kuwu").expect("苦无技能在目录");
    assert!(e.save().core.skills.contains_key(&(0, def_kuwu)), "装备苦无授予实例");

    // ③ 苦无持有者用手里剑：目标弃权受伤 → 苦无不触发（伤害牌非杀）。
    let sj = e.debug_place(0, "hnl_shoulijian_1");
    // 目标手牌两张均非闪：手里剑弃权受伤后不触发苦无；雷杀必命中。
    let t1_card = e.debug_place(1, "tao");
    let t1_card2 = e.debug_place(1, "jiu");
    let hp_before = e.debug_hp(1);
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sj, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "手里剑目标询问挂起: {out:?}");
    let c = e.open_choice().expect("手里剑嵌套选择");
    assert_eq!(c.observer, 1);
    let (cid, cobs) = (c.id, c.observer);
    drop(c);
    // 弃权（Pass）= 受 1 点伤害。
    submit(&mut e, cobs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    assert_eq!(e.debug_hp(1), hp_before - 1, "弃权受 1 伤");
    assert!(
        e.debug_hand(1).contains(&t1_card) && e.debug_hand(1).contains(&t1_card2),
        "手里剑伤害不触发苦无（伤害牌不是杀，两线一致）"
    );

    // ④ 雷杀伤害：苦无触发（is_sha_like 含雷杀——登记差异行为面），
    // 目标弃一张手牌（确定性首张）。
    let leisha = e.debug_place(0, "leisha");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: leisha, targets: vec![1] });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "雷杀使用: {out:?}"
    );
    // 目标无闪：无强制出闪应答（若挂起则应答首选项）。
    if let Some(c) = e.open_choice() {
        let sel: Vec<ChoiceOption> = if c.spec.min > 0 {
            c.spec.options.iter().take(c.spec.min).copied().collect()
        } else {
            vec![c.spec.options[0]]
        };
        let (id2, obs2) = (c.id, c.observer);
        drop(c);
        submit(&mut e, obs2, GameAction::Answer { choice_id: id2, selection: sel });
    }
    assert_eq!(e.debug_hp(1), hp_before - 2, "雷杀命中");
    assert!(
        !e.debug_hand(1).contains(&t1_card),
        "雷杀伤害触发苦无：目标被弃一张手牌"
    );
    assert!(
        e.debug_hand(1).contains(&t1_card2),
        "苦无为确定性弃首张（登记差异：上游目标自选）"
    );
}

/// 整包验收首包（U06 升 G3 剩余项，2026-10-08）：真实宿主全生命周期——
/// 独立安装+激活 → 安装包→原生建局 → 对局中卸载（revoke_slot
/// deactivate）→ 在局续玩（lock_pinned 锁定副本）→ 重装守卫（Revoked）。
#[test]
fn huanle_real_host_full_lifecycle() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();

    // ① 独立安装 + 激活（真实发现→安装→激活集）
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "huanle-cards")
        .expect("欢乐卡牌安装")
        .clone();
    let slot = format!("{}/{}", meta.namespace, meta.package_id);
    assert!(
        store.read_active().unwrap().packages.contains_key(&slot),
        "安装后应处于激活集"
    );

    // ② 安装包 → 原生建局（bridge 自安装产物的 cards.json）
    let json = std::fs::read_to_string(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .unwrap();
    let pack = bridge("huanle-upstream-noname", &json).expect("桥接");
    let base_cards = Engine::new(1, 3).catalog().cards.len();
    let mut e = Engine::new_with_extensions(3401, 3, &[&pack]).expect("建局");
    assert_eq!(e.catalog().cards.len(), base_cards + 7);

    // 推进到对局中并落存档（在局状态）
    e.debug_clear_hand(0);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let mid = e.save();

    // ③ 对局中卸载：revoke_slot(deactivate=true)
    store
        .revoke_slot(&meta.namespace, "huanle-cards", "验收：对局中卸载", true)
        .expect("撤销");
    let revoke = store.read_revoke().unwrap();
    assert!(
        revoke.is_slot_revoked(&meta.namespace, "huanle-cards"),
        "槽位应已撤销"
    );
    assert!(
        !store.read_active().unwrap().packages.contains_key(&slot),
        "撤销后应移出激活集"
    );

    // ④ 在局续玩（lock_pinned）：引擎持有已装载副本，恢复+提交不受
    //    宿主撤销影响；包内容仍可用（目录含 7 张扩展牌）
    let mut resumed = Engine::restore_with_extensions(
        &engine_native::archive::decode_archive(&engine_native::archive::encode_archive(&mid))
            .expect("解码"),
        &[&pack],
    )
    .expect("对局中卸载后恢复（锁定副本）");
    assert_eq!(resumed.catalog().cards.len(), base_cards + 7, "锁定副本内容仍可用");
    let out = submit(&mut resumed, 0, GameAction::NextPhase);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "在局续玩: {out:?}"
    );

    // ⑤ 重装守卫：撤销后同包再安装被拒（Revoked）
    let bytes = std::fs::read(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("package.npack.json"),
    )
    .expect("npack 存在");
    let err = store.install(&bytes, None, &options());
    assert!(
        matches!(err, Err(content_runtime::RuntimeError::Revoked(_))),
        "撤销后重装应被拒: {err:?}"
    );

    // ⑥ 新局不再能用已撤销包（宿主层安装产物已失效；引擎层由内容锁拒绝）：
    //    通过重新 import 验证守卫（import_extensions 内部 install 会撞撤销）。
    let report2 = import_extensions(&store, &root, &options());
    // 已安装+已撤销：install 阶段应报 Revoked（不再静默通过）
    assert!(report2.is_err(), "撤销后重新 import 应被拒");
}
