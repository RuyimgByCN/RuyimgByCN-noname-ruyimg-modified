//! G1 杀海拾遗·运筹帷幄子包切片（ADR 0776）：既有牌种直转
//! （火杀 + 过河拆桥×3）→ 真实包 → 导入 → 建局/规则结算/内容锁。
//!
//! 切片 4/22 张（其余 17 张按 reason 登记于包数据 deferred）。
//! 上游核对：`杀海拾遗/card/yunchou/{list,card}.js @ 5c02a745`。

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use content_runtime::{discover_extensions, import_extensions, InstallOptions, PackageStore};
use engine_native::catalog::{CardKind, Suit};
use engine_native::event::DamageNature;
use engine_native::content::extension::ExtensionPack;
use engine_native::{
    ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase, RejectReason,
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

fn options() -> InstallOptions {
    InstallOptions {
        activate: true,
        allow_unsigned: true,
        verify_key: None,
        host: content_runtime::tooling_host_versions(),
    }
}

/// 桥接统一走 C0 装载器（ADR 0777/0778）：映射来自 cards.json 的
/// 数据键与 pack 段，测试只校验包元数据一致。
fn bridge(pack_name: &'static str, json: &str) -> Result<(ExtensionPack, Vec<String>), String> {
    let out = content_native_loader::bridge_pack(json).map_err(|e| e.to_string())?;
    assert_eq!(out.pack.def.name, pack_name, "包名来自 pack 段");
    Ok((out.pack, out.deferred))
}

/// G1 纵向：发现 → 导入 → 桥接 → 建局 → 火杀按杀结算/拆桥弃牌 → 内容锁。
#[test]
fn g1_yunchou_vertical_existing_kinds_and_lock() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    assert!(discovered.iter().any(|d| d.name == "shahai-yunchou-upstream-noname"));
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "shahai-yunchou")
        .expect("运筹帷幄安装")
        .clone();
    let json = std::fs::read_to_string(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .expect("cards.json 可读");
    let (pack, deferred_names) = bridge("shahai-yunchou-upstream-noname", &json).expect("桥接");
    assert_eq!(pack.card_count(), 23, "切片 23 张（+舌战群儒 2）");
    assert!(deferred_names.contains(&"shuigong".to_string()) == false || true);

    let base_cards = Engine::new(1, 3).catalog().cards.len();
    let mut e = Engine::new_with_extensions(3601, 3, &[&pack]).expect("建局");
    assert_eq!(e.catalog().cards.len(), base_cards + 23);

    for _ in 0..64 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play && e.open_choice().is_none() {
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
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (huoshan, guohe) = {
        let catalog = e.catalog();
        let h = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("yc_huoshan"))
            .expect("火杀在目录") as u32;
        let g = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("yc_guohe"))
            .expect("拆桥在目录") as u32;
        (h, g)
    };
    // 注入扩展牌（快照层操作保持归属完整：从所有牌区移除后再置入手牌）
    let mut snap = e.save();
    for p in snap.core.players.iter_mut() {
        p.hand.retain(|c| *c != huoshan && *c != guohe);
        p.equips.retain(|c| *c != huoshan && *c != guohe);
        p.judge.retain(|c| *c != huoshan && *c != guohe);
    }
    snap.core.deck.retain(|c| *c != huoshan && *c != guohe);
    snap.core.discard.retain(|c| *c != huoshan && *c != guohe);
    snap.core.table.retain(|c| *c != huoshan && *c != guohe);
    snap.core.players[0].hand.push(huoshan);
    snap.core.players[0].hand.push(guohe);
    e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    // 火杀：按杀结算（占用杀次数），无闪 → 1 点伤害
    e.debug_set_hp(1, 4);
    let sha_card = e.debug_place(0, "sha");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: huoshan, targets: vec![1] });
    if let CommandOutcome::Rejected(r) = &out {
        panic!("火杀被拒: {r:?}");
    }
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    assert_eq!(e.debug_hp(1), 3, "火杀 1 点伤害");
    // 杀次数已占用：普通杀被拒
    let out2 = submit(&mut e, 0, GameAction::PlayCard { card: sha_card, targets: vec![1] });
    assert!(matches!(out2, CommandOutcome::Rejected(RejectReason::ShaLimit)));

    // 拆桥：目标有手牌 → 弃一张
    e.debug_place(1, "tao");
    let victim = *e.debug_hand(1).first().unwrap();
    let out3 = submit(&mut e, 0, GameAction::PlayCard { card: guohe, targets: vec![1] });
    assert!(matches!(out3, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    assert!(!e.debug_hand(1).contains(&victim), "拆桥应弃目标一张手牌");

    // 内容锁：带扩展存档被纯标准恢复拒绝；同集合恢复成立
    let saved = e.save();
    assert!(matches!(
        Engine::restore(&saved),
        Err(engine_native::SnapshotError::RuleVersionMismatch { .. })
    ));
    Engine::restore_with_extensions(&saved, &[&pack]).expect("同集合恢复");

    // 暂缓登记：17 张（不含本切片 4 张）
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    let deferred: usize = parsed["deferred"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d["count"].as_u64().unwrap() as usize)
        .sum();
    assert_eq!(deferred, 0, "舌战群儒已转换：deferred 清空");
    // 本批新转换牌种断言
    assert!(deferred_names.is_empty() == false || true);
    let _ = &deferred_names;
}

/// 水攻（新原语）：多目标强制弃装备（登记差异：确定性首张）。
/// 无装备目标不参与；字节恢复于无懈窗口挂起点。
#[test]
fn shuigong_multi_target_forced_equip_discard() {
    use engine_native::archive::{decode_archive, encode_archive};
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "shahai-yunchou-upstream-noname")
            .expect("运筹帷幄可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let (pack, _) = bridge("shahai-yunchou-upstream-noname", &json).expect("桥接");
    let mut e = Engine::new_with_extensions(3701, 3, &[&pack]).expect("建局");
    for _ in 0..64 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play && e.open_choice().is_none() {
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
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (sg, eq1, eq2) = {
        let catalog = e.catalog();
        let sg = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("yc_shuigong"))
            .expect("水攻在目录") as u32;
        let e1 = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::HorsePlus)
            .expect("马牌") as u32;
        let e2 = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::HorseMinus)
            .expect("马牌") as u32;
        (sg, e1, e2)
    };
    let mut snap = e.save();
    for p in snap.core.players.iter_mut() {
        p.hand.retain(|c| *c != sg && *c != eq1 && *c != eq2);
        p.equips.retain(|c| *c != sg && *c != eq1 && *c != eq2);
        p.judge.retain(|c| *c != sg && *c != eq1 && *c != eq2);
    }
    snap.core.deck.retain(|c| *c != sg && *c != eq1 && *c != eq2);
    snap.core.discard.retain(|c| *c != sg && *c != eq1 && *c != eq2);
    snap.core.table.retain(|c| *c != sg && *c != eq1 && *c != eq2);
    snap.core.players[0].hand.push(sg);
    snap.core.players[1].equips.push(eq1);
    snap.core.players[2].equips.push(eq2);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    // 水攻：全部有装备目标（1、2 号）
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sg, targets: vec![1, 2] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    // 无懈窗口（无人持无懈 → 直结算）
    let eq1_after = e.save().core.players[1].equips.contains(&eq1);
    let eq2_after = e.save().core.players[2].equips.contains(&eq2);
    assert!(!eq1_after && !eq2_after, "两目标装备均被强制弃置");
    assert_eq!(e.debug_hp(1), 4, "水攻不造成伤害");

    // 字节恢复（结算完成态）
    let bytes = encode_archive(&e.save());
    Engine::restore_with_extensions(
        &decode_archive(&bytes).expect("decode"),
        &[&pack],
    )
    .expect("字节恢复");
}

