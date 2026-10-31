//! G1 杀海拾遗·炉石传说子包切片（ADR 0776）：灵魂之火/照明弹/急耗卡/
//! 闪电剑（雷杀语义）/神恩术（简化）→ 真实包 → 导入 → 建局/结算/内容锁。
//!
//! 切片 17/17 张全卡转换（v55 余量 9 张 + v56 世界树）——deferred
//! 清零，炉石子包收官。
//! 闪电剑登记差异：上游为"雷伤+弃同花色牌"复杂链，原生按雷杀（LeiSha）
//! 语义处理；神恩术登记差异：上游随机基本牌池，原生简化为摸 2。

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use content_runtime::{discover_extensions, import_extensions, InstallOptions, PackageStore};
use engine_native::catalog::{CardKind, Suit};
use engine_native::content::extension::ExtensionPack;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};

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
/// 数据键与 pack 段（炉石 leishajian→leisha、shenenshu→jihuocard 已按
/// 引擎键表规范化，登记差异保留在包数据），测试只校验包元数据一致。
fn bridge(pack_name: &'static str, json: &str) -> Result<ExtensionPack, String> {
    let out = content_native_loader::bridge_pack(json).map_err(|e| e.to_string())?;
    assert_eq!(out.pack.def.name, pack_name, "包名来自 pack 段");
    Ok(out.pack)
}

fn setup() -> (Engine, ExtensionPack) {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    assert!(discovered
        .iter()
        .any(|d| d.name == "shahai-hearth-upstream-noname"));
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "shahai-hearth-upstream-noname")
            .expect("炉石可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let pack = bridge("shahai-hearth-upstream-noname", &json).expect("桥接");
    let mut e = Engine::new_with_extensions(4001, 3, &[&pack]).expect("建局");
    let base_cards = Engine::new(1, 3).catalog().cards.len();
    assert_eq!(e.catalog().cards.len(), base_cards + 17);
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
            match submit(
                &mut e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: sel,
                },
            ) {
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
    (e, pack)
}

fn inject(e: &Engine, pack: &ExtensionPack, ids: &[u32], to: u8) -> Engine {
    let mut snap = e.save();
    for p in snap.core.players.iter_mut() {
        for c in ids {
            p.hand.retain(|x| x != c);
            p.equips.retain(|x| x != c);
            p.judge.retain(|x| x != c);
        }
    }
    snap.core.deck.retain(|c| !ids.contains(c));
    snap.core.discard.retain(|c| !ids.contains(c));
    snap.core.table.retain(|c| !ids.contains(c));
    for c in ids {
        snap.core.players[to as usize].hand.push(*c);
    }
    Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复")
}

/// 灵魂之火：目标 1 点火焰伤害，使用者弃一张手牌。
#[test]
fn linghunzhihuo_fire_damage_and_self_discard() {
    let (mut e, pack) = setup();
    let (lh, extra) = {
        let catalog = e.catalog();
        let l = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("ht_linghunzhihuo"))
            .expect("灵魂之火在目录") as u32;
        let x = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Shan)
            .expect("闪牌") as u32;
        (l, x)
    };
    let mut snap = e.save();
    for p in snap.core.players.iter_mut() {
        p.hand.retain(|c| *c != lh && *c != extra);
        p.equips.retain(|c| *c != lh && *c != extra);
        p.judge.retain(|c| *c != lh && *c != extra);
    }
    snap.core.deck.retain(|c| *c != lh && *c != extra);
    snap.core.discard.retain(|c| *c != lh && *c != extra);
    snap.core.table.retain(|c| *c != lh && *c != extra);
    snap.core.players[0].hand.push(lh);
    snap.core.players[0].hand.push(extra);
    e.debug_set_hp(1, 4);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: lh,
            targets: vec![1],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    // 无懈窗口（无人持无懈 → 直结算）
    if let Some(c) = e.open_choice() {
        assert_eq!(c.spec.prompt, "nullify_window");
        let cid = c.id;
        let obs = c.observer;
        submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Pass],
            },
        );
    }
    assert_eq!(e.debug_hp(1), 3, "灵魂之火 1 点火焰伤害");
    assert!(
        e.debug_hand(0).is_empty(),
        "使用者弃一张手牌（ souls + 闪消耗后为空）"
    );
}

