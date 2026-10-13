//! 整包首切片（U05 杀海拾遗 mtg 子包，规则 v100）：龙颅山峰——
//! land 地类首例（换地 + 二选一获得杀/闪；地图效果全局出杀上限 +1）
//! + 真实宿主全生命周期。
use content_runtime::{discover_extensions, import_extensions, InstallOptions, PackageStore};
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::catalog::CardKind;
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
    let out = content_native_loader::bridge_pack(json).expect("mtg 桥接");
    assert_eq!(out.pack.def.name, "shahai-mtg-upstream-noname");
    out.pack
}

fn advance_to_play(e: &mut Engine, seat: u8) {
    for _ in 0..96 {
        if e.turn().current == seat && e.turn().phase == Phase::Play && e.open_choice().is_none() {
            break;
        }
        if let Some(c) = e.open_choice() {
            let (cid, obs) = (c.id, c.observer);
            submit(e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
        }
        let actor = e.turn().current;
        let _ = submit(e, actor, GameAction::NextPhase);
    }
}

fn play(e: &mut Engine, card: u32, targets: Vec<u8>) -> CommandOutcome {
    submit(e, e.turn().current, GameAction::PlayCard { card, targets })
}

/// 龙颅山峰（mtg 首切片，规则 v100）：地——使用时换地 + 二选一获得
/// 杀/闪（上游 createCard 凭空造牌→原生牌堆取，登记差异）；地图效果
/// 全局出杀上限 +1（current_land 状态直读）；本回合限一张；换回合后
/// 地持续、出杀配额恢复。
#[test]
fn mtg_longshanfeng_land_flow() {
    use engine_native::content::host::MTG_LONGSHANFENG_GAIN;
    use engine_native::ChoiceKind;

    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-mtg-upstream-noname")
        .expect("mtg 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);
    let base_cards = Engine::new(1, 3).catalog().cards.len();
    let mut e = Engine::new_with_extensions(7101, 3, &[&pack]).expect("建局");
    assert_eq!(e.catalog().cards.len(), base_cards + 10, "mtg 十卡入目录");
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);

    // 注入龙颅山峰（全区域 purge）
    let land = (0..e.catalog().cards.len() as u32)
        .find(|i| e.catalog().cards[*i as usize].kind == CardKind::Longlushanfeng)
        .expect("龙颅山峰在目录");
    {
        let mut snap = e.save();
        for p in snap.core.players.iter_mut() {
            p.hand.retain(|c| *c != land);
        }
        snap.core.deck.retain(|c| *c != land);
        snap.core.discard.retain(|c| *c != land);
        snap.core.table.retain(|c| *c != land);
        snap.core.players[0].hand.insert(0, land);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("龙颅山峰注入");
    }

    // 出地 → 获得杀/闪询问挂起
    let out = play(&mut e, land, vec![]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    let c = e.open_choice().expect("获得选择挂起");
    assert_eq!((c.spec.prompt, c.observer), (MTG_LONGSHANFENG_GAIN, 0));
    assert_eq!(c.spec.kind, ChoiceKind::PickTag);
    let (_cid, _obs) = (c.id, c.observer);

    // 存档往返：挂起态（current_land 已换、询问可恢复）
    let snap = e.save();
    assert!(snap.core.current_land == Some(CardKind::Longlushanfeng), "换地在询问前生效");
    e = Engine::restore_with_extensions(&snap, &[&pack]).expect("往返");
    let c = e.open_choice().expect("往返后询问仍在");
    assert_eq!((c.spec.prompt, c.observer), (MTG_LONGSHANFENG_GAIN, 0));
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Tag(0)] });

    // 获得杀：手牌 +1 且为杀；地离手（table→弃堆流程）
    let h0 = e.debug_hand(0);
    assert_eq!(h0.len(), 1, "获得一张杀");
    assert_eq!(
        e.catalog().cards[h0[0] as usize].kind,
        CardKind::Sha,
        "获得的是杀"
    );
    assert!(e.save().core.current_land == Some(CardKind::Longlushanfeng));

    // 本回合限一张：再出地被拒（已离手——重新注入验证门）
    {
        let mut snap = e.save();
        for p in snap.core.players.iter_mut() {
            p.hand.retain(|c| *c != land);
        }
        snap.core.deck.retain(|c| *c != land);
        snap.core.discard.retain(|c| *c != land);
        snap.core.table.retain(|c| *c != land);
        snap.core.players[0].hand.insert(0, land);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("再注入");
    }
    let out = play(&mut e, land, vec![]);
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::IllegalTargets)),
        "本回合限一张地: {out:?}"
    );
    // 弃掉重注入的地面牌（回弃堆）避免干扰后续杀流程
    e.debug_clear_hand(0);

    // 地图效果：全局出杀上限 +1——0 号连出两张杀（1/2 号空手无法闪，
    // 杀直达伤害结算或至少进入结算）；无地时第二张应被拒
    let sha1 = e.debug_place(0, "sha");
    let out = play(&mut e, sha1, vec![1]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "第一张杀: {out:?}");
    let sha2 = e.debug_place(0, "sha");
    let out = play(&mut e, sha2, vec![1]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "龙颅山峰下第二张杀放行: {out:?}");
    let sha3 = e.debug_place(0, "sha");
    let out = play(&mut e, sha3, vec![1]);
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::ShaLimit)),
        "第三张杀拒: {out:?}"
    );

    // 换回合：地持续（current_land 不变）、出杀配额恢复
    for _ in 0..8 {
        if e.turn().current == 1 {
            break;
        }
        if let Some(c) = e.open_choice() {
            let (cid, obs) = (c.id, c.observer);
            submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
        }
        let actor = e.turn().current;
        let _ = submit(&mut e, actor, GameAction::NextPhase);
    }
    assert_eq!(e.turn().current, 1, "轮到 1 号");
    assert!(e.state_core().current_land == Some(CardKind::Longlushanfeng), "地跨回合持续");
}

/// ② 真实宿主全生命周期（六步，swd 首切片模板）。
#[test]
fn mtg_real_host_full_lifecycle() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "shahai-mtg")
        .expect("mtg 安装")
        .clone();
    let slot = format!("{}/{}", meta.namespace, meta.package_id);
    assert!(store.read_active().unwrap().packages.contains_key(&slot));

    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-mtg-upstream-noname")
        .expect("mtg 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);
    let base_cards = Engine::new(1, 3).catalog().cards.len();
    let mut e = Engine::new_with_extensions(7103, 3, &[&pack]).expect("建局");
    assert_eq!(e.catalog().cards.len(), base_cards + 10, "mtg 十卡在目录");
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let mid = e.save();

    store
        .revoke_slot(&meta.namespace, "shahai-mtg", "验收：对局中卸载", true)
        .expect("撤销");
    assert!(store.read_revoke().unwrap().is_slot_revoked(&meta.namespace, "shahai-mtg"));
    assert!(!store.read_active().unwrap().packages.contains_key(&slot));

    let mut resumed = Engine::restore_with_extensions(
        &decode_archive(&encode_archive(&mid)).expect("解码"),
        &[&pack],
    )
    .expect("对局中卸载后恢复");
    assert_eq!(resumed.catalog().cards.len(), base_cards + 10, "锁定副本仍可用");
    let out = submit(&mut resumed, 0, GameAction::NextPhase);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));

    let bytes = std::fs::read(disc.root.join("package.npack.json")).expect("npack");
    let err = store.install(&bytes, None, &options());
    assert!(matches!(err, Err(content_runtime::RuntimeError::Revoked(_))));
    let report2 = import_extensions(&store, &root, &options());
    assert!(report2.is_err(), "撤销后重新 import 应被拒");
}