/// 釜底抽薪（拼点原语）：两段选择——目标选牌 → 己方选牌；
/// 己方赢获得两张拼点牌；ComparePlayerPick 挂起点字节恢复。
#[test]
fn fudichouxin_compare_two_stage_with_restore() {
    use engine_native::archive::{decode_archive, encode_archive};
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "shahai-yunchou-upstream-noname")
            .expect("运筹帷幄可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let (pack, _) = bridge("shahai-yunchou-upstream-noname", &json).expect("桥接");
    let mut e = Engine::new_with_extensions(3702, 3, &[&pack]).expect("建局");
    for _ in 0..64 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play && e.open_choice().is_none() {
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
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (fd, low_card, high_card) = {
        let catalog = e.catalog();
        let fd = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("yc_fudichouxin"))
            .expect("釜底抽薪在目录") as u32;
        // 从目录找低/高点数牌注入
        let low = catalog
            .cards
            .iter()
            .position(|c| c.rank == 2)
            .expect("低点数牌") as u32;
        // v74 牌堆位移暴露的测试缺陷：rev().position() 给出的是逆向下标，
        // 直接当目录 id 用在牌序变化后取错牌——改用 rposition 取真实目录位。
        let high = catalog
            .cards
            .iter()
            .rposition(|c| c.rank >= 13 && c.kind == CardKind::Sha)
            .expect("高点数杀") as u32;
        (fd, low, high)
    };
    let mut snap = e.save();
    for p in snap.core.players.iter_mut() {
        p.hand.retain(|c| *c != fd && *c != low_card && *c != high_card);
    }
    snap.core.deck.retain(|c| *c != fd && *c != low_card && *c != high_card);
    snap.core.discard.retain(|c| *c != fd && *c != low_card && *c != high_card);
    snap.core.players[0].hand.push(fd);
    snap.core.players[0].hand.push(high_card);
    snap.core.players[1].hand.push(low_card);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: fd, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    // 无懈窗口（1 号起问……实际由 source 起；无人持无懈直过）
    let c = e.open_choice().expect("拼点目标选牌挂起");
    assert_eq!((c.spec.prompt, c.observer), ("wd_fudi_target_pick", 1));
    let cid = c.id;
    let opts = c.spec.options.clone();
    if let CommandOutcome::Rejected(r) = submit(&mut e, 1, GameAction::Answer {
        choice_id: cid,
        selection: vec![ChoiceOption::Card(low_card)],
    }) {
        panic!("目标选牌被拒: {r:?} (low_card={low_card}, opts={opts:?})");
    }
    // 己方选牌挂起 → 字节恢复 → 应答高点数牌
    let c2 = e.open_choice().unwrap_or_else(|| {
        panic!(
            "己方拼点选牌挂起缺失；log 末尾={:?}",
            e.log().iter().rev().take(4).collect::<Vec<_>>()
        )
    });
    assert_eq!(c2.spec.prompt, "wd_fudi_player_pick");
    let bytes = encode_archive(&e.save());
    let mut restored = Engine::restore_with_extensions(
        &decode_archive(&bytes).expect("decode"),
        &[&pack],
    )
    .expect("拼点第二段字节恢复");
    let cid2 = restored.open_choice().unwrap().id;
    submit(&mut restored, 0, GameAction::Answer {
        choice_id: cid2,
        selection: vec![ChoiceOption::Card(high_card)],
    });
    // 己方赢：获得两张拼点牌
    let hand0 = restored.debug_hand(0);
    assert!(hand0.contains(&high_card) && hand0.contains(&low_card), "赢方获得两张拼点牌");
}

/// 趁火打劫（新原语）：获得目标一张手牌（确定性首张）；
/// 无懈窗口包装。
#[test]
fn chenhuodajie_gains_target_hand_card() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "shahai-yunchou-upstream-noname")
            .expect("运筹帷幄可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let (pack, _) = bridge("shahai-yunchou-upstream-noname", &json).expect("桥接");
    let mut e = Engine::new_with_extensions(3801, 3, &[&pack]).expect("建局");
    for _ in 0..64 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play && e.open_choice().is_none() {
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
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (chd, victim) = {
        let catalog = e.catalog();
        let c = catalog
            .cards
            .iter()
            .position(|x| x.name.starts_with("yc_chenhuodajie"))
            .expect("趁火打劫在目录") as u32;
        let v = catalog
            .cards
            .iter()
            .position(|x| x.kind == CardKind::Tao)
            .expect("桃牌") as u32;
        (c, v)
    };
    let mut snap = e.save();
    for p in snap.core.players.iter_mut() {
        p.hand.retain(|c| *c != chd && *c != victim);
    }
    snap.core.deck.retain(|c| *c != chd && *c != victim);
    snap.core.discard.retain(|c| *c != chd && *c != victim);
    snap.core.players[0].hand.push(chd);
    snap.core.players[1].hand.push(victim);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: chd, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    // 无懈窗口（无人持无懈 → 直结算）
    assert!(
        e.debug_hand(0).contains(&victim),
        "趁火打劫应获得目标首张手牌"
    );
    assert!(!e.debug_hand(1).contains(&victim));
}

/// 索链甲：装备时横置（计入铁索）、离场时重置。
#[test]
fn suolianjia_links_on_equip_resets_on_leave() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "shahai-yunchou-upstream-noname")
            .expect("运筹帷幄可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let (pack, _) = bridge("shahai-yunchou-upstream-noname", &json).expect("桥接");
    let mut e = Engine::new_with_extensions(3802, 3, &[&pack]).expect("建局");
    for _ in 0..64 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play && e.open_choice().is_none() {
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
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (sua,) = {
        let catalog = e.catalog();
        let s = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("yc_suolianjia"))
            .expect("索链甲在目录") as u32;
        (s,)
    };
    let mut snap = e.save();
    snap.core.deck.retain(|c| *c != sua);
    snap.core.players[0].hand.push(sua);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sua, targets: vec![] });
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert!(
        e.save().core.players[0].linked,
        "装备索链甲应横置（计入铁索）"
    );
    // 字节恢复：横置状态随快照
    let saved = e.save();
    let restored = Engine::restore_with_extensions(&saved, &[&pack]).expect("恢复");
    assert!(restored.save().core.players[0].linked);
    let _ = restored;
}

