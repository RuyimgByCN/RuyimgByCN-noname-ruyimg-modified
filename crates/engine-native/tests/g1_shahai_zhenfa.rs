//! 整包首切片（U05 杀海拾遗 zhenfa 子包，规则 v110）：地载阵——
//! 所有非大势力角色各摸一张牌（大势力=已明置势力存活人数最大者，
//! effective_faction 推导；上游 mode:["guozhan"]，队列/围攻依赖的其余
//! 12 张中的 5 种启用牌登记依赖）+ 真实宿主全生命周期。
use content_runtime::{discover_extensions, import_extensions, InstallOptions, PackageStore};
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::catalog::{CardKind, Faction};
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

fn bridge(json: &str) -> engine_native::content::extension::ExtensionPack {
    let out = content_native_loader::bridge_pack(json).expect("zhenfa 桥接");
    assert_eq!(out.pack.def.name, "shahai-zhenfa-upstream-noname");
    out.pack
}

fn advance_to_play(e: &mut Engine, seat: u8) {
    for _ in 0..96 {
        if e.turn().current == seat && e.turn().phase == Phase::Play && e.open_choice().is_none() {
            break;
        }
        if let Some(c) = e.open_choice() {
            let (cid, obs) = (c.id, c.observer);
            drop(c);
            submit(e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
        }
        let actor = e.turn().current;
        let _ = submit(e, actor, GameAction::NextPhase);
    }
}

fn play(e: &mut Engine, card: u32, targets: Vec<u8>) -> CommandOutcome {
    submit(e, e.turn().current, GameAction::PlayCard { card, targets })
}

/// 找不同势力的两个武将名（魏 + 蜀；STANDARD_GENERALS 基础表）。
fn two_generals() -> (String, String) {
    let wei = engine_native::content::standard::STANDARD_GENERALS
        .iter()
        .find(|(_, f, ..)| *f == Faction::Wei)
        .map(|(n, ..)| n.to_string())
        .expect("魏将");
    let shu = engine_native::content::standard::STANDARD_GENERALS
        .iter()
        .find(|(_, f, ..)| *f == Faction::Shu)
        .map(|(n, ..)| n.to_string())
        .expect("蜀将");
    (wei, shu)
}

/// 地载阵（zhenfa 首切片，规则 v110）：所有非大势力存活角色各摸一张
/// （大势力=effective_faction 存活人数最大者，并列同算；未明置/无主将
/// 不属大势力）。子局①：魏魏蜀——两魏并列最大均大，蜀非大独摸；
/// 子局②：全魏——无非大者，出牌被拒（上游 enable 同口径）。
#[test]
fn zf_dizaizhen_major_faction_draw() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-zhenfa-upstream-noname")
        .expect("zhenfa 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);
    let base_cards = Engine::new(1, 3).catalog().cards.len();
    let (wei, shu) = two_generals();

    // 子局①：0/1 魏（并列最大→大势力）、2 蜀（非大）→ 仅 2 号摸 1
    let mut e = Engine::new_with_extensions(8101, 3, &[&pack]).expect("建局");
    assert_eq!(e.catalog().cards.len(), base_cards + 3, "zhenfa 三张地载阵入目录");
    advance_to_play(&mut e, 0);
    e.debug_assemble_general(0, &wei);
    e.debug_assemble_general(1, &wei);
    e.debug_assemble_general(2, &shu);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    let dz = (0..e.catalog().cards.len() as u32)
        .find(|i| e.catalog().cards[*i as usize].kind == CardKind::Dizaizhen)
        .expect("地载阵在目录");
    {
        let mut snap = e.save();
        for p in snap.core.players.iter_mut() {
            p.hand.retain(|c| *c != dz);
        }
        snap.core.deck.retain(|c| *c != dz);
        snap.core.discard.retain(|c| *c != dz);
        snap.core.table.retain(|c| *c != dz);
        snap.core.players[0].hand.insert(0, dz);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("地载阵注入");
    }
    let out = play(&mut e, dz, vec![]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        drop(c);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    assert_eq!(e.debug_hand(2).len(), 1, "非大势力（蜀）摸 1");
    assert_eq!(e.debug_hand(0).len(), 0, "大势力（魏）不摸");
    assert_eq!(e.debug_hand(1).len(), 0, "大势力（魏）不摸");
    assert!(
        !e.save().core.table.iter().any(|c| e.catalog().cards[*c as usize].kind == CardKind::Dizaizhen),
        "本体牌离结算区"
    );

    // 子局②：全魏——无非大者，出牌被拒（上游 enable 同口径）
    let mut e = Engine::new_with_extensions(8102, 3, &[&pack]).expect("建局②");
    advance_to_play(&mut e, 0);
    e.debug_assemble_general(0, &wei);
    e.debug_assemble_general(1, &wei);
    e.debug_assemble_general(2, &wei);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    {
        let mut snap = e.save();
        for p in snap.core.players.iter_mut() {
            p.hand.retain(|c| *c != dz);
        }
        snap.core.deck.retain(|c| *c != dz);
        snap.core.discard.retain(|c| *c != dz);
        snap.core.table.retain(|c| *c != dz);
        snap.core.players[0].hand.insert(0, dz);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("地载阵注入②");
    }
    let out = play(&mut e, dz, vec![]);
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::IllegalTargets)),
        "无非大者应拒: {out:?}"
    );
}

/// ② 真实宿主全生命周期（六步，mtg 首切片模板）。
#[test]
fn zf_real_host_full_lifecycle() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "shahai-zhenfa")
        .expect("zhenfa 安装")
        .clone();
    let slot = format!("{}/{}", meta.namespace, meta.package_id);
    assert!(store.read_active().unwrap().packages.contains_key(&slot));

    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-zhenfa-upstream-noname")
        .expect("zhenfa 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);
    let base_cards = Engine::new(1, 3).catalog().cards.len();
    let mut e = Engine::new_with_extensions(8103, 3, &[&pack]).expect("建局");
    assert_eq!(e.catalog().cards.len(), base_cards + 3, "zhenfa 卡在目录");
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let mid = e.save();

    store
        .revoke_slot(&meta.namespace, "shahai-zhenfa", "验收：对局中卸载", true)
        .expect("撤销");
    assert!(store.read_revoke().unwrap().is_slot_revoked(&meta.namespace, "shahai-zhenfa"));
    assert!(!store.read_active().unwrap().packages.contains_key(&slot));

    let mut resumed = Engine::restore_with_extensions(
        &decode_archive(&encode_archive(&mid)).expect("解码"),
        &[&pack],
    )
    .expect("对局中卸载后恢复");
    assert_eq!(resumed.catalog().cards.len(), base_cards + 3, "锁定副本仍可用");
    let out = submit(&mut resumed, 0, GameAction::NextPhase);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));

    let bytes = std::fs::read(disc.root.join("package.npack.json")).expect("npack");
    let err = store.install(&bytes, None, &options());
    assert!(matches!(err, Err(content_runtime::RuntimeError::Revoked(_))));
    let report2 = import_extensions(&store, &root, &options());
    assert!(report2.is_err(), "撤销后重新 import 应被拒");
}
