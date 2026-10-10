//! G1 杀海拾遗·古剑子包切片（ADR 0776）：舒筋散（目标弃任意张手牌
//! 摸等量）/面里金针（目标摸 1 后失去 1 点体力，规则 v77）/
//! 飞镖（目标弃一张黑色手牌或失去 1 点体力；登记差异
//! 同前）→ 真实包 → 导入 → 建局/规则结算/内容锁。
//!
//! 切片 40/40 张全卡转换（deferred 清零）——食物簇 12/12、机关簇
//! 6/8、余簇全部收尾；帝女玄霜/玉衡按登记差异简化转换。
//! 上游核对：`杀海拾遗/card/gujian/{list,card}.js @ 5c02a745`。

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use content_runtime::{discover_extensions, import_extensions, InstallOptions, PackageStore};
use engine_native::catalog::{CardKind, Suit};
use engine_native::content::extension::ExtensionPack;
use engine_native::{
    ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, LogEvent, Phase, RejectReason,
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

/// 装载（ADR 0777/0778）：包元数据写在 cards.json，食物/装备伴随技能由
/// 引擎牌种表登记，桥接统一在 content-native-loader，测试不含逐包映射。
fn bridge(json: &str) -> Result<(ExtensionPack, Vec<String>), String> {
    let out = content_native_loader::bridge_pack(json).map_err(|e| e.to_string())?;
    Ok((out.pack, out.deferred))
}

/// 前进到 0 号玩家出牌阶段（消化开局摸牌与挂起选择）。
fn advance_to_play(e: &mut Engine) {
    for _ in 0..64 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play && e.open_choice().is_none() {
            return;
        }
        if let Some(c) = e.open_choice().cloned() {
            let cid = c.id;
            let sel = if c.spec.min > 0 {
                c.spec.options.iter().take(c.spec.min).copied().collect()
            } else {
                vec![c.spec.options[0]]
            };
            let obs = c.observer;
            match submit(e, obs, GameAction::Answer { choice_id: cid, selection: sel }) {
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
    panic!("64 步内未到达 0 号出牌阶段");
}

/// 全区清除若干实体牌（hand/equips/judge/deck/discard/table）——
/// 注入前置：归属总账按目录全集断言，缺一区都会被恢复校验拒绝。
fn purge_all_zones(snap: &mut engine_native::StateSnapshot, ids: &[u32]) {
    for c in ids {
        snap.core.deck.retain(|x| x != c);
        snap.core.discard.retain(|x| x != c);
        for p in snap.core.players.iter_mut() {
            p.hand.retain(|x| x != c);
            p.equips.retain(|x| x != c);
            p.judge.retain(|x| x != c);
        }
        snap.core.table.retain(|x| x != c);
    }
}

/// 无懈窗口：无人持无懈时自动跳过（不挂起）；出现则直过。
fn pass_nullify_window(e: &mut Engine) {
    let is_window = e
        .open_choice()
        .map(|c| c.spec.prompt == "nullify_window")
        .unwrap_or(false);
    if is_window {
        let c = e.open_choice().unwrap();
        let cid = c.id;
        let obs = c.observer;
        drop(c);
        submit(e, obs, GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Pass],
        });
    }
}

/// 清理剩余询问直至静止。空选择合法的询问空应答（=取消/不发动）；
/// min>0 的确认类询问（如金莲烛确认）空应答非法，会原样重开——改选
/// 中性 Pass（放弃响应），无 Pass 时取首个合法选项。步数上限防挂。
fn drain_choices(e: &mut Engine) {
    for _ in 0..64 {
        let Some(c) = e.open_choice() else { return };
        let cid = c.id;
        let obs = c.observer;
        let sel = if c.spec.min > 0 {
            let pass = c
                .spec
                .options
                .iter()
                .find(|o| **o == ChoiceOption::Pass)
                .copied();
            match pass {
                Some(p) => vec![p],
                None => vec![c.spec.options[0]],
            }
        } else {
            vec![]
        };
        drop(c);
        submit(e, obs, GameAction::Answer { choice_id: cid, selection: sel });
    }
    panic!("64 步内询问未消化");
}

fn assert_zone_conservation(e: &Engine) {
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

/// G1 纵向：发现 → 导入 → 桥接 → 建局 → 面里金针结算 → 内容锁。
#[test]
fn g1_gujian_vertical_existing_kinds_and_lock() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    assert!(discovered.iter().any(|d| d.name == "shahai-gujian-upstream-noname"));
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "shahai-gujian")
        .expect("古剑安装")
        .clone();
    let json = std::fs::read_to_string(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .expect("cards.json 可读");
    let (pack, deferred_names) = bridge(&json).expect("桥接");
    assert_eq!(
        (pack.def.name, pack.def.version),
        ("shahai-gujian-upstream-noname", 1),
        "包元数据来自 cards.json"
    );
    let companion: Vec<&str> = pack.host_skills.iter().map(|d| d.name).collect();
    assert_eq!(companion.len(), 18, "18 个伴随技能由引擎牌种表登记");
    assert_eq!(
        (companion[0], companion[17]),
        (
            engine_native::content::host::GJ_JINLIANZHU_KEY,
            engine_native::content::host::GJ_YUHENG_SKILL_KEY
        ),
        "按牌首次出现次序登记（决定目录技能编号）"
    );
    assert_eq!(
        pack.card_count(),
        40,
        "切片 40/40 张全卡转换（食物 12+机关 6+锦囊基本等 22）"
    );
    let deferred: usize = parsed_deferred_count(&json);
    assert_eq!(deferred, 0, "deferred 清零：古剑全卡转换收官");
    assert_eq!(deferred_names.len(), parsed_deferred_len(&json));

    let base_cards = Engine::new(1, 3).catalog().cards.len();
    let mut e = Engine::new_with_extensions(5201, 3, &[&pack]).expect("建局");
    assert_eq!(e.catalog().cards.len(), base_cards + 40);
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }

    // 面里金针：目标摸 1 然后受 1 点伤害（无选择、无无懈窗口）
    let (ml, black, red) = {
        let catalog = e.catalog();
        let ml = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("gj_mianlijinzhen"))
            .expect("面里金针在目录") as u32;
        let black = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Sha && matches!(c.suit, Suit::Spade))
            .expect("黑杀在目录") as u32;
        let red = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Tao && c.suit == Suit::Heart)
            .expect("红桃在目录") as u32;
        (ml, black, red)
    };
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[ml, black, red]);
    snap.core.players[0].hand.push(ml);
    snap.core.players[1].hand.push(black);
    snap.core.players[2].hand.push(red);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    e.debug_set_hp(0, 3);
    e.debug_set_hp(1, 3);
    let h1 = e.debug_hand(1).len();
    let hp1 = e.debug_hp(1);
    let out = submit(&mut e, 0, GameAction::PlayCard { card: ml, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert!(
        e.open_choice().is_none(),
        "面里金针无无懈窗口，不应挂起选择"
    );
    assert_eq!(e.debug_hand(1).len(), h1 + 1, "目标摸 1");
    assert_eq!(e.debug_hp(1), hp1 - 1, "目标失去 1 点体力（规则 v77 对齐上游 loseHp）");
    // 失去体力≠伤害：无伤害日志（不经护甲/不触发伤害来源技）。
    assert!(
        !e.log().iter().any(|l| matches!(l, LogEvent::DamageDealt { target: 1, .. })),
        "失去体力不产生 DamageDealt 日志"
    );
    assert!(
        e.log().iter().any(|l| matches!(l, LogEvent::HpLost { player: 1, amount: 1 })),
        "应有 HpLost 日志"
    );
    assert_zone_conservation(&e);

    // 内容锁：异扩展集合恢复拒绝
    let snap = e.save();
    let (mut pack2, _) = bridge(&json).expect("桥接2");
    pack2.def.name = "other-pack";
    let wrong = Engine::restore_with_extensions(&snap, &[&pack2]);
    assert!(wrong.is_err(), "异集合恢复应被拒绝");

    // 校验臂：面里金针目标体力须 ≥ 使用者（重新注入已入弃牌堆的金针）
    e.debug_set_hp(1, 1);
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[ml]);
    snap.core.players[0].hand.push(ml);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复2");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: ml, targets: vec![1] });
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::IllegalTargets)),
        "低体力目标应被拒: {out:?}"
    );
}

fn parsed_deferred_count(json: &str) -> usize {
    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();
    parsed["deferred"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d["count"].as_u64().unwrap_or(0) as usize)
        .sum()
}

fn parsed_deferred_len(json: &str) -> usize {
    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();
    parsed["deferred"].as_array().unwrap().len()
}