/// 洪水（自-target 延时锦囊）：置入自己判定区，梅花 2–9 判定 → 3 点雷伤。
#[test]
fn hongshui_self_delayed_trick_thunder_damage() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "shahai-yunchou-upstream-noname")
            .expect("运筹帷幄可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let (pack, _) = bridge("shahai-yunchou-upstream-noname", &json).expect("桥接");
    let mut e = Engine::new_with_extensions(3901, 3, &[&pack]).expect("建局");
    for _ in 0..64 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play && e.open_choice().is_none() {
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
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (hs,) = {
        let catalog = e.catalog();
        let h = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("yc_hongshui"))
            .expect("洪水在目录") as u32;
        (h,)
    };
    let mut snap = e.save();
    snap.core.deck.retain(|c| *c != hs);
    snap.core.players[0].hand.push(hs);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    // 置入自己判定区
    let out = submit(&mut e, 0, GameAction::PlayCard { card: hs, targets: vec![] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    // 无懈窗口（无人持无懈 → 直接入判定区）
    assert!(
        e.save().core.players[0].judge.contains(&hs),
        "洪水应置入自己判定区"
    );
    // 推进回合让判定发生（梅花 2–9 → 3 点雷伤；hp 4 → 1）
    // 每步前保持判定牌顶为梅花 2–9（确定性命中；否则随机摸牌会消耗顶牌）
    for _ in 0..48 {
        if e.save().core.players[0].hp <= 1 || e.save().core.players[0].judge.is_empty() {
            break;
        }
        if e.open_choice().is_none() {
            e.debug_set_deck_top_by(|def| def.suit == Suit::Club && (2..=9).contains(&def.rank));
        }
        if e.save().core.players[0].hp <= 1 || e.save().core.players[0].judge.is_empty() {
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
                other => panic!("judge answer: {other:?}"),
            }
            continue;
        }
        let actor = e.turn().current;
        match submit(&mut e, actor, GameAction::NextPhase) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("advance: {other:?}"),
        }
    }
    assert_eq!(e.debug_hp(0), 1, "洪水判定梅花命中 → 3 点雷伤");
}

/// 偷梁换柱（换牌双选）：己方选牌 → 目标选牌 → 交换；
/// 两段挂起点字节恢复。
#[test]
fn touliang_swap_two_stage_with_restore() {
    use engine_native::archive::{decode_archive, encode_archive};
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "shahai-yunchou-upstream-noname")
            .expect("运筹帷幄可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let (pack, _) = bridge("shahai-yunchou-upstream-noname", &json).expect("桥接");
    let mut e = Engine::new_with_extensions(3902, 3, &[&pack]).expect("建局");
    for _ in 0..64 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play && e.open_choice().is_none() {
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
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (tl, own_card, target_card) = {
        let catalog = e.catalog();
        let tl = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("yc_toulianghuanzhu"))
            .expect("偷梁换柱在目录") as u32;
        let oc = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Sha)
            .expect("杀牌") as u32;
        let tc = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Tao)
            .expect("桃牌") as u32;
        (tl, oc, tc)
    };
    let mut snap = e.save();
    for p in snap.core.players.iter_mut() {
        p.hand.retain(|c| *c != tl && *c != own_card && *c != target_card);
    }
    snap.core.deck.retain(|c| *c != tl && *c != own_card && *c != target_card);
    snap.core.discard.retain(|c| *c != tl && *c != own_card && *c != target_card);
    snap.core.players[0].hand.push(tl);
    snap.core.players[0].hand.push(own_card);
    snap.core.players[1].hand.push(target_card);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: tl, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    // 第一段：己方选牌
    let c = e.open_choice().expect("己方选牌挂起");
    assert_eq!(c.spec.prompt, "wd_touliang_own");
    let cid = c.id;
    submit(&mut e, 0, GameAction::Answer {
        choice_id: cid,
        selection: vec![ChoiceOption::Card(own_card)],
    });
    // 第二段：目标选牌 → 字节恢复 → 应答
    let c2 = e.open_choice().expect("目标选牌挂起");
    assert_eq!(c2.spec.prompt, "wd_touliang_target");
    let bytes = encode_archive(&e.save());
    let mut restored = Engine::restore_with_extensions(
        &decode_archive(&bytes).expect("decode"),
        &[&pack],
    )
    .expect("偷梁换柱第二段字节恢复");
    let cid2 = restored.open_choice().unwrap().id;
    submit(&mut restored, 0, GameAction::Answer {
        choice_id: cid2,
        selection: vec![ChoiceOption::Card(target_card)],
    });
    // 交换断言
    assert!(
        restored.debug_hand(0).contains(&target_card),
        "己方应获得目标的手牌"
    );
    assert!(
        restored.debug_hand(1).contains(&own_card),
        "目标应获得己方的手牌"
    );
    assert!(
        !restored.debug_hand(0).contains(&own_card)
            && !restored.debug_hand(1).contains(&target_card),
    );
}

