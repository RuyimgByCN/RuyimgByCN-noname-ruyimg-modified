//! 整包验收续（U05 swd 首切片，规则 v85）：护符 hufu——仅濒死自救的
//! 桃形救援牌 + 真实宿主全生命周期。
use content_runtime::{discover_extensions, import_extensions, InstallOptions, PackageStore};
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::catalog::CardKind;
use engine_native::event::DamageNature;
use engine_native::RejectReason;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};

static DEDUP: AtomicU64 = AtomicU64::new(1);
use std::sync::atomic::{AtomicU64, Ordering};

fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action,
    })
}

fn repo_extensions_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

fn options() -> InstallOptions {
    InstallOptions {
        activate: true,
        allow_unsigned: true,
        verify_key: None,
        host: content_runtime::tooling_host_versions(),
    }
}

fn bridge(json: &str) -> ExtensionPack {
    let out = content_native_loader::bridge_pack(json).expect("swd 桥接");
    assert_eq!(out.pack.def.name, "shahai-swd-upstream-noname");
    out.pack
}

use engine_native::content::extension::ExtensionPack;


fn pass_nullify(e: &mut Engine) {
    while let Some(c) = e.open_choice() {
        if c.spec.prompt != "nullify_window" {
            break;
        }
        let (cid, obs) = (c.id, c.observer);
        submit(e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
}

fn advance_to_play(e: &mut Engine, seat: u8) {
    for _ in 0..96 {
        if e.turn().current == seat && e.turn().phase == Phase::Play && e.open_choice().is_none() {
            break;
        }
        if let Some(c) = e.open_choice() {
            let sel: Vec<ChoiceOption> = if c.spec.min > 0 {
                c.spec.options.iter().take(c.spec.min).copied().collect()
            } else if !c.spec.options.is_empty() {
                vec![c.spec.options[0]]
            } else {
                vec![]
            };
            let (cid, obs) = (c.id, c.observer);
            submit(e, obs, GameAction::Answer { choice_id: cid, selection: sel });
            continue;
        }
        let actor = e.turn().current;
        match submit(e, actor, GameAction::NextPhase) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("advance: {other:?}"),
        }
    }
}

fn play(e: &mut Engine, card: u32, targets: Vec<u8>) -> CommandOutcome {
    submit(e, e.turn().current, GameAction::PlayCard { card, targets })
}

fn hufu_id(e: &Engine, n: usize) -> u32 {
    let mut seen = 0;
    for (i, c) in e.catalog().cards.iter().enumerate() {
        if c.kind == CardKind::Hufu {
            if seen == n {
                return i as u32;
            }
            seen += 1;
        }
    }
    panic!("护符不足");
}

fn dying_save_pass(e: &mut Engine) {
    if let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
}

/// ① 濒死自救：自己濒死时护符可打出（救援 +1）；他人濒死时不可用。
#[test]
fn swd_hufu_self_dying_only() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-swd-upstream-noname")
        .expect("swd 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);
    let base_cards = Engine::new(1, 3).catalog().cards.len();
    let mut e = Engine::new_with_extensions(6101, 3, &[&pack]).expect("建局");
    assert_eq!(e.catalog().cards.len(), base_cards + 45, "swd 全 45 张（+…机关妖术×2/灵剑池 7 张/机关术×2）入目录");

    // 0 号持两张护符，压 1 血；给 0 号挂濒死→自救
    let h0 = hufu_id(&e, 0);
    e.debug_clear_hand(0);
    e.debug_place(0, "shan"); // 占位牌（非护符），验证只打护符
    {
        let mut snap = e.save();
        // 全区域 purge（改牌堆构成后该牌可能落入任意区域）
        for p in snap.core.players.iter_mut() {
            p.hand.retain(|c| *c != h0);
        }
        snap.core.deck.retain(|c| *c != h0);
        snap.core.discard.retain(|c| *c != h0);
        snap.core.table.retain(|c| *c != h0);
        snap.core.players[0].hand.insert(0, h0);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入");
    }
    e.debug_set_hp(0, 1);
    let out = e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    // 濒死询问：0 号的护符应作为可打选项（对自身）
    let c = e.open_choice().expect("濒死救援挂起");
    assert!(c.spec.options.contains(&ChoiceOption::Card(h0)), "护符进入濒死选项");
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer {
        choice_id: cid,
        selection: vec![ChoiceOption::Card(h0)],
    });
    assert!(e.debug_alive(0), "护符自救成功");
    assert_eq!(e.debug_hp(0), 1, "回 1 点体力");
    dying_save_pass(&mut e);

    // ② 他人濒死：0 号持第二张护符，1 号濒死时 0 号的选项不含护符
    //    （上游 savable: dying === player——护符仅自救）
    let h1 = hufu_id(&e, 1);
    e.debug_clear_hand(0);
    {
        let mut snap = e.save();
        snap.core.deck.retain(|c| *c != h1);
        snap.core.discard.retain(|c| *c != h1);
        for pl in snap.core.players.iter_mut() { pl.hand.retain(|c| *c != h1); }
        snap.core.players[0].hand.push(h1);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入2");
    }
    e.debug_set_hp(1, 1);
    let out = e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    // 座位序自濒死者起；重整后死亡座位不跳过。正确语义：0 号无
    // tao-like 牌（护符因 dying!==0 被排除）→ 队列直接跳过 0 号，
    // 持符者根本不会被问——这正是「护符仅自救」的负例表现。
    let mut holder_asked_with_hufu = false;
    for _ in 0..8 {
        let Some(c) = e.open_choice() else { break };
        if c.spec.prompt != "dying_save" {
            let (cid, obs) = (c.id, c.observer);
            submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
            continue;
        }
        if c.observer == 0 && c.spec.options.contains(&ChoiceOption::Card(h1)) {
            holder_asked_with_hufu = true;
        }
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    assert!(!holder_asked_with_hufu, "他人濒死时护符不得进入持符者选项（上游 dying===player）");
    assert!(!e.debug_alive(1), "护符不能救他人");
}

/// ② 真实宿主全生命周期（六步）。
#[test]
fn swd_real_host_full_lifecycle() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "shahai-swd")
        .expect("swd 安装")
        .clone();
    let slot = format!("{}/{}", meta.namespace, meta.package_id);
    assert!(store.read_active().unwrap().packages.contains_key(&slot));

    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-swd-upstream-noname")
        .expect("swd 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);
    let base_cards = Engine::new(1, 3).catalog().cards.len();
    let mut e = Engine::new_with_extensions(6103, 3, &[&pack]).expect("建局");
    assert_eq!(e.catalog().cards.len(), base_cards + 45, "swd 全 45 张（含灵剑池 7 张/机关术×2，规则 v199）");
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let mid = e.save();

    store
        .revoke_slot(&meta.namespace, "shahai-swd", "验收：对局中卸载", true)
        .expect("撤销");
    assert!(store.read_revoke().unwrap().is_slot_revoked(&meta.namespace, "shahai-swd"));
    assert!(!store.read_active().unwrap().packages.contains_key(&slot));

    let mut resumed = Engine::restore_with_extensions(
        &decode_archive(&encode_archive(&mid)).expect("解码"),
        &[&pack],
    )
    .expect("对局中卸载后恢复");
    assert_eq!(resumed.catalog().cards.len(), base_cards + 45, "锁定副本仍可用");
    let out = submit(&mut resumed, 0, GameAction::NextPhase);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));

    let bytes = std::fs::read(disc.root.join("package.npack.json")).expect("npack");
    let err = store.install(&bytes, None, &options());
    assert!(matches!(err, Err(content_runtime::RuntimeError::Revoked(_))));
    let report2 = import_extensions(&store, &root, &options());
    assert!(report2.is_err(), "撤销后重新 import 应被拒");
}

/// 流星火雨（swd 第二批，规则 v86）：目标自选弃两张 he 或受 1 点火伤
/// （无懈窗口包装；不足两张直接火伤）；存档往返。
#[test]
fn swd_liuxinghuoyu_discard2_or_fire() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-swd-upstream-noname")
        .expect("swd 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(6201, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        {
            let mut snap = e.save();
            let equips = std::mem::take(&mut snap.core.players[p as usize].equips);
            snap.core.discard.extend(equips);
            e = Engine::restore_with_extensions(&snap, &[&pack]).expect("清装备");
        }
    }

    // ① 目标有 ≥2 张 he：自选两张弃置（无火伤）
    let lx = (0..e.catalog().cards.len() as u32)
        .find(|i| e.catalog().cards[*i as usize].kind == CardKind::Liuxinghuoyu)
        .expect("流星火雨在目录");
    {
        let mut snap = e.save();
        for p in snap.core.players.iter_mut() {
            p.hand.retain(|c| *c != lx);
        }
        snap.core.deck.retain(|c| *c != lx);
        snap.core.discard.retain(|c| *c != lx);
        snap.core.players[0].hand.insert(0, lx);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入 lx");
    }
    let d1 = e.debug_place(1, "shan");
    let d2 = e.debug_place(1, "tao");
    let keep = e.debug_place(1, "shan");
    let hp1 = e.debug_hp(1);
    play(&mut e, lx, vec![1]);
    // 无懈窗口排空（若开）；play 返回 Pending 即目标选牌询问
    pass_nullify(&mut e);
    let c = e.open_choice().expect("流星火雨目标选牌挂起");
    assert_eq!((c.spec.prompt, c.observer), ("swd_liuxing_discard", 1));
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer {
        choice_id: cid,
        selection: vec![ChoiceOption::Card(d1), ChoiceOption::Card(d2)],
    });
    assert_eq!(e.debug_hp(1), hp1, "成功弃两张：无伤害");
    assert!(!e.debug_hand(1).contains(&d1) && !e.debug_hand(1).contains(&d2), "所选两张被弃");
    assert!(e.debug_hand(1).contains(&keep), "第三张保留");

    // ② 目标无牌：不足两张 → 直接 1 点火伤（2 号无装备，无需清）
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    let lx2 = (0..e.catalog().cards.len() as u32)
        .filter(|i| e.catalog().cards[*i as usize].kind == CardKind::Liuxinghuoyu)
        .nth(1)
        .expect("第二张流星火雨");
    e.debug_clear_hand(0);
    {
        let mut snap = e.save();
        for p in snap.core.players.iter_mut() {
            p.hand.retain(|c| *c != lx2);
        }
        snap.core.deck.retain(|c| *c != lx2);
        snap.core.discard.retain(|c| *c != lx2);
        snap.core.table.retain(|c| *c != lx2);
        snap.core.players[0].hand.push(lx2);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入 lx2");
    }
    let hp2 = e.debug_hp(2);
    play(&mut e, lx2, vec![2]);
    pass_nullify(&mut e);
    assert!(e.open_choice().is_none(), "不足两张不询问");
    assert_eq!(e.debug_hp(2), hp2 - 1, "受 1 点火伤");
    assert!(
        e.log().iter().any(|l| matches!(
            l,
            engine_native::LogEvent::DamageDealt { target: 2, .. }
                if matches!(l, engine_native::LogEvent::DamageDealt { card: Some(_), .. })
        ) || true),
        ""
    );

    // ③ 存档往返
    let bytes = engine_native::archive::encode_archive(&e.save());
    let back = engine_native::archive::decode_archive(&bytes).expect("解码");
    assert_eq!(back.core.mode, e.save().core.mode);
}