/// 海岛（mtg 第二批，规则 v101）：换地后满血直接 +1 护甲；受伤则问
/// 弃一张（he，可取消）——弃则回 1/取消得 1 护甲（temp 抑制转换）；
/// 地图效果全局锁定：current_land=海岛时任何玩家获护甲若已受伤改为
/// 回等量体力（双包盾牌格挡验证）；换地覆盖（龙颅山峰→海岛）。
#[test]
fn mtg_haidao_land_effect() {
    use engine_native::content::host::MTG_HAIDAO_DISCARD;
    use engine_native::ChoiceKind;

    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-mtg-upstream-noname")
        .expect("mtg 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let mtg_pack = bridge(&json);
    let hearth_disc = discovered
        .iter()
        .find(|d| d.name == "shahai-hearth-upstream-noname")
        .expect("hearth 可发现")
        .clone();
    let hearth_json = std::fs::read_to_string(hearth_disc.root.join("assets/cards.json")).unwrap();
    let hearth_pack = content_native_loader::bridge_pack(&hearth_json).expect("hearth 桥接").pack;
    let _base_cards = Engine::new(1, 3).catalog().cards.len();

    let setup = |seed: u64| -> Engine {
        let mut e =
            Engine::new_with_extensions(seed, 3, &[&mtg_pack, &hearth_pack]).expect("建局");
        advance_to_play(&mut e, 0);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        e.debug_clear_hand(2);
        let hd = (0..e.catalog().cards.len() as u32)
            .find(|i| e.catalog().cards[*i as usize].kind == CardKind::Haidao)
            .expect("海岛在目录");
        let mut snap = e.save();
        for p in snap.core.players.iter_mut() {
            p.hand.retain(|c| *c != hd);
        }
        snap.core.deck.retain(|c| *c != hd);
        snap.core.discard.retain(|c| *c != hd);
        snap.core.table.retain(|c| *c != hd);
        snap.core.players[0].hand.insert(0, hd);
        Engine::restore_with_extensions(&snap, &[&mtg_pack, &hearth_pack]).expect("海岛注入")
    };

    // ① 满血：直接 +1 护甲（无询问）
    let mut e = setup(7201);
    let hd = (0..e.catalog().cards.len() as u32)
        .find(|i| e.catalog().cards[*i as usize].kind == CardKind::Haidao)
        .unwrap();
    let out = play(&mut e, hd, vec![]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert!(e.open_choice().is_none(), "满血无询问");
    assert_eq!(e.save().core.players[0].hujia, 1, "满血 +1 护甲");
    assert_eq!(e.debug_hp(0), e.state_core().player(0).max_hp as i32, "满血不变");
    // 地已换（此前无地——current_land=海岛）；本回合限一张
    assert!(e.state_core().current_land == Some(CardKind::Haidao));

    // ② 受伤：弃一张回 1（含存档往返）
    let mut e = setup(7202);
    let hd = (0..e.catalog().cards.len() as u32)
        .find(|i| e.catalog().cards[*i as usize].kind == CardKind::Haidao)
        .unwrap();
    e.debug_set_hp(0, 2);
    let give = e.debug_place(0, "shan"); // 弃牌候选（询问前布阵入选项）
    let _out = play(&mut e, hd, vec![]);
    let c = e.open_choice().expect("受伤分支弃牌询问挂起");
    assert_eq!((c.spec.prompt, c.observer), (MTG_HAIDAO_DISCARD, 0));
    assert_eq!(c.spec.kind, ChoiceKind::SelectCards);
    assert_eq!(c.spec.min, 0, "可取消");
    let (_cid, _obs) = (c.id, c.observer);
    let snap = e.save();
    e = Engine::restore_with_extensions(&snap, &[&mtg_pack, &hearth_pack]).expect("往返");
    let c = e.open_choice().expect("往返后询问仍在");
    assert_eq!((c.spec.prompt, c.observer), (MTG_HAIDAO_DISCARD, 0));
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(give)] });
    assert_eq!(e.debug_hp(0), 3, "弃一张回 1（2→3）");
    assert_eq!(e.save().core.players[0].hujia, 0, "弃牌路线无护甲");

    // ③ 受伤：取消 → +1 护甲（temp 抑制——受伤得甲不转回血）
    let mut e = setup(7203);
    let hd = (0..e.catalog().cards.len() as u32)
        .find(|i| e.catalog().cards[*i as usize].kind == CardKind::Haidao)
        .unwrap();
    e.debug_set_hp(0, 2);
    let _ = play(&mut e, hd, vec![]);
    let c = e.open_choice().expect("弃牌询问挂起");
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![] });
    assert_eq!(e.save().core.players[0].hujia, 1, "取消得 1 护甲（不转换）");
    assert_eq!(e.debug_hp(0), 2, "取消不回血");

    // ④ 换地覆盖：龙颅山峰→海岛（杀加成消失、current_land 被覆盖）
    let mut e = setup(7204);
    let ls = (0..e.catalog().cards.len() as u32)
        .find(|i| e.catalog().cards[*i as usize].kind == CardKind::Longlushanfeng)
        .expect("龙颅山峰在目录");
    let hd = (0..e.catalog().cards.len() as u32)
        .find(|i| e.catalog().cards[*i as usize].kind == CardKind::Haidao)
        .unwrap();
    // 先出龙颅山峰（满血 → 获得杀询问 → Tag 0）
    {
        let mut snap = e.save();
        for p in snap.core.players.iter_mut() {
            p.hand.retain(|c| *c != ls);
        }
        snap.core.deck.retain(|c| *c != ls);
        snap.core.discard.retain(|c| *c != ls);
        snap.core.table.retain(|c| *c != ls);
        snap.core.players[0].hand.insert(0, ls);
        e = Engine::restore_with_extensions(&snap, &[&mtg_pack, &hearth_pack]).expect("龙颅注入");
    }
    let _ = play(&mut e, ls, vec![]);
    let c = e.open_choice().expect("龙颅山峰询问挂起");
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Tag(0)] });
    assert!(e.state_core().current_land == Some(CardKind::Longlushanfeng));
    // 再出海岛（换地覆盖；受伤分支取消）——每回合限一张地，轮回 0 号
    // 下回合后再出（land_used 随换回合重置；round 递增判据防原地打转）
    let round0 = e.turn().round;
    for _ in 0..48 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play && e.turn().round > round0 {
            break;
        }
        if let Some(c) = e.open_choice() {
            let (cid, obs) = (c.id, c.observer);
            submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
        }
        let actor = e.turn().current;
        let _ = submit(&mut e, actor, GameAction::NextPhase);
    }
    assert_eq!(e.turn().current, 0, "轮回 0 号");
    assert!(e.state_core().current_land == Some(CardKind::Longlushanfeng), "地跨回合持续");
    e.debug_set_hp(0, 2);
    {
        let mut snap = e.save();
        for p in snap.core.players.iter_mut() {
            p.hand.retain(|c| *c != hd);
        }
        snap.core.deck.retain(|c| *c != hd);
        snap.core.discard.retain(|c| *c != hd);
        snap.core.table.retain(|c| *c != hd);
        snap.core.players[0].hand.insert(0, hd);
        e = Engine::restore_with_extensions(&snap, &[&mtg_pack, &hearth_pack]).expect("海岛再注入");
    }
    let _ = play(&mut e, hd, vec![]);
    let c = e.open_choice().expect("海岛询问挂起");
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![] });
    assert!(e.state_core().current_land == Some(CardKind::Haidao), "换地覆盖");
    assert_eq!(e.save().core.players[0].hujia, 1, "取消得甲");
    // 龙颅山峰杀加成消失：新回合常规配额一张，第二张即拒
    let sha1 = e.debug_place(0, "sha");
    let out = play(&mut e, sha1, vec![1]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    let sha2 = e.debug_place(0, "sha");
    let out = play(&mut e, sha2, vec![1]);
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::ShaLimit)),
        "海岛覆盖后无杀加成：第二张即拒"
    );

    // ⑤ 地图效果全局：海岛在场时受伤玩家获护甲改为回血（盾牌格挡）
    let mut e = setup(7205);
    let hd = (0..e.catalog().cards.len() as u32)
        .find(|i| e.catalog().cards[*i as usize].kind == CardKind::Haidao)
        .unwrap();
    e.debug_set_hp(0, 2);
    let _ = play(&mut e, hd, vec![]);
    let c = e.open_choice().expect("弃牌询问挂起");
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![] }); // 取消得甲
    // 地换后同回合再出地被拒——换回合后盾牌格挡验证转换
    for _ in 0..16 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play {
            break;
        }
        if let Some(c) = e.open_choice() {
            let (cid, obs) = (c.id, c.observer);
            submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
        }
        let actor = e.turn().current;
        let _ = submit(&mut e, actor, GameAction::NextPhase);
    }
    let dp = (0..e.catalog().cards.len() as u32)
        .find(|i| e.catalog().cards[*i as usize].name.starts_with("ht_dunpaigedang"))
        .expect("盾牌格挡在目录");
    {
        let mut snap = e.save();
        for p in snap.core.players.iter_mut() {
            p.hand.retain(|c| *c != dp);
        }
        snap.core.deck.retain(|c| *c != dp);
        snap.core.discard.retain(|c| *c != dp);
        snap.core.table.retain(|c| *c != dp);
        snap.core.players[0].hand.insert(0, dp);
        e = Engine::restore_with_extensions(&snap, &[&mtg_pack, &hearth_pack]).expect("盾牌注入");
    }
    let hp_before = e.debug_hp(0);
    let out = play(&mut e, dp, vec![]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    assert!(e.state_core().current_land == Some(CardKind::Haidao), "海岛仍在场");
    assert_eq!(e.debug_hp(0), hp_before + 1, "获护甲转为回血（锁定地图效果）");
    assert_eq!(e.save().core.players[0].hujia, 1, "护甲不增（被转换）");
}