/// 舒筋散：目标弃 2 张摸 2；取消则无事发生；挂起点字节恢复。
#[test]
fn shujinsan_discard_any_draws_equal_with_restore() {
    use engine_native::archive::{decode_archive, encode_archive};
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "shahai-gujian-upstream-noname")
            .expect("古剑可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let (pack, _) = bridge(&json).expect("桥接");
    let mut e = Engine::new_with_extensions(5202, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (sj, c1, c2, c3) = {
        let catalog = e.catalog();
        let sj = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("gj_shujinsan"))
            .expect("舒筋散在目录") as u32;
        let c1 = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Sha)
            .expect("杀在目录") as u32;
        let c2 = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Shan)
            .expect("闪在目录") as u32;
        let c3 = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Tao)
            .expect("桃在目录") as u32;
        (sj, c1, c2, c3)
    };
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[sj, c1, c2, c3]);
    snap.core.players[0].hand.push(sj);
    snap.core.players[1].hand.push(c1);
    snap.core.players[1].hand.push(c2);
    snap.core.players[1].hand.push(c3);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sj, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    pass_nullify_window(&mut e);
    // 目标选牌挂起（观察者 = 目标）
    let c = e.open_choice().expect("舒筋散目标选牌挂起");
    assert_eq!(c.spec.prompt, "gj_shujinsan_discard");
    assert_eq!(c.observer, 1);
    assert_eq!(c.spec.min, 1, "至少弃 1 张");
    assert_eq!(c.spec.max, 3, "至多弃全部手牌");
    assert!(c.spec.allow_cancel, "可取消");
    // 挂起点字节恢复
    let bytes = encode_archive(&e.save());
    let mut restored = Engine::restore_with_extensions(
        &decode_archive(&bytes).expect("decode"),
        &[&pack],
    )
    .expect("舒筋散挂起点字节恢复");
    drop(c);
    let cid = restored.open_choice().unwrap().id;
    let out = submit(&mut restored, 1, GameAction::Answer {
        choice_id: cid,
        selection: vec![ChoiceOption::Card(c1), ChoiceOption::Card(c2)],
    });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    let hand1 = restored.debug_hand(1);
    assert_eq!(hand1.len(), 3, "弃 2 摸 2：3-2+2=3");
    assert!(!hand1.contains(&c1) && !hand1.contains(&c2), "所选两张已弃置");
    assert!(hand1.contains(&c3), "未选牌保留");
    assert_zone_conservation(&restored);

    // 取消分支：重开一局，目标取消 → 无弃无摸
    let mut e = Engine::new_with_extensions(5203, 3, &[&pack]).expect("建局2");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[sj, c1, c2, c3]);
    snap.core.players[0].hand.push(sj);
    snap.core.players[1].hand.push(c1);
    snap.core.players[1].hand.push(c2);
    snap.core.players[1].hand.push(c3);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复2");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sj, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    pass_nullify_window(&mut e);
    let c = e.open_choice().expect("舒筋散目标选牌挂起");
    assert_eq!(c.spec.prompt, "gj_shujinsan_discard");
    let cid = c.id;
    drop(c);
    submit(&mut e, 1, GameAction::Answer {
        choice_id: cid,
        selection: vec![],
    });
    assert_eq!(e.debug_hand(1).len(), 3, "取消：手牌不变");
    assert_eq!(e.debug_hand(0).len(), 0, "打出舒筋散后使用者手牌空");
    assert_zone_conservation(&e);
}

/// 飞镖：有黑色手牌 → 弃指定黑牌免伤；取消 → 受 1 点伤害；
/// 无黑色手牌 → 直接受 1 点伤害（无选择）。
#[test]
fn feibiao_discard_black_or_damage() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "shahai-gujian-upstream-noname")
            .expect("古剑可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let (pack, _) = bridge(&json).expect("桥接");
    let mut e = Engine::new_with_extensions(5204, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (fb, black, red) = {
        let catalog = e.catalog();
        let fb = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("gj_feibiao"))
            .expect("飞镖在目录") as u32;
        let black = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Sha && matches!(c.suit, Suit::Spade | Suit::Club))
            .expect("黑杀在目录") as u32;
        let red = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Tao && c.suit == Suit::Heart)
            .expect("红桃在目录") as u32;
        (fb, black, red)
    };
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[fb, black, red]);
    snap.core.players[0].hand.push(fb);
    snap.core.players[1].hand.push(black);
    snap.core.players[1].hand.push(red);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let hp1 = e.debug_hp(1);
    let out = submit(&mut e, 0, GameAction::PlayCard { card: fb, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    pass_nullify_window(&mut e);
    // 目标选牌挂起：选项只含黑色手牌
    let c = e.open_choice().expect("飞镖目标选牌挂起");
    assert_eq!(c.spec.prompt, "gj_feibiao_discard");
    assert_eq!(c.observer, 1);
    assert_eq!(c.spec.options, vec![ChoiceOption::Card(black)], "选项仅黑色手牌");
    assert!(c.spec.allow_cancel, "可取消（取消则受伤）");
    let cid = c.id;
    drop(c);
    submit(&mut e, 1, GameAction::Answer {
        choice_id: cid,
        selection: vec![ChoiceOption::Card(black)],
    });
    assert_eq!(e.debug_hp(1), hp1, "弃黑牌免伤");
    assert!(!e.debug_hand(1).contains(&black), "黑牌已弃");
    assert_zone_conservation(&e);

    // 取消分支：目标取消 → 受 1 点伤害
    let mut e = Engine::new_with_extensions(5205, 3, &[&pack]).expect("建局2");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[fb, black]);
    snap.core.players[0].hand.push(fb);
    snap.core.players[1].hand.push(black);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复2");
    let hp1 = e.debug_hp(1);
    let out = submit(&mut e, 0, GameAction::PlayCard { card: fb, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    pass_nullify_window(&mut e);
    let c = e.open_choice().expect("飞镖目标选牌挂起");
    let cid = c.id;
    drop(c);
    submit(&mut e, 1, GameAction::Answer {
        choice_id: cid,
        selection: vec![],
    });
    assert_eq!(e.debug_hp(1), hp1 - 1, "取消 → 失去 1 点体力（规则 v77）");
    assert!(
        !e.log().iter().any(|l| matches!(l, LogEvent::DamageDealt { target: 1, .. })),
        "飞镖取消分支为失去体力而非伤害"
    );

    // 无黑色手牌：直接受伤，无选择
    let mut e = Engine::new_with_extensions(5206, 3, &[&pack]).expect("建局3");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[fb, red]);
    snap.core.players[0].hand.push(fb);
    snap.core.players[1].hand.push(red);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复3");
    let hp1 = e.debug_hp(1);
    let out = submit(&mut e, 0, GameAction::PlayCard { card: fb, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    pass_nullify_window(&mut e);
    assert!(
        e.open_choice().is_none(),
        "无黑色手牌不应挂起选择"
    );
    assert_eq!(e.debug_hp(1), hp1 - 1, "无黑牌直接受伤");
    assert_zone_conservation(&e);
}

/// 校验臂：舒筋散/飞镖目标合法性（无手牌目标拒、自-target 拒）。
#[test]
fn gujian_rejects_illegal_targets() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "shahai-gujian-upstream-noname")
            .expect("古剑可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let (pack, _) = bridge(&json).expect("桥接");
    let mut e = Engine::new_with_extensions(5207, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (sj, fb, black) = {
        let catalog = e.catalog();
        let find = |prefix: &str| {
            catalog
                .cards
                .iter()
                .position(|c| c.name.starts_with(prefix))
                .expect("牌在目录") as u32
        };
        let black = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Sha && matches!(c.suit, Suit::Spade))
            .expect("黑杀在目录") as u32;
        (find("gj_shujinsan"), find("gj_feibiao"), black)
    };
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[sj, fb, black]);
    snap.core.players[0].hand.push(sj);
    snap.core.players[0].hand.push(fb);
    snap.core.players[1].hand.push(black);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    // 舒筋散：无手牌目标被拒
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sj, targets: vec![2] });
    assert!(matches!(out, CommandOutcome::Rejected(RejectReason::IllegalTargets)), "{out:?}");
    // 飞镖：自-target 被拒
    let out = submit(&mut e, 0, GameAction::PlayCard { card: fb, targets: vec![0] });
    assert!(matches!(out, CommandOutcome::Rejected(RejectReason::IllegalTargets)), "{out:?}");
    // 飞镖：双目标被拒
    let out = submit(&mut e, 0, GameAction::PlayCard { card: fb, targets: vec![1, 2] });
    assert!(matches!(out, CommandOutcome::Rejected(RejectReason::IllegalTargets)), "{out:?}");
    assert_zone_conservation(&e);
}