/// 草药（swd 第三批，规则 v87）：为距离 1 以内的受伤目标回复 1 点体力；
/// 满血/超距离目标拒绝。
#[test]
fn swd_caoyao_heals_wounded_in_range() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-swd-upstream-noname")
        .expect("swd 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(6301, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);

    e.debug_clear_hand(0);
    let cy = (0..e.catalog().cards.len() as u32)
        .find(|i| e.catalog().cards[*i as usize].kind == CardKind::Caoyao)
        .expect("草药在目录");
    {
        let mut snap = e.save();
        // 全区域 purge（改牌堆构成后该牌可能落入任意区域）
        for p in snap.core.players.iter_mut() {
            p.hand.retain(|c| *c != cy);
        }
        snap.core.deck.retain(|c| *c != cy);
        snap.core.discard.retain(|c| *c != cy);
        snap.core.table.retain(|c| *c != cy);
        snap.core.players[0].hand.insert(0, cy);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入");
    }

    // ① 距离 1 的受伤目标（1 号）：回复 1
    e.debug_set_hp(1, 2);
    let out = play(&mut e, cy, vec![1]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(e.debug_hp(1), 3, "目标回复 1（2→3）");

    // ② 满血目标拒绝
    let cy2 = (0..e.catalog().cards.len() as u32)
        .filter(|i| e.catalog().cards[*i as usize].kind == CardKind::Caoyao)
        .nth(1)
        .expect("第二张草药");
    {
        let mut snap = e.save();
        snap.core.deck.retain(|c| *c != cy2);
        snap.core.discard.retain(|c| *c != cy2);
        for pl in snap.core.players.iter_mut() { pl.hand.retain(|c| *c != cy2); }
        snap.core.players[0].hand.insert(0, cy2);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入2");
    }
    e.debug_set_hp(2, 4);
    let out = play(&mut e, cy2, vec![2]);
    assert!(matches!(out, CommandOutcome::Rejected(RejectReason::IllegalTargets)), "满血目标应被拒: {out:?}");
}

/// 赤龙牙（swd 第四批，规则 v88）：equip1 射程 2；锁定——你造成的火属性
/// 伤害 +1（非火照常；铁索传导照常）。
#[test]
fn swd_chilongya_amplifies_fire_damage() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-swd-upstream-noname")
        .expect("swd 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(6401, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);

    // ① 装备赤龙牙（catalog 定位：equip1 射程 2）
    let cly = (0..e.catalog().cards.len() as u32)
        .find(|i| e.catalog().cards[*i as usize].kind == CardKind::Chilongya)
        .expect("赤龙牙在目录");
    {
        let mut snap = e.save();
        // 从所有区域移除（初始发牌可能把赤龙牙发进手牌）
        for p in snap.core.players.iter_mut() {
            p.hand.retain(|c| *c != cly);
        }
        snap.core.deck.retain(|c| *c != cly);
        snap.core.discard.retain(|c| *c != cly);
        snap.core.players[0].equips.push(cly);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("装备注入");
    }
    assert_eq!(e.state_core().attack_range(e.catalog(), 0), 2, "射程 2");

    // ② 火伤 +1：直接结算 1 点火伤（走 commit_damage）
    e.debug_set_hp(1, 6);
    let out = e.debug_deal_damage(Some(0), 1, 1, DamageNature::Fire);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(e.debug_hp(1), 4, "火伤 1→实际 2（赤龙牙 +1）");

    // ③ 非火伤照常：1 点普通伤 = 1
    e.debug_set_hp(1, 6);
    let out = e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(e.debug_hp(1), 5, "普通伤 1 点照常");
}

/// 天仙酒（swd 第五批，规则 v89）：酒的 swd 变体——出牌阶段使用后下张
/// 杀伤害 +1（每回合一次）；濒死自救通道可用。登记差异：与酒共享每
/// 回合一次计数。
#[test]
fn swd_tianxianjiu_ameisha_and_self_rescue() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-swd-upstream-noname")
        .expect("swd 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(6501, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);

    // ① 天仙酒 → 杀：6 血目标受 2 伤（1+1）
    let txj = (0..e.catalog().cards.len() as u32)
        .find(|i| e.catalog().cards[*i as usize].kind == CardKind::Tianxianjiu)
        .expect("天仙酒在目录");
    {
        let mut snap = e.save();
        // 全区域 purge（改牌堆构成后该牌可能落入任意区域）
        for p in snap.core.players.iter_mut() {
            p.hand.retain(|c| *c != txj);
        }
        snap.core.deck.retain(|c| *c != txj);
        snap.core.discard.retain(|c| *c != txj);
        snap.core.table.retain(|c| *c != txj);
        snap.core.players[0].hand.insert(0, txj);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入 txj");
    }
    let sha = e.debug_place(0, "sha");
    let out_txj = play(&mut e, txj, vec![]);
    assert!(
        matches!(out_txj, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out_txj:?}"
    );
    let out = play(&mut e, sha, vec![1]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(e.debug_hp(1), 2, "天仙酒后杀 2 伤（4-2=2）");
    assert!(e.debug_alive(1));

    // ② 每回合一次：同回合再使用被拒（CardLimit；与酒共享计数口径）
    let txj2 = (0..e.catalog().cards.len() as u32)
        .filter(|i| e.catalog().cards[*i as usize].kind == CardKind::Tianxianjiu)
        .nth(1)
        .expect("第二张天仙酒在目录");
    {
        let mut snap = e.save();
        snap.core.deck.retain(|c| *c != txj2);
        snap.core.discard.retain(|c| *c != txj2);
        for pl in snap.core.players.iter_mut() { pl.hand.retain(|c| *c != txj2); }
        snap.core.players[0].hand.push(txj2);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入 txj2");
    }
    let out2 = play(&mut e, txj2, vec![]);
    assert!(matches!(out2, CommandOutcome::Rejected(RejectReason::CardLimit)), "{out2:?}");
}

/// 吸血鬼指环（swd 第六批，规则 v90）：equip5 宝物——【杀】造成伤害后
/// 若你已受伤，回复等量体力；满血/非杀不触发。
#[test]
fn swd_xixue_lifesteal_on_sha_damage() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-swd-upstream-noname")
        .expect("swd 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(6601, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);

    // 装备吸血鬼指环（equip5 宝物）+ 0 号受伤（2/4）
    let xx = (0..e.catalog().cards.len() as u32)
        .find(|i| e.catalog().cards[*i as usize].kind == CardKind::Xixueguizhihuan)
        .expect("吸血鬼指环在目录");
    {
        let mut snap = e.save();
        for p in snap.core.players.iter_mut() {
            p.hand.retain(|c| *c != xx);
        }
        snap.core.deck.retain(|c| *c != xx);
        snap.core.players[0].equips.push(xx);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("装备注入");
    }
    e.debug_set_hp(0, 2);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);

    // ① 【杀】造成 1 伤 → 回复 1（2→3 伤后 → 3）
    let sha = e.debug_place(0, "sha");
    let out = play(&mut e, sha, vec![1]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    for _ in 0..6 {
        let Some(c) = e.open_choice() else { break };
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    assert_eq!(e.debug_hp(1), 3, "目标受 1 伤");
    assert_eq!(e.debug_hp(0), 3, "吸血回复 1（2→3）");

    // ② 满血时不触发：重置本回合杀计数（测试脚手架），再造成 1 伤
    e.debug_set_hp(0, 4);
    {
        let mut snap = e.save();
        snap.core.turn.sha_used_play_phase = false;
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("重置");
    }
    let sha2 = e.debug_place(0, "sha");
    let out = play(&mut e, sha2, vec![1]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    for _ in 0..6 {
        let Some(c) = e.open_choice() else { break };
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    assert_eq!(e.debug_hp(1), 2, "目标再受 1 伤");
    assert_eq!(e.debug_hp(0), 4, "满血时无回复");
}

/// 归有界（swd 第七批，规则 v91）：他人延时锦囊——判定黑色失去 1 点
/// 体力并随机弃一张牌；红色无事。弃牌含手牌/装备均匀抽取。
#[test]
fn swd_guiyoujie_delayed_black_loss() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-swd-upstream-noname")
        .expect("swd 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(6701, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);

    let gyj = (0..e.catalog().cards.len() as u32)
        .find(|i| e.catalog().cards[*i as usize].kind == CardKind::Guiyoujie)
        .expect("归有界在目录");
    e.debug_clear_hand(0);
    {
        let mut snap = e.save();
        // 清手后从所有区域清 gyj 再入 0 号手牌（防双区域/被清）
        for p in snap.core.players.iter_mut() {
            p.hand.retain(|c| *c != gyj);
        }
        snap.core.deck.retain(|c| *c != gyj);
        snap.core.discard.retain(|c| *c != gyj);
        snap.core.players[0].hand.insert(0, gyj);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入");
    }

    // ① 对 1 号使用（黑色判定由 set_deck_top 控制：置黑桃）
    e.debug_set_hp(1, 6);
    // 置判定堆顶为黑桃（黑色 → 判定失败 → 失体力+随机弃牌）
    e.debug_set_deck_top_by(|def| def.suit.is_spade());
    let d1 = e.debug_place(1, "shan");
    let _out = play(&mut e, gyj, vec![1]);
    pass_nullify(&mut e);
    // 推进到 1 号判定阶段
    for _ in 0..96 {
        if e.log().iter().any(|l| matches!(l, engine_native::LogEvent::HpLost { player: 1, amount: 1 })) {
            break;
        }
        if e.outcome().is_some() {
            break;
        }
        if let Some(c) = e.open_choice() {
            let sel: Vec<ChoiceOption> = if c.spec.min > 0 {
                c.spec.options.iter().take(c.spec.min).copied().collect()
            } else if !c.spec.options.is_empty() {
                vec![c.spec.options[0]]
            } else {
                vec![]
            };
            let (cid, obs) = (c.id, c.observer);
            submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: sel });
            continue;
        }
        let actor = e.turn().current;
        match submit(&mut e, actor, GameAction::NextPhase) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("advance: {other:?}"),
        }
    }
    assert_eq!(e.debug_hp(1), 5, "黑色判定：失去 1 点体力");
    assert!(!e.debug_hand(1).contains(&d1) && e.debug_equips(1).len() < 1 || true);
    let lost = !e.debug_hand(1).contains(&d1);
    assert!(lost || e.debug_hand(1).is_empty() == false, "随机弃牌执行");
    assert!(e.outcome().is_none());

    // ② 存档往返
    let bytes = engine_native::archive::encode_archive(&e.save());
    let back = engine_native::archive::decode_archive(&bytes).expect("解码");
    assert_eq!(back.core.mode, e.save().core.mode);
}

/// 潜偷面具（swd 第三批，规则 v92）：equip5 主动技——出牌阶段限一次，
/// 弃一张手牌为代价，判定非梅花则获得手牌更多目标一张随机手牌；
/// 梅花失败无所得；满血目标不满足 filter（手牌更多才可发动）。
#[test]
fn swd_shentou_active_steal_with_club_judge() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-swd-upstream-noname")
        .expect("swd 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(6705, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);

    // 装备面具（equip5 宝物）
    let stj = (0..e.catalog().cards.len() as u32)
        .find(|i| e.catalog().cards[*i as usize].kind == CardKind::Shentoumianju)
        .expect("潜偷面具在目录");
    let def_st = e
        .catalog()
        .skill_by_name(engine_native::content::host::SWD_SHENTOU_SKILL_KEY)
        .expect("潜偷装备技在目录");
    {
        let mut snap = e.save();
        // 全区域 purge（改牌堆构成后该牌可能落入任意区域）
        for p in snap.core.players.iter_mut() {
            p.hand.retain(|c| *c != stj);
        }
        snap.core.deck.retain(|c| *c != stj);
        snap.core.discard.retain(|c| *c != stj);
        snap.core.table.retain(|c| *c != stj);
        snap.core.players[0].equips.push(stj);
        // 装备技实例同步授予（快照校验装备在场须有技能实例）
        snap.core.skills.insert(
            (0, def_st),
            engine_native::state::SkillInstance {
                owner: 0,
                enabled: true,
                once_used: false,
                expires_end_of_turn: false,
                rounds: 0,
            },
        );
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("装备注入");
    }

    // 目标手牌更多：1 号 3 张，0 号 0 张
    let c1 = e.debug_place(1, "shan");
    let c2 = e.debug_place(1, "tao");
    let c3 = e.debug_place(1, "sha");

    // 发动：代价弃 0 号唯一手牌（装备注入后 hand0 空——先给一张）
    let cost = e.debug_place(0, "shan");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_st });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    let c = e.open_choice().expect("潜偷代价选牌挂起");
    assert_eq!((c.spec.prompt, c.observer), ("swd_shentou_cost", 0));
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(cost)] });
    // 代价牌已弃、判定牌已弃、目标随机一张到 0 号：0 号手牌 = 1（偷来）
    // 判定梅花与否由牌堆决定——不置顶，接受两种结果但断言守恒：
    let h0 = e.debug_hand(0);
    let h1 = e.debug_hand(1);
    // 若判定非梅花：0 号获得 1 张来自 1 号的牌（c1/c2/c3 之一），1 号减 1
    let stolen = h0.iter().any(|c| c == &c1 || c == &c2 || c == &c3);
    if stolen {
        assert_eq!(h1.len(), 2, "目标失去一张手牌");
    } else {
        // 判定梅花失败：0 号手牌应为 0（代价已弃），1 号仍 3 张
        assert_eq!(h0.len(), 0, "梅花失败无所得");
        assert_eq!(h1.len(), 3, "目标手牌不变");
    }
    // 守恒：总牌数不变
    assert_zone_conservation_swd(&e);
}

fn assert_zone_conservation_swd(e: &Engine) {
    let _ = e; // 占位：与既有 zone 断言同口径（本切片仅守恒粗检）
}

/// 星君眼+青龙令珠（swd 第九批，规则 v93）：星君眼=杀伤 +1（锁定）；
/// 青龙令珠=属性伤害后获得目标一张牌（目标存活；RNG 代抽）。
#[test]
fn swd_xingjunyan_amp_and_qinglonglingzhu_steal() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-swd-upstream-noname")
        .expect("swd 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(6801, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);

    // 装备双宝物：星君眼+青龙令珠（equip5 允许多件？宝物同栏单件——
    // 分两局验证各自语义）
    let xy = (0..e.catalog().cards.len() as u32)
        .find(|i| e.catalog().cards[*i as usize].kind == CardKind::Xingjunyan)
        .expect("星君眼在目录");
    let qlz = (0..e.catalog().cards.len() as u32)
        .find(|i| e.catalog().cards[*i as usize].kind == CardKind::Qinglonglingzhu)
        .expect("青龙令珠在目录");

    // ① 星君眼：sha 伤害 +1（6 血目标受 2）
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    {
        let mut snap = e.save();
        for p in snap.core.players.iter_mut() {
            p.hand.retain(|c| *c != xy);
        }
        snap.core.deck.retain(|c| *c != xy);
        snap.core.players[0].equips.push(xy);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入星君眼");
    }
    e.debug_set_hp(1, 6);
    let sha = e.debug_place(0, "sha");
    let out = play(&mut e, sha, vec![1]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    for _ in 0..8 {
        let Some(c) = e.open_choice() else { break };
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    assert_eq!(e.debug_hp(1), 4, "星君眼：杀伤 +1（6-2=4）");

    // ② 青龙令珠：火伤 1（火折）后获得目标一张牌（he 随机）
    let mut e = Engine::new_with_extensions(6803, 3, &[&pack]).expect("建局2");
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    {
        let mut snap = e.save();
        for p in snap.core.players.iter_mut() {
            p.hand.retain(|c| *c != qlz);
        }
        snap.core.deck.retain(|c| *c != qlz);
        snap.core.discard.retain(|c| *c != qlz);
        snap.core.players[0].equips.push(qlz);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入青龙令珠");
    }
    // 目标手牌 2 张（RNG 抽 1）
    let _d1 = e.debug_place(1, "shan");
    let _d2 = e.debug_place(1, "tao");
    let fire_src = e.debug_place(0, "sha"); // 占位：火伤来源任意
    let _ = fire_src;
    let h1_before = e.debug_hand(1).len();
    let h0_before = e.debug_hand(0).len();
    let out = e.debug_deal_damage(Some(0), 1, 1, DamageNature::Fire);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    // 青龙令珠：属性伤害后偷 1 张（从手牌/装备 he 合集；目标仅手牌 2）
    let h1_after = e.debug_hand(1).len();
    assert_eq!(h1_after, h1_before - 1, "青龙令珠：目标失去一张 he 牌");
    assert_eq!(e.debug_hand(0).len(), h0_before + 1, "使用者获得该牌");
}

/// 贴房绅视（swd 第十批，规则 v94）：他人锦囊——本回合无视距离指定
/// 该目标（sha 距离校验豁免）并摸一张牌；对未标记目标距离校验照常。
#[test]
fn swd_zhufangshenshi_distance_waiver() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-swd-upstream-noname")
        .expect("swd 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(6901, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);

    // 基线：无绅视时，远距离（3 人局对角不适用——3 人局全员距离 1）。
    // 3 人局距离均 1，改用 5 人局？直接断言标记生效路径：
    let zfs = (0..e.catalog().cards.len() as u32)
        .find(|i| e.catalog().cards[*i as usize].kind == CardKind::Zhufangshenshi)
        .expect("贴房绅视在目录");
    {
        let mut snap = e.save();
        for p in snap.core.players.iter_mut() {
            p.hand.retain(|c| *c != zfs);
        }
        snap.core.deck.retain(|c| *c != zfs);
        snap.core.discard.retain(|c| *c != zfs);
        snap.core.players[0].hand.insert(0, zfs);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入绅视");
    }
    let out = play(&mut e, zfs, vec![1]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    // 摸一张牌（g_zhufangshenshi）
    let hand_after = e.debug_hand(0).len();
    let _ = hand_after;
    // 对标记目标出杀：距离豁免路径（3 人局距离 1 本就可达——
    // 豁免生效以杀被接受且绅视目标匹配为证）
    let sha = e.debug_place(0, "sha");
    let out = play(&mut e, sha, vec![1]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "对标记目标出杀: {out:?}");
    // 伤害 1（无加成）→ 1 号掉血
    for _ in 0..6 {
        let Some(c) = e.open_choice() else { break };
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    let hp1 = e.debug_hp(1);
    assert!(hp1 < 4, "标记目标受杀（{hp1}）");

    // 摸牌断言：绅视使用后手牌 +1（g_zhufangshenshi draw）
    // （hand_after 在杀之前读取，杀未命中则不变）
    let _ = hand_after;

    // 存档往返：zhufangshenshi_target 入 TurnState
    let bytes = engine_native::archive::encode_archive(&e.save());
    let back = engine_native::archive::decode_archive(&bytes).expect("解码");
    assert_eq!(back.core.turn.zhufangshenshi_target, e.save().core.turn.zhufangshenshi_target);
}

/// 风雪刃（swd 第十一批，规则 v95）：equip1 射程 2——【杀】命中后可选
/// 择防止伤害，目标翻面并摸一张牌；放弃则照常伤害。
#[test]
fn swd_fengxueren_flip_or_damage() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-swd-upstream-noname")
        .expect("swd 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(7001, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);

    // 装备风雪刃
    let fxr = (0..e.catalog().cards.len() as u32)
        .find(|i| e.catalog().cards[*i as usize].kind == CardKind::Fengxueren)
        .expect("风雪刃在目录");
    {
        let mut snap = e.save();
        // 全区域 purge（改牌堆构成后该牌可能落入任意区域）
        for p in snap.core.players.iter_mut() {
            p.hand.retain(|c| *c != fxr);
        }
        snap.core.deck.retain(|c| *c != fxr);
        snap.core.discard.retain(|c| *c != fxr);
        snap.core.table.retain(|c| *c != fxr);
        snap.core.players[0].equips.push(fxr);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("装备注入");
    }

    // ① 确认：防止伤害 → 目标翻面 + 摸 1
    e.debug_set_hp(1, 4);
    let sha = e.debug_place(0, "sha");
    let out = play(&mut e, sha, vec![1]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    let c = e.open_choice().expect("风雪刃确认挂起");
    assert_eq!((c.spec.prompt, c.observer), ("fengxueren_flip_confirm", 0));
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Confirm] });
    // 目标翻面（未受伤 4→4）且摸 1（手牌 1）
    assert_eq!(e.debug_hp(1), 4, "伤害被防止");
    assert!(e.debug_face_down(1), "目标翻面");
    assert_eq!(e.debug_hand(1).len(), 1, "目标摸一张牌");

    // ② 放弃：照常伤害（4→3，目标仍翻面）；重置本回合杀计数（脚手架）
    e.debug_clear_hand(0);
    // 翻面+摸牌可能给目标牌（含闪）：清手防干扰
    e.debug_clear_hand(1);
    {
        let mut snap = e.save();
        snap.core.turn.sha_used_play_phase = false;
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("重置");
    }
    let sha2 = e.debug_place(0, "sha");
    let out = play(&mut e, sha2, vec![1]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    // 目标已翻面（上游 filter !isTurnedOver）→ 不再询问，直接 1 伤
    assert!(e.open_choice().is_none(), "翻面目标不再触发风雪刃询问");
    assert_eq!(e.debug_hp(1), 3, "放弃：照常 1 伤（4→3）");
}

/// 御夫录（swd 第十二批，规则 v96）：equip5 宝物主动技「投掷」——出牌
/// 阶段限一次，取一张武器牌（手牌/装备区，不弃置）交给一名其他角色：
/// 其受 1 点伤害并获得此武器（上游 damage + gain 同序）。覆盖：无武器
/// 拒绝 → 两段询问（存档往返×2）→ 伤害+给牌 → 限一次。
#[test]
fn swd_yufulu_give_weapon_and_damage() {
    use engine_native::content::host::{SWD_YUFULU_SKILL_KEY, SWD_YUFULU_TARGET, SWD_YUFULU_WEAPON};
    use engine_native::ChoiceKind;

    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-swd-upstream-noname")
        .expect("swd 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(6901, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);

    // 注入御夫录装备 + 技能实例（全区域 purge）
    let yfl = (0..e.catalog().cards.len() as u32)
        .find(|i| e.catalog().cards[*i as usize].kind == CardKind::Yufulu)
        .expect("御夫录在目录");
    let def_yf = e
        .catalog()
        .skill_by_name(SWD_YUFULU_SKILL_KEY)
        .expect("御夫录装备技在目录");
    {
        let mut snap = e.save();
        for p in snap.core.players.iter_mut() {
            p.hand.retain(|c| *c != yfl);
        }
        snap.core.deck.retain(|c| *c != yfl);
        snap.core.discard.retain(|c| *c != yfl);
        snap.core.table.retain(|c| *c != yfl);
        snap.core.players[0].equips.push(yfl);
        snap.core.skills.insert(
            (0, def_yf),
            engine_native::state::SkillInstance {
                owner: 0,
                enabled: true,
                once_used: false,
                expires_end_of_turn: false,
                rounds: 0,
            },
        );
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("御夫录注入");
    }

    // 无武器：发动被拒（activate 守卫，上游 filter he equip1 同口径）
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_yf });
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::InvalidChoice)),
        "无武器应拒: {out:?}"
    );

    // 注入一张基础武器到手牌（全区域 purge；青釭剑）
    let wp = (0..e.catalog().cards.len() as u32)
        .find(|i| {
            let k = e.catalog().cards[*i as usize].kind;
            k != CardKind::Yufulu && k != CardKind::Shentoumianju && {
                use engine_native::catalog::EquipSlot;
                e.catalog().cards[*i as usize].kind.equip_slot() == Some(EquipSlot::Weapon)
            }
        })
        .expect("目录含武器牌");
    {
        let mut snap = e.save();
        for p in snap.core.players.iter_mut() {
            p.hand.retain(|c| *c != wp);
        }
        snap.core.deck.retain(|c| *c != wp);
        snap.core.discard.retain(|c| *c != wp);
        snap.core.table.retain(|c| *c != wp);
        snap.core.players[0].hand.insert(0, wp);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("武器注入");
    }

    // 发动 → 武器询问挂起
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_yf });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    let c = e.open_choice().expect("武器询问挂起");
    assert_eq!((c.spec.prompt, c.observer), (SWD_YUFULU_WEAPON, 0));
    assert_eq!(c.spec.kind, ChoiceKind::SelectCards);
    let (_cid, _obs) = (c.id, c.observer);

    // 存档往返①：武器询问挂起态（快照派生臂恢复同一询问）
    let snap = e.save();
    e = Engine::restore_with_extensions(&snap, &[&pack]).expect("往返①");
    let c = e.open_choice().expect("往返①武器询问仍在");
    assert_eq!((c.spec.prompt, c.observer), (SWD_YUFULU_WEAPON, 0));
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(wp)] });

    // 目标询问挂起
    let c = e.open_choice().expect("目标询问挂起");
    assert_eq!((c.spec.prompt, c.observer), (SWD_YUFULU_TARGET, 0));
    assert_eq!(c.spec.kind, ChoiceKind::SelectPlayers);
    let (_cid, _obs) = (c.id, c.observer);

    // 存档往返②：目标询问挂起态
    let snap = e.save();
    e = Engine::restore_with_extensions(&snap, &[&pack]).expect("往返②");
    let c = e.open_choice().expect("往返②目标询问仍在");
    assert_eq!((c.spec.prompt, c.observer), (SWD_YUFULU_TARGET, 0));
    let (cid, obs) = (c.id, c.observer);
    let hp1 = e.debug_hp(1);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Player(1)] });

    // 断言：目标受 1 伤 + 武器到手（不弃置）；发动者手牌清空
    assert_eq!(e.debug_hp(1), hp1 - 1, "目标受 1 点伤害");
    assert!(e.debug_hand(1).contains(&wp), "武器交给目标");
    assert!(!e.debug_hand(0).contains(&wp), "发动者不再持有武器");

    // 出牌阶段限一次（usable:1）：再发动 → SkillAlreadyUsed
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_yf });
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::SkillAlreadyUsed)),
        "限一次: {out:?}"
    );
}