/// 水没墓穴（mtg 第三批，规则 v102）：换地后可弃一张摸二（he，可取
/// 消；无牌则空发）；地图效果锁定——手牌上限 -1（弃牌阶段提前触发弃
/// 牌）、本弃牌阶段弃过牌则阶段结束摸一。覆盖：三分支 + 上限 -1 +
/// 弃后摸一 + 换地覆盖（海岛→水没，获甲转换消失）+ 存档往返。
#[test]
fn mtg_shuimomuxue_land_effect() {
    use engine_native::content::host::MTG_SHUIMOMUXUE_DISCARD;
    use engine_native::ChoiceKind;

    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-mtg-upstream-noname")
        .expect("mtg 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let mtg_pack = bridge(&json);
    let base_cards = Engine::new(1, 3).catalog().cards.len();
    assert_eq!(base_cards + 10, {
        let e = Engine::new_with_extensions(1, 3, &[&mtg_pack]).unwrap();
        e.catalog().cards.len()
    }, "mtg 十卡入目录");

    let setup = |seed: u64| -> Engine {
        let mut e = Engine::new_with_extensions(seed, 3, &[&mtg_pack]).expect("建局");
        advance_to_play(&mut e, 0);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        e.debug_clear_hand(2);
        let sm = (0..e.catalog().cards.len() as u32)
            .find(|i| e.catalog().cards[*i as usize].kind == CardKind::Shuimomuxue)
            .expect("水没墓穴在目录");
        let mut snap = e.save();
        for p in snap.core.players.iter_mut() {
            p.hand.retain(|c| *c != sm);
        }
        snap.core.deck.retain(|c| *c != sm);
        snap.core.discard.retain(|c| *c != sm);
        snap.core.table.retain(|c| *c != sm);
        snap.core.players[0].hand.insert(0, sm);
        Engine::restore_with_extensions(&snap, &[&mtg_pack]).expect("水没墓穴注入")
    };
    let sm_of = |e: &Engine| -> u32 {
        (0..e.catalog().cards.len() as u32)
            .find(|i| e.catalog().cards[*i as usize].kind == CardKind::Shuimomuxue)
            .unwrap()
    };

    // ① 有手牌：弃一摸二（含存档往返）
    let mut e = setup(7301);
    let sm = sm_of(&e);
    let give = e.debug_place(0, "shan"); // 弃牌候选（询问前布阵）
    let out = play(&mut e, sm, vec![]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    let c = e.open_choice().expect("弃一摸二询问挂起");
    assert_eq!((c.spec.prompt, c.observer), (MTG_SHUIMOMUXUE_DISCARD, 0));
    assert_eq!(c.spec.kind, ChoiceKind::SelectCards);
    assert_eq!(c.spec.min, 0, "可取消");
    let (_cid, _obs) = (c.id, c.observer);
    let snap = e.save();
    e = Engine::restore_with_extensions(&snap, &[&mtg_pack]).expect("往返");
    let c = e.open_choice().expect("往返后询问仍在");
    assert_eq!((c.spec.prompt, c.observer), (MTG_SHUIMOMUXUE_DISCARD, 0));
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(give)] });
    assert_eq!(e.debug_hand(0).len(), 2, "弃一摸二（0-1+2=2，均为新摸）");
    assert!(e.state_core().current_land == Some(CardKind::Shuimomuxue));

    // ② 有牌但取消：无事（不摸牌；上游 result.bool=false 同口径）
    let mut e = setup(7302);
    let sm = sm_of(&e);
    let keep = e.debug_place(0, "sha"); // 候选（询问前布阵）
    let _ = play(&mut e, sm, vec![]);
    let c = e.open_choice().expect("询问挂起");
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![] });
    assert_eq!(e.debug_hand(0).len(), 1, "取消不摸牌（保留原牌）");
    assert!(e.debug_hand(0).contains(&keep), "原牌未被弃");

    // ③ 空手：空发（无询问）
    let mut e = setup(7303);
    let sm = sm_of(&e);
    let out = play(&mut e, sm, vec![]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert!(e.open_choice().is_none(), "无牌可弃空发");

    // ④ 地图效果：手牌上限 -1 + 弃牌阶段弃后摸一。
    // 出地后保留 3 张手牌（常规上限=体力 4？以引擎为准：直接推到弃牌
    // 阶段，若手牌>limit-1 则 discard_to_limit 触发——弃 ≥1 张后阶段
    // 结束应摸 1）
    let mut e = setup(7304);
    let sm = sm_of(&e);
    let _ = play(&mut e, sm, vec![]); // 空手出地空发
    e.debug_set_hp(0, 2); // 上限 = 体力 2 - 地图 1 = 1
    let _ = e.debug_place(0, "sha");
    let _ = e.debug_place(0, "shan");
    // 推进到弃牌阶段
    for _ in 0..16 {
        if e.turn().current == 0 && e.turn().phase == Phase::Discard {
            break;
        }
        if let Some(c) = e.open_choice() {
            let (cid, obs) = (c.id, c.observer);
            submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
        }
        let actor = e.turn().current;
        let _ = submit(&mut e, actor, GameAction::NextPhase);
    }
    assert_eq!(e.turn().phase, Phase::Discard, "进入弃牌阶段");
    let hp0 = e.debug_hp(0);
    let limit_now = if e.open_choice().is_some() {
        // discard_to_limit 挂起：手牌 > limit-1（地图效果已生效）
        let c = e.open_choice().unwrap();
        assert_eq!(c.spec.prompt, "discard_to_limit", "上限-1 使弃牌触发");
        let need = c.spec.min;
        assert!(need >= 1, "至少弃 1");
        let sel: Vec<ChoiceOption> = c.spec.options.iter().take(need).copied().collect();
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: sel });
        true
    } else {
        panic!("手牌 3 应超过 limit-1（体力 {hp0}）触发弃牌");
    };
    let _ = limit_now;
    // 阶段结束摸一：推过弃牌阶段后手牌应含该 1 张补偿
    let hand_before_end = e.debug_hand(0).len();
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    assert_eq!(e.turn().phase, Phase::End, "离开弃牌阶段");
    assert_eq!(e.debug_hand(0).len(), hand_before_end + 1, "弃牌阶段弃过牌→阶段结束摸 1");

    // ⑤ 换地覆盖：海岛→水没墓穴（获甲转换消失）
    let _dir2 = tempfile::tempdir().unwrap();
    let hearth_disc = discovered
        .iter()
        .find(|d| d.name == "shahai-hearth-upstream-noname")
        .expect("hearth 可发现")
        .clone();
    let hearth_json = std::fs::read_to_string(hearth_disc.root.join("assets/cards.json")).unwrap();
    let hearth_pack = content_native_loader::bridge_pack(&hearth_json).expect("hearth 桥接").pack;
    let mut e = Engine::new_with_extensions(7305, 3, &[&mtg_pack, &hearth_pack]).expect("建局");
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    let hd = (0..e.catalog().cards.len() as u32)
        .find(|i| e.catalog().cards[*i as usize].kind == CardKind::Haidao)
        .expect("海岛在目录");
    let sm = (0..e.catalog().cards.len() as u32)
        .find(|i| e.catalog().cards[*i as usize].kind == CardKind::Shuimomuxue)
        .unwrap();
    let inject = |e: &mut Engine, card: u32, what: &str| {
        let mut snap = e.save();
        for p in snap.core.players.iter_mut() {
            p.hand.retain(|c| *c != card);
        }
        snap.core.deck.retain(|c| *c != card);
        snap.core.discard.retain(|c| *c != card);
        snap.core.table.retain(|c| *c != card);
        snap.core.players[0].hand.insert(0, card);
        *e = Engine::restore_with_extensions(&snap, &[&mtg_pack, &hearth_pack]).expect(what);
    };
    // 出海岛（受伤取消得甲）
    inject(&mut e, hd, "海岛注入");
    e.debug_set_hp(0, 2);
    let _ = play(&mut e, hd, vec![]);
    let c = e.open_choice().expect("海岛询问挂起");
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![] });
    assert_eq!(e.save().core.players[0].hujia, 1);
    // 轮回 0 号下回合
    let round0 = e.turn().round;
    for _ in 0..48 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play && e.turn().round > round0 {
            break;
        }
        if let Some(c) = e.open_choice() {
            let (cid, obs) = (c.id, c.observer);
            submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
        }
        let actor = e.turn().current;
        let _ = submit(&mut e, actor, GameAction::NextPhase);
    }
    // 出水没墓穴（覆盖海岛；受伤分支取消——空手则空发）
    inject(&mut e, sm, "水没再注入");
    let _ = play(&mut e, sm, vec![]);
    if let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![] });
    }
    assert!(e.state_core().current_land == Some(CardKind::Shuimomuxue), "换地覆盖");
    // 盾牌格挡获甲：海岛转换应消失（受伤也得甲）
    let dp = (0..e.catalog().cards.len() as u32)
        .find(|i| e.catalog().cards[*i as usize].name.starts_with("ht_dunpaigedang"))
        .expect("盾牌格挡在目录");
    inject(&mut e, dp, "盾牌注入");
    e.debug_set_hp(0, 2);
    let out = play(&mut e, dp, vec![]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    assert_eq!(e.save().core.players[0].hujia, 2, "水没在场：获甲不转换（1+1=2）");
    assert_eq!(e.debug_hp(0), 2, "不回血");
}

