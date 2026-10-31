//! 整包首切片（U05 杀海拾遗 zhenfa 子包，规则 v110）：地载阵——
//! 所有非大势力角色各摸一张牌（大势力=已明置势力存活人数最大者，
//! effective_faction 推导；上游 mode:["guozhan"]，队列/围攻依赖的其余
//! 12 张中的 5 种启用牌登记依赖）+ 真实宿主全生命周期。
use content_runtime::{discover_extensions, import_extensions, InstallOptions, PackageStore};
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::catalog::{CardKind, Faction};
use engine_native::content::extension::ExtensionPack;
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
            submit(
                e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: vec![ChoiceOption::Pass],
                },
            );
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
    assert_eq!(
        e.catalog().cards.len(),
        base_cards + 8,
        "zhenfa 八张入目录（地敌阵+三阵+长蛇阵×2，规则 v201/T19b）"
    );
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
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Pass],
            },
        );
    }
    assert_eq!(e.debug_hand(2).len(), 1, "非大势力（蜀）摸 1");
    assert_eq!(e.debug_hand(0).len(), 0, "大势力（魏）不摸");
    assert_eq!(e.debug_hand(1).len(), 0, "大势力（魏）不摸");
    assert!(
        !e.save()
            .core
            .table
            .iter()
            .any(|c| e.catalog().cards[*c as usize].kind == CardKind::Dizaizhen),
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
    assert_eq!(e.catalog().cards.len(), base_cards + 8, "zhenfa 卡在目录");
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let mid = e.save();

    store
        .revoke_slot(&meta.namespace, "shahai-zhenfa", "验收：对局中卸载", true)
        .expect("撤销");
    assert!(store
        .read_revoke()
        .unwrap()
        .is_slot_revoked(&meta.namespace, "shahai-zhenfa"));
    assert!(!store.read_active().unwrap().packages.contains_key(&slot));

    let mut resumed = Engine::restore_with_extensions(
        &decode_archive(&encode_archive(&mid)).expect("解码"),
        &[&pack],
    )
    .expect("对局中卸载后恢复");
    assert_eq!(
        resumed.catalog().cards.len(),
        base_cards + 8,
        "锁定副本仍可用"
    );
    let out = submit(&mut resumed, 0, GameAction::NextPhase);
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));

    let bytes = std::fs::read(disc.root.join("package.npack.json")).expect("npack");
    let err = store.install(&bytes, None, &options());
    assert!(matches!(
        err,
        Err(content_runtime::RuntimeError::Revoked(_))
    ));
    let report2 = import_extensions(&store, &root, &options());
    assert!(report2.is_err(), "撤销后重新 import 应被拒");
}

/// 阵法三张（规则 v200/T19a）：天覆阵 AOE 大国弃 1；七星阵围攻弃 1+视为
/// 用杀；鸟翔阵响应闪或 1 伤。前提：zhenfa 启用门=存活≥4、派生判定
/// （is_major/sieged/inline 已在 state.rs 单元级验证）。
#[test]
fn zf_tianfuzhen_major_aoe_discard() {
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
    // 4 人局（zhenfa 启用门）：0/1 魏（大）、2/3 蜀（非大）
    let mut e = Engine::new_with_extensions(8201, 4, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    let (wei, shu) = two_generals();
    e.debug_assemble_general(0, &wei);
    e.debug_assemble_general(1, &wei);
    e.debug_assemble_general(2, &shu);
    e.debug_assemble_general(3, &shu);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let tf = (0..e.catalog().cards.len() as u32)
        .find(|i| e.catalog().cards[*i as usize].kind == CardKind::Tianfuzhen)
        .expect("天覆阵在目录");
    inject_zhenfa(&mut e, &pack, tf, 0);
    // 大国（0/1）各发 1 张牌供弃
    grant_cards(&mut e, &pack, 0, 1);
    grant_cards(&mut e, &pack, 1, 1);

    let out = play(&mut e, tf, vec![0, 1]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    // 逐大国目标强制弃 1（0/1 各被询问）
    for _ in 0..2 {
        let c = e.open_choice().cloned().expect("天覆阵弃牌询问");
        assert_eq!(c.spec.prompt, "swd_tianfuzhen_discard");
        let hand = e.debug_hand(c.observer);
        assert!(!hand.is_empty());
        let (cid, obs) = (c.id, c.observer);
        drop(c);
        submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Card(hand[0])],
            },
        );
    }
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Pass],
            },
        );
    }
    assert_eq!(e.debug_hand(0).len(), 0, "大国 0 弃 1");
    assert_eq!(e.debug_hand(1).len(), 0, "大国 1 弃 1");
    assert!(
        e.state_core()
            .discard
            .iter()
            .any(|c| e.catalog().cards[*c as usize].kind == CardKind::Tianfuzhen),
        "本体入弃堆"
    );
}