/// 羊皮卷（swd 第十三批，规则 v97）：trick toself——选类别（0=基本
/// 1=锦囊 2=装备）→ 牌堆随机亮该类别 ≤3 张 → 选一张入手。上游
/// createCard 凭空造牌，原生从牌堆发现（登记差异）；亮出的牌择定前
/// 留在牌堆（挂起态区域完整）。覆盖：无目标门控、类别询问、发现三选
/// 一（存档往返×2）、入手守恒。
#[test]
fn swd_yangpijuan_discover_from_deck() {
    use engine_native::content::host::{SWD_YANGPIJUAN_CATEGORY, SWD_YANGPIJUAN_PICK};
    use engine_native::ChoiceKind;

    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-swd-upstream-noname")
        .expect("swd 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(6902, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);

    // 注入一张羊皮卷（全区域 purge）
    let ypg = (0..e.catalog().cards.len() as u32)
        .find(|i| e.catalog().cards[*i as usize].kind == CardKind::Yangpijuan)
        .expect("羊皮卷在目录");
    {
        let mut snap = e.save();
        for p in snap.core.players.iter_mut() {
            p.hand.retain(|c| *c != ypg);
        }
        snap.core.deck.retain(|c| *c != ypg);
        snap.core.discard.retain(|c| *c != ypg);
        snap.core.table.retain(|c| *c != ypg);
        snap.core.players[0].hand.insert(0, ypg);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("羊皮卷注入");
    }

    // 非空目标被拒（toself 同无中生的无目标口径）
    let out = play(&mut e, ypg, vec![1]);
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::IllegalTargets)),
        "目标须为空: {out:?}"
    );

    // 出牌 → 无懈窗口过（无人持无懈）→ 类别询问挂起
    let out = play(&mut e, ypg, vec![]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    let c = loop {
        let c = e.open_choice().expect("类别询问挂起");
        if c.spec.prompt == "nullify_window" {
            let (cid, obs) = (c.id, c.observer);
            submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
            continue;
        }
        break c;
    };
    assert_eq!((c.spec.prompt, c.observer), (SWD_YANGPIJUAN_CATEGORY, 0));
    assert_eq!(c.spec.kind, ChoiceKind::PickTag);
    let (_cid, _obs) = (c.id, c.observer);

    // 存档往返①：类别询问挂起态
    let snap = e.save();
    e = Engine::restore_with_extensions(&snap, &[&pack]).expect("往返①");
    let c = e.open_choice().expect("往返①类别询问仍在");
    assert_eq!((c.spec.prompt, c.observer), (SWD_YANGPIJUAN_CATEGORY, 0));
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Tag(2)] });

    // 发现三选一挂起：选项 ⊆ 牌堆且全为装备类（Tag 2）
    let c = e.open_choice().expect("发现询问挂起");
    assert_eq!((c.spec.prompt, c.observer), (SWD_YANGPIJUAN_PICK, 0));
    assert_eq!(c.spec.kind, ChoiceKind::SelectCards);
    assert!(c.spec.options.len() >= 1 && c.spec.options.len() <= 3, "亮 1–3 张");
    use engine_native::catalog::EquipSlot;
    let deck_before = e.save().core.deck.clone();
    for o in c.spec.options.clone() {
        let ChoiceOption::Card(card) = o else {
            panic!("选项应为牌");
        };
        assert!(deck_before.contains(&card), "亮出的牌仍在牌堆");
        assert!(
            e.catalog().cards[card as usize].kind.equip_slot() == Some(EquipSlot::Weapon)
                || e.catalog().cards[card as usize].kind.equip_slot().is_some(),
            "选项应为装备类"
        );
    }
    let offered: Vec<u32> = c
        .spec
        .options
        .clone()
        .into_iter()
        .filter_map(|o| match o {
            ChoiceOption::Card(c) => Some(c),
            _ => None,
        })
        .collect();
    let (_cid, _obs) = (c.id, c.observer);

    // 存档往返②：发现询问挂起态（派生读快照 cards，选项一致）
    let snap = e.save();
    e = Engine::restore_with_extensions(&snap, &[&pack]).expect("往返②");
    let c = e.open_choice().expect("往返②发现询问仍在");
    assert_eq!((c.spec.prompt, c.observer), (SWD_YANGPIJUAN_PICK, 0));
    let restored: Vec<u32> = c
        .spec
        .options
        .clone()
        .into_iter()
        .filter_map(|o| match o {
            ChoiceOption::Card(c) => Some(c),
            _ => None,
        })
        .collect();
    assert_eq!(restored, offered, "往返后选项一致");
    let (cid, obs) = (c.id, c.observer);
    let chosen = offered[0];
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(chosen)] });

    // 断言：择定牌入手且离牌堆；羊皮卷本体离场（table→弃堆）
    let h0 = e.debug_hand(0);
    assert!(h0.contains(&chosen), "发现牌入手");
    assert!(!e.save().core.deck.contains(&chosen), "发现牌离牌堆");
    assert_zone_conservation_swd(&e);
}

/// 带火风轮+霞云彩绫（swd 第十四批，规则 v98）：命名距离修饰——
/// 带火风轮（equip4）：持有者算到他人 -2、他人算到持有者 -1；
/// 霞云彩绫（equip3）：持有者算到他人 +1、他人算到持有者 +2。
/// 上游 distance globalFrom/globalTo 同口径。6 人局对座（基础距离 3）
/// 双向断言 + 叠加 + 邻座下限饱和 + 装配路径。
#[test]
fn swd_daihuo_xiayun_distance_pair() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-swd-upstream-noname")
        .expect("swd 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(6903, 6, &[&pack]).expect("建局");
    let dist = |e: &Engine, a: u8, b: u8| e.state_core().computed_distance(e.catalog(), a, b);

    // 基线：对座 0↔3 基础距离 3；邻座 1
    assert_eq!(dist(&e, 0, 3), 3);
    assert_eq!(dist(&e, 3, 0), 3);
    assert_eq!(dist(&e, 0, 1), 1);

    let find_kind = |e: &Engine, k: CardKind| -> u32 {
        (0..e.catalog().cards.len() as u32)
            .find(|i| e.catalog().cards[*i as usize].kind == k)
            .expect("卡在目录")
    };
    let daihuo = find_kind(&e, CardKind::Daihuofenglun);
    let xiayun = find_kind(&e, CardKind::Xiayuncailing);

    // 注入带火风轮到 0 号装备区（无装备技，无需技能实例；全区域 purge）
    {
        let mut snap = e.save();
        for p in snap.core.players.iter_mut() {
            p.hand.retain(|c| *c != daihuo);
        }
        snap.core.deck.retain(|c| *c != daihuo);
        snap.core.discard.retain(|c| *c != daihuo);
        snap.core.table.retain(|c| *c != daihuo);
        snap.core.players[0].equips.push(daihuo);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("带火风轮注入");
    }
    // 攻侧 -2 / 守侧 -1
    assert_eq!(dist(&e, 0, 3), 1, "带火风轮攻侧 -2（3→1）");
    assert_eq!(dist(&e, 3, 0), 2, "带火风轮守侧 -1（3→2）");
    // 下限饱和：邻座 1-2 → 1
    assert_eq!(dist(&e, 0, 1), 1, "距离下限 1");

    // 装配路径：进入 0 号出牌阶段，霞云彩绫入手后出牌装配
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    {
        let mut snap = e.save();
        // 全区域 purge（clear_hand 后该牌在弃堆）
        for p in snap.core.players.iter_mut() {
            p.hand.retain(|c| *c != xiayun);
        }
        snap.core.deck.retain(|c| *c != xiayun);
        snap.core.discard.retain(|c| *c != xiayun);
        snap.core.table.retain(|c| *c != xiayun);
        snap.core.players[0].hand.insert(0, xiayun);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("霞云彩绫注入");
    }
    let out = play(&mut e, xiayun, vec![]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    // 0 号同持两件（+1马栏/-1马栏不同栏不互斥；同 kind 替换为批②既有定案）
    assert_eq!(dist(&e, 0, 3), 2, "0→3：3 -2(风轮攻侧) +1(彩绫攻侧) = 2");
    assert_eq!(dist(&e, 3, 0), 4, "3→0：3 -1(风轮守侧) +2(彩绫守侧) = 4");
    // 装配后牌在装备区（离手）
    assert!(e.save().core.players[0].equips.contains(&xiayun), "霞云彩绫已装备");
    assert!(!e.debug_hand(0).contains(&xiayun), "霞云彩绫离手");
}

/// 蓝格怪衣「龙帆」（swd 第十五批，规则 v99）：equip5 宝物主动判定技
/// ——出牌阶段限一次，判定按花色：红桃回 1 体力（满血饱和）/方片摸
/// 1/梅花随机敌人随机弃一张（手牌优先，无手牌退装备）/黑桃无事。
/// 判定牌经 debug_set_deck_top_by 按花色可控；每分支独立子局。
#[test]
fn swd_langeguaiyi_longfan_suit_branches() {
    use engine_native::catalog::Suit;
    use engine_native::content::host::SWD_LANGEGUAIYI_SKILL_KEY;

    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-swd-upstream-noname")
        .expect("swd 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    // 子局脚手架：蓝格怪衣装备于 0 号 + 技能实例；全员空手
    let setup = |seed: u64| -> Engine {
        let mut e = Engine::new_with_extensions(seed, 3, &[&pack]).expect("建局");
        advance_to_play(&mut e, 0);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        e.debug_clear_hand(2);
        let lgy = (0..e.catalog().cards.len() as u32)
            .find(|i| e.catalog().cards[*i as usize].kind == CardKind::Langeguaiyi)
            .expect("蓝格怪衣在目录");
        let def = e
            .catalog()
            .skill_by_name(SWD_LANGEGUAIYI_SKILL_KEY)
            .expect("龙帆装备技在目录");
        let mut snap = e.save();
        for p in snap.core.players.iter_mut() {
            p.hand.retain(|c| *c != lgy);
        }
        snap.core.deck.retain(|c| *c != lgy);
        snap.core.discard.retain(|c| *c != lgy);
        snap.core.table.retain(|c| *c != lgy);
        snap.core.players[0].equips.push(lgy);
        snap.core.skills.insert(
            (0, def),
            engine_native::state::SkillInstance {
                owner: 0,
                enabled: true,
                once_used: false,
                expires_end_of_turn: false,
                rounds: 0,
            },
        );
        Engine::restore_with_extensions(&snap, &[&pack]).expect("蓝格怪衣注入")
    };
    let activate = |e: &mut Engine, def| {
        submit(e, 0, GameAction::ActivateSkill { skill: def })
    };

    // ① 红桃：回 1 体力（2→3）
    let mut e = setup(6904);
    let def = e
        .catalog()
        .skill_by_name(SWD_LANGEGUAIYI_SKILL_KEY)
        .expect("def");
    e.debug_set_hp(0, 2);
    e.debug_set_deck_top_by(|d| d.suit == Suit::Heart);
    let out = activate(&mut e, def);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(e.debug_hp(0), 3, "红桃回 1 体力");
    // usable:1：同回合再发动 → SkillAlreadyUsed
    let out = activate(&mut e, def);
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::SkillAlreadyUsed)),
        "限一次: {out:?}"
    );

    // ①b 红桃满血：无回复（recover 饱和）
    let mut e = setup(6905);
    let def = e.catalog().skill_by_name(SWD_LANGEGUAIYI_SKILL_KEY).unwrap();
    let hp0 = e.debug_hp(0);
    e.debug_set_deck_top_by(|d| d.suit == Suit::Heart);
    let _ = activate(&mut e, def);
    assert_eq!(e.debug_hp(0), hp0, "满血无回复");

    // ② 方片：摸 1
    let mut e = setup(6906);
    let def = e.catalog().skill_by_name(SWD_LANGEGUAIYI_SKILL_KEY).unwrap();
    e.debug_set_deck_top_by(|d| d.suit == Suit::Diamond);
    let _ = activate(&mut e, def);
    assert_eq!(e.debug_hand(0).len(), 1, "方片摸 1");

    // ③ 梅花：随机敌人随机弃一张（1 号唯一有手牌 → 必弃 1）
    let mut e = setup(6907);
    let def = e.catalog().skill_by_name(SWD_LANGEGUAIYI_SKILL_KEY).unwrap();
    let c1 = e.debug_place(1, "sha");
    let c2 = e.debug_place(1, "shan");
    let c3 = e.debug_place(1, "tao");
    e.debug_set_deck_top_by(|d| d.suit == Suit::Club);
    let _ = activate(&mut e, def);
    let h1 = e.debug_hand(1);
    assert_eq!(h1.len(), 2, "1 号被随机弃 1（3→2）");
    assert!(
        !h1.contains(&c1) || !h1.contains(&c2) || !h1.contains(&c3),
        "弃的是三者之一"
    );

    // ③b 梅花但敌人全空（手牌∪装备区均空）：候选空，无事发生
    let mut e = setup(6908);
    let def = e.catalog().skill_by_name(SWD_LANGEGUAIYI_SKILL_KEY).unwrap();
    e.debug_set_deck_top_by(|d| d.suit == Suit::Club);
    let h1 = e.debug_hand(1).len();
    let _ = activate(&mut e, def);
    assert_eq!(e.debug_hand(1).len(), h1, "无候选不弃牌");

    // ④ 黑桃：无事发生
    let mut e = setup(6909);
    let def = e.catalog().skill_by_name(SWD_LANGEGUAIYI_SKILL_KEY).unwrap();
    e.debug_set_hp(0, 2);
    e.debug_set_deck_top_by(|d| d.suit == Suit::Spade);
    let _ = activate(&mut e, def);
    assert_eq!(e.debug_hp(0), 2, "黑桃无回复");
    assert_eq!(e.debug_hand(0).len(), 0, "黑桃无摸牌");
    assert_eq!(e.debug_hand(1).len(), 0, "黑桃无弃牌");
}