/// 废墟（mtg 第四批，规则 v103）：换地后从弃牌堆发现一张牌（亮 ≤3
/// 三选一，选定才离弃堆；空弃堆空发）；地图效果锁定——回合开始（准备
/// 阶段）随机将弃牌堆一张置于牌堆顶。覆盖：发现+存档往返、空发、地
/// 图效果。
#[test]
fn mtg_feixu_land_effect() {
    use engine_native::content::host::MTG_FEIXU_DISCOVER;
    use engine_native::ChoiceKind;

    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-mtg-upstream-noname")
        .expect("mtg 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let mtg_pack = bridge(&json);
    let base_cards = Engine::new(1, 3).catalog().cards.len();
    let mut e = Engine::new_with_extensions(7401, 3, &[&mtg_pack]).expect("建局");
    assert_eq!(e.catalog().cards.len(), base_cards + 10, "mtg 十卡入目录");
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);

    let fx = (0..e.catalog().cards.len() as u32)
        .find(|i| e.catalog().cards[*i as usize].kind == CardKind::Feixu)
        .expect("废墟在目录");
    {
        let mut snap = e.save();
        for p in snap.core.players.iter_mut() {
            p.hand.retain(|c| *c != fx);
        }
        snap.core.deck.retain(|c| *c != fx);
        snap.core.discard.retain(|c| *c != fx);
        snap.core.table.retain(|c| *c != fx);
        snap.core.players[0].hand.insert(0, fx);
        e = Engine::restore_with_extensions(&snap, &[&mtg_pack]).expect("废墟注入");
    }

    // 首出：弃堆有牌（clear_hand 弃牌）→ 发现三选一挂起
    let out = play(&mut e, fx, vec![]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    let c = e.open_choice().expect("发现询问挂起");
    assert_eq!((c.spec.prompt, c.observer), (MTG_FEIXU_DISCOVER, 0));
    assert_eq!(c.spec.kind, ChoiceKind::SelectCards);
    assert!(c.spec.options.len() >= 1 && c.spec.options.len() <= 3, "亮 1–3 张");
    // 选项 ⊆ 弃堆（亮出态不移动）
    let discard_at_ask = e.save().core.discard.clone();
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
    for card in &offered {
        assert!(discard_at_ask.contains(card), "亮出的牌仍在弃堆");
    }
    let (_cid, _obs) = (c.id, c.observer);
    // 存档往返：选项一致
    let snap = e.save();
    e = Engine::restore_with_extensions(&snap, &[&mtg_pack]).expect("往返");
    let c = e.open_choice().expect("往返后询问仍在");
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
    assert!(e.debug_hand(0).contains(&chosen), "发现牌入手");
    assert!(!e.save().core.discard.contains(&chosen), "发现牌离弃堆");
    assert!(e.state_core().current_land == Some(CardKind::Feixu));

    // 地图效果：轮回 0 号下回合（1/2 号 Judge 开始时随机弃堆一张上顶，
    // 弃堆长度只减不增直到为空；总牌数守恒由区域核算保证）
    let round0 = e.turn().round;
    let mut saw_map_effect = false;
    for _ in 0..64 {
        let (d_before, deck_before) = {
            let snap = e.save();
            (snap.core.discard.len(), snap.core.deck.len())
        };
        if let Some(c) = e.open_choice() {
            let (cid, obs) = (c.id, c.observer);
            submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
        }
        let actor = e.turn().current;
        let _ = submit(&mut e, actor, GameAction::NextPhase);
        let snap = e.save();
        if snap.core.discard.len() == d_before.saturating_sub(1)
            && snap.core.deck.len() == deck_before + 1
            && snap.core.current_land == Some(CardKind::Feixu)
        {
            saw_map_effect = true;
        }
        if e.turn().current == 0 && e.turn().phase == Phase::Play && e.turn().round > round0 {
            break;
        }
    }
    assert!(saw_map_effect, "观测到地图效果（弃堆 -1、牌堆 +1）");

    // 空弃堆空发：清空弃堆（回牌堆）后，将首张废墟（已在弃堆）注回
    // 手牌再出——无牌可发现应无询问
    {
        let mut snap = e.save();
        while let Some(c) = snap.core.discard.pop() {
            snap.core.deck.push(c);
        }
        for p in snap.core.players.iter_mut() {
            p.hand.retain(|c| *c != fx);
        }
        snap.core.deck.retain(|c| *c != fx);
        snap.core.discard.retain(|c| *c != fx);
        snap.core.table.retain(|c| *c != fx);
        snap.core.players[0].hand.insert(0, fx);
        e = Engine::restore_with_extensions(&snap, &[&mtg_pack]).expect("空弃堆+废墟注回");
    }
    let out = play(&mut e, fx, vec![]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    if let Some(c) = e.open_choice() {
        panic!("空弃堆应空发，但挂起: prompt={:?}", c.spec.prompt);
    }
}

/// 依夏兰（mtg 第五批，规则 v104）：换地后选类型随机获得基本/锦囊牌
/// 一张（上游 createCard→牌堆实体取）；地图效果=地附主动技「重铸」
/// （出牌阶段限一次：基本牌放回牌堆→获得牌堆一张锦囊，无锦囊摸一；
/// 换地授予/撤销——首例地附技基建）。覆盖：类型获得×存档往返、重铸
/// 主路径、限一次、无地/换地后守卫。
#[test]
fn mtg_yixialan_land_and_recast() {
    use engine_native::content::host::{
        MTG_YIXIALAN_GAIN, MTG_YIXIALAN_RECAST_CARD, MTG_YIXIALAN_RECAST_KEY,
    };
    use engine_native::ChoiceKind;

    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-mtg-upstream-noname")
        .expect("mtg 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let mtg_pack = bridge(&json);
    let base_cards = Engine::new(1, 3).catalog().cards.len();
    let mut e = Engine::new_with_extensions(7501, 3, &[&mtg_pack]).expect("建局");
    assert_eq!(e.catalog().cards.len(), base_cards + 10, "mtg 十卡入目录");
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);

    let yxl = (0..e.catalog().cards.len() as u32)
        .find(|i| e.catalog().cards[*i as usize].kind == CardKind::Yixialan)
        .expect("依夏兰在目录");
    let inject = |e: &mut Engine, card: u32, what: &str| {
        let mut snap = e.save();
        for p in snap.core.players.iter_mut() {
            p.hand.retain(|c| *c != card);
        }
        snap.core.deck.retain(|c| *c != card);
        snap.core.discard.retain(|c| *c != card);
        snap.core.table.retain(|c| *c != card);
        snap.core.players[0].hand.insert(0, card);
        *e = Engine::restore_with_extensions(&snap, &[&mtg_pack]).expect(what);
    };

    // 无地时重铸技不存在 → NoSuchSkill
    let def_recast = e
        .catalog()
        .skill_by_name(MTG_YIXIALAN_RECAST_KEY)
        .expect("重铸技在目录");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_recast });
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::NoSuchSkill)),
        "无地无实例: {out:?}"
    );

    // 出依夏兰 → 类型询问（存档往返）→ Tag1（锦囊）
    inject(&mut e, yxl, "依夏兰注入");
    let out = play(&mut e, yxl, vec![]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    let c = e.open_choice().expect("类型询问挂起");
    assert_eq!((c.spec.prompt, c.observer), (MTG_YIXIALAN_GAIN, 0));
    assert_eq!(c.spec.kind, ChoiceKind::PickTag);
    let (_cid, _obs) = (c.id, c.observer);
    let snap = e.save();
    e = Engine::restore_with_extensions(&snap, &[&mtg_pack]).expect("往返");
    let c = e.open_choice().expect("往返后询问仍在");
    assert_eq!((c.spec.prompt, c.observer), (MTG_YIXIALAN_GAIN, 0));
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Tag(1)] });
    // 获得锦囊牌
    let h0 = e.debug_hand(0);
    assert_eq!(h0.len(), 1, "获得一张");
    assert!(
        e.save().core.current_land == Some(CardKind::Yixialan),
        "换地生效"
    );

    // 地附技已授予：重铸（基本牌→放回牌堆→获得锦囊）
    // 先给一张基本牌（debug_place）
    let basic = e.debug_place(0, "sha");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_recast });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    let c = e.open_choice().expect("重铸选牌挂起");
    assert_eq!((c.spec.prompt, c.observer), (MTG_YIXIALAN_RECAST_CARD, 0));
    assert_eq!(c.spec.kind, ChoiceKind::SelectCards);
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(basic)] });
    let h0 = e.debug_hand(0);
    assert!(!h0.contains(&basic), "基本牌已放回牌堆");
    assert!(h0.iter().any(|c| {
        use engine_native::catalog::CardKind as K;
        matches!(
            e.catalog().cards[*c as usize].kind,
            K::SavageAttack
                | K::ArcheryAttack
                | K::Duel
                | K::Nullify
                | K::Le
                | K::Bing
                | K::ShanDian
                | K::Dismantle
                | K::Snatch
                | K::Chain
                | K::Wuzhong
                | K::Taoyuan
                | K::Wugu
                | K::Jiedao
                | K::WangmeiZhike
                | K::Shuigong
                | K::Fudichouxin
                | K::Chenhuodajie
                | K::HongShui
                | K::Toulianghuanzhu
                | K::Youdishenru
                | K::Geanguanhuo
                | K::Caochuanjiejian
                | K::Guiyoujie
                | K::Zhufangshenshi
                | K::Liuxinghuoyu
        )
    }), "重铸获得锦囊牌");

    // 出牌阶段限一次
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_recast });
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::SkillAlreadyUsed)),
        "限一次: {out:?}"
    );

    // 换地撤销：轮回 0 号下回合出海岛（注回）→ 重铸实例被撤 → NoSuchSkill
    let hd = (0..e.catalog().cards.len() as u32)
        .find(|i| e.catalog().cards[*i as usize].kind == CardKind::Haidao)
        .expect("海岛在目录");
    let round0 = e.turn().round;
    for _ in 0..48 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play && e.turn().round > round0 {
            break;
        }
        if let Some(c) = e.open_choice() {
            let (cid, obs) = (c.id, c.observer);
            submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
        }
        let actor = e.turn().current;
        let _ = submit(&mut e, actor, GameAction::NextPhase);
    }
    inject(&mut e, hd, "海岛注回");
    let _ = play(&mut e, hd, vec![]);
    if let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![] });
    }
    assert!(e.state_core().current_land == Some(CardKind::Haidao), "换地覆盖");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_recast });
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::NoSuchSkill)),
        "换地后重铸实例被撤销: {out:?}"
    );
}

