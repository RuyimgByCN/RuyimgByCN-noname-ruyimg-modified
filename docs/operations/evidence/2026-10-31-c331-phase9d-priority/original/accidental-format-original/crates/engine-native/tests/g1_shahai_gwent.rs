//! G1 杀海拾遗·gwent 子包首切片（ADR 0776）：燕子药水（最小手牌数
//! 摸 2 否则 1）/奥祖之雷（1 雷伤后摸 1）/瘟疫（弃首张或失去体力——
//! 登记差异：随机改首张、失去体力简化为伤害）/置换军（反复失去体力
//! 上限）/暴雪药水（弃前两张摸 1——登记差异：确定性前两张）。
//!
//! 切片 21/21 张全卡转换（v60 祖墨手铠封印收官）——deferred 清零，
//! gwent 子包收官。
//! 上游核对：`杀海拾遗/card/gwent/{list,card}.js @ 5c02a745`。

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

/// 桥接统一走 C0 装载器（ADR 0777/0778）：映射来自 cards.json 的
/// 数据键与 pack 段，测试只校验包元数据一致。
fn bridge(pack_name: &'static str, json: &str) -> Result<(ExtensionPack, Vec<String>), String> {
    let out = content_native_loader::bridge_pack(json).map_err(|e| e.to_string())?;
    assert_eq!(out.pack.def.name, pack_name, "包名来自 pack 段");
    Ok((out.pack, out.deferred))
}

fn setup(seed: u64) -> (Engine, ExtensionPack) {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "shahai-gwent-upstream-noname")
            .expect("gwent 可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let (pack, _) = bridge("shahai-gwent-upstream-noname", &json).expect("桥接");
    let mut e = Engine::new_with_extensions(seed, 3, &[&pack]).expect("建局");
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
            drop(c);
            submit(
                &mut e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: sel,
                },
            );
            continue;
        }
        let actor = e.turn().current;
        submit(&mut e, actor, GameAction::NextPhase);
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

/// gwent 纵向：发现→导入→桥接→建局→内容锁→计数对账。
#[test]
fn g1_gwent_vertical_and_lock() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    assert!(discovered
        .iter()
        .any(|d| d.name == "shahai-gwent-upstream-noname"));
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    import_extensions(&store, &root, &options()).expect("import");
    let (e, pack) = setup(5301);
    let base_cards = Engine::new(1, 3).catalog().cards.len();
    assert_eq!(e.catalog().cards.len(), base_cards + 21, "切片 21/21 张");
    let _ = pack;
    assert_zone_conservation(&e);
}