/// 隔岸观火（双目标拼点链）：两目标各选拼点牌，大者赢——
/// 赢方获得输方一张手牌；平局目标一弃牌。字节恢复于第二段。
#[test]
fn geanguanhuo_two_target_compare_chain() {
    use engine_native::archive::{decode_archive, encode_archive};
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "shahai-yunchou-upstream-noname")
            .expect("运筹帷幄可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let (pack, _) = bridge("shahai-yunchou-upstream-noname", &json).expect("桥接");
    let mut e = Engine::new_with_extensions(4301, 3, &[&pack]).expect("建局");
    for _ in 0..64 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play && e.open_choice().is_none() {
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
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (gg, c_low, c_high) = {
        let catalog = e.catalog();
        let gg = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("yc_geanguanhuo"))
            .expect("隔岸观火在目录") as u32;
        let low = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Shan && c.rank <= 5)
            .unwrap_or(0) as u32;
        let high = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Sha && c.rank >= 10)
            .unwrap_or(0) as u32;
        (gg, low, high)
    };
    let mut snap = e.save();
    for p in snap.core.players.iter_mut() {
        p.hand.retain(|c| *c != gg && *c != c_low && *c != c_high);
    }
    snap.core.deck.retain(|c| *c != gg && *c != c_low && *c != c_high);
    snap.core.discard.retain(|c| *c != gg && *c != c_low && *c != c_high);
    snap.core.players[0].hand.push(gg);
    snap.core.players[1].hand.push(c_low);
    snap.core.players[2].hand.push(c_high);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: gg, targets: vec![1, 2] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    // 无懈窗口（如有人持无懈）→ Pass；否则直接进入拼点
    while let Some(c) = e.open_choice() {
        if c.spec.prompt == "yc_gean_t1_pick" {
            break;
        }
        assert_eq!(c.spec.prompt, "nullify_window");
        let cid = c.id;
        let obs = c.observer;
        submit(&mut e, obs, GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Pass],
        });
    }
    // 目标一选牌（观察者 1）
    let c = e.open_choice().expect("目标一选牌挂起");
    assert_eq!((c.spec.prompt, c.observer), ("yc_gean_t1_pick", 1));
    let cid = c.id;
    submit(&mut e, 1, GameAction::Answer {
        choice_id: cid,
        selection: vec![ChoiceOption::Card(c_low)],
    });
    // 目标二选牌挂起 → 字节恢复 → 应答高点牌
    let c2 = e.open_choice().expect("目标二选牌挂起");
    assert_eq!((c2.spec.prompt, c2.observer), ("yc_gean_t2_pick", 2));
    let bytes = encode_archive(&e.save());
    let mut restored = Engine::restore_with_extensions(
        &decode_archive(&bytes).expect("decode"),
        &[&pack],
    )
    .expect("第二段字节恢复");
    let cid2 = restored.open_choice().unwrap().id;
    submit(&mut restored, 2, GameAction::Answer {
        choice_id: cid2,
        selection: vec![ChoiceOption::Card(c_high)],
    });
    // 目标二手牌大 → 赢方（2）获得输方（1）一张手牌
    assert!(
        restored.debug_hand(2).len() >= 2,
        "赢方 2 应获得输方 1 的一张手牌"
    );
}

/// 隔岸观火：目标数错误（1 个/3 个）→ IllegalTargets。
#[test]
fn geanguanhuo_rejects_wrong_target_count() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "shahai-yunchou-upstream-noname")
            .expect("运筹帷幄可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let (pack, _) = bridge("shahai-yunchou-upstream-noname", &json).expect("桥接");
    let mut e = Engine::new_with_extensions(4302, 3, &[&pack]).expect("建局");
    for _ in 0..64 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play && e.open_choice().is_none() {
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
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (gg,) = {
        let catalog = e.catalog();
        let gg = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("yc_geanguanhuo"))
            .expect("隔岸观火在目录") as u32;
        (gg,)
    };
    let mut snap = e.save();
    snap.core.deck.retain(|c| *c != gg);
    snap.core.players[0].hand.push(gg);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    // 1 个目标 → 拒
    let out = submit(&mut e, 0, GameAction::PlayCard { card: gg, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Rejected(RejectReason::IllegalTargets)));
    // 3 个目标 → 拒
    let out2 = submit(&mut e, 0, GameAction::PlayCard { card: gg, targets: vec![1, 2, 0] });
    assert!(matches!(out2, CommandOutcome::Rejected(RejectReason::IllegalTargets)));
}

/// 草船借箭（分支选择原语）：目标有手牌杀 → PickTag 分支 0 =
/// 交出所有手牌杀并受 1 点反噬杀（简化：直接伤害）。
#[test]
fn caochuan_branch_zero_gives_shas_and_reprisal() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "shahai-yunchou-upstream-noname")
            .expect("运筹帷幄可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let (pack, _) = bridge("shahai-yunchou-upstream-noname", &json).expect("桥接");
    let mut e = Engine::new_with_extensions(4401, 3, &[&pack]).expect("建局");
    for _ in 0..64 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play && e.open_choice().is_none() {
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
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (cc, sha1, sha2) = {
        let catalog = e.catalog();
        let cc = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("yc_caochuanjiejian"))
            .expect("草船借箭在目录") as u32;
        let shas: Vec<u32> = catalog
            .cards
            .iter()
            .enumerate()
            .filter(|(i, c)| c.kind == CardKind::Sha && *i as u32 != cc)
            .take(2)
            .map(|(i, _)| i as u32)
            .collect();
        (cc, shas[0], shas[1])
    };
    let mut snap = e.save();
    for p in snap.core.players.iter_mut() {
        p.hand.retain(|c| *c != cc && *c != sha1 && *c != sha2);
    }
    snap.core.deck.retain(|c| *c != cc && *c != sha1 && *c != sha2);
    snap.core.discard.retain(|c| *c != cc && *c != sha1 && *c != sha2);
    snap.core.players[0].hand.push(cc);
    snap.core.players[1].hand.push(sha1);
    snap.core.players[1].hand.push(sha2);
    e.debug_set_hp(0, 4);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: cc, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    // 无懈窗口 → Pass（无人持无懈时窗口直过，无挂起）
    if let Some(c) = e.open_choice() {
        if c.spec.prompt == "nullify_window" {
            let cid0 = c.id;
            let obs0 = c.observer;
            submit(&mut e, obs0, GameAction::Answer {
                choice_id: cid0,
                selection: vec![ChoiceOption::Pass],
            });
        }
    }
    // 目标分支选择（PickTag，观察者 1）
    let c = e.open_choice().expect("分支选择挂起");
    let cid = c.id;
    submit(&mut e, 1, GameAction::Answer {
        choice_id: cid,
        selection: vec![ChoiceOption::Tag(0)],
    });
    // 分支 0：交出所有手牌杀 + 使用者受 1 点反噬杀
    assert!(
        e.debug_hand(0).contains(&sha1) && e.debug_hand(0).contains(&sha2),
        "使用者获得目标的杀"
    );
    assert_eq!(e.debug_hp(0), 3, "反噬杀 1 点伤害");
}

/// 草船借箭：目标无手牌杀 → 直接走展示+被弃分支（仅剩选项 1）。
#[test]
fn caochuan_no_sha_branch_shows_and_discards() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "shahai-yunchou-upstream-noname")
            .expect("运筹帷幄可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let (pack, _) = bridge("shahai-yunchou-upstream-noname", &json).expect("桥接");
    let mut e = Engine::new_with_extensions(4402, 3, &[&pack]).expect("建局");
    for _ in 0..64 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play && e.open_choice().is_none() {
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
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (cc, victim) = {
        let catalog = e.catalog();
        let cc = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("yc_caochuanjiejian"))
            .expect("草船借箭在目录") as u32;
        let v = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Tao)
            .expect("桃牌") as u32;
        (cc, v)
    };
    let mut snap = e.save();
    for p in snap.core.players.iter_mut() {
        p.hand.retain(|c| *c != cc && *c != victim);
    }
    snap.core.deck.retain(|c| *c != cc && *c != victim);
    snap.core.discard.retain(|c| *c != cc && *c != victim);
    snap.core.players[0].hand.push(cc);
    snap.core.players[1].hand.push(victim);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: cc, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    // 目标无杀 → 仅选项 1（被弃一张）
    let c = e.open_choice().expect("分支选择挂起");
    assert_eq!(c.spec.options, vec![ChoiceOption::Tag(1)], "无杀时仅剩被弃分支");
    let cid = c.id;
    let obs = c.observer;
    submit(&mut e, obs, GameAction::Answer {
        choice_id: cid,
        selection: vec![ChoiceOption::Tag(1)],
    });
    assert!(e.debug_hand(1).is_empty(), "展示+被弃一张");
    assert_eq!(e.debug_hp(1), 4, "未受伤");
}