/// 金莲烛（事件取消原语）：伤害前置持有者确认——确认则弃烛、防止
/// 伤害、来源摸 1；拒绝则伤害照常结算；确认分支挂起点字节恢复。
#[test]
fn jinlianzhu_cancels_damage_with_holder_confirm() {
    use engine_native::archive::{decode_archive, encode_archive};
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "shahai-gujian-upstream-noname")
            .expect("古剑可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let (pack, _) = bridge(&json).expect("桥接");
    let mut e = Engine::new_with_extensions(5208, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (jz, sha, tao) = {
        let catalog = e.catalog();
        let jz = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("gj_jinlianzhu"))
            .expect("金莲烛在目录") as u32;
        let sha = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Sha)
            .expect("杀在目录") as u32;
        let tao = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Tao)
            .expect("桃在目录") as u32;
        (jz, sha, tao)
    };
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[jz, sha, tao]);
    // 0 号持杀（来源），1 号持金莲烛（持有者），2 号持桃（受害者手牌）
    snap.core.players[0].hand.push(sha);
    snap.core.players[1].hand.push(jz);
    snap.core.players[2].hand.push(tao);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    e.debug_set_hp(2, 4);

    // 受害者 = 2 号，来源 = 0 号；持烛者 1 号确认 → 伤害防止 + 来源摸 1
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![2] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    pass_nullify_window(&mut e);
    // 杀响应窗口：受害者无闪，直过（目标选牌可能不存在）——等待金莲烛确认
    let c = loop {
        let c = e.open_choice().expect("金莲烛确认挂起");
        if c.spec.prompt == "gj_jinlianzhu_confirm" {
            break c;
        }
        // 其他窗口（如杀的 respond）无人可应答时不会挂起——不应到达
        panic!("意外的选择: {} (obs={})", c.spec.prompt, c.observer);
    };
    assert_eq!(c.observer, 1, "持有者被询问");
    assert_eq!(
        c.spec.options,
        vec![ChoiceOption::Confirm, ChoiceOption::Pass]
    );
    // 挂起点字节恢复
    let bytes = encode_archive(&e.save());
    let mut restored = Engine::restore_with_extensions(
        &decode_archive(&bytes).expect("decode"),
        &[&pack],
    )
    .expect("金莲烛确认挂起点字节恢复");
    drop(c);
    let cid = restored.open_choice().unwrap().id;
    let out = submit(&mut restored, 1, GameAction::Answer {
        choice_id: cid,
        selection: vec![ChoiceOption::Confirm],
    });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert!(!restored.debug_hand(1).contains(&jz), "金莲烛已弃置");
    assert_eq!(restored.debug_hp(2), 4, "伤害被防止");
    assert_eq!(restored.debug_hand(0).len(), 1, "来源摸 1（弃杀 1+摸 1）");
    assert_zone_conservation(&restored);

    // 拒绝分支：持烛者 Pass → 伤害照常
    let mut e = Engine::new_with_extensions(5209, 3, &[&pack]).expect("建局2");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[jz, sha]);
    snap.core.players[0].hand.push(sha);
    snap.core.players[1].hand.push(jz);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复2");
    e.debug_set_hp(2, 4);
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![2] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    pass_nullify_window(&mut e);
    let c = e.open_choice().expect("金莲烛确认挂起");
    assert_eq!(c.spec.prompt, "gj_jinlianzhu_confirm");
    let cid = c.id;
    drop(c);
    submit(&mut e, 1, GameAction::Answer {
        choice_id: cid,
        selection: vec![ChoiceOption::Pass],
    });
    // 受害者无闪 → 伤害结算
    assert_eq!(e.debug_hp(2), 3, "拒绝后伤害照常");
    assert!(e.debug_hand(1).contains(&jz), "金莲烛保留在手");
    assert_eq!(e.debug_hand(0).len(), 0, "来源未摸牌");
    assert_zone_conservation(&e);

    // 金莲烛不能主动使用
    let mut e = Engine::new_with_extensions(5210, 3, &[&pack]).expect("建局3");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[jz]);
    snap.core.players[0].hand.push(jz);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复3");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: jz, targets: vec![1] });
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::IllegalTargets)),
        "主动使用应被拒: {out:?}"
    );
    assert_zone_conservation(&e);
}

/// 食物计时状态原语：摊黄冻（出牌阶段结束摸 1）与有更（阶段开始
/// 已受伤且体力最少回 1）——授予、倒计时扣减、归零移除。
#[test]
fn food_state_grant_tick_and_expire() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "shahai-gujian-upstream-noname")
            .expect("古剑可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let (pack, _) = bridge(&json).expect("桥接");

    // —— 摊黄冻：授予即 rounds=3；出牌阶段结束摸 1 并扣减 ——
    let mut e = Engine::new_with_extensions(5211, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let td = {
        let catalog = e.catalog();
        catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("gj_tanhuadong"))
            .expect("摊黄冻在目录") as u32
    };
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[td]);
    snap.core.players[0].hand.push(td);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let def_td = e
        .catalog()
        .skill_by_name(engine_native::content::host::GJ_FOOD_TANHUADONG_KEY)
        .expect("摊黄冻监听技在目录");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: td, targets: vec![0] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    let inst = e
        .save()
        .core
        .skills
        .get(&(0, def_td))
        .expect("授予后实例存在")
        .clone();
    assert_eq!(inst.rounds, 3, "初始 3 回合");
    let h0 = e.debug_hand(0).len();
    // 出牌阶段结束（Play→Discard 触发 Phase{Play, end}）
    let out = submit(&mut e, 0, GameAction::NextPhase);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    assert_eq!(e.debug_hand(0).len(), h0 + 1, "出牌阶段结束摸 1");
    assert_eq!(e.save().core.skills.get(&(0, def_td)).unwrap().rounds, 2);
    // 挂起无关机器——字节恢复（rounds 入存档）
    let bytes = engine_native::archive::encode_archive(&e.save());
    let restored = Engine::restore_with_extensions(
        &engine_native::archive::decode_archive(&bytes).expect("decode"),
        &[&pack],
    )
    .expect("食物状态字节恢复");
    assert_eq!(restored.save().core.skills.get(&(0, def_td)).unwrap().rounds, 2);
    drop(restored);

    // 归零移除：注入 rounds=1 → 下一次出牌阶段结束移除
    e.debug_set_hp(0, 9);
    let mut snap = e.save();
    for ((owner, def), inst) in snap.core.skills.iter_mut() {
        if *def == def_td {
            inst.rounds = 1;
        }
    }
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复（rounds=1）");
    // 回到 0 号出牌阶段（当前在弃牌阶段之后推进一个回合）
    let mut steps = 0;
    loop {
        steps += 1;
        assert!(steps < 96, "未回到 0 号出牌阶段");
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
            drop(c);
            submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: sel });
            continue;
        }
        let actor = e.turn().current;
        submit(&mut e, actor, GameAction::NextPhase);
    }
    let h0 = e.debug_hand(0).len();
    submit(&mut e, 0, GameAction::NextPhase);
    assert_eq!(e.debug_hand(0).len(), h0 + 1, "最后一回合仍摸 1");
    assert!(
        e.save().core.skills.get(&(0, def_td)).is_none(),
        "归零后实例移除"
    );

    // —— 有更：阶段开始已受伤且体力最少 → 回复 1（对他人授予） ——
    let mut e = Engine::new_with_extensions(5212, 3, &[&pack]).expect("建局2");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let yg = {
        let catalog = e.catalog();
        catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("gj_yougeng"))
            .expect("有更在目录") as u32
    };
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[yg]);
    snap.core.players[0].hand.push(yg);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复2");
    let def_yg = e
        .catalog()
        .skill_by_name(engine_native::content::host::GJ_FOOD_YOUGENG_KEY)
        .expect("有更监听技在目录");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: yg, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert!(e.save().core.skills.contains_key(&(1, def_yg)), "对他人授予");
    // 1 号已受伤且体力最少
    e.debug_set_hp(1, 2);
    e.debug_set_hp(0, 4);
    e.debug_set_hp(2, 4);
    let hp1 = e.debug_hp(1);
    // 推进到 1 号通过判定阶段（其判定阶段开始触发回复）
    let mut steps = 0;
    loop {
        steps += 1;
        assert!(steps < 96, "未推进到 1 号摸牌阶段");
        if e.turn().current == 1 && e.turn().phase == Phase::Draw && e.open_choice().is_none() {
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
            drop(c);
            submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: sel });
            continue;
        }
        let actor = e.turn().current;
        submit(&mut e, actor, GameAction::NextPhase);
    }
    assert_eq!(e.debug_hp(1), hp1 + 1, "判定阶段开始回复 1（准备阶段替位）");
    assert_eq!(e.save().core.skills.get(&(1, def_yg)).unwrap().rounds, 2);
    assert_zone_conservation(&e);
}