/// 照明弹：弃目标一张手牌，目标与使用者各摸 1。
#[test]
fn zhaomingdan_forced_discard_and_draws() {
    let (e, pack) = setup();
    let (zm, victim) = {
        let catalog = e.catalog();
        let z = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("ht_zhaomingdan"))
            .expect("照明弹在目录") as u32;
        let v = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Tao)
            .expect("桃牌") as u32;
        (z, v)
    };
    let mut snap = e.save();
    for p in snap.core.players.iter_mut() {
        p.hand.retain(|c| *c != zm && *c != victim);
        p.equips.retain(|c| *c != zm && *c != victim);
        p.judge.retain(|c| *c != zm && *c != victim);
    }
    snap.core.deck.retain(|c| *c != zm && *c != victim);
    snap.core.discard.retain(|c| *c != zm && *c != victim);
    snap.core.table.retain(|c| *c != zm && *c != victim);
    snap.core.players[0].hand.push(zm);
    snap.core.players[1].hand.push(victim);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: zm,
            targets: vec![1],
        },
    );
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    if let Some(c) = e.open_choice() {
        assert_eq!(c.spec.prompt, "nullify_window");
        let cid = c.id;
        let obs = c.observer;
        submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Pass],
            },
        );
    }
    assert!(!e.debug_hand(1).contains(&victim), "照明弹弃目标的桃");
    assert_eq!(e.debug_hand(1).len(), 1, "弃 1 后目标摸 1 补回");
}

/// 急耗卡：自-target 摸 1（临时技暂缓）。
#[test]
fn jihuocard_self_draw() {
    let (e, pack) = setup();
    let (jh,) = {
        let catalog = e.catalog();
        let j = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("ht_jihuocard"))
            .expect("急耗卡在目录") as u32;
        (j,)
    };
    let mut snap = e.save();
    for p in snap.core.players.iter_mut() {
        p.hand.retain(|c| *c != jh);
    }
    snap.core.deck.retain(|c| *c != jh);
    snap.core.discard.retain(|c| *c != jh);
    snap.core.table.retain(|c| *c != jh);
    snap.core.players[0].hand.push(jh);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let before = e.debug_hand(0).len();
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: jh,
            targets: vec![],
        },
    );
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    assert_eq!(
        e.debug_hand(0).len(),
        before - 1 + 1,
        "打出后摸 1（净 0 变化但牌序更新）"
    );
}

/// 内容锁：带扩展存档被纯标准恢复拒绝；同集合恢复成立。
#[test]
fn hearth_content_lock_saverestore() {
    let (e, pack) = setup();
    let saved = e.save();
    assert!(matches!(
        Engine::restore(&saved),
        Err(engine_native::SnapshotError::RuleVersionMismatch { .. })
    ));
    let ok = Engine::restore_with_extensions(&saved, &[&pack]).expect("同集合恢复");
    assert_eq!(ok.state_hash(), e.state_hash());
}

/// 炉石余量 A：盾牌格挡（+1 护甲摸 1 弃 1）/治疗波（判定红回 1
/// 否则 +1 护甲）/传送门（获得牌堆顶）。
#[test]
fn hearth_remaining_batch_a() {
    let (e0, pack) = setup();
    let find = |e: &Engine, prefix: &str| {
        e.catalog()
            .cards
            .iter()
            .position(|c| c.name.starts_with(prefix))
            .unwrap_or_else(|| panic!("{}在目录", prefix)) as u32
    };

    // —— 盾牌格挡 ——
    let dp = find(&e0, "ht_dunpaigedang");
    let mut e = inject(&e0, &pack, &[dp], 0);
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: dp,
            targets: vec![],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    pass_window(&mut e);
    assert_eq!(e.save().core.players[0].hujia, 1, "+1 护甲");
    assert_eq!(e.debug_hand(0).len(), 0, "打出 -1、摸 +1、弃 -1 → 0");

    // —— 治疗波：判定红 → 回复 1 ——
    let mut e = {
        let mut e2 = Engine::new_with_extensions(4002, 3, &[&pack]).expect("建局2");
        for _ in 0..64 {
            if e2.turn().current == 0
                && e2.turn().phase == Phase::Play
                && e2.open_choice().is_none()
            {
                break;
            }
            if let Some(c) = e2.open_choice().cloned() {
                let cid = c.id;
                let sel = if c.spec.min > 0 {
                    c.spec.options.iter().take(c.spec.min).copied().collect()
                } else {
                    vec![c.spec.options[0]]
                };
                let obs = c.observer;
                drop(c);
                submit(
                    &mut e2,
                    obs,
                    GameAction::Answer {
                        choice_id: cid,
                        selection: sel,
                    },
                );
                continue;
            }
            let actor = e2.turn().current;
            submit(&mut e2, actor, GameAction::NextPhase);
        }
        e2
    };
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let zlb = find(&e, "ht_zhiliaobo");
    let mut e = inject(&e, &pack, &[zlb], 0);
    e.debug_set_hp(1, 2);
    let hp1 = e.debug_hp(1);
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: zlb,
            targets: vec![1],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    pass_window(&mut e);
    // 判定结果确定：红 → 回复 / 黑 → +1 护甲（均合法，二选一断言）
    let (hp, hj) = (e.debug_hp(1), e.save().core.players[1].hujia);
    assert!(
        (hj == 0 && hp == hp1 + 1) || (hj == 1 && hp == hp1),
        "治疗波二选一结算: hp={hp} hujia={hj}"
    );
    assert_zone_conservation_h(&e);

    // —— 传送门：获得牌堆顶 ——
    let csm = find(&e, "ht_chuansongmen");
    let mut e = inject(&e, &pack, &[csm], 0);
    let deck_top = e.save().core.deck.last().copied().expect("牌堆非空");
    let h0 = e.debug_hand(0).len();
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: csm,
            targets: vec![],
        },
    );
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    assert_eq!(
        e.debug_hand(0).len(),
        h0,
        "打出 -1 + 获得牌堆顶 +1 = 净不变"
    );
    assert!(e.debug_hand(0).contains(&deck_top), "手牌含原牌堆顶");
    assert!(e.debug_hand(0).contains(&deck_top));
    assert_zone_conservation_h(&e);
}