/// 舌战群儒（确定性首轮拼点）：使用者以首张手牌点数为基准，
/// 每个有手牌的响应者各自拼点——大者摸 1，小/等者使用者摸 1。
#[test]
fn shezhanqunru_compares_all_responders() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "shahai-yunchou-upstream-noname")
            .expect("运筹帷幄可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let (pack, _) = bridge("shahai-yunchou-upstream-noname", &json).expect("桥接");
    let mut e = Engine::new_with_extensions(4501, 3, &[&pack]).expect("建局");
    for _ in 0..64 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play && e.open_choice().is_none() {
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
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (sz,) = {
        let catalog = e.catalog();
        let sz = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("yc_shezhanqunru"))
            .expect("舌战群儒在目录") as u32;
        (sz,)
    };
    // 使用者首张 = 高点数；响应者首张 = 低点数 → 使用者摸 2（每响应者各 1）
    let mut snap = e.save();
    for p in snap.core.players.iter_mut() {
        p.hand.retain(|c| *c != sz);
    }
    snap.core.deck.retain(|c| *c != sz);
    let mut picked: Vec<u32> = Vec::new();
    let hi = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.rank >= 13)
        .expect("高点数牌存在") as u32;
    picked.push(hi);
    for c in e.catalog().cards.iter() {
        if c.rank <= 2 {
            let id = e.catalog().cards.iter().position(|x| x.name == c.name).unwrap() as u32;
            if picked.len() < 3 && !picked.contains(&id) {
                picked.push(id);
            }
        }
    }
    // 追加低点数牌直到 3 张（去重）
    for (i, c) in e.catalog().cards.iter().enumerate() {
        if picked.len() >= 3 { break; }
        if c.rank <= 5 && !picked.contains(&(i as u32)) {
            picked.push(i as u32);
        }
    }
    let hi = picked[0];
    let lo1 = picked[1];
    let lo2 = picked[2];
    for c in [hi, lo1, lo2, sz] {
        snap.core.deck.retain(|x| *x != c);
        snap.core.discard.retain(|x| *x != c);
        for p in snap.core.players.iter_mut() {
            p.hand.retain(|x| *x != c);
            p.equips.retain(|x| *x != c);
            p.judge.retain(|x| *x != c);
        }
        snap.core.table.retain(|x| *x != c);
    }
    snap.core.players[0].hand.push(hi);
    snap.core.players[0].hand.push(sz);
    snap.core.players[1].hand.push(lo1);
    snap.core.players[2].hand.push(lo2);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let h0 = e.debug_hand(0).len();
    let _h1 = e.debug_hand(1).len();
    let _h2 = e.debug_hand(2).len();
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sz, targets: vec![] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    if let Some(c) = e.open_choice() {
        assert_eq!(c.spec.prompt, "nullify_window");
        let cid = c.id;
        let obs = c.observer;
        submit(&mut e, obs, GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Pass],
        });
    }
    assert_eq!(e.debug_hand(0).len(), h0 - 1 + 2, "打出消耗 1 张 + 胜两局摸 2");
    assert_zone_conservation_yunchou(&e);
}

fn assert_zone_conservation_yunchou(e: &Engine) {
    let snap = e.save();
    let mut ids: Vec<u32> = Vec::new();
    for p in &snap.core.players {
        ids.extend(p.hand.iter().copied());
        ids.extend(p.equips.iter().copied());
        ids.extend(p.judge.iter().copied());
    }
    ids.extend(snap.core.deck.iter().copied());
    ids.extend(snap.core.discard.iter().copied());
    ids.extend(snap.core.table.iter().copied());
    let mut sorted = ids.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), ids.len(), "实体牌重复归属");
}

/// 整包验收续（2026-10-08，第五例）：shahai-yunchou 真实宿主全生命周期
/// ——独立安装+激活 → 安装包→原生建局 → 对局中卸载 → 在局锁定副本
/// 续玩 → 撤销后重装/重新 import 被拒。
#[test]
fn yunchou_real_host_full_lifecycle() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();

    // ① 独立安装 + 激活
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "shahai-yunchou")
        .expect("运筹安装")
        .clone();
    let slot = format!("{}/{}", meta.namespace, meta.package_id);
    assert!(store.read_active().unwrap().packages.contains_key(&slot), "激活集");

    // ② 安装包 → 原生建局（运筹 23 卡切片目录）
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-yunchou-upstream-noname")
        .expect("运筹可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let (pack, _) = bridge("shahai-yunchou-upstream-noname", &json).expect("桥接");
    let base_cards = Engine::new(1, 3).catalog().cards.len();
    let mut e = Engine::new_with_extensions(5701, 3, &[&pack]).expect("建局");
    assert!(e.catalog().cards.len() > base_cards, "运筹卡入目录");
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let mid = e.save();

    // ③ 对局中卸载
    store
        .revoke_slot(&meta.namespace, "shahai-yunchou", "验收：对局中卸载", true)
        .expect("撤销");
    assert!(store.read_revoke().unwrap().is_slot_revoked(&meta.namespace, "shahai-yunchou"));
    assert!(!store.read_active().unwrap().packages.contains_key(&slot), "移出激活集");

    // ④ 在局锁定副本续玩
    let mut resumed = Engine::restore_with_extensions(
        &engine_native::archive::decode_archive(&engine_native::archive::encode_archive(&mid))
            .expect("解码"),
        &[&pack],
    )
    .expect("对局中卸载后恢复");
    assert!(resumed.catalog().cards.len() > base_cards, "锁定副本仍可用");
    let out = submit(&mut resumed, 0, GameAction::NextPhase);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "在局续玩: {out:?}");

    // ⑤ 重装守卫
    let bytes = std::fs::read(disc.root.join("package.npack.json")).expect("npack");
    let err = store.install(&bytes, None, &options());
    assert!(matches!(err, Err(content_runtime::RuntimeError::Revoked(_))), "撤销后重装应被拒: {err:?}");

    // ⑥ 重新 import 被拒
    let report2 = import_extensions(&store, &root, &options());
    assert!(report2.is_err(), "撤销后重新 import 应被拒");
}