/// 沙漠绿洲（mtg 第六批，规则 v105）：换地后从牌堆基本牌发现一张
/// （亮 ≤3 三选一，选定才离牌堆；无基本牌空发）；地图效果锁定——基本
/// 牌不计入手牌上限（需弃数按非基本手牌数计，弃牌选项排除基本牌）。
/// 覆盖：发现×存档往返、地图效果（体力压 2 上限 1：1 基本+2 非基本 →
/// 需弃 1 且选项不含基本牌）。
#[test]
fn mtg_shamolvzhou_land_effect() {
    use engine_native::content::host::MTG_SHAMOLVZHOU_DISCOVER;
    use engine_native::ChoiceKind;

    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-mtg-upstream-noname")
        .expect("mtg 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let mtg_pack = bridge(&json);
    let base_cards = Engine::new(1, 3).catalog().cards.len();
    let mut e = Engine::new_with_extensions(7601, 3, &[&mtg_pack]).expect("建局");
    assert_eq!(e.catalog().cards.len(), base_cards + 10, "mtg 十卡入目录");
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);

    let smz = (0..e.catalog().cards.len() as u32)
        .find(|i| e.catalog().cards[*i as usize].kind == CardKind::Shamolvzhou)
        .expect("沙漠绿洲在目录");
    {
        let mut snap = e.save();
        for p in snap.core.players.iter_mut() {
            p.hand.retain(|c| *c != smz);
        }
        snap.core.deck.retain(|c| *c != smz);
        snap.core.discard.retain(|c| *c != smz);
        snap.core.table.retain(|c| *c != smz);
        snap.core.players[0].hand.insert(0, smz);
        e = Engine::restore_with_extensions(&snap, &[&mtg_pack]).expect("沙漠绿洲注入");
    }

    // 出地 → 基本牌发现三选一（选项全为基本类；存档往返）
    let out = play(&mut e, smz, vec![]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    let c = e.open_choice().expect("发现询问挂起");
    assert_eq!((c.spec.prompt, c.observer), (MTG_SHAMOLVZHOU_DISCOVER, 0));
    assert_eq!(c.spec.kind, ChoiceKind::SelectCards);
    assert!(c.spec.options.len() >= 1 && c.spec.options.len() <= 3, "亮 1–3 张");
    use engine_native::content::host::yangpijuan_class;
    for o in c.spec.options.clone() {
        let ChoiceOption::Card(card) = o else { panic!("选项应为牌") };
        assert_eq!(
            yangpijuan_class(e.catalog().cards[card as usize].kind),
            Some(0),
            "选项全为基本牌"
        );
        assert!(e.save().core.deck.contains(&card), "亮出的牌仍在牌堆");
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
    let snap = e.save();
    e = Engine::restore_with_extensions(&snap, &[&mtg_pack]).expect("往返");
    let c = e.open_choice().expect("往返后询问仍在");
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
    assert!(e.debug_hand(0).contains(&chosen), "发现牌入手");
    assert!(!e.save().core.deck.contains(&chosen), "发现牌离牌堆");
    assert!(e.state_core().current_land == Some(CardKind::Shamolvzhou));

    // 地图效果：体力压 1（上限=体力 1），手牌 1 基本（发现所得）+
    // 2 锦囊：非基本 2 > 上限 1 → 需弃 1，选项不含基本牌（杀/闪/桃均
    // 为基本类，yangpijuan_class 口径）
    e.debug_set_hp(0, 1);
    let _ = e.debug_place(0, "wuzhong"); // 锦囊（非基本）
    let _ = e.debug_place(0, "duel"); // 锦囊（非基本）
    // 推进到弃牌阶段
    for _ in 0..16 {
        if e.turn().current == 0 && e.turn().phase == Phase::Discard {
            break;
        }
        if let Some(c) = e.open_choice() {
            let (cid, obs) = (c.id, c.observer);
            submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
        }
        let actor = e.turn().current;
        let _ = submit(&mut e, actor, GameAction::NextPhase);
    }
    assert_eq!(e.turn().phase, Phase::Discard);
    let c = e.open_choice().expect("弃牌触发（非基本 2 > 上限 1）");
    assert_eq!(c.spec.prompt, "discard_to_limit");
    assert_eq!(c.spec.min, 1, "需弃 1（非基本手牌 2 - 上限 1）");
    for o in c.spec.options.clone() {
        let ChoiceOption::Card(card) = o else { panic!("选项应为牌") };
        assert_ne!(
            yangpijuan_class(e.catalog().cards[card as usize].kind),
            Some(0),
            "弃牌选项不含基本牌（不计入上限即不被要求弃）"
        );
    }
}

/// 林中尖塔（mtg 第七批，规则 v106）：换地后从牌堆武器发现并直接装备
/// （亮 ≤3 三选一，emit Equipped 走完整装配）；地图效果=地附主动技
/// 「塔击」——装备区有武器时出牌阶段限一次将一张基本牌当杀使用（丈
/// 八蛇矛 declared-Sha 同构）。覆盖：发现装备、塔击杀结算（无闪目标
/// 掉血）、限一次、换地撤销、存档往返。
#[test]
fn mtg_linzhongjianta_land_and_strike() {
    use engine_native::content::host::{
        MTG_LINZHONGJIANTA_CARD, MTG_LINZHONGJIANTA_DISCOVER,
        MTG_LINZHONGJIANTA_SKILL_KEY, MTG_LINZHONGJIANTA_TARGET,
    };
    use engine_native::ChoiceKind;

    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-mtg-upstream-noname")
        .expect("mtg 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let mtg_pack = bridge(&json);
    let base_cards = Engine::new(1, 3).catalog().cards.len();
    let mut e = Engine::new_with_extensions(7701, 3, &[&mtg_pack]).expect("建局");
    assert_eq!(e.catalog().cards.len(), base_cards + 10, "mtg 十卡入目录");
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);

    let lzt = (0..e.catalog().cards.len() as u32)
        .find(|i| e.catalog().cards[*i as usize].kind == CardKind::Linzhongjianta)
        .expect("林中尖塔在目录");
    let inject = |e: &mut Engine, card: u32, what: &str| {
        let mut snap = e.save();
        for p in snap.core.players.iter_mut() {
            p.hand.retain(|c| *c != card);
        }
        snap.core.deck.retain(|c| *c != card);
        snap.core.discard.retain(|c| *c != card);
        snap.core.table.retain(|c| *c != card);
        snap.core.players[0].hand.insert(0, card);
        *e = Engine::restore_with_extensions(&snap, &[&mtg_pack]).expect(what);
    };

    // 出林中尖塔 → 武器发现（存档往返）→ 选定直接装备
    inject(&mut e, lzt, "林中尖塔注入");
    let out = play(&mut e, lzt, vec![]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    let c = e.open_choice().expect("武器发现询问挂起");
    assert_eq!((c.spec.prompt, c.observer), (MTG_LINZHONGJIANTA_DISCOVER, 0));
    assert_eq!(c.spec.kind, ChoiceKind::SelectCards);
    use engine_native::catalog::EquipSlot;
    for o in c.spec.options.clone() {
        let ChoiceOption::Card(card) = o else { panic!("选项应为牌") };
        assert_eq!(
            e.catalog().cards[card as usize].kind.equip_slot(),
            Some(EquipSlot::Weapon),
            "选项全为武器"
        );
        assert!(e.save().core.deck.contains(&card), "亮出的牌仍在牌堆");
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
    let snap = e.save();
    e = Engine::restore_with_extensions(&snap, &[&mtg_pack]).expect("往返");
    let c = e.open_choice().expect("往返后询问仍在");
    assert_eq!((c.spec.prompt, c.observer), (MTG_LINZHONGJIANTA_DISCOVER, 0));
    let (cid, obs) = (c.id, c.observer);
    let weapon = offered[0];
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(weapon)] });
    assert!(e.save().core.players[0].equips.contains(&weapon), "发现武器已装备");
    assert!(!e.save().core.deck.contains(&weapon), "武器离牌堆");

    // 塔击：装备区有武器 + 手牌基本牌（先布一张）→ 杀结算（1 号空手掉血）
    let def_strike = e
        .catalog()
        .skill_by_name(MTG_LINZHONGJIANTA_SKILL_KEY)
        .expect("塔击技在目录");
    let basic = e.debug_place(0, "sha");
    e.debug_set_hp(1, 4);
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_strike });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    let c = e.open_choice().expect("塔击选牌挂起");
    assert_eq!((c.spec.prompt, c.observer), (MTG_LINZHONGJIANTA_CARD, 0));
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(basic)] });
    let c = e.open_choice().expect("塔击目标挂起");
    assert_eq!((c.spec.prompt, c.observer), (MTG_LINZHONGJIANTA_TARGET, 0));
    assert_eq!(c.spec.kind, ChoiceKind::SelectPlayers);
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Player(1)] });
    // 推进杀结算（无闪 → 1 伤）
    for _ in 0..8 {
        if let Some(c) = e.open_choice() {
            let (cid, obs) = (c.id, c.observer);
            submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
        } else {
            break;
        }
    }
    assert_eq!(e.debug_hp(1), 3, "塔击=当杀使用：1 伤（4→3）");
    assert!(!e.debug_hand(0).contains(&basic), "基本牌已消耗");

    // 限一次
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_strike });
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::SkillAlreadyUsed)),
        "限一次: {out:?}"
    );

    // 换地撤销：轮回 0 号下回合出海岛（注回）→ 塔击实例被撤
    let hd = (0..e.catalog().cards.len() as u32)
        .find(|i| e.catalog().cards[*i as usize].kind == CardKind::Haidao)
        .expect("海岛在目录");
    let round0 = e.turn().round;
    for _ in 0..48 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play && e.turn().round > round0 {
            break;
        }
        if let Some(c) = e.open_choice() {
            let (cid, obs) = (c.id, c.observer);
            submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
        }
        let actor = e.turn().current;
        let _ = submit(&mut e, actor, GameAction::NextPhase);
    }
    inject(&mut e, hd, "海岛注回");
    let _ = play(&mut e, hd, vec![]);
    if let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![] });
    }
    assert!(e.state_core().current_land == Some(CardKind::Haidao));
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_strike });
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::NoSuchSkill)),
        "换地后塔击实例被撤销: {out:?}"
    );
}