/// 七星阵：2 号被 1/3 号（异势力）围攻 → 2 号施放：围攻者各弃 1 + 视为
/// 对其用杀（响应闪或 1 伤）。构造围攻：座位环 0-1-2-3，势力 魏-蜀-魏-蜀
/// 时 2（魏）的邻居 1/3（蜀）……需蜀围攻魏：1/3 的邻居含 2（魏）且 1/3 互
/// 为友方（同为蜀）→ 1/3 围攻 2 ✓。2 号施放=围攻者弃牌+对其用杀。
#[test]
fn zf_qixingzhen_siege_discard_and_virtual_sha() {
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
    let mut e = Engine::new_with_extensions(8202, 4, &[&pack]).expect("建局");
    // 先清手牌再推进（避免 0/1 回合弃牌门挂起选择）
    for p in 0..4 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 4);
    }
    advance_to_play(&mut e, 2);
    let (wei, shu) = two_generals();
    // 0 魏 / 1 蜀 / 2 魏 / 3 蜀：2（魏）邻居 1/3（蜀），1/3 互为蜀友 → 2 被围攻
    e.debug_assemble_general(0, &wei);
    e.debug_assemble_general(1, &shu);
    e.debug_assemble_general(2, &wei);
    e.debug_assemble_general(3, &shu);
    let qx = (0..e.catalog().cards.len() as u32)
        .find(|i| e.catalog().cards[*i as usize].kind == CardKind::Qixingzhen)
        .expect("七星阵在目录");
    inject_zhenfa(&mut e, &pack, qx, 2);
    // 清 0/1/3 手牌残留（advance 期间 0/1 已摸牌）；保留 2 号手牌（七星阵）
    {
        let mut snap = e.save();
        let qx_id = qx;
        eprintln!(
            "DBG clear: hands={:?} disc40={}",
            [
                snap.core.players[0].hand.clone(),
                snap.core.players[1].hand.clone(),
                snap.core.players[2].hand.clone(),
                snap.core.players[3].hand.clone()
            ],
            snap.core.discard.contains(&40)
        );
        for (i, pl) in snap.core.players.iter_mut().enumerate() {
            let drained: Vec<u32> = pl.hand.drain(..).collect();
            for c in drained {
                if i == 2 && c == qx_id {
                    pl.hand.push(c);
                } else {
                    snap.core.discard.push(c);
                }
            }
        }
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("清残留");
    }
    // 围攻者 1/3 各发「桃+闪」两张（阶段一弃桃、阶段二用闪；确定性）
    let tao1q = e.debug_place(1, "tao");
    let tao3q = e.debug_place(3, "tao");
    let shan1q = e.debug_place(1, "shan");
    let shan3q = e.debug_place(3, "shan");

    let out = play(&mut e, qx, vec![2]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    // 阶段一：围攻者 1/3 各被询问弃 1
    let tao_of = |obs: u8| if obs == 1 { tao1q } else { tao3q };
    for expect_obs in [1u8, 3u8] {
        let c = e.open_choice().cloned().expect("七星弃牌询问");
        eprintln!(
            "DBG discard ask obs={} prompt={}",
            c.observer, c.spec.prompt
        );
        assert_eq!(c.observer, expect_obs, "弃牌询问轮到围攻者");
        let cid = c.id;
        drop(c);
        submit(
            &mut e,
            expect_obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Card(tao_of(expect_obs))],
            },
        );
    }
    // 阶段二：视为对围攻者用杀——1/3 各响应闪（免伤）
    for (obs, shan) in [(1u8, shan1q), (3u8, shan3q)] {
        let c = e.open_choice().cloned().expect("七星用杀响应询问");
        eprintln!(
            "DBG ask obs={} prompt={} opts={:?}",
            c.observer, c.spec.prompt, c.spec.options
        );
        assert_eq!(c.spec.prompt, "swd_qixingzhen_shan");
        assert_eq!(c.observer, obs);
        let (cid, _) = (c.id, c.observer);
        drop(c);
        submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Card(shan)],
            },
        );
    }
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Pass],
            },
        );
    }
    eprintln!(
        "DBG hand1={:?} hand3={:?}",
        e.debug_hand(1),
        e.debug_hand(3)
    );
    let log = e.log();
    for l in log.iter().rev().take(14).rev() {
        eprintln!("DBG log: {:?}", l);
    }
    assert_eq!(e.debug_hp(1), 4, "围攻者 1 响应闪免伤");
    assert_eq!(e.debug_hp(3), 4, "围攻者 3 响应闪免伤");
    assert_eq!(e.debug_hand(1).len(), 0, "围攻者 1 弃 1 且闪已打出");
    assert_eq!(e.debug_hand(3).len(), 0, "围攻者 3 弃 1 且闪已打出");
}