/// 燕子药水（最少手牌摸 2）/奥祖之雷（雷伤+摸 1）/瘟疫（弃首张或
/// 受伤）。
#[test]
fn gwent_spells_yanzhi_aozu_wenyi() {
    let (e0, pack) = setup(5302);
    let find = |e: &Engine, prefix: &str| {
        e.catalog()
            .cards
            .iter()
            .position(|c| c.name.starts_with(prefix))
            .unwrap_or_else(|| panic!("{}在目录", prefix)) as u32
    };

    // 燕子药水：目标手牌最少 → 摸 2
    let yz = find(&e0, "gw_yanziyaoshui");
    let mut e = inject(&e0, &pack, &[yz], 0);
    e.debug_set_hp(1, 9);
    let h2 = e.debug_hand(2).len();
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: yz,
            targets: vec![2],
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
    assert_eq!(e.debug_hand(2).len(), h2 + 2, "最少手牌摸 2");

    // 奥祖之雷：1 雷伤 + 摸 1
    let az = find(&e0, "gw_aozuzhilei");
    let sha = {
        let catalog = e0.catalog();
        catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Sha && c.suit == Suit::Spade)
            .expect("黑杀在目录") as u32
    };
    let mut e = inject(&e0, &pack, &[az, sha], 0);
    e.debug_set_hp(1, 9);
    let hp1 = e.debug_hp(1);
    let h1 = e.debug_hand(1).len();
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: az,
            targets: vec![1],
        },
    );
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    while let Some(c) = e.open_choice() {
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
    assert_eq!(e.debug_hp(1), hp1 - 1, "1 点雷电伤害");
    assert_eq!(e.debug_hand(1).len(), h1 + 1, "目标摸 1");

    // 瘟疫：有手牌弃首张
    let wy = find(&e0, "gw_wenyi");
    let e = inject(&e0, &pack, &[wy], 0);
    let mut e = inject(&e, &pack, &[sha], 1);
    let h1 = e.debug_hand(1).len();
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: wy,
            targets: vec![1],
        },
    );
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    assert_eq!(e.debug_hand(1).len(), h1 - 1, "弃首张手牌");
    // 无手牌 → 受 1 点伤害
    let hp2 = e.debug_hp(2);
    let mut e = inject(&e, &pack, &[wy], 0);
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: wy,
            targets: vec![2],
        },
    );
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    assert_eq!(
        e.debug_hp(2),
        hp2 - 1,
        "无手牌失去 1 点体力（规则 v77 对齐上游 loseHp）"
    );
    assert!(
        !e.log()
            .iter()
            .any(|l| matches!(l, LogEvent::DamageDealt { target: 2, .. })),
        "瘟疫无手牌分支为失去体力而非伤害"
    );
    assert_zone_conservation(&e);
}

/// 置换军（反复失去体力上限）/暴雪药水（弃前两张摸 1）。
#[test]
fn gwent_spells_zhihuan_baoxue() {
    let (e0, pack) = setup(5303);
    let find = |e: &Engine, prefix: &str| {
        e.catalog()
            .cards
            .iter()
            .position(|c| c.name.starts_with(prefix))
            .unwrap_or_else(|| panic!("{}在目录", prefix)) as u32
    };
    let sha_of = |e: &Engine, suit: Suit| {
        e.catalog()
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Sha && c.suit == suit)
            .expect("杀在目录") as u32
    };

    // 置换军：目标已受伤且手牌 < 上限 → 反复 -1 上限
    let zhj = find(&e0, "gw_zhihuanjun");
    let mut e = inject(&e0, &pack, &[zhj], 0);
    e.debug_set_hp(1, 2);
    e.debug_set_hp(2, 4);
    let max1 = e.save().core.players[1].max_hp;
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: zhj,
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
    let new_max = e.save().core.players[1].max_hp;
    assert!(new_max < max1, "体力上限下降: {max1} → {new_max}");
    // 目标未受伤（hp==max）→ 不再下降
    let hp1 = e.debug_hp(1);
    assert_eq!(new_max as i32, hp1, "循环终止于未受伤（hp==max）");
    assert_zone_conservation(&e);

    // 暴雪药水：目标自选两张手牌弃置后摸 1（逐技对照批五 v79 对齐上游
    // chooseToDiscard("h",2,true)——选 s2、s3 保留 s1 验证自选语义）
    let bx = find(&e0, "gw_baoxueyaoshui");
    let s1 = sha_of(&e0, Suit::Spade);
    let s2 = sha_of(&e0, Suit::Club);
    let s3 = sha_of(&e0, Suit::Heart);
    let e = inject(&e0, &pack, &[bx], 0);
    let mut e = inject(&e, &pack, &[s1, s2, s3], 2);
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: bx,
            targets: vec![2],
        },
    );
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    let c = e.open_choice().expect("暴雪目标选牌挂起");
    assert_eq!((c.spec.prompt, c.observer), ("gw_baoxue_discard", 2));
    assert_eq!((c.spec.min, c.spec.max), (2, 2), "强制选两张");
    let cid = c.id;
    submit(
        &mut e,
        2,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(s2), ChoiceOption::Card(s3)],
        },
    );
    let h2 = e.debug_hand(2).len();
    // 弃所选两张 + 摸 1：4 -1(打出) -2(弃) +1(摸) = 2
    assert_eq!(h2, 2, "弃 2 摸 1 后剩 2 张");
    assert!(
        !e.debug_hand(2).contains(&s2) && !e.debug_hand(2).contains(&s3),
        "所选两张被弃"
    );
    assert!(e.debug_hand(2).contains(&s1), "未选的第一张保留");
    assert_zone_conservation(&e);
}