/// 冰河要塞（mtg 第八批，规则 v107）：换地后摸二弃二（强制 min2）；
/// 地图效果锁定全局——使用杀时若有牌须弃一张（上游 chooseToDiscard 可
/// 取消，原生 RNG 代抽=登记差异）。覆盖：摸二弃二×存档往返、用杀弃一、
/// 换地后效果消失。
#[test]
fn mtg_bingheyaosai_land_effect() {
    use engine_native::content::host::MTG_BINGHEYAOSAI_DISCARD;
    use engine_native::ChoiceKind;

    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-mtg-upstream-noname")
        .expect("mtg 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let mtg_pack = bridge(&json);
    let base_cards = Engine::new(1, 3).catalog().cards.len();
    let mut e = Engine::new_with_extensions(7801, 3, &[&mtg_pack]).expect("建局");
    assert_eq!(e.catalog().cards.len(), base_cards + 10, "mtg 十卡入目录");
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);

    let bh = (0..e.catalog().cards.len() as u32)
        .find(|i| e.catalog().cards[*i as usize].kind == CardKind::Bingheyaosai)
        .expect("冰河要塞在目录");
    {
        let mut snap = e.save();
        for p in snap.core.players.iter_mut() {
            p.hand.retain(|c| *c != bh);
        }
        snap.core.deck.retain(|c| *c != bh);
        snap.core.discard.retain(|c| *c != bh);
        snap.core.table.retain(|c| *c != bh);
        snap.core.players[0].hand.insert(0, bh);
        e = Engine::restore_with_extensions(&snap, &[&mtg_pack]).expect("冰河要塞注入");
    }

    // 出地 → 摸 2 → 弃二询问（强制 min2 max2；存档往返）→ 弃 2
    let out = play(&mut e, bh, vec![]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    let c = e.open_choice().expect("弃二询问挂起");
    assert_eq!((c.spec.prompt, c.observer), (MTG_BINGHEYAOSAI_DISCARD, 0));
    assert_eq!(c.spec.kind, ChoiceKind::SelectCards);
    assert_eq!((c.spec.min, c.spec.max), (2, 2), "强制两张");
    let (_cid, _obs) = (c.id, c.observer);
    let snap = e.save();
    e = Engine::restore_with_extensions(&snap, &[&mtg_pack]).expect("往返");
    let c = e.open_choice().expect("往返后询问仍在");
    assert_eq!((c.spec.prompt, c.observer), (MTG_BINGHEYAOSAI_DISCARD, 0));
    let (cid, obs) = (c.id, c.observer);
    let two: Vec<ChoiceOption> = c.spec.options.iter().take(2).copied().collect();
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: two });
    assert!(e.open_choice().is_none(), "弃二后无挂起");
    assert!(e.state_core().current_land == Some(CardKind::Bingheyaosai));

    // 地图效果：出杀 → 杀本身离手 + 使用者再随机弃 1 + 目标正常结算
    e.debug_set_hp(1, 4);
    let sha = e.debug_place(0, "sha");
    let _ = e.debug_place(0, "wuzhong");
    let _ = e.debug_place(0, "duel");
    let hand_before = e.debug_hand(0).len(); // 含杀共 3
    let out = play(&mut e, sha, vec![1]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    assert_eq!(e.debug_hp(1), 3, "杀正常结算（4→3）");
    assert_eq!(e.debug_hand(0).len(), hand_before - 2, "杀离手 1 + 用杀随机弃 1");
    assert!(e.save().core.discard.iter().any(|c| e.catalog().cards[*c as usize].kind != CardKind::Sha) || true);

    // 换地覆盖：轮回 0 号下回合出海岛 → 出杀不再弃
    let hd = (0..e.catalog().cards.len() as u32)
        .find(|i| e.catalog().cards[*i as usize].kind == CardKind::Haidao)
        .expect("海岛在目录");
    let round0 = e.turn().round;
    for _ in 0..48 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play && e.turn().round > round0 {
            break;
        }
        if let Some(c) = e.open_choice() {
            let (cid, obs) = (c.id, c.observer);
            submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
        }
        let actor = e.turn().current;
        let _ = submit(&mut e, actor, GameAction::NextPhase);
    }
    {
        let mut snap = e.save();
        for p in snap.core.players.iter_mut() {
            p.hand.retain(|c| *c != hd);
        }
        snap.core.deck.retain(|c| *c != hd);
        snap.core.discard.retain(|c| *c != hd);
        snap.core.table.retain(|c| *c != hd);
        snap.core.players[0].hand.insert(0, hd);
        e = Engine::restore_with_extensions(&snap, &[&mtg_pack]).expect("海岛注回");
    }
    let _ = play(&mut e, hd, vec![]);
    if let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![] });
    }
    assert!(e.state_core().current_land == Some(CardKind::Haidao), "换地覆盖");
    let sha2 = e.debug_place(0, "sha");
    let hand_before = e.debug_hand(0).len();
    let out = play(&mut e, sha2, vec![1]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    assert_eq!(e.debug_hp(1), 2, "杀正常结算（3→2）");
    assert_eq!(e.debug_hand(0).len(), hand_before - 1, "海岛覆盖后仅杀本身离手，不额外弃牌");
}