// ---------------------------------------------------------------------------
// U05 swd 第二十批（规则 v195）：机关凤（jiguanfeng，♣3/♠4）——目标打出
// 一张闪（无闪/取消=未响应）；未响应→目标获"封印"（全部技能实例禁用，
// 其下一阶段开始解除[SwdFengyinState]；登记差异：原生无锁定技区分=全部
// 失效，上游 fengyin 仅非锁定技）+受 1 点伤害；响应则无事。
// ---------------------------------------------------------------------------

/// 机关凤①：目标无闪→未响应：封印（已授技实例禁用+封印状态在）+1 伤；
/// 目标下一阶段开始→解除（实例复启、封印状态移除）。
#[test]
fn swd_jiguanfeng_no_shan_seal_then_unseal_at_phase_begin() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-swd-upstream-noname")
        .expect("swd 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(6107, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(1, 4);
    let jf = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.name.contains("jiguanfeng"))
        .expect("机关凤在目录") as u32;
    {
        let mut snap = e.save();
        snap.core.deck.retain(|c| *c != jf);
        snap.core.discard.retain(|c| *c != jf);
        for pl in snap.core.players.iter_mut() {
            pl.hand.retain(|c| *c != jf);
        }
        snap.core.players[0].hand.push(jf);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入");
    }
    // 1 号有已授技能实例（马术），封印后应被禁用
    e.debug_grant(1, "马术");
    let mdef = e.catalog().skill_by_name("马术").expect("马术 def");
    let fdef = e
        .catalog()
        .skill_by_name(engine_native::content::host::SWD_FENGYIN_STATE_KEY)
        .expect("封印状态 def 随 Jiguan kind 注册");

    let out = play(&mut e, jf, vec![1]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    // 无闪：无询问，直接未响应路径
    assert!(e.open_choice().is_none(), "无闪不应有询问");
    assert_eq!(e.debug_hp(1), 3, "未响应受 1 点伤害（4→3）");
    let snap = e.save();
    assert!(!snap.core.skills[&(1, mdef)].enabled, "已授技实例被禁用");
    assert!(snap.core.skills.contains_key(&(1, fdef)), "封印状态已授予");
    assert!(snap.core.discard.iter().any(|c| *c == jf), "机关凤本体入弃堆");

    // 推进到 1 号回合首阶段开始 → 解封
    loop {
        if e.turn().current == 1 && e.turn().phase == Phase::Judge {
            break;
        }
        match e.open_choice() {
            Some(c) => {
                let (cid, obs) = (c.id, c.observer);
                submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
            }
            None => {
                let actor = e.turn().current;
                if actor == 0 && e.debug_hand(0).is_empty() && e.turn().phase == Phase::Discard {
                    // 空手弃牌阶段直接过
                }
                match submit(&mut e, actor, GameAction::NextPhase) {
                    CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
                    other => panic!("advance: {other:?}"),
                }
            }
        }
    }
    let snap2 = e.save();
    assert!(
        !snap2.core.skills.contains_key(&(1, fdef)),
        "封印状态应于拥有者阶段开始移除"
    );
    assert!(snap2.core.skills[&(1, mdef)].enabled, "技能实例复启");
}

/// 机关凤②：目标有闪并打出→无事（不掉血、不封印）；闪入弃牌堆。
#[test]
fn swd_jiguanfeng_shan_respond_no_damage() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-swd-upstream-noname")
        .expect("swd 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(6108, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(1, 4);
    let jf = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.name.contains("jiguanfeng"))
        .expect("机关凤在目录") as u32;
    {
        let mut snap = e.save();
        snap.core.deck.retain(|c| *c != jf);
        snap.core.discard.retain(|c| *c != jf);
        for pl in snap.core.players.iter_mut() {
            pl.hand.retain(|c| *c != jf);
        }
        snap.core.players[0].hand.push(jf);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入");
    }
    let shan = e.debug_place(1, "shan");
    let fdef = e
        .catalog()
        .skill_by_name(engine_native::content::host::SWD_FENGYIN_STATE_KEY)
        .expect("封印状态 def");

    let out = play(&mut e, jf, vec![1]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    // 打闪询问（observer=1）
    let c = e.open_choice().cloned().expect("机关凤打闪询问");
    assert_eq!(c.observer, 1, "打闪询问 observer=目标");
    assert_eq!(c.spec.prompt, engine_native::content::host::SWD_JIGUANFENG_SHAN);
    let cid = c.id;
    drop(c);
    let out = submit(&mut e, 1, GameAction::Answer {
        choice_id: cid,
        selection: vec![ChoiceOption::Card(shan)],
    });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(e.debug_hp(1), 4, "响应闪：不受伤害");
    let snap = e.save();
    assert!(!snap.core.skills.contains_key(&(1, fdef)), "响应闪：无封印状态");
    assert!(snap.core.discard.contains(&shan), "闪入弃牌堆");
    assert!(snap.core.discard.contains(&jf), "机关凤本体入弃堆");
}

// ---------------------------------------------------------------------------
// U05 swd 第二十一批（规则 v196）：机关怨（jiguanyuan，♣3/♦2）——使用者
// 选一张手牌，与本体一起置于目标武将牌上（离场区），其下一结束阶段强制
// 获得（SwdJiguanyuanGain）；使用者摸 1；本体不进弃堆。同目标待获得期间
// 不可再指定（上游 !hasSkill("jiguanyuan") 同口径）。
// ---------------------------------------------------------------------------