/// 天气状态群：倾盆大雨（手牌上限 -1 至弃牌阶段结束）/碧日浓雾
/// （不能使用杀至出牌阶段结束）/刺骨寒霜（摸牌阶段结束弃首张）。
/// 目标+距离 1 邻座（3 人局全员），已持有者跳过。
#[test]
fn gwent_weather_states() {
    let (e0, pack) = setup(5304);
    let find = |e: &Engine, prefix: &str| {
        e.catalog()
            .cards
            .iter()
            .position(|c| c.name.starts_with(prefix))
            .unwrap_or_else(|| panic!("{}在目录", prefix)) as u32
    };

    // —— 倾盆大雨：手牌上限 -1 ——
    let qpd = find(&e0, "gw_qinpendayu");
    let mut e = inject(&e0, &pack, &[qpd], 0);
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: qpd,
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
    // 3 人局全员为邻座 → 全员获天气
    for _p in 0..3 {
        let hand: Vec<u32> = (0..3).map(|_| 0).collect();
        let _ = hand;
    }
    // 手牌上限 = hp - 1：给 0 号 3 点体力 → 上限 2
    e.debug_set_hp(0, 3);
    // 推进到弃牌阶段前手牌为 0（无弃牌），验证状态清除时机
    // （状态清除于 Discard end；此处仅验证 hook 生效需手牌超限——
    //  简化为直接确认阶段结束后状态移除）
    eprintln!(
        "DBG after-play skills={:?}",
        e.save().core.skills.keys().collect::<Vec<_>>()
    );
    submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    if e.open_choice().is_some() {
        let c = e.open_choice().unwrap();
        let cid = c.id;
        let sel: Vec<ChoiceOption> = c
            .spec
            .options
            .iter()
            .take(c.spec.min.max(1))
            .copied()
            .collect();
        submit(
            &mut e,
            0,
            GameAction::Answer {
                choice_id: cid,
                selection: sel,
            },
        );
    }
    eprintln!(
        "DBG before-discard-end skills={:?}",
        e.save().core.skills.keys().collect::<Vec<_>>()
    );
    submit(&mut e, 0, GameAction::NextPhase); // Discard→End（Discard end 触发移除）
    eprintln!(
        "DBG after-discard-end skills={:?}",
        e.save().core.skills.keys().collect::<Vec<_>>()
    );
    let qpen_def = e
        .catalog()
        .skill_by_name(engine_native::content::host::GW_QINPEN_STATE_KEY)
        .expect("倾盆大雨状态技在目录");
    assert!(
        e.save().core.skills.get(&(0, qpen_def)).is_none(),
        "弃牌阶段结束后状态移除"
    );
    assert_zone_conservation(&e);

    // —— 碧日浓雾：不能使用杀 ——
    let (mut e, pack) = setup(5305);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let brw = find(&e, "gw_birinongwu");
    let sha = {
        let catalog = e.catalog();
        catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Sha && c.suit == Suit::Spade)
            .expect("黑杀在目录") as u32
    };
    let mut e = inject(&e, &pack, &[brw, sha], 0);
    e.debug_set_hp(0, 3);
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: brw,
            targets: vec![1],
        },
    );
    if let CommandOutcome::Rejected(r) = &out {
        panic!("碧日浓雾被拒: {r:?}");
    }
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    // 自己（0 号）也被邻座覆盖 → 打杀被拒
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::IllegalTargets)),
        "浓雾下使用杀应被拒: {out:?}"
    );
    // 出牌阶段结束 → 状态移除 → 可用
    submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    if e.open_choice().is_some() {
        let c = e.open_choice().unwrap();
        let cid = c.id;
        let sel: Vec<ChoiceOption> = c
            .spec
            .options
            .iter()
            .take(c.spec.min.max(1))
            .copied()
            .collect();
        submit(
            &mut e,
            0,
            GameAction::Answer {
                choice_id: cid,
                selection: sel,
            },
        );
    }
    // 回到能出杀的时点需再入自己的出牌阶段——此处仅验证雾已清除：
    let brin_def = e
        .catalog()
        .skill_by_name(engine_native::content::host::GW_BIRIN_STATE_KEY)
        .expect("碧日浓雾状态技在目录");
    assert!(
        e.save().core.skills.get(&(0, brin_def)).is_none(),
        "出牌阶段结束后状态移除"
    );
    assert_zone_conservation(&e);

    // —— 刺骨寒霜：摸牌阶段**开始**时摸牌数 -1（上游 `phaseDrawBegin` + `trigger.num--`） ——
    let (e0, pack) = setup(5306);
    let cgh = find(&e0, "gw_ciguhanshuang");
    let mut e = inject(&e0, &pack, &[cgh], 0);
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: cgh,
            targets: vec![1],
        },
    );
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    // 0 号自己也有霜（邻座覆盖）→ 0 号下个摸牌阶段少摸 1：
    // 推进一个整回合，停在 0 号**判定阶段**（此时状态还在）
    let mut steps = 0;
    loop {
        steps += 1;
        assert!(steps < 96, "未回到 0 号摸牌阶段后");
        if e.turn().current == 0 && e.turn().phase == Phase::Judge && e.open_choice().is_none() {
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
            submit(
                &mut e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: sel,
                },
            );
            continue;
        }
        let actor = e.turn().current;
        submit(&mut e, actor, GameAction::NextPhase);
    }
    let h0 = e.debug_hand(0).len();
    let cigu_def = e
        .catalog()
        .skill_by_name(engine_native::content::host::GW_CIGU_STATE_KEY)
        .expect("刺骨寒霜状态技在目录");
    assert!(
        e.save().core.skills.contains_key(&(0, cigu_def)),
        "判定阶段时状态还在"
    );
    submit(&mut e, 0, GameAction::NextPhase); // Judge→Draw：开始即少摸 1
    assert_eq!(
        e.debug_hand(0).len(),
        h0 + 1,
        "摸牌阶段开始：摸牌数 2→1（只摸 1 张）"
    );
    assert!(
        e.save().core.skills.get(&(0, cigu_def)).is_none(),
        "一次性状态移除"
    );
    assert_zone_conservation(&e);
}