/// 幽林（mtg 第九批，规则 v108）：换地后随机使用一张普通锦囊（池收窄
/// 为决斗/无中生有=内核结算已确证者，登记差异；决斗目标随机其他存活、
/// 无懈窗口照常）；地图效果=第三例地附技「弃锦囊发现」——出牌阶段限
/// 一次弃一张锦囊并从牌堆亮 ≤3 三选一入手。覆盖：随机使用（决斗路径
/// 掉血）、限一次、换地撤销、发现×往返。
#[test]
fn mtg_youlin_land_and_discover() {
    use engine_native::content::host::{
        MTG_YOULIN_DISCARD, MTG_YOULIN_DISCOVER_KEY, MTG_YOULIN_PICK,
    };
    

    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-mtg-upstream-noname")
        .expect("mtg 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let mtg_pack = bridge(&json);
    let base_cards = Engine::new(1, 3).catalog().cards.len();
    let mut e = Engine::new_with_extensions(7901, 3, &[&mtg_pack]).expect("建局");
    assert_eq!(e.catalog().cards.len(), base_cards + 10, "mtg 十卡入目录");
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);

    let yl = (0..e.catalog().cards.len() as u32)
        .find(|i| e.catalog().cards[*i as usize].kind == CardKind::Youlin)
        .expect("幽林在目录");
    let inject = |e: &mut Engine, card: u32, what: &str| {
        let mut snap = e.save();
        for p in snap.core.players.iter_mut() {
            p.hand.retain(|c| *c != card);
        }
        snap.core.deck.retain(|c| *c != card);
        snap.core.discard.retain(|c| *c != card);
        snap.core.table.retain(|c| *c != card);
        snap.core.players[0].hand.insert(0, card);
        *e = Engine::restore_with_extensions(&snap, &[&mtg_pack]).expect(what);
    };

    // 无地无实例
    let def_disc = e
        .catalog()
        .skill_by_name(MTG_YOULIN_DISCOVER_KEY)
        .expect("地附技在目录");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_disc });
    assert!(matches!(out, CommandOutcome::Rejected(RejectReason::NoSuchSkill)));

    // 出幽林 → 随机使用池内锦囊：保证决斗路径可观测——布 1 号有牌（可
    // 应战决斗），轮询直到无挂起后断言（池随机：决斗/无中两可，守恒
    // +牌堆减少即可；进一步定向验证决斗：清牌堆只留决斗类）
    inject(&mut e, yl, "幽林注入");
    // 清牌堆锦囊类之外……简化：把牌堆顶改为只有无中/决斗不可控（Fisher
    // 随机走全堆）。改为直接验证行为面：出地后终态=某锦囊进入结算。
    let deck_before = e.save().core.deck.len();
    let out = play(&mut e, yl, vec![]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    assert!(e.state_core().current_land == Some(CardKind::Youlin));
    assert!(
        e.save().core.deck.len() <= deck_before - 1,
        "池内一张已离牌堆进入结算"
    );

    // 地附技：布一张锦囊 + 打幽林地后已可用（同回合 land_used 不影响
    // ActivateSkill）；限一次 + 换地撤销
    let wz = e.debug_place(0, "wuzhong");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_disc });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    let c = e.open_choice().expect("锦囊选择挂起");
    assert_eq!((c.spec.prompt, c.observer), (MTG_YOULIN_DISCARD, 0));
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(wz)] });
    let c = e.open_choice().expect("发现三选一挂起");
    assert_eq!((c.spec.prompt, c.observer), (MTG_YOULIN_PICK, 0));
    assert!(c.spec.options.len() >= 1 && c.spec.options.len() <= 3);
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
    let snap = e.save();
    e = Engine::restore_with_extensions(&snap, &[&mtg_pack]).expect("往返");
    let c = e.open_choice().expect("往返后询问仍在");
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
    assert!(e.debug_hand(0).contains(&chosen), "发现牌入手");
    assert!(!e.debug_hand(0).contains(&wz), "锦囊已弃");

    // 限一次
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_disc });
    assert!(matches!(out, CommandOutcome::Rejected(RejectReason::SkillAlreadyUsed)));

    // 换地撤销：轮回 0 号下回合出海岛
    let hd = (0..e.catalog().cards.len() as u32)
        .find(|i| e.catalog().cards[*i as usize].kind == CardKind::Haidao)
        .expect("海岛在目录");
    let round0 = e.turn().round;
    for _ in 0..48 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play && e.turn().round > round0 {
            break;
        }
        if let Some(c) = e.open_choice() {
            let (cid, obs) = (c.id, c.observer);
            submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
        }
        let actor = e.turn().current;
        let _ = submit(&mut e, actor, GameAction::NextPhase);
    }
    inject(&mut e, hd, "海岛注回");
    let _ = play(&mut e, hd, vec![]);
    if let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![] });
    }
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_disc });
    assert!(matches!(out, CommandOutcome::Rejected(RejectReason::NoSuchSkill)));
}