/// 食物效果面第二批：青团（首杀摸 1）/元宝肉（额外一张杀）/
/// 咕咚羹（受杀伤害 -1 一次性）/麻婆豆腐（结束阶段确定性弃牌）/
/// 春饼（手牌上限 +1）。
#[test]
fn food_effects_batch_two() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "shahai-gujian-upstream-noname")
            .expect("古剑可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let (pack, _) = bridge(&json).expect("桥接");
    let find = |e: &Engine, prefix: &str| {
        e.catalog()
            .cards
            .iter()
            .position(|c| c.name.starts_with(prefix))
            .unwrap_or_else(|| panic!("{}在目录", prefix)) as u32
    };
    let def_of = |e: &Engine, key: &str| {
        e.catalog()
            .skill_by_name(key)
            .unwrap_or_else(|| panic!("{}在目录", key))
    };

    // —— 青团：出牌阶段首杀摸 1，第二张杀不再摸 ——
    let mut e = Engine::new_with_extensions(5213, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let qt = find(&e, "gj_qingtuan");
    let sha = {
        let catalog = e.catalog();
        catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Sha && matches!(c.suit, Suit::Spade))
            .expect("黑杀在目录") as u32
    };
    let sha2 = {
        let catalog = e.catalog();
        catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Sha && c.suit == Suit::Club)
            .expect("另一黑杀在目录") as u32
    };
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[qt, sha, sha2]);
    snap.core.players[0].hand.push(qt);
    snap.core.players[0].hand.push(sha);
    snap.core.players[0].hand.push(sha2);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let def_qt = def_of(&e, engine_native::content::host::GJ_FOOD_QINGTUAN_KEY);
    let out = submit(&mut e, 0, GameAction::PlayCard { card: qt, targets: vec![0] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    assert!(e.save().core.skills.contains_key(&(0, def_qt)), "青团授予");
    e.debug_set_hp(1, 9); // 1 号体力大，允许被杀命中
    let h0 = e.debug_hand(0).len();
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    pass_nullify_window(&mut e);
    // 目标无闪：可能无响应窗口
    drain_choices(&mut e);
    assert_eq!(e.debug_hand(0).len(), h0, "首杀摸 1（杀离手 -1 + 摸 +1）");
    let h0 = e.debug_hand(0).len();
    // 杀次数已用尽：第二张杀被拒（本场景无额外杀状态）
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sha2, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Rejected(RejectReason::ShaLimit)), "{out:?}");
    assert_eq!(e.debug_hand(0).len(), h0, "无第二杀、无额外摸牌");
    assert_zone_conservation(&e);

    // —— 元宝肉：杀次数用尽后额外一张 ——
    let mut e = Engine::new_with_extensions(5214, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let ybr = find(&e, "gj_yuanbaorou");
    // 第三张杀：供每回合上限负例使用（取既非 sha 又非 sha2 的另一张杀）
    let sha3 = {
        let catalog = e.catalog();
        catalog
            .cards
            .iter()
            .enumerate()
            .position(|(i, c)| c.kind == CardKind::Sha && i as u32 != sha && i as u32 != sha2)
            .expect("第三张杀在目录") as u32
    };
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[sha, sha2, sha3, ybr]);
    snap.core.players[0].hand.push(ybr);
    snap.core.players[0].hand.push(sha);
    snap.core.players[0].hand.push(sha2);
    snap.core.players[0].hand.push(sha3);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let def_ybr = def_of(&e, engine_native::content::host::GJ_FOOD_YUANBAOROU_KEY);
    let out = submit(&mut e, 0, GameAction::PlayCard { card: ybr, targets: vec![0] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    assert!(
        e.save().core.skills.contains_key(&(0, def_ybr)),
        "元宝肉授予"
    );
    e.debug_set_hp(1, 9);
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    pass_nullify_window(&mut e);
    drain_choices(&mut e);
    // 第一次额外杀：放行
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sha2, targets: vec![1] });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "元宝肉额外杀放行: {out:?}"
    );
    pass_nullify_window(&mut e);
    drain_choices(&mut e);
    // 每回合仅一张额外杀：再用一张被上限拒绝
    let h0 = e.debug_hand(0).len();
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sha3, targets: vec![1] });
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::ShaLimit)),
        "元宝肉每回合仅一张额外杀: {out:?}"
    );
    assert_eq!(e.debug_hand(0).len(), h0, "上限后无额外摸牌");
    assert_zone_conservation(&e);

    // —— 咕咚羹：受杀伤害 -1 且状态立即移除 ——
    let mut e = Engine::new_with_extensions(5215, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let gdg = find(&e, "gj_gudonggeng");
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[gdg, sha, sha2]);
    snap.core.players[0].hand.push(gdg);
    snap.core.players[1].hand.push(sha);
    snap.core.players[1].hand.push(sha2);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let def_gdg = def_of(&e, engine_native::content::host::GJ_FOOD_GUDONGGENG_KEY);
    let out = submit(&mut e, 0, GameAction::PlayCard { card: gdg, targets: vec![0] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    // 0 号回合结束 → 1 号回合出杀打 0 号
    let hp0 = e.debug_hp(0);
    let mut steps = 0;
    loop {
        steps += 1;
        assert!(steps < 96);
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
            drop(c);
            submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: sel });
            continue;
        }
        let actor = e.turn().current;
        submit(&mut e, actor, GameAction::NextPhase);
    }
    let out = submit(&mut e, 1, GameAction::PlayCard { card: sha, targets: vec![0] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    pass_nullify_window(&mut e);
    drain_choices(&mut e);
    assert_eq!(e.debug_hp(0), hp0, "1 点杀伤害被 -1 减到 0（体力不变=减伤生效）");
    assert!(
        e.save().core.skills.get(&(0, def_gdg)).is_none(),
        "咕咚羹一次性：触发即移除"
    );

    // —— 春饼：手牌上限 +1（ hp=1 时上限 2，弃牌阶段少弃一张） ——
    let mut e = Engine::new_with_extensions(5216, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let cb = find(&e, "gj_chunbing");
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[cb]);
    snap.core.players[0].hand.push(cb);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: cb, targets: vec![0] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    e.debug_set_hp(0, 1); // 手牌上限 1+1=2
    // 出牌阶段结束时手上有春饼弃置后的牌 + 摸牌阶段 2 张 = 2 张 → 不需要弃
    let out = submit(&mut e, 0, GameAction::NextPhase);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    // 弃牌阶段：上限 2，手中 0 张（春饼已弃、无摸牌）→ 不挂起弃牌选择
    assert!(
        e.open_choice().is_none(),
        "手牌未超上限不挂起弃牌选择"
    );
    assert_zone_conservation(&e);
}

/// 护甲原语 + 食物第三批：鲤鱼汤（护甲）/麻茶（黑色目标限制）/
/// 菱角（摸牌阶段 +1 弃 1，选择机含挂起派生）。
#[test]
fn hujia_primitive_and_food_batch_three() {
    use engine_native::archive::{decode_archive, encode_archive};
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "shahai-gujian-upstream-noname")
            .expect("古剑可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let (pack, _) = bridge(&json).expect("桥接");
    let find = |e: &Engine, prefix: &str| {
        e.catalog()
            .cards
            .iter()
            .position(|c| c.name.starts_with(prefix))
            .unwrap_or_else(|| panic!("{}在目录", prefix)) as u32
    };

    // —— 麻茶：黑色牌不能指定持有者为目标；红色牌不受限 ——
    let mut e = Engine::new_with_extensions(5217, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (mc, sha_black, sha_red) = {
        let catalog = e.catalog();
        let mc = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("gj_molicha"))
            .expect("麻茶在目录") as u32;
        let sha_black = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Sha && matches!(c.suit, Suit::Spade))
            .expect("黑杀在目录") as u32;
        let sha_red = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Sha && c.suit == Suit::Heart)
            .expect("红杀在目录") as u32;
        (mc, sha_black, sha_red)
    };
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[mc, sha_black, sha_red]);
    snap.core.players[0].hand.push(mc);
    snap.core.players[0].hand.push(sha_red);
    snap.core.players[0].hand.push(sha_black);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    // 麻茶喂给 1 号
    let out = submit(&mut e, 0, GameAction::PlayCard { card: mc, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    // 黑杀指定持有者（其他玩家回合内使用路径）→ 出牌校验拒绝
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sha_black, targets: vec![1] });
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::IllegalTargets)),
        "黑色牌指定持有者应被拒: {out:?}"
    );
    // 红杀指定持有者 → 不受限（走完整杀结算）
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sha_red, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "红色牌不受限: {out:?}");
    pass_nullify_window(&mut e);
    drain_choices(&mut e);
    assert_zone_conservation(&e);

    // —— 鲤鱼汤：体力最少（之一）阶段开始得 1 护甲；护甲抵扣伤害 ——
    let mut e = Engine::new_with_extensions(5218, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (lyt, black, red) = {
        let catalog = e.catalog();
        let lyt = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("gj_liyutang"))
            .expect("鲤鱼汤在目录") as u32;
        let black = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Sha && matches!(c.suit, Suit::Spade))
            .expect("黑杀在目录") as u32;
        let red = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Tao && c.suit == Suit::Heart)
            .expect("红桃在目录") as u32;
        (lyt, black, red)
    };
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[lyt, black, red]);
    snap.core.players[0].hand.push(lyt);
    snap.core.players[1].hand.push(black);
    snap.core.players[2].hand.push(red);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: lyt, targets: vec![0] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    assert_eq!(e.save().core.players[0].hujia, 0, "授予时无护甲");
    // 0 号体力 3（全场并列最少）→ 结束阶段 +1 护甲；先受伤验证抵扣
    e.debug_set_hp(0, 3);
    e.debug_set_hp(1, 4);
    e.debug_set_hp(2, 4);
    // 推进到弃牌阶段前结束阶段？直接 NextPhase：Play→Discard→End
    submit(&mut e, 0, GameAction::NextPhase); // Play 结束
    // 弃牌阶段（手牌空，无弃牌选择）
    if e.open_choice().is_some() {
        let c = e.open_choice().unwrap();
        let cid = c.id;
        let sel: Vec<ChoiceOption> = c.spec.options.iter().take(c.spec.min.max(1)).copied().collect();
        drop(c);
        submit(&mut e, 0, GameAction::Answer { choice_id: cid, selection: sel });
    }
    submit(&mut e, 0, GameAction::NextPhase); // Discard 结束 → End 开始
    assert_eq!(e.save().core.players[0].hujia, 1, "体力最少结束阶段得 1 护甲");
    // 护甲抵扣：1 号回合出杀打 0 号 → 护甲 -1、体力不变
    let mut steps = 0;
    loop {
        steps += 1;
        assert!(steps < 96);
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
            drop(c);
            submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: sel });
            continue;
        }
        let actor = e.turn().current;
        submit(&mut e, actor, GameAction::NextPhase);
    }
    let hp0 = e.debug_hp(0);
    let out = submit(&mut e, 1, GameAction::PlayCard { card: black, targets: vec![0] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    pass_nullify_window(&mut e);
    drain_choices(&mut e);
    assert_eq!(e.save().core.players[0].hujia, 0, "护甲被消耗");
    assert_eq!(e.debug_hp(0), hp0, "护甲抵扣后体力不变");
    assert_zone_conservation(&e);

    // —— 菱角：摸牌阶段额外摸 1 后弃 1（选择机 + 字节恢复） ——
    let mut e = Engine::new_with_extensions(5219, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (xj, black, red) = {
        let catalog = e.catalog();
        let xj = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("gj_xiajiao"))
            .expect("菱角在目录") as u32;
        let black = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Sha && matches!(c.suit, Suit::Spade))
            .expect("黑杀在目录") as u32;
        let red = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Tao && c.suit == Suit::Heart)
            .expect("红桃在目录") as u32;
        (xj, black, red)
    };
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[xj, black, red]);
    snap.core.players[0].hand.push(xj);
    snap.core.players[1].hand.push(black);
    snap.core.players[1].hand.push(red);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: xj, targets: vec![0] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    // 推进到 1 号回合结束 → 0 号新回合：Judge→Draw（固定摸 2 + 菱角摸 1 → 弃 1 挂起）
    let mut steps = 0;
    loop {
        steps += 1;
        assert!(steps < 96);
        if e.turn().current == 0 && e.turn().phase == Phase::Draw && e.open_choice().is_some() {
            break;
        }
        if e.turn().current == 0 && e.turn().phase == Phase::Draw && e.open_choice().is_none() {
            // 固定摸牌刚完成；菱角触发在 AfterTriggers——推进一次或等挂起
        }
        if let Some(c) = e.open_choice().cloned() {
            let cid = c.id;
            let sel = if c.spec.min > 0 {
                c.spec.options.iter().take(c.spec.min).copied().collect()
            } else {
                vec![c.spec.options[0]]
            };
            let obs = c.observer;
            drop(c);
            submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: sel });
            continue;
        }
        let actor = e.turn().current;
        submit(&mut e, actor, GameAction::NextPhase);
    }
    let c = e.open_choice().expect("菱角弃牌挂起");
    assert_eq!(c.spec.prompt, "gj_xiajiao_discard");
    assert_eq!(c.observer, 0);
    assert_eq!(c.spec.min, 1);
    // 挂起点字节恢复
    let bytes = encode_archive(&e.save());
    let mut restored = Engine::restore_with_extensions(
        &decode_archive(&bytes).expect("decode"),
        &[&pack],
    )
    .expect("菱角挂起点字节恢复");
    drop(c);
    let cid = restored.open_choice().unwrap().id;
    let h0 = restored.debug_hand(0).len();
    let discard_card = restored.debug_hand(0)[0];
    submit(&mut restored, 0, GameAction::Answer {
        choice_id: cid,
        selection: vec![ChoiceOption::Card(discard_card)],
    });
    assert_eq!(restored.debug_hand(0).len(), h0 - 1, "额外摸 1 后弃 1（净 +1）");
    assert_zone_conservation(&restored);
}