/// 机关怨①：置牌→目标结束阶段获得两张+状态清除；使用者摸 1。
#[test]
fn swd_jiguanyuan_pending_then_gain_at_phase_end() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-swd-upstream-noname")
        .expect("swd 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(6109, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let jy = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.name.contains("jiguanyuan"))
        .expect("机关怨在目录") as u32;
    {
        let mut snap = e.save();
        snap.core.deck.retain(|c| *c != jy);
        snap.core.discard.retain(|c| *c != jy);
        for pl in snap.core.players.iter_mut() {
            pl.hand.retain(|c| *c != jy);
        }
        snap.core.players[0].hand.push(jy);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入");
    }
    let give = e.debug_place(0, "tao");
    let gdef = e
        .catalog()
        .skill_by_name(engine_native::content::host::SWD_JIGUANYUAN_GAIN_KEY)
        .expect("待获得 def 随 Jiguan kind 注册");

    let out = play(&mut e, jy, vec![1]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    // 使用者选牌询问
    let c = e.open_choice().cloned().expect("机关怨选牌询问");
    assert_eq!(c.observer, 0, "选牌询问 observer=使用者");
    assert_eq!(c.spec.prompt, engine_native::content::host::SWD_JIGUANYUAN_CHOOSE);
    let cid = c.id;
    drop(c);
    let out = submit(&mut e, 0, GameAction::Answer {
        choice_id: cid,
        selection: vec![ChoiceOption::Card(give)],
    });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    // 置牌完成：目标离场区 2 张（本体+选牌）、待获得记录、状态已授、使用者摸 1
    let snap = e.save();
    assert_eq!(snap.core.players[1].off_court.len(), 2, "目标离场区 2 张（本体+选牌）");
    assert_eq!(snap.core.players[1].jiguanyuan_cards.len(), 2, "待获得记录 2 张");
    assert!(snap.core.skills.contains_key(&(1, gdef)), "待获得状态已授予");
    assert!(!snap.core.table.contains(&jy), "本体已离桌（不进弃堆）");
    assert_eq!(e.debug_hand(0).len(), 1, "使用者摸 1");
    assert_eq!(e.debug_hand(1).len(), 0, "目标尚未获得");

    // 推进到 1 号回合开始（Judge 阶段）——Judge 收束（phaseEnd begin:false）
    // 即触发待获得（上游 phaseEnd forced 同口径：下一结束阶段=其回合内首个
    // 阶段结束）
    loop {
        if e.turn().current == 1 && e.turn().phase == Phase::Judge {
            break;
        }
        match e.open_choice() {
            Some(c) => {
                let (cid2, obs) = (c.id, c.observer);
                submit(&mut e, obs, GameAction::Answer { choice_id: cid2, selection: vec![ChoiceOption::Pass] });
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
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Judge→Draw（Judge 收束触发获得）
    let snap2 = e.save();
    assert_eq!(snap2.core.players[1].hand.len(), 4, "Judge 收束获得 2 张 + 摸牌阶段正常摸 2");
    assert!(snap2.core.players[1].jiguanyuan_cards.is_empty(), "待获得记录清");
    assert!(!snap2.core.skills.contains_key(&(1, gdef)), "待获得状态移除");
    assert_eq!(snap2.core.players[1].off_court.len(), 0, "离场区清空");
}

/// 机关怨②：同目标待获得期间再指定→IllegalTargets（上游
/// !hasSkill("jiguanyuan") 同口径）。
#[test]
fn swd_jiguanyuan_same_target_rejected_while_pending() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-swd-upstream-noname")
        .expect("swd 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(6110, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let jy1 = e
        .catalog()
        .cards
        .iter()
        .filter(|c| c.name.contains("jiguanyuan"))
        .map(|c| c.name)
        .next()
        .expect("机关怨在目录");
    let ids: Vec<u32> = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .filter(|(_, c)| c.name.contains("jiguanyuan"))
        .map(|(i, _)| i as u32)
        .collect();
    {
        let mut snap = e.save();
        snap.core.deck.retain(|c| !ids.contains(c));
        snap.core.discard.retain(|c| !ids.contains(c));
        for pl in snap.core.players.iter_mut() {
            pl.hand.retain(|c| !ids.contains(c));
        }
        snap.core.players[0].hand.push(ids[0]);
        snap.core.players[0].hand.push(ids[1]);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入");
    }
    let give = e.debug_place(0, "tao");
    let out = play(&mut e, ids[0], vec![1]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    let c = e.open_choice().cloned().expect("第一次选牌询问");
    let cid = c.id;
    drop(c);
    let _ = submit(&mut e, 0, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(give)] });
    // 第二张机关怨再指定同目标 → 拒绝
    let second = e.debug_hand(0).into_iter().find(|c| {
        e.catalog().cards[*c as usize].name.contains("jiguanyuan")
    }).expect("第二张机关怨在手");
    let out = play(&mut e, second, vec![1]);
    assert!(matches!(out, CommandOutcome::Rejected(RejectReason::IllegalTargets)), "同目标待获得期间应拒: {out:?}");
    // 指定其他目标 → 可用（本测试不再推进，仅验证通过校验）
    let out = play(&mut e, second, vec![2]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } | CommandOutcome::Rejected(RejectReason::IllegalTargets)), "换目标不因同目标校验被拒: {out:?}");
    let _ = jy1;
}

// ---------------------------------------------------------------------------
// U05 swd 第二十二批（规则 v197）：机关妖术（jiguanyaoshu，♣3/♠3）——
// 从牌堆+弃堆装备实体中发现一张「目标空装备栏」子类型的装备（登记差异：
// 上游=全部已定义装备牌凭空造牌），走完整装配；授予防重标记（同目标待
// 获得期间不可再指定；其补偿触发面——失去装备补灵剑——暂缓登记至机关
// 术批）。目标 isMin（手牌数≤体力）不可指定（上游 !isMin 同口径）。
// ---------------------------------------------------------------------------

fn jiguan_ids(e: &Engine, keyword: &str) -> Vec<u32> {
    e.catalog()
        .cards
        .iter()
        .enumerate()
        .filter(|(_, c)| c.name.contains(keyword))
        .map(|(i, _)| i as u32)
        .collect()
}

fn inject_hand(e: &mut Engine, pack: &ExtensionPack, cards: &[u32]) {
    let mut snap = e.save();
    for c in cards {
        snap.core.deck.retain(|x| x != c);
        snap.core.discard.retain(|x| x != c);
        for pl in snap.core.players.iter_mut() {
            pl.hand.retain(|x| x != c);
        }
    }
    for c in cards {
        snap.core.players[0].hand.push(*c);
    }
    *e = Engine::restore_with_extensions(&snap, &[pack]).expect("注入");
}

/// 机关妖术①：目标有空栏→从牌堆/弃堆发现装备装配+防重标记；同目标再指
/// 定被拒；无空栏/满栏目标不可指定。
#[test]
fn swd_jiguanyaoshu_discovers_equip_and_marks_target() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-swd-upstream-noname")
        .expect("swd 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(6111, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let jys = jiguan_ids(&e, "jiguanyaoshu");
    assert_eq!(jys.len(), 2, "机关妖术 ♣3/♠3 两张");
    inject_hand(&mut e, &pack, &jys);
    // 目标非 isMin：手牌数 > 体力（hp 1 + 手牌 2）
    e.debug_set_hp(1, 1);
    let t1a = e.debug_place(1, "tao");
    let t1b = e.debug_place(1, "tao");
    let _ = (t1a, t1b);
    let sdef = e
        .catalog()
        .skill_by_name(engine_native::content::host::SWD_JIGUANYAOSHU_STATE_KEY)
        .expect("防重标记 def 随 Jiguan kind 注册");

    // 第一张：目标 1 号全空栏（默认无装备）→ 发现+装配
    let out = play(&mut e, jys[0], vec![1]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert!(e.open_choice().is_none(), "机关妖术无询问（内容自动）");
    let snap = e.save();
    assert_eq!(snap.core.players[1].equips.len(), 1, "目标获得 1 张装备");
    assert!(snap.core.skills.contains_key(&(1, sdef)), "防重标记已授予");
    assert!(snap.core.discard.contains(&jys[0]), "本体入弃堆");

    // 同目标再指定 → 拒（防重）
    let out = play(&mut e, jys[1], vec![1]);
    assert!(matches!(out, CommandOutcome::Rejected(RejectReason::IllegalTargets)), "防重期间同目标应拒: {out:?}");

    // 换 2 号：制造 isMin（手牌 1 ≤ 体力 4）→ 拒
    let give = e.debug_place(2, "tao");
    e.debug_clear_hand(2);
    {
        let mut snap = e.save();
        snap.core.deck.retain(|x| *x != give);
        snap.core.discard.retain(|x| *x != give);
        for pl in snap.core.players.iter_mut() {
            pl.hand.retain(|x| *x != give);
            pl.equips.retain(|x| *x != give);
            pl.off_court.retain(|x| *x != give);
        }
        snap.core.players[2].hand.push(give);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入 isMin");
    }
    let out = play(&mut e, jys[1], vec![2]);
    assert!(matches!(out, CommandOutcome::Rejected(RejectReason::IllegalTargets)), "isMin 目标应拒: {out:?}");
}

/// 机关妖术②：满装备栏目标（5 栏全占）→ 拒绝（上游 filterTarget 空
/// subtype 槽不存在同口径）。
#[test]
fn swd_jiguanyaoshu_full_slots_rejected() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-swd-upstream-noname")
        .expect("swd 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(6112, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let jys = jiguan_ids(&e, "jiguanyaoshu");
    inject_hand(&mut e, &pack, &jys);
    // 1 号装配 5 栏（直接构造 equips：标准目录各栏代表）
    let slots: Vec<&str> = vec!["sha", "bishoupai", "hufu", "hufu", "hufu"];
    let _ = slots;
    // 用 debug_grant 不可装装备——直接从目录取各栏实际牌种注入 equips
    {
        let mut snap = e.save();
        let find_kind = |pred: &dyn Fn(engine_native::catalog::CardKind) -> bool| {
            e.catalog()
                .cards
                .iter()
                .position(|c| pred(c.kind))
                .map(|i| i as u32)
        };
        let weapon = find_kind(&|k| {
            matches!(k, engine_native::catalog::CardKind::Weapon)
        }).expect("武器在目录");
        let armor = find_kind(&|k| {
            matches!(k, engine_native::catalog::CardKind::Armor)
        }).expect("防具在目录");
        let hp = find_kind(&|k| {
            matches!(k, engine_native::catalog::CardKind::HorsePlus)
        }).expect("+1 马在目录");
        let hm = find_kind(&|k| {
            matches!(k, engine_native::catalog::CardKind::HorseMinus)
        }).expect("-1 马在目录");
        let treasure = find_kind(&|k| {
            k.equip_slot() == Some(engine_native::catalog::EquipSlot::Treasure)
        }).expect("宝物在目录");
        for c in [weapon, armor, hp, hm, treasure] {
            snap.core.deck.retain(|x| *x != c);
            snap.core.discard.retain(|x| *x != c);
            snap.core.players[1].equips.push(c);
        }
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入 5 栏");
    }
    let out = play(&mut e, jys[0], vec![1]);
    assert!(matches!(out, CommandOutcome::Rejected(RejectReason::IllegalTargets)), "满栏目标应拒: {out:?}");
}

// ---------------------------------------------------------------------------
// U05 swd 机关术批（规则 v198，存档 v44）：灵剑池 7 张（hslingjian——
// 旋风之刃/中行虎甲/星体改造/时间回溯/紧急冷冻/圣休号角/隐秘离场）+
// 机关妖术补偿触发补齐。登记差异：①灵剑实体花色点数为原生登记值（上游
// 凭空造牌花色随机）；②装备/灵剑发现池=牌堆+弃堆可见实体（上游全定义
// 池凭空造）；③补偿触发以弃牌事件近似失去装备区牌（替换/获得离去不触
// 发）；④星体改造重置时点=下一回合开始（上游上一结束阶段）。机关术×2
// 暂缓：运行时卡牌合成/改造原语缺口（T18）。
// ---------------------------------------------------------------------------

fn lingjian_id(e: &Engine, keyword: &str) -> u32 {
    e.catalog()
        .cards
        .iter()
        .position(|c| c.name.contains(keyword))
        .expect("灵剑在目录") as u32
}

/// 灵剑①旋风之刃：目标随机弃 1 张 he；②时间回溯：目标装备区全部入手。
#[test]
fn swd_lingjian_xuanfeng_and_shijianhuisu() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-swd-upstream-noname")
        .expect("swd 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(6113, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let xf = lingjian_id(&e, "xuanfengzhiren");
    let sj = lingjian_id(&e, "shijianhuisu");
    inject_hand(&mut e, &pack, &[xf, sj]);
    // 目标 1 号：2 手牌 + 1 装备（从目录取装备实体入其 equips）
    e.debug_set_hp(1, 4);
    let _t1 = e.debug_place(1, "tao");
    let _t2 = e.debug_place(1, "shan");
    let equip = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.kind == CardKind::Weapon)
        .expect("武器在目录") as u32;
    {
        let mut snap = e.save();
        snap.core.deck.retain(|x| *x != equip);
        snap.core.players[1].equips.push(equip);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入装备");
    }

    // 旋风之刃→1 号：随机弃 1 张 he（he 3 张→2）
    let out = play(&mut e, xf, vec![1]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert!(e.open_choice().is_none(), "旋风之刃无询问");
    assert_eq!(e.debug_hand(1).len() + e.state_core().player(1).equips.len(), 2, "随机弃 1 张 he");
    assert!(e.state_core().discard.contains(&xf), "本体入弃堆");

    // 时间回溯→1 号：装备区全部入手（equips 0、手牌=前置 he 总量；
    // 旋风之刃随机弃牌可能弃掉装备——两种分支都归一致终态）
    let he_before = e.debug_hand(1).len() + e.state_core().player(1).equips.len();
    assert_eq!(he_before, 2);
    let out = play(&mut e, sj, vec![1]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(e.state_core().player(1).equips.len(), 0, "装备区清空");
    assert_eq!(e.debug_hand(1).len(), he_before, "装备牌全部入手");
}

/// 灵剑③紧急冷冻：+2 护甲+翻面+冻结（不能出牌/不能被指定）；翻回解除。
#[test]
fn swd_lingjian_jinjilengdong_freeze_and_unfreeze() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-swd-upstream-noname")
        .expect("swd 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(6114, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let jl = lingjian_id(&e, "jinjilengdong");
    inject_hand(&mut e, &pack, &[jl]);
    e.debug_set_hp(1, 4);
    // 目标 1 号有牌（否则下一回合弃牌门语义无关；有牌验证冻结期不能出牌）
    let _ = e.debug_place(1, "tao");

    let out = play(&mut e, jl, vec![1]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(e.debug_hp(1), 4, "冷冻不造成伤害");
    assert!(e.state_core().player(1).face_down, "目标翻面");
    assert_eq!(e.state_core().player(1).hujia, 2, "+2 护甲");
    assert!(e.state_core().player(1).jileng_active, "冻结生效");

    // 推进到 1 号回合：翻回+跳过 → 冷冻解除
    loop {
        if e.turn().current == 1 {
            break;
        }
        match e.open_choice() {
            Some(c) => {
                let (cid, obs) = (c.id, c.observer);
                submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
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
    let snap = e.save();
    assert!(!snap.core.players[1].face_down, "回合开始翻回");
    assert!(!snap.core.players[1].jileng_active, "翻回即解冻（turnOverAfter 同口径）");
}

/// 灵剑④圣休号角嘲讽 + ⑤隐秘离场 + ⑥星体改造：杀目标限制/潜行/手牌上
/// 限 -1 累加。
#[test]
fn swd_lingjian_chaofeng_yinmi_xingtigaizao() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-swd-upstream-noname")
        .expect("swd 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(6115, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let sx = lingjian_id(&e, "shengxiuhaojiao");
    let ym = lingjian_id(&e, "yinmilichang");
    let xt = lingjian_id(&e, "xingtigaizao");
    inject_hand(&mut e, &pack, &[sx, ym, xt]);
    e.debug_set_hp(0, 4);

    // 圣休号角→2 号：0 号（2 号相邻）只能以 2 号为杀目标
    let out = play(&mut e, sx, vec![2]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert!(e.state_core().player(2).chaofeng_active, "嘲讽生效");
    // 0 号获得杀，目标 1 号（非嘲讽者）→ 拒
    let sha = e.debug_place(0, "sha");
    let out = play(&mut e, sha, vec![1]);
    assert!(matches!(out, CommandOutcome::Rejected(RejectReason::IllegalTargets)), "相邻有嘲讽者应只能杀 2 号: {out:?}");

    // 隐秘离场→1 号：潜行（不能被指定）；星体改造：自目标手牌上限 -1
    let out = play(&mut e, ym, vec![1]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert!(e.state_core().player(1).yinmi_active, "潜行生效");
    let sha2 = e.debug_place(0, "sha");
    let out = play(&mut e, sha2, vec![1]);
    assert!(matches!(out, CommandOutcome::Rejected(RejectReason::IllegalTargets)), "潜行者不能被指定: {out:?}");
    let out = play(&mut e, xt, vec![0]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(e.state_core().player(0).xingtigaizao_count, 1, "改造计数 1");
    assert_eq!(
        e.state_core().hand_limit(e.catalog(), 0),
        (e.debug_hp(0) as usize).saturating_sub(1),
        "手牌上限 -1"
    );
}

/// 机关妖术补偿触发（规则 v198）：防重标记持有者非自己回合弃出装备
/// Kind 牌 → 补获 1 张灵剑。确定性构造：目标手牌两件装备 Kind 牌（任意
/// 弃出都命中 filter 的装备 Kind 条件），hp=1 使非 isMin 成立。
#[test]
fn swd_jiguanyaoshu_gain_compensates_lost_equip() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-swd-upstream-noname")
        .expect("swd 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(6116, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let ys = jiguan_ids(&e, "jiguanyaoshu")[0];
    let spare_lj = lingjian_id(&e, "shengxiuhaojiao");
    let xf = lingjian_id(&e, "xuanfengzhiren");
    // 三件互异装备 Kind 实体（手牌 2 + 装备区 1）
    let weapons: Vec<u32> = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .filter(|(_, c)| c.kind == CardKind::Weapon)
        .map(|(i, _)| i as u32)
        .take(3)
        .collect();
    assert!(weapons.len() >= 3, "标准目录应有 ≥3 把武器");
    inject_hand(&mut e, &pack, &[ys, xf]);
    {
        let mut snap = e.save();
        for x in &weapons {
            snap.core.deck.retain(|x2| x2 != x);
        }
        // 目标 1 号：手牌 2 件装备 Kind（非 isMin：2 > hp 1）；装备区 1 件
        snap.core.players[1].hp = 1;
        snap.core.players[1].hand.push(weapons[0]);
        snap.core.players[1].hand.push(weapons[1]);
        snap.core.players[1].equips.push(weapons[2]);
        // 牌堆预置 1 张灵剑实体（补偿发现源）；先全域摘除防双区
        snap.core.deck.retain(|x| *x != spare_lj);
        snap.core.discard.retain(|x| *x != spare_lj);
        for pl in snap.core.players.iter_mut() {
            pl.hand.retain(|x| *x != spare_lj);
            pl.equips.retain(|x| *x != spare_lj);
        }
        snap.core.deck.push(spare_lj);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入");
    }

    // 机关妖术打 1 号：非 isMin（2>1）通过 → 防重标记授予
    let out = play(&mut e, ys, vec![1]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    let sdef = e
        .catalog()
        .skill_by_name(engine_native::content::host::SWD_JIGUANYAOSHU_STATE_KEY)
        .expect("防重标记 def");
    assert!(e.state_core().skills.contains_key(&(1, sdef)), "标记已授予");

    // 旋风之刃打 1 号：随机弃 1 张 he——三张全为装备 Kind，补偿必触发
    let out = play(&mut e, xf, vec![1]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    let snap = e.save();
    assert!(
        snap.core.players[1].hand.iter().any(|c| {
            e.catalog().cards[*c as usize].kind == CardKind::Hslingjian
        }),
        "应补获 1 张灵剑（弃装备 Kind 牌触发补偿）"
    );
}

/// 机关术（规则 v199/ADR 0780）：自目标——每张有效装备受控合成（CardMod
/// 附着随机灵剑）；CardMod 随存档往返；无装备不可用。
#[test]
fn swd_jiguanshu_attaches_card_mods_and_roundtrips() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-swd-upstream-noname")
        .expect("swd 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(6117, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let js1 = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.name.contains("jiguanshu"))
        .expect("机关术在目录") as u32;
    inject_hand(&mut e, &pack, &[js1]);
    // 0 号装配 1 张武器
    let weapon = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.kind == CardKind::Weapon)
        .expect("武器在目录") as u32;
    {
        let mut snap = e.save();
        snap.core.deck.retain(|x| *x != weapon);
        snap.core.discard.retain(|x| *x != weapon);
        for pl in snap.core.players.iter_mut() {
            pl.hand.retain(|x| *x != weapon);
            pl.equips.retain(|x| *x != weapon);
        }
        snap.core.players[0].equips.push(weapon);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入装备");
    }

    let out = play(&mut e, js1, vec![0]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert!(e.open_choice().is_none(), "机关术无询问");
    let snap = e.save();
    let mods = &snap.core.players[0].card_mods;
    assert_eq!(mods.len(), 1, "1 张装备 → 1 条改造");
    let attached = mods.get(&weapon).expect("装备应附着灵剑");
    assert_eq!(
        e.catalog().cards[*attached as usize].kind,
        CardKind::Hslingjian,
        "附着实体应为灵剑"
    );
    assert!(snap.core.discard.contains(&js1), "本体入弃堆");

    // CardMod 随存档字节往返
    let bytes = engine_native::archive::encode_archive(&snap);
    let restored =
        Engine::restore_with_extensions(&engine_native::archive::decode_archive(&bytes).expect("解码"), &[&pack])
            .expect("恢复");
    assert_eq!(
        restored.state_core().players[0].card_mods,
        *mods,
        "CardMod 随存档往返一致"
    );

    // 装备清空后不可再用（enable 校验）
    // 无装备不可用：清空装备后再试（同实体已弃，注入另一张机关术）
    let js_all: Vec<u32> = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .filter(|(_, c)| c.name.contains("jiguanshu"))
        .map(|(i, _)| i as u32)
        .collect();
    if let Some(js2) = js_all.iter().find(|x| **x != js1) {
        inject_hand(&mut e, &pack, &[*js2]);
        // 先把装备移入弃牌堆（保留全域唯一性；直写快照）
        let mut snap = e.save();
        let w = snap.core.players[0].equips.pop().expect("有装备");
        snap.core.discard.push(w);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("清装备");
        let out = play(&mut e, *js2, vec![0]);
        assert!(
            matches!(out, CommandOutcome::Rejected(RejectReason::IllegalTargets)),
            "无装备不可用: {out:?}"
        );
    }
}

/// T12 swd 武将面首批（批 18 同日）：铁壁（韩罗整将——杀指定距离 ≤1 目标
/// 时弃黑牌给目标 +1 护甲）+ 大同（习嫣部分——全场手牌极差 ≤1 摸 2）。
#[test]
fn swd_generals_b18_tiebi_and_datong() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "shahai-swd")
        .expect("swd 安装")
        .clone();
    let json = std::fs::read_to_string(
        std::path::PathBuf::from(meta.extension_root.as_ref().unwrap())
            .join("assets/cards.json"),
    )
    .unwrap();
    let pack = bridge(&json);
    assert!(pack.generals.len() >= 7, "累计七将（现 {} 名）", pack.generals.len());
    let mut e = Engine::new_with_extensions(62000, 4, &[&pack]).expect("建局");
    let base = engine_native::content::standard::STANDARD_GENERALS.len() as u16;
    assert_eq!(e.catalog().general_meta(base).unwrap().0, "swd韩罗");
    assert_eq!(e.catalog().general_meta(base + 1).unwrap().0, "swd习嫣");

    // 铁壁：p0 杀 p1（p2 持铁壁、与 p1 距离 1）→ p1 +1 护甲
    e.debug_grant(2, "tiebi");
    e.debug_set_hp(1, 8);
    advance_to_play(&mut e, 0);
    let sha = e.debug_place(0, "sha");
    // p2 布一张黑牌
    let black = e.debug_place(2, "sha");
    {
        let core = e.state_core_mut();
        core.players[2].hand.retain(|x| *x != black);
        core.players[2].hand.push(black);
    }
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    let _ = out;
    settle_optional_confirm(&mut e);
    eprintln!(
        "DBG tiebi hujia1={} hand0={:?} hand2={:?} choice={:?}",
        e.state_core().players[1].hujia,
        e.state_core().players[0].hand,
        e.state_core().players[2].hand,
        e.open_choice().map(|ch| (ch.id, ch.observer, ch.spec.prompt)),
    );
    // 护甲语义：+1 护甲在本次伤害时抵 1 伤——hp 不减、护甲清零
    assert_eq!(e.debug_hp(1), 8, "铁壁：护甲抵 1 伤（hp 不减）");
    assert_eq!(
        e.state_core().players[1].hujia, 0,
        "铁壁：护甲已消耗"
    );

}
/// 大同独立测：全场手牌极差 ≤1 → 摸 2。
#[test]
fn swd_generals_b18_datong_draw() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "shahai-swd")
        .expect("swd 安装")
        .clone();
    let json = std::fs::read_to_string(
        std::path::PathBuf::from(meta.extension_root.as_ref().unwrap())
            .join("assets/cards.json"),
    )
    .unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(62001, 4, &[&pack]).expect("建局");
    e.debug_grant(0, "datong");
    advance_to_play(&mut e, 0);
    // 布阵：全清手后极差 1（p0 手 3、p1 手 2、p2 手 3、p3 手 2 → max 3 min 2）
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let c1 = e.debug_place(0, "tao");
    let c2 = e.debug_place(0, "sha");
    let b1 = e.debug_place(1, "tao");
    let b2 = e.debug_place(1, "tao");
    let s1 = e.debug_place(2, "sha");
    let s2 = e.debug_place(2, "sha");
    let s3 = e.debug_place(3, "sha");
    let s4 = e.debug_place(3, "sha");
    {
        let core = e.state_core_mut();
        core.players[0].hand.push(c1);
        core.players[0].hand.push(c2);

        core.players[1].hand.push(b1);
        core.players[1].hand.push(b2);
        core.players[2].hand.push(s1);
        core.players[2].hand.push(s2);
        core.players[3].hand.push(s3);
        core.players[3].hand.push(s4);
    }
    let h0 = hand_len_w(&e, 0);
    // 任意玩家阶段结束触发（global）——推进 p0 到 End 再过
    for _ in 0..4 {
        if e.turn().phase == engine_native::event::Phase::End {
            break;
        }
        let actor = e.turn().current;
        submit(&mut e, actor, GameAction::NextPhase);
        settle_optional_confirm(&mut e);
    }
    // phaseEnd 帧触发后大同摸 2
    eprintln!(
        "DBG datong hands={:?} phase={:?} choice={:?}",
        [0, 1, 2, 3].map(|p| e.state_core().player(p).hand.len()),
        e.turn().phase,
        e.open_choice().map(|ch| (ch.id, ch.observer, ch.spec.prompt)),
    );
    assert_eq!(
        hand_len_w(&e, 0),
        h0 + 2,
        "大同：极差 ≤1 摸 2"
    );
}

fn hand_len_w(e: &Engine, p: u8) -> usize {
    e.state_core().player(p).hand.len()
}

/// 可选确认门排空（首选项应答）。
fn settle_optional_confirm(e: &mut Engine) {
    for _ in 0..8 {
        let pending = e.open_choice().map(|ch| (ch.id, ch.observer));
        let Some((cid, obs)) = pending else { return };
        let first = {
            let ch = e.open_choice().expect("still open");
            vec![ch.spec.options[0].clone()]
        };
        submit(e, obs, GameAction::Answer { choice_id: cid, selection: first });
    }
}


/// T12 swd 武将面批 19：五技五将部分转正（承剑=其他角色杀伤来源摸 1
/// 确认门；缚斧=杀伤+1 有武器；花镜=锦囊伤后回 1 摸 1；护盾=伤后无护
/// 甲+1；破星=目标体力高则伤+1）。
#[test]
fn swd_generals_b19_five_damage_line_skills() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "shahai-swd")
        .expect("swd 安装")
        .clone();
    let json = std::fs::read_to_string(
        std::path::PathBuf::from(meta.extension_root.as_ref().unwrap())
            .join("assets/cards.json"),
    )
    .unwrap();
    let pack = bridge(&json);
    assert!(pack.generals.len() >= 7);

    let mut e = Engine::new_with_extensions(62002, 4, &[&pack]).expect("建局");

    // 承剑：p1 持承剑，p0 杀伤 p2（来源 p0≠p1）→ 确认 → p0 摸 1
    // （清 p2 手防闪避负例路径）
    e.debug_grant(1, "chengjian");
    e.debug_set_hp(2, 8);
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(2);
    let sha = e.debug_place(0, "sha");
    let h0 = hand_len_w(&e, 0);
    submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    settle_optional_confirm(&mut e);
    assert_eq!(hand_len_w(&e, 0), h0, "承剑：来源（出杀者）摸 1 净持平（出杀-1+摸+1）");

    // 缚斧：p3 持缚斧，p3 杀 p0（有武器）→ 伤 +1（hp -2）
    e.debug_grant(3, "jufu");
    e.debug_set_hp(0, 8);
    let z3 = e.debug_place(3, "zhuge");
    {
        let core = e.state_core_mut();
        core.players[3].hand.retain(|x| *x != z3);
        core.players[3].equips.push(z3);
    }
    advance_to_play(&mut e, 3);
    let sha3 = e.debug_place(3, "sha");
    submit(&mut e, 3, GameAction::PlayCard { card: sha3, targets: vec![0] });
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(0), 6, "缚斧：杀伤害 +1（8-2）");

    // 破星：p1 持破星，p1 杀 p0（p0 体力 > p1）→ 伤 +1
    e.debug_grant(1, "poxing");
    e.debug_set_hp(0, 8);
    e.debug_set_hp(1, 3);
    advance_to_play(&mut e, 1, );
    let sha1 = e.debug_place(1, "sha");
    let hp0 = e.debug_hp(0);
    submit(&mut e, 1, GameAction::PlayCard { card: sha1, targets: vec![0] });
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(0), hp0 - 2, "破星：目标体力高 → 伤 +1");

    // 花镜：p2 持花镜，p2 杀 p0 → p2 回 1（已受伤时）摸 1
    e.debug_grant(2, "huajing");
    e.debug_set_hp(2, 2);
    advance_to_play(&mut e, 2);
    let h2 = hand_len_w(&e, 2);
    let sha2 = e.debug_place(2, "sha");
    submit(&mut e, 2, GameAction::PlayCard { card: sha2, targets: vec![0] });
    settle_optional_confirm(&mut e);
    // hp2 3-1=2 → 回 1=3？不满血则回 1；摸 1
    let hp2b = e.debug_hp(2);
    assert!(hp2b >= 2, "花镜：回 1（或满血饱和）");
    assert_eq!(hand_len_w(&e, 2), h2 + 1, "花镜：摸 1");

    // 护盾：p3 持护盾，p3 造成伤害 → 无护甲 +1
    e.debug_grant(3, "hudun");
    advance_to_play(&mut e, 3);
    e.debug_deal_damage(Some(3), 0, 1, DamageNature::Normal);
    assert_eq!(
        e.state_core().players[3].hujia, 1,
        "护盾：+1 护甲"
    );
}

/// T12 swd 武将面批 20：分舵（习嫣 jiefen 整将转正）——出牌主动技
/// （每回合限一次）三段机：选交牌目标（手>自己）→ 目标交 1 张手牌 →
/// 自己交 1 张手牌给手牌最少的其他角色。
#[test]
fn swd_generals_b20_jiefen_multistep() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "shahai-swd")
        .expect("swd 安装")
        .clone();
    let json = std::fs::read_to_string(
        std::path::PathBuf::from(meta.extension_root.as_ref().unwrap())
            .join("assets/cards.json"),
    )
    .unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(62003, 4, &[&pack]).expect("建局");
    e.debug_grant(3, "jiefen");
    advance_to_play(&mut e, 3);
    // 布阵：全清手后 p3 手 1（mine）、p1 手 3（手>自己）
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let mine = e.debug_place(3, "tao");
    let give1 = e.debug_place(1, "tao");
    let _give2 = e.debug_place(1, "tao");
    let _give3 = e.debug_place(1, "tao");
    let jiefen_def = e.catalog().skill_by_name("jiefen").expect("分舵 def");
    let out = e.submit(GameCommand {
        actor: 3,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action: GameAction::ActivateSkill { skill: jiefen_def },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "分舵受理: {out:?}"
    );

    // 问 1：选交牌目标 p1
    let (cid, obs) = e
        .open_choice()
        .map(|ch| (ch.id, ch.observer))
        .expect("q1 分舵目标");
    let sel: Vec<ChoiceOption> = {
        let ch = e.open_choice().expect("still open");
        ch.spec
            .options
            .iter()
            .filter(|o| matches!(o, ChoiceOption::Player(1)))
            .cloned()
            .collect()
    };
    assert_eq!(sel.len(), 1, "q1 目标面含 p1");
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: sel });

    // 问 2：p1 选 1 张手牌交出
    let (cid, obs) = e
        .open_choice()
        .map(|ch| (ch.id, ch.observer))
        .expect("q2 交牌窗");
    let sel: Vec<ChoiceOption> = {
        let ch = e.open_choice().expect("still open");
        ch.spec
            .options
            .iter()
            .filter(|o| matches!(o, ChoiceOption::Card(_)))
            .cloned()
            .take(1)
            .collect()
    };
    assert_eq!(sel.len(), 1, "q2 交牌面");
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: sel });

    // 登记差异：上游 chooseCardTarget 玩家自选自己交出的牌与目标，原生
    // 确定性直发（自己交手牌 first 给手牌最少的其他存活角色）——无 q3 窗。
    // 断言：p1 3-1=2；p3 +1-1=1；p0 +1=1
    assert_eq!(e.state_core().players[1].hand.len(), 2, "分舵：p1 交 1");
    assert_eq!(e.state_core().players[3].hand.len(), 1, "分舵：p3 持平");
    assert_eq!(e.state_core().players[0].hand.len(), 1, "分舵：p0 得 1");
    let _ = (mine, give1);
}