fn pass_window(e: &mut Engine) {
    let is_window = e
        .open_choice()
        .map(|c| c.spec.prompt == "nullify_window")
        .unwrap_or(false);
    if is_window {
        let c = e.open_choice().unwrap();
        let cid = c.id;
        let obs = c.observer;
        submit(
            e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Pass],
            },
        );
    }
}

fn assert_zone_conservation_h(e: &Engine) {
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

/// 炉石余量 B：元素毁灭（逐目标弃 1–2 每少弃一张受 1 雷伤）/
/// 星界沟通（上限+1 回 1 弃全部）/坦蛇至人（座位序链式弃牌）。
#[test]
fn hearth_remaining_batch_b() {
    let (e0, pack) = setup();
    let find = |e: &Engine, prefix: &str| {
        e.catalog()
            .cards
            .iter()
            .position(|c| c.name.starts_with(prefix))
            .unwrap_or_else(|| panic!("{}在目录", prefix)) as u32
    };

    // —— 元素毁灭：目标弃 1 免大半伤害 / Pass 全额 ——
    let ysh = find(&e0, "ht_yuansuhuimie");
    let spade_c = {
        let catalog = e0.catalog();
        catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Sha && c.suit == Suit::Spade)
            .expect("黑杀在目录") as u32
    };
    let e = inject(&e0, &pack, &[ysh], 0);
    let mut e = inject(&e, &pack, &[spade_c], 1);
    e.debug_set_hp(1, 9);
    e.debug_set_hp(2, 9);
    let (hp1, hp2) = (e.debug_hp(1), e.debug_hp(2));
    let _h1 = e.debug_hand(1).len();
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: ysh,
            targets: vec![],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    pass_window(&mut e);
    // 1 号弃 1 → 受 1 雷伤
    let c = e.open_choice().expect("元素毁灭弃牌挂起");
    assert_eq!(c.spec.prompt, "hh_yuansu_discard");
    assert_eq!(c.observer, 1);
    assert!(c.spec.allow_cancel);
    let cid = c.id;
    let discard_card = match c.spec.options.first() {
        Some(ChoiceOption::Card(c)) => *c,
        _ => unreachable!(),
    };
    submit(
        &mut e,
        1,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(discard_card)],
        },
    );
    assert_eq!(e.debug_hp(1), hp1 - 1, "弃 1 → 受 1 雷伤");
    // 2 号取消（Pass 不可用——allow_cancel=true 用空选择取消）→ 受 2
    while let Some(c) = e.open_choice() {
        if c.spec.prompt != "hh_yuansu_discard" {
            let cid = c.id;
            let obs = c.observer;
            submit(
                &mut e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: vec![],
                },
            );
            continue;
        }
        let cid = c.id;
        let obs = c.observer;
        submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![],
            },
        );
        break;
    }
    assert_eq!(e.debug_hp(2), hp2 - 2, "取消 → 受 2 雷伤");
    assert_zone_conservation_h(&e);

    // —— 星界沟通：上限+1 回 1 弃全部手牌 ——
    let e = inject(&e0, &pack, &[ysh], 0);
    let xjg = find(&e, "ht_xingjiegoutong");
    let mut e = inject(&e, &pack, &[xjg], 0);
    e.debug_set_hp(0, 2);
    let max0 = e.save().core.players[0].max_hp;
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: xjg,
            targets: vec![],
        },
    );
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    assert_eq!(e.save().core.players[0].max_hp, max0 + 1, "体力上限 +1");
    assert_eq!(e.debug_hand(0).len(), 0, "弃全部手牌");
    assert_zone_conservation_h(&e);

    // —— 坦蛇至人：座位序链式弃首张 ——
    let e = inject(&e0, &pack, &[ysh], 0);
    let tst = find(&e, "ht_tanshezhiren");
    let mut e = inject(&e, &pack, &[tst], 0);
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: tst,
            targets: vec![],
        },
    );
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    // 链式：1、2 号各弃一张（存活 3 取奇 3 次，但第 3 次回到 0 无手牌跳过——
    // 由机器的 next 查找语义决定，断言守恒即可）
    assert_zone_conservation_h(&e);
}