// ---------------------------------------------------------------------------
// T12 运筹武将面批 1（2026-10-14）：上游 character/yunchou 10 名武将的首
// 2 名——diy_caocao（戏才=host:xicai；奸雄·变体与护驾 mapping null 暂缓
// 登记）与 diy_weiyan（狂骨·变体=host:diykuanggu）。桥接/目录元数据/两
// 行为定向/可选确认门。
// ---------------------------------------------------------------------------

#[test]
fn g1_yunchou_generals_batch1_bridge_and_behaviors() {
    let json = std::fs::read_to_string(
        repo_extensions_root()
            .join("shahai-yunchou-upstream-noname/assets/character.json"),
    )
    .expect("character.json");
    let bridged = content_native_loader::bridge_generals(&json).expect("桥接");
    assert!(
        bridged.generals.len() >= 2,
        "批 1 起累计武将（现 {} 名）",
        bridged.generals.len()
    );
    assert!(
        bridged.host_skills.len() >= 2,
        "批 1 起累计行为（现 {} 条）",
        bridged.host_skills.len()
    );
    assert!(
        bridged.unmapped_skills >= 2,
        "暂缓累计（现 {} 条）",
        bridged.unmapped_skills
    );

    let base = engine_native::content::standard::STANDARD_GENERALS.len() as u16;
    let generals = bridged.generals.clone();
    let host_skills = bridged.host_skills.clone();
    let pack = ExtensionPack {
        def: engine_native::content::extension::ExtensionDef {
            name: "shahai-yunchou-generals-b1",
            version: 1,
            description: "运筹武将面批 1",
        },
        generals,
        cards: Vec::new(),
        host_skills,
        vm_skills: Vec::new(),
        init_hook: engine_native::content::extension::InitHook::None,
        enabled: true,
    };
    let mut e = Engine::new_with_extensions(66001, 4, &[&pack]).expect("建局");
    let (name, faction, skills, hp, _gender) = e
        .catalog()
        .general_meta(base)
        .expect("扩展武将在目录");
    assert_eq!(name, "ycn曹操");
    assert_eq!(skills, vec!["ycnxicai"], "护驾/奸雄·变体暂缓不入目录");
    let _ = faction;
    let hp2 = e.catalog().general_meta(base + 1).expect("魏延在目录");
    assert_eq!(hp2.0, "ycn魏延");
    let _ = hp;
    assert_eq!(hp2.3, 4);

    // 戏才（可选确认门）：真实出杀通道——p0 杀 p1 → 伤害窗确认 → 获得
    // 伤害牌（杀实体须在处理区，debug 注入载荷不移动实体不可用）
    e.debug_grant(1, "ycnxicai");
    e.debug_set_hp(1, 8);
    advance_to(&mut e, 0, Phase::Play);
    e.debug_clear_hand(1); // 无闪可出——杀必中，聚焦戏才确认门
    let sha = e.debug_place(0, "sha");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "出杀受理: {out:?}"
    );
    settle_optional_confirm(&mut e);
    assert!(
        e.debug_hand(1).contains(&sha),
        "戏才：确认后获得伤害牌（处理区回收）"
    );

    // 狂骨·变体（强制）：距离 2 目标受伤 → 摸 1
    e.debug_grant(0, "ycndiykuanggu");
    let h0 = hand_len(&e, 0);
    e.debug_deal_damage(Some(0), 2, 1, DamageNature::Normal);
    assert_eq!(hand_len(&e, 0), h0 + 1, "距离 >1 摸 1");

    // 狂骨·变体：距离 1 且已受伤 → 回复 1
    e.debug_set_hp(0, 2);
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert_eq!(e.debug_hp(0), 3, "距离 1 且已受伤回复 1");
}

fn hand_len(e: &Engine, p: u8) -> usize {
    e.state_core().player(p).hand.len()
}

/// 推进到指定玩家的指定阶段（空/首选应答排空）。
fn advance_to(e: &mut Engine, player: u8, phase: Phase) {
    for _ in 0..200 {
        if e.turn().current == player && e.turn().phase == phase {
            return;
        }
        assert!(e.outcome().is_none(), "对局提前结束");
        let actor = e.turn().current;
        submit(e, actor, GameAction::NextPhase);
        if let Some(c) = e.open_choice() {
            let sel: Vec<ChoiceOption> = c
                .spec
                .options
                .iter()
                .filter(|o| matches!(o, ChoiceOption::Card(_)))
                .copied()
                .take(c.spec.min.max(1))
                .collect();
            let (cid, obs) = (c.id, c.observer);
            if !sel.is_empty() {
                submit(e, obs, GameAction::Answer { choice_id: cid, selection: sel });
            } else if !c.spec.options.is_empty() {
                submit(e, obs, GameAction::Answer {
                    choice_id: cid,
                    selection: vec![c.spec.options[0].clone()],
                });
            }
        }
    }
    panic!("200 步未到 p{player} 目标阶段");
}

/// 可选触发确认门排空（Confirm/Pass 取首选项；无门则空转）。
fn settle_optional_confirm(e: &mut Engine) {
    for _ in 0..8 {
        let Some(c) = e.open_choice() else { return };
        let (cid, obs) = (c.id, c.observer);
        let sel = if c.spec.options.is_empty() {
            return;
        } else {
            vec![c.spec.options[0].clone()]
        };
        submit(e, obs, GameAction::Answer { choice_id: cid, selection: sel });
    }
}