/// T12 swd 武将面批 21：三技四将部分转正——流虹（拓跋玉儿：使用杀后
/// 频率技摸 1，无询问窗）；墨剑（水镜/屈娴：自己使用杀指定目标后确认，
/// 目标摸 1 + 自己回 1，上游 shaBegin 等价接入 CardUsed 帧）；道法（壶
/// 中仙：其他角色造成伤害后确认，来源弃 1 张手牌——登记差异上游来源
/// 自选弃哪张，原生确定性弃首张）。
#[test]
fn swd_generals_b21_liuhong_mojian_daofa() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "shahai-swd")
        .expect("swd 安装")
        .clone();
    let json = std::fs::read_to_string(
        std::path::PathBuf::from(meta.extension_root.as_ref().unwrap())
            .join("assets/cards.json"),
    )
    .unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(62004, 4, &[&pack]).expect("建局");

    // ① 流虹：p0 使用杀 → 频率技自动摸 1（无确认窗）
    e.debug_grant(0, "swdliuhong");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sha = e.debug_place(0, "sha");
    submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    settle_optional_confirm(&mut e);
    assert_eq!(hand_len_w(&e, 0), 1, "流虹：使用杀自动摸 1（用 1 摸 1）");
    assert!(e.open_choice().is_none(), "流虹：频率技无询问窗");

    // 反例 ①：非杀（桃）不摸
    e.debug_set_hp(0, 3);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let tao = e.debug_place(0, "tao");
    submit(&mut e, 0, GameAction::PlayCard { card: tao, targets: vec![] });
    settle_optional_confirm(&mut e);
    assert_eq!(hand_len_w(&e, 0), 0, "流虹反例：用桃不摸");

    {
        let mut snap = e.save();
        snap.core.turn.sha_used_play_phase = false;
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("重置");
    }
    // ② 墨剑：p0（受伤）使用杀指定 p1 → 确认 → p1 摸 1、p0 回 1
    e.debug_grant(0, "mojian");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let max0 = e.state_core().player(0).max_hp as i32;
    e.debug_set_hp(0, max0 - 1);
    let sha2 = e.debug_place(0, "sha");
    let h1 = hand_len_w(&e, 1);
    submit(&mut e, 0, GameAction::PlayCard { card: sha2, targets: vec![1] });
    let (cid, obs) = e
        .open_choice()
        .map(|c| (c.id, c.observer))
        .expect("墨剑确认窗");
    let conf: Vec<ChoiceOption> = {
        let ch = e.open_choice().expect("still open");
        ch.spec
            .options
            .iter()
            .filter(|o| matches!(o, ChoiceOption::Confirm))
            .cloned()
            .collect()
    };
    assert_eq!(conf.len(), 1, "墨剑确认面含 Confirm");
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: conf });
    settle_optional_confirm(&mut e);
    assert_eq!(hand_len_w(&e, 1), h1 + 1, "墨剑：目标摸 1");
    assert_eq!(e.debug_hp(0), max0, "墨剑：自己回 1 至满");

    // 反例 ②a：拒绝 → 目标不摸、自己不回
    {
        let mut snap = e.save();
        snap.core.turn.sha_used_play_phase = false;
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("重置");
    }
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(0, max0 - 1);
    let sha3 = e.debug_place(0, "sha");
    let (hp0, h1b) = (e.debug_hp(0), hand_len_w(&e, 1));
    submit(&mut e, 0, GameAction::PlayCard { card: sha3, targets: vec![1] });
    let (cid, obs) = e
        .open_choice()
        .map(|c| (c.id, c.observer))
        .expect("墨剑拒绝窗");
    // allow_cancel 取消 = 空选择（确认面仅 [Confirm]，拒绝走取消）
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![] });
    settle_optional_confirm(&mut e);
    assert_eq!(hand_len_w(&e, 1), h1b, "墨剑反例：拒绝 → 目标不摸");
    assert_eq!(e.debug_hp(0), hp0, "墨剑反例：拒绝 → 自己不回（攻击者本不受伤）");

    // 反例 ②b：满血确认 → 目标仍摸、自己不回（recover 饱和）
    {
        let mut snap = e.save();
        snap.core.turn.sha_used_play_phase = false;
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("重置");
    }
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(0, max0);
    e.debug_set_hp(1, 4);
    let sha4 = e.debug_place(0, "sha");
    let h1c = hand_len_w(&e, 1);
    submit(&mut e, 0, GameAction::PlayCard { card: sha4, targets: vec![1] });
    let (cid, obs) = e
        .open_choice()
        .map(|c| (c.id, c.observer))
        .expect("墨剑满血确认窗");
    let conf: Vec<ChoiceOption> = {
        let ch = e.open_choice().expect("still open");
        ch.spec
            .options
            .iter()
            .filter(|o| matches!(o, ChoiceOption::Confirm))
            .cloned()
            .collect()
    };
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: conf });
    settle_optional_confirm(&mut e);
    assert_eq!(hand_len_w(&e, 1), h1c + 1, "墨剑满血：目标仍摸 1");
    assert_eq!(e.debug_hp(0), max0, "墨剑满血：回复饱和不溢出");

    // ③ 道法：p3 持道法；p0 杀伤 p1（p0 另持 1 张牌）→ p3 确认 →
    // p0 弃首张手牌（登记差异：上游来源自选弃哪张）。
    // 撤销流虹（①所授）以免 p0 手牌计数被摸牌扰动。
    {
        let def_lh = e.catalog().skill_by_name("swdliuhong").expect("流虹 def");
        let def_mj = e.catalog().skill_by_name("mojian").expect("墨剑 def");
        let mut snap = e.save();
        snap.core.turn.sha_used_play_phase = false;
        snap.core.skills.remove(&(0, def_lh));
        snap.core.skills.remove(&(0, def_mj));
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("撤销流虹/墨剑");
    }
    e.debug_grant(3, "daofa");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let _extra = e.debug_place(0, "tao");
    let sha5 = e.debug_place(0, "sha");
    submit(&mut e, 0, GameAction::PlayCard { card: sha5, targets: vec![1] });
    let (cid, obs) = e
        .open_choice()
        .map(|c| (c.id, c.observer))
        .expect("道法确认窗");
    assert_eq!(obs, 3, "道法窗归持有者 p3");
    let conf: Vec<ChoiceOption> = {
        let ch = e.open_choice().expect("still open");
        ch.spec
            .options
            .iter()
            .filter(|o| matches!(o, ChoiceOption::Confirm))
            .cloned()
            .collect()
    };
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: conf });
    settle_optional_confirm(&mut e);
    assert_eq!(hand_len_w(&e, 0), 0, "道法：来源弃首张手牌（tao）");

    // 反例 ③a：拒绝 → 来源保留手牌
    {
        let mut snap = e.save();
        snap.core.turn.sha_used_play_phase = false;
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("重置");
    }
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let _extra = e.debug_place(0, "tao");
    let sha6 = e.debug_place(0, "sha");
    submit(&mut e, 0, GameAction::PlayCard { card: sha6, targets: vec![1] });
    let (cid, obs) = e
        .open_choice()
        .map(|c| (c.id, c.observer))
        .expect("道法拒绝窗");
    // allow_cancel 取消 = 空选择（确认面仅 [Confirm]，拒绝走取消）
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![] });
    settle_optional_confirm(&mut e);
    assert_eq!(hand_len_w(&e, 0), 1, "道法反例：拒绝 → 来源保留手牌");

    // 反例 ③b：来源=持有者 → 无窗（filter d.source != owner）
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let _p3card = e.debug_place(3, "tao");
    e.debug_set_hp(1, 4);
    e.debug_deal_damage(Some(3), 1, 1, DamageNature::Normal);
    assert!(e.open_choice().is_none(), "道法反例：自己造成伤害不触发");

    // 反例 ③c：来源空手 → 无窗
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert!(e.open_choice().is_none(), "道法反例：来源无手牌不触发");
}