/// gwent 收官：昆恩法印（防普通伤害，按次数计数）/山崩术（弃其他
/// 角色前两件装备）/破晓（清除天气+判定区 或 获得牌堆首张天气）。
#[test]
fn gwent_closeout_kunen_shanbeng_poxiao() {
    let (e0, pack) = setup(5307);
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

    // —— 昆恩法印：防普通伤害 + 计数递减 ——
    let knf = find(&e0, "gw_kunenfayin");
    let sha = card_of(&e0, CardKind::Sha, Suit::Spade);
    let e = inject(&e0, &pack, &[knf], 0);
    let mut e = inject(&e, &pack, &[sha], 1);
    e.debug_set_hp(1, 9);
    // 0 号出牌阶段使用昆恩法印（自-target）→ 获得状态
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: knf,
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
    let knf_def = e
        .catalog()
        .skill_by_name(engine_native::content::host::GW_KUNEN_STATE_KEY)
        .expect("昆恩状态技在目录");
    assert!(
        e.save().core.skills.contains_key(&(0, knf_def)),
        "昆恩状态已授予"
    );
    // 1 号出杀打 0 号 → 昆恩防止
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
            submit(
                &mut e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: sel,
                },
            );
            continue;
        }
        let actor = e.turn().current;
        submit(&mut e, actor, GameAction::NextPhase);
    }
    let hp0 = e.debug_hp(0);
    let out = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha,
            targets: vec![0],
        },
    );
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    while let Some(c) = e.open_choice() {
        if c.spec.prompt != "respond_shan" {
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
            continue;
        }
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
    assert_eq!(e.debug_hp(0), hp0, "昆恩防止普通伤害");
    let knf_def = e
        .catalog()
        .skill_by_name(engine_native::content::host::GW_KUNEN_STATE_KEY)
        .expect("昆恩状态技在目录");
    let rounds_after = e.save().core.skills.get(&(0, knf_def)).unwrap().rounds;
    assert!(rounds_after < 5, "计数递减");
    assert_zone_conservation(&e);

    // —— 山崩术：其他角色前两件装备被弃 ——
    let sbs = find(&e0, "gw_shanbengshu");
    let armor = {
        let catalog = e0.catalog();
        catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Armor)
            .expect("防具在目录") as u32
    };
    let armor2 = {
        let catalog = e0.catalog();
        catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Weapon)
            .expect("武器在目录") as u32
    };
    let e = inject(&e0, &pack, &[sbs], 0);
    let e = inject(&e, &pack, &[armor], 1);
    let mut e = inject(&e, &pack, &[armor2], 2);
    let _h_none = e.debug_hand(0).len();
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sbs,
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
    assert_eq!(e.debug_hand(0).len(), 0, "自身手牌不变（打出后为空）");
    assert!(e.save().core.players[1].equips.is_empty(), "1 号装备被弃");
    assert!(e.save().core.players[2].equips.is_empty(), "2 号装备被弃");
    assert_zone_conservation(&e);

    // —— 破晓分支 1：获得牌堆首张天气法术 ——
    let px = find(&e0, "gw_poxiao");
    let qpd = find(&e0, "gw_qinpendayu");
    let mut e = inject(&e0, &pack, &[px, qpd], 0);
    // v78 起分支 1 为「随机获得一张铜卡法术」——改为集合成员断言：
    // 记录出牌前牌堆中的全部铜法术，结算后手牌应获得其中之一
    let deck_bronzes: Vec<u32> = {
        let s = e.save();
        let catalog = e.catalog();
        s.core
            .deck
            .iter()
            .filter(|c| {
                matches!(
                    catalog.card_kind(**c),
                    CardKind::Qinpendayu
                        | CardKind::Birinongwu
                        | CardKind::Ciguhanshuang
                        | CardKind::Baoxueyaoshui
                )
            })
            .copied()
            .collect()
    };
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: px,
            targets: vec![],
        },
    );
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    let c = e.open_choice().expect("破晓分支挂起");
    assert_eq!(c.spec.prompt, "gw_poxiao_branch");
    let cid = c.id;
    submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Tag(1)],
        },
    );
    assert!(
        e.debug_hand(0).iter().any(|c| deck_bronzes.contains(c)),
        "获得牌堆中任一铜卡法术（随机）"
    );
    assert_zone_conservation(&e);
}