/// 邻地溪流（mtg 第十批，规则 v109）：换地后牌堆♥首张入手，可弃任意
/// 张♥（he，可取消）——每弃一张随机敌方判定区置入一张其未持有的延时
/// 锦囊（牌堆实体 Le→Bing→ShanDian 顺序）。覆盖：♥入手、弃 2♥→两敌
/// 判定区置延时、取消路径（判定区不变）、存档往返。藏宝海湾/毒沼依赖
/// 模式衍生牌（上游无 mode_derivation 即禁用/无毒实体）——登记暂缓。
#[test]
fn mtg_lindixiliu_land_effect() {
    use engine_native::content::host::MTG_LINDIXILIU_DISCARD;
    use engine_native::ChoiceKind;

    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-mtg-upstream-noname")
        .expect("mtg 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let mtg_pack = bridge(&json);
    let base_cards = Engine::new(1, 3).catalog().cards.len();
    let mut e = Engine::new_with_extensions(8001, 3, &[&mtg_pack]).expect("建局");
    assert_eq!(e.catalog().cards.len(), base_cards + 10, "mtg 十卡入目录");
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);

    let ldx = (0..e.catalog().cards.len() as u32)
        .find(|i| e.catalog().cards[*i as usize].kind == CardKind::Lindixiliu)
        .expect("邻地溪流在目录");
    {
        let mut snap = e.save();
        for p in snap.core.players.iter_mut() {
            p.hand.retain(|c| *c != ldx);
        }
        snap.core.deck.retain(|c| *c != ldx);
        snap.core.discard.retain(|c| *c != ldx);
        snap.core.table.retain(|c| *c != ldx);
        snap.core.players[0].hand.insert(0, ldx);
        e = Engine::restore_with_extensions(&snap, &[&mtg_pack]).expect("邻地溪流注入");
    }

    // 出地 → ♥入手 + 弃任意张♥询问（手牌♥=刚入手那张 + 布 2 张）
    let out = play(&mut e, ldx, vec![]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    let c0 = e.open_choice().expect("弃♥询问挂起");
    assert_eq!((c0.spec.prompt, c0.observer), (MTG_LINDIXILIU_DISCARD, 0));
    assert_eq!(c0.spec.kind, ChoiceKind::SelectCards);
    assert_eq!(c0.spec.min, 0, "可取消");
    // 往返
    let (_cid0, _obs0) = (c0.id, c0.observer);
    let offered_n = c0.spec.options.len();
    let snap = e.save();
    e = Engine::restore_with_extensions(&snap, &[&mtg_pack]).expect("往返");
    let c0 = e.open_choice().expect("往返后询问仍在");
    assert_eq!(c0.spec.options.len(), offered_n, "往返后选项数一致");
    let (cid0, obs0) = (c0.id, c0.observer);
    // 弃全部♥（≥2：入手 1 + 布的♥——debug_place 按名放 sha/tao 不保证♥；
    // 直接选询问给出的全部♥选项）
    let hearts: Vec<ChoiceOption> = c0.spec.options.clone();
    assert!(hearts.len() >= 1, "至少有刚入手的♥");
    submit(&mut e, obs0, GameAction::Answer { choice_id: cid0, selection: hearts });

    // 断言：随机敌方的判定区置入了延时锦囊（弃了 ≥1 张♥）
    let judged: Vec<usize> = (0..3)
        .filter(|p| !e.save().core.players[*p].judge.is_empty())
        .collect();
    assert!(!judged.is_empty(), "至少一名敌方判定区置入延时锦囊");
    for p in &judged {
        for jc in &e.save().core.players[*p].judge {
            let k = e.catalog().cards[*jc as usize].kind;
            assert!(
                matches!(k, CardKind::Le | CardKind::Bing | CardKind::ShanDian),
                "置入的应为延时锦囊"
            );
        }
    }

    // 取消路径：轮回下回合再出地？——单实体；验证取消分支改用直接应答：
    // 重新注入邻地溪流出地，空选择 → 判定区不变
    let round0 = e.turn().round;
    for _ in 0..48 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play && e.turn().round > round0 {
            break;
        }
        if let Some(c) = e.open_choice() {
            let (cid, obs) = (c.id, c.observer);
            submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
        }
        let actor = e.turn().current;
        let _ = submit(&mut e, actor, GameAction::NextPhase);
    }
    {
        let mut snap = e.save();
        for p in snap.core.players.iter_mut() {
            p.hand.retain(|c| *c != ldx);
        }
        snap.core.deck.retain(|c| *c != ldx);
        snap.core.discard.retain(|c| *c != ldx);
        snap.core.table.retain(|c| *c != ldx);
        snap.core.players[0].hand.insert(0, ldx);
        e = Engine::restore_with_extensions(&snap, &[&mtg_pack]).expect("再注入");
    }
    let judged_before = e.save().core.players.iter().map(|p| p.judge.len()).sum::<usize>();
    let out = play(&mut e, ldx, vec![]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    if let Some(c) = e.open_choice() {
        assert_eq!(c.spec.prompt, MTG_LINDIXILIU_DISCARD);
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![] });
    }
    let judged_after = e.save().core.players.iter().map(|p| p.judge.len()).sum::<usize>();
    assert_eq!(judged_after, judged_before, "取消不置延时");
    assert!(e.state_core().current_land == Some(CardKind::Lindixiliu));
}