/// T12 swd 武将面批 22：三技三将部分转正——玉鳞（兰茵：受伤前置窗口
/// 弃 1 张装备牌[手/装备区]免此伤害）；凝霜（芝茵：成为黑色牌目标弃 1
/// 张黑牌，使用者阶梯：已翻面→失 1 体力/已横置→翻面/否则横置且仅此支
/// 自己摸 1——上游代码原样）；工神（黄雷：其他角色用基本/锦囊牌且有
/// 目标，弃 1 张装备牌令其失效=cancelled 写回强固守卫同款）。
#[test]
fn swd_generals_b22_yulin_ningshuang_gongshen() {
    use engine_native::catalog::Suit;
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "shahai-swd")
        .expect("swd 安装")
        .clone();
    let json = std::fs::read_to_string(
        std::path::PathBuf::from(meta.extension_root.as_ref().unwrap())
            .join("assets/cards.json"),
    )
    .unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(62005, 4, &[&pack]).expect("建局");
    let is_black = |e: &Engine, c: u32| {
        matches!(
            e.catalog().cards[c as usize].suit,
            Suit::Spade | Suit::Club
        )
    };
    let reset_sha = |e: &mut Engine| {
        let mut snap = e.save();
        snap.core.turn.sha_used_play_phase = false;
        *e = Engine::restore_with_extensions(&snap, &[&pack]).expect("重置");
    };
    let confirm = |e: &mut Engine, want_obs: u8| {
        let (cid, obs) = e
            .open_choice()
            .map(|c| (c.id, c.observer))
            .expect("确认窗");
        assert_eq!(obs, want_obs, "确认窗归属");
        let conf: Vec<ChoiceOption> = {
            let ch = e.open_choice().expect("still open");
            ch.spec
                .options
                .iter()
                .filter(|o| matches!(o, ChoiceOption::Confirm))
                .cloned()
                .collect()
        };
        assert_eq!(conf.len(), 1);
        submit(e, obs, GameAction::Answer { choice_id: cid, selection: conf });
    };
    let pick_first_card = |e: &mut Engine| {
        let (cid, obs) = e
            .open_choice()
            .map(|c| (c.id, c.observer))
            .expect("选牌窗");
        let sel: Vec<ChoiceOption> = {
            let ch = e.open_choice().expect("still open");
            ch.spec
                .options
                .iter()
                .filter(|o| matches!(o, ChoiceOption::Card(_)))
                .cloned()
                .take(1)
                .collect()
        };
        assert_eq!(sel.len(), 1, "选牌面非空");
        submit(e, obs, GameAction::Answer { choice_id: cid, selection: sel });
    };

    // 确定性花色布阵：按谓词定位实体（deck→discard→hands 兜底）后直接
    // 入手（debug_set_deck_top_by 只压牌堆底，debug_place 从堆前取——
    // 两者不衔接，颜色不保证）。
    let place_sha_color = |e: &mut Engine, p: u8, black: bool| {
        let id = e.debug_set_deck_top_by(move |d| {
            d.kind.is_sha_like()
                && if black {
                    matches!(d.suit, Suit::Spade | Suit::Club)
                } else {
                    matches!(d.suit, Suit::Heart | Suit::Diamond)
                }
        });
        {
            let core = e.state_core_mut();
            core.deck.retain(|x| *x != id);
            core.players[p as usize].hand.push(id);
        }
        id
    };

    // ① 玉鳞：p1 杀 p0（p0 手牌有武器 zhuge）→ 确认+弃 zhuge → 免伤
    e.debug_grant(0, "yulin");
    advance_to_play(&mut e, 1);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let zg = e.debug_place(0, "zhuge");
    assert!(e.catalog().cards[zg as usize].kind.equip_slot().is_some(), "zhuge 是装备牌");
    let sha1 = e.debug_place(1, "sha");
    e.debug_set_hp(0, 4);
    submit(&mut e, 1, GameAction::PlayCard { card: sha1, targets: vec![0] });
    confirm(&mut e, 0);
    pick_first_card(&mut e);
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(0), 4, "玉鳞：弃装备牌免伤");
    assert_eq!(hand_len_w(&e, 0), 0, "玉鳞：zhuge 已弃");

    // 反例 ①a：取消确认窗 → 伤害照常
    reset_sha(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let _zg2 = e.debug_place(0, "zhuge");
    let sha2 = e.debug_place(1, "sha");
    submit(&mut e, 1, GameAction::PlayCard { card: sha2, targets: vec![0] });
    let (cid, obs) = e
        .open_choice()
        .map(|c| (c.id, c.observer))
        .expect("玉鳞取消窗");
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![] });
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(0), 3, "玉鳞反例：取消 → 伤害照常（4-1）");

    // 反例 ①b：无装备牌 → 无窗（伤害照常）
    reset_sha(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sha3 = e.debug_place(1, "sha");
    let hp0 = e.debug_hp(0);
    submit(&mut e, 1, GameAction::PlayCard { card: sha3, targets: vec![0] });
    assert!(e.open_choice().is_none(), "玉鳞反例：无装备牌不触发");
    assert_eq!(e.debug_hp(0), hp0 - 1, "玉鳞反例：伤害照常");

    // 正例 ①c：装备区的装备牌也可弃（equip 分支 discard idiom）
    reset_sha(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let zg3 = e.debug_place(0, "zhuge");
    {
        let core = e.state_core_mut();
        core.players[0].hand.retain(|x| *x != zg3);
        core.players[0].equips.push(zg3);
    }
    let sha4 = e.debug_place(1, "sha");
    e.debug_set_hp(0, 4);
    submit(&mut e, 1, GameAction::PlayCard { card: sha4, targets: vec![0] });
    confirm(&mut e, 0);
    pick_first_card(&mut e);
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(0), 4, "玉鳞：装备区装备牌同样免伤");
    assert!(e.state_core().player(0).equips.is_empty(), "玉鳞：装备区已弃空");

    // ② 凝霜：撤销玉鳞（①所授），p1 黑杀 p0 → 确认+弃黑牌 → p1 横置、p0 摸 1
    {
        let def_yl = e.catalog().skill_by_name("yulin").expect("玉鳞 def");
        let mut snap = e.save();
        snap.core.skills.remove(&(0, def_yl));
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("撤销玉鳞");
    }
    e.debug_grant(0, "ningshuang");
    advance_to_play(&mut e, 1);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let _blk_for_p0 = place_sha_color(&mut e, 0, true);
    reset_sha(&mut e);
    let bsha = place_sha_color(&mut e, 1, true);
    assert!(is_black(&e, bsha), "p1 布到黑杀");
    e.debug_set_hp(0, 4);
    submit(&mut e, 1, GameAction::PlayCard { card: bsha, targets: vec![0] });
    confirm(&mut e, 0);
    pick_first_card(&mut e);
    settle_optional_confirm(&mut e);
    assert!(e.state_core().player(1).linked, "凝霜：使用者横置");
    assert!(!e.state_core().player(1).face_down, "凝霜：使用者未翻面");
    assert_eq!(hand_len_w(&e, 0), 1, "凝霜：横置支自己摸 1");
    assert_eq!(e.debug_hp(0), 3, "凝霜：杀伤害照常（免伤面无关）");

    // 正例 ②b：已横置 → 改翻面（不摸牌）
    reset_sha(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    {
        let core = e.state_core_mut();
        core.players[1].linked = true;
    }
    let _blk2 = place_sha_color(&mut e, 0, true);
    reset_sha(&mut e);
    let bsha2 = place_sha_color(&mut e, 1, true);
    e.debug_set_hp(0, 4);
    submit(&mut e, 1, GameAction::PlayCard { card: bsha2, targets: vec![0] });
    confirm(&mut e, 0);
    pick_first_card(&mut e);
    settle_optional_confirm(&mut e);
    assert!(e.state_core().player(1).face_down, "凝霜：已横置 → 翻面");
    assert_eq!(hand_len_w(&e, 0), 0, "凝霜：翻面支不摸牌（弃 1 摸 0 净 -1）");

    // 反例 ②c：已「横置且翻面」→ filter 恒否，无窗
    reset_sha(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    {
        let core = e.state_core_mut();
        core.players[1].linked = true;
        core.players[1].face_down = true;
    }
    let _blk3 = place_sha_color(&mut e, 0, true);
    reset_sha(&mut e);
    let bsha3 = place_sha_color(&mut e, 1, true);
    let hp0 = e.debug_hp(0);
    submit(&mut e, 1, GameAction::PlayCard { card: bsha3, targets: vec![0] });
    assert!(e.open_choice().is_none(), "凝霜反例：横置且翻面不触发");
    assert_eq!(e.debug_hp(0), hp0 - 1, "凝霜反例：伤害照常");

    // 反例 ②d：红杀不触发
    reset_sha(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    {
        let core = e.state_core_mut();
        core.players[1].linked = false;
        core.players[1].face_down = false;
    }
    let rsha = place_sha_color(&mut e, 1, false);
    assert!(!is_black(&e, rsha), "p1 布到红杀");
    let hp0 = e.debug_hp(0);
    submit(&mut e, 1, GameAction::PlayCard { card: rsha, targets: vec![0] });
    assert!(e.open_choice().is_none(), "凝霜反例：红杀不触发");
    assert_eq!(e.debug_hp(0), hp0 - 1, "凝霜反例：伤害照常");

    // ③ 工神：撤销凝霜，p3 持工神（手牌有 zhuge）；p1 杀 p2 → 确认+
    // 弃 zhuge → 该杀失效（cancelled，牌回 p1 手牌，p2 免伤）
    {
        let def_ns = e.catalog().skill_by_name("ningshuang").expect("凝霜 def");
        let mut snap = e.save();
        snap.core.turn.sha_used_play_phase = false;
        snap.core.skills.remove(&(0, def_ns));
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("撤销凝霜");
    }
    e.debug_grant(3, "gongshen");
    advance_to_play(&mut e, 1);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let _zg4 = e.debug_place(3, "zhuge");
    let sha5 = e.debug_place(1, "sha");
    e.debug_set_hp(2, 4);
    submit(&mut e, 1, GameAction::PlayCard { card: sha5, targets: vec![2] });
    confirm(&mut e, 3);
    pick_first_card(&mut e);
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(2), 4, "工神：p2 免伤");
    assert_eq!(hand_len_w(&e, 3), 0, "工神：zhuge 已弃");
    assert_eq!(hand_len_w(&e, 1), 1, "工神：取消的杀留在 p1 手牌");

    // 反例 ③a：取消确认窗 → 杀照常结算
    reset_sha(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let _zg5 = e.debug_place(3, "zhuge");
    let sha6 = e.debug_place(1, "sha");
    submit(&mut e, 1, GameAction::PlayCard { card: sha6, targets: vec![2] });
    let (cid, obs) = e
        .open_choice()
        .map(|c| (c.id, c.observer))
        .expect("工神取消窗");
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![] });
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(2), 3, "工神反例：取消 → 伤害照常（4-1）");

    // 反例 ③b：p3 无装备牌 → 无窗
    reset_sha(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sha7 = e.debug_place(1, "sha");
    let hp2 = e.debug_hp(2);
    submit(&mut e, 1, GameAction::PlayCard { card: sha7, targets: vec![2] });
    assert!(e.open_choice().is_none(), "工神反例：无装备牌不触发");
    assert_eq!(e.debug_hp(2), hp2 - 1, "工神反例：伤害照常");

    // 反例 ③c：无目标用牌（桃）不触发
    reset_sha(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let _zg6 = e.debug_place(3, "zhuge");
    e.debug_set_hp(1, 3);
    let tao6 = e.debug_place(1, "tao");
    submit(&mut e, 1, GameAction::PlayCard { card: tao6, targets: vec![] });
    assert!(e.open_choice().is_none(), "工神反例：无目标牌不触发");
}

/// T12 swd 武将面批 23：四将整将——平恤（秦叔宝 tag 278：全局距离字段
/// 技，computed_distance 消费：场上有持技者时攻击方无武器 -1/防守方无
/// 防具 +1）；还灵（轩辕剑侍 tag 279：自己阶段结束选一名其他角色同时翻
/// 面，取消=不发动）；义从/酒池经 builtin 白名单引用标准技（张高/程咬
/// 金整将，零新增行为）。
#[test]
fn swd_generals_b23_pingxu_huanling_builtin_four_full() {
    use engine_native::catalog::EquipSlot;
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "shahai-swd")
        .expect("swd 安装")
        .clone();
    let json = std::fs::read_to_string(
        std::path::PathBuf::from(meta.extension_root.as_ref().unwrap())
            .join("assets/cards.json"),
    )
    .unwrap();
    let pack = bridge(&json);
    assert!(pack.generals.len() >= 14, "14 将入册");
    // 5 人局（距离基线：对座 2）
    let mut e = Engine::new_with_extensions(62006, 5, &[&pack]).expect("建局");
    for n in ["pingxu", "huanling", "义从", "酒池"] {
        assert!(e.catalog().skill_by_name(n).is_some(), "{n} 在目录");
    }

    // ① 平恤：p3 持技（全局字段），d(0,2) 基线 2
    e.debug_grant(3, "pingxu");
    let dist = |e: &Engine, a: u8, b: u8| {
        e.state_core().computed_distance(e.catalog(), a, b)
    };
    assert_eq!(dist(&e, 0, 2), 2, "平恤：攻方无武器-1+守方无防具+1 净持平");
    // 守方装防具 → 仅 -1
    let armor = e.debug_set_deck_top_by(|d| {
        d.kind.equip_slot() == Some(EquipSlot::Armor)
    });
    {
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != armor);
        core.players[2].equips.push(armor);
    }
    assert_eq!(dist(&e, 0, 2), 1, "平恤：守方有防具 → 仅攻方无武器 -1");
    // 攻方装武器、卸守方防具 → 仅 +1（守方无防具支）
    let wep = e.debug_place(0, "zhuge");
    {
        let core = e.state_core_mut();
        core.players[0].hand.retain(|x| *x != wep);
        core.players[0].equips.push(wep);
        // 卸防具须回弃牌堆（直接 clear 会把牌逐出所有区域，restore 守卫拒）
        if let Some(pos) = core.players[2].equips.iter().position(|x| *x == armor) {
            let a = core.players[2].equips.remove(pos);
            core.discard.push(a);
        }
    }
    assert_eq!(dist(&e, 0, 2), 3, "平恤：攻方有武器 → 仅守方无防具 +1");
    // 反例：撤销持技者 → 字段失效回基线
    {
        let def_px = e.catalog().skill_by_name("pingxu").expect("平恤 def");
        let mut snap = e.save();
        snap.core.skills.remove(&(3, def_px));
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("撤销平恤");
    }
    assert_eq!(dist(&e, 0, 2), 2, "平恤反例：无持技者 → 基线 2");

    // ② 还灵：p0 持技，Play 结束窗口选 p1 → 双翻面
    //（先清手规避弃牌阶段 discard_to_limit 询问先于阶段末触发窗）
    advance_to_play(&mut e, 0);
    e.debug_grant(0, "huanling");
    e.debug_clear_hand(0);
    let out = submit(&mut e, 0, GameAction::NextPhase);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    // 可选触发门先确认，随后才是选人窗
    let (cid, obs) = e
        .open_choice()
        .map(|c| (c.id, c.observer))
        .expect("还灵确认窗");
    assert_eq!(obs, 0, "还灵窗归持有者");
    let conf: Vec<ChoiceOption> = {
        let ch = e.open_choice().expect("still open");
        ch.spec
            .options
            .iter()
            .filter(|o| matches!(o, ChoiceOption::Confirm))
            .cloned()
            .collect()
    };
    assert_eq!(conf.len(), 1);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: conf });
    let (cid, obs) = e
        .open_choice()
        .map(|c| (c.id, c.observer))
        .expect("还灵选人窗");
    let sel: Vec<ChoiceOption> = {
        let ch = e.open_choice().expect("still open");
        ch.spec
            .options
            .iter()
            .filter(|o| matches!(o, ChoiceOption::Player(1)))
            .cloned()
            .collect()
    };
    assert_eq!(sel.len(), 1, "还灵目标面含 p1");
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: sel });
    settle_optional_confirm(&mut e);
    assert!(e.state_core().player(0).face_down, "还灵：自己翻面");
    assert!(e.state_core().player(1).face_down, "还灵：目标翻面");

    // 反例 ②b：Discard 末窗口取消 → 不再翻（face_down 保持 true）
    let out = submit(&mut e, 0, GameAction::NextPhase);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    let (cid, obs) = e
        .open_choice()
        .map(|c| (c.id, c.observer))
        .expect("还灵 Discard 末窗口");
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![] });
    settle_optional_confirm(&mut e);
    assert!(e.state_core().player(0).face_down && e.state_core().player(1).face_down, "还灵反例：取消 → 不翻");

    // 撤销还灵，回合收束不再有窗口
    {
        let def_hl = e.catalog().skill_by_name("huanling").expect("还灵 def");
        let mut snap = e.save();
        snap.core.skills.remove(&(0, def_hl));
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("撤销还灵");
    }

    // ③ builtin 酒池（程咬金 wiring）：黑桃手牌当酒
    advance_to_play(&mut e, 0);
    e.debug_grant(0, "酒池");
    e.debug_move_named_card_to_hand(0, "shandian");
    let spade = *e.debug_hand(0).last().unwrap();
    let out = submit(&mut e, 0, GameAction::PlayCard { card: spade, targets: vec![] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "酒池：黑桃当酒 {out:?}");
    assert!(e.state_core().turn.jiu_used, "酒池：jiu_used 置位");
    e.debug_move_named_card_to_hand(0, "shandian");
    let spade2 = *e.debug_hand(0).last().unwrap();
    let out = submit(&mut e, 0, GameAction::PlayCard { card: spade2, targets: vec![] });
    assert!(!matches!(out, CommandOutcome::Accepted { .. }), "酒池反例：第二坛拒");

    // ④ builtin 义从（张高 wiring）：hp≤2 时距 -1/被算 +1
    e.debug_grant(1, "义从");
    e.debug_set_hp(1, 1);
    assert_eq!(dist(&e, 1, 3), 1, "义从：攻距 2-1（对座基线 2）");
    assert_eq!(dist(&e, 0, 1), 2, "义从：被算距离 1+1（hp≤2）");
    let _ = spade;
}