/// 祖墨手铠（封印/技能失效原语）：目标全部技能实例失效，其任意阶段
/// 结束解除并移除状态——以倾盆大雨状态作为可观察技能验证禁用/恢复。
#[test]
fn gwent_fengyin_disables_target_skills() {
    let (e0, pack) = setup(5308);
    let find = |e: &Engine, prefix: &str| {
        e.catalog()
            .cards
            .iter()
            .position(|c| c.name.starts_with(prefix))
            .unwrap_or_else(|| panic!("{}在目录", prefix)) as u32
    };

    // 先给 1 号挂倾盆大雨状态（可观察的技能实例）
    let qpd = find(&e0, "gw_qinpendayu");
    let mut e = inject(&e0, &pack, &[qpd], 0);
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: qpd,
            targets: vec![1],
        },
    );
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    let qpen_def = e
        .catalog()
        .skill_by_name(engine_native::content::host::GW_QINPEN_STATE_KEY)
        .expect("倾盆大雨状态技在目录");
    let fyd_def = e
        .catalog()
        .skill_by_name(engine_native::content::host::GW_FENGYIN_STATE_KEY)
        .expect("封印状态技在目录");
    let zumo = find(&e, "gw_zumoshoukao");
    let mut e = inject(&e, &pack, &[zumo], 0);
    // 祖墨手铠 → 1 号封印：倾盆大雨实例失效 + 封印实例存在
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: zumo,
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
    let qpen_inst = e.save().core.skills.get(&(1, qpen_def)).cloned();
    let fyd_inst = e.save().core.skills.get(&(1, fyd_def)).cloned();
    assert_eq!(
        qpen_inst.as_ref().map(|i| i.enabled),
        Some(false),
        "倾盆大雨被禁用"
    );
    assert_eq!(
        fyd_inst.as_ref().map(|i| i.enabled),
        Some(true),
        "封印自身保持启用"
    );
    // 推进到 1 号的任意阶段结束（Draw end）→ 封印解除
    let mut steps = 0;
    loop {
        steps += 1;
        assert!(steps < 96, "未到达 1 号摸牌阶段后");
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
            submit(
                &mut e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: sel,
                },
            );
            continue;
        }
        let actor = e.turn().current;
        submit(&mut e, actor, GameAction::NextPhase);
    }
    let h_after = e.debug_hand(1).len();
    submit(&mut e, 1, GameAction::NextPhase); // Draw→Judge（Draw end 触发解除）
    let qpen_inst2 = e.save().core.skills.get(&(1, qpen_def)).cloned();
    assert_eq!(
        qpen_inst2.as_ref().map(|i| i.enabled),
        Some(true),
        "封印解除后倾盆大雨重新启用"
    );
    assert!(
        e.save().core.skills.get(&(1, fyd_def)).is_none(),
        "封印实例自身移除"
    );
    // 倾盆大雨仍在（未到期，仅被解禁）——手牌数不变仅验证状态
    let _ = h_after;
    assert_zone_conservation(&e);
}