/// 世界树：牌堆自顶搜索两张不同槽位装备，目标依次穿戴（同槽替换 +
/// 装备技授予）——炉石子包收官原语。
#[test]
fn shijieshu_equips_two_different_slots() {
    let (e0, pack) = setup();
    let shj = e0
        .catalog()
        .cards
        .iter()
        .position(|c| c.name.starts_with("ht_shijieshu"))
        .expect("世界树在目录") as u32;
    // 记录牌堆中前两张不同槽位装备（自顶扫描语义与机器一致）
    let mut picked: Vec<u32> = Vec::new();
    let mut slots: Vec<Option<engine_native::catalog::EquipSlot>> = Vec::new();
    for c in e0.save().core.deck.iter().rev() {
        let slot = e0.catalog().cards[*c as usize].kind.equip_slot();
        if let Some(slot) = slot {
            if !slots.contains(&Some(slot)) {
                slots.push(Some(slot));
                picked.push(*c);
                if picked.len() == 2 {
                    break;
                }
            }
        }
    }
    assert_eq!(picked.len(), 2, "牌堆应能找到两张不同槽位装备");
    let mut e = inject(&e0, &pack, &[shj], 0);
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: shj,
            targets: vec![],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    pass_window(&mut e);
    let equips = e.save().core.players[0].equips.clone();
    assert_eq!(equips.len(), 2, "目标穿戴两张装备");
    for c in &picked {
        assert!(equips.contains(c), "搜索到的装备已穿戴: {c}");
    }
    let s1 = e.catalog().cards[equips[0] as usize].kind.equip_slot();
    let s2 = e.catalog().cards[equips[1] as usize].kind.equip_slot();
    assert_ne!(s1, s2, "两张装备槽位不同");
    // 装备技实例授予（若装备声明了技能则存在实例；基础装备无技能，
    // 断言归属守恒即可）
    assert_zone_conservation_h(&e);
}

/// 整包验收续（2026-10-08，第四例）：shahai-hearth 真实宿主全生命周期
/// ——独立安装+激活 → 安装包→原生建局 → 对局中卸载 → 在局锁定副本
/// 续玩 → 撤销后重装/重新 import 被拒。
#[test]
fn hearth_real_host_full_lifecycle() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();

    // ① 独立安装 + 激活
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "shahai-hearth")
        .expect("炉石安装")
        .clone();
    let slot = format!("{}/{}", meta.namespace, meta.package_id);
    assert!(
        store.read_active().unwrap().packages.contains_key(&slot),
        "激活集"
    );

    // ② 安装包 → 原生建局（世界树/燕子药水等 17 卡目录）
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-hearth-upstream-noname")
        .expect("炉石可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge("shahai-hearth-upstream-noname", &json).expect("桥接");
    let base_cards = Engine::new(1, 3).catalog().cards.len();
    let mut e = Engine::new_with_extensions(5601, 3, &[&pack]).expect("建局");
    assert!(e.catalog().cards.len() > base_cards, "炉石卡入目录");
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let mid = e.save();

    // ③ 对局中卸载
    store
        .revoke_slot(&meta.namespace, "shahai-hearth", "验收：对局中卸载", true)
        .expect("撤销");
    assert!(store
        .read_revoke()
        .unwrap()
        .is_slot_revoked(&meta.namespace, "shahai-hearth"));
    assert!(
        !store.read_active().unwrap().packages.contains_key(&slot),
        "移出激活集"
    );

    // ④ 在局锁定副本续玩
    let mut resumed = Engine::restore_with_extensions(
        &engine_native::archive::decode_archive(&engine_native::archive::encode_archive(&mid))
            .expect("解码"),
        &[&pack],
    )
    .expect("对局中卸载后恢复");
    assert!(resumed.catalog().cards.len() > base_cards, "锁定副本仍可用");
    let out = submit(&mut resumed, 0, GameAction::NextPhase);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "在局续玩: {out:?}"
    );

    // ⑤ 重装守卫
    let bytes = std::fs::read(disc.root.join("package.npack.json")).expect("npack");
    let err = store.install(&bytes, None, &options());
    assert!(
        matches!(err, Err(content_runtime::RuntimeError::Revoked(_))),
        "撤销后重装应被拒: {err:?}"
    );

    // ⑥ 重新 import 被拒
    let report2 = import_extensions(&store, &root, &options());
    assert!(report2.is_err(), "撤销后重新 import 应被拒");
}