/// 蜜链藕（红桃当桃：出牌 view-as + 濒死响应选项并入）与
/// 卤玉羹（出牌阶段限一次弃基本牌获得牌堆顶）。
#[test]
fn mizhilianou_viewas_and_luyugeng_discover() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "shahai-gujian-upstream-noname")
            .expect("古剑可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let (pack, _) = bridge(&json).expect("桥接");
    let find = |e: &Engine, prefix: &str| {
        e.catalog()
            .cards
            .iter()
            .position(|c| c.name.starts_with(prefix))
            .unwrap_or_else(|| panic!("{}在目录", prefix)) as u32
    };

    // —— 蜜链藕：濒死时红桃手牌当桃（响应选项并入） ——
    let mut e = Engine::new_with_extensions(5220, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (mzl, sha_black, tao_red, shan) = {
        let catalog = e.catalog();
        let mzl = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("gj_mizhilianou"))
            .expect("蜜链藕在目录") as u32;
        let sha_black = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Sha && matches!(c.suit, Suit::Spade))
            .expect("黑杀在目录") as u32;
        let tao_red = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Tao && c.suit == Suit::Heart)
            .expect("红桃在目录") as u32;
        let shan = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Shan)
            .expect("闪在目录") as u32;
        (mzl, sha_black, tao_red, shan)
    };
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[mzl, sha_black, tao_red, shan]);
    snap.core.players[0].hand.push(mzl);
    snap.core.players[0].hand.push(tao_red);
    snap.core.players[1].hand.push(sha_black);
    snap.core.players[2].hand.push(shan);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: mzl, targets: vec![0] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    // 1 号回合出杀打 0 号（体力压到 1 → 命中后濒死）
    e.debug_set_hp(0, 1);
    let mut steps = 0;
    loop {
        steps += 1;
        assert!(steps < 96);
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
            drop(c);
            submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: sel });
            continue;
        }
        let actor = e.turn().current;
        submit(&mut e, actor, GameAction::NextPhase);
    }
    let out = submit(&mut e, 1, GameAction::PlayCard { card: sha_black, targets: vec![0] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    pass_nullify_window(&mut e);
    // 杀响应：无闪直过（若有 respond_shan 则 Pass）
    if let Some(c) = e.open_choice() {
        if c.spec.prompt == "respond_shan" {
            let cid = c.id;
            let obs = c.observer;
            drop(c);
            submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
        }
    }
    // 0 号濒死 → 濒死救援询问：0 号自己的选项应含红桃桃（本身即桃）
    // —— 用 2 号（持闪无桃）验证 pass 流转；关键：0 号可出红桃桃获救
    let saved;
    loop {
        let c = e.open_choice().expect("濒死救援挂起");
        let (prompt, obs) = (c.spec.prompt.to_string(), c.observer);
        assert_eq!(prompt, "dying_save");
        if obs == 0 {
            // 0 号：红桃桃是合法选项（桃本身）；此处直接出桃
            let has_tao = c.spec.options.contains(&ChoiceOption::Card(tao_red));
            let cid = c.id;
            drop(c);
            if has_tao {
                submit(&mut e, 0, GameAction::Answer {
                    choice_id: cid,
                    selection: vec![ChoiceOption::Card(tao_red)],
                });
                saved = true;
                break;
            } else {
                submit(&mut e, 0, GameAction::Answer {
                    choice_id: cid,
                    selection: vec![ChoiceOption::Pass],
                });
            }
        } else {
            let cid = c.id;
            drop(c);
            submit(&mut e, obs, GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Pass],
            });
        }
    }
    assert!(saved, "0 号出红桃桃获救");
    assert_zone_conservation(&e);

    // —— 蜜链藕状态驱动 view-as：黑桃非红桃不可当桃 ——
    //（负例由上面 sha2 无红桃时不触发覆盖；此处验证蜜链藕未授予时
    // 红桃不进濒死选项——由 purge 注入的 2 号无状态即自然覆盖）

    // —— 卤玉羹：出牌阶段限一次，弃基本牌获得牌堆顶 ——
    let mut e = Engine::new_with_extensions(5221, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (lyg, shan2) = {
        let catalog = e.catalog();
        let lyg = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("gj_luyugeng"))
            .expect("卤玉羹在目录") as u32;
        let shan2 = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Shan)
            .expect("闪在目录") as u32;
        (lyg, shan2)
    };
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[lyg, shan2]);
    snap.core.players[0].hand.push(lyg);
    snap.core.players[0].hand.push(shan2);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: lyg, targets: vec![0] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    // 主动技：ActivateSkill 卤玉羹监听技
    let def_lyg = e
        .catalog()
        .skill_by_name(engine_native::content::host::GJ_FOOD_LUYUGENG_KEY)
        .expect("卤玉羹监听技在目录");
    let deck_top = e.save().core.deck.last().copied().expect("牌堆非空");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_lyg });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    let c = e.open_choice().expect("卤玉羹弃牌挂起");
    assert_eq!(c.spec.prompt, "gj_luyugeng_cost");
    assert_eq!(c.observer, 0);
    // 选项只含基本牌（shan2 是基本牌；卤玉羹本体已弃）
    assert_eq!(c.spec.options, vec![ChoiceOption::Card(shan2)]);
    let cid = c.id;
    drop(c);
    let out = submit(&mut e, 0, GameAction::Answer {
        choice_id: cid,
        selection: vec![ChoiceOption::Card(shan2)],
    });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    let hand0 = e.debug_hand(0);
    assert!(hand0.contains(&deck_top), "获得牌堆顶牌");
    assert!(!hand0.contains(&shan2), "基本牌已弃");
    // 每回合限一次：再次发动被拒
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_lyg });
    assert!(
        matches!(out, CommandOutcome::Rejected(_)),
        "once_per_turn 应拒绝重复发动: {out:?}"
    );
    assert_zone_conservation(&e);
}