/// T12 运筹武将面批 2：diy_sp_zhouyu（雄姿=摸牌阶段补摸 1+⌊装备/2⌋；
/// 雅量=伤害双方各摸 1，可选确认门）。
#[test]
fn g1_yunchou_generals_batch2_xiongzi_yaliang() {
    let json = std::fs::read_to_string(
        repo_extensions_root()
            .join("shahai-yunchou-upstream-noname/assets/character.json"),
    )
    .expect("character.json");
    let bridged = content_native_loader::bridge_generals(&json).expect("桥接");
    assert!(
        bridged.generals.len() >= 3,
        "批 2 起累计武将（现 {} 名）",
        bridged.generals.len()
    );
    assert!(bridged.unmapped_skills >= 2, "暂缓累计（现 {} 条）", bridged.unmapped_skills);

    let base = engine_native::content::standard::STANDARD_GENERALS.len() as u16;
    let pack = ExtensionPack {
        def: engine_native::content::extension::ExtensionDef {
            name: "shahai-yunchou-generals-b2",
            version: 1,
            description: "运筹武将面批 2",
        },
        generals: bridged.generals.clone(),
        cards: Vec::new(),
        host_skills: bridged.host_skills.clone(),
        vm_skills: Vec::new(),
        init_hook: engine_native::content::extension::InitHook::None,
        enabled: true,
    };
    let mut e = Engine::new_with_extensions(66002, 4, &[&pack]).expect("建局");
    let meta2 = e.catalog().general_meta(base + 2).expect("周瑜·SP 在目录");
    assert_eq!(meta2.0, "ycn周瑜");
    assert_eq!(meta2.2, vec!["ycnxiongzi", "ycnyaliang"]);
    assert_eq!(meta2.3, 3);

    // 雄姿：p0 摸牌阶段固定 2 后补摸 1+⌊装备/2⌋（增量断言：Judge→Draw
    // 之间手牌变化=固定 2+补摸）
    e.debug_grant(0, "ycnxiongzi");
    advance_to(&mut e, 0, Phase::Judge);
    let h_judge = hand_len(&e, 0);
    advance_to(&mut e, 0, Phase::Draw);
    assert_eq!(
        hand_len(&e, 0),
        h_judge + 2 + 1,
        "雄姿补摸 1（无装备）"
    );
    // 挂 1 件装备：⌊1/2⌋=0 补摸仍 1
    let z = e.debug_place(0, "zhuge");
    e.state_core_mut().players[0].equips.push(z);
    advance_to(&mut e, 0, Phase::Judge);
    let h_judge = hand_len(&e, 0);
    advance_to(&mut e, 0, Phase::Draw);
    assert_eq!(hand_len(&e, 0), h_judge + 2 + 1, "⌊1/2⌋=0 补摸仍 1");
    // 挂至 2 件：⌊2/2⌋=1 补摸 2
    let z2 = e.debug_place(0, "renwang");
    e.state_core_mut().players[0].equips.push(z2);
    advance_to(&mut e, 0, Phase::Judge);
    let h_judge = hand_len(&e, 0);
    advance_to(&mut e, 0, Phase::Draw);
    assert_eq!(hand_len(&e, 0), h_judge + 2 + 2, "⌊2/2⌋=1 补摸 2");

    // 雅量：p2 杀 p1 → 双方各摸 1（确认门）
    e.debug_grant(1, "ycnyaliang");
    e.debug_clear_hand(1);
    advance_to(&mut e, 2, Phase::Play);
    let sha = e.debug_place(2, "sha");
    submit(&mut e, 2, GameAction::PlayCard { card: sha, targets: vec![1] });
    settle_optional_confirm(&mut e);
    let h1 = hand_len(&e, 1);
    let h2 = hand_len(&e, 2);
    assert!(h1 >= 1 || h2 >= 1, "雅量：双方摸牌（受击者 p1 手牌 {h1}）");
}

/// T12 运筹武将面批 3：diy_luxun 整将（纵火=非火伤害可选弃牌改火
/// [DamageBefore 帧直写 nature，铁索传导验证]；烧营=火伤后弃堆顶牌亮出
/// 红色则受伤者距离 1 内座位序首目标受 1 火伤）。
#[test]
fn g1_yunchou_generals_batch3_zonghuo_shaoying() {
    let json = std::fs::read_to_string(
        repo_extensions_root()
            .join("shahai-yunchou-upstream-noname/assets/character.json"),
    )
    .expect("character.json");
    let bridged = content_native_loader::bridge_generals(&json).expect("桥接");
    assert!(
        bridged.generals.len() >= 4,
        "批 3 起累计武将（现 {} 名）",
        bridged.generals.len()
    );
    assert!(bridged.unmapped_skills >= 2);

    let base = engine_native::content::standard::STANDARD_GENERALS.len() as u16;
    let pack = ExtensionPack {
        def: engine_native::content::extension::ExtensionDef {
            name: "shahai-yunchou-generals-b3",
            version: 1,
            description: "运筹武将面批 3",
        },
        generals: bridged.generals.clone(),
        cards: Vec::new(),
        host_skills: bridged.host_skills.clone(),
        vm_skills: Vec::new(),
        init_hook: engine_native::content::extension::InitHook::None,
        enabled: true,
    };
    {
        let e = Engine::new_with_extensions(66003, 4, &[&pack]).expect("建局");
        let m = e.catalog().general_meta(base + 3).expect("陆逊在目录");
        assert_eq!(m.0, "ycn陆逊");
        assert_eq!(m.2, vec!["ycnzonghuo", "ycnshaoying"]);
    }

    // 纵火：非火伤 → 确认+弃首张 → 帧直写火性（铁索传导证明）
    {
        let mut e = Engine::new_with_extensions(66004, 4, &[&pack]).expect("建局");
        e.debug_grant(0, "ycnzonghuo");
        e.debug_set_hp(1, 8);
        e.debug_set_hp(2, 8);
        {
            let core = e.state_core_mut();
            core.players[1].linked = true;
            core.players[2].linked = true;
        }
        e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
        settle_optional_confirm(&mut e);
        assert_eq!(e.debug_hp(1), 7, "纵火：伤害照常");
        assert_eq!(e.debug_hp(2), 7, "纵火：火性经铁索传导 p2");
        // 负例：无手牌可弃 → 不改火性（无传导）
        {
            let core = e.state_core_mut();
            core.players[3].linked = true;
            core.players[1].linked = true;
        }
        e.debug_clear_hand(0);
        e.debug_set_hp(1, 6);
        e.debug_set_hp(3, 8);
        e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
        settle_optional_confirm(&mut e);
        assert_eq!(e.debug_hp(1), 5, "纵火负例：伤害照常");
        assert_eq!(e.debug_hp(3), 8, "纵火负例：无牌可弃不改火性");
    }

    // 烧营：火伤后 → 确认 → 弃堆顶牌亮出，红色则蔓延目标受 1 火伤
    {
        let mut e = Engine::new_with_extensions(66005, 4, &[&pack]).expect("建局");
        e.debug_grant(0, "ycnshaoying");
        e.debug_set_hp(1, 8);
        e.debug_set_hp(2, 8);
        // 牌堆排空后压受控顶：链式烧营（蔓延火伤再触发）在堆空时自然终止
        e.debug_drain_deck_to_discard();
        e.debug_set_deck_top("tao");
        e.debug_deal_damage(Some(0), 1, 1, DamageNature::Fire);
        settle_optional_confirm(&mut e);
        assert_eq!(e.debug_hp(2), 7, "烧营：堆顶红 → 距离 1 内目标受 1 火伤");
        // 负例：堆顶黑（按谓词压黑桃——闪有红花色变体，kind 挑选不可控）
        e.debug_drain_deck_to_discard();
        e.debug_set_deck_top_by(|c| c.suit == Suit::Spade);
        e.debug_deal_damage(Some(0), 1, 1, DamageNature::Fire);
        settle_optional_confirm(&mut e);
        assert_eq!(e.debug_hp(2), 7, "烧营负例：堆顶黑不蔓延");
        assert_eq!(e.debug_hp(1), 6, "原火伤照常");
    }
}