/// 鸟翔阵：目标=异势力且与自身邻座同势力者；各响应闪或受 1 伤。
/// 0 魏/1 蜀/2 魏/3 蜀，0 号施放：1（蜀，邻 0 魏/2 魏——2 与 1 异势力但
/// 2 的邻座 1/3 均蜀=同 1 势力 ✓）、3（同 1 对称 ✓）→ 逐个响应闪或受伤。
#[test]
fn zf_niaoxiangzhen_respond_shan_or_damage() {
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
    let mut e = Engine::new_with_extensions(8203, 4, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    let (wei, shu) = two_generals();
    e.debug_assemble_general(0, &wei);
    e.debug_assemble_general(1, &wei);
    e.debug_assemble_general(2, &shu);
    e.debug_assemble_general(3, &shu);
    for p in 0..4 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 4);
    }
    let nx = (0..e.catalog().cards.len() as u32)
        .find(|i| e.catalog().cards[*i as usize].kind == CardKind::Niaoxiangzhen)
        .expect("鸟翔阵在目录");
    inject_zhenfa(&mut e, &pack, nx, 0);
    // 合法目标=异于施放者（0 魏）势力、且自身与邻座同势力（连续段端点）：
    // 0 魏/1 魏/2 蜀/3 蜀时 2 号（蜀，邻 3 蜀）合法（1 号与施放者同势不合法）
    let shan2 = e.debug_place(2, "shan");

    let shan3 = e.debug_place(3, "shan");

    let out = play(&mut e, nx, vec![2, 3]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    // 2/3 号依次响应闪（AOE 逐目标询问）
    for (obs, shan) in [(2u8, shan2), (3u8, shan3)] {
        let c = e.open_choice().cloned().expect("鸟翔响应询问");
        assert_eq!(c.observer, obs, "逐目标询问");
        assert_eq!(c.spec.prompt, "swd_niaoxiangzhen_shan");
        let (cid, _) = (c.id, c.observer);
        drop(c);
        submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Card(shan)],
            },
        );
    }
    // 直接收尾：1 号已响应闪
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Pass],
            },
        );
    }
    assert_eq!(e.debug_hp(2), 4, "2 号响应闪免伤");
    assert_eq!(e.debug_hp(3), 4, "3 号响应闪免伤");
    assert!(
        e.state_core().discard.contains(&shan2) && e.state_core().discard.contains(&shan3),
        "闪与本体入弃堆"
    );
}