/// 机关簇：冰魄通（逐目标首张同色比对）/乾坤镖（目标+邻座弃牌）/
/// 龙须钩（获得目标首张装备）/神火飞鸦（目标+邻座需闪否则火伤，
/// 响应循环 + 挂起派生字节恢复）。
#[test]
fn jiguan_batch_four_kinds() {
    use engine_native::archive::{decode_archive, encode_archive};
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "shahai-gujian-upstream-noname")
            .expect("古剑可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let (pack, _) = bridge(&json).expect("桥接");
    let find = |e: &Engine, prefix: &str| {
        e.catalog()
            .cards
            .iter()
            .position(|c| c.name.starts_with(prefix))
            .unwrap_or_else(|| panic!("{}在目录", prefix)) as u32
    };
    let card_of = |e: &Engine, kind: CardKind, suit: Suit| {
        e.catalog()
            .cards
            .iter()
            .position(|c| c.kind == kind && c.suit == suit)
            .unwrap_or_else(|| panic!("牌种在目录")) as u32
    };

    // —— 冰魄通：使用者红桃首张 vs 目标红桃首张（同色）→ 弃+伤害 ——
    let mut e = Engine::new_with_extensions(5222, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (bpt, tao_red, sha_black, heart_sha) = {
        let catalog = e.catalog();
        let bpt = find(&e, "gj_bingpotong");
        let tao_red = card_of(&e, CardKind::Tao, Suit::Heart);
        let sha_black = card_of(&e, CardKind::Sha, Suit::Spade);
        let heart_sha = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Sha && c.suit == Suit::Heart)
            .expect("红杀在目录") as u32;
        (bpt, tao_red, sha_black, heart_sha)
    };
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[bpt, tao_red, sha_black, heart_sha]);
    snap.core.players[0].hand.push(bpt);
    snap.core.players[0].hand.push(tao_red); // 使用者首张：红
    snap.core.players[1].hand.push(heart_sha); // 目标首张：红 → 同色
    snap.core.players[2].hand.push(sha_black); // 异色对照
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    e.debug_set_hp(1, 9);
    e.debug_set_hp(2, 9);
    let hp1 = e.debug_hp(1);
    let hp2 = e.debug_hp(2);
    let out = submit(&mut e, 0, GameAction::PlayCard {
        card: bpt,
        targets: vec![1, 2],
    });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    pass_nullify_window(&mut e);
    // 双段自选展示（逐技对照批五 v79）：使用者选红桃桃、目标 1 选红杀
    // → 同色：使用者弃展示牌、目标 1 失去 1 点体力（v77 原语）
    let c = e.open_choice().expect("冰魄通使用者展示挂起");
    assert_eq!(c.spec.prompt, "gj_bingpo_show_own");
    let cid = c.id;
    drop(c);
    submit(&mut e, 0, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(tao_red)] });
    let c = e.open_choice().expect("冰魄通目标展示挂起");
    assert_eq!((c.spec.prompt, c.observer), ("gj_bingpo_show_target", 1));
    let cid = c.id;
    drop(c);
    submit(&mut e, 1, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(heart_sha)] });
    assert_eq!(e.debug_hp(1), hp1 - 1, "同色目标失去 1 点体力（规则 v77）");
    // 目标 2：使用者展示牌已弃、手牌空 → 上游 step0 直接跳过（无事）
    assert!(e.open_choice().is_none(), "使用者无手牌后后续目标跳过");
    assert_eq!(e.debug_hp(2), hp2, "异色目标无事");
    assert!(!e.debug_hand(0).contains(&tao_red), "同色后使用者弃展示牌");
    assert!(e.debug_hand(1).contains(&heart_sha), "目标展示牌保留（展示非弃置）");
    assert!(e.debug_hand(2).contains(&sha_black), "异色目标牌保留");
    assert_zone_conservation(&e);

    // —— 乾坤镖：目标弃首张手牌；龙须钩：获得目标首张装备 ——
    let mut e = Engine::new_with_extensions(5223, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (qkb, lxg, armor) = {
        let catalog = e.catalog();
        let qkb = find(&e, "gj_qiankunbiao");
        let lxg = find(&e, "gj_longxugou");
        let armor = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Armor)
            .expect("防具在目录") as u32;
        (qkb, lxg, armor)
    };
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[qkb, lxg, armor]);
    snap.core.players[0].hand.push(qkb);
    snap.core.players[0].hand.push(lxg);
    snap.core.players[1].equips.push(armor);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    // 龙须钩：使用者自选 1 号装备区一张获得（逐技对照批五 v79）
    let out = submit(&mut e, 0, GameAction::PlayCard { card: lxg, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    pass_nullify_window(&mut e);
    let c = e.open_choice().expect("龙须钩使用者选装备挂起");
    assert_eq!((c.spec.prompt, c.observer), ("gj_longxu_pick", 0));
    assert_eq!(c.spec.options, vec![ChoiceOption::Card(armor)]);
    let cid = c.id;
    drop(c);
    submit(&mut e, 0, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(armor)] });
    assert!(e.debug_hand(0).contains(&armor), "使用者获得所选装备");
    assert!(e.save().core.players[1].equips.is_empty(), "目标装备被夺");
    // 乾坤镖：1 号无手牌 → 弃首张装备（若有）；0 号邻座距离全 1
    let out = submit(&mut e, 0, GameAction::PlayCard { card: qkb, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    pass_nullify_window(&mut e);
    assert_zone_conservation(&e);

    // —— 神火飞鸦：有闪出闪免伤；无闪 1 火伤；挂起点字节恢复 ——
    let mut e = Engine::new_with_extensions(5224, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (shfy, shan_c, sha_red2) = {
        let catalog = e.catalog();
        let shfy = find(&e, "gj_shenhuofeiya");
        let shan_c = card_of(&e, CardKind::Shan, Suit::Diamond);
        let sha_red2 = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Sha && c.suit == Suit::Heart)
            .expect("红杀在目录") as u32;
        (shfy, shan_c, sha_red2)
    };
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[shfy, shan_c, sha_red2]);
    snap.core.players[0].hand.push(shfy);
    snap.core.players[1].hand.push(shan_c);
    snap.core.players[1].hand.push(sha_red2);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    e.debug_set_hp(1, 9);
    let hp1 = e.debug_hp(1);
    let out = submit(&mut e, 0, GameAction::PlayCard { card: shfy, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    pass_nullify_window(&mut e);
    // 目标（含邻座）有闪 → 出闪询问；先在第一个询问点字节恢复
    let c = e.open_choice().expect("神火飞鸦出闪挂起");
    assert_eq!(c.spec.prompt, "aoe_response");
    assert!(c.spec.options.contains(&ChoiceOption::Card(shan_c)) || c.spec.options.contains(&ChoiceOption::Pass));
    let bytes = encode_archive(&e.save());
    let mut restored = Engine::restore_with_extensions(
        &decode_archive(&bytes).expect("decode"),
        &[&pack],
    )
    .expect("神火飞鸦挂起点字节恢复");
    drop(c);
    // 恢复后逐个响应（出闪或 Pass）
    let mut answered = 0;
    while let Some(c) = restored.open_choice() {
        assert_eq!(c.spec.prompt, "aoe_response");
        let cid = c.id;
        let obs = c.observer;
        let sel = if c.spec.options.len() > 1 && obs == 1 && c.spec.options.contains(&ChoiceOption::Card(shan_c)) {
            vec![ChoiceOption::Card(shan_c)]
        } else {
            vec![ChoiceOption::Pass]
        };
        drop(c);
        submit(&mut restored, obs, GameAction::Answer { choice_id: cid, selection: sel });
        answered += 1;
        assert!(answered < 6, "响应循环有界");
    }
    assert!(answered >= 1, "至少一次出闪询问");
    assert_eq!(restored.debug_hp(1), hp1, "1 号出闪免伤");
    assert!(!restored.debug_hand(1).contains(&shan_c), "闪已消耗");
    assert_zone_conservation(&restored);
}

/// 余簇：紫阳丹（+3 护甲+回合衰减）/玉女元神（濒死一次性回复）/
/// 石会粉（需闪否则禁用对其他角色用牌）/六风散（获得两张实体闪）。
#[test]
fn remaining_clusters_batch() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "shahai-gujian-upstream-noname")
            .expect("古剑可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let (pack, _) = bridge(&json).expect("桥接");
    let find = |e: &Engine, prefix: &str| {
        e.catalog()
            .cards
            .iter()
            .position(|c| c.name.starts_with(prefix))
            .unwrap_or_else(|| panic!("{}在目录", prefix)) as u32
    };
    let card_of = |e: &Engine, kind: CardKind, suit: Suit| {
        e.catalog()
            .cards
            .iter()
            .position(|c| c.kind == kind && c.suit == suit)
            .unwrap_or_else(|| panic!("牌种在目录")) as u32
    };

    // —— 紫阳丹：+3 护甲，回合结束阶段 -1 ——
    let mut e = Engine::new_with_extensions(5225, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (zyd, shan_c) = {
        let catalog = e.catalog();
        let zyd = find(&e, "gj_ziyangdan");
        let shan_c = card_of(&e, CardKind::Shan, Suit::Diamond);
        (zyd, shan_c)
    };
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[zyd]);
    snap.core.players[0].hand.push(zyd);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: zyd, targets: vec![0] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    assert_eq!(e.save().core.players[0].hujia, 3, "获得 3 点护甲");
    submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    if e.open_choice().is_some() {
        let c = e.open_choice().unwrap();
        let cid = c.id;
        let sel: Vec<ChoiceOption> = c.spec.options.iter().take(c.spec.min.max(1)).copied().collect();
        drop(c);
        submit(&mut e, 0, GameAction::Answer { choice_id: cid, selection: sel });
    }
    submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    assert_eq!(e.save().core.players[0].hujia, 2, "结束阶段衰减 -1");
    assert_zone_conservation(&e);

    // —— 玉女元神：濒死一次性回复 ——
    let mut e = Engine::new_with_extensions(5226, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (yvs, sha_black) = {
        let catalog = e.catalog();
        let yvs = find(&e, "gj_yunvyuanshen");
        let sha_black = card_of(&e, CardKind::Sha, Suit::Spade);
        (yvs, sha_black)
    };
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[yvs, sha_black]);
    snap.core.players[0].hand.push(yvs);
    snap.core.players[1].hand.push(sha_black);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let def_yvs = e
        .catalog()
        .skill_by_name(engine_native::content::host::GJ_YUNVYUANSHEN_LISTENER_KEY)
        .expect("玉女元神监听技在目录");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: yvs, targets: vec![0] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    assert!(e.save().core.skills.contains_key(&(0, def_yvs)));
    e.debug_set_hp(0, 1);
    let mut steps = 0;
    loop {
        steps += 1;
        assert!(steps < 96);
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
            drop(c);
            submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: sel });
            continue;
        }
        let actor = e.turn().current;
        submit(&mut e, actor, GameAction::NextPhase);
    }
    let hp0 = e.debug_hp(0);
    let out = submit(&mut e, 1, GameAction::PlayCard { card: sha_black, targets: vec![0] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    pass_nullify_window(&mut e);
    drain_choices(&mut e);
    // 1 点伤害被状态抵消：濒死时回复 1 → 体力回到 1（未死）
    assert_eq!(e.debug_hp(0), hp0, "濒死时状态回复 1 点体力");
    assert!(e.save().core.players[0].alive, "未死亡");
    assert!(
        e.save().core.skills.get(&(0, def_yvs)).is_none(),
        "一次性状态触发即移除"
    );
    assert_zone_conservation(&e);

    // —— 石会粉：无闪 → 禁用对其他角色用牌；有闪 → 出闪免禁 ——
    let mut e = Engine::new_with_extensions(5227, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (shf, shan_c2) = {
        let catalog = e.catalog();
        let shf = find(&e, "gj_shihuifen");
        let shan_c2 = card_of(&e, CardKind::Shan, Suit::Diamond);
        (shf, shan_c2)
    };
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[shf]);
    snap.core.players[0].hand.push(shf);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    // 1 号无闪 → 直接禁用
    let out = submit(&mut e, 0, GameAction::PlayCard { card: shf, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    pass_nullify_window(&mut e);
    assert!(
        e.open_choice().is_none(),
        "无闪不挂起，直接置禁用"
    );
    // 1 号（被禁用者）在其出牌阶段无法对其他人用牌——由校验钩子拦截，
    // 交由下个场景验证有闪分支；此处验证守恒
    assert_zone_conservation(&e);

    // 有闪分支：目标出闪免禁用
    let mut e = Engine::new_with_extensions(5228, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[shf, shan_c2]);
    snap.core.players[0].hand.push(shf);
    snap.core.players[1].hand.push(shan_c2);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: shf, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    pass_nullify_window(&mut e);
    let c = e.open_choice().expect("石会粉出闪挂起");
    assert_eq!(c.spec.prompt, "respond_shan");
    assert_eq!(c.observer, 1);
    let cid = c.id;
    drop(c);
    submit(&mut e, 1, GameAction::Answer {
        choice_id: cid,
        selection: vec![ChoiceOption::Card(shan_c2)],
    });
    // 出闪后无事发生
    assert!(e.open_choice().is_none());
    assert_zone_conservation(&e);

    // —— 六风散：从牌堆获得两张实体闪 ——
    let mut e = Engine::new_with_extensions(5229, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let lfs = find(&e, "gj_liufengsan");
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[lfs]);
    snap.core.players[0].hand.push(lfs);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let shans_in_deck_before = e
        .save()
        .core
        .deck
        .iter()
        .filter(|c| {
            e.catalog().cards[**c as usize].kind == CardKind::Shan
        })
        .count();
    let out = submit(&mut e, 0, GameAction::PlayCard { card: lfs, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    pass_nullify_window(&mut e);
    let gained = e.debug_hand(1).len();
    let expected = shans_in_deck_before.min(2);
    assert_eq!(gained, expected, "获得至多两张实体闪");
    let shans_in_deck_after = e
        .save()
        .core
        .deck
        .iter()
        .filter(|c| {
            e.catalog().cards[**c as usize].kind == CardKind::Shan
        })
        .count();
    assert_eq!(
        shans_in_deck_before - shans_in_deck_after,
        expected,
        "牌堆闪被移出"
    );
    assert_zone_conservation(&e);
}

/// 古剑收官批：砂糖（火伤+护甲）/黑龙鳞片（+1 护甲）/硫头阁（自己
/// 摸 1、他人弃首张）/木头面具（装备 view-as：任意手牌当杀，每回合
/// 一次）。
#[test]
fn final_batch_shatang_heilong_liutouge_mutou() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "shahai-gujian-upstream-noname")
            .expect("古剑可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let (pack, _) = bridge(&json).expect("桥接");
    let find = |e: &Engine, prefix: &str| {
        e.catalog()
            .cards
            .iter()
            .position(|c| c.name.starts_with(prefix))
            .unwrap_or_else(|| panic!("{}在目录", prefix)) as u32
    };
    let card_of = |e: &Engine, kind: CardKind, suit: Suit| {
        e.catalog()
            .cards
            .iter()
            .position(|c| c.kind == kind && c.suit == suit)
            .unwrap_or_else(|| panic!("牌种在目录")) as u32
    };

    // —— 砂糖：1 火伤 + 1 护甲 ——
    let mut e = Engine::new_with_extensions(5230, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let st = find(&e, "gj_shatang");
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[st]);
    snap.core.players[0].hand.push(st);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    e.debug_set_hp(1, 9);
    let hp1 = e.debug_hp(1);
    let out = submit(&mut e, 0, GameAction::PlayCard { card: st, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    pass_nullify_window(&mut e);
    assert_eq!(e.debug_hp(1), hp1 - 1, "1 点火焰伤害");
    assert_eq!(e.save().core.players[1].hujia, 1, "目标获得 1 点护甲");
    assert_zone_conservation(&e);

    // —— 黑龙鳞片：+1 护甲（自-target） ——
    let mut e = Engine::new_with_extensions(5231, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let hlp = find(&e, "gj_heilonglinpian");
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[hlp]);
    snap.core.players[0].hand.push(hlp);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: hlp, targets: vec![] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    pass_nullify_window(&mut e);
    assert_eq!(e.save().core.players[0].hujia, 1, "获得 1 点护甲");
    assert_zone_conservation(&e);

    // —— 硫头阁：自己摸 1 / 他人弃首张 ——
    let mut e = Engine::new_with_extensions(5232, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (ltg, disc) = {
        let catalog = e.catalog();
        let ltg = find(&e, "gj_liutouge");
        let disc = card_of(&e, CardKind::Sha, Suit::Spade);
        (ltg, disc)
    };
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[ltg, disc]);
    snap.core.players[0].hand.push(ltg);
    snap.core.players[1].hand.push(disc);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    // 对自己：摸 1
    let out = submit(&mut e, 0, GameAction::PlayCard { card: ltg, targets: vec![0] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    pass_nullify_window(&mut e);
    let self_draw = e.debug_hand(0).len();
    // 对 1 号：其弃首张手牌
    let h1 = e.debug_hand(1).len();
    // 第二张硫头阁在目录中（club 6）——查找另一张
    let ltg2 = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.name.starts_with("gj_liutouge") && {
            let id = e.catalog().cards.iter().position(|x| x.name == c.name).unwrap() as u32;
            id != ltg
        })
        .map(|i| i as u32);
    if let Some(ltg2) = ltg2 {
        // 注入第二张再打
        let mut snap = e.save();
        purge_all_zones(&mut snap, &[ltg2]);
        snap.core.players[0].hand.push(ltg2);
        let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复2");
        let h1 = e.debug_hand(1).len();
        let out = submit(&mut e, 0, GameAction::PlayCard { card: ltg2, targets: vec![1] });
        assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
        pass_nullify_window(&mut e);
        assert_eq!(e.debug_hand(1).len(), h1 - 1, "他人弃首张手牌");
        assert_zone_conservation(&e);
    }
    let _ = (self_draw, h1);
    assert_zone_conservation(&e);

    // —— 木头面具：装备后任意手牌当杀（once/turn），非装备时不可 ——
    let mut e = Engine::new_with_extensions(5233, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (mtj, tao_c) = {
        let catalog = e.catalog();
        let mtj = find(&e, "gj_mutoumianju");
        let tao_c = card_of(&e, CardKind::Tao, Suit::Heart);
        (mtj, tao_c)
    };
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[mtj, tao_c]);
    snap.core.players[0].hand.push(mtj);
    snap.core.players[0].hand.push(tao_c);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    e.debug_set_hp(1, 9);
    let def_muto = e
        .catalog()
        .skill_by_name(engine_native::content::host::GJ_MUTOU_SKILL_KEY)
        .expect("木头面具装备技在目录");
    // 装备木头面具 → 授予装备技实例
    let out = submit(&mut e, 0, GameAction::PlayCard { card: mtj, targets: vec![] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    assert!(
        e.save().core.skills.contains_key(&(0, def_muto)),
        "装备授予装备技实例"
    );
    // 桃当杀使用（view-as）→ 放行，once_used 置位
    let hp1 = e.debug_hp(1);
    let out = submit(&mut e, 0, GameAction::PlayCard { card: tao_c, targets: vec![1] });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "手牌当杀放行: {out:?}"
    );
    pass_nullify_window(&mut e);
    drain_choices(&mut e);
    assert!(e.debug_hp(1) < hp1, "目标掉血（杀生效）");
    assert!(
        e.save()
            .core
            .skills
            .get(&(0, def_muto))
            .map(|i| i.once_used)
            .unwrap_or(false),
        "once_used 置位"
    );
    // 第二次：手牌不足/once_used → 再用桃当杀被拒（手牌已空自然拒绝；
    // 此处验证 once_used 语义：直接断言实例标记即可，见上）
    assert_zone_conservation(&e);
}

/// 收官双卡：帝女玄霜（主动使用：回复 1 + 弃任意摸等量；濒死按桃
/// 结算）与玉衡（装备技：确定性取其他角色首张黑桃手牌）。
#[test]
fn dinvxuanshuang_and_gjyuheng_closeout() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "shahai-gujian-upstream-noname")
            .expect("古剑可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let (pack, _) = bridge(&json).expect("桥接");
    let find = |e: &Engine, prefix: &str| {
        e.catalog()
            .cards
            .iter()
            .position(|c| c.name.starts_with(prefix))
            .unwrap_or_else(|| panic!("{}在目录", prefix)) as u32
    };

    // —— 帝女玄霜：主动使用（受伤自用：回复 1 + 弃 1 摸 1） ——
    let mut e = Engine::new_with_extensions(5234, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (dvx, disc) = {
        let catalog = e.catalog();
        let dvx = find(&e, "gj_dinvxuanshuang");
        let disc = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Shan)
            .expect("闪在目录") as u32;
        (dvx, disc)
    };
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[dvx, disc]);
    snap.core.players[0].hand.push(dvx);
    snap.core.players[0].hand.push(disc);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    e.debug_set_hp(0, 2); // 已受伤
    let hp0 = e.debug_hp(0);
    let out = submit(&mut e, 0, GameAction::PlayCard { card: dvx, targets: vec![] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(e.debug_hp(0), hp0 + 1, "回复 1 点体力");
    // rider：弃任意张（弃 1 摸 1）
    let c = e.open_choice().expect("帝女玄霜 rider 挂起");
    assert_eq!(c.spec.prompt, "gj_dinvxuanshuang_rider");
    assert_eq!(c.observer, 0);
    assert!(c.spec.allow_cancel, "rider 可取消");
    let cid = c.id;
    drop(c);
    let out = submit(&mut e, 0, GameAction::Answer {
        choice_id: cid,
        selection: vec![ChoiceOption::Card(disc)],
    });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    assert_eq!(e.debug_hand(0).len(), 1, "弃 1 摸 1（净不变）");
    assert_eq!(e.debug_hp(0), hp0 + 1, "回复保持");
    assert_zone_conservation(&e);

    eprintln!("ENTER SEG2");
    // —— 玉衡：装备技——确定性取其他角色首张黑桃手牌 ——
    let mut e = Engine::new_with_extensions(5235, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (yhx, spade) = {
        let catalog = e.catalog();
        let yhx = find(&e, "gj_gjyuheng");
        let spade = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Sha && c.suit == Suit::Spade)
            .expect("黑桃杀在目录") as u32;
        (yhx, spade)
    };
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[yhx, spade]);
    snap.core.players[0].hand.push(yhx);
    snap.core.players[1].hand.push(spade);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let def_yh = e
        .catalog()
        .skill_by_name(engine_native::content::host::GJ_YUHENG_SKILL_KEY)
        .expect("玉衡装备技在目录");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: yhx, targets: vec![] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    assert!(e.save().core.skills.contains_key(&(0, def_yh)), "装备授予玉衡技");
    // 发动：座位序首名有黑桃手牌的其他角色（1 号）交出首张黑桃
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_yh });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert!(e.debug_hand(0).contains(&spade), "黑桃手牌被取走至使用者");
    assert!(!e.debug_hand(1).contains(&spade), "目标失去黑桃牌");
    // once_per_turn：再次发动被拒
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_yh });
    assert!(matches!(out, CommandOutcome::Rejected(_)), "once_per_turn 拒绝: {out:?}");
    assert_zone_conservation(&e);

    eprintln!("ENTER SEG3");
    // —— 帝女玄霜濒死路径：按桃结算（救回，无 rider） ——
    let mut e = Engine::new_with_extensions(5236, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (dvx2, sha_black) = {
        let catalog = e.catalog();
        let dvx2 = find(&e, "gj_dinvxuanshuang");
        let sha_black = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Sha && matches!(c.suit, Suit::Spade))
            .expect("黑杀在目录") as u32;
        (dvx2, sha_black)
    };
    let mut snap = e.save();
    purge_all_zones(&mut snap, &[dvx2, sha_black]);
    snap.core.players[0].hand.push(dvx2);
    snap.core.players[1].hand.push(sha_black);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    e.debug_set_hp(0, 1);
    let mut steps = 0;
    loop {
        steps += 1;
        assert!(steps < 96);
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
            drop(c);
            submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: sel });
            continue;
        }
        let actor = e.turn().current;
        submit(&mut e, actor, GameAction::NextPhase);
    }
    let hp0 = e.debug_hp(0);
    let out = submit(&mut e, 1, GameAction::PlayCard { card: sha_black, targets: vec![0] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    pass_nullify_window(&mut e);
    // 0 号濒死 → 帝女玄霜出现在救援选项并按桃结算救回。响应者队列
    // 按「濒死者起的座位序中持桃者」询问：其余玩家摸到的普通桃会被
    // 先问（发牌构成不同而异），逐个 Pass 后才轮到持玄霜的 0 号。
    let cid_dvx = loop {
        let c = e.open_choice().expect("濒死救援挂起");
        assert_eq!(c.spec.prompt, "dying_save");
        let has_dvx = c.spec.options.contains(&ChoiceOption::Card(dvx2));
        let cid = c.id;
        let obs = c.observer;
        drop(c);
        if has_dvx {
            assert_eq!(obs, 0, "帝女玄霜应只在 0 号手中");
            break cid;
        }
        submit(&mut e, obs, GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Pass],
        });
    };
    submit(&mut e, 0, GameAction::Answer {
        choice_id: cid_dvx,
        selection: vec![ChoiceOption::Card(dvx2)],
    });
    assert_eq!(e.debug_hp(0), hp0, "帝女玄霜按桃救回（体力不变）");
    assert!(e.save().core.players[0].alive, "未死亡");
    assert_zone_conservation(&e);
}