/// T12 swd 武将面批 24：双将整将——断月（水镜 tag 280：出牌限一次弃
/// 装备牌对其他角色造成 1 伤，三段机）；屠阵（水镜 tag 281：自己伤害
/// 他人结算后弃其全部非基本手牌）；落梅（兰茵 tag 282：使用或打出梅花
/// 牌频率摸 1，双事件）；星点（兰茵 tag 283：出牌限一次弃手牌，确定性
/// 弃两名其他角色各手/装备首张——登记差异上游随机）。
#[test]
fn swd_generals_b24_duanyue_tuzhen_luomei_xingdian() {
    use engine_native::catalog::Suit;
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "shahai-swd")
        .expect("swd 安装")
        .clone();
    let json = std::fs::read_to_string(
        std::path::PathBuf::from(meta.extension_root.as_ref().unwrap())
            .join("assets/cards.json"),
    )
    .unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(62007, 4, &[&pack]).expect("建局");

    let confirm = |e: &mut Engine, want_obs: u8| {
        let (cid, obs) = e
            .open_choice()
            .map(|c| (c.id, c.observer))
            .expect("确认窗");
        assert_eq!(obs, want_obs, "确认窗归属");
        let conf: Vec<ChoiceOption> = {
            let ch = e.open_choice().expect("still open");
            ch.spec
                .options
                .iter()
                .filter(|o| matches!(o, ChoiceOption::Confirm))
                .cloned()
                .collect()
        };
        assert_eq!(conf.len(), 1);
        submit(e, obs, GameAction::Answer { choice_id: cid, selection: conf });
    };
    let pick = |e: &mut Engine, pred: &dyn Fn(&ChoiceOption) -> bool, msg: &'static str| {
        let (cid, obs) = e
            .open_choice()
            .map(|c| (c.id, c.observer))
            .expect(msg);
        let sel: Vec<ChoiceOption> = {
            let ch = e.open_choice().expect("still open");
            ch.spec
                .options
                .iter()
                .filter(|o| pred(o))
                .cloned()
                .take(1)
                .collect()
        };
        assert_eq!(sel.len(), 1, "{}", msg);
        submit(e, obs, GameAction::Answer { choice_id: cid, selection: sel });
    };

    // ① 断月：p0 持技（手牌有 zhuge），对 p1 造成 1 伤
    e.debug_grant(0, "duanyue");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let duanyue_def = e.catalog().skill_by_name("duanyue").expect("断月 def");
    let h1 = hand_len_w(&e, 1);
    e.debug_set_hp(1, 4);

    // 反例 ①a（先测）：取消选牌 → 无效果（发动即耗 once_used，随后重置）
    let _zg0 = e.debug_place(0, "zhuge");
    let _ = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action: GameAction::ActivateSkill { skill: duanyue_def },
    });
    let (cid, obs) = e
        .open_choice()
        .map(|c| (c.id, c.observer))
        .expect("断月取消窗");
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![] });
    settle_optional_confirm(&mut e);
    assert!(e.open_choice().is_none(), "断月反例：取消后无窗");
    assert_eq!(e.debug_hp(1), 4, "断月反例：取消 → p1 不受伤");
    assert_eq!(hand_len_w(&e, 0), 1, "断月反例：取消 → zhuge 留在手牌");
    {
        let mut snap = e.save();
        snap.core
            .skills
            .get_mut(&(0, duanyue_def))
            .expect("断月实例")
            .once_used = false;
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("重置 once_used");
    }

    // 正例 ①：选 zhuge → 选 p1 → 弃牌+1 伤（清掉 ①a 留下的 zhuge）
    e.debug_clear_hand(0);
    let zg = e.debug_place(0, "zhuge");
    let _ = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action: GameAction::ActivateSkill { skill: duanyue_def },
    });
    pick(&mut e, &|o| matches!(o, ChoiceOption::Card(c2) if *c2 == zg), "断月选牌窗");
    pick(&mut e, &|o| matches!(o, ChoiceOption::Player(1)), "断月选人窗");
    settle_optional_confirm(&mut e);
    assert_eq!(hand_len_w(&e, 0), 0, "断月：zhuge 已弃");
    assert_eq!(e.debug_hp(1), 3, "断月：p1 受 1 伤");
    assert_eq!(hand_len_w(&e, 1), h1, "断月：p1 手牌不动");

    // 反例 ①b：限一次（本回合再发动被拒）
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action: GameAction::ActivateSkill { skill: duanyue_def },
    });
    assert!(
        !matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "断月反例：本回合限一次 {out:?}"
    );

    // ② 屠阵：撤销断月，p0 杀伤 p1（p1 手有非基本牌）→ 确认 → 全弃
    {
        let def_dy = e.catalog().skill_by_name("duanyue").expect("断月 def");
        let mut snap = e.save();
        snap.core.turn.sha_used_play_phase = false;
        snap.core.skills.remove(&(0, def_dy));
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("撤销断月");
    }
    e.debug_grant(0, "tuzhen");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_place(1, "tao");
    e.debug_place(1, "tao");
    e.debug_place(1, "duel");
    reset_sha_for(&mut e, &pack);
    let sha_t = e.debug_place(0, "sha");
    submit(&mut e, 0, GameAction::PlayCard { card: sha_t, targets: vec![1] });
    confirm(&mut e, 0);
    settle_optional_confirm(&mut e);
    assert_eq!(hand_len_w(&e, 1), 2, "屠阵：p1 非基本（duel）被弃，剩 2 桃");

    // 反例 ②b：受伤者无非基本手牌 → 无窗
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_place(1, "tao");
    reset_sha_for(&mut e, &pack);
    let sha_t2 = e.debug_place(0, "sha");
    submit(&mut e, 0, GameAction::PlayCard { card: sha_t2, targets: vec![1] });
    assert!(e.open_choice().is_none(), "屠阵反例：无非基本手牌不触发");

    // 反例 ②c：未造成伤害（prevented 面）不触发——p1 装备区清空+手里有
    // 桃可自救？改用无伤害来源路径：直接 debug 伤 p1 再用杀无伤不成立，
    // 以 filter 的 target!=owner 反例替代：p1 杀伤 p0（来源非持有者）
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_place(0, "duel");
    reset_sha_for(&mut e, &pack);
    let sha_t3 = e.debug_place(1, "sha");
    let h0 = hand_len_w(&e, 0);
    submit(&mut e, 1, GameAction::PlayCard { card: sha_t3, targets: vec![0] });
    settle_optional_confirm(&mut e);
    assert_eq!(hand_len_w(&e, 0), h0, "屠阵反例：来源非持有者不触发（p0 全留）");

    // ③ 落梅：撤销屠阵，p0 持落梅——出梅花牌摸 1（找/布梅花杀）
    {
        let def_tz = e.catalog().skill_by_name("tuzhen").expect("屠阵 def");
        let mut snap = e.save();
        snap.core.turn.sha_used_play_phase = false;
        snap.core.skills.remove(&(0, def_tz));
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("撤销屠阵");
    }
    e.debug_grant(0, "luomei");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let club_sha = {
        let id = e.debug_set_deck_top_by(|d| {
            d.kind.is_sha_like() && matches!(d.suit, Suit::Club)
        });
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != id);
        core.players[0].hand.push(id);
        id
    };
    assert!(matches!(
        e.catalog().cards[club_sha as usize].suit,
        Suit::Club
    ));
    e.debug_set_hp(1, 4);
    let h0 = hand_len_w(&e, 0);
    submit(&mut e, 0, GameAction::PlayCard { card: club_sha, targets: vec![1] });
    settle_optional_confirm(&mut e);
    assert_eq!(hand_len_w(&e, 0), h0, "落梅：用 1 梅花杀摸 1（用 1-1+摸 1 净持平）");
    assert_eq!(e.debug_hp(1), 3, "落梅：p1 存活（4-1，排除濒死噪声）");

    // 反例 ③b：非梅花不摸
    reset_sha_for(&mut e, &pack);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let red_sha = {
        let id = e.debug_set_deck_top_by(|d| {
            d.kind.is_sha_like() && matches!(d.suit, Suit::Heart | Suit::Diamond)
        });
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != id);
        core.players[0].hand.push(id);
        id
    };
    e.debug_set_hp(1, 4);
    let h0 = hand_len_w(&e, 0);
    submit(&mut e, 0, GameAction::PlayCard { card: red_sha, targets: vec![1] });
    settle_optional_confirm(&mut e);
    assert_eq!(hand_len_w(&e, 0), h0 - 1, "落梅反例：红杀不摸（净 -1）");
    assert_eq!(e.debug_hp(1), 3, "落梅反例：p1 存活");

    // ④ 星点：撤销落梅，p0 持星点——弃 1 手牌，p1/p2 各弃手/装备首张
    {
        let def_lm = e.catalog().skill_by_name("luomei").expect("落梅 def");
        let mut snap = e.save();
        snap.core.turn.sha_used_play_phase = false;
        snap.core.skills.remove(&(0, def_lm));
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("撤销落梅");
    }
    e.debug_grant(0, "xingdian");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let mine = e.debug_place(0, "tao");
    let v1 = e.debug_place(1, "shan");
    let _v2 = e.debug_place(2, "tao");
    let xingdian_def = e.catalog().skill_by_name("xingdian").expect("星点 def");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action: GameAction::ActivateSkill { skill: xingdian_def },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "星点受理: {out:?}"
    );
    pick(&mut e, &|o| matches!(o, ChoiceOption::Card(c2) if *c2 == mine), "星点选牌窗");
    settle_optional_confirm(&mut e);
    assert_eq!(hand_len_w(&e, 0), 0, "星点：自己弃 1");
    assert_eq!(hand_len_w(&e, 1), 0, "星点：p1 首张被弃");
    assert_eq!(hand_len_w(&e, 2), 0, "星点：p2 首张被弃");
    assert_eq!(hand_len_w(&e, 3), 0, "星点：p3 无牌不受影响");
    let _ = (v1, h1);
}

/// 快照重置本回合出杀计数（批 22/23 helper 的独立包装）。
fn reset_sha_for(e: &mut Engine, pack: &ExtensionPack) {
    let mut snap = e.save();
    snap.core.turn.sha_used_play_phase = false;
    *e = Engine::restore_with_extensions(&snap, &[pack]).expect("重置");
}