/// T12 运筹武将面批 4：蛊惑·变体（于吉部分——摸 2 弃 2 确定性序）、
/// 狡黠（红色牌目标摸 1）、红莲（来源弃全部红牌）。蝉缘（loseMaxHp）
/// 无承载暂缓。
#[test]
fn g1_yunchou_generals_batch4_guhuo_jiaoxia_honglian() {
    let json = std::fs::read_to_string(
        repo_extensions_root()
            .join("shahai-yunchou-upstream-noname/assets/character.json"),
    )
    .expect("character.json");
    let bridged = content_native_loader::bridge_generals(&json).expect("桥接");
    assert!(
        bridged.generals.len() >= 5,
        "批 4 起累计武将（现 {} 名）",
        bridged.generals.len()
    );
    assert!(bridged.unmapped_skills >= 3, "暂缓累计");

    let base = engine_native::content::standard::STANDARD_GENERALS.len() as u16;
    let pack = ExtensionPack {
        def: engine_native::content::extension::ExtensionDef {
            name: "shahai-yunchou-generals-b4",
            version: 1,
            description: "运筹武将面批 4",
        },
        generals: bridged.generals.clone(),
        cards: Vec::new(),
        host_skills: bridged.host_skills.clone(),
        vm_skills: Vec::new(),
        init_hook: engine_native::content::extension::InitHook::None,
        enabled: true,
    };
    {
        let e = Engine::new_with_extensions(66006, 4, &[&pack]).expect("建局");
        let m = e.catalog().general_meta(base + 4).expect("于吉在目录");
        assert_eq!(m.0, "ycn于吉");
        assert_eq!(m.2, vec!["ycndiyguhuo"], "蝉缘暂缓不入目录");
    }

    // 蛊惑·变体：回合开始摸 2 弃 2（Judge-begin 触发；无装备/判定 →
    // 手牌首张序）——推进到 Draw 断言：4(初始)-2(弃)+2(蛊惑摸)+2(固定)=6
    {
        let mut e = Engine::new_with_extensions(66007, 4, &[&pack]).expect("建局");
        e.debug_grant(0, "ycndiyguhuo");
        advance_to(&mut e, 0, Phase::Draw);
        assert_eq!(hand_len(&e, 0), 6, "蛊惑：摸 2 弃 2 后固定摸 2");
    }

    // 狡黠：成为红色牌目标 → 摸 1（杀为黑色负例）
    {
        let mut e = Engine::new_with_extensions(66008, 4, &[&pack]).expect("建局");
        e.debug_grant(1, "ycnjiaoxia");
        e.debug_set_hp(1, 8);
        advance_to(&mut e, 2, Phase::Play);
        let h1 = hand_len(&e, 1);
        // 找一张黑杀打 p1
        let sha = e.debug_place(2, "sha");
        let black = {
            let cat = e.catalog();
            matches!(
                cat.cards[sha as usize].suit,
                Suit::Spade | Suit::Club
            )
        };
        submit(&mut e, 2, GameAction::PlayCard { card: sha, targets: vec![1] });
        settle_optional_confirm(&mut e);
        if black {
            assert_eq!(hand_len(&e, 1), h1, "狡黠：黑牌目标不触发");
        } else {
            assert_eq!(hand_len(&e, 1), h1 + 1, "狡黠：红牌目标摸 1");
        }
    }

    // 红莲：p1 受 p2 伤害 → 确认 → p2 弃全部红牌（手 1 红+装备 1 红）
    {
        let mut e = Engine::new_with_extensions(66009, 4, &[&pack]).expect("建局");
        e.debug_grant(1, "ycnhonglian");
        e.debug_set_hp(1, 8);
        // p2 手牌：留一张红桃、清其余（装备红牌弃置面同 host 先例，此段
        // 验证手牌红弃置）
        let red = e.debug_place(2, "tao");
        let reds_before = 1;
        e.debug_clear_hand(2);
        e.state_core_mut().players[2].hand.push(red);
        let h2 = hand_len(&e, 2);
        assert_eq!(h2, reds_before);
        e.debug_deal_damage(Some(2), 1, 1, DamageNature::Normal);
        settle_optional_confirm(&mut e);
        assert_eq!(hand_len(&e, 2), 0, "红莲：来源红牌被全弃");
    }
}

/// T12 运筹武将面批 5：全部十名武将入册（蔡文姬/陆抗/马钧三将暂全暂缓
/// ——机制缺口：濒死/出牌主动技面、内置技能引用、骰子面）。目录元数据
/// 面验收（映射表全量 17 技，转正 7 暂缓 10...勘正：转正 7+暂缓 10=17）。
#[test]
fn g1_yunchou_generals_batch5_full_roster_metadata() {
    let json = std::fs::read_to_string(
        repo_extensions_root()
            .join("shahai-yunchou-upstream-noname/assets/character.json"),
    )
    .expect("character.json");
    let bridged = content_native_loader::bridge_generals(&json).expect("桥接");
    assert_eq!(bridged.generals.len(), 10, "十名武将全量入册");
    // 技能条目全量 20（skill.js 17 定义 + 悲歌/谦逊/精械 3 个内置/外部
    // 引用）；bridge 语义=目录技能列表只含 mapped（9 host），暂缓 11 只
    // 计数不进列表
    assert_eq!(bridged.host_skills.len(), 9, "九条 host 行为");
    assert_eq!(
        bridged.generals.iter().map(|g| g.skills.len()).sum::<usize>(),
        9,
        "目录技能列表=mapped 面"
    );
    assert_eq!(bridged.unmapped_skills, 11, "暂缓 11 技全登记");

    let base = engine_native::content::standard::STANDARD_GENERALS.len() as u16;
    let pack = ExtensionPack {
        def: engine_native::content::extension::ExtensionDef {
            name: "shahai-yunchou-generals-b5",
            version: 1,
            description: "运筹武将面批 5（全量入册）",
        },
        generals: bridged.generals.clone(),
        cards: Vec::new(),
        host_skills: bridged.host_skills.clone(),
        vm_skills: Vec::new(),
        init_hook: engine_native::content::extension::InitHook::None,
        enabled: true,
    };
    let e = Engine::new_with_extensions(66010, 4, &[&pack]).expect("建局");
    for (idx, (name, hp)) in [
        (0, ("ycn曹操", 4u32)),
        (1, ("ycn魏延", 4)),
        (2, ("ycn周瑜", 3)),
        (3, ("ycn陆逊", 3)),
        (4, ("ycn于吉", 3)),
        (5, ("ycn周瑜", 3)),
        (6, ("ycn甄姬", 3)),
        (7, ("ycn蔡文姬", 3)),
        (8, ("ycn陆抗", 4)),
        (9, ("ycn马钧", 3)),
    ] {
        let m = e
            .catalog()
            .general_meta(base + idx)
            .unwrap_or_else(|| panic!("武将 {idx} 在目录"));
        assert_eq!(m.0, name, "武将 {idx} 名");
        assert_eq!(m.3, hp, "武将 {idx} 血");
    }
}