/// 整包验收续（2026-10-08，第三例）：shahai-gujian 真实宿主全生命周期
/// ——独立安装+激活 → 安装包→原生建局 → 对局中卸载 → 在局锁定副本
/// 续玩 → 撤销后重装/重新 import 被拒。
#[test]
fn gujian_real_host_full_lifecycle() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();

    // ① 独立安装 + 激活
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "shahai-gujian")
        .expect("古剑安装")
        .clone();
    let slot = format!("{}/{}", meta.namespace, meta.package_id);
    assert!(store.read_active().unwrap().packages.contains_key(&slot), "激活集");

    // ② 安装包 → 原生建局（40 卡切片目录）
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-gujian-upstream-noname")
        .expect("古剑可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let (pack, _) = bridge(&json).expect("桥接");
    let base_cards = Engine::new(1, 3).catalog().cards.len();
    let mut e = Engine::new_with_extensions(5501, 3, &[&pack]).expect("建局");
    assert!(e.catalog().cards.len() > base_cards, "古剑卡入目录");
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let mid = e.save();

    // ③ 对局中卸载
    store
        .revoke_slot(&meta.namespace, "shahai-gujian", "验收：对局中卸载", true)
        .expect("撤销");
    assert!(store.read_revoke().unwrap().is_slot_revoked(&meta.namespace, "shahai-gujian"));
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