/// 整包验收（2026-10-08，第二包）：shahai-gwent 真实宿主全生命周期——
/// 独立安装+激活 → 安装包→原生建局 → 对局中卸载 → 在局锁定副本续玩 →
/// 撤销后重装/重新 import 被拒。
#[test]
fn gwent_real_host_full_lifecycle() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();

    // ① 独立安装 + 激活
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "shahai-gwent")
        .expect("gwent 安装")
        .clone();
    let slot = format!("{}/{}", meta.namespace, meta.package_id);
    assert!(
        store.read_active().unwrap().packages.contains_key(&slot),
        "激活集"
    );

    // ② 安装包 → 原生建局（昆恩/山崩/破晓等 21 卡目录）
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-gwent-upstream-noname")
        .expect("gwent 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let (pack, _) = bridge("shahai-gwent-upstream-noname", &json).expect("桥接");
    let base_cards = Engine::new(1, 3).catalog().cards.len();
    let mut e = Engine::new_with_extensions(5401, 3, &[&pack]).expect("建局");
    assert!(e.catalog().cards.len() > base_cards, "gwent 卡入目录");
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let mid = e.save();

    // ③ 对局中卸载
    store
        .revoke_slot(&meta.namespace, "shahai-gwent", "验收：对局中卸载", true)
        .expect("撤销");
    assert!(store
        .read_revoke()
        .unwrap()
        .is_slot_revoked(&meta.namespace, "shahai-gwent"));
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

    // ⑤ 重装守卫：撤销后同 npack 再安装被拒
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