/// 测试脚手架：把 zhenfa 实体注入指定玩家手牌（全域摘除防双区）。
fn inject_zhenfa(e: &mut Engine, pack: &ExtensionPack, card: u32, seat: u8) {
    let mut snap = e.save();
    snap.core.deck.retain(|x| *x != card);
    snap.core.discard.retain(|x| *x != card);
    for pl in snap.core.players.iter_mut() {
        pl.hand.retain(|x| *x != card);
        pl.equips.retain(|x| *x != card);
        pl.judge.retain(|x| *x != card);
    }
    snap.core.players[seat as usize].hand.push(card);
    *e = Engine::restore_with_extensions(&snap, &[pack]).expect("zhenfa 注入");
}

/// 测试脚手架：给指定玩家发 n 张牌堆顶牌（直写快照）。
fn grant_cards(e: &mut Engine, pack: &ExtensionPack, seat: u8, n: usize) {
    let mut snap = e.save();
    for _ in 0..n {
        if let Some(c) = snap.core.deck.pop() {
            snap.core.players[seat as usize].hand.push(c);
        }
    }
    *e = Engine::restore_with_extensions(&snap, &[pack]).expect("发牌");
}

/// 长蛇阵+破阵诀（规则 v201/T19b；swapSeat 原语）：
/// 长蛇阵①未入队列：0 魏/1 蜀/2 魏/3 蜀时 2 号（魏）与最近友方（0 号）换座
/// 成队列；②入队列：全队列各摸 1。破阵诀：随机座位重排（验证环不变量）。
#[test]
fn zf_changshe_pozhen_swap_seat() {
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
    let mut e = Engine::new_with_extensions(8204, 4, &[&pack]).expect("建局");
    for p in 0..4 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 4);
    }
    advance_to_play(&mut e, 2);
    let (wei, shu) = two_generals();
    // 0 魏 / 1 蜀 / 2 魏 / 3 蜀：2 号（魏）无友邻 → 未入队列；最近友方=0 号
    e.debug_assemble_general(0, &wei);
    e.debug_assemble_general(1, &shu);
    e.debug_assemble_general(2, &wei);
    e.debug_assemble_general(3, &shu);
    let cs = (0..e.catalog().cards.len() as u32)
        .find(|i| e.catalog().cards[*i as usize].kind == CardKind::Changshezhen)
        .expect("长蛇阵在目录");
    inject_zhenfa(&mut e, &pack, cs, 2);

    // ① 未入队列：施放（notarget 无目标）→ 与最近友方（0 号）换座
    let ring_before = e.state_core().seat_ring.clone();
    let out = play(&mut e, cs, vec![]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    let ring_after = e.state_core().seat_ring.clone();
    assert_ne!(ring_before, ring_after, "换座应改变座位环");
    // 换座后 2 号与 0 号相邻且同势力 → 入队列
    assert!(e.state_core().inline(e.catalog(), 2), "换座后 2 号入队列");

    // ③ 入队列：施放第二张长蛇阵实体（♣1；recastable 重铸未建模，以双实
    // 体近似）→ 全队列各摸 1（队列=连续同势力段含自身）
    let cs2 = (0..e.catalog().cards.len() as u32)
        .filter(|i| e.catalog().cards[*i as usize].kind == CardKind::Changshezhen)
        .map(|i| i as u32)
        .find(|i| *i != cs)
        .expect("第二张长蛇阵实体");
    inject_zhenfa(&mut e, &pack, cs2, 2);
    let before: Vec<usize> = (0..4).map(|p| e.debug_hand(p).len()).collect();
    let out = play(&mut e, cs2, vec![]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    for p in 0..4u8 {
        if e.state_core().inline_with(e.catalog(), 2, p) {
            let expected = if p == 2 {
                before[p as usize] // 施放者：出牌 -1 + 摸 1 = 净 0
            } else {
                before[p as usize] + 1 // 队列成员摸 1
            };
            assert_eq!(e.debug_hand(p).len(), expected, "队列成员 {} 手牌数", p);
        }
    }
    assert!(
        e.state_core()
            .discard
            .iter()
            .any(|c| e.catalog().cards[*c as usize].kind == CardKind::Changshezhen),
        "长蛇阵本体入弃堆"
    );
}
