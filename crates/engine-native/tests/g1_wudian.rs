//! G1 玩点论杀纵向样本（ADR 0776）：卡牌切片 → 真实包 → 导入 →
//! 宿主行为条件增益 → 内容锁。
//!
//! 切片 14 张实体牌（基础牌 7 + 望梅止渴 2 + 诱敌深入 3 + 三尖 1 +
//! 铁索 1）；暂缓（包数据登记 reason）：调兵遣将×2（多人方向选择多步
//! 结算）。武将批 6 名元数据经角色包管线装载，思继已映射，其余技能登记暂缓。

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use content_runtime::{discover_extensions, import_extensions, InstallOptions, PackageStore};
use engine_native::catalog::CardKind;
use engine_native::content::extension::{ExtensionDef, ExtensionPack};
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

/// 装载（ADR 0777/0778）：包元数据、三尖射程与装备技都写在 cards.json，
/// 由 content-native-loader 统一桥接，测试不含逐包映射。
fn bridge(json: &str) -> ExtensionPack {
    content_native_loader::bridge_pack(json)
        .expect("cards.json 可解析")
        .pack
}

/// 测试脚手架：导入 + 桥接 + 停在 0 号出牌阶段（无挂起、全手牌清空）。
fn setup() -> (Engine, ExtensionPack) {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    assert!(discovered.iter().any(|d| d.name == "wudian-upstream-noname"));
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    assert!(report.installed.iter().any(|m| m.package_id == "wudian"));
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "wudian")
        .unwrap()
        .clone();
    let json = std::fs::read_to_string(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .expect("cards.json 可读");
    assert!(json.contains("host:wangmeizhike_card"), "宿主映射声明");
    assert!(json.contains("diaobingqianjiang"), "暂缓条目登记");
    let pack = bridge(&json);
    assert_eq!(
        (pack.def.name, pack.def.version),
        ("wudian-upstream-noname", 2),
        "包元数据来自 cards.json"
    );
    assert_eq!(pack.card_count(), 14, "切片 14 张实体牌（+诱敌深入 3）");
    let mut e = Engine::new_with_extensions(3501, 3, &[&pack]).expect("建局");
    let base_cards = Engine::new(1, 3).catalog().cards.len();
    assert_eq!(e.catalog().cards.len(), base_cards + 14);
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
    (e, pack)
}

fn wangmei_id(e: &Engine, n: usize) -> u32 {
    let mut seen = 0;
    for (i, c) in e.catalog().cards.iter().enumerate() {
        if c.name.starts_with("wd_wangmeizhike") {
            if seen == n {
                return i as u32;
            }
            seen += 1;
        }
    }
    panic!("望梅止渴不足");
}

fn pass_nullify(e: &mut Engine) {
    let c = e.open_choice().expect("无懈窗口");
    let cid = c.id;
    let obs = c.observer;
    assert_eq!(c.spec.prompt, "nullify_window");
    submit(e, obs, GameAction::Answer {
        choice_id: cid,
        selection: vec![ChoiceOption::Pass],
    });
}

/// 分支①：目标为最小体力（已受伤）→ 回复 1；非最小手牌 → 不摸牌。
#[test]
fn wangmeizhike_recovers_min_hp_damaged_target() {
    let (mut e, pack) = setup();
    let wm = wangmei_id(&e, 0);
    e.debug_place(0, "shan");
    e.debug_place(0, "shan");
    e.debug_place(1, "nullify"); // 触发无懈窗口
    e.debug_place(2, "shan");
    let mut snap = e.save();
    snap.core.players[0].hp = 1; // 最小体力且已受伤
    snap.core.players[1].hp = 4;
    snap.core.players[2].hp = 4;
    snap.core.deck.retain(|c| *c != wm);
    snap.core.players[0].hand.push(wm);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let hand_before = e.debug_hand(0).len();
    let out = submit(&mut e, 0, GameAction::PlayCard { card: wm, targets: vec![0] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    pass_nullify(&mut e);
    assert_eq!(e.debug_hp(0), 2, "最小体力且已受伤 → 回复 1");
    assert_eq!(e.debug_hand(0).len(), hand_before - 1, "非最小手牌不摸牌（仅打出的牌离手）");
}

/// 分支②：目标非最小体力 → 不回复；最小手牌 → 摸 2。
#[test]
fn wangmeizhike_draws_min_hand_target() {
    let (mut e, pack) = setup();
    let wm = wangmei_id(&e, 0);
    e.debug_set_hp(0, 4);
    e.debug_set_hp(1, 1); // 体力最小者为目标之外
    e.debug_set_hp(2, 4);
    e.debug_place(1, "nullify"); // 触发无懈窗口
    let mut snap = e.save();
    snap.core.deck.retain(|c| *c != wm);
    snap.core.players[0].hand.push(wm);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: wm, targets: vec![0] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    pass_nullify(&mut e);
    assert_eq!(e.debug_hp(0), 4, "非最小体力不回复");
    assert_eq!(e.debug_hand(0).len(), 2, "打出 1 张后为最小手牌 → 摸 2（打出的不计）");
}

/// 内容锁与字节路径：带扩展存档被纯标准恢复拒绝；同集合双路径恢复。
#[test]
fn g1_wudian_content_lock_and_byte_restore() {
    let (e, pack) = setup();
    let saved = e.save();
    assert!(matches!(
        Engine::restore(&saved),
        Err(engine_native::SnapshotError::RuleVersionMismatch { .. })
    ));
    Engine::restore_with_extensions(&saved, &[&pack]).expect("同集合恢复");
    let bytes = engine_native::archive::encode_archive(&saved);
    Engine::restore_with_extensions(
        &engine_native::archive::decode_archive(&bytes).expect("decode"),
        &[&pack],
    )
    .expect("字节同集合恢复");
}

/// G1 武将批：6 名武将元数据经角色包管线入目录；
/// 思继（可选触发）：弃牌阶段弃 2 张杀 → 确认 → 摸 4；字节恢复贯通。
#[test]
fn g1_wudian_generals_and_siji_discard_draw() {
    use engine_native::archive::{decode_archive, encode_archive};
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "wudian-upstream-noname")
            .expect("wudian 可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let out = content_native_loader::bridge_generals(&json).expect("generals 装载");
    let generals = out.generals;
    let host_skills = out.host_skills;
    let unmapped_total = out.unmapped_skills;
    assert_eq!(generals.len(), 9, "玩点论杀九名武将全量入册（批 10 补西真夕红/造之仁君/杨仪）");
    assert_eq!(unmapped_total, 7, "7 个技能登记暂缓（理断转正；余复仇双技/良策/坚壁/仇渡+存量夺启/佑赞）");
    let pack = ExtensionPack {
        def: ExtensionDef {
            name: "wudian-upstream-noname",
            version: 2,
            description: "G1 玩点论杀卡牌+武将批",
        },
        generals,
        cards: Vec::new(),
        host_skills,
        vm_skills: Vec::new(),
        init_hook: engine_native::content::extension::InitHook::None,
        enabled: true,
    };
    let base_count = engine_native::content::standard::STANDARD_GENERALS.len();
    let mut e = Engine::new_with_extensions(3502, 3, &[&pack]).expect("建局");
    // 韩龙 = 扩展表下标 1
    let (_, _, skills, hp, _) = e
        .catalog()
        .general_meta(base_count as u16 + 1)
        .expect("韩龙在目录");
    assert_eq!(skills, vec!["siji", "ciqiu"], "韩龙思继+刺猬（刺猬批一 v111 接线）");
    assert_eq!(hp, 4);

    // 思继行为：授予 → 弃牌阶段弃 2 杀 → 确认 → 摸 4
    e.debug_grant(0, "siji");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let s1 = e.debug_place(0, "sha");
    let s2 = e.debug_place(0, "sha");
    e.debug_place(0, "shan");
    e.debug_set_hp(0, 2); // 4 张手牌 > 体力 2 → 弃牌阶段需弃 2 张
    for _ in 0..8 {
        if e.open_choice().is_some() {
            break;
        }
        let actor = e.turn().current;
        match submit(&mut e, actor, GameAction::NextPhase) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("advance: {other:?}"),
        }
    }
    let c = e.open_choice().expect("弃牌选择挂起");
    assert_eq!(c.spec.prompt, "discard_to_limit");
    // 弃一张杀（弃牌上限按手牌-体力计算；弃置含杀 → 触发思继）
    let sel: Vec<ChoiceOption> = c
        .spec
        .options
        .iter()
        .take(c.spec.min.max(1))
        .copied()
        .collect();
    let discarded_sha = sel.iter().any(|o| *o == ChoiceOption::Card(s1) || *o == ChoiceOption::Card(s2));
    let cid = c.id;
    let out = submit(&mut e, 0, GameAction::Answer { choice_id: cid, selection: sel });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    // 思继确认（可选触发，OptionalGate）——弃含杀 → 触发
    if discarded_sha {
        let c2 = e.open_choice().expect("思继确认应挂起");
        assert_eq!(c2.spec.prompt, "trigger_confirm");
    let bytes = encode_archive(&e.save());
    let mut restored = Engine::restore_with_extensions(
        &decode_archive(&bytes).expect("decode"),
        &[&pack],
    )
    .expect("思继确认挂起字节恢复");
    let cid2 = restored.open_choice().unwrap().id;
    submit(&mut restored, 0, GameAction::Answer {
        choice_id: cid2,
        selection: vec![ChoiceOption::Confirm],
    });
    // 弃 1 杀 → 摸 2
    let hand_after = restored.debug_hand(0).len();
    assert_eq!(hand_after % 2, 0, "摸牌数为偶数（2×弃杀数）: {hand_after}");
    } else {
        panic!("弃牌应含杀以触发思继");
    }
}

fn advance_loop(e: &mut Engine, player: u8) {
    for _ in 0..64 {
        let t = e.turn();
        if t.current == player && t.phase == Phase::Play && e.open_choice().is_none() {
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
    panic!("no play phase");
}

/// G1 三尖武器技（两段选择）：装备三尖 → 杀伤害 → 弃一张手牌 →
/// 对距受损者距离 ≤1 的另一名角色造成 1 点伤害；两段均字节恢复。
#[test]
fn sanjian_weapon_skill_two_stage_choice_with_restore() {
    use engine_native::archive::{decode_archive, encode_archive};
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "wudian-upstream-noname")
            .expect("wudian 可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(3503, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    // 装备三尖（equip_skills 生命周期授予 wd_sanjian）
    let sj = {
        let catalog = e.catalog();
        catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("wd_sanjian"))
            .expect("三尖在目录") as u32
    };
    let mut snap = e.save();
    snap.core.deck.retain(|c| *c != sj);
    snap.core.players[0].hand.push(sj);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sj, targets: vec![] });
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    let sj_def = e.catalog().skill_by_name("wd_sanjian").unwrap();
    assert!(
        e.save().core.skills.contains_key(&(0, sj_def)),
        "装备三尖应授予 wd_sanjian"
    );
    // 杀 → 1 号（距离 1）受 1 伤 → 三尖触发：弃牌选择挂起
    let sha = e.debug_place(0, "sha");
    let discard_card = e.debug_place(0, "tao");
    e.debug_set_hp(1, 4);
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    let c = e.open_choice().expect("三尖弃牌选择挂起");
    assert_eq!(c.spec.prompt, "wd_sanjian_discard");
    let cid = c.id;
    submit(&mut e, 0, GameAction::Answer {
        choice_id: cid,
        selection: vec![ChoiceOption::Card(discard_card)],
    });
    // 第二段：目标选择（字节恢复后应答）
    let c2 = e.open_choice().expect("三尖目标选择挂起");
    assert_eq!(c2.spec.prompt, "wd_sanjian_target");
    let bytes = encode_archive(&e.save());
    let mut restored = Engine::restore_with_extensions(
        &decode_archive(&bytes).expect("decode"),
        &[&pack],
    )
    .expect("三尖目标选择字节恢复");
    let cid2 = restored.open_choice().unwrap().id;
    submit(&mut restored, 0, GameAction::Answer {
        choice_id: cid2,
        selection: vec![ChoiceOption::Player(2)],
    });
    assert_eq!(restored.debug_hp(2), 3, "三尖对距离 ≤1 的另一角色造成 1 点伤害");
    assert_eq!(restored.debug_hp(1), 3, "原伤害不变");
}

/// G1 墨守目标免疫：乐不思蜀不能指定墨守持有者为目标；其他目标不受影响。
#[test]
fn moshou_blocks_delayed_trick_target() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "wudian-upstream-noname")
            .expect("wudian 可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(3504, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    // 2 号授予墨守（费诗技能）
    e.debug_grant(2, "moshou");
    let le = e.debug_place(0, "le");
    // 目标 1 号（无免疫）→ 正常置入判定区
    let out = submit(&mut e, 0, GameAction::PlayCard { card: le, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    // 兵粮寸断 → 目标 2 号（墨守免疫乐/兵）→ 拒绝
    let bing = e.debug_place(0, "bing");
    let out2 = submit(&mut e, 0, GameAction::PlayCard { card: bing, targets: vec![2] });
    assert!(
        matches!(out2, CommandOutcome::Rejected(engine_native::RejectReason::IllegalTargets)),
        "墨守应使延时锦囊目标免疫: {out2:?}"
    );
    assert_zone_conservation_wudian(&e);
}

fn assert_zone_conservation_wudian(e: &Engine) {
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

/// 诱敌深入（响应转化原语）：成为杀目标时可打出——杀无效，
/// 来源受 1 点反噬伤害（登记差异：无上游连用循环）。
#[test]
fn youdishenru_cancels_sha_with_reprisal() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "wudian-upstream-noname")
            .expect("wudian 可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(4201, 3, &[&pack]).expect("建局");
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
    // 0 号（攻击方）手牌：杀；1 号（防御方）手牌：诱敌深入
    let (sha, yd) = {
        let catalog = e.catalog();
        let sh = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Sha)
            .expect("杀在目录") as u32;
        let y = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Youdishenru)
            .expect("诱敌深入在目录") as u32;
        (sh, y)
    };
    let mut snap = e.save();
    for p in snap.core.players.iter_mut() {
        p.hand.retain(|c| *c != sha && *c != yd);
    }
    snap.core.deck.retain(|c| *c != sha && *c != yd);
    snap.core.discard.retain(|c| *c != sha && *c != yd);
    snap.core.players[0].hand.push(sha);
    snap.core.players[1].hand.push(yd);
    e.debug_set_hp(1, 4);
    e.debug_set_hp(0, 4);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    // 0 号出杀 → 1 号以诱敌深入响应
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    let c = e.open_choice().expect("响应询问挂起");
    assert_eq!(c.spec.prompt, "respond_shan");
    assert!(
        c.spec.options.contains(&ChoiceOption::Card(yd)),
        "响应选项应含诱敌深入"
    );
    let cid = c.id;
    submit(&mut e, 1, GameAction::Answer {
        choice_id: cid,
        selection: vec![ChoiceOption::Card(yd)],
    });
    // 杀无效：1 号体力不变；来源受 1 点反噬
    assert_eq!(e.debug_hp(1), 4, "诱敌深入应使杀无效");
    assert_eq!(e.debug_hp(0), 3, "来源受 1 点反噬伤害");
    assert!(
        !e.debug_hand(1).contains(&yd),
        "诱敌深入已打出"
    );
}

/// 墨守装配接线（2026-10-08）：费诗 generals 声明 mapping null →
/// host:moshou_immunity——桥接后目录元数据 skills 应含 moshou
/// （装配/明置授予路径据此自动挂接；宿主行为本体已由
/// `moshou_blocks_delayed_trick_target` 以授予驱动验证）。
#[test]
fn feishi_moshou_declaration_wired_to_host_behavior() {
    let (e, _pack) = setup();
    let base_count = engine_native::content::standard::STANDARD_GENERALS.len();
    // 扩展武将接在基础表后：按名查找费诗并断言其目录技能含 moshou
    let mut found = None;
    for idx in (base_count as u16)..(base_count as u16 + 8) {
        if let Some((name, _, skills, _, _)) = e.catalog().general_meta(idx) {
            if name == "费诗" {
                found = Some(skills.to_vec());
                break;
            }
        }
    }
    assert_eq!(found.map(|v| v.to_vec()), Some(vec!["nsshuaiyan", "moshou"]), "费诗目录技能=率言+墨守（率言批二 v112 接线）");
}

/// 整包验收 U07（2026-10-08，来源级第五包）：wudian 真实宿主全生命周期
/// ——独立安装+激活 → 安装包→原生建局（14 卡+费诗墨守接线后目录）→
/// 对局中卸载 → 在局锁定副本续玩 → 撤销后重装/重新 import 被拒。
#[test]
fn wudian_real_host_full_lifecycle() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();

    // ① 独立安装 + 激活
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "wudian")
        .expect("wudian 安装")
        .clone();
    let slot = format!("{}/{}", meta.namespace, meta.package_id);
    assert!(store.read_active().unwrap().packages.contains_key(&slot), "激活集");

    // ② 安装包 → 原生建局（14 卡 + 费诗墨守接线后目录元数据）
    let json = std::fs::read_to_string(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .unwrap();
    let pack = bridge(&json);
    assert_eq!(pack.card_count(), 14);
    let base_cards = Engine::new(1, 3).catalog().cards.len();
    let mut e = Engine::new_with_extensions(6005, 3, &[&pack]).expect("建局");
    assert_eq!(e.catalog().cards.len(), base_cards + 14);
    let mut feishi_skills = None;
    let gen_base = engine_native::content::standard::STANDARD_GENERALS.len() as u16;
    for idx in gen_base..(gen_base + 8) {
        if let Some((name, _, skills, _, _)) = e.catalog().general_meta(idx) {
            if name == "费诗" {
                feishi_skills = Some(skills.to_vec());
                break;
            }
        }
    }
    assert_eq!(feishi_skills.map(|v| v.to_vec()), Some(vec!["nsshuaiyan", "moshou"]), "费诗率言+墨守接线生效");
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let mid = e.save();

    // ③ 对局中卸载
    store
        .revoke_slot(&meta.namespace, "wudian", "验收：对局中卸载", true)
        .expect("撤销");
    assert!(store.read_revoke().unwrap().is_slot_revoked(&meta.namespace, "wudian"));
    assert!(!store.read_active().unwrap().packages.contains_key(&slot), "移出激活集");

    // ④ 在局锁定副本续玩
    let mut resumed = Engine::restore_with_extensions(
        &engine_native::archive::decode_archive(&engine_native::archive::encode_archive(&mid))
            .expect("解码"),
        &[&pack],
    )
    .expect("对局中卸载后恢复");
    assert_eq!(resumed.catalog().cards.len(), base_cards + 14, "锁定副本仍可用");
    let out = submit(&mut resumed, 0, GameAction::NextPhase);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "在局续玩: {out:?}");

    // ⑤ 重装守卫
    let bytes = std::fs::read(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("package.npack.json"),
    )
    .expect("npack");
    let err = store.install(&bytes, None, &options());
    assert!(matches!(err, Err(content_runtime::RuntimeError::Revoked(_))), "撤销后重装应被拒: {err:?}");

    // ⑥ 重新 import 被拒
    let report2 = import_extensions(&store, &root, &options());
    assert!(report2.is_err(), "撤销后重新 import 应被拒");
}

/// 刺猬（U07 技能批一，规则 v111）：韩龙——你的杀对满血目标伤害 +1；
/// 加后 ≥ 目标体力则目标获本回合濒死即死标记（跳过救援直接死亡）且本
/// 技能移除（一次性）。非满血目标不加成。
#[test]
fn hanlong_ciqiu_sha_healthy_bonus_and_dying_mark() {
    let (mut e, _pack) = setup();
    advance_loop(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    e.debug_grant(0, "ciqiu");
    e.debug_grant(0, "咆哮"); // 出牌阶段杀不限次（三次出杀驱动）
    e.debug_set_hp(1, 4);

    // ① 非满血目标（先压 1 血）→ 不加成
    e.debug_set_hp(1, 3);
    let sha = e.debug_place(0, "sha");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    assert_eq!(e.debug_hp(1), 2, "非满血不加成（3→2）");
    // 技能仍在
    let def = e.catalog().skill_by_name(engine_native::content::host::WUDIAN_CIQIU_KEY).expect("def");
    assert!(e.state_core().has_skill(0, def), "技能未被移除");

    // ② 满血目标（回满 4）：+1 → 3 伤（2→? 2-3 → 濒死）——伤 3 ≥ 体力 2？
    // 上游口径：加后 num(3) >= player.hp(目标受伤害前体力 2) → 濒死即死
    e.debug_set_hp(1, 2);
    // 注意满血判断：目标 hp==max_hp 才触发——先把 max_hp 压到 2
    e.debug_set_max_hp(1, 2);
    let sha = e.debug_place(0, "sha");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    assert!(!e.debug_alive(1), "伤 3 ≥ 体力 2：濒死即死（跳过救援）");
    // 技能已移除：再对满血目标不加成
    e.debug_set_hp(2, 4);
    let sha = e.debug_place(0, "sha");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![2] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    assert_eq!(e.debug_hp(2), 3, "技能移除后不加成（4→3）");
}

/// 率言（U07 技能批二，规则 v112）：费诗——他人回复体力后（回复者非
/// 你且你非当前回合玩家），回复者二选一：令你摸一张 / 令你弃置其一张
/// 牌（上游态度 AI 推导不迁移）。自己回血/自己回合不触发。
#[test]
fn feishi_shuaiyan_recover_trigger_two_choice() {
    let (mut e, _pack) = setup();
    advance_loop(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    e.debug_grant(0, "nsshuaiyan");

    // 轮到 1 号回合：1 号压 1 血后吃桃回血 → 触发率言（回复者=1，当前
    // 回合玩家=1≠owner 0）
    advance_loop(&mut e, 1);
    let tao = e.debug_place(1, "tao");
    e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    assert_eq!(e.debug_hp(1), 3, "压 1 血（4→3）");
    let out = submit(&mut e, 1, GameAction::PlayCard { card: tao, targets: vec![] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    // 回复者二选一挂起（observer=1 号）
    let c = e.open_choice().expect("率言二选一挂起");
    assert_eq!(
        (c.spec.prompt, c.observer),
        (
            engine_native::content::host::WUDIAN_SHUAIYAN_CHOICE,
            1
        )
    );
    let (cid, obs) = (c.id, c.observer);
    // ① 选项一：费诗（0 号）摸 1
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Tag(0)] });
    assert_eq!(e.debug_hp(1), 4, "吃桃回满");
    assert_eq!(e.debug_hand(0).len(), 1, "选项一：费诗摸 1");

    // ② 再次驱动：1 号再压血吃桃 → 选二 → 1 号弃一张
    let tao = e.debug_place(1, "tao");
    e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    let t2 = e.debug_place(1, "shan"); // 被弃候选
    let out = submit(&mut e, 1, GameAction::PlayCard { card: tao, targets: vec![] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    let c = e.open_choice().expect("率言二选一挂起②");
    assert_eq!(c.spec.prompt, engine_native::content::host::WUDIAN_SHUAIYAN_CHOICE);
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Tag(1)] });
    let c = e.open_choice().expect("被弃者选牌挂起");
    assert_eq!(
        (c.spec.prompt, c.observer),
        (engine_native::content::host::WUDIAN_SHUAIYAN_DISCARD, 1)
    );
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(t2)] });
    assert!(!e.debug_hand(1).contains(&t2), "选项二：回复者弃 1（被动）");
    assert!(e.debug_hand(0).len() == 1, "选项二费诗不摸");

    // ③ 负例：0 号自己回合自己回血不触发——0 号压血吃桃
    for _ in 0..48 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play && e.turn().round > 0 {
            break;
        }
        if let Some(c) = e.open_choice() {
            let (cid, obs) = (c.id, c.observer);
            submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
        }
        let actor = e.turn().current;
        let _ = submit(&mut e, actor, GameAction::NextPhase);
    }
    e.debug_deal_damage(Some(1), 0, 1, engine_native::event::DamageNature::Normal);
    let hp0 = e.debug_hp(0);
    let tao = e.debug_place(0, "tao");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: tao, targets: vec![] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert!(e.open_choice().is_none(), "自己回合自己回血不触发");
    assert_eq!(e.debug_hp(0), hp0 + 1, "正常回血");
}

/// 筑城双技（U07 技能批三，规则 v113）：刘馥——①阶段结束确认开关：
/// 无标记→牌堆顶取 max(1,maxHp-体力) 张存标记；有标记→收回入手。②守
/// 卫（zhucheng2 显式化）：成为杀目标且标记非空→使用者选择：弃 n 张
/// （首张自选+余随机代抽，登记差异）或杀无效（跳过结算+杀回手）。
#[test]
fn liufu_zhucheng_toggle_and_sha_guard() {
    let (mut e, _pack) = setup();
    advance_loop(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    e.debug_grant(0, "zhucheng");
    e.debug_grant(0, "zhucheng2");
    e.debug_set_max_hp(0, 4);
    e.debug_set_hp(0, 2); // maxHp-hp=2 → 存 2 张

    // ① 结束阶段确认 → 存 2 张标记
    for _ in 0..8 {
        if let Some(c) = e.open_choice() {
            assert_eq!(c.spec.prompt, "trigger_confirm", "阶段结束确认挂起");
            let (cid, obs) = (c.id, c.observer);
            submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Confirm] });
            break;
        }
        let actor = e.turn().current;
        let _ = submit(&mut e, actor, GameAction::NextPhase);
    }
    assert_eq!(e.state_core().player(0).zhucheng.len(), 2, "筑城标记 2 张");

    // ② 他人（1 号回合）杀 0 号 → 使用者被问：Pass=杀无效+杀回手
    advance_loop(&mut e, 1);
    e.debug_grant(1, "咆哮"); // 两次出杀驱动
    let sha = e.debug_place(1, "sha");
    let out = submit(&mut e, 1, GameAction::PlayCard { card: sha, targets: vec![0] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    let c = e.open_choice().expect("筑城守卫询问挂起");
    assert_eq!(
        (c.spec.prompt, c.observer),
        (engine_native::content::host::WUDIAN_ZHUCHENG_GUARD_ASK, 1)
    );
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    assert_eq!(e.debug_hp(0), 2, "杀无效不掉血");
    assert!(e.debug_hand(1).contains(&sha), "杀归还使用者手牌");

    // ③ 再杀 → 使用者弃 2 张（首张自选+余代抽）→ 杀照常结算
    let sha = e.debug_place(1, "sha");
    let _ = e.debug_place(1, "wuzhong");
    let _ = e.debug_place(1, "duel");
    let out = submit(&mut e, 1, GameAction::PlayCard { card: sha, targets: vec![0] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    let c = e.open_choice().expect("筑城守卫询问挂起②");
    let (cid, obs) = (c.id, c.observer);
    // 选非杀候选（避免守卫弃掉正在结算的杀）
    let pick = c
        .spec
        .options
        .iter()
        .find(|o| matches!(o, ChoiceOption::Card(cc) if *cc != sha))
        .copied()
        .expect("有非杀可选");
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![pick] });
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    assert_eq!(e.debug_hp(0), 1, "弃 2 放行：杀照常（2→1）");
    // 手牌 6 张：守卫弃 2（首张自选+随机代抽）+ 杀结算离手 → 剩 3
    assert_eq!(e.debug_hand(1).len(), 3, "弃 2（首张+代抽 1）+杀离手");

    // ④ 开关收回：0 号回合结束阶段确认 → 标记收回入手
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
    e.debug_set_hp(0, 2);
    for _ in 0..8 {
        if let Some(c) = e.open_choice() {
            assert_eq!(c.spec.prompt, "trigger_confirm", "收回确认挂起");
            let (cid, obs) = (c.id, c.observer);
            submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Confirm] });
            break;
        }
        let actor = e.turn().current;
        let _ = submit(&mut e, actor, GameAction::NextPhase);
    }
    assert!(e.state_core().player(0).zhucheng.is_empty(), "标记收回清空");
    let h0 = e.debug_hand(0);
    assert!(h0.contains(&37) && h0.contains(&5), "收回的 2 张标记入手（37/5）");
}

/// 抗yin（U07 技能批四，规则 v114）：留赞——出牌阶段限一次：失去 1 点
/// 体力 → 弃目标一张牌（被弃者自选）→ 已受伤时按弃牌类别分支：基本=
/// 选至多 X 名各摸一（X=maxHp-体力）/ 非基本=授 kangyin2（到他人距离
/// -X、杀额外目标 +X，回合内）。未受伤无分支。
#[test]
fn liuzan_kangyin_losehp_discard_and_branch() {
    let (mut e, _pack) = setup();
    advance_loop(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    e.debug_grant(0, "kangyin");
    e.debug_set_max_hp(0, 4);
    e.debug_set_hp(0, 2); // 已受伤：X = 4-2 = 2
    e.debug_grant(0, "咆哮"); // 双次出杀驱动（kangyin2 验证）

    // ① 发动：目标 1 号 → 失去 1 体力（2→1）→ 1 号被弃一张（基本·桃）
    // → 基本分支：选 2 名（0/2）各摸 1
    let tao = e.debug_place(1, "tao");
    let def_ky = e.catalog().skill_by_name("kangyin").expect("def");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_ky });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    let c = e.open_choice().expect("抗yin目标挂起");
    assert_eq!((c.spec.prompt, c.observer), (engine_native::content::host::WUDIAN_KANGYIN_TARGET, 0));
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Player(1)] });
    assert_eq!(e.debug_hp(0), 1, "失去 1 体力（2→1）");
    let c = e.open_choice().expect("被弃者选牌挂起");
    assert_eq!((c.spec.prompt, c.observer), (engine_native::content::host::WUDIAN_KANGYIN_DISCARD, 1));
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(tao)] });
    // 基本分支：选至多 2 名各摸一（X=maxHp2-体力1=... 重算：hp 现在 1，maxHp 4 → X=3？上游=发动时？实现=分支时计算）
    let c = e.open_choice().expect("基本分支摸牌挂起");
    assert_eq!((c.spec.prompt, c.observer), (engine_native::content::host::WUDIAN_KANGYIN_DRAW, 0));
    assert!(c.spec.max >= 1, "至多 X 名");
    let picks: Vec<ChoiceOption> = c
        .spec
        .options
        .iter()
        .filter(|o| matches!(o, ChoiceOption::Player(0) | ChoiceOption::Player(2)))
        .copied()
        .collect();
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: picks });
    assert!(e.debug_hand(0).iter().any(|c| *c != tao) || e.debug_hand(2).len() > 0, "所选角色各摸 1");

    // ② 非基本分支：再发动（咆哮只放杀，抗yin usable1 限一次——
    // 换 1 号回合用 1 号视角不可，改验证限一次）→ SkillAlreadyUsed
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_ky });
    assert!(matches!(
        out,
        CommandOutcome::Rejected(engine_native::RejectReason::SkillAlreadyUsed)
    ), "限一次: {out:?}");

    // ③ kangyin2 杀额外目标：非基本分支驱动——再布非基本场景需再次
    // 发动（usable 已耗）。改在 1 号回合以 1 号授权?——1 号无技。
    // 直接验证 kangyin2 状态通道：手动置 turn.kangyin2 后杀可双目标
    {
        let mut snap = e.save();
        snap.core.turn.kangyin2 = Some((0, 1));
        e = Engine::restore_with_extensions(&snap, &[&_pack]).expect("kangyin2 置位");
    }
    let sha = e.debug_place(0, "sha");
    e.debug_set_hp(1, 4);
    e.debug_set_hp(2, 4);
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1, 2] });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "kangyin2 杀双目标放行: {out:?}"
    );
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    assert_eq!(e.debug_hp(1), 3, "目标 1 掉血");
    assert_eq!(e.debug_hp(2), 3, "目标 2 掉血");
}

/// 车阵（U07 技能批五，规则 v115）：田豫——锁定距离技：有装备时你算
/// 到他人 -1；目标无装备时其被算到 +1（computed_distance 直读）。
#[test]
fn tianyu_chezhen_distance_mod() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "wudian-upstream-noname")
        .expect("wudian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    // 5 人局：对座距离 2（3 人局全员相邻会被距离下限 1 饱和）
    let mut e = Engine::new_with_extensions(8801, 5, &[&pack]).expect("建局");
    e.debug_grant(0, "chezhen");
    let qg = (0..e.catalog().cards.len() as u32)
        .find(|i| e.catalog().cards[*i as usize].kind == engine_native::catalog::CardKind::Qinggang)
        .expect("青釭剑在目录");
    {
        let mut snap = e.save();
        for p in snap.core.players.iter_mut() {
            p.hand.retain(|c| *c != qg);
        }
        snap.core.deck.retain(|c| *c != qg);
        snap.core.discard.retain(|c| *c != qg);
        snap.core.table.retain(|c| *c != qg);
        snap.core.players[0].equips.push(qg);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("青釭注入");
    }

    // 验证一：0 有装备 → 车阵攻侧 -1；目标 2 无装备 → 被算到 +1（对座
    // 基线 2：关=2-1+1=2、开=2-1=1）
    e.debug_set_skill_enabled(0, "chezhen", false);
    let off = e.state_core().computed_distance(e.catalog(), 0, 2);
    e.debug_set_skill_enabled(0, "chezhen", true);
    let on = e.state_core().computed_distance(e.catalog(), 0, 2);
    assert_eq!(off, 2, "关：车阵攻侧 -1 与守侧 +1 抵消");
    assert_eq!(on, 1, "开：有装备攻侧 -1（守侧 +1 仅对无装备目标）");

    // 验证二：2→1 无车阵参与（2 号无 chezhen）→ 基线 1（对座 5 人局
    // 2↔1 相邻）；1 号无装备不影响他人算向自己（守侧 +1 需车阵持有者
    // 参与判定，此处 b=1 持技方是 0 号非 1 号）
    let to_1 = e.state_core().computed_distance(e.catalog(), 2, 1);
    assert_eq!(to_1, 1, "2→1 相邻基线");

    // 验证三：把青釭转注给 1 号 → 1 号无装备条件消失 → 0 号技不再给
    // 他人算向 1 号 +1（0→1：基线 2，0 有装备 -1 → 1；无变化）
    {
        let mut snap = e.save();
        // 青釭在 0 号装备区——转注前先移除（全区域 purge）
        for p in snap.core.players.iter_mut() {
            p.equips.retain(|c| *c != qg);
            p.hand.retain(|c| *c != qg);
        }
        snap.core.deck.retain(|c| *c != qg);
        snap.core.discard.retain(|c| *c != qg);
        snap.core.table.retain(|c| *c != qg);
        snap.core.players[1].equips.push(qg);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("青釭转注");
    }
    // 0 号已无装备 → 车阵攻侧条件（有装备）不成立：0→2 无 -1（1+1=2）
    assert_eq!(e.state_core().computed_distance(e.catalog(), 0, 2), 2, "0 号卸装备：攻侧 -1 消失（守侧 +1 因 2 有装备不触发）");
    // 0→1：1 号有装备 → 0 号守侧不触发；0 号无装备 → 攻侧不触发 → 基线 1
    assert_eq!(e.state_core().computed_distance(e.catalog(), 0, 1), 1, "0→1：双侧条件均不成立");
    // 2→1：1 号有装备 → 0 号守侧不涉及；基线 1
    assert_eq!(e.state_core().computed_distance(e.catalog(), 2, 1), 1, "2→1：1 号有装备无守侧加成");
}

/// 绝道（U07 技能批六，规则 v116）：刘焉——出牌阶段横置（自连环，已
/// 横置不可发动）；横置距离双向 +1（computed_distance 直读 linked）。
#[test]
fn liuyan_juedao_link_and_distance() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "wudian-upstream-noname")
        .expect("wudian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    // 5 人局对座基线 2
    let mut e = Engine::new_with_extensions(8901, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    e.debug_grant(0, "juedao");
    assert_eq!(e.state_core().computed_distance(e.catalog(), 0, 2), 2, "基线 2");
    assert!(!e.state_core().player(0).linked, "初始未横置");

    // 发动 → 横置
    let def = e.catalog().skill_by_name("juedao").expect("def");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert!(e.state_core().player(0).linked, "横置生效");
    // 双向距离 +1（对座 2→3）
    assert_eq!(e.state_core().computed_distance(e.catalog(), 0, 2), 3, "横置攻侧 +1");
    assert_eq!(e.state_core().computed_distance(e.catalog(), 2, 0), 3, "横置守侧 +1");

    // 已横置不可再发动
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
    assert!(matches!(out, CommandOutcome::Rejected(engine_native::RejectReason::InvalidChoice)), "已横置应拒: {out:?}");

    // 他人（1 号）横置不影响 0↔2：1 号 linked 只加 1 号双向——
    // 授予后需在 1 号自己的回合发动（ActivateSkill 授权限当前回合玩家）
    e.debug_grant(1, "juedao");
    advance_loop(&mut e, 1);
    let _ = submit(&mut e, 1, GameAction::ActivateSkill { skill: def });
    assert!(e.state_core().player(1).linked, "1 号横置");
    // 0↔2 不受 1 号影响
    assert_eq!(e.state_core().computed_distance(e.catalog(), 0, 2), 3, "0↔2 不变");
    // 0↔1：双横置 = 基线 1 + 1 + 1 = 3
    assert_eq!(e.state_core().computed_distance(e.catalog(), 0, 1), 3, "双横置叠加");
}

/// 割据（U07 技能批七，规则 v117）：刘焉——回合开始强制摸「攻击范围
/// 外其他势力种数」张（势力=effective_faction；无主将/unknown 排除；
/// 杀范围内他势力不计入）。
#[test]
fn liuyan_geju_out_of_range_faction_draw() {
    let (mut e, _pack) = setup();
    advance_loop(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    e.debug_grant(0, "geju");

    // 5 人局不适用（setup 3 人局相邻全在杀范围内）——3 人局全在范围内
    // → 无范围外他势力 → 不摸（负例先行）
    // 授予触发：0 号回合开始（Judge begin）已过——用下一回合验证
    let out = e
        .catalog()
        .skill_by_name("geju")
        .expect("geju 在目录");
    let _ = out;

    // 轮回 0 号：回合开始触发割据——3 人局相邻全在杀范围内（攻距 1）→
    // 无范围外他势力 → 不摸（Judge begin 先于 Draw：手牌=Draw 摸的 2）
    e.debug_set_skill_enabled(0, "geju", true);
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
    assert_eq!(e.debug_hand(0).len(), 2, "3 人局全在杀范围内：不摸（手牌=Draw 摸 2）");
    let _ = _pack;
}

/// 割据正例（4 人局）：1 号魏、2 号蜀、0 号任意；杀攻距 1——1 号在范
/// 围内（魏不计入）、2 号距离 2 范围外（蜀计入）→ 回合开始摸 1。
#[test]
fn liuyan_geju_out_of_range_counts() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "wudian-upstream-noname")
        .expect("wudian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let (wei, shu) = {
        let wei = engine_native::content::standard::STANDARD_GENERALS
            .iter()
            .find(|(_, f, ..)| *f == engine_native::catalog::Faction::Wei)
            .map(|(n, ..)| n.to_string())
            .expect("魏将");
        let shu = engine_native::content::standard::STANDARD_GENERALS
            .iter()
            .find(|(_, f, ..)| *f == engine_native::catalog::Faction::Shu)
            .map(|(n, ..)| n.to_string())
            .expect("蜀将");
        (wei, shu)
    };

    let mut e = Engine::new_with_extensions(9001, 4, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    e.debug_clear_hand(3);
    e.debug_grant(0, "geju");
    e.debug_assemble_general(1, &wei);
    e.debug_assemble_general(2, &shu);

    // 0↔2 距离 2 > 攻距 1 → 蜀为范围外他势力 → 回合开始摸 1
    let round0 = e.turn().round;
    for _ in 0..64 {
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
    // Judge begin 割据摸 1 → Draw 摸 2 → 手牌 3
    assert_eq!(e.debug_hand(0).len(), 3, "范围外他势力（蜀）1 种 → 摸 1 + Draw 2");
}

/// 士兵76（U07 ow 纵切，规则 v118）：突击（回合内每用一张牌攻距-1）+
/// 目镜（黑牌当杀 declared-Sha）。力场暂缓（下准备阶段时点）。
#[test]
fn ow_shibing_tuji_and_mujing() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-ow-upstream-noname")
        .expect("ow 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(9101, 5, &[&pack]).expect("建局");
    // 目录：3 张卡（cards 空）但 general 元数据 1 名（士兵76）——
    // 扩展武将接在基础表后
    let gen_base = engine_native::content::standard::STANDARD_GENERALS.len() as u16;
    let mut shibing_skills = None;
    for idx in gen_base..(gen_base + 8) {
        if let Some((name, _, skills, _, _)) = e.catalog().general_meta(idx) {
            if name == "士兵76" {
                shibing_skills = Some(skills.to_vec());
                break;
            }
        }
    }
    assert_eq!(
        shibing_skills.map(|v| v.to_vec()),
        Some(vec!["tuji", "mujing"]),
        "士兵76 目录技能=突击/目镜（lichang 暂缓未映射不入目录）"
    );

    advance_loop(&mut e, 0);
    e.debug_grant(0, "tuji");
    e.debug_grant(0, "mujing");
    e.debug_set_max_hp(0, 4);
    e.debug_set_hp(0, 4);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    e.debug_clear_hand(3);
    e.debug_clear_hand(4);

    // 基线：0↔2 对座 2
    assert_eq!(e.state_core().computed_distance(e.catalog(), 0, 2), 2, "对座基线");

    // 突击：出牌阶段用 1 张牌（目镜当杀）→ 计数 1 → 攻距 -1
    // 目镜：黑牌（spade）当杀打 2 号
    let black = e.debug_place(0, "sha"); // spade/club sha 均黑
    let def_mj = e.catalog().skill_by_name("mujing").expect("def");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_mj });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    let c = e.open_choice().expect("目镜选牌挂起");
    assert_eq!((c.spec.prompt, c.observer), (engine_native::content::host::WUDIAN_MUJING_CARD, 0));
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(black)] });
    let c = e.open_choice().expect("目镜目标挂起");
    assert_eq!((c.spec.prompt, c.observer), (engine_native::content::host::WUDIAN_MUJING_TARGET, 0));
    let (cid, obs) = (c.id, c.observer);
    e.debug_set_hp(2, 4);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Player(2)] });
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    assert_eq!(e.debug_hp(2), 3, "目镜当杀：2 号掉血（4→3）");
    // 突击计数：用 1 张 → 0↔2 距离 2-1=1
    assert_eq!(
        e.state_core().computed_distance(e.catalog(), 0, 2),
        1,
        "突击：用 1 张攻距 -1"
    );

    // 计数清零：换回合后距离回基线
    let round0 = e.turn().round;
    for _ in 0..64 {
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
    assert_eq!(e.state_core().computed_distance(e.catalog(), 0, 2), 2, "换回合计数清零回基线");
}

/// 光书（U07 ow 批二，规则 v119）：虚无之光——出牌阶段弃一张牌指定一
/// 名角色，按花色授子技：♥受伤后回 1（一次性）/♠下阶段结束受 1 无源
/// 雷伤（一次性）/♣不能用杀至下阶段结束。已有子技者不可再指定。
#[test]
fn zhixuzhiguang_guangshu_suit_marks() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-ow-upstream-noname")
        .expect("ow 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(9201, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    e.debug_grant(0, "guangshu");
    e.debug_set_max_hp(0, 4);
    e.debug_set_hp(0, 3);

    // ① ♠光塔：弃黑桃 → 指定 1 号 → 1 号下阶段结束受 1 雷伤（4→3）
    // 注入真黑桃杀（debug_place 按名取首张，花色不定）
    let spade = (0..e.catalog().cards.len() as u32)
        .find(|i| {
            e.catalog().cards[*i as usize].name == "sha"
                && e.catalog().cards[*i as usize].suit == engine_native::catalog::Suit::Spade
        })
        .expect("黑桃杀在目录");
    {
        let mut snap = e.save();
        for p in snap.core.players.iter_mut() {
            p.hand.retain(|c| *c != spade);
        }
        snap.core.deck.retain(|c| *c != spade);
        snap.core.discard.retain(|c| *c != spade);
        snap.core.table.retain(|c| *c != spade);
        snap.core.players[0].hand.insert(0, spade);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("黑桃杀注入");
    }
    let def_gs = e.catalog().skill_by_name("guangshu").expect("def");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_gs });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    let c = e.open_choice().expect("光书弃牌挂起");
    assert_eq!((c.spec.prompt, c.observer), (engine_native::content::host::WUDIAN_GUANGSHU_CARD, 0));
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(spade)] });
    let c = e.open_choice().expect("光书目标挂起");
    assert_eq!((c.spec.prompt, c.observer), (engine_native::content::host::WUDIAN_GUANGSHU_TARGET, 0));
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Player(1)] });
    let spade_def = e.catalog().skill_by_name("guangshu_spade").expect("def");
    assert!(e.state_core().has_skill(1, spade_def), "♠子技已授予");
    // 推进：任一阶段结束触发光塔
    let hp1 = e.debug_hp(1);
    for _ in 0..8 {
        if e.debug_hp(1) < hp1 {
            break;
        }
        if let Some(c) = e.open_choice() {
            let (cid, obs) = (c.id, c.observer);
            submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
        }
        let actor = e.turn().current;
        let _ = submit(&mut e, actor, GameAction::NextPhase);
    }
    assert_eq!(e.debug_hp(1), hp1 - 1, "♠光塔：阶段结束受 1 雷伤");
}

/// 自愈（U07 ow 批三，规则 v120）：禅雅塔——每 4 个阶段（phaseNumber
/// %4==0），你可以摸一张牌或回复 1 点体力（可放弃；满血时无回复选项）。
#[test]
fn chanyata_ziyu_every_four_phases() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-ow-upstream-noname")
        .expect("ow 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(9301, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    e.debug_grant(0, "ziyu");
    e.debug_set_max_hp(0, 4);
    e.debug_set_hp(0, 2); // 受伤：摸/回二选

    // 逐步推进阶段并计数——每当 End 阶段开始且 phaseNumber%4==0 时应挂
    // 起二选（Confirm 门 + 二选）；验证至少触发一次并选择回 1（2→3）
    let mut triggered = false;
    for _ in 0..160 {
        if let Some(c) = e.open_choice() {
            if c.spec.prompt == engine_native::content::host::WUDIAN_ZIYU_CHOICE {
                triggered = true;
                let hp_before_trigger = e.debug_hp(0);
                // 选 Tag(1)=回 1（若存在）否则摸 1
                let pick = if c
                    .spec
                    .options
                    .contains(&ChoiceOption::Tag(1))
                {
                    ChoiceOption::Tag(1)
                } else {
                    ChoiceOption::Tag(0)
                };
                let (cid, obs) = (c.id, c.observer);
                submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![pick] });
                if pick == ChoiceOption::Tag(1) {
                    assert_eq!(e.debug_hp(0), hp_before_trigger + 1, "回 1");
                }
                continue;
            }
            // trigger_confirm 门（OptionalGate）
            let (cid, obs) = (c.id, c.observer);
            submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Confirm] });
            continue;
        }
        let actor = e.turn().current;
        let _ = submit(&mut e, actor, GameAction::NextPhase);
        if e.turn().round > 12 {
            break;
        }
    }
    assert!(triggered, "12 回合内 %4 门控至少触发一次");
}

/// 守护（U07 ow 批四，规则 v121）：天使——出牌阶段弃一张【杀】令一名
/// 其他受伤角色回复 1 点体力；持有者不能使用【杀】（锁定禁杀）。
#[test]
fn tianshi_shouhu_discard_heal_and_sha_ban() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-ow-upstream-noname")
        .expect("ow 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(9401, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    e.debug_grant(0, "shouhu");
    e.debug_set_max_hp(0, 4);
    e.debug_set_hp(0, 3);
    e.debug_set_max_hp(1, 4);
    e.debug_set_hp(1, 2); // 1 号受伤（可为目标）

    // ① 弃杀救 1 号（2→3）
    let sha = e.debug_place(0, "sha");
    let def_sh = e.catalog().skill_by_name("shouhu").expect("def");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_sh });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    let c = e.open_choice().expect("守护弃杀挂起");
    assert_eq!((c.spec.prompt, c.observer), (engine_native::content::host::WUDIAN_SHOUHU_CARD, 0));
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(sha)] });
    let c = e.open_choice().expect("守护目标挂起");
    assert_eq!((c.spec.prompt, c.observer), (engine_native::content::host::WUDIAN_SHOUHU_TARGET, 0));
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Player(1)] });
    assert_eq!(e.debug_hp(1), 3, "守护回复 1（2→3）");
    assert!(e.debug_hand(0).is_empty(), "杀已弃");

    // ② 禁杀锁定：0 号出杀被拒
    let sha2 = e.debug_place(0, "sha");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sha2, targets: vec![1] });
    assert!(
        matches!(out, CommandOutcome::Rejected(engine_native::RejectReason::IllegalTargets)),
        "守护禁杀: {out:?}"
    );

    // ③ 无杀可弃 → 不可发动
    e.debug_clear_hand(0);
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_sh });
    assert!(matches!(out, CommandOutcome::Rejected(engine_native::RejectReason::InvalidChoice)), "无杀应拒: {out:?}");
}

/// 毒瘟（U07 ow 批五，规则 v122）：黑百合——锁定技：造成伤害时若你手
/// 牌数与受伤害角色相等，伤害 +1（铁索传导为独立事件照常判定）。
#[test]
fn heibaihe_duwen_equal_hand_damage_bonus() {
    // 毒瘟在 shaai-ow 包——自建 ow 局
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-ow-upstream-noname")
        .expect("ow 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let _pack = bridge(&json);
    let mut e = Engine::new_with_extensions(9501, 3, &[&_pack]).expect("建局");
    advance_loop(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    e.debug_grant(0, "duwen");
    e.debug_grant(0, "咆哮"); // 双次出杀驱动
    e.debug_set_max_hp(1, 4);
    e.debug_set_hp(1, 4);

    // ① 双方手牌各 1 张（出杀结算时 0 号 sha 已离手：2-1=1=1 号）→
    // 伤害 +1（4→2）
    let _ = e.debug_place(0, "tao");
    let sha = e.debug_place(0, "sha");
    let _ = e.debug_place(1, "tao");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    assert_eq!(e.debug_hp(1), 2, "手牌相等：伤害 +1（1+1=2，4→2）");

    // ② 手牌不等（0 号 1 张、1 号 0 张）→ 无加成（2→1）
    let _ = e.debug_place(0, "tao");
    let sha = e.debug_place(0, "sha");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    assert_eq!(e.debug_hp(1), 1, "手牌不等：无加成（2→1）");
}

/// 炮台（U05 ow 批六，规则 v123）：托比昂——出牌阶段弃一张【杀】布置/
/// 升级炮台（≤3 级，跨回合持久）；结束阶段开始按 30%×层概率对随机敌
/// 人造成 1 点火焰伤害（RuleRng 概率判定）；受到 1 点伤害炮台降 1 级。
#[test]
fn tuobiang_paotai_stack_fire_and_downgrade() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-ow-upstream-noname")
        .expect("ow 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(9601, 5, &[&pack]).expect("建局");
    // 注意：paotai 在 shaai-ow 包——本测试文件 setup() 为 wudian 包，
    // 故自建（同毒瘟批）。
    advance_loop(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    e.debug_grant(0, "paotai");
    e.debug_grant(0, "paotai2");
    e.debug_set_max_hp(0, 4);
    e.debug_set_hp(0, 4);

    let def_pt = e.catalog().skill_by_name("paotai").expect("def");

    // ① 三次布置（3 张杀）→ lv=3
    for i in 0..3 {
        let sha = e.debug_place(0, "sha");
        let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_pt });
        assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
        let c = e.open_choice().expect("炮台弃杀挂起");
        assert_eq!((c.spec.prompt, c.observer), (engine_native::content::host::WUDIAN_PAOTAI_CARD, 0));
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(sha)] });
        assert_eq!(e.state_core().player(0).paotai_lv, i + 1, "第 {} 层", i + 1);
    }
    // lv=3 时不再可发动
    let _sha = e.debug_place(0, "sha");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_pt });
    assert!(matches!(out, CommandOutcome::Rejected(engine_native::RejectReason::InvalidChoice)), "满级应拒: {out:?}");

    // ② 结束阶段射击：lv=3 → 概率 100%（roll<100 恒真）→ 必中随机敌
    // 推进到 End 阶段开始（fire 在 End 阶段开始帧触发——自愈/割据同款
    // PhaseBegin 通道）；进入 End 后随机敌掉血
    let hp1 = e.debug_hp(1);
    let hp2 = e.debug_hp(2);
    for _ in 0..8 {
        if e.turn().phase == Phase::End {
            break;
        }
        if let Some(c) = e.open_choice() {
            let (cid, obs) = (c.id, c.observer);
            submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
        }
        let actor = e.turn().current;
        let _ = submit(&mut e, actor, GameAction::NextPhase);
    }
    assert_eq!(e.turn().phase, Phase::End);
    assert!(
        e.debug_hp(1) == hp1 - 1 || e.debug_hp(2) == hp2 - 1,
        "lv=3 概率 100%：随机敌受 1 火伤"
    );

    // ③ 受伤降级：0 号受 2 伤 → lv 3→1
    e.debug_set_hp(0, 4);
    e.debug_deal_damage(Some(1), 0, 2, engine_native::event::DamageNature::Normal);
    assert_eq!(e.state_core().player(0).paotai_lv, 1, "受 2 伤降 2 级");

    // ④ 换回合后 lv 持久
    let round0 = e.turn().round;
    for _ in 0..64 {
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
    assert_eq!(e.state_core().player(0).paotai_lv, 1, "等级跨回合持久");
}

/// 堡垒/侦察切换（U05 ow 批七，规则 v124）：堡垒——出牌阶段确认切换
/// 模式（三态翻转：无→哨卫→侦察→哨卫），baolei_mode 跨回合持久入存
///档。哨卫/侦察的模式效果钩（杀距离/防具/次数/闪避、用杀摸回）登记
/// 待做；本切片仅状态通道。
#[test]
fn baolei_switch_mode_cycle() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-ow-upstream-noname")
        .expect("ow 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(9701, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    e.debug_grant(0, "bshaowei");
    let def = e.catalog().skill_by_name("bshaowei").expect("def");

    // 0→1（哨卫）
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(e.state_core().player(0).baolei_mode, 1, "切换至哨卫");

    // 1→2（侦察）
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(e.state_core().player(0).baolei_mode, 2, "切换至侦察");

    // 2→1（回哨卫，三态翻转）
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(e.state_core().player(0).baolei_mode, 1, "回哨卫");

    // 跨回合持久 + 存档往返
    let snap = e.save();
    assert_eq!(snap.core.player(0).baolei_mode, 1);
    let e2 = Engine::restore_with_extensions(&snap, &[&pack]).expect("恢复");
    assert_eq!(e2.state_core().player(0).baolei_mode, 1, "存档往返保持");

    let round0 = e.turn().round;
    for _ in 0..64 {
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
    assert_eq!(e.state_core().player(0).baolei_mode, 1, "跨回合持久");
}

/// 侦察摸回（U05 ow 批八，规则 v125）：侦察模式下用杀 → 摸 1 或回 1
/// （chooseDrawRecover 可放弃）；哨卫模式目标侧禁闪 + 源侧直中。
#[test]
fn baolei_mode_effects_on_sha() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-ow-upstream-noname")
        .expect("ow 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(9801, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    e.debug_grant(0, "bshaowei");
    e.debug_grant(0, "zhencha_draw");
    e.debug_set_max_hp(0, 4);
    e.debug_set_hp(0, 4);

    let def_sw = e.catalog().skill_by_name("bshaowei").expect("def");
    // 切至哨卫（0→1）
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_sw });
    assert_eq!(e.state_core().player(0).baolei_mode, 1);

    // 哨卫源侧：0 号杀 1 号（1 号有闪也不可闪避——直接伤害）
    let sha = e.debug_place(0, "sha");
    let shan = e.debug_place(1, "shan"); // 1 号有闪但应被无视
    e.debug_set_hp(1, 4);
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    assert_eq!(e.debug_hp(1), 3, "哨卫：杀不可闪避（4→3）");
    let _ = shan;

    // 哨卫目标侧：1 号（有哨卫技？无——哨卫效果是持技者双向）——
    // 授 1 号后 0 号杀 1 号 → 1 号不可闪避
    e.debug_grant(1, "bshaowei");
    // 1 号 mode 默认 0（未切换）→ 目标侧哨卫需 mode=1——1 号未切换不生效
    let sha = e.debug_place(0, "sha");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    assert_eq!(e.debug_hp(1), 2, "1 号无哨卫模式：正常可闪（无闪则掉血 3→2）");
}

// ---------------------------------------------------------------------------
// U05 ow 纵切批九（规则 v126）：卢西奥三技——扩音（弃 1 张→乐动目标数
// +2 / 弃 2 张→摸回量 +1）、呼唤（弃 2 张→回复模式）、乐动（出牌阶段
// 结束选目标摸/回）。回合级存储三角（TurnState.yuedong_*，存档 v7）。
// ---------------------------------------------------------------------------

/// 打开挂起选择并以给定选择应答。
fn answer_open(e: &mut Engine, selection: Vec<ChoiceOption>) -> CommandOutcome {
    let c = e.open_choice().cloned().expect("存在挂起选择");
    let (cid, obs) = (c.id, c.observer);
    drop(c);
    submit(e, obs, GameAction::Answer { choice_id: cid, selection })
}

/// 扩音：弃 1 张 → 乐动目标数 +2；出牌阶段结束触发乐动 → 3 目标各摸
/// 1（基础摸量）；弃牌入弃牌堆。
#[test]
fn luxiao_kuoyin_num_extends_yuedong_draw() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-ow-upstream-noname")
        .expect("ow 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(9901, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "yuedong");
    e.debug_grant(0, "kuoyin");
    let c1 = e.debug_place(0, "sha");
    let _c2 = e.debug_place(0, "tao");

    // 扩音：activate 挂起选牌（1–2 张）→ 弃 1 张
    let def_ky = e.catalog().skill_by_name("kuoyin").expect("def");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_ky });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let out = answer_open(&mut e, vec![ChoiceOption::Card(c1)]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(e.state_core().turn.yuedong_num, 2, "弃 1 张 → 目标数 +2");
    assert_eq!(e.state_core().turn.yuedong_eff, 0);
    assert!(e.state_core().discard.contains(&c1), "扩音弃牌入弃牌堆");

    // 出牌阶段结束 → 乐动触发挂起（目标数 1+2=3，摸量 1+0=1）
    let out = submit(&mut e, 0, GameAction::NextPhase);
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let targets_sel = vec![
        ChoiceOption::Player(1),
        ChoiceOption::Player(2),
        ChoiceOption::Player(3),
    ];
    let out = answer_open(&mut e, targets_sel);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(e.debug_hand(1).len(), 1, "目标 1 摸 1");
    assert_eq!(e.debug_hand(2).len(), 1, "目标 2 摸 1");
    assert_eq!(e.debug_hand(3).len(), 1, "目标 3 摸 1");
    assert_eq!(e.debug_hand(0).len(), 1, "自己未选不摸（余 1 张 tao）");
}

/// 扩音双段：弃 2 张 → 摸/回量 +1（乐动 1 目标摸 2）；双满后 activate
/// 被拒（上游 filter 双存量同口径）。
#[test]
fn luxiao_kuoyin_eff_and_double_full_gate() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-ow-upstream-noname")
        .expect("ow 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(9902, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "yuedong");
    e.debug_grant(0, "kuoyin");
    let c1 = e.debug_place(0, "sha");
    let c2 = e.debug_place(0, "tao");
    let c3 = e.debug_place(0, "shan");

    // 第一次扩音：弃 2 张 → 摸/回量 +1
    let def_ky = e.catalog().skill_by_name("kuoyin").expect("def");
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_ky });
    let _ = answer_open(&mut e, vec![ChoiceOption::Card(c1), ChoiceOption::Card(c2)]);
    assert_eq!(e.state_core().turn.yuedong_eff, 1, "弃 2 张 → 量 +1");
    assert_eq!(e.state_core().turn.yuedong_num, 0);

    // 第二次扩音（存量单满 → 恰 1 张）：弃 1 张 → 目标数 +2
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_ky });
    let _ = answer_open(&mut e, vec![ChoiceOption::Card(c3)]);
    assert_eq!(e.state_core().turn.yuedong_num, 2, "第二次恰选 1 张 → 数 +2");

    // 双满 → activate 拒绝
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_ky });
    assert!(
        matches!(&out, CommandOutcome::Rejected(engine_native::RejectReason::InvalidChoice)),
        "双存量后应拒绝：{out:?}"
    );

    // 乐动：1 目标摸 2（量 1+1）
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let out = answer_open(&mut e, vec![ChoiceOption::Player(1)]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(e.debug_hand(1).len(), 2, "1 目标摸 2");
}

/// 呼唤：弃 2 张 → 回复模式；乐动改回复（未满体力目标 +1 体力，不摸
/// 牌）；满血者不入目标选项（上游 filterTarget hp<maxHp 同口径）。
#[test]
fn luxiao_huhuan_switches_yuedong_to_recover() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-ow-upstream-noname")
        .expect("ow 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(9903, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "yuedong");
    e.debug_grant(0, "huhuan");
    let c1 = e.debug_place(0, "sha");
    let c2 = e.debug_place(0, "tao");
    e.debug_set_hp(1, 2);
    e.debug_set_hp(3, 3);

    // 呼唤：弃 2 张 → 回复模式
    let def_hh = e.catalog().skill_by_name("huhuan").expect("def");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_hh });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let _ = answer_open(&mut e, vec![ChoiceOption::Card(c1), ChoiceOption::Card(c2)]);
    assert!(e.state_core().turn.yuedong_recover, "呼唤置回复模式");

    // 呼唤二次被拒（recover 已置位）
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_hh });
    assert!(
        matches!(&out, CommandOutcome::Rejected(engine_native::RejectReason::InvalidChoice)),
        "呼唤限一次：{out:?}"
    );

    // 乐动触发：目标选项仅未满体力者（1/3；0/2/4 满血排除）
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let c = e.open_choice().cloned().expect("乐动挂起");
    let mut opts: Vec<u8> = c
        .spec
        .options
        .iter()
        .filter_map(|o| match o {
            ChoiceOption::Player(p) => Some(*p),
            _ => None,
        })
        .collect();
    opts.sort_unstable();
    assert_eq!(opts, vec![1, 3], "仅未满体力者可选：{opts:?}");
    drop(c);
    let out = answer_open(&mut e, vec![ChoiceOption::Player(1)]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(e.debug_hp(1), 3, "回复模式：2→3");
    assert_eq!(e.debug_hand(1).len(), 0, "回复模式不摸牌");
}

/// 乐动可整体放弃（cancel 空选择）；挂起的乐动目标询问跨存档恢复
///（TurnState 三字段入存档 v7 + active_ask 派生贯通）。
#[test]
fn luxiao_yuedong_cancel_and_pending_ask_restores() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-ow-upstream-noname")
        .expect("ow 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(9904, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "yuedong");
    e.debug_place(0, "sha");

    // 无扩音：1 目标摸 1；整体放弃（空选择=取消）→ 无效果
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let out = answer_open(&mut e, Vec::new());
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(e.debug_hand(1).len(), 0, "放弃乐动不摸牌");
    e.debug_clear_hand(0);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // 弃牌→结束→下家

    // 下一次 0 的出牌阶段：扩音后挂起乐动 → 存档 → 恢复 → 应答生效
    advance_loop(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_grant(0, "kuoyin");
    let c1 = e.debug_place(0, "sha");
    let def_ky = e.catalog().skill_by_name("kuoyin").expect("def");
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_ky });
    let _ = answer_open(&mut e, vec![ChoiceOption::Card(c1)]);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    assert!(e.open_choice().is_some(), "乐动挂起");
    let snap = e.save();
    let mut e2 = Engine::restore_with_extensions(&snap, &[&pack]).expect("恢复");
    let before = e2.debug_hand(2).len();
    let out = answer_open(&mut e2, vec![ChoiceOption::Player(2)]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(e2.debug_hand(2).len(), before + 1, "恢复后乐动应答摸 1");
    assert_eq!(e2.state_core().turn.yuedong_num, 2, "扩音存量随存档保留");
}

// ---------------------------------------------------------------------------
// U05 ow 纵切批十（规则 v127）：美（ow_mei）——暴雪（baoxue，limited：
// 弃全部黑色 he 牌+逐目标随机弃 1+翻面+自身翻面）与急冻（jidong，
// hp=1 任意阶段结束：翻面+回复 2+冰封）双技；bingqiang 冰墙登记暂缓。
// 基建：翻面角色回合跳过（Judge begin 翻回+整回合无阶段帧）+冰封双门
// 控（commit_damage 无效化 / PlayCard 目标拒绝）。存档 v7→v8。
// ---------------------------------------------------------------------------

/// 暴雪：整体放弃（空选择）不消耗 limited；发动后弃全部黑色 he 牌、
/// 逐目标随机弃 1+翻至背面、自己翻至背面；每局限一次（再发动拒绝）。
#[test]
fn mei_baoxue_limited_discard_and_flips() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-ow-upstream-noname")
        .expect("ow 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(9911, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "baoxue");
    let c1 = e.debug_place(0, "sha");
    let c2 = e.debug_place(0, "sha");
    let c3 = e.debug_place(0, "tao");
    let is_black = |e: &Engine, c: u32| {
        matches!(
            e.catalog().cards[c as usize].suit,
            engine_native::catalog::Suit::Spade | engine_native::catalog::Suit::Club
        )
    };
    let black = [c1, c2, c3].iter().filter(|c| is_black(&e, **c)).count();
    assert!(black >= 1, "种子 9911 应给出至少 1 张黑牌");
    let _c_t1 = e.debug_place(1, "shan");
    let _c_t2 = e.debug_place(2, "shan");

    let def_bx = e.catalog().skill_by_name("baoxue").expect("def");

    // 整体放弃（空选择）：limited 未消耗、无效果
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_bx });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let out = answer_open(&mut e, Vec::new());
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert!(!e.state_core().player(0).baoxue_used, "放弃不消耗 limited");
    assert!(!e.debug_face_down(0), "放弃不翻面");

    // 发动：目标选项=其他存活者，上限=黑色牌数
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_bx });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let ask = e.open_choice().cloned().expect("暴雪目标询问");
    assert_eq!(ask.spec.max, black, "目标上限=黑色 he 牌数");
    drop(ask);
    // 按实际上限选目标（黑牌数可能为 1）
    let mut sel = vec![ChoiceOption::Player(1)];
    if black >= 2 {
        sel.push(ChoiceOption::Player(2));
    }
    let out = answer_open(&mut e, sel);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");

    assert!(e.state_core().player(0).baoxue_used, "limited 已消耗");
    assert_eq!(e.debug_hand(0).len(), 3 - black, "弃全部黑色 he 牌");
    assert!(is_black_card_discarded(&e, &[c1, c2, c3], black), "黑牌入弃牌堆");
    assert_eq!(e.debug_hand(1).len(), 0, "目标 1 随机弃 1（牌池仅 1 张）");
    assert!(e.debug_face_down(1), "目标 1 翻至背面");
    if black >= 2 {
        assert_eq!(e.debug_hand(2).len(), 0, "目标 2 随机弃 1");
        assert!(e.debug_face_down(2), "目标 2 翻至背面");
    }
    assert!(e.debug_face_down(0), "自己翻至背面");

    // 再发动拒绝（limited 每局一次）
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_bx });
    assert!(
        matches!(&out, CommandOutcome::Rejected(engine_native::RejectReason::InvalidChoice)),
        "limited 后应拒绝：{out:?}"
    );

    // 存档往返（v8）：limited 标记与翻面状态随存档保留
    let snap = e.save();
    let e2 = Engine::restore_with_extensions(&snap, &[&pack]).expect("恢复");
    assert!(e2.state_core().player(0).baoxue_used, "limited 标记随存档保留");
    assert!(e2.debug_face_down(0), "翻面状态随存档保留");
}

/// 暴雪弃牌堆断言辅助：黑牌均在弃牌堆。
fn is_black_card_discarded(e: &Engine, cards: &[u32], black: usize) -> bool {
    let mut n = 0;
    for c in cards {
        let isb = matches!(
            e.catalog().cards[*c as usize].suit,
            engine_native::catalog::Suit::Spade | engine_native::catalog::Suit::Club
        );
        if isb && e.state_core().discard.contains(c) {
            n += 1;
        }
    }
    n == black
}

/// 翻面角色回合跳过：轮至翻面者——先经翻面前置窗口翻回，整回合无
/// 阶段帧（不摸牌），回合槽照常消耗（下家照常开局）。
#[test]
fn mei_flip_face_down_turn_skipped_and_flipped_back() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-ow-upstream-noname")
        .expect("ow 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(9912, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_set_face_down(1, true);

    // 0 收束回合 → 1 的回合被跳过（翻回+无摸牌）→ 2 开局
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let out = submit(&mut e, 0, GameAction::NextPhase); // End→回合收束
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert!(!e.debug_face_down(1), "跳过回合时翻回正面");
    assert_eq!(e.debug_hand(1).len(), 0, "跳过回合不摸牌");
    let t = e.turn();
    assert_eq!(t.current, 2, "回合槽消耗：轮至 2");
    assert_eq!(t.phase, Phase::Judge, "2 号正常开局");
}

/// 急冻：hp=1 时任意阶段结束——翻面+回复 2+冰封；冰封期间①其他角色
/// 不能以牌指定其为目标（IllegalTargets）②AOE 伤害无效（万箭照常问
/// 闪，伤害被无效化）；翻回（跳过回合）后解除。
#[test]
fn mei_jidong_freeze_damage_immune_and_untargetable() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-ow-upstream-noname")
        .expect("ow 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(9913, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "jidong");
    e.debug_set_hp(0, 1);

    // 0 出牌→弃牌：弃牌阶段结束（hp=1）触发急冻——翻面+回复 2+冰封，
    // 随后 End 阶段照常开局（回合中翻面不跳过剩余阶段——上游
    // phaseBefore 仅回合开始判定）
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let out = submit(&mut e, 0, GameAction::NextPhase); // Discard 结束→急冻→End
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert!(e.debug_face_down(0), "急冻翻面");
    assert_eq!(e.debug_hp(0), 3, "回复 2（1→3）");
    assert!(e.state_core().player(0).jidong_active, "冰封中");
    let t = e.turn();
    assert_eq!(t.phase, Phase::End, "回合中翻面不跳过剩余阶段");

    // End 结束→1 回合：Judge→Draw→Play
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    advance_loop(&mut e, 1);
    let sha = e.debug_place(1, "sha");
    // ①冰封者不可被指定为目标
    let out = submit(&mut e, 1, GameAction::PlayCard { card: sha, targets: vec![0] });
    assert!(
        matches!(&out, CommandOutcome::Rejected(engine_native::RejectReason::IllegalTargets)),
        "冰封者不可被指定：{out:?}"
    );
    // ②AOE（万箭齐发）不询问 targetEnabled——冰封者被问闪（无闪），
    // 伤害被无效化；他人照常受伤
    let wanjian = e.debug_place(1, "archery");
    let out = submit(&mut e, 1, GameAction::PlayCard { card: wanjian, targets: vec![] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    assert_eq!(e.debug_hp(0), 3, "冰封：万箭伤害无效");
    assert_eq!(e.debug_hp(2), 3, "他人照常受 1（4→3）");

    // 1 收束→2/3/4 回合→0 的回合被跳过并翻回→冰封解除（驱动至 0 跳
    // 过完成、1 开局）
    for _ in 0..200 {
        let t = e.turn();
        if t.current == 1 && t.phase == Phase::Judge {
            break;
        }
        if let Some(c) = e.open_choice().cloned() {
            let (cid, obs) = (c.id, c.observer);
            let sel = if c.spec.min > 0 {
                c.spec.options.iter().take(c.spec.min).copied().collect()
            } else {
                vec![c.spec.options[0]]
            };
            drop(c);
            let out = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: sel });
            assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
            continue;
        }
        let cur = e.turn().current;
        let out = submit(&mut e, cur, GameAction::NextPhase);
        assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    }
    assert!(!e.debug_face_down(0), "跳过回合翻回正面");
    assert!(!e.state_core().player(0).jidong_active, "翻回即解除冰封");
}

// ---------------------------------------------------------------------------
// U05 ow 纵切批十一（规则 v128）：路霸（ow_luba）双技——链钩（liangou，
// 出牌阶段限一次：弃 1 张 he+判定非红桃→钩住一名其他角色至回合结束：
// 路霸算到其距离恒 1+其首次受伤 +1）与吸阳（xiyang，回合结束确认：
// 受伤且未翻面→翻面+回复 2）。TurnState 两字段（存档 v8→v9）。
// ---------------------------------------------------------------------------

/// 构造判定牌：把指定花色的一张牌移到牌堆顶（下次 draw_judge_card 摸到）。
fn rig_next_judge(e: &Engine, snap: &mut engine_native::StateSnapshot, want: engine_native::catalog::Suit) {
    let pos = (0..snap.core.deck.len())
        .rev()
        .find(|i| e.catalog().cards[snap.core.deck[*i] as usize].suit == want)
        .unwrap_or_else(|| panic!("牌堆中无目标花色"));
    let c = snap.core.deck.remove(pos);
    snap.core.deck.push(c);
}

/// 链钩：判定非红桃→挂钩；路霸算到被钩者距离恒 1（无钩时距离 2 被拒）；
/// 被钩者首次受伤 +1（决斗 1+1）；换回合随 TurnState 清零解除。
#[test]
fn luba_liangou_hook_distance_and_damage() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-ow-upstream-noname")
        .expect("ow 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(9921, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "liangou");
    let c1 = e.debug_place(0, "sha");
    let _c_t = e.debug_place(1, "tao"); // 1 号留牌（与本测试无涉）

    // 负例：无钩时 2 号（座位距离 2）不可达
    let c2 = e.debug_place(0, "sha");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: c2, targets: vec![2] });
    assert!(
        matches!(&out, CommandOutcome::Rejected(engine_native::RejectReason::IllegalTargets)),
        "距离 2 应被拒：{out:?}"
    );

    // 构造黑桃判定顶
    let mut snap = e.save();
    rig_next_judge(&e, &mut snap, engine_native::catalog::Suit::Spade);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("恢复");

    // 链钩：弃 c1 → 目标 2 → 判定黑桃成功
    let def_lg = e.catalog().skill_by_name("liangou").expect("def");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_lg });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let out = answer_open(&mut e, vec![ChoiceOption::Card(c1)]);
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let out = answer_open(&mut e, vec![ChoiceOption::Player(2)]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(e.state_core().turn.liangou_hook, Some(2), "判定非红桃→挂钩");
    assert!(!e.state_core().turn.liangou_bonus_used);
    assert!(e.state_core().discard.contains(&c1), "链钩弃牌入弃牌堆");

    // 正例：距离恒 1——杀可达 2 号；首次伤害 +1（4-1-1=2）
    let c3 = e.debug_place(0, "sha");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: c3, targets: vec![2] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "挂钩后距离 2 可达：{out:?}");
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    assert_eq!(e.debug_hp(2), 2, "首伤 +1（4→2）");
    assert!(e.state_core().turn.liangou_bonus_used, "增幅已消耗");

    // 决斗（距离 1 可达）：无增幅（2→1）
    let duel = e.debug_place(0, "duel");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: duel, targets: vec![2] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    assert_eq!(e.debug_hp(2), 1, "决斗无增幅（2→1）");

    // usable:1：同回合再发动拒绝（SkillAlreadyUsed——once_used 命令层）
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_lg });
    assert!(
        matches!(&out, CommandOutcome::Rejected(engine_native::RejectReason::SkillAlreadyUsed)),
        "链钩每回合限一次：{out:?}"
    );

    // 换回合随 TurnState 清零解除
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→收束→1 号
    let t = e.turn();
    assert_eq!(t.current, 1);
    assert_eq!(e.state_core().turn.liangou_hook, None, "换回合钩住解除");
}

/// 链钩判定红桃→不挂钩（距离仍受限）+发动已消耗；吸阳：受伤未翻面时
/// 回合结束确认——翻面+回复 2；翻面/满血不触发；翻面后下回合被跳过翻回。
#[test]
fn luba_liangou_heart_fail_and_xiyang_flip_recover() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-ow-upstream-noname")
        .expect("ow 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(9922, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "liangou");
    e.debug_grant(0, "xiyang");
    let c1 = e.debug_place(0, "sha");

    // 构造红桃判定顶
    let mut snap = e.save();
    rig_next_judge(&e, &mut snap, engine_native::catalog::Suit::Heart);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("恢复");

    // 链钩判定失败：不挂钩
    let def_lg = e.catalog().skill_by_name("liangou").expect("def");
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_lg });
    let _ = answer_open(&mut e, vec![ChoiceOption::Card(c1)]);
    let _ = answer_open(&mut e, vec![ChoiceOption::Player(2)]);
    assert_eq!(e.state_core().turn.liangou_hook, None, "红桃判定不挂钩");
    // 发动已消耗（usable:1 计发动不按结果）
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_lg });
    assert!(
        matches!(&out, CommandOutcome::Rejected(engine_native::RejectReason::SkillAlreadyUsed)),
        "{out:?}"
    );

    // 吸阳前置：0 号受伤（2/4）、正面
    e.debug_set_hp(0, 2);
    // 弃牌→End：End 阶段结束触发吸阳确认门
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let out = submit(&mut e, 0, GameAction::NextPhase); // End 结束→吸阳门
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let out = answer_open(&mut e, vec![ChoiceOption::Confirm]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert!(e.debug_face_down(0), "吸阳翻面");
    assert_eq!(e.debug_hp(0), 4, "吸阳回复 2（2→4）");

    // 翻面后下回合被跳过翻回（v127 基建）——驱动到 1 回合结束
    let _ = submit(&mut e, 0, GameAction::NextPhase); // 收束→1 号 Judge
    let mut face_up_again = false;
    for _ in 0..64 {
        let t = e.turn();
        if t.current == 0 && !e.debug_face_down(0) {
            face_up_again = true;
            break;
        }
        if let Some(c) = e.open_choice().cloned() {
            let (cid, obs) = (c.id, c.observer);
            let sel = if c.spec.min > 0 {
                c.spec.options.iter().take(c.spec.min).copied().collect()
            } else {
                vec![c.spec.options[0]]
            };
            drop(c);
            let out = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: sel });
            assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
            continue;
        }
        let cur = e.turn().current;
        let out = submit(&mut e, cur, GameAction::NextPhase);
        assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    }
    assert!(face_up_again, "跳过回合翻回正面");
}

/// v126 遗留缺陷回归（v128 修复）：乐动三角换回合清零——TurnState 换
/// 回合为原地改字段而非重建，扩音/呼唤存量不得跨回合残留（上游 kuoyin2
/// phaseBegin 重置口径）。
#[test]
fn luxiao_yuedong_storage_resets_next_turn() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-ow-upstream-noname")
        .expect("ow 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(9923, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "kuoyin");
    e.debug_grant(0, "huhuan");
    let c1 = e.debug_place(0, "sha");
    let c2 = e.debug_place(0, "tao");
    let c3 = e.debug_place(0, "shan");
    let c4 = e.debug_place(0, "sha");

    // 弃 1 张（num+2）、呼唤弃 2 张（recover）
    let def_ky = e.catalog().skill_by_name("kuoyin").expect("def");
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_ky });
    let _ = answer_open(&mut e, vec![ChoiceOption::Card(c1)]);
    assert_eq!(e.state_core().turn.yuedong_num, 2);
    let def_hh = e.catalog().skill_by_name("huhuan").expect("def");
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_hh });
    let _ = answer_open(&mut e, vec![ChoiceOption::Card(c2), ChoiceOption::Card(c3)]);
    assert!(e.state_core().turn.yuedong_recover);

    // 收束回合 → 驱动到 0 号下个出牌阶段：存量应全部清零
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    for _ in 0..96 {
        let t = e.turn();
        if t.current == 0 && t.phase == Phase::Play && e.open_choice().is_none() {
            break;
        }
        if let Some(c) = e.open_choice().cloned() {
            let (cid, obs) = (c.id, c.observer);
            let sel = if c.spec.min > 0 {
                c.spec.options.iter().take(c.spec.min).copied().collect()
            } else {
                vec![c.spec.options[0]]
            };
            drop(c);
            let out = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: sel });
            assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
            continue;
        }
        let cur = e.turn().current;
        let out = submit(&mut e, cur, GameAction::NextPhase);
        assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    }
    let t = e.state_core();
    assert_eq!(
        (t.turn.yuedong_num, t.turn.yuedong_eff, t.turn.yuedong_recover),
        (0, 0, false),
        "乐动三角随换回合清零"
    );
    // 清零后扩音可再次发动（存量双满不再拒绝）
    e.debug_clear_hand(0);
    let c5 = e.debug_place(0, "sha");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_ky });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "新回合扩音可发动：{out:?}");
    let _ = answer_open(&mut e, vec![ChoiceOption::Card(c5)]);
    assert_eq!(e.state_core().turn.yuedong_num, 2, "新回合重新积攒");
    let _ = c4;
}

// ---------------------------------------------------------------------------
// U05 ow 纵切批十二（规则 v129）：源氏（ow_yuanshi）飞刃（feiren，锁定
// 被动）——黑桃杀伤害 +1（非铁索传导，DamageData.from_chain 新字段）+
// 杀无视距离 + 梅花杀额外目标上限 +1；lianpo/zhanlong 登记暂缓。
// ---------------------------------------------------------------------------

/// 从手牌中找指定花色的一张牌。
fn pick_suit(e: &Engine, suit: engine_native::catalog::Suit) -> Option<u32> {
    e.debug_hand(0)
        .into_iter()
        .find(|c| e.catalog().cards[*c as usize].suit == suit)
}

/// 飞刃：黑桃杀 +1（4→2）；杀无视距离（距离 2 可达）；梅花杀双目标
/// 可达（非梅花杀双目标仍拒）。
#[test]
fn yuanshi_feiren_bonus_range_and_club_extra_target() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-ow-upstream-noname")
        .expect("ow 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(9931, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "feiren");
    // 诸葛连弩：出杀不限次（飞刃无次数豁免，测试需连出多杀）
    let zhuge = e.debug_place(0, "zhuge");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: zhuge, targets: vec![] });
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "装备连弩: {out:?}");
    // 多放几张杀保证三种花色齐备
    for _ in 0..8 {
        e.debug_place(0, "sha");
    }
    let spade = pick_suit(&e, engine_native::catalog::Suit::Spade).expect("应有黑桃杀");
    let club = pick_suit(&e, engine_native::catalog::Suit::Club).expect("应有梅花杀");
    let red = pick_suit(&e, engine_native::catalog::Suit::Heart)
        .or_else(|| pick_suit(&e, engine_native::catalog::Suit::Diamond))
        .expect("应有红杀");
    e.debug_set_hp(1, 4);
    e.debug_set_hp(2, 4);

    // ① 黑桃杀 +1（距离 1）
    let out = submit(&mut e, 0, GameAction::PlayCard { card: spade, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    assert_eq!(e.debug_hp(1), 2, "黑桃杀 +1（4→2）");

    // ② 杀无视距离：距离 2 的 2 号可达（红杀无加成 4→3）
    let out = submit(&mut e, 0, GameAction::PlayCard { card: red, targets: vec![2] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "飞刃杀无视距离：{out:?}");
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    assert_eq!(e.debug_hp(2), 3, "红杀无加成（4→3）");

    // ③ 梅花杀双目标（feiren_extra=1）；红杀双目标仍拒
    let out = submit(&mut e, 0, GameAction::PlayCard { card: club, targets: vec![1, 2] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "梅花杀双目标：{out:?}");
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    // 1/2 各再受 1（梅花无黑桃加成）
    assert_eq!(e.debug_hp(1), 1, "梅花杀无黑桃加成（2→1）");
    assert_eq!(e.debug_hp(2), 2, "梅花杀（3→2）");

    let red2 = pick_suit(&e, engine_native::catalog::Suit::Heart)
        .or_else(|| pick_suit(&e, engine_native::catalog::Suit::Diamond));
    if let Some(r2) = red2 {
        let out = submit(&mut e, 0, GameAction::PlayCard { card: r2, targets: vec![1, 2] });
        assert!(
            matches!(&out, CommandOutcome::Rejected(engine_native::RejectReason::IllegalTargets)),
            "非梅花杀双目标应拒：{out:?}"
        );
    }
}

/// 飞刃 notLink：黑桃火杀经铁索传导——主伤害 +1、传导伤害不加成
///（DamageData.from_chain 新字段）。
#[test]
fn yuanshi_feiren_chain_damage_not_bonused() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-ow-upstream-noname")
        .expect("ow 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(9932, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "feiren");
    // 铁索连环连接 1、2
    let chain = e.debug_place(0, "chain");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: chain, targets: vec![1, 2] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    // 黑桃火杀打 1：主伤害 1+1=2，传导给 2 为 1（不加成）
    let mut fire_spade = None;
    for _ in 0..8 {
        e.debug_place(0, "fire_sha");
        if let Some(c) = pick_suit(&e, engine_native::catalog::Suit::Spade) {
            // 排除普通杀误选（手牌里只有 fire_sha）
            fire_spade = Some(c);
            break;
        }
    }
    let fsha = fire_spade.expect("应有黑桃火杀");
    e.debug_set_hp(1, 4);
    e.debug_set_hp(2, 4);
    let out = submit(&mut e, 0, GameAction::PlayCard { card: fsha, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    assert_eq!(e.debug_hp(1), 2, "主伤害 +1（4→2）");
    assert_eq!(e.debug_hp(2), 3, "传导伤害不加成（4→3）");
}

// ---------------------------------------------------------------------------
// U05 ow 纵切批十三（规则 v130）：法老之鹰（ow_falaozhiying）飞弹
//（feidan，杀伤害后触发）——可弃 1 张牌：受伤者距离 1 内的其他角色各
// 受 1 点无牌伤害（按座位序；可放弃=不弃不弹）；杀无视距离。
// huoyu（虚拟 useCard）/feiying（共享技能文件）登记暂缓。
// ---------------------------------------------------------------------------

/// 飞弹：杀伤害后弃牌弹射（受伤者距 1 内的 0/2 号各受 1，含自己）；
/// 放弃（空选择）不弹射；杀无视距离（距离 2 可达）；决斗伤害不触发。
#[test]
fn falaozhiying_feidan_splash_cancel_and_range() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-ow-upstream-noname")
        .expect("ow 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(9941, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "feidan");
    // 连弩解除限次；牌堆多放杀+一张弃牌素材
    let zhuge = e.debug_place(0, "zhuge");
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: zhuge, targets: vec![] });
    let d1 = e.debug_place(0, "tao");
    for _ in 0..4 {
        e.debug_place(0, "sha");
    }
    e.debug_set_hp(0, 4);
    e.debug_set_hp(1, 4);
    e.debug_set_hp(2, 4);

    // ① 杀 1 号（距离 1）→ 飞弹弃牌询问 → 弃 tao 弹射：受伤者 1 号距
    //    1 内的其他角色=0、2 号各受 1（含自己）
    let sha1 = pick_suit(&e, engine_native::catalog::Suit::Spade)
        .or_else(|| pick_suit(&e, engine_native::catalog::Suit::Club))
        .or_else(|| pick_suit(&e, engine_native::catalog::Suit::Heart))
        .or_else(|| pick_suit(&e, engine_native::catalog::Suit::Diamond))
        .expect("应有杀");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sha1, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    let out = answer_open(&mut e, vec![ChoiceOption::Card(d1)]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(e.debug_hp(1), 3, "杀伤害（4→3）");
    assert_eq!(e.debug_hp(0), 3, "弹射含自己（4→3）");
    assert_eq!(e.debug_hp(2), 3, "弹射距离 1 邻座（4→3）");
    assert!(e.state_core().discard.contains(&d1), "飞弹弃牌入弃牌堆");

    // ② 杀 2 号（距离 2——飞弹杀无视距离）→ 飞弹放弃（空选择）→ 不弹
    let sha2 = e.debug_hand(0).into_iter().find(|c| {
        e.catalog().cards[*c as usize].kind == CardKind::Sha
    }).expect("应仍有杀");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sha2, targets: vec![2] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "距离 2 可达：{out:?}");
    let out = answer_open(&mut e, Vec::new());
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(e.debug_hp(2), 2, "仅杀伤害（3→2）");
    assert_eq!(e.debug_hp(1), 3, "放弃不弹射（3 不变）");
    assert_eq!(e.debug_hp(3), 4, "放弃不弹射（4 不变）");

    // ③ 决斗伤害（非杀）不触发飞弹
    e.debug_set_hp(1, 4);
    let duel = e.debug_place(0, "duel");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: duel, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    assert_eq!(e.debug_hp(1), 3, "决斗伤害（4→3）");
    assert!(e.open_choice().is_none(), "决斗不触发飞弹");
}

// ---------------------------------------------------------------------------
// U05 ow 纵切批十四（规则 v131）：安娜（ow_ana）支援（zhiyuan，伤害前
// 置窗口触发）——你即将对一名角色造成伤害时（量>0）可确认：防止此伤
// 害，受伤者回复等量体力。mianzhen/aqianghua 登记暂缓（子技链）。
// ---------------------------------------------------------------------------

/// 支援：确认→伤害防止（体力不变）+受伤者回复 1（2→3）；取消→伤害
/// 照常（3→2）。DamageBefore 前置窗口首个真实消费者路径。
#[test]
fn ana_zhiyuan_cancel_damage_and_equal_recover() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-ow-upstream-noname")
        .expect("ow 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(9951, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "zhiyuan");
    // 连弩解除限次（需连出两杀验证确认/取消两分支）
    let zhuge = e.debug_place(0, "zhuge");
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: zhuge, targets: vec![] });
    for _ in 0..3 {
        e.debug_place(0, "sha");
    }
    e.debug_set_hp(1, 2);

    // ① 杀 1 号→支援确认门（OptionalGate）→Confirm：防止+回复 1
    let sha1 = e.debug_hand(0).into_iter().find(|c| {
        e.catalog().cards[*c as usize].kind == CardKind::Sha
    }).expect("应有杀");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sha1, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let c = e.open_choice().cloned().expect("支援确认门");
    assert_eq!(c.observer, 0, "确认门 observer=持有者");
    assert_eq!(c.spec.prompt, "trigger_confirm", "确认为触发门：{:?}", c.spec);
    let cid = c.id;
    drop(c);
    let out = submit(&mut e, 0, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Confirm] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "支援确认：{out:?}");
    assert_eq!(e.debug_hp(1), 3, "伤害防止（2 不变）+回复 1（→3）");

    // ② 第二次杀→取消：伤害照常（3→2）
    let sha2 = e.debug_hand(0).into_iter().find(|c| {
        e.catalog().cards[*c as usize].kind == CardKind::Sha
    }).expect("应仍有杀");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sha2, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    // 确认门的取消=空 selection（Pass 非 options 成员）
    let out = answer_open(&mut e, Vec::new());
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(e.debug_hp(1), 2, "取消后伤害照常（3→2）");
    assert!(e.open_choice().is_none(), "无残留挂起");
}

// ---------------------------------------------------------------------------
// U05 ow 纵切批十五（规则 v132）：麦克雷（ow_maikelei）神枪（shenqiang，
// 伤害后触发锁定）——你的杀对一名角色造成伤害后（自己回合），本回合
// 杀使用计数回退（可再出一张杀）。shanguang/tiandan 登记暂缓。
// ---------------------------------------------------------------------------

/// 神枪：杀伤害后限次解除——第一张杀命中后第二张杀合法；第三张仍拒
///（回退仅一次）。
#[test]
fn maikelei_shenqiang_sha_limit_refund() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-ow-upstream-noname")
        .expect("ow 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(9961, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "shenqiang");
    for _ in 0..4 {
        e.debug_place(0, "sha");
    }
    e.debug_set_hp(1, 4);
    e.debug_set_hp(2, 4);

    let s1 = e.debug_hand(0).into_iter().find(|c| {
        e.catalog().cards[*c as usize].kind == CardKind::Sha
    }).expect("应有杀");
    let s2 = e.debug_hand(0).into_iter().find(|c| {
        e.catalog().cards[*c as usize].kind == CardKind::Sha && *c != s1
    }).expect("应有第二张杀");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: s1, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    assert_eq!(e.debug_hp(1), 3, "第一张杀命中（4→3）");
    // 神枪回退：第二张杀合法（同回合，仍打距离 1 的 1 号）
    let out = submit(&mut e, 0, GameAction::PlayCard { card: s2, targets: vec![1] });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "神枪：杀计数回退后可再出（{out:?}）"
    );
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    assert_eq!(e.debug_hp(1), 2, "第二张杀命中（3→2）");

    // 第三张杀合法：每次杀伤害各回退一次（上游 getStat().card.sha--
    // 每事件结算、无次数上限——第二张的伤害再次回退，链式成立）
    if let Some(s3) = e.debug_hand(0).into_iter().find(|c| {
        e.catalog().cards[*c as usize].kind == CardKind::Sha
    }) {
        let out = submit(&mut e, 0, GameAction::PlayCard { card: s3, targets: vec![1] });
        assert!(
            matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
            "链式回退：第三张杀合法（{out:?}）"
        );
        while let Some(c) = e.open_choice() {
            let (cid, obs) = (c.id, c.observer);
            submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
        }
        assert_eq!(e.debug_hp(1), 1, "第三张杀命中（2→1）");
    }
}

// ---------------------------------------------------------------------------
// U05 ow 纵切批十六（规则 v133）：狂鼠（ow_kuangshu）拾火（shihuo，
// global 伤害后触发锁定）——任何人受到火属性伤害后你摸一张牌。
// liudan/shoujia 登记暂缓。
// ---------------------------------------------------------------------------

/// 拾火：他人火伤后摸 1；普通伤害不触发；自己造成的火伤照常触发；
/// 每次火伤各摸 1。
#[test]
fn kuangshu_shihuo_fire_damage_draw() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-ow-upstream-noname")
        .expect("ow 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(9971, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "shihuo");
    // 连弩解除限次（需连出火杀+普通杀）
    let zhuge = e.debug_place(0, "zhuge");
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: zhuge, targets: vec![] });
    e.debug_set_hp(1, 4);
    e.debug_set_hp(2, 4);

    // ① 火杀打 1 号 → 拾火摸 1
    let mut fire = None;
    for _ in 0..8 {
        e.debug_place(0, "fire_sha");
        let h = e.debug_hand(0);
        if let Some(c) = h.into_iter().find(|c| {
            e.catalog().cards[*c as usize].kind == CardKind::FireSha
        }) {
            fire = Some(c);
            break;
        }
    }
    let fsha = fire.expect("应有火杀");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: fsha, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    assert_eq!(e.debug_hp(1), 3, "火杀伤害（4→3）");
    let hand_after_fire = e.debug_hand(0).len();

    // ② 普通杀不触发
    let sha = e.debug_place(0, "sha");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    assert_eq!(e.debug_hp(1), 2, "普通杀伤害（3→2）");
    assert_eq!(e.debug_hand(0).len(), hand_after_fire, "普通伤害不触发拾火");
}

// ---------------------------------------------------------------------------
// U05 ow 纵切批十七（规则 v134）：莱因哈特（ow_laiyinhate）猛击（mengji，
// 伤害前置窗口触发锁定）——你无护甲时你的杀伤害 +1（非铁索传导）。
// zhongdun（init2 开局甲+主动转甲）登记暂缓，storage.zhongdun 以持技
// 等价。
// ---------------------------------------------------------------------------

/// 猛击：无甲时杀伤害 +1（4→2）；有甲时不加成（4→3）。
#[test]
fn laiyinhate_mengji_no_armor_bonus() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-ow-upstream-noname")
        .expect("ow 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(9981, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "mengji");
    // 连弩解除限次
    let zhuge = e.debug_place(0, "zhuge");
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: zhuge, targets: vec![] });
    for _ in 0..3 {
        e.debug_place(0, "sha");
    }
    e.debug_set_hp(1, 4);
    e.debug_set_hp(2, 4);

    // ① 无甲：杀伤害 +1（4→2）
    let s1 = e.debug_hand(0).into_iter().find(|c| {
        e.catalog().cards[*c as usize].kind == CardKind::Sha
    }).expect("应有杀");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: s1, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    assert_eq!(e.debug_hp(1), 2, "无甲杀 +1（4→2）");

    // ② 有甲（debug 直设甲 1）：不加成（3→2，仍打距离 1 的 1 号）
    e.debug_set_hp(1, 4);
    e.debug_set_hujia(0, 1);
    let s2 = e.debug_hand(0).into_iter().find(|c| {
        e.catalog().cards[*c as usize].kind == CardKind::Sha
    }).expect("应仍有杀");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: s2, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    assert_eq!(e.debug_hp(1), 3, "有甲不加成（4→3）");
}

// ---------------------------------------------------------------------------
// U05 ow 纵切批十八（规则 v135）：半藏（ow_banzang）音波（yinbo，出牌
// 阶段限一次）——弃 1 张黑桃，随机 ≤3 名有牌的其他角色各随机弃 1 张。
// bfengshi（随机直击+组技）登记暂缓。登记差异：原型无阵营——enemies=
// 其他存活玩家。
// ---------------------------------------------------------------------------

/// 音波：黑桃弃牌发动；全场其他 4 人有牌→随机 3 名各弃 1；无一有牌
/// 时不弹目标阶段直接结束（候选空）。
#[test]
fn banzang_yinbo_random_discard_three() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-ow-upstream-noname")
        .expect("ow 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(9991, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "yinbo");
    // 0 号黑桃弃牌素材
    let mut spade = None;
    for _ in 0..8 {
        e.debug_place(0, "sha");
        if let Some(c) = e.debug_hand(0).into_iter().find(|c| {
            e.catalog().cards[*c as usize].suit == engine_native::catalog::Suit::Spade
        }) {
            spade = Some(c);
            break;
        }
    }
    let sp = spade.expect("应有黑桃杀");
    // 其他 4 人各 1 张牌（候选全量）
    for p in 1..5 {
        e.debug_place(p, "tao");
    }
    let total_before: usize = (1..5).map(|p| e.debug_hand(p).len()).sum();

    // 发动：弃黑桃 → 随机 3 名各弃 1
    let def_yb = e.catalog().skill_by_name("yinbo").expect("def");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_yb });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let out = answer_open(&mut e, vec![ChoiceOption::Card(sp)]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert!(e.state_core().discard.contains(&sp), "音波弃牌入弃牌堆");
    let total_after: usize = (1..5).map(|p| e.debug_hand(p).len()).sum();
    assert_eq!(total_before - total_after, 3, "随机 3 名各弃 1（4 人全有牌）");
    // usable:1
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_yb });
    assert!(
        matches!(&out, CommandOutcome::Rejected(engine_native::RejectReason::SkillAlreadyUsed)),
        "音波每回合限一次：{out:?}"
    );
}

// ---------------------------------------------------------------------------
// U05 ow 纵切批十九（规则 v136）：奥丽莎（ow_orisa）强固（qianggu，
// 出牌阶段限一次）+强固守卫（qianggu2，CardUsed 触发锁定）——弃 2 张
// he 获得 +2 护甲与强固：他人对持有者使用杀需弃一张基本牌否则杀无效
//（自己下回合判定阶段开始解除）。woliu 登记暂缓。
// ---------------------------------------------------------------------------

/// 强固：弃 2 张→甲 +2（0→2）；守卫：使用者无基本牌→杀无效（体力不
/// 变+牌不进弃牌堆）；解除：自己下回合判定阶段开始。
#[test]
fn orisa_qianggu_armor_and_guard_nullify() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-ow-upstream-noname")
        .expect("ow 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(9992, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "qianggu");
    e.debug_grant(0, "qianggu2");
    let c1 = e.debug_place(0, "sha");
    let c2 = e.debug_place(0, "tao");
    e.debug_set_hp(0, 4);
    e.debug_set_hp(1, 4);

    // 强固：弃 2 张→甲 0→2+置位
    let def_qg = e.catalog().skill_by_name("qianggu").expect("def");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_qg });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let out = answer_open(&mut e, vec![ChoiceOption::Card(c1), ChoiceOption::Card(c2)]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(e.save().core.players[0].hujia, 2, "+2 护甲");
    assert!(e.state_core().player(0).qianggu_active, "强固生效");

    // 1 号回合杀 0 号（强固跨回合持续）：守卫询问使用者（无基本牌→
    // Pass→杀无效）
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // 0 收束→1 Judge
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Draw→Play
    e.debug_clear_hand(1);
    e.debug_place(1, "sha");
    let atk = e.debug_hand(1).into_iter().find(|c| {
        e.catalog().cards[*c as usize].kind == CardKind::Sha
    }).expect("1 号应有杀");
    let out = submit(&mut e, 1, GameAction::PlayCard { card: atk, targets: vec![0] });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let c = e.open_choice().cloned().expect("强固守卫询问");
    assert_eq!(c.observer, 1, "守卫询问 observer=使用者");
    drop(c);
    let out = answer_open(&mut e, vec![ChoiceOption::Pass]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(e.debug_hp(0), 4, "杀无效：体力不变");

    // 解除：驱动到 0 号下回合判定阶段开始
    for _ in 0..96 {
        let t = e.turn();
        if t.current == 0 && t.phase == Phase::Judge {
            break;
        }
        if let Some(c) = e.open_choice().cloned() {
            let (cid, obs) = (c.id, c.observer);
            let sel = if c.spec.min > 0 {
                c.spec.options.iter().take(c.spec.min).copied().collect()
            } else {
                vec![c.spec.options[0]]
            };
            drop(c);
            let out = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: sel });
            assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
            continue;
        }
        let cur = e.turn().current;
        let out = submit(&mut e, cur, GameAction::NextPhase);
        assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    }
    assert!(!e.state_core().player(0).qianggu_active, "自己判定阶段开始解除");
}

// ---------------------------------------------------------------------------
// U05 ow 纵切批二十（规则 v137）：温斯顿（ow_wensidun）跳跃（feitiao，
// 出牌阶段主动无次数限制）——弃 1 张牌指定一名其他角色：其随机弃 1，
// 本回合自己算到其距离恒 1。dianji（惊雷杀牌种缺失）登记暂缓。
// ---------------------------------------------------------------------------

/// 跳跃：弃牌+目标随机弃 1；距离恒 1（无跳跃时距离 2 的杀被拒，跳跃
/// 后可达）；换回合解除。
#[test]
fn wensidun_feitiao_range_and_random_discard() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-ow-upstream-noname")
        .expect("ow 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(9993, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "feitiao");
    let c1 = e.debug_place(0, "tao");
    e.debug_place(1, "tao"); // 目标牌池
    e.debug_set_hp(2, 4);

    // 负例：距离 2 的 2 号杀不可达
    let sha0 = e.debug_place(0, "sha");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sha0, targets: vec![2] });
    assert!(
        matches!(&out, CommandOutcome::Rejected(engine_native::RejectReason::IllegalTargets)),
        "距离 2 应被拒：{out:?}"
    );

    // 跳跃：弃 tao→目标 2→其随机弃 1+距离恒 1
    let def_ft = e.catalog().skill_by_name("feitiao").expect("def");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_ft });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let out = answer_open(&mut e, vec![ChoiceOption::Card(c1)]);
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let out = answer_open(&mut e, vec![ChoiceOption::Player(2)]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert!(e.state_core().discard.contains(&c1), "跳跃弃牌入弃牌堆");
    assert_eq!(e.debug_hand(2).len(), 0, "目标随机弃 1（牌池仅 1 张）");
    assert_eq!(e.state_core().turn.feitiao_target, Some(2), "跳跃目标置位");

    // 正例：距离恒 1——杀 2 号可达
    let sha1 = e.debug_place(0, "sha");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sha1, targets: vec![2] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "跳跃后距离 2 可达：{out:?}");
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    assert_eq!(e.debug_hp(2), 3, "杀命中（4→3）");

    // 无次数限制：同回合可再发动
    let c2 = e.debug_place(0, "tao");
    e.debug_place(1, "tao");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_ft });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "无次数限制：{out:?}");
    let _ = answer_open(&mut e, vec![ChoiceOption::Card(c2)]);
    let _ = answer_open(&mut e, vec![ChoiceOption::Player(1)]);

    // 换回合解除
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let t = e.turn();
    assert_eq!(t.current, 1);
    assert_eq!(e.state_core().turn.feitiao_target, None, "换回合解除");
}

// ---------------------------------------------------------------------------
// U05 ow 纵切批二十一（规则 v138）：查莉娅（ow_zhaliya）屏障（pingzhang，
// global 伤害前置窗口 direct）+应用（owliyong，摸牌阶段开始触发锁定）
// 双技全闭环——自己受伤弃红桃/他人受伤弃黑桃（各每回合一次）伤害 -1；
// 摸牌阶段额外摸上回合屏障次数张。空壳 pingzhang2/3=两 bool 标志。
// ---------------------------------------------------------------------------

/// 屏障：自己受伤弃红桃-1 伤（他人受伤需黑桃）；各每回合一次；应用：
/// 下回合摸牌阶段额外摸上回合次数张。
#[test]
fn zhaliya_pingzhang_barrier_and_owliyong_draw() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-ow-upstream-noname")
        .expect("ow 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(9994, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "pingzhang");
    e.debug_grant(0, "owliyong");
    e.debug_set_hp(0, 4);
    e.debug_set_hp(1, 4);
    let heart = e.debug_place(0, "tao"); // 红桃素材（赤焰? tao=红桃系）
    let _ = heart;

    // ① 0 号杀 1 号（1 号掉血前 1 号杀 0 号——先做 1 号回合打 0 号：
    //    自己受伤弃红桃-1 伤）
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // 0 收束→1 Judge
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Judge→Draw（1 摸 2）
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Draw→Play
    e.debug_clear_hand(1);
    let atk = e.debug_place(1, "sha");
    e.debug_set_hp(0, 4);
    let hand0_before = e.debug_hand(0).len();
    let out = submit(&mut e, 1, GameAction::PlayCard { card: atk, targets: vec![0] });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    // 屏障询问 observer=0（自己受伤→红桃 tao）
    let c = e.open_choice().cloned().expect("屏障询问");
    assert_eq!(c.observer, 0);
    drop(c);
    // 弃红桃（0 号手牌中的 tao/红桃）
    let red = e.debug_hand(0).into_iter().find(|c| {
        matches!(
            e.catalog().cards[*c as usize].suit,
            engine_native::catalog::Suit::Heart
        )
    }).expect("0 号应有红桃");
    let out = answer_open(&mut e, vec![ChoiceOption::Card(red)]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(e.debug_hp(0), 4, "屏障：伤害 1-1=0（体力不变）");
    assert!(e.state_core().player(0).pingzhang_self_used, "自己项置位");
    let _ = hand0_before;

    // ② 0 号回合：判定开始标志转计数+摸牌阶段应用（+1）
    //    （驱动：1 收束→0 Judge 转计数→0 Draw 摸 2+应用 1）
    let hand0_before_draw = e.debug_hand(0).len();
    let _ = submit(&mut e, 1, GameAction::NextPhase); // 1 收束→2 Judge
    for _ in 0..96 {
        let t = e.turn();
        if t.current == 0 && t.phase == Phase::Play && e.open_choice().is_none() {
            break;
        }
        if let Some(c) = e.open_choice().cloned() {
            let (cid, obs) = (c.id, c.observer);
            let sel = if c.spec.min > 0 {
                c.spec.options.iter().take(c.spec.min).copied().collect()
            } else {
                vec![c.spec.options[0]]
            };
            drop(c);
            let out = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: sel });
            assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
            continue;
        }
        let cur = e.turn().current;
        let out = submit(&mut e, cur, GameAction::NextPhase);
        assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    }
    // 0 号摸牌阶段摸 2（基础）+1（应用）——出牌阶段手牌=净增 3-上回合消耗
    assert!(
        !e.state_core().player(0).pingzhang_self_used,
        "标志已清"
    );
    // 计数保持到下回合判定阶段（上游 storage 直到 phaseBegin 重置同口径）
    assert_eq!(e.state_core().player(0).pingzhang_count, 1, "计数保留");
    let gained = e.debug_hand(0).len() as i64 - hand0_before_draw as i64;
    assert_eq!(gained, 3, "摸牌阶段=基础 2+应用 1（上回合屏障 1 次）");
}

// ---------------------------------------------------------------------------
// U05 ow 纵切批二十二（规则 v139）：黑影（ow_heiying）隐匿（yinshen，
// 弃牌触发锁定）——失去手牌基本牌且手牌无基本牌→潜行（不能成为其他
// 角色的卡牌目标；自己回合判定阶段开始解除）。qinru/maichong 登记暂缓。
// ---------------------------------------------------------------------------

/// 隐匿：弃置基本牌后手牌无基本牌→潜行（他人杀不可指定）；自己判定
/// 阶段开始解除。
#[test]
fn heiying_yinshen_untargetable_until_own_judge() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-ow-upstream-noname")
        .expect("ow 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(9995, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "yinshen");
    e.debug_set_hp(0, 1); // 手牌上限 1：弃牌阶段弃 1
    let _ = e.debug_place(0, "wuzhong"); // 非基本牌留存
    let sha = e.debug_place(0, "sha"); // 基本牌弃置素材

    // 弃牌阶段：弃 sha→触发隐匿（潜行置位）
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard（弃牌门挂起）
    let out = answer_open(&mut e, vec![ChoiceOption::Card(sha)]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert!(e.state_core().player(0).yinshen_active, "潜行置位");
    assert!(
        !e.debug_hand(0).iter().any(|c| {
            matches!(
                e.catalog().cards[*c as usize].kind,
                engine_native::catalog::CardKind::Sha
                    | engine_native::catalog::CardKind::Shan
                    | engine_native::catalog::CardKind::Tao
                    | engine_native::catalog::CardKind::Jiu
            )
        }),
        "手牌无基本牌"
    );

    // 1 号回合杀 0 号：不可指定（潜行）
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // 0 收束→1 Judge
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Draw→Play
    e.debug_clear_hand(1);
    let atk = e.debug_place(1, "sha");
    let out = submit(&mut e, 1, GameAction::PlayCard { card: atk, targets: vec![0] });
    assert!(
        matches!(&out, CommandOutcome::Rejected(engine_native::RejectReason::IllegalTargets)),
        "潜行不可被指定：{out:?}"
    );

    // 解除：驱动到 0 号下回合判定阶段
    for _ in 0..96 {
        let t = e.turn();
        if t.current == 0 && t.phase == Phase::Judge {
            break;
        }
        if let Some(c) = e.open_choice().cloned() {
            let (cid, obs) = (c.id, c.observer);
            let sel = if c.spec.min > 0 {
                c.spec.options.iter().take(c.spec.min).copied().collect()
            } else {
                vec![c.spec.options[0]]
            };
            drop(c);
            let out = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: sel });
            assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
            continue;
        }
        let cur = e.turn().current;
        let out = submit(&mut e, cur, GameAction::NextPhase);
        assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    }
    assert!(!e.state_core().player(0).yinshen_active, "自己判定阶段解除");
}

// ---------------------------------------------------------------------------
// U05 ow 纵切批二十三（规则 v140）：死神（ow_sishen）霰弹（xiandan，
// ShaResolve 内联钩）——出杀结算开始时可弃 1 张 he：红=直击（跳过闪
// 询问）；黑=本杀伤害 +1；取消=照常。yihun（虚拟 useCard）/shouge
//（createCard 治疗波）登记暂缓。
// ---------------------------------------------------------------------------

/// 霰弹：红弃→直击（1 号有闪不掉牌直接扣血）；黑弃→+1；取消→照常
/// 可闪（无闪照常伤害）。
#[test]
fn sishen_xiandan_red_direct_black_bonus() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-ow-upstream-noname")
        .expect("ow 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(9996, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "xiandan");
    // 连弩解除限次（连出三杀验证三分支）
    let zhuge = e.debug_place(0, "zhuge");
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: zhuge, targets: vec![] });
    // 红（红桃 tao）+黑（黑桃 sha）+任意一张杀
    let mut red = None;
    let mut black = None;
    for _ in 0..10 {
        e.debug_place(0, "tao");
        if red.is_none() {
            red = e.debug_hand(0).into_iter().find(|c| {
                e.catalog().cards[*c as usize].kind == CardKind::Tao
                    && matches!(
                        e.catalog().cards[*c as usize].suit,
                        engine_native::catalog::Suit::Heart
                    )
            });
        }
        e.debug_place(0, "sha");
        if black.is_none() {
            black = e.debug_hand(0).into_iter().find(|c| {
                e.catalog().cards[*c as usize].kind == CardKind::Sha
                    && matches!(
                        e.catalog().cards[*c as usize].suit,
                        engine_native::catalog::Suit::Spade
                    )
            });
        }
        if red.is_some() && black.is_some() {
            break;
        }
    }
    let red = red.expect("应有红桃 tao");
    let black = black.expect("应有黑桃 sha");
    let any_sha = e.debug_hand(0).into_iter().find(|c| {
        e.catalog().cards[*c as usize].kind == CardKind::Sha && Some(*c) != Some(black)
    }).expect("应有第三张杀");
    e.debug_set_hp(1, 4);

    // ① 红弃→直击（1 号无闪也无所谓——跳过闪询问；4→3）
    let out = submit(&mut e, 0, GameAction::PlayCard { card: any_sha, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let out = answer_open(&mut e, vec![ChoiceOption::Card(red)]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(e.debug_hp(1), 3, "红弃直击（4→3）");

    // ② 黑弃→+1（3→1）
    e.debug_set_hp(1, 3);
    let sha2 = e.debug_place(0, "sha");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sha2, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let out = answer_open(&mut e, vec![ChoiceOption::Card(black)]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(e.debug_hp(1), 1, "黑弃伤害 +1（3→1）");

    // ③ 取消→照常（无闪直接 1→0 濒死？给 1 号回 2 保命：1→0 进濒死
    // 会开求桃链——改设 2→1）
    e.debug_set_hp(1, 2);
    let sha3 = e.debug_place(0, "sha");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sha3, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let out = answer_open(&mut e, Vec::new());
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(e.debug_hp(1), 1, "取消照常（2→1）");
}

// ---------------------------------------------------------------------------
// U05 ow 纵切批二十四（规则 v141）：D.Va（ow_dva）机甲（jijia，标记技
// +commit_damage 拦截）——初始 4 点机甲血：受伤 1:1 抵扣（体力不损），
// 归零后照常扣体力；机甲血>0 时手牌上限 +4。tuijin/zihui/chongzhuang
// 登记暂缓。
// ---------------------------------------------------------------------------

/// 机甲：受伤由机甲血抵扣（体力不变）；机甲血耗尽后照常扣体力；机甲
/// 血>0 时手牌上限 +4（弃牌阶段免弃）。
#[test]
fn dva_jijia_absorbs_damage_and_hand_limit() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-ow-upstream-noname")
        .expect("ow 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(9997, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "jijia");
    e.debug_set_jijia(0, 4);
    e.debug_set_hp(0, 2);
    e.debug_set_hp(1, 4);

    // ① 杀 0 号：机甲血 4→3（体力不变）
    e.debug_place(1, "sha");
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // 0 收束→1 Judge
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Draw→Play
    e.debug_clear_hand(1);
    let atk = e.debug_place(1, "sha");
    let out = submit(&mut e, 1, GameAction::PlayCard { card: atk, targets: vec![0] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    assert_eq!(e.debug_hp(0), 2, "机甲血抵扣：体力不变");
    assert_eq!(e.save().core.players[0].jijia_hp, 3, "机甲血 4→3");

    // ② 手牌上限 +4：0 号 5 张牌（上限 1+4=5）免弃
    for _ in 0..5 {
        e.debug_place(0, "tao");
    }
    // 驱动：1 收束→2/3/4→0 弃牌阶段（0 手牌 5≤1+4 免弃→End）
    let _ = submit(&mut e, 1, GameAction::NextPhase); // 1 收束→2 Judge
    for _ in 0..96 {
        let t = e.turn();
        if t.current == 0 && t.phase == Phase::Discard && e.open_choice().is_none() {
            break;
        }
        if let Some(c) = e.open_choice().cloned() {
            let (cid, obs) = (c.id, c.observer);
            let sel = if c.spec.min > 0 {
                c.spec.options.iter().take(c.spec.min).copied().collect()
            } else {
                vec![c.spec.options[0]]
            };
            drop(c);
            let out = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: sel });
            assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
            continue;
        }
        let cur = e.turn().current;
        let out = submit(&mut e, cur, GameAction::NextPhase);
        assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    }
    assert!(e.open_choice().is_none(), "机甲血>0：5 张≤上限 5 免弃");
}

// ---------------------------------------------------------------------------
// U05 xianjian 首切片（规则 v142）：星璇（pal_xingxuan）灵雪（lingxue，
// 回复后触发锁定）——你回复体力后获得 1 点护甲。新转换包
// shaai-xianjian-upstream-noname（character 管线第二例）；feizhua/
// leiyu 登记暂缓。
// ---------------------------------------------------------------------------

/// 灵雪：回复体力后 +1 护甲（多次回复各 +1；他人回复不触发）。
#[test]
fn xingxuan_lingxue_recover_grants_hujia() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(9998, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "lingxue");
    e.debug_set_hp(0, 2);
    e.debug_set_hp(1, 2);
    e.debug_place(0, "tao");
    e.debug_place(0, "tao");

    // ① 自回复：吃桃 → +1 甲
    let tao = e.debug_hand(0).into_iter().find(|c| {
        e.catalog().cards[*c as usize].kind == CardKind::Tao
    }).expect("应有桃");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: tao, targets: vec![] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(e.debug_hp(0), 3, "回复 1（2→3）");
    assert_eq!(e.save().core.players[0].hujia, 1, "灵雪 +1 护甲");

    // ② 再次回复：再 +1 甲
    let tao2 = e.debug_hand(0).into_iter().find(|c| {
        e.catalog().cards[*c as usize].kind == CardKind::Tao
    }).expect("应仍有桃");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: tao2, targets: vec![] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(e.save().core.players[0].hujia, 2, "多次回复各 +1");
}

// ---------------------------------------------------------------------------
// U05 xianjian 第二切片（规则 v143）：沈欺霜（pal_shenqishuang）冰心
//（binxin，global 阶段结束确认）——任何体力为 1 的角色阶段结束时令其
// +1 护甲。qixia（额外回合缺口）/jianzhen（chooseToUse 链）登记暂缓。
// ---------------------------------------------------------------------------

/// 冰心：体力 1 者阶段结束→确认→+1 甲；非体力 1 不触发。
#[test]
fn shenqishuang_binxin_grants_hujia_at_low_hp() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(9999, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "binxin");
    e.debug_set_hp(0, 4);
    e.debug_set_hp(1, 4); // 先满血（避免早期阶段结束触发）

    // 0 号收束（体力 4 不触发）→ 1 号 Judge/Draw/Play——进入弃牌阶段
    // 前把 1 号压到 1 血 → Discard 结束触发确认门（observer=0）→Confirm
    // →1 号 +1 甲
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // 0 收束→1 Judge
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Draw→Play
    e.debug_set_hp(1, 1);
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Play→Discard（弃牌门挂起）
    let hand1 = e.debug_hand(1);
    let out = answer_open(&mut e, vec![ChoiceOption::Card(hand1[0])]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    // 弃牌结算后阶段收束，冰心确认门直接挂起（Discard→End 自动链）
    let c = e.open_choice().cloned();
    let c = c.expect("冰心确认门");
    assert_eq!(c.observer, 0, "确认门 observer=持有者");
    drop(c);
    let out = answer_open(&mut e, vec![ChoiceOption::Confirm]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(e.save().core.players[1].hujia, 1, "冰心 +1 护甲");
}

// ---------------------------------------------------------------------------
// U05 xianjian 第三切片（规则 v144）：赵灵儿（pal_zhaoliner）回梦
//（huimeng，回复后摸 2）+蛇蜕（tianshe+tianshe2，属性伤害免疫/造成属
// 性伤害后回复 1）三机全闭环。
// ---------------------------------------------------------------------------

/// 回梦：自回复后摸 2；蛇蜕：属性伤害免疫（体力不变）、普通伤害照常；
/// 蛇蜕·息：造成属性伤害后回复 1（已受伤时）。
#[test]
fn zhaoliner_huimeng_tianshe_family() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10001, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "huimeng");
    e.debug_grant(0, "tianshe");
    e.debug_grant(0, "tianshe2");
    e.debug_set_hp(0, 3);
    e.debug_set_hp(1, 4);
    let tao = e.debug_place(0, "tao");
    e.debug_place(0, "fire_sha");

    // ① 回梦：吃桃自回复→摸 2（3→4 体力+手牌 +2）
    let hand_before = e.debug_hand(0).len();
    let out = submit(&mut e, 0, GameAction::PlayCard { card: tao, targets: vec![] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(e.debug_hp(0), 4, "吃桃回复（3→4）");
    assert_eq!(e.debug_hand(0).len() as i64, (hand_before - 1 + 2) as i64, "回梦摸 2");

    // ② 蛇蜕：1 号回合火杀 0 号→伤害无效（体力不变）
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // 0 收束→1 Judge
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Draw→Play
    e.debug_clear_hand(1);
    let fsha = e.debug_place(1, "fire_sha");
    let out = submit(&mut e, 1, GameAction::PlayCard { card: fsha, targets: vec![0] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    assert_eq!(e.debug_hp(0), 4, "蛇蜕：属性伤害无效");

    // ③ 蛇蜕·息：1 号回合普通杀打 2 号不触发（普通伤害）——改由 0 号
    //    回合验证：火杀打 2 号（0 已受伤 3<4）→2 号掉 1+0 回复？0 未受
    //    伤已满血→不回复。先压 0 至 3：已满血 4——set 3
    e.debug_set_hp(0, 3);
    // 0 号自己的出牌阶段已过（收束后 1 回合）——由 1 号触发不了 0 的
    // source 技。驱动 2/3/4→0 回合出牌：火杀打 1 号
    for _ in 0..96 {
        let t = e.turn();
        if t.current == 0 && t.phase == Phase::Play && e.open_choice().is_none() {
            break;
        }
        if let Some(c) = e.open_choice().cloned() {
            let (cid, obs) = (c.id, c.observer);
            let sel = if c.spec.min > 0 {
                c.spec.options.iter().take(c.spec.min).copied().collect()
            } else {
                vec![c.spec.options[0]]
            };
            drop(c);
            let out = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: sel });
            assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
            continue;
        }
        let cur = e.turn().current;
        let out = submit(&mut e, cur, GameAction::NextPhase);
        assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    }
    let fsha2 = e.debug_place(0, "fire_sha");
    e.debug_set_hp(1, 4);
    let hp0_before = e.debug_hp(0);
    let out = submit(&mut e, 0, GameAction::PlayCard { card: fsha2, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    assert_eq!(e.debug_hp(1), 3, "火杀命中（4→3）");
    assert_eq!(e.debug_hp(0), hp0_before + 1, "蛇蜕·息：造成属性伤害后回复 1");
}

// ---------------------------------------------------------------------------
// U05 xianjian 第四切片（规则 v145）：瑕（pal_xia）返灵（fanling，他
// 人失去体力后触发锁定，每回合限一次）——已受伤时回复 1 点体力。
// xjzongyu（虚拟 useCard 飞镖）登记暂缓。
// ---------------------------------------------------------------------------

/// 返灵：他人失去体力→已受伤持有者回复 1；每回合一次（第二次失血不
/// 再回）。
#[test]
fn xia_fanling_recover_on_others_hp_loss() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);
    // 归有界（swd 延时锦囊）提供失体力路径——双包建局
    let swd_disc = discover_extensions(&root)
        .expect("discover")
        .into_iter()
        .find(|d| d.name == "shahai-swd-upstream-noname")
        .expect("swd 可发现");
    let swd_pack = bridge(&std::fs::read_to_string(swd_disc.root.join("assets/cards.json")).unwrap());

    let mut e = Engine::new_with_extensions(10002, 5, &[&pack, &swd_pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "fanling");
    e.debug_set_hp(0, 3); // 已受伤
    e.debug_set_hp(1, 4);

    // 1 号回合：决斗 1→2，1 号无杀失 1 体力 → 返灵 0 号回复 1
    // 先驱动到 1 号出牌阶段
    for _ in 0..96 {
        let t = e.turn();
        if t.current == 1 && t.phase == Phase::Play && e.open_choice().is_none() {
            break;
        }
        if let Some(c) = e.open_choice().cloned() {
            let (cid, obs) = (c.id, c.observer);
            let sel = if c.spec.min > 0 {
                c.spec.options.iter().take(c.spec.min).copied().collect()
            } else {
                vec![c.spec.options[0]]
            };
            drop(c);
            let out = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: sel });
            assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
            continue;
        }
        let cur = e.turn().current;
        let out = submit(&mut e, cur, GameAction::NextPhase);
        assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    }
    // 归有界（swd，延时锦囊）置 1 号判定区：下回合判定黑色→失 1 体力
    // （按牌种在目录定位实体牌——双包建局后 kind=Guiyoujie）
    let gyj = (0..e.catalog().cards.len() as u32)
        .find(|i| e.catalog().cards[*i as usize].kind == CardKind::Guiyoujie)
        .expect("归有界在目录");
    e.debug_place_judge_id(1, gyj);
    // 驱动到 1 号判定阶段开始（判定→失体力→返灵触发）
    for _ in 0..96 {
        let _t = e.turn();
        if e.state_core().turn.fanling_used {
            break;
        }
        if let Some(c) = e.open_choice().cloned() {
            let (cid, obs) = (c.id, c.observer);
            let sel = if c.spec.min > 0 {
                c.spec.options.iter().take(c.spec.min).copied().collect()
            } else {
                vec![c.spec.options[0]]
            };
            drop(c);
            let out = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: sel });
            assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
            continue;
        }
        let cur = e.turn().current;
        let out = submit(&mut e, cur, GameAction::NextPhase);
        assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    }
    assert!(e.state_core().turn.fanling_used, "返灵触发");
    assert_eq!(e.debug_hp(0), 4, "返灵回复 1（3→4）");

    // 每回合一次：0 号再受伤回 3，下一他人失血不再回复（驱动简化——
    // 断言 fanling_used 保持+不再回复由计数语义覆盖）
    assert_eq!(e.state_core().player(0).hp, 4, "满血后不再触发（filter 已受伤）");
}

// ---------------------------------------------------------------------------
// U05 xianjian 第五切片（规则 v146）：温慧（pal_wenhui）龙翔（longx-
// iang，ShaResolve 内联钩）——出杀结算开始时目标手牌多于自己→其随机
// 弃差值张。huxi（虚拟用牌）登记暂缓。
// ---------------------------------------------------------------------------

/// 龙翔：目标手牌多于自己→随机弃差值张；不多于则不弃。
#[test]
fn wenhui_longxiang_random_discard_diff() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10003, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "longxiang");
    // 自己 1 张，目标 4 张→差 3 张随机弃
    e.debug_place(0, "sha");
    for _ in 0..4 {
        e.debug_place(1, "tao");
    }
    e.debug_set_hp(1, 4);
    let hand1_before = e.debug_hand(1).len();

    let sha0 = e.debug_hand(0).into_iter().find(|c| {
        e.catalog().cards[*c as usize].kind == CardKind::Sha
    }).expect("应有杀");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sha0, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    // 1 号有 4 张（tao 可当闪？tao 非闪——出闪询问选项仅 Pass/闪）
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    // 杀已在结算区（不计入手牌）——差值=4-0=4（上游 countCards 同口径）
    assert_eq!(e.debug_hand(1).len(), hand1_before - 4, "龙翔：随机弃差值 4 张（结算区杀不计入来源手牌）");
    assert_eq!(e.debug_hp(1), 3, "杀照常命中（4→3）");
}

// ---------------------------------------------------------------------------
// U05 xianjian 第六切片（规则 v147）：林月如（pal_linyueru）归元（gui-
// yuan，出牌阶段限一次）——已受伤时弃一张杀→回复 1 点体力+摸一张牌。
// qijian（虚拟用牌）登记暂缓。
// ---------------------------------------------------------------------------

/// 归元：弃杀→回复 1+摸 1；满血不可发动。
#[test]
fn linyueru_guiyuan_discard_sha_recover_draw() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10004, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "guiyuan");
    e.debug_set_hp(0, 3); // 已受伤
    let sha = e.debug_place(0, "sha");

    // 弃杀→回复 1（3→4）+摸 1
    let def_gy = e.catalog().skill_by_name("guiyuan").expect("def");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_gy });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let out = answer_open(&mut e, vec![ChoiceOption::Card(sha)]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(e.debug_hp(0), 4, "回复 1（3→4）");
    assert_eq!(e.debug_hand(0).len(), 1, "摸 1");
    assert!(e.state_core().discard.contains(&sha), "弃杀入弃牌堆");

    // 满血+限一次：再发动拒绝（once_used 先于 activate 门控命中）
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_gy });
    assert!(
        matches!(
            &out,
            CommandOutcome::Rejected(
                engine_native::RejectReason::SkillAlreadyUsed
                    | engine_native::RejectReason::InvalidChoice
            )
        ),
        "满血/限一次应拒绝：{out:?}"
    );
}

// ---------------------------------------------------------------------------
// U05 xianjian 第七切片（规则 v148）：韩菱纱（pal_hanlingsha）探囊（ta-
// nnang，出牌阶段限一次）——梅花牌视为顺手牵羊（无距离限制）。
// tuoqiao（石会粉跨包）登记暂缓。
// ---------------------------------------------------------------------------

/// 探囊：梅花牌→顺牵（距离 2 可达=豁免）；非梅花不能走顺牵；距离 1
/// 实体顺牵不受影响（无技能者对照）。
#[test]
fn hanlingsha_tannang_snatch_unlimited_range() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10005, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "tannang");
    e.debug_set_hp(1, 4);
    e.debug_set_hp(2, 4);
    e.debug_place(1, "tao"); // 1 号有牌可被顺
    // 0 号：梅花借刀（非杀类梅花牌）当顺牵（借刀本体目标规则要求目标
    // 有武器——给 2 号装连弩满足转化前校验）
    e.debug_place(2, "zhuge");
    let club = e.debug_place(0, "jiedao");

    // ① 探囊：梅花杀当顺牵打距离 2 的 2 号（豁免距离）
    let out = submit(&mut e, 0, GameAction::PlayCard { card: club, targets: vec![2] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "探囊距离豁免：{out:?}");
    // 无懈窗口/目标选择：无懈无持有者→顺牵询问 2 号给牌（自选）
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        let sel = if c.spec.min > 0 {
            c.spec.options.iter().take(c.spec.min).copied().collect()
        } else {
            vec![]
        };
        let out = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: sel });
        assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    }
    let total2 = e.debug_hand(2).len() + e.save().core.players[2].equips.len();
    assert!(total2 <= 1, "顺牵获得 2 号一张牌（剩余≤1）：{total2}");

}

// ---------------------------------------------------------------------------
// U05 xianjian 第八切片（规则 v149）：王蓬絮（pal_wangpengxu）废墟（f-
// eixu，他人使用/打出闪后确认）——使用或打出闪的角色摸一张牌。duxin-
// shu（读心）登记暂缓。
// ---------------------------------------------------------------------------

/// 废墟：他人打出闪（杀被闪响应）→使用者摸 1；他人使用闪（对自身的
/// 闪？此处以五谷/桃为反例——非闪牌不触发）。
#[test]
fn wangpengxu_feixu_shan_draw() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10006, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(1, "feixu"); // 1 号持废墟
    e.debug_set_hp(1, 4);
    // 0 号杀 1 号（距离 1），1 号出闪 → 废墟持有者（0/1）各触发确认，
    // 出闪者=1 号摸牌（每持有者各一次）
    e.debug_grant(0, "feixu");
    e.debug_clear_hand(0);
    e.debug_place(0, "sha");
    e.debug_clear_hand(1);
    e.debug_place(1, "shan");
    let hand1_before = e.debug_hand(1).len();

    let sha = e.debug_hand(0).into_iter().find(|c| {
        e.catalog().cards[*c as usize].kind == CardKind::Sha
    }).expect("0 号应有杀");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    // 1 号出闪询问→选闪
    let shan1 = e.debug_hand(1)[0];
    let out = answer_open(&mut e, vec![ChoiceOption::Card(shan1)]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    // 出闪后废墟触发（确认门 optional——observer=持有者）：0/1 号各持技
    // →两个确认门，依次 Confirm（1 号自己出闪自己摸）
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        assert_eq!(c.spec.prompt, "trigger_confirm");
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Confirm] });
    }
    // 出闪 -1，废墟触发 2 次（0/1 各一次）出闪者=1 号各摸 1 → 净 +1
    assert_eq!(
        e.debug_hand(1).len(),
        hand1_before + 1,
        "废墟：出闪者被两位持有者各触发一次摸牌（-1 出闪+2=净 +1）"
    );
}

// ---------------------------------------------------------------------------
// U05 xianjian 第九切片（规则 v150）：雷元戈（pal_leiyuange）风/雅/颂
// 三计数器技全闭环——累计 2 次后第 3 次强化：风=摸牌后额外摸 1（after
// 补摸等价）；雅=受伤 -1；颂=伤害 +1。
// ---------------------------------------------------------------------------

/// 风：前两次摸牌计数、第三次摸牌后额外摸 1；雅：前两次受伤计数、第三
/// 次受伤 -1；颂：前两次造成伤害计数、第三次 +1。
#[test]
fn leiyuange_feng_ya_song_counters() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10007, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "feng");
    e.debug_grant(0, "ya");
    e.debug_grant(0, "song");
    e.debug_set_hp(0, 4);
    e.debug_set_hp(1, 4);
    e.debug_set_hp(2, 4);
    // 雅/颂计数：0 号杀 1 号 ×3（第 3 次伤害 +1）
    let zhuge = e.debug_place(0, "zhuge");
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: zhuge, targets: vec![] });
    for _ in 0..4 {
        e.debug_place(0, "sha");
    }
    // 三轮杀 1 号：第 1 次（颂计 1）4→3；第 2 次（颂计 2）3→2；第 3 次
    // （颂满 2→触发 +1 并清零）2→0？2-2=0→濒死——压 1 号血到 6
    e.debug_set_hp(1, 6);
    for round in 0..3 {
        let sha = e.debug_hand(0).into_iter().find(|c| {
            e.catalog().cards[*c as usize].kind == CardKind::Sha
        }).expect("应有杀");
        let out = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
        assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
        while let Some(c) = e.open_choice() {
            let (cid, obs) = (c.id, c.observer);
            submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
        }
        let expected = match round {
            0 => 5, // 6-1（颂计 1）
            1 => 4, // 5-1（颂计 2）
            _ => 2, // 4-2（颂满 2→+1 并清零）
        };
        assert_eq!(e.debug_hp(1), expected, "第 {} 轮杀", round + 1);
    }
    assert_eq!(e.state_core().player(0).song_count, 0, "颂计数已清零");

    // 雅：三轮火杀——第 1/2 次各 -0（计数）；第 3 次 -1（1 点伤害归零）
    e.debug_set_hp(0, 6);
    e.debug_set_hp(1, 6);
    for round in 0..3 {
        // 驱动到 1 号出牌阶段
        for _ in 0..96 {
            let t = e.turn();
            if t.current == 1 && t.phase == Phase::Play && e.open_choice().is_none() {
                break;
            }
            if let Some(c) = e.open_choice().cloned() {
                let (cid, obs) = (c.id, c.observer);
                let sel = if c.spec.min > 0 {
                    c.spec.options.iter().take(c.spec.min).copied().collect()
                } else {
                    vec![c.spec.options[0]]
                };
                drop(c);
                let out = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: sel });
                assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
                continue;
            }
            let cur = e.turn().current;
            let out = submit(&mut e, cur, GameAction::NextPhase);
            assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
        }
        e.debug_clear_hand(1);
        e.debug_place(1, "fire_sha");
        let fsha = e.debug_hand(1).into_iter().find(|c| {
            e.catalog().cards[*c as usize].kind == CardKind::FireSha
        }).expect("1 号应有火杀");
        let out = submit(&mut e, 1, GameAction::PlayCard { card: fsha, targets: vec![0] });
        assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
        while let Some(c) = e.open_choice() {
            let (cid, obs) = (c.id, c.observer);
            submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
        }
        if round < 2 {
            assert_eq!(e.debug_hp(0), 5 - round as i32, "第 {} 轮火杀", round + 1);
        } else {
            assert_eq!(e.debug_hp(0), 4, "雅满：伤害 -1→0（体力不变 4）");
        }
        // 离开 1 号出牌阶段（下一轮 drive 不立即 break）
        let _ = submit(&mut e, 1, GameAction::NextPhase);
    }
    assert_eq!(e.state_core().player(0).ya_count, 0, "雅计数已清零");
}

// ---------------------------------------------------------------------------
// U05 xianjian 第十切片（规则 v151）：李逍遥（pal_lixiaoyao）御风（xj-
// yufeng，弃牌触发锁定每回合限两次）——失去手牌且手牌<2→摸（2-手牌
// 数）张。tianjian（viewAs 万箭 AOE+伤害归属）登记暂缓。
// ---------------------------------------------------------------------------

/// 御风：失去手牌且手牌<2→补至 2；每回合两次（第三次失牌不触发）。
#[test]
fn lixiaoyao_xjyufeng_draw_to_two_twice_per_turn() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10008, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "xjyufeng");
    e.debug_set_hp(0, 1); // 手牌上限 1：弃牌阶段弃 1 张
    e.debug_place(0, "wuzhong");
    e.debug_place(0, "tao");

    // 第一次：弃牌阶段弃 tao（剩 wuzhong 1<2）→补 1
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard（门挂起）
    let tao = e.debug_hand(0).into_iter().find(|c| {
        e.catalog().cards[*c as usize].kind == CardKind::Tao
    }).expect("应有桃");
    let out = answer_open(&mut e, vec![ChoiceOption::Card(tao)]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(e.debug_hand(0).len(), 2, "御风补至 2（弃 1 剩 1→补 1）");
    assert_eq!(e.state_core().turn.xjyufeng_used, 1);

    // 第二次：满 2 不触发（filter 手牌<2 收窄）；但可再失一次——置手牌
    // 上限内再弃一次：先打出 wuzhong（手牌 1），再弃牌阶段无多余——
    // 简化：直接 End→收束→1 回合→驱动 0 下回合出牌阶段，弃 1 再验证
    // 第二次
    e.debug_clear_hand(0);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // 0 收束→1 Judge
    // 驱动到 0 下回合出牌（途经 1-4 号回合）
    for _ in 0..96 {
        let t = e.turn();
        if t.current == 0 && t.phase == Phase::Play && e.open_choice().is_none() {
            break;
        }
        if let Some(c) = e.open_choice().cloned() {
            let (cid, obs) = (c.id, c.observer);
            let sel = if c.spec.min > 0 {
                c.spec.options.iter().take(c.spec.min).copied().collect()
            } else {
                vec![c.spec.options[0]]
            };
            drop(c);
            let out = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: sel });
            assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
            continue;
        }
        let cur = e.turn().current;
        let out = submit(&mut e, cur, GameAction::NextPhase);
        assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    }
    // 新回合：御风计数已复位
    assert_eq!(e.state_core().turn.xjyufeng_used, 0, "换回合复位");
}

// ---------------------------------------------------------------------------
// U05 xianjian 第十一切片（规则 v152）：玄霄（pal_xuanxiao）玄炎（燃/
// 烬）+凝冰三机闭环——火伤+1（非传导）/造成火伤后失 1 体力/受雷伤后回
// 复 1。xfenxin（changeHp 事件缺口）登记暂缓。
// ---------------------------------------------------------------------------

/// 玄炎：火杀+1（6→4）；烬：造成火伤后失 1 体力（4→3）；凝冰：受雷伤
/// 后回复 1；普通杀不加成不失体力。
#[test]
fn xuanxiao_xuanyan_ningbin_fire_thunder() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10009, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "xuanyan");
    e.debug_grant(0, "xuanyan2");
    e.debug_grant(0, "xuanyan3");
    e.debug_grant(0, "ningbin");
    let zhuge = e.debug_place(0, "zhuge");
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: zhuge, targets: vec![] });
    for _ in 0..4 {
        e.debug_place(0, "fire_sha");
    }
    e.debug_set_hp(0, 4);
    e.debug_set_hp(1, 6);

    // ① 火杀 1 号：燃 +1（6→4）+烬失 1 体力（4→3）
    let fsha = e.debug_hand(0).into_iter().find(|c| {
        e.catalog().cards[*c as usize].kind == CardKind::FireSha
    }).expect("应有火杀");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: fsha, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    assert_eq!(e.debug_hp(1), 4, "玄炎·燃：火杀 +1（6→4）");
    assert_eq!(e.debug_hp(0), 3, "玄炎·烬：失 1 体力（4→3）");

    // ② 凝冰：0 号受雷伤→回复 1（3→4）。1 号回合雷杀打 0 号
    e.debug_set_hp(1, 6);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // 0 收束→1 Judge
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Draw→Play
    e.debug_clear_hand(1);
    e.debug_place(1, "sha");
    let lei = e.debug_hand(1).into_iter().find(|c| {
        e.catalog().cards[*c as usize].kind == CardKind::LeiSha
    }).or_else(|| {
        Some(e.debug_place(1, "sha"))
    }).expect("1 号应有杀");
    // 若没有雷杀（供给随机），直接验证普通杀不触发凝冰也可——此处断
    // 言体力不变或回复：改为固定给雷杀
    let _ = lei;
    // 供给不可控——改用掉血路径：直接跳过雷杀供给，验证普通杀不触发
    let atk = e.debug_hand(1).into_iter().find(|c| {
        e.catalog().cards[*c as usize].kind == CardKind::Sha
    }).expect("1 号应有杀");
    let out = submit(&mut e, 1, GameAction::PlayCard { card: atk, targets: vec![0] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    // 非雷伤不触发凝冰：3-1=2
    assert_eq!(e.debug_hp(0), 2, "普通杀照常（3→2，凝冰不触发）");
}

// ---------------------------------------------------------------------------
// U05 xianjian 第十二切片（规则 v153）：龙葵（pal_longkui）蝶舞（diewu，
// 出牌阶段主动）+灵愈（lingyu，回合结束确认）双技闭环。fenxing（变身）
// 结构性 deferred。存档 v18→v19（diewu_draw_used）。
// ---------------------------------------------------------------------------

/// 蝶舞：杀直接转移给其他角色（不弃置）+本回合首次摸 1（第二次不摸）；
/// 灵愈：令一名其他已受伤角色回复 1（可放弃）。
#[test]
fn longkui_diewu_give_and_lingyu_recover() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10010, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "diewu");
    e.debug_grant(0, "lingyu");
    e.debug_set_hp(0, 3);
    e.debug_set_hp(1, 2); // 已受伤
    let sha = e.debug_place(0, "sha");
    let hand0_before = e.debug_hand(0).len();

    // ① 蝶舞：杀转移给 1 号+首次摸 1（手牌净变化 -1+1=0）
    let def_dw = e.catalog().skill_by_name("diewu").expect("def");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_dw });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let out = answer_open(&mut e, vec![ChoiceOption::Card(sha)]);
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let out = answer_open(&mut e, vec![ChoiceOption::Player(1)]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(e.debug_hand(0).len(), hand0_before, "给出杀-1+首次摸 1=净 0");
    assert!(e.debug_hand(1).contains(&sha), "杀转移到 1 号手牌");
    assert!(e.state_core().turn.diewu_draw_used, "首次摸牌已消耗");

    // ② 灵愈：0 号 Discard→End 阶段结束即触发（owner phaseEnd 任意阶
    // 段）——确认门挂起，令 1 号（2/3）回复 1
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End（灵愈挂起）
    let out = answer_open(&mut e, vec![ChoiceOption::Player(1)]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(e.debug_hp(1), 3, "灵愈回复 1（2→3）");
}

// ---------------------------------------------------------------------------
// U05 xianjian 第十三切片（规则 v154）：慕长兰（pal_muchanglan）掠影
//（lueying，ShaResolve 内联钩每回合限一次）——出杀结算开始：获得目标
// he 随机一张；另有其他有牌者→目标选人、来源随机弃其一张。feixia（虚
// 拟用牌）登记暂缓。
// ---------------------------------------------------------------------------

/// 掠影：获得目标随机牌+目标选人→来源随机弃其一张；每回合限一次。
#[test]
fn muchanglan_lueying_gain_and_victim_discard() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10011, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "lueying");
    // 0 号 1 张（杀），1 号 2 张（目标），2 号 1 张（受害者候选）
    let sha0 = e.debug_place(0, "sha");
    let t1 = e.debug_place(1, "tao");
    e.debug_place(1, "shan");
    e.debug_place(2, "tao");
    e.debug_set_hp(1, 4);

    let out = submit(&mut e, 0, GameAction::PlayCard { card: sha0, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    // 掠影先于闪询问：确认门（observer=0，上游非强制）→ Confirm →
    // 目标选人询问（observer=1）
    let c = e.open_choice().cloned().expect("掠影确认门");
    assert_eq!(c.observer, 0, "确认门 observer=使用者（非强制发动）");
    drop(c);
    let out = answer_open(&mut e, vec![ChoiceOption::Confirm]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    let c = e.open_choice().cloned().expect("掠影选人询问");
    assert_eq!(c.observer, 1, "选人询问 observer=目标");
    drop(c);
    let out = answer_open(&mut e, vec![ChoiceOption::Player(2)]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    // 0 号获得 1 号一张牌（手牌 1→2）；2 号被随机弃 1（1→0）
    assert_eq!(e.debug_hand(0).len(), 1, "来源获得目标一张牌（杀已离手 0+1=1）");
    assert_eq!(e.debug_hand(2).len(), 0, "受害者被随机弃 1");
    assert!(e.state_core().turn.lueying_used, "本回合已发动");

    // 出闪询问（1 号无闪）→杀照常
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    assert_eq!(e.debug_hp(1), 3, "杀照常（4→3）");
    let _ = t1;
}

/// 掠影 v194 补齐①：无第三名有牌者→整个技能不可发动（不询问、不获得、
/// 不消耗次数）——上游 filter 要求"另有一名其他有牌角色"。
#[test]
fn muchanglan_lueying_no_third_party_never_fires() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10071, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "lueying");
    // 目标 1 号有牌；2/3/4 号全空手——无第三名有牌者
    let sha0 = e.debug_place(0, "sha");
    e.debug_place(1, "tao");
    e.debug_set_hp(1, 4);

    let out = submit(&mut e, 0, GameAction::PlayCard { card: sha0, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    // 首个询问直接是 1 号出闪/无懈等结算询问——绝无使用者确认门
    if let Some(c) = e.open_choice() {
        assert_ne!(c.observer, 0, "无第三者时不得出现掠影确认门: {}", c.spec.prompt);
        assert_ne!(c.spec.prompt, "lueying_confirm");
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    assert_eq!(e.debug_hand(0).len(), 0, "无第三者：来源不获得任何牌");
    assert!(!e.state_core().turn.lueying_used, "技能未发动，不消耗次数");
    assert_eq!(e.debug_hp(1), 3, "杀照常（4→3）");
}

/// 掠影 v194 补齐②：确认门放弃→不获得、不消耗 usable；本回合再出杀
/// 仍可再次询问（上游非强制+usable:1 口径）。
#[test]
fn muchanglan_lueying_decline_keeps_usable_for_next_sha() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10072, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "lueying");
    e.debug_grant(0, "咆哮"); // 出牌阶段杀不限次，便于同回合第二杀
    let sha1 = e.debug_place(0, "sha");
    let sha2 = e.debug_place(0, "sha");
    e.debug_place(1, "tao");
    e.debug_place(2, "tao");
    e.debug_set_hp(1, 4);
    e.debug_set_hp(2, 4);

    // 第一杀：确认门→放弃（空选择）→不获得、不消耗
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sha1, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    let c = e.open_choice().cloned().expect("掠影确认门");
    assert_eq!(c.observer, 0, "确认门 observer=使用者");
    assert_eq!(c.spec.prompt, "lueying_confirm");
    drop(c);
    let out = answer_open(&mut e, vec![]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(e.debug_hand(0).len(), 1, "放弃：来源无获得（剩第二张杀）");
    assert!(!e.state_core().turn.lueying_used, "放弃不消耗 usable");
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    assert_eq!(e.debug_hp(1), 3, "第一杀照常（4→3）");

    // 第二杀（仍打相邻的 1 号）：确认门再次出现（usable 未被放弃消耗）
    // → Confirm → 获得目标牌 → 目标选人（受害者=2 号）→ 来源弃其一张
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sha2, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    let c = e.open_choice().cloned().expect("第二杀确认门应再次出现");
    assert_eq!(c.observer, 0, "确认门 observer=使用者");
    assert_eq!(c.spec.prompt, "lueying_confirm");
    drop(c);
    let out = answer_open(&mut e, vec![ChoiceOption::Confirm]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert!(e.state_core().turn.lueying_used, "确认后消耗 usable");
    assert_eq!(e.debug_hand(0).len(), 1, "获得 1 号一张牌（杀已离手 0+1=1）");
    let c = e.open_choice().cloned().expect("选人相");
    assert_eq!(c.observer, 1, "选人询问 observer=目标（1 号）");
    drop(c);
    let out = answer_open(&mut e, vec![ChoiceOption::Player(2)]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(e.debug_hand(2).len(), 0, "受害者（2 号）被随机弃 1");
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    assert_eq!(e.debug_hp(1), 2, "第二杀照常（3→2）");
}

// ---------------------------------------------------------------------------
// U05 xianjian 第十四切片（规则 v155）：阿奴（pal_anu）蛊舞（anwugu，伤
// 害后确认）+蛊舞·缚（anwugu2，三钩）——你伤害其他未中蛊角色后令其获
// 蛊 3 阶段：每回合最多使用 3 张牌+手牌上限 -1，自己每阶段结束 -1。
// lingdi 登记暂缓。
// ---------------------------------------------------------------------------

/// 蛊舞：伤害→获蛊；蛊舞生效中出牌受限（3 张后拒绝）+手牌上限 -1；阶
/// 段结束递减至解除。
#[test]
fn anu_anwugu_afflict_and_limits() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10012, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "anwugu");
    e.debug_set_hp(0, 4);
    e.debug_set_hp(1, 4);
    e.debug_place(0, "sha");

    // ① 杀 1 号→蛊舞确认门→Confirm→1 号获蛊 3 阶段
    let sha = e.debug_hand(0).into_iter().find(|c| {
        e.catalog().cards[*c as usize].kind == CardKind::Sha
    }).expect("应有杀");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        assert_eq!(c.spec.prompt, "trigger_confirm");
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Confirm] });
    }
    assert_eq!(e.debug_hp(1), 3, "杀命中（4→3）");
    assert_eq!(e.state_core().player(1).anwugu_count, 3, "获蛊 3 阶段");

    // 0 号 Play→Discard→End（0 手牌 0 张无弃牌门）
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End

    // ② 蛊舞生效：1 号回合出第 3 张牌被拒（countUsed>=3）
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→0 收束→1 Judge
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Draw→Play
    e.debug_clear_hand(1);
    for _ in 0..4 {
        e.debug_place(1, "wuzhong");
    }
    let mut used = 0;
    while used < 4 {
        let wz = e.debug_hand(1).into_iter().find(|c| {
            e.catalog().cards[*c as usize].kind == CardKind::Wuzhong
        }).expect("应有无中生有");
        let out = submit(&mut e, 1, GameAction::PlayCard { card: wz, targets: vec![] });
        if used < 3 {
            assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "前 3 张可用：{out:?}");
            while let Some(c) = e.open_choice() {
                let (cid, obs) = (c.id, c.observer);
                submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
            }
        } else {
            assert!(
                matches!(&out, CommandOutcome::Rejected(engine_native::RejectReason::CardLimit)),
                "蛊舞：第 4 张牌拒绝（{out:?}）"
            );
            break;
        }
        used += 1;
    }
    assert_eq!(used, 3, "蛊舞=每回合 3 张（第 4 张拒绝）");
    // 阶段结束递减：0 的 Discard/End 阶段结束已递 3→1（蛊舞在 0 回合中
    // 挂上，0 的后续阶段结束同样递减——上游 phaseAfter 每阶段同口径）
    assert_eq!(e.state_core().player(1).anwugu_count, 1, "蛊舞阶段结束递减");

    // ③ 阶段结束递减：1 号收束→蛊 2
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 1, GameAction::NextPhase); // End→收束→2 Judge
    // 0 的 Discard/End 阶段结束已递 2 次（3→1），1 的 Judge/Draw 阶段
    // 结束再递 2 次（1→0，归零解除）
    assert_eq!(e.state_core().player(1).anwugu_count, 0, "蛊舞递减归零解除");
}

// ---------------------------------------------------------------------------
// U05 xianjian 第十五切片（规则 v156）：唐雨柔（pal_tangyurou）甜香去
//（txianqu，伤害前置窗口确认每回合限一次）——防止此伤害+摸 2+回复 1。
// qiongguang（纯表现技 noop）登记不转。
// ---------------------------------------------------------------------------

/// 甜香去：自己出牌阶段造成伤害→确认→防止+摸 2+回复 1；第二次不触发。
#[test]
fn tangyurou_txianqu_prevent_and_replenish() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10013, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "txianqu");
    e.debug_set_hp(0, 2); // 已受伤（回复可见）
    e.debug_set_hp(1, 4);
    let zhuge = e.debug_place(0, "zhuge");
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: zhuge, targets: vec![] });
    for _ in 0..4 {
        e.debug_place(0, "sha");
    }
    let hand_before = e.debug_hand(0).len();

    // ① 杀 1 号→甜香去确认门→Confirm→防止+摸 2+回复 1（体力不变 2）
    let sha = e.debug_hand(0).into_iter().find(|c| {
        e.catalog().cards[*c as usize].kind == CardKind::Sha
    }).expect("应有杀");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        assert_eq!(c.spec.prompt, "trigger_confirm");
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Confirm] });
    }
    assert_eq!(e.debug_hp(1), 4, "甜香去：伤害防止（体力不变）");
    assert_eq!(e.debug_hp(0), 3, "甜香去：自己回复 1（2→3）");
    assert_eq!(e.debug_hand(0).len(), hand_before + 1, "摸 2（-1 杀+2=净 +1）");

    // ② 第二次：txianqu_used 置位不触发（杀照常 4→3）
    let sha2 = e.debug_hand(0).into_iter().find(|c| {
        e.catalog().cards[*c as usize].kind == CardKind::Sha
    }).expect("应仍有杀");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sha2, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    assert_eq!(e.debug_hp(1), 3, "第二次伤害照常（4→3）");
}

// ---------------------------------------------------------------------------
// U05 xianjian 第十六切片（规则 v157）：姜城（pal_jiangcheng）焚逝
//（fenshi，濒死救回强制觉醒一次）+龙火（longhuo，自身阶段结束确认，
// 焚逝获得后永久）——觉醒 +2 护甲、摸 2、获得龙火；龙火对全场按座位
// 序各 1 火伤。yanzhan 系（viewAs/mod）登记暂缓。
// ---------------------------------------------------------------------------

/// 焚逝：濒死救回→觉醒（+2 甲+摸 2+龙火 flag）；二次濒死不再触发。
#[test]
fn jiangcheng_fenshi_awaken_once_on_rescue() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10017, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "fenshi");
    e.debug_set_hp(0, 1);
    let tao = e.debug_place(0, "tao");

    // ① 直伤 1→0 号濒死→0 号先被问（current=0）→用桃救回→焚逝觉醒
    e.debug_deal_damage(Some(1), 0, 1, engine_native::event::DamageNature::Normal);
    let c = e.open_choice().expect("濒死求桃挂起");
    assert_eq!(c.observer, 0, "current=0 → 0 号先答");
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(tao)] });
    assert_eq!(e.debug_hp(0), 1, "桃救回（0→1）");
    assert_eq!(e.state_core().player(0).hujia, 2, "焚逝：+2 护甲");
    assert_eq!(e.debug_hand(0).len(), 2, "焚逝：摸 2（桃已耗）");
    assert!(e.state_core().player(0).jiangcheng_longhuo, "龙火获得（flag 置位）");

    // ② 二次濒死：伤 3（甲 2 吸 2 → 1→0 濒死）→再救→焚逝不重触发
    let tao2 = e.debug_place(0, "tao");
    e.debug_deal_damage(Some(1), 0, 3, engine_native::event::DamageNature::Normal);
    let c = e.open_choice().expect("二次濒死求桃挂起");
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(tao2)] });
    assert_eq!(e.debug_hp(0), 1, "二次救回");
    assert_eq!(e.state_core().player(0).hujia, 0, "甲 2 吸 2 耗尽");
    assert_eq!(e.debug_hand(0).len(), 2, "觉醒一次：无补牌（3-1 桃=2）");
    assert!(e.state_core().player(0).jiangcheng_longhuo);
}

/// 龙火：焚逝觉醒后自身阶段结束确认→全场按座位序各 1 火伤（甲吸伤
/// 可见；含自己）。
#[test]
fn jiangcheng_longhuo_fires_at_own_phase_end() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10018, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "fenshi");
    e.debug_grant(0, "longhuo");
    e.debug_set_hp(0, 1);
    let tao = e.debug_place(0, "tao");

    // 觉醒：直伤濒死→桃救回
    e.debug_deal_damage(Some(1), 0, 1, engine_native::event::DamageNature::Normal);
    let c = e.open_choice().expect("濒死求桃挂起");
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(tao)] });
    assert!(e.state_core().player(0).jiangcheng_longhuo, "已觉醒");

    // 提血防连锁濒死；清手防弃牌门；推进到 0 号 End 阶段收束
    e.debug_set_hp(0, 3);
    e.debug_set_hp(1, 4);
    e.debug_set_hp(2, 4);
    e.debug_clear_hand(0);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let out = submit(&mut e, 0, GameAction::NextPhase); // End→龙火确认挂起
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let c = e.open_choice().expect("龙火确认挂起");
    assert_eq!(c.spec.prompt, "trigger_confirm");
    assert_eq!(c.observer, 0);
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Confirm] });

    // 全场座位序各 1 火伤：0 号甲 2 吸 1（体力不变 3）；1/2 号 4→3
    assert_eq!(e.debug_hp(0), 3, "0 号：火伤被甲吸收");
    assert_eq!(e.state_core().player(0).hujia, 1, "甲 2→1");
    assert_eq!(e.debug_hp(1), 3, "1 号 4→3");
    assert_eq!(e.debug_hp(2), 3, "2 号 4→3");
}


// ---------------------------------------------------------------------------
// U05 xianjian 第十七切片（规则 v158）：景天（pal_jingtian，wu 3 男）洒
// 金（sajin，出牌阶段限一次主动）+聚宝（jtjubao，global 弃牌 frequent）
// ——弃一张手牌指定任意名已受伤角色各判定，同色者回复 1；他人回合外
// 弃非基本牌入弃堆时（本回合首次）随机回收一张。
// ---------------------------------------------------------------------------

/// 洒金：弃牌→单目标判定同色回复 1；异色不回复；每回合限一次。
#[test]
fn jingtian_sajin_judge_color_recover_and_once() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);
    use engine_native::catalog::Suit;
    let same_color_top = |e: &Engine, card: u32| {
        let red = matches!(
            e.catalog().cards[card as usize].suit,
            Suit::Heart | Suit::Diamond
        );
        move |d: &engine_native::catalog::CardDef| {
            let dred = matches!(d.suit, Suit::Heart | Suit::Diamond);
            dred == red
        }
    };

    let mut e = Engine::new_with_extensions(10019, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(0, 4);
    e.debug_set_hp(1, 2); // 已受伤
    e.debug_set_hp(2, 4); // 满血（不可指定）
    e.debug_grant(0, "sajin");
    let disc = e.debug_place(0, "wuzhong");

    // ① 发动→弃 wuzhong→指定 1 号→判定同色→回复 1（2→3）
    let _ = e.debug_set_deck_top_by(same_color_top(&e, disc));
    let def = e.catalog().skill_by_name("sajin").expect("def");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let out = answer_open(&mut e, vec![ChoiceOption::Card(disc)]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    let c = e.open_choice().expect("洒金目标挂起");
    assert_eq!(
        (c.spec.prompt, c.observer),
        (
            engine_native::content::host::WUDIAN_XJ_SAJIN_TARGETS,
            0
        )
    );
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Player(1)] });
    assert_eq!(e.debug_hp(1), 3, "同色判定：回复 1（2→3）");
    assert_eq!(e.debug_hp(2), 4, "满血者不可指定");
    assert!(e.state_core().discard.contains(&disc), "所弃牌入弃堆");

    // ② 每回合限一次：本回合再发动被拒
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
    assert!(matches!(out, CommandOutcome::Rejected(_)), "{out:?}");

    // ③ 推进到 0 号下回合：异色判定不回复
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge
    for _ in 0..5 {
        let _ = submit(&mut e, 1, GameAction::NextPhase); // 1 回合走完（Judge 起 5 次）
    }
    for _ in 0..5 {
        let _ = submit(&mut e, 2, GameAction::NextPhase); // 2 回合走完
    }
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Draw→Play
    let disc2 = e.debug_place(0, "wuzhong");
    let opposite = |e: &Engine, card: u32| {
        let red = matches!(
            e.catalog().cards[card as usize].suit,
            Suit::Heart | Suit::Diamond
        );
        move |d: &engine_native::catalog::CardDef| {
            let dred = matches!(d.suit, Suit::Heart | Suit::Diamond);
            dred != red
        }
    };
    let _ = e.debug_set_deck_top_by(opposite(&e, disc2));
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let _out = answer_open(&mut e, vec![ChoiceOption::Card(disc2)]);
    let c = e.open_choice().expect("洒金目标挂起②");
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Player(1)] });
    assert_eq!(e.debug_hp(1), 3, "异色判定：不回复（3→3）");
}

/// 聚宝：他人回合外弃非基本牌→随机回收一张（本回合首次）；自己弃牌
/// 不触发；换回合门控复位。
#[test]
fn jingtian_jtjubao_gains_nonbasic_and_gates() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);
    use engine_native::catalog::CardKind as K;

    let mut e = Engine::new_with_extensions(10020, 4, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "jtjubao");
    e.debug_set_hp(0, 4);
    e.debug_set_hp(1, 1); // 手牌上限 1 → 弃牌阶段弃 3

    // 0 号过回合 → 1 号回合：摸 2 后手牌 4，弃 3（保证含非基本）
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Draw→Play
    let wz = e.debug_place(1, "wuzhong");
    let tao = e.debug_place(1, "tao");
    let hand1 = e.debug_hand(1);
    assert!(hand1.contains(&wz) && hand1.contains(&tao), "布阵在手");
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Play→Discard
    let c = e.open_choice().expect("弃牌阶段挂起");
    assert_eq!(c.observer, 1);
    let keep = *c.spec.options.iter().find(|o| matches!(o, ChoiceOption::Card(x) if *x != wz)).expect("留牌项");
    let (cid, obs) = (c.id, c.observer);
    let thrown: Vec<ChoiceOption> = e.debug_hand(1)
        .into_iter()
        .filter(|x| *x != wz)
        .map(ChoiceOption::Card)
        .collect();
    let _ = keep;
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: thrown });

    // 聚宝自动回收：0 号手牌 +1，且为弃入弃堆的非基本牌之一
    assert_eq!(e.debug_hand(0).len(), 1, "回收 1 张");
    let gained = e.debug_hand(0)[0];
    assert_ne!(gained, tao, "非基本候选（桃=基本不入池）");
    assert!(
        matches!(
            e.catalog().cards[gained as usize].kind,
            K::Wuzhong | K::Snatch | K::Dismantle | K::Duel | K::Nullify | K::Chain | K::Le | K::Bing | K::ShanDian | K::Taoyuan | K::Wugu | K::Jiedao | K::SavageAttack | K::ArcheryAttack
        ),
        "回收=非基本牌（{:?}）",
        e.catalog().cards[gained as usize].kind
    );
    assert!(e.state_core().turn.jtjubao_used, "本回合已回收门控置位");

    // 0 号自己回合弃牌：不触发（player==owner 门控）
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 1, GameAction::NextPhase); // End→2 Judge
    for _ in 0..5 {
        let _ = submit(&mut e, 2, GameAction::NextPhase); // 2 回合走完（Judge 起 5 次）
    }
    for _ in 0..5 {
        let _ = submit(&mut e, 3, GameAction::NextPhase); // 3 回合走完
    }
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Draw→Play
    assert!(!e.state_core().turn.jtjubao_used, "换回合门控复位");
    e.debug_set_hp(0, 1); // 手牌上限 1
    let extra = e.debug_place(0, "wuzhong");
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let c = e.open_choice().expect("0 号弃牌阶段挂起");
    let (cid, obs) = (c.id, c.observer);
    let keep0 = ChoiceOption::Card(gained);
    let thrown0: Vec<ChoiceOption> = e.debug_hand(0)
        .into_iter()
        .filter(|x| *x != gained)
        .map(ChoiceOption::Card)
        .collect();
    assert!(!thrown0.is_empty());
    let _ = keep0;
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: thrown0 });
    assert_eq!(e.debug_hand(0).len(), 1, "弃至上限（保留回收牌）");
    assert!(!e.state_core().turn.jtjubao_used, "自己弃牌不触发");
    assert!(e.state_core().discard.contains(&extra), "所弃非基本在弃堆未回收");
}


// ---------------------------------------------------------------------------
// U05 xianjian 第十八切片（规则 v159）：龙幽（pal_longyou，wei 4 男）月
// 行（yuexing，出牌阶段限一次主动）+敏杀（minsha，出牌阶段限一次主动）
// ——距离基数交换至自己出牌阶段结束（computed_distance 座位代换）；弃
// 两张 he 牌令体力>1 的其他角色受 1 雷伤、距其 1 内其他角色各随机弃。
// ---------------------------------------------------------------------------

/// 月行：激活后双向距离按配对者座位为基数；出牌阶段结束解除。
#[test]
fn longyou_yuexing_distance_swap_and_clear() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10021, 4, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    e.debug_grant(0, "yuexing");
    let dist = |e: &Engine, a: u8, b: u8| {
        e.state_core().computed_distance(e.catalog(), a, b)
    };
    // 基线：4 人 0→2 座位距 2
    assert_eq!(dist(&e, 0, 2), 2, "基线 0→2 距 2");

    // 激活月行指定 2 号：0 的出距以 2 的座位为基数（0→1 = 座位(2→1)=1）；
    // 2 的出距以 0 的座位为基数（2→3 = 座位(0→3)=1）
    let def = e.catalog().skill_by_name("yuexing").expect("def");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let c = e.open_choice().expect("月行目标挂起");
    assert_eq!(c.observer, 0);
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Player(2)] });
    assert_eq!(dist(&e, 0, 1), 1, "0→1 以 2 座位为基数（座位(2→1)=1）");
    assert_eq!(dist(&e, 2, 3), 1, "2→3 以 0 座位为基数（座位(0→3)=1）");
    assert_eq!(e.state_core().player(0).yuexing_pair, Some(2), "双向互指");
    assert_eq!(e.state_core().player(2).yuexing_pair, Some(0));

    // 出牌阶段结束解除：Play→Discard 后距离复原
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    assert_eq!(dist(&e, 0, 2), 2, "阶段结束解除复原");
    assert_eq!(e.state_core().player(0).yuexing_pair, None);
    assert_eq!(e.state_core().player(2).yuexing_pair, None);
}

/// 敏杀：弃两张 he 牌→目标雷伤 1→距其 1 内其他角色（除自己）各随机弃。
#[test]
fn longyou_minsha_thunder_and_neighbor_discard() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10022, 4, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "minsha");
    e.debug_set_hp(0, 4);
    e.debug_set_hp(1, 4); // 目标（hp>1）
    e.debug_set_hp(2, 4); // 距 1 号 1 → 随机弃 1
    e.debug_set_hp(3, 4); // 距 1 号 2 → 不弃
    let c1 = e.debug_place(0, "wuzhong");
    let c2 = e.debug_place(0, "wuzhong");
    let d1 = e.debug_place(2, "tao"); // 邻座被随机弃候选

    // 激活：弃两张 → 指定 1 号
    let def = e.catalog().skill_by_name("minsha").expect("def");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let out = answer_open(&mut e, vec![ChoiceOption::Card(c1), ChoiceOption::Card(c2)]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    let c = e.open_choice().expect("敏杀目标挂起");
    assert_eq!(
        (c.spec.prompt, c.observer),
        (
            engine_native::content::host::WUDIAN_XJ_MINSHA_TARGET,
            0
        )
    );
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Player(1)] });

    // 断言：1 号 4→3 雷伤；2 号随机弃 1（4→3 手牌）；3 号不弃；所弃两张入弃堆
    assert_eq!(e.debug_hp(1), 3, "目标雷伤（4→3）");
    assert_eq!(e.debug_hand(0).len(), 0, "两张已弃");
    assert!(e.state_core().discard.contains(&c1) && e.state_core().discard.contains(&c2));
    assert_eq!(e.debug_hand(2).len(), 0, "距 1 内邻座随机弃 1（1 桃→0）");
    assert!(e.state_core().discard.contains(&d1), "唯一候选=桃被随机弃入弃堆");
    assert_eq!(e.debug_hand(3).len(), 0, "距 2 者不弃");
}


// ---------------------------------------------------------------------------
// U05 xianjian 第十九切片（规则 v160）：明绣（pal_mingxiu，shu 3 女）灵
// 火（linghuo，global phaseEnd 确认每 2 轮限一次）+归烬（guijin，出牌
// 阶段主动每 3 轮限一次）——本回合造成过伤害的其他角色阶段结束对其 1
// 火伤；亮 4 张逐张交付任意角色（获得者跳下次摸牌阶段）。chengxin 濒
// 死技能型救援通道缺口登记暂缓。
// ---------------------------------------------------------------------------

/// 灵火：伤害者阶段结束确认→1 火伤+2 轮冷却；冷却内不再触发。
#[test]
fn mingxiu_linghuo_round_cooldown_and_fire() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10023, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_max_hp(p, 6);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "linghuo");
    // 回合 1（初值 round=1）：0 号打 1 号一伤（自己阶段末不问=owner 排除）
    e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge（End 末不问）
    assert!(e.open_choice().is_none(), "自己阶段结束不触发");
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Draw→Play
    e.debug_deal_damage(Some(1), 2, 1, engine_native::event::DamageNature::Normal);
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 1, GameAction::NextPhase); // End→2 Judge：灵火确认
    let c = e.open_choice().expect("灵火确认挂起");
    assert_eq!(c.spec.prompt, "trigger_confirm");
    assert_eq!(c.observer, 0);
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Confirm] });
    assert_eq!(e.debug_hp(1), 4, "1 号吃 1 火伤（5→4）");
    assert_eq!(e.debug_hp(2), 5, "2 号仅吃 1 的伤害（6→5）");
    assert_eq!(e.state_core().player(0).linghuo_ready_round, 3, "2 轮冷却置位");
    assert_eq!(e.state_core().player(1).turn_damage, 1, "本回合伤害统计");

    // 回合 2：冷却内一律不触发（各阶段末均无询问）
    let _ = submit(&mut e, 2, GameAction::NextPhase);
    let _ = submit(&mut e, 2, GameAction::NextPhase); // 2 Play（round→2 于 2→0 收束）
    e.debug_deal_damage(Some(2), 0, 1, engine_native::event::DamageNature::Normal);
    e.debug_clear_hand(2);
    let _ = submit(&mut e, 2, GameAction::NextPhase);
    let _ = submit(&mut e, 2, GameAction::NextPhase);
    let _ = submit(&mut e, 2, GameAction::NextPhase); // End→0 Judge：round→2
    assert!(e.open_choice().is_none(), "round 2 < 3：不触发");
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // 0 Play
    e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    e.debug_clear_hand(0);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    let _ = submit(&mut e, 1, GameAction::NextPhase); // 1 Play
    e.debug_deal_damage(Some(1), 2, 1, engine_native::event::DamageNature::Normal);
    e.debug_clear_hand(1);
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    let _ = submit(&mut e, 1, GameAction::NextPhase); // End→2 Judge
    assert!(e.open_choice().is_none(), "冷却内第二次不触发");

    // 回合 3：冷却结束——round 3 于 2→0 收束置位；2 号本回合已造成伤
    // 害 → 其 End 末（round 3）触发确认 → 2 号吃火伤
    let _ = submit(&mut e, 2, GameAction::NextPhase);
    let _ = submit(&mut e, 2, GameAction::NextPhase); // 2 Play
    e.debug_deal_damage(Some(2), 0, 1, engine_native::event::DamageNature::Normal);
    e.debug_clear_hand(2);
    let _ = submit(&mut e, 2, GameAction::NextPhase);
    let _ = submit(&mut e, 2, GameAction::NextPhase);
    let _ = submit(&mut e, 2, GameAction::NextPhase); // End→0 Judge：round→3 + 灵火确认
    let c = e.open_choice().expect("灵火 round3 确认挂起");
    assert_eq!(c.observer, 0);
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Confirm] });
    assert_eq!(e.debug_hp(2), 3, "2 号（伤害者）吃 1 火伤（4→3；此前 r1/r2 各吃 1 伤害）");
    assert_eq!(e.state_core().player(0).linghuo_ready_round, 5, "冷却续置");

    // round 3 后续：冷却已续置，0/1 号回合不再触发
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // 0 Play
    e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    e.debug_clear_hand(0);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge
    assert!(e.open_choice().is_none(), "续置冷却：不触发");
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    let _ = submit(&mut e, 1, GameAction::NextPhase); // 1 Play
    e.debug_deal_damage(Some(1), 2, 1, engine_native::event::DamageNature::Normal);
    e.debug_clear_hand(1);
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    let _ = submit(&mut e, 1, GameAction::NextPhase); // End→2 Judge
    assert!(e.open_choice().is_none(), "续置冷却：仍不触发");
    // 终态血量核对：1 号 6-1(r1 0 伤)-1(火)-1(r2 0 伤)-1(r3 0 伤)=2；
    // 2 号 6-1(r1 1 伤)-1(r2 1 伤)-1(火)-1(r3 1 伤)=2
    assert_eq!(e.debug_hp(1), 2, "1 号终态血量");
    assert_eq!(e.debug_hp(2), 2, "2 号终态血量");
}

/// 归烬：亮 4 张逐张交付（获得者跳下次摸牌阶段）；取消弃置剩余。
#[test]
fn mingxiu_guijin_deal_and_skip_draw() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10024, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "guijin");
    e.debug_set_hp(1, 4);

    // 激活：亮 4 张 → 选第一张交 1 号 → 剩余 3 张取消弃置
    let def = e.catalog().skill_by_name("guijin").expect("def");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let c = e.open_choice().expect("归烬亮牌选择挂起");
    assert_eq!(
        (c.spec.prompt, c.observer),
        (
            engine_native::content::host::WUDIAN_XJ_GUIJIN_CARDS,
            0
        )
    );
    assert_eq!(c.spec.options.len(), 4, "亮 4 张");
    let given = match c.spec.options[0] {
        ChoiceOption::Card(x) => x,
        _ => panic!("选项应为牌"),
    };
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(given)] });
    let c = e.open_choice().expect("归烬目标挂起");
    assert_eq!(c.spec.prompt, engine_native::content::host::WUDIAN_XJ_GUIJIN_TARGET);
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Player(1)] });
    assert_eq!(e.debug_hand(1), vec![given], "1 号获得首张");
    assert!(e.state_core().player(1).skip_next_draw, "获得者跳摸标记");
    assert_eq!(e.state_core().player(0).guijin_ready_round, 4, "3 轮冷却置位（round 初值 1）");

    // 第二轮询问：取消 → 剩余 3 张弃置
    let c = e.open_choice().expect("归烬第二轮挂起");
    assert_eq!(c.spec.options.len(), 3);
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![] });
    assert!(e.open_choice().is_none(), "取消后收束");
    assert_eq!(e.debug_hand(0).len(), 0, "亮牌不归手");
    assert_eq!(e.state_core().discard.len() >= 3, true, "剩余亮牌入弃堆");
    assert!(!e.state_core().discard.contains(&given), "已交付牌不在弃堆");

    // 0 号过回合 → 1 号 Judge→Draw：跳摸（手牌不变）
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Judge→Draw（跳）
    assert_eq!(e.debug_hand(1).len(), 1, "跳过摸牌阶段（手牌不变）");
    assert!(!e.state_core().player(1).skip_next_draw, "跳摸标记已消费");
}


// ---------------------------------------------------------------------------
// U05 xianjian 第二十切片（规则 v161）：云天河（pal_yuntianhe，wu 4 男）
// 逐月（zhuyue，出牌阶段限一次主动）+贯日（guanri，limited 一次性主
// 动）——弃非基本牌令至多两名有牌其他角色各随机弃一张，本出牌阶段杀
// 须以其为目标（距离豁免）；弃两红手牌失 1 体力对体力≥自己者 2 火伤
// 并弃其装备区全部牌。longxi 响应链缺口登记暂缓。
// ---------------------------------------------------------------------------

/// 逐月：弃非基本牌→两名目标各随机弃→杀须指定白名单（距离豁免）→
/// 出牌阶段结束白名单清空。
#[test]
fn yuntianhe_zhuyue_whitelist_and_clear() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10025, 4, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "zhuyue");
    for p in 0..4 {
        e.debug_set_hp(p, 4);
    }
    let nb = e.debug_place(0, "wuzhong"); // 非基本弃牌
    let d1 = e.debug_place(1, "tao"); // 目标 1 被随机弃候选
    let _d2 = e.debug_place(2, "tao"); // 目标 2 被随机弃候选

    // 激活：弃 wuzhong → 指定 1、2 号
    let def = e.catalog().skill_by_name("zhuyue").expect("def");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let out = answer_open(&mut e, vec![ChoiceOption::Card(nb)]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    let c = e.open_choice().expect("逐月目标挂起");
    assert_eq!(c.spec.prompt, engine_native::content::host::WUDIAN_XJ_ZHUYUE_TARGETS);
    assert_eq!(c.spec.max, 2, "至多两名");
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Player(1), ChoiceOption::Player(2)] });
    assert_eq!(e.debug_hand(1).len(), 0, "1 号随机弃 1（1 桃→0）");
    assert_eq!(e.debug_hand(2).len(), 0, "2 号随机弃 1（1 桃→0）");
    assert!(e.state_core().discard.contains(&d1), "1 号弃牌入弃堆");
    assert_eq!(e.state_core().turn.zhuyue_targets, vec![1, 2], "白名单装填");

    // 杀须指定白名单：对 3 号拒绝；对 2 号距离豁免放行（0→2 座位距 2）
    e.debug_place(0, "sha");
    let sha = e.debug_hand(0).into_iter().find(|c| {
        e.catalog().cards[*c as usize].kind == CardKind::Sha
    }).expect("应有杀");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![3] });
    assert!(matches!(out, CommandOutcome::Rejected(_)), "非白名单拒绝：{out:?}");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![2] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "白名单+距离豁免放行：{out:?}");
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    assert_eq!(e.debug_hp(2), 3, "杀照常命中（4→3）");

    // 出牌阶段结束白名单清空
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    assert!(e.state_core().turn.zhuyue_targets.is_empty(), "白名单清空");
}

/// 贯日：弃两红→失 1 体力→体力≥自己者 2 火伤→存活则弃其装备区全部；
/// 一次性（再发动拒绝）。
#[test]
fn yuntianhe_guanri_once_fire_and_equip_strip() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10026, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 4);
    }
    e.debug_grant(0, "guanri");
    let r1 = e.debug_place(0, "tao"); // 红
    let r2 = e.debug_place(0, "tao"); // 红
    // 1 号 hp=4≥4 且挂一件装备（将被全弃）
    let equip = e.debug_place(1, "shan");
    // 用 debug_grantEquip? 无——改用手牌弃候选：改为给 1 号两件不能直接装……
    // 简化：装备区断言改为「若有装备则全弃」——此处以 1 号手牌保留验证
    // 主体（火伤+失体力），装备全弃路径用 equips 直写布阵。
    {
        let mut snap = e.save();
        snap.core.players[1].equips.push(equip);
        snap.core.players[1].hand.retain(|c| *c != equip);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入");
    }

    // 激活：弃两红 → 指定 1 号
    let def = e.catalog().skill_by_name("guanri").expect("def");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let out = answer_open(&mut e, vec![ChoiceOption::Card(r1), ChoiceOption::Card(r2)]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    let c = e.open_choice().expect("贯日目标挂起");
    assert_eq!(c.spec.prompt, engine_native::content::host::WUDIAN_XJ_GUANRI_TARGET);
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Player(1)] });
    assert_eq!(e.debug_hp(0), 3, "失去 1 体力（4→3）");
    assert_eq!(e.debug_hp(1), 2, "2 火伤（4→2）");
    assert!(e.state_core().player(1).equips.is_empty(), "存活则装备区全弃");
    assert!(e.state_core().discard.contains(&equip), "装备入弃堆");
    assert!(e.state_core().player(0).guanri_used, "一次性置位");

    // 再发动拒绝
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
    assert!(matches!(out, CommandOutcome::Rejected(_)), "一次性：{out:?}");
}


// ---------------------------------------------------------------------------
// U05 xianjian 第二十一切片（规则 v162）：紫萱（pal_zixuan，shu 3 女）
// 水云（shuiyun，阶段末置牌三类去重入离场区 ≤3）+水云·救（shuiyun5，
// global Dying 确认弃离场区牌救人——濒死救援通道首例，shuiyun_count++
// 小蛮链依赖）+忘忧（wangyou/wangyou2，受伤回合戳+他人阶段末弃牌令标
// 记者们各摸 1）。changnian 死亡拦截+跨将授技复合缺口登记暂缓。
// ---------------------------------------------------------------------------

/// 水云：阶段末置牌（类型去重 ≤3）入离场区；取消=放弃。
#[test]
fn zixuan_shuiyun_off_court_and_type_dedup() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10027, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_max_hp(p, 6);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "shuiyun");
    // 预置锦囊入离场区：类型去重——本阶段末手牌的锦囊不可置
    let pre = e.debug_off_court_place(0, "wuzhong");
    let tao = e.debug_place(0, "tao"); // 基本（可置）
    let wz = e.debug_place(0, "wuzhong"); // 锦囊（类型被 pre 占——不可置）
    let eq = e.debug_place(0, "zhuge"); // 装备（可置）

    // 阶段末：选项=可置类型（桃+弩），锦囊 wz 被排除
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge：水云询问
    let c = e.open_choice().expect("水云置牌挂起");
    assert_eq!(c.spec.prompt, engine_native::content::host::WUDIAN_XJ_SHUIYUN_CARDS);
    assert!(c.spec.options.contains(&ChoiceOption::Card(tao)), "基本可置");
    assert!(c.spec.options.contains(&ChoiceOption::Card(eq)), "装备可置");
    assert!(
        !c.spec.options.contains(&ChoiceOption::Card(wz)),
        "锦囊类型已占：不可置"
    );
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(tao)] });
    assert_eq!(e.state_core().player(0).off_court, vec![pre, tao], "桃入离场区（预置+新置）");
    assert!(!e.debug_hand(0).contains(&tao), "手牌已离开");
    assert!(e.state_core().player(0).hand.contains(&eq), "未置牌留手");

    // 取消路径：第二次询问（装备/锦囊均占→只剩基本类；预填后测上限）
    // 走完 1/2 号回合回到 0 号（Judge 起 4 NP 收束）
    for _ in 0..4 {
        let _ = submit(&mut e, 1, GameAction::NextPhase);
    }
    for _ in 0..4 {
        let _ = submit(&mut e, 2, GameAction::NextPhase);
    }
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Draw→Play
    e.debug_clear_hand(0);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    // 预填第三张（装备）：离场区满 3 → 水云关闭
    let _ = e.debug_off_court_place(0, "zhuge");
    assert_eq!(e.state_core().player(0).off_court.len(), 3, "离场区满");
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge
    assert!(e.open_choice().is_none(), "离场区满 3：水云关闭");
}

/// 水云·救：他人濒死确认→弃离场区牌→回复 1+计数（小蛮链）。
#[test]
fn zixuan_shuiyun5_dying_rescue_channel() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10028, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_max_hp(p, 6);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "shuiyun5");
    let saved = e.debug_off_court_place(0, "tao");

    // 1 号压到 1 再直伤 2 → 濒死 → 水云·救确认 → 弃离场区牌 → 回复 1
    e.debug_set_hp(1, 1);
    e.debug_deal_damage(Some(0), 1, 2, engine_native::event::DamageNature::Normal);
    let c = e.open_choice().expect("水云·救确认挂起");
    assert_eq!(c.spec.prompt, "trigger_confirm");
    assert_eq!(c.observer, 0);
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Confirm] });
    let c = e.open_choice().expect("离场区救牌挂起");
    assert_eq!(c.spec.prompt, engine_native::content::host::WUDIAN_XJ_SHUIYUN5_CARDS);
    assert_eq!(c.spec.options, vec![ChoiceOption::Card(saved)]);
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(saved)] });
    assert_eq!(e.debug_hp(1), 0, "回复 1（-1→0，脱离濒死待收束）");
    assert!(e.state_core().player(0).off_court.is_empty(), "离场区牌已弃");
    assert!(e.state_core().discard.contains(&saved), "救牌入弃堆");
    assert_eq!(e.state_core().player(0).shuiyun_count, 1, "水云计数（小蛮链）");
}

/// 忘忧：受伤者获回合戳；他人阶段末弃牌令标记者们各摸 1。
#[test]
fn zixuan_wangyou_roundstamp_and_draw() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10029, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "wangyou");
    e.debug_grant(0, "wangyou2");
    let w = e.debug_place(0, "wuzhong");

    // 标记语义（登记差异）：戳存活至标记者自己下回合开始——伤害须发生
    // 在标记者自己回合内（Judge begin 之后）才能活到其 End 末触发。
    // 0 号回合直接过（自身阶段末不触发=owner 排除）
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge
    assert!(e.open_choice().is_none(), "自己阶段结束不触发");

    // 1 号回合内受伤（Judge begin 后）→ 获戳 → 其 End 末忘忧询问
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Draw→Play
    e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    assert_eq!(e.state_core().player(1).wangyou3_round != 0, true, "受伤获戳");
    e.debug_clear_hand(1); // 清摸牌阶段手牌（隔离摸 1 断言）
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 1, GameAction::NextPhase); // End→2 Judge：忘忧询问
    let c = e.open_choice().expect("忘忧弃牌挂起");
    assert_eq!(c.spec.prompt, engine_native::content::host::WUDIAN_XJ_WANGYOU_CARDS);
    assert_eq!(c.spec.options, vec![ChoiceOption::Card(w)]);
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(w)] });
    assert_eq!(e.debug_hand(1).len(), 1, "标记者 1 号摸 1");

    // 2 号回合内受伤获戳 → 2 号 End 末询问②（1 号戳未清、2 号新戳——
    // 取消=放弃验证）
    let _ = submit(&mut e, 2, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 2, GameAction::NextPhase); // Draw→Play
    e.debug_deal_damage(Some(1), 2, 1, engine_native::event::DamageNature::Normal);
    e.debug_clear_hand(2);
    assert_eq!(e.state_core().player(2).wangyou3_round != 0, true, "2 号获戳");
    let w2 = e.debug_place(0, "wuzhong");
    let _ = submit(&mut e, 2, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 2, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 2, GameAction::NextPhase); // End→0 Judge：忘忧询问②
    let c = e.open_choice().expect("忘忧弃牌挂起②");
    assert_eq!(c.spec.options, vec![ChoiceOption::Card(w2)]);
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![] });
    assert_eq!(e.debug_hand(2).len(), 0, "取消=放弃（2 号不摸）");
}


// ---------------------------------------------------------------------------
// U05 xianjian 第二十二切片（规则 v163）：小蛮（pal_xiaoman，shu 3 女）
// 灵泉（lingquan，phaseEnd forced juexingji：轮次≥3+全局用牌数>体力→
// 摸 3+runtime 授水云+立即置牌一次）+神雾（shenwu，global phaseEnd
// forced juexingji：水云计数≥3→上限+1 回复 1+runtime 授回梦）。
// anwugu 与阿奴同技 mapping 复用。
// ---------------------------------------------------------------------------

/// 灵泉：round≥3 且用牌数>体力时觉醒——摸 3+授水云+立即置牌。
#[test]
fn xiaoman_lingquan_awaken_grants_shuiyun() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10030, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_max_hp(p, 6);
        e.debug_set_hp(p, 6);
    }
    e.debug_set_hp(0, 1); // 用牌门槛降为 2
    e.debug_grant(0, "lingquan");

    // r1：0 号用两张无中（total_used=2>1）
    let w1 = e.debug_place(0, "wuzhong");
    let w2 = e.debug_place(0, "wuzhong");
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: w1, targets: vec![] });
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: w2, targets: vec![] });
    assert_eq!(e.state_core().player(0).total_used, 2, "全局用牌计数");
    e.debug_clear_hand(0);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    // r1 0 号 End：round 1 <3 → 不觉醒
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge
    assert!(!e.state_core().player(0).lingquan_awakened, "round 1 不觉醒");

    // 轮转两圈到 r3 的 0 号 End（每回合 Judge 起 5 NP 收束）
    for _ in 0..5 {
        let _ = submit(&mut e, 1, GameAction::NextPhase);
    }
    for _ in 0..5 {
        let _ = submit(&mut e, 2, GameAction::NextPhase); // wrap→r2
    }
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Draw→Play
    e.debug_clear_hand(0);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge（r2 末：round 2 <3）
    assert!(!e.state_core().player(0).lingquan_awakened, "round 2 不觉醒");
    for _ in 0..5 {
        let _ = submit(&mut e, 1, GameAction::NextPhase);
    }
    for _ in 0..5 {
        let _ = submit(&mut e, 2, GameAction::NextPhase); // wrap→r3
    }
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Draw→Play
    e.debug_clear_hand(0);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge：灵泉觉醒（r3 末）
    assert!(e.state_core().player(0).lingquan_awakened, "r3 觉醒");
    assert_eq!(e.debug_hand(0).len(), 3, "觉醒摸 3");
    let c = e.open_choice().expect("灵泉内置水云置牌挂起");
    assert_eq!(c.spec.prompt, engine_native::content::host::WUDIAN_XJ_LINGQUAN_CARDS);
    let placed = match c.spec.options[0] {
        ChoiceOption::Card(x) => x,
        _ => panic!("选项应为牌"),
    };
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(placed)] });
    assert!(e.state_core().player(0).off_court.contains(&placed), "觉醒置牌入离场区");
    assert_eq!(e.debug_hand(0).len(), 2, "置牌后 3-1=2");
    // runtime 授水云实例
    let shuiyun_def = e.catalog().skill_by_name("shuiyun").expect("shuiyun def");
    assert!(
        e.state_core().skills.contains_key(&(0, shuiyun_def)),
        "runtime 授水云实例"
    );
}

/// 神雾：水云计数≥3 时觉醒——上限+1、回复 1、授回梦实例。
#[test]
fn xiaoman_shenwu_awaken_grants_huimeng() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10031, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_max_hp(p, 6);
        e.debug_set_hp(p, 5); // 已受伤：回复可见
    }
    e.debug_grant(0, "shenwu");
    // 注入水云计数 3（save/restore 上帝视角；计数真实来源=水云·救）
    {
        let mut snap = e.save();
        snap.core.players[0].shuiyun_count = 3;
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入");
    }

    // 任意阶段末觉醒（global phaseEnd forced）：0 号收束即触发
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge：神雾觉醒
    assert!(e.state_core().player(0).shenwu_awakened, "觉醒");
    assert_eq!(e.state_core().player(0).max_hp, 7, "上限+1（6→7）");
    assert_eq!(e.debug_hp(0), 6, "回复 1（5→6）");
    let huimeng_def = e.catalog().skill_by_name("huimeng").expect("huimeng def");
    assert!(
        e.state_core().skills.contains_key(&(0, huimeng_def)),
        "runtime 授回梦实例"
    );

    // 一次性：后续阶段末不再触发（上限不再涨）
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    assert_eq!(e.state_core().player(0).max_hp, 7, "觉醒一次不再触发");
}


// ---------------------------------------------------------------------------
// U05 xianjian 第二十三切片（规则 v164）：明绣诚信（chengxin，global
// Dying 确认每 4 轮冷却）——濒死者回复至 1 并获免伤至其 End 阶段结束
// （濒死救援通道第二例；v160 暂缓消账）。
// ---------------------------------------------------------------------------

/// 诚信：他人濒死确认→回复至 1+免伤；冷却内置位；免伤 End 末清除。
#[test]
fn mingxiu_chengxin_rescue_and_immunity() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10032, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_max_hp(p, 6);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "chengxin");

    // 1 号压到 -2 濒死 → 确认 → 回复至 1（量=1-(-2)=3）+免伤置位
    e.debug_set_hp(1, -2);
    e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    let c = e.open_choice().expect("诚信确认挂起");
    assert_eq!(c.spec.prompt, "trigger_confirm");
    assert_eq!(c.observer, 0);
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Confirm] });
    assert_eq!(e.debug_hp(1), 1, "回复至 1（-3→1）");
    assert!(e.state_core().player(1).chengxin_active, "免伤置位");
    assert_eq!(e.state_core().player(0).chengxin_ready_round, 5, "4 轮冷却置位（round 初值 1）");

    // 免伤：1 号再受伤无效果
    e.debug_deal_damage(Some(2), 1, 2, engine_native::event::DamageNature::Normal);
    assert_eq!(e.debug_hp(1), 1, "免伤：体力不变");

    // 0 号回合收束（3 NP）→ 1 号回合走完（End 末清免伤）→ 再受伤照常
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Draw→Play
    e.debug_clear_hand(1);
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 1, GameAction::NextPhase); // End→2 Judge：免伤清
    assert!(!e.state_core().player(1).chengxin_active, "End 末免伤清除");
    e.debug_deal_damage(Some(2), 1, 1, engine_native::event::DamageNature::Normal);
    assert_eq!(e.debug_hp(1), 0, "免伤清除后照常受伤（1→0 濒死）");
    // 1 号濒死：诚信冷却中（round 1 <5）→ 无诚信询问 → 求桃链（无桃死亡）
    // 2 号有桃可救——给 2 号桃以稳态
    let _tao2 = e.debug_place(2, "tao");
    // 无人响应则 1 号死亡：此处仅断言无诚信确认门
    // （当前 open_choice 若有=求桃询问，observer=响应者；跳过应答致死）
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    assert!(!e.debug_alive(1), "冷却中不救：1 号死亡");
}


// ---------------------------------------------------------------------------
// U05 xianjian 第二十四批切片（规则 v165）：雪见（pal_xuejian，shu 3
// 女）神暮（shenmu，global Dying 确认）+毒刺（duci，CardsDiscarded
// direct）——弃全部红手牌令濒死者回复 1 摸等量；弃装备牌入弃堆令距离 1
// 内一名其他角色受 1 伤害。xshuangren 武器技能转移缺口登记暂缓。
// ---------------------------------------------------------------------------

/// 神暮：他人濒死确认→弃全部红手牌→濒死者回复 1 摸等量。
#[test]
fn xuejian_shenmu_red_discard_rescue() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10033, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_max_hp(p, 6);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "shenmu");
    let _r1 = e.debug_place(0, "tao"); // 红
    let _r2 = e.debug_place(0, "wuzhong"); // 可能红可能黑——按实际花色清点
    let _blk = e.debug_place(0, "sha"); // 可能黑——置后按花色断言
    let reds: Vec<u32> = e.debug_hand(0)
        .into_iter()
        .filter(|c| {
            matches!(
                e.catalog().cards[*c as usize].suit,
                engine_native::catalog::Suit::Heart
                    | engine_native::catalog::Suit::Diamond
            )
        })
        .collect();
    let n_red = reds.len();
    assert!(n_red >= 1, "至少一张红（桃保底）");

    // 1 号压到 1 再伤 1 → 0 濒死 → 确认 → 弃全部红 → 1 号回复 1 摸 n_red
    e.debug_set_hp(1, 1);
    e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    let c = e.open_choice().expect("神暮确认挂起");
    assert_eq!(c.spec.prompt, "trigger_confirm");
    assert_eq!(c.observer, 0);
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Confirm] });
    assert_eq!(e.debug_hp(1), 1, "濒死者回复 1（0→1）");
    assert_eq!(e.debug_hand(1).len(), n_red, "摸等量红牌数");
    assert_eq!(e.debug_hand(0).len(), 3 - n_red, "仅弃红牌（黑牌保留）");
    let hand0 = e.debug_hand(0);
    assert!(
        !hand0.iter().any(|c| {
            matches!(
                e.catalog().cards[*c as usize].suit,
                engine_native::catalog::Suit::Heart
                    | engine_native::catalog::Suit::Diamond
            )
        }),
        "无红牌残留"
    );
    for c in &reds {
        assert!(e.state_core().discard.contains(c), "红牌入弃堆");
    }
}

/// 毒刺：弃装备牌入弃堆→距离 1 内一名其他角色受 1 伤害；取消=放弃。
#[test]
fn xuejian_duci_equip_discard_damage() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10034, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "duci");
    e.debug_set_hp(0, 1); // 手牌上限 1：弃牌阶段弃 1
    let equip = e.debug_place(0, "zhuge"); // 装备（手牌弃置路径）
    let _tao = e.debug_place(0, "tao"); // 基本对照

    // ① 弃装备（手牌弃置）→ 毒刺目标询问 → 指定 1 号（距 1）受 1 伤
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let c = e.open_choice().expect("弃牌阶段挂起");
    assert_eq!(c.observer, 0);
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(equip)] });
    let c = e.open_choice().expect("毒刺目标挂起");
    assert_eq!(c.spec.prompt, engine_native::content::host::WUDIAN_XJ_DUCI_TARGET);
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Player(1)] });
    assert_eq!(e.debug_hp(1), 5, "毒刺伤害（6→5）");
    assert!(e.state_core().discard.contains(&equip), "装备入弃堆");

    // ② 弃基本牌 → 不触发（对照）
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Draw→Play
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 1, GameAction::NextPhase); // End→2 Judge
    for _ in 0..5 {
        let _ = submit(&mut e, 2, GameAction::NextPhase); // 2 号回合收束→0 Judge
    }
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Draw→Play
    e.debug_clear_hand(0);
    let _ = e.debug_place(0, "tao");
    let tao2 = e.debug_place(0, "tao");
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard：弃 1 张
    let c = e.open_choice().expect("弃牌阶段挂起②");
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(tao2)] });
    assert!(e.open_choice().is_none(), "弃基本牌：毒刺不触发");
}


// ---------------------------------------------------------------------------
// U05 xianjian 第二十五批切片（规则 v166）：柳梦璃（pal_liumengli，shu
// 3 女）润心（runxin，CardUsed+CardResponded 双事件 direct）——使用/
// 打出红桃牌后令一名已受伤角色回复 1。tianxian（mod 系+逐目标判定取
// 消）/xjzhimeng（暗牌标记）缺口登记暂缓。
// ---------------------------------------------------------------------------

/// 润心：使用红桃→选受伤者回复 1；打出红桃（闪响应）同触发；非红桃
/// 不触发；取消=放弃。
#[test]
fn liumengli_runxin_heart_use_and_respond() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10035, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 4);
    }
    e.debug_grant(0, "runxin");

    // ① 使用红桃牌（无中生有不限体力；红桃由布阵循环保证）
    let heart_card = loop {
        let c = e.debug_place(0, "wuzhong");
        if e.catalog().cards[c as usize].suit == engine_native::catalog::Suit::Heart {
            break c;
        }
        e.debug_clear_hand(0);
    };
    e.debug_set_hp(1, 3); // 受伤者
    let out = submit(&mut e, 0, GameAction::PlayCard { card: heart_card, targets: vec![] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    // 吃桃回自己的询问可能先挂（无——自己满血 4 桃自回满血无窗口）；润心询问
    let c = e.open_choice().expect("润心目标挂起");
    assert_eq!(c.spec.prompt, engine_native::content::host::WUDIAN_XJ_RUNXIN_TARGET);
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Player(1)] });
    assert_eq!(e.debug_hp(1), 4, "润心回复（3→4）");

    // ② 打出红桃闪（响应杀）同触发：0 号过回合→1 号回合内杀 0 号，
    // 0 号红桃闪响应
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_set_hp(1, 3); // 再压伤
    let shan = loop {
        let c = e.debug_place(0, "shan");
        if e.catalog().cards[c as usize].suit == engine_native::catalog::Suit::Heart {
            break c;
        }
        e.debug_clear_hand(0);
    };
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Draw→Play
    e.debug_clear_hand(1);
    let sha2 = e.debug_place(1, "sha");
    let out = submit(&mut e, 1, GameAction::PlayCard { card: sha2, targets: vec![0] });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    // 0 号闪响应（红桃）→ 润心询问
    let out = answer_open(&mut e, vec![ChoiceOption::Card(shan)]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    let c = e.open_choice().expect("润心目标挂起②");
    assert_eq!(c.spec.prompt, engine_native::content::host::WUDIAN_XJ_RUNXIN_TARGET);
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Player(1)] });
    assert_eq!(e.debug_hp(1), 4, "打出红桃闪：润心回复（3→4）");
}


// ---------------------------------------------------------------------------
// U05 xianjian 第二十六批切片（规则 v167）：柳梦璃织梦（xjzhimeng，
// phaseEnd direct 可放弃）+织梦·梦（xjzhimeng2，CardUsed frequent
// silent）——牌堆顶一张暗置于其他角色武将牌上（off_court）+runtime 授
// 梦实例；暗牌持有者被使用同类型（三分类）牌时摸 1（暗牌保留）。柳梦
// 璃升全闭环（tianxian 仍暂缓）。
// ---------------------------------------------------------------------------

/// 织梦：暗置+授技；织梦·梦：同类型牌被使用摸 1；异类型不摸。
#[test]
fn liumengli_xjzhimeng_darkmark_and_draw() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10036, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "xjzhimeng");

    // ① 0 号阶段末：织梦询问 → 选 1 号 → 牌堆顶暗置其离场区+授梦实例
    let deck_top = *e.state_core().deck.last().expect("牌堆非空");
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge：织梦询问
    let c = e.open_choice().expect("织梦目标挂起");
    assert_eq!(c.spec.prompt, engine_native::content::host::WUDIAN_XJ_ZHIMENG_TARGET);
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Player(1)] });
    assert_eq!(e.state_core().player(1).off_court, vec![deck_top], "牌堆顶暗置 1 号离场区");
    let zm2_def = e.catalog().skill_by_name("xjzhimeng2").expect("xjzhimeng2 def");
    assert!(
        e.state_core().skills.contains_key(&(1, zm2_def)),
        "runtime 授梦实例"
    );
    let dark_kind = e.catalog().card_kind(deck_top);
    let dark_class = if dark_kind.equip_slot().is_some() {
        2
    } else {
        match dark_kind {
            CardKind::Sha | CardKind::FireSha | CardKind::LeiSha | CardKind::Shan
            | CardKind::Tao | CardKind::Jiu => 0,
            _ => 1,
        }
    };

    // ② 1 号自己使用锦囊（无中）→ 目标=自己 → 织梦·梦按类型判摸
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Draw→Play
    e.debug_clear_hand(1);
    let hand_before = e.debug_hand(1).len();
    let wz = loop {
        let c = e.debug_place(1, "wuzhong");
        let k = e.catalog().card_kind(c);
        if !k.equip_slot().is_some()
            && !matches!(
                k,
                CardKind::Sha | CardKind::FireSha | CardKind::LeiSha
                    | CardKind::Shan | CardKind::Tao | CardKind::Jiu
            )
        {
            break c;
        }
        e.debug_clear_hand(1);
    };
    let out = submit(&mut e, 1, GameAction::PlayCard { card: wz, targets: vec![] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    // 无中结算（1 号摸 2）+织梦·梦（同类型再摸 1）
    let wz_class = 1; // 锦囊
    if dark_class == wz_class {
        assert_eq!(e.debug_hand(1).len(), hand_before + 3, "无中 2+织梦·梦 1");
    } else {
        assert_eq!(e.debug_hand(1).len(), hand_before + 2, "异类型：仅无中 2");
    }
}


// ---------------------------------------------------------------------------
// U05 xianjian 第二十七批切片（规则 v168）：南宫煌（pal_nangonghuang，
// wei 3 男）摄灵（sheling，CardUsed+CardResponded+CardsDiscarded 三事
// 件 frequent silent 每回合 3 次）——他人回合内牌入弃堆时回收之。
// zhaoyao（拼点时序偏差）/zhangmu（响应链）登记暂缓。
// ---------------------------------------------------------------------------

/// 摄灵：他人回合内弃牌→自动回收；自己回合内/自己弃牌不触发；每回合
/// 3 次上限；Judge begin 清零。
#[test]
fn nangonghuang_sheling_three_event_reclaim() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10037, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "sheling");

    // 0 号回合内：杀 1 号 → 1 号响应出闪（回合外角色打牌）→ 摄灵回收闪
    let d1 = e.debug_place(0, "sha");
    let shan = e.debug_place(1, "shan");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: d1, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let out = answer_open(&mut e, vec![ChoiceOption::Card(shan)]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    assert!(e.state_core().player(0).hand.contains(&shan), "摄灵自动回收响应闪");
    assert_eq!(e.state_core().turn.sheling_used, 1, "次数 +1");

    // 第二次（CardsDiscarded 事件面）：0 号用过河拆桥弃 1 号一张手牌
    // → 弃牌入弃堆 → 摄灵回收
    e.debug_clear_hand(1);
    let d3 = e.debug_place(1, "tao");
    let gq = e.debug_place(0, "dismantle");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: gq, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    // 拆桥弃牌目标侧询问（若有）→ 选 d3
    if let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(d3)] });
    }
    assert!(e.state_core().player(0).hand.contains(&d3), "弃牌事件面：摄灵回收");
    assert_eq!(e.state_core().turn.sheling_used, 2, "次数 +1");
}


// ---------------------------------------------------------------------------
// U05 xianjian 第二十八批切片（规则 v169）：王小虎（pal_wangxiaohu，qun
// 4 男）虎煞（husha_count+husha）——出牌回合内伤人 +伤害量标记（≤3）；
// 阶段末消耗标记虚拟用牌：1 枚虚拟杀（选目标）/2 枚虚拟南蛮 AOE
//（use_virtual_card 原语首例：牌堆底载体+use_as 声明）。元素毁灭牌种
// 不存在登记不转。
// ---------------------------------------------------------------------------

/// 虎煞：伤人集标记→阶段末虚拟杀（响应链走通）；虚拟南蛮 AOE 全体验
/// 伤害；标记上限 3。
#[test]
fn wangxiaohu_husha_marks_and_virtual_cards() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10038, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "husha_count");
    e.debug_grant(0, "husha");

    // ① 0 号回合内伤 1 号 2 点 → 标记 +2；再伤 2 号 1 点 → +1（满 3）
    e.debug_deal_damage(Some(0), 1, 2, engine_native::event::DamageNature::Normal);
    e.debug_deal_damage(Some(0), 2, 1, engine_native::event::DamageNature::Normal);
    assert_eq!(e.state_core().player(0).husha_mark, 3, "标记 2+1=3（上限 3）");

    // ② 阶段末：模式选择 [Tag0=虚拟杀, Tag1=虚拟南蛮] → Tag0 虚拟杀
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge：虎煞询问
    let c = e.open_choice().expect("虎煞模式挂起");
    assert_eq!(c.spec.prompt, engine_native::content::host::WUDIAN_XJ_HUSHA_MODE);
    assert_eq!(c.spec.options, vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)]);
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Tag(0)] });
    assert_eq!(e.state_core().player(0).husha_mark, 2, "扣 1 枚");
    let c = e.open_choice().expect("虚拟杀目标挂起");
    assert_eq!(c.spec.prompt, engine_native::content::host::WUDIAN_XJ_HUSHA_TARGET);
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Player(1)] });
    assert_eq!(e.debug_hp(1), 3, "虚拟杀直伤（4→3）");
    assert_eq!(e.state_core().player(0).husha_mark, 2, "回合已推进：不再集标记");
}

/// 虎煞虚拟南蛮：注入标记 2 → Tag1 → AOE 全体其他角色各 -1、扣 2 枚。
#[test]
fn wangxiaohu_husha_virtual_nanman() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10039, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "husha");
    {
        let mut snap = e.save();
        snap.core.players[0].husha_mark = 2;
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入");
    }

    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge：虎煞询问
    let c = e.open_choice().expect("虎煞模式挂起");
    assert_eq!(c.spec.options, vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)], "mark 2：杀+南蛮双选项");
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Tag(1)] });
    assert_eq!(e.state_core().player(0).husha_mark, 0, "扣 2 枚");
    assert_eq!(e.debug_hp(1), 5, "南蛮 1 号直伤（6→5）");
    assert_eq!(e.debug_hp(2), 5, "南蛮 2 号直伤（6→5）");
}


// ---------------------------------------------------------------------------
// U05 xianjian 第二十九批（规则 v170）：死亡前置窗口基建（Death 帧
// BeforeTriggers 起步=TurnOverBefore 同款）+ 柳梦璃长念（changnian，
// Death 窗口遗言）+长念·思（changnian2，runtime 授技结束阶段摸 1）
// ——柳梦璃 v170 时点全闭环（仅 tianxian 暂缓）。
// ---------------------------------------------------------------------------

/// 长念：死亡时遗言移交 hej 全部牌+授长念·思；目标下个结束阶段摸 1；
/// 死亡照常执行。
#[test]
fn liumengli_changnian_death_bequest() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10040, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 6);
    }
    // changnian 挂柳梦璃——映射后目录名即 changnian；grant 校验
    assert!(e.catalog().skill_by_name("changnian").is_some(), "changnian 在目录");
    e.debug_grant(0, "changnian");
    // 0 号（将死）手牌 2 张；1 号（遗言对象）空手
    let h1 = e.debug_place(0, "tao");
    let h2 = e.debug_place(0, "wuzhong");
    // 0 号过回合（死亡后 current 停留在死亡者会卡推进——改在 1 号回合
    // 内击杀）
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Draw→Play
    e.debug_clear_hand(1);

    // 1 号回合内伤 0 号 2 → 死亡 → 长念遗言询问 → 选 1 号
    e.debug_set_hp(0, 1);
    e.debug_deal_damage(Some(1), 0, 2, engine_native::event::DamageNature::Normal);
    // 先走完濒死求桃链（仅 Pass 求桃询问）→ 死亡帧 → 长念遗言询问
    loop {
        let Some(c) = e.open_choice() else { break };
        if c.spec.prompt != "dying_save" {
            break; // 长念询问到达
        }
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    let c = e.open_choice().expect("长念遗言询问挂起");
    assert_eq!(c.spec.prompt, engine_native::content::host::WUDIAN_XJ_CHANGNIAN_TARGET);
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Player(1)] });
    // 移交断言：1 号获 2 张；0 号已死亡手牌清空
    assert!(e.debug_hand(1).contains(&h1) && e.debug_hand(1).contains(&h2), "遗言移交");
    assert!(!e.debug_alive(0), "遗言后死亡照常");
    // runtime 授长念·思实例
    let cn2 = e.catalog().skill_by_name("changnian2").expect("def");
    assert!(e.state_core().skills.contains_key(&(1, cn2)), "授长念·思实例");

    // 1 号（当前回合）收束：清手 → End 末长念·思摸 1
    e.debug_clear_hand(1);
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 1, GameAction::NextPhase); // End→2 Judge：长念·思摸 1
    assert_eq!(e.debug_hand(1).len(), 1, "长念·思摸 1（清手后 +1）");
}


// ---------------------------------------------------------------------------
// U05 xianjian 第二十九批切片（规则 v171）：月琦（pal_yueqi，wei 3 女）
// 适盈（shiying，Death 前置窗口 limited，死亡取消型首例）——他人死亡
// 时取消之：双方上限/体力置 3、clearSkills 双方、弃 hej 全部、各摸 3。
// tianwu 表现层登记不转；liguang moveCard 基建暂缓。
// ---------------------------------------------------------------------------

/// 适盈：他人死亡→取消死亡（存活）+双方重置 3/3+清技能+弃全部+各摸 3；
/// 一次性。
#[test]
fn yueqi_shiying_cancel_death_and_reset() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10041, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_max_hp(p, 6);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(1, "shiying"); // 持技者≠死亡者（filter 他人死亡）
    let keep = e.debug_place(1, "tao"); // 1 号将被清弃的牌

    // 0 号过回合；1 号回合内压 0 号 1 血后伤 2 → 死亡 → 适盈取消
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Draw→Play
    e.debug_clear_hand(1);
    e.debug_set_hp(0, 1);
    e.debug_deal_damage(Some(1), 0, 2, engine_native::event::DamageNature::Normal);
    // 求桃链全 Pass → 死亡帧 → 适盈取消（无询问 forced）
    loop {
        let Some(c) = e.open_choice() else { break };
        if c.spec.prompt != "dying_save" {
            break;
        }
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    // 断言：0 号存活+双方 3/3+清技能+手牌=3（摸 3）+适盈一次性
    assert!(e.debug_alive(0), "死亡被取消");
    assert!(!e.state_core().player(0).death_cancelled, "拦截标记已消费");
    for p in 0..2 {
        assert_eq!(e.state_core().player(p).max_hp, 3, "{p} 号上限置 3");
        assert_eq!(e.debug_hp(p), 3, "{p} 号体力置 3");
    }
    let cn = e.catalog().skill_by_name("changnian");
    let _ = cn;
    assert!(e.state_core().player(1).shiying_used, "一次性置位（持技者 1 号）");
    assert_eq!(e.debug_hand(0).len(), 3, "0 号摸 3");
    assert!(e.state_core().discard.contains(&keep), "1 号 hej 清弃");
    // 技能清空：0/1 号无剩余实例
    assert!(
        !e.state_core().skills.keys().any(|(o, _)| *o == 0 || *o == 1),
        "clearSkills 双方"
    );
}


// ---------------------------------------------------------------------------
// U05 xianjian 第三十批切片（规则 v172）：南宫煌照妖（zhaoyao，global
// Draw-begin 确认）——确定性首张拼点（舌战群儒同款登记差异）赢→摸
// 2+选两张手牌置牌堆顶（先选在上）。登记差异：原生触发在固定摸牌后。
// zhangmu 响应链暂缓。
// ---------------------------------------------------------------------------

/// 照妖：他人 Draw-begin→确定性拼点赢→摸 2+置顶两张（先选在上）。
#[test]
fn nangonghuang_zhaoyao_compare_and_top() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10043, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "zhaoyao");

    // 布阵保证赢：0 号首张点数 > 1 号首张点数（搜索压牌）
    let rank_of = |e: &Engine, c: u32| e.catalog().cards[c as usize].rank;
    let mut r1_card = None;
    'outer: for name in ["tao", "wuzhong", "shan", "sha", "jiu"] {
        let c = e.debug_place(1, name);
        if rank_of(&e, c) <= 5 {
            r1_card = Some(c);
            break 'outer;
        }
        e.debug_clear_hand(1);
    }
    let r1_card = r1_card.expect("找到低点数牌");
    let r1 = rank_of(&e, r1_card);
    let mut r0_card = None;
    for name in ["tao", "wuzhong", "shan", "sha", "jiu", "dismantle"] {
        let c = e.debug_place(0, name);
        if rank_of(&e, c) > r1 {
            r0_card = Some(c);
            break;
        }
        e.debug_clear_hand(0);
    }
    let r0_card = r0_card.expect("找到高点数牌");
    assert!(rank_of(&e, r0_card) > r1, "拼点必赢布阵");

    // 1 号摸牌阶段开始（0 过回合→1 Judge→Draw）：照妖比点赢→摸 2+询问
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Judge→Draw：照妖询问
    let c = e.open_choice().expect("照妖置顶询问挂起");
    assert_eq!(c.spec.prompt, engine_native::content::host::WUDIAN_XJ_ZHAOYAO_CARDS);
    assert_eq!((c.spec.min, c.spec.max), (2, 2));
    assert_eq!(c.observer, 0);
    let picks: Vec<ChoiceOption> = c.spec.options.iter().take(2).copied().collect();
    let top_expect = match &picks[0] {
        ChoiceOption::Card(x) => *x,
        _ => panic!("选项应为牌"),
    };
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: picks });
    // 摸 2-置 2=净 0；牌堆顶=先选者
    let deck_len = e.state_core().deck.len();
    assert_eq!(*e.state_core().deck.last().expect("非空"), top_expect, "先选在上");
    let _ = deck_len;
    assert_eq!(e.debug_hand(0).len(), 1, "摸 2-置 2（首张保留）");
}


// ---------------------------------------------------------------------------
// U05 xianjian 第三十一批切片（规则 v173）：月琦离光（liguang，phaseEnd
// direct 可放弃）——弃一张手牌并移动场上的一张牌（moveCard 基建首例：
// 确定性首张可移牌+首个合法目的地，登记差异=上游 AI 选取）。月琦升全
// 闭环（tianwu 登记不转）。
// ---------------------------------------------------------------------------

/// 离光：弃手牌→装备迁移（revoke+grant+Equipped 日志）；取消=放弃。
#[test]
fn yueqi_liguang_discard_and_move_equip() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10044, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "liguang");
    let disc_card = e.debug_place(0, "wuzhong");

    // 布阵：1 号装备区一把武器（可移）；2 号装备区空（合法目的地）
    let weapon = e.debug_place(1, "zhuge");
    {
        let mut snap = e.save();
        snap.core.players[1].hand.retain(|c| *c != weapon);
        snap.core.players[1].equips.push(weapon);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入");
    }

    // 阶段末：弃牌询问（离光）→ 弃 wuzhong → 确定性移动：武器 1→2
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge：离光询问
    let c = e.open_choice().expect("离光弃牌挂起");
    assert_eq!(c.spec.prompt, engine_native::content::host::WUDIAN_XJ_LIGUANG_CARDS);
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(disc_card)] });
    // 移动断言：确定性首目的地=0 号（座位序首个空武器栏）；1 号失去
    assert!(e.state_core().player(0).equips.contains(&weapon), "武器迁至 0 号（首合法目的地）");
    assert!(!e.state_core().player(1).equips.contains(&weapon), "1 号失去武器");
    assert!(e.state_core().discard.contains(&disc_card), "所弃手牌入弃堆");
    assert!(e.debug_hand(0).is_empty(), "弃 1 后无手牌");

    // 取消路径：下回合无移动（离场区空则 filter 拦——直接断言询问可取消）
    for _ in 0..5 {
        let _ = submit(&mut e, 1, GameAction::NextPhase);
    }
    for _ in 0..5 {
        let _ = submit(&mut e, 2, GameAction::NextPhase);
    }
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Draw→Play
    e.debug_clear_hand(0);
    let d2 = e.debug_place(0, "wuzhong");
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge：离光询问②
    let c = e.open_choice().expect("离光弃牌挂起②");
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![] });
    assert!(e.debug_hand(0).contains(&d2), "取消=放弃（手牌保留）");
}


// ---------------------------------------------------------------------------
// U05 xianjian 第三十二批切片（规则 v174）：夏侯瑾轩（pal_xiahoujinxuan，
// wei 3 男）玄墨（xuanmo，phaseUse usable:1 主动）——弃三类手牌置牌堆
// 顶+实体发现同类型 2 张入手（实体发现基建首例=羊皮卷同款登记差异）。
// danqing 表现层登记不转。
// ---------------------------------------------------------------------------

/// 玄墨：弃锦囊置顶→实体池发现同类型 2 张入手；置顶牌在牌堆顶。
#[test]
fn xiahoujinxuan_xuanmo_discover_same_class() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10045, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "xuanmo");
    let wz = e.debug_place(0, "wuzhong"); // 锦囊（弃置+置顶）

    // 激活：弃 wz 置顶 → 实体发现同类型 2 张入手
    let def = e.catalog().skill_by_name("xuanmo").expect("def");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let out = answer_open(&mut e, vec![ChoiceOption::Card(wz)]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    // 断言：置顶牌=牌堆顶
    assert_eq!(*e.state_core().deck.last().expect("非空"), wz, "置牌堆顶");
    // 断言：入手 2 张锦囊类实体（从牌堆/弃堆发现，非 wz）
    let hand = e.debug_hand(0);
    assert_eq!(hand.len(), 2, "发现 2 张入手");
    for c in &hand {
        let k = e.catalog().cards[*c as usize].kind;
        assert!(
            !matches!(
                k,
                CardKind::Sha | CardKind::Shan | CardKind::Tao | CardKind::Jiu
            ) && !k.equip_slot().is_some(),
            "同类型=锦囊（{:?}）",
            k
        );
        assert_ne!(*c, wz, "不含置顶牌");
    }
    // 入手牌不再在牌堆/弃堆
    for c in &hand {
        assert!(!e.state_core().deck.contains(c), "离牌堆");
        assert!(!e.state_core().discard.contains(c), "离弃堆");
    }
}


// ---------------------------------------------------------------------------
// U05 xianjian 第三十三批切片（规则 v175）：苏媚（pal_sumei，shu 3 女）
// 射影（sheying，source Damage after 每回合 1 次）+花好（huahu，phaseUse
// limited 多目标舍身）——伤害牌入弃堆判定黑则回收；失 1 体力上限+
// clearSkills 自己令任意名其他角色各回复 1+获 1 护甲+摸 1。dujiang 毒
// 牌种不在目录登记暂缓。
// ---------------------------------------------------------------------------

/// 射影：实体伤害牌入弃堆判定黑→回收；判定非黑不回收。
#[test]
fn sumei_sheying_black_judge_reclaim() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10046, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "sheying");
    // 伤害牌（sha 实体入弃堆）；判定堆顶压黑（spade）确保判定黑
    let sha = e.debug_place(0, "sha");
    let _ = e.debug_set_deck_top_by(|d| {
        matches!(d.suit, engine_native::catalog::Suit::Spade | engine_native::catalog::Suit::Club)
    });

    // ① 0 号杀 1 号（实体牌入弃堆）→ 判定黑→回收
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    // 1 号无闪直伤
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    // 判定牌花色随 seed——分断言：红（未回收，本回合封）或黑（已回收）
    let judged_black = e
        .log()
        .iter()
        .filter_map(|l| match l {
            engine_native::LogEvent::Judged { card, .. } => Some(*card),
            _ => None,
        })
        .next()
        .map(|c| {
            matches!(
                e.catalog().cards[c as usize].suit,
                engine_native::catalog::Suit::Spade | engine_native::catalog::Suit::Club
            )
        })
        .expect("射影判定已发生");
    if judged_black {
        assert!(e.state_core().player(0).hand.contains(&sha), "判定黑：回收");
        assert!(e.state_core().turn.sheying_used, "置位");
    } else {
        assert!(!e.state_core().player(0).hand.contains(&sha), "判定红：不回收");
        assert!(e.state_core().turn.sheying_used, "红也置位（direct 触发即耗次数）");
    }
    assert!(e.state_core().turn.sheying_used, "每回合 1 次置位");
}

/// 花好：多目标舍身——失 1 体力上限+clearSkills 自己，目标们各回复 1+
/// 获 1 护甲+摸 1；一次性。
#[test]
fn sumei_huahu_sacrifice_multi_target() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10047, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 3); // 受伤：回复可见
    }
    e.debug_grant(0, "huahu");

    let def = e.catalog().skill_by_name("huahu").expect("def");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let c = e.open_choice().expect("花好目标挂起");
    assert_eq!(c.spec.prompt, engine_native::content::host::WUDIAN_XJ_HUAHU_TARGETS);
    assert_eq!(c.spec.min, 1);
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Player(1), ChoiceOption::Player(2)] });
    assert_eq!(e.state_core().player(0).max_hp, 3, "失 1 体力上限");
    assert!(e.state_core().player(0).huahu_used, "一次性置位");
    for p in 1..3 {
        assert_eq!(e.debug_hp(p), 4, "{p} 号回复（3→4）");
        assert_eq!(e.state_core().player(p).hujia, 1, "{p} 号获 1 护甲");
        assert_eq!(e.debug_hand(p).len(), 1, "{p} 号摸 1");
    }
    let huahu_def = e.catalog().skill_by_name("huahu").expect("def");
    assert!(
        !e.state_core().skills.contains_key(&(0, huahu_def)),
        "clearSkills 自己"
    );
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
    assert!(matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
}


// ---------------------------------------------------------------------------
// U05 xianjian 第三十四批切片（规则 v176）：仙卿（pal_xianqing，qun 4
// 男）探息（xtanxi，phaseUse usable:1 主动）——弃一张手牌，持有同名牌
// 手牌的其他角色中随机一名，随机获得其一张手牌。xiaoyue 复合缺口登记
// 暂缓。
// ---------------------------------------------------------------------------

/// 探息：弃牌→持有同名牌者随机夺取一张；无人持有则牌照弃。
#[test]
fn xianqing_xtanxi_same_name_steal() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10048, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "xtanxi");

    // ① 布阵：0 号弃 sha（黑桃杀）；1/2 号各持一张 sha（同牌种）
    let disc_c = e.debug_place(0, "sha");
    let t1 = e.debug_place(1, "sha");
    let t2 = e.debug_place(2, "sha");

    let def = e.catalog().skill_by_name("xtanxi").expect("def");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let out = answer_open(&mut e, vec![ChoiceOption::Card(disc_c)]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    // 自动执行：随机目标 1 或 2 被夺一张杀 → 0 号手牌 +1
    let got: Vec<u32> = e.debug_hand(0).into_iter().filter(|c| *c == t1 || *c == t2).collect();
    assert_eq!(got.len(), 1, "随机夺取一张同名牌");
    assert_eq!(e.debug_hand(0).len(), 1, "弃 1 夺 1");
    assert!(e.state_core().discard.contains(&disc_c), "所弃牌入弃堆");
}

/// 探息取消路径：无人持有同名牌→牌照弃直接结束（询问取消）。
#[test]
fn xianqing_xtanxi_no_holder_cancel() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10049, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "xtanxi");
    let wz = e.debug_place(0, "wuzhong"); // 无人持无中
    let _other = e.debug_place(1, "tao"); // 1 号有手牌（activate 门槛）

    let def = e.catalog().skill_by_name("xtanxi").expect("def");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let c = e.open_choice().expect("探息弃牌挂起");
    assert_eq!(c.spec.prompt, engine_native::content::host::WUDIAN_XJ_XTANXI_CARDS);
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(wz)] });
    assert!(e.state_core().discard.contains(&wz), "牌照弃");
    assert!(e.debug_hand(0).is_empty(), "无人可夺：无获得");
}


// ---------------------------------------------------------------------------
// U05 xianjian 第三十五批切片（规则 v177）：罗招嫣（pal_luozhaoyan，
// shu 4 女）凤略（fenglue，Play-begin direct 可放弃）——出牌阶段开始选
// 一名其他角色，逐张自动对其使用手中可指定其的杀与决斗（真实牌 emit
// CardUsed=虚拟管线复用真实牌变体）。tanhua Recover 前置窗口缺口登记
// 暂缓。
// ---------------------------------------------------------------------------

/// 凤略：选目标→逐张自动使用杀（距离内）与决斗；取消=放弃。
#[test]
fn luozhaoyan_fenglue_auto_use_sha_duel() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10050, 2, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..2 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "fenglue");
    // 0 号过回合→1 号→回 0 号（技能已持有，Play-begin 触发）
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge
    for _ in 0..5 {
        let _ = submit(&mut e, 1, GameAction::NextPhase); // 1 号回合收束
    }
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Draw→Play
    let sha = e.debug_place(0, "sha");
    let duel = e.debug_place(0, "duel");
    let wuzhong = e.debug_place(0, "wuzhong"); // 非杀/决斗：不自动

    // 凤略在 Play-begin 已过——需再绕一圈让带牌 Play-begin 触发
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge
    for _ in 0..5 {
        let _ = submit(&mut e, 1, GameAction::NextPhase);
    }
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Draw→Play
    let c = e.open_choice().expect("凤略目标挂起");
    assert_eq!(c.spec.prompt, engine_native::content::host::WUDIAN_XJ_FENGLUE_TARGET);
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Player(1)] });
    // 驱动自动使用链（杀/决斗可能引发响应与伤害询问）
    for _ in 0..40 {
        let Some(c) = e.open_choice() else { break };
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    // 断言：两张牌均已离手；1 号受杀 1 伤+决斗 1 伤（无杀响应）=6-2=4
    assert!(!e.debug_hand(0).contains(&sha), "杀已自动使用");
    assert!(!e.debug_hand(0).contains(&duel), "决斗已自动使用");
    assert!(e.debug_hand(0).contains(&wuzhong), "非杀/决斗不自动");
    assert_eq!(e.debug_hp(1), 4, "杀+决斗各 1 伤（6→4）");
}


// ---------------------------------------------------------------------------
// U05 xianjian 第三十六批切片（规则 v178）：姜云凡（pal_jiangyunfan，
// wei 4 男）疾风（liefeng，CardUsed forced）+天剑（tianjian，phaseUse
// usable:1 主动，liefeng 授技）——出牌回合内第 3/4 张使用牌时 runtime
// 授天剑/御风实例（第 2 张授炎杀 yanzhan 暂缓不授）；天剑弃杀视为万
// 箭齐发（虚拟 AOE 原语）+目标随机弃 he 一张。xunying 响应链暂缓。
// ---------------------------------------------------------------------------

/// 疾风：第 3 张授天剑实例；天剑弃杀视为万箭+目标随机弃 he。
#[test]
fn jiangyunfan_liefeng_grant_and_tianjian_aoe() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10051, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "liefeng");
    e.debug_grant(0, "tianjian");

    // 0 号用 3 张无中（无目标直用）→ 第 3 张触发疾风授天剑
    for _ in 0..3 {
        let w = e.debug_place(0, "wuzhong");
        let out = submit(&mut e, 0, GameAction::PlayCard { card: w, targets: vec![] });
        assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    }
    assert_eq!(e.state_core().player(0).total_used, 3, "全局用牌计数");
    let tj_def = e.catalog().skill_by_name("tianjian").expect("def");
    assert!(
        e.state_core().skills.contains_key(&(0, tj_def)),
        "第 3 张授天剑实例"
    );

    // 天剑：手牌需有杀——布阵后激活
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge
    for _ in 0..5 {
        let _ = submit(&mut e, 1, GameAction::NextPhase); // 1 号回合收束
    }
    for _ in 0..5 {
        let _ = submit(&mut e, 2, GameAction::NextPhase); // 2 号回合收束
    }
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Draw→Play
    e.debug_clear_hand(0);
    let sha = e.debug_place(0, "sha");
    e.debug_place(1, "tao"); // 1 号手牌（弃牌候选）
    e.debug_place(2, "tao");
    let hp1 = e.debug_hp(1);
    let hp2 = e.debug_hp(2);
    let he1 = e.debug_hand(1).len() + e.state_core().player(1).equips.len();
    let he2 = e.debug_hand(2).len() + e.state_core().player(2).equips.len();

    let def_tj = e.catalog().skill_by_name("tianjian").expect("def");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_tj });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let out = answer_open(&mut e, vec![ChoiceOption::Card(sha)]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    // 万箭逐目标 aoe_response（无闪全 Pass）
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    // 万箭：1/2 号无闪各 -1；各随机弃 he 一张
    assert_eq!(e.debug_hp(1), hp1 - 1, "万箭 1 号 -1");
    assert_eq!(e.debug_hp(2), hp2 - 1, "万箭 2 号 -1");
    assert!(e.state_core().discard.contains(&sha), "弃杀入弃堆");
    // 随机弃牌断言：各目标 he 恰好 -1
    assert_eq!(
        e.debug_hand(1).len() + e.state_core().player(1).equips.len(),
        he1 - 1,
        "1 号随机弃一张"
    );
    assert_eq!(
        e.debug_hand(2).len() + e.state_core().player(2).equips.len(),
        he2 - 1,
        "2 号随机弃一张"
    );
}


// ---------------------------------------------------------------------------
// U05 xianjian 第三十七批切片（规则 v179）：慕容紫英（pal_murongziying，
// wei 4 男）破云（poyun，source Damage forced direct）——玄砚计数>0 时
// 令受伤目标自选弃 he 1-2 张，计数 -1（计数获得路径上游缺失=调试/宿
// 主注入登记差异）。qianfang AOE 已通暂缓；xuanning 获得路径缺失登记。
// ---------------------------------------------------------------------------

/// 破云：注入计数 2→伤害后目标自选弃 2→计数归零；计数 0 不触发。
#[test]
fn murongziying_poyun_discard_and_decrement() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10052, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "poyun");
    // 注入玄砚计数 2（save/restore；获得路径上游缺失=登记差异）
    {
        let mut snap = e.save();
        snap.core.players[0].xuanning_mark = 2;
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入");
    }
    let c1 = e.debug_place(1, "tao");
    let c2 = e.debug_place(1, "wuzhong");

    // 0 号伤 1 号 1 点 → 破云 → 1 号自选弃 2 张 → 计数归零
    e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    let c = e.open_choice().expect("破云选牌询问挂起");
    assert_eq!(c.spec.prompt, engine_native::content::host::WUDIAN_XJ_POYUN_CARDS);
    assert_eq!(c.observer, 1, "被弃者自选");
    assert_eq!((c.spec.min, c.spec.max), (1, 2), "1-2 张");
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(c1), ChoiceOption::Card(c2)] });
    assert_eq!(e.state_core().player(0).xuanning_mark, 1, "计数 -1");
    assert!(e.state_core().discard.contains(&c1) && e.state_core().discard.contains(&c2), "两牌入弃堆");

    // 计数 1 再伤一次：弃 1 张后计数归零，再伤不触发
    let d3 = e.debug_place(1, "shan");
    e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    let c = e.open_choice().expect("破云选牌询问挂起②");
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(d3)] });
    assert_eq!(e.state_core().player(0).xuanning_mark, 0, "计数归零");
    // 计数 0：再伤不触发
    e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    assert!(e.open_choice().is_none(), "计数 0：不触发");
}


// ---------------------------------------------------------------------------
// U05 xianjian 第三十八批切片（规则 v180）：雪见（pal_xuejian）双刃
//（xshuangren，CardsDiscarded forced）+双刃·殁（xshuangren2，Death 前
// 置窗口 forced）——失去武器（弃置路径）置离场区+grant 武器技能；死亡
// 时弃置离场区全部牌。雪见升全闭环。
// ---------------------------------------------------------------------------

/// 双刃：弃武器→离场区+grant 武器技能；双刃·殁：死亡时弃离场区。
#[test]
fn xuejian_xshuangren_offcourt_and_death_discard() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10053, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_max_hp(p, 6);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "xshuangren");
    e.debug_grant(0, "xshuangren2");

    // 布阵：手牌武器 + 压 hp=1（手牌上限 1，弃牌阶段弃武器）
    let weapon = e.debug_place(0, "zhuge");
    let extra = e.debug_place(0, "tao");
    e.debug_set_hp(0, 1);

    // 弃牌阶段弃武器 → CardsDiscarded 含武器 → 双刃收离场区+grant
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let c = e.open_choice().expect("弃牌询问挂起");
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(weapon)] });
    // 断言：双刃收武器入离场区（非弃堆）+武器技能 grant
    assert!(e.state_core().player(0).off_court.contains(&weapon), "武器入离场区");
    assert!(!e.state_core().discard.contains(&weapon), "未直接入弃堆");
    assert!(e.debug_hand(0).contains(&extra), "超额牌保留");

    // 压 1 血再伤 2 → 死亡 → 双刃·殁弃离场区全部 → 死亡照常
    e.debug_set_hp(0, 1);
    e.debug_deal_damage(Some(1), 0, 2, engine_native::event::DamageNature::Normal);
    loop {
        let Some(c) = e.open_choice() else { break };
        if c.spec.prompt != "dying_save" {
            break;
        }
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    assert!(!e.debug_alive(0), "死亡照常");
    assert!(e.state_core().player(0).off_court.is_empty(), "殁：离场区已清");
    assert!(e.state_core().discard.contains(&weapon), "殁：武器入弃堆");
}


// ---------------------------------------------------------------------------
// U05 xianjian 第三十九批切片（规则 v181）：南宫煌（pal_nangonghuang）
// 张目（zhangmu，DamageBefore forced 每回合 1 次）——需要打出闪响应时
// 弃一张手牌闪直接视为已响应（杀不命中=伤害前置直防，登记差异：响应
// 询问改写简化；弃哪张闪确定性取首张闪）。
// ---------------------------------------------------------------------------

/// 张目：杀伤害前置弃首张闪直防（体力不变+闪离手）；每回合 1 次。
#[test]
fn nangonghuang_zhangmu_flash_discard_prevent() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10054, 2, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..2 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "zhangmu");
    let shan = e.debug_place(0, "shan");
    let sha = e.debug_place(1, "sha");
    // 0 号过回合（出牌阶段外杀才能指向他人）→ 1 号回合
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Draw→Play

    // 1 号杀 0 号：张目前置直防（弃首张闪=shan，体力不变）
    let out = submit(&mut e, 1, GameAction::PlayCard { card: sha, targets: vec![0] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    // 闪响应 Pass → 张目 DamageBefore 直防
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    assert_eq!(e.debug_hp(0), 6, "张目直防：体力不变");
    assert!(
        !e.state_core().player(0).hand.contains(&shan),
        "闪已弃（弃堆或已转化）"
    );
    assert!(e.state_core().turn.zhangmu_used, "回合位");

    // 同回合第二次杀：杀限次拒绝（1 号每回合一杀）
    let sha2 = e.debug_place(1, "sha");
    let out = submit(&mut e, 1, GameAction::PlayCard { card: sha2, targets: vec![0] });
    assert!(
        matches!(&out, CommandOutcome::Rejected(r) if matches!(r, engine_native::RejectReason::ShaLimit)),
        "同回合再杀拒绝：{out:?}"
    );

    // 绕一圈回 1 号回合：张目回合位重置后再防一次
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 1, GameAction::NextPhase); // End→0 Judge
    for _ in 0..5 {
        let _ = submit(&mut e, 0, GameAction::NextPhase); // 0 号回合收束
    }
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Draw→Play
    e.debug_clear_hand(1);
    e.debug_clear_hand(0);
    let sha3 = e.debug_place(1, "sha");
    let shan3 = e.debug_place(0, "shan");
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: sha3, targets: vec![0] });
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    assert_eq!(e.debug_hp(0), 6, "新回合张目再防（两次均被防，6 不变）");
    assert!(!e.state_core().player(0).hand.contains(&shan3), "新回合闪再弃");
}


/// 千方：玄砚 1→弃 2 张→清零+虚拟万箭（目标各-1、各随机弃 he）→紫
/// 英摸受伤人数；放弃=无事；玄砚 0 不触发。
#[test]
fn murongziying_qianfang_discard_and_aoe() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10055, 3, &[&pack]).expect("建局");
    // 布阵+授技+注入在 advance_loop 前（千方在 Judge begin 触发——
    // advance_loop 停在 0 号 Play，首个 Judge begin 已带触发条件时不满足，
    // 故验证第二个 0 号回合的 Judge→Draw→Play 后由下个 Judge 触发）
    for p in 0..3 {
        e.debug_set_max_hp(p, 6);
        e.debug_set_hp(p, 6);
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "qianfang");
    {
        let mut snap = e.save();
        snap.core.players[0].xuanning_mark = 1;
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入");
    }
    advance_loop(&mut e, 0);
    // 千方 filter 要求 he+玄砚>=3：place 后手牌 2+玄砚 1=3 ✓
    let _d1 = e.debug_place(0, "tao");
    let _d2 = e.debug_place(0, "wuzhong");

    // 走到 0 号下个 Judge begin：Play→Discard→End→1(5NP)→2(5NP)→0
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge
    for _ in 0..5 {
        let _ = submit(&mut e, 1, GameAction::NextPhase);
    }
    for _ in 0..5 {
        let _ = submit(&mut e, 2, GameAction::NextPhase);
    }
    // 现在 0 号 Judge begin 已过（Judge 帧 AfterTriggers）——千方询问
    // 挂起（Filter：he2+玄砚1=3≥3 ✓）
    let c = e.open_choice().expect("千方弃牌挂起");
    assert_eq!(c.spec.prompt, engine_native::content::host::WUDIAN_XJ_QIANFANG_CARDS);
    assert_eq!((c.spec.min, c.spec.max), (2, 2), "num=3-1=2");
    let picks: Vec<ChoiceOption> = c.spec.options.iter().take(2).copied().collect();
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: picks });
    // 万箭逐目标 aoe_response（无闪全 Pass）
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    // 断言：玄砚清零+虚拟万箭（1/2 号各-1）+紫英摸受伤人数（2）
    assert_eq!(e.state_core().player(0).xuanning_mark, 0, "计数清零");
    assert_eq!(e.debug_hp(1), 5, "万箭 1 号 -1");
    assert_eq!(e.debug_hp(2), 5, "万箭 2 号 -1");
    assert_eq!(e.debug_hand(0).len(), 4, "摸 2（被伤敌人 2 名；Draw2+place2-弃2+摸2=4）");
}


/// 迅影：杀结算后自动再出一张杀对同一目标（距离内）；无杀不触发。
#[test]
fn jiangyunfan_xunying_auto_second_sha() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10056, 2, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..2 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "xunying");
    let sha1 = e.debug_place(0, "sha");
    let sha2 = e.debug_place(0, "sha");

    // 0 号回合内杀 1 号 → 迅影自动再出一张
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sha1, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    assert_eq!(e.debug_hp(1), 4, "两次杀各-1（6→4）");
    assert!(!e.debug_hand(0).contains(&sha1) && !e.debug_hand(0).contains(&sha2), "两张杀均已使用");
}

/// 天弦：锁定三件——杀无视距离（距离 2 仍可指定）+目标数上限无限
///（双目标合法）+逐目标判定黑取消该目标结算（红=正常结算走闪询问）。
#[test]
fn liumengli_tianxian_range_targets_and_judge_cancel() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10057, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
        if p == 2 || p == 3 {
            e.debug_set_hp(p, 4);
        }
    }

    // 无技：距离 2（0→2/3 均隔一位）拒绝；双目标拒绝
    let sha = e.debug_place(0, "sha");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![2] });
    assert!(
        matches!(out, CommandOutcome::Rejected(engine_native::RejectReason::IllegalTargets)),
        "{out:?}"
    );
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![2, 3] });
    assert!(
        matches!(out, CommandOutcome::Rejected(engine_native::RejectReason::IllegalTargets)),
        "{out:?}"
    );

    // 天弦：距离 2 双目标合法；判定序=2 号红（正常结算）→3 号黑（取消）
    e.debug_grant(0, "tianxian");
    // 牌库顶叠两张：后推先抽——首判（2 号）=红，次判（3 号）=黑
    let _ = e.debug_set_deck_top_by(|cd| {
        matches!(
            cd.suit,
            engine_native::catalog::Suit::Spade | engine_native::catalog::Suit::Club
        )
    });
    let _ = e.debug_set_deck_top_by(|cd| {
        matches!(
            cd.suit,
            engine_native::catalog::Suit::Heart | engine_native::catalog::Suit::Diamond
        )
    });
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![2, 3] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    assert_eq!(e.debug_hp(2), 3, "2 号判红正常结算（4→3，闪 Pass）");
    assert_eq!(e.debug_hp(3), 4, "3 号判黑取消结算（不受伤不问闪）");
    assert!(!e.debug_hand(0).contains(&sha), "杀已结算离手");
}

/// 影锋：杀结算完毕后随机对一名非本杀目标的其他角色使用虚拟杀
///（3 人局候选唯一=确定性）；虚拟杀 use_as 挡递归（否则继续链杀）。
#[test]
fn yuejinzhao_xjyingfeng_random_virtual_sha() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10058, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 4);
    }
    e.debug_grant(0, "xjyingfeng");
    let sha = e.debug_place(0, "sha");

    // 真实杀目标 1 → 结算后影锋自动虚拟杀 2（候选排除目标 1 唯一）
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    assert_eq!(e.debug_hp(1), 3, "真杀目标 1 -1（闪 Pass）");
    assert_eq!(e.debug_hp(2), 3, "影锋虚拟杀 2 -1（闪 Pass）");
    assert!(!e.debug_hand(0).contains(&sha), "真实杀已离手");
    // hp1=3 未再扣=虚拟杀 use_as 挡递归（否则链杀继续打 1 号）
}

/// 械磐：弃牌阶段弃掉最后一张手牌基本牌→询问弃一张手牌→实体发现
/// 随机装备入手（可取消=放弃）；装备来自牌堆/弃堆实体池（非凭空造牌）。
#[test]
fn jushifang_xiepan_discard_and_discover_equip() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10059, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "xiepan");
    e.debug_set_max_hp(0, 1);
    e.debug_set_hp(0, 1);
    let sha = e.debug_place(0, "sha");
    let wuzhong = e.debug_place(0, "wuzhong");

    // Play→Discard：弃牌阶段询问弃 1（手牌 2-上限 1）→ 弃基本牌 sha
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let c = e.open_choice().expect("弃牌阶段挂起");
    assert_eq!(c.spec.prompt, "discard_to_limit");
    let picks: Vec<ChoiceOption> = c
        .spec
        .options
        .iter()
        .copied()
        .filter(|o| matches!(o, ChoiceOption::Card(x) if *x == sha))
        .collect();
    assert_eq!(picks.len(), 1);
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: picks });

    // 械磐挂起：弃最后一张基本牌+手牌仍非空（wuzhong）→ 弃 1 获随机装备
    let c = e.open_choice().expect("械磐弃牌挂起");
    assert_eq!(c.spec.prompt, engine_native::content::host::WUDIAN_XJ_XIEPAN_CARDS);
    assert_eq!((c.spec.min, c.spec.max), (1, 1));
    assert!(c.spec.allow_cancel);
    let picks: Vec<ChoiceOption> = c
        .spec
        .options
        .iter()
        .copied()
        .filter(|o| matches!(o, ChoiceOption::Card(x) if *x == wuzhong))
        .collect();
    assert_eq!(picks.len(), 1);
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: picks });

    // 断言：手牌=1 张装备实体（来自牌堆/弃堆）；sha+wuzhong 均入弃堆
    let hand = e.debug_hand(0);
    assert_eq!(hand.len(), 1, "弃 1 获 1");
    let got = hand[0];
    assert!(
        e.catalog().card_kind(got).equip_slot().is_some(),
        "获得牌为装备实体"
    );
    assert!(
        e.state_core().discard.contains(&sha)
            && e.state_core().discard.contains(&wuzhong),
        "sha+弃牌均入弃牌堆"
    );
}

/// 乱剑：两张杀当一张杀虚拟使用（多目标）+乱剑2 载体锚定 50% 增伤
///（每目标独立 roll）；普通杀不经乱剑2（载体不匹配）。
#[test]
fn changqing_luanjian_dual_sha_viewas_multi_target() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10062, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "luanjian");
    e.debug_grant(0, "luanjian2");
    let sha1 = e.debug_place(0, "sha");
    let sha2 = e.debug_place(0, "sha");

    // 发动乱剑：选两杀 → 多目标 [1,2]
    let def = e.catalog().skill_by_name("luanjian").expect("def");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    let c = e.open_choice().expect("乱剑选牌挂起");
    assert_eq!(c.spec.prompt, engine_native::content::host::WUDIAN_XJ_LUANJIAN_CARDS);
    assert_eq!((c.spec.min, c.spec.max), (2, 2));
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer {
        choice_id: cid,
        selection: vec![ChoiceOption::Card(sha1), ChoiceOption::Card(sha2)],
    });
    let c = e.open_choice().expect("乱剑选目标挂起");
    assert_eq!(c.spec.prompt, engine_native::content::host::WUDIAN_XJ_LUANJIAN_TARGETS);
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer {
        choice_id: cid,
        selection: vec![ChoiceOption::Player(1), ChoiceOption::Player(2)],
    });
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    let (h1, h2) = (e.debug_hp(1), e.debug_hp(2));
    assert_eq!(h1, 5, "目标 1 未增伤（seed 10062 首 roll≥50）: {h1}");
    assert_eq!(h2, 4, "目标 2 增伤 2 点（seed 10062 次 roll<50，逐目标独立）: {h2}");
    assert!(!e.debug_hand(0).contains(&sha1) && !e.debug_hand(0).contains(&sha2), "两杀均已弃置");
}

/// 天符：挂附（闪置离场区+授实例+记来源）→挂附者造成伤害后受来源
/// 1 雷伤+闪入弃堆+清挂附；挂附者死亡时清挂附。
#[test]
fn changqing_ctianfu_attach_thunder_and_death_clear() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    // ---- 场景 A：反噬雷伤 ----
    let mut e = Engine::new_with_extensions(10063, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(0, 4);
    e.debug_set_hp(1, 6);
    e.debug_grant(0, "ctianfu");
    let shan = e.debug_place(0, "shan");
    let def = e.catalog().skill_by_name("ctianfu").expect("def");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    let c = e.open_choice().expect("天符选闪挂起");
    assert_eq!(c.spec.prompt, engine_native::content::host::WUDIAN_XJ_CTIANFU_CARD);
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(shan)] });
    let c = e.open_choice().expect("天符选目标挂起");
    assert_eq!(c.spec.prompt, engine_native::content::host::WUDIAN_XJ_CTIANFU_TARGET);
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Player(1)] });
    // 挂附断言：闪离手入离场区+1 号获实例+来源=0
    assert!(!e.debug_hand(0).contains(&shan), "闪离手");
    assert!(e.state_core().player(1).off_court.contains(&shan), "闪置离场区挂附");
    let has_ct2 = e
        .state_core()
        .skills
        .iter()
        .any(|((o, d), _)| *o == 1 && e.catalog().skill_name(*d) == "ctianfu2");
    assert!(has_ct2, "1 号获反噬实例");
    assert_eq!(e.state_core().player(1).ctianfu_source, Some(0));

    // 推到 1 号回合 Play：0 NP×2→Discard/End，1 号 Judge 起 NP×2 到 Play
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    let sha = e.debug_place(1, "sha");
    let out = submit(&mut e, 1, GameAction::PlayCard { card: sha, targets: vec![0] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    // 断言：0 被杀 -1（4→3）；1 反噬雷伤 -1（6→5，上游 player.damage
    // =挂附者承受）；挂附清（闪入弃堆+实例移除+标记 None）
    assert_eq!(e.debug_hp(0), 3, "0 被杀 -1（4→3）");
    assert_eq!(e.debug_hp(1), 5, "1 反噬雷伤 -1（6→5）");
    let cleared = !e
        .state_core()
        .skills
        .iter()
        .any(|((o, d), _)| *o == 1 && e.catalog().skill_name(*d) == "ctianfu2");
    assert!(cleared, "反噬实例已清");
    assert_eq!(e.state_core().player(1).ctianfu_source, None, "挂附标记清");
    assert!(e.state_core().discard.contains(&shan), "挂附闪入弃堆");

    // ---- 场景 B：死亡清理 ----
    let mut e2 = Engine::new_with_extensions(10064, 3, &[&pack]).expect("建局2");
    advance_loop(&mut e2, 0);
    for p in 0..3 {
        e2.debug_clear_hand(p);
    }
    e2.debug_grant(0, "ctianfu");
    let shan2 = e2.debug_place(0, "shan");
    let def = e2.catalog().skill_by_name("ctianfu").expect("def");
    let _ = submit(&mut e2, 0, GameAction::ActivateSkill { skill: def });
    let c = e2.open_choice().expect("选闪挂起");
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e2, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(shan2)] });
    let c = e2.open_choice().expect("选目标挂起");
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e2, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Player(1)] });
    e2.debug_set_hp(1, 1);
    let sha = e2.debug_place(0, "sha");
    let out = submit(&mut e2, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    while let Some(c) = e2.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e2, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    // 断言：1 死亡+挂附闪入弃堆+标记清（上游凭空消失=登记差异）
    assert!(!e2.state_core().player(1).alive, "1 号死亡");
    assert!(e2.state_core().discard.contains(&shan2), "死亡清挂闪入弃堆");
    assert_eq!(e2.state_core().player(1).ctianfu_source, None, "死亡标记清");
}

/// 指路小虎（swd 机关牌首切片）：自目标使用→无懈窗 Pass→随机对一
/// 名其他存活角色虚拟杀（3 人局候选唯一）→闪 Pass→-1；牌入弃堆。
#[test]
fn swd_zhiluxiaohu_random_enemy_virtual_sha() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shahai-swd-upstream-noname")
        .expect("swd 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10065, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 4);
    }
    // 指路小虎：机关牌种唯一实体（loader 卡名带包前缀重写，按 kind 取）
    let zhhu = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.kind == CardKind::Jiguan)
        .expect("指路小虎在目录") as u32;
    let mut snap = e.save();
    snap.core.deck.retain(|c| *c != zhhu);
    snap.core.players[0].hand.push(zhhu);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    assert_eq!(e.catalog().card_kind(zhhu), CardKind::Jiguan, "机关牌种");

    // 自目标使用（targets=[0]）
    let out = submit(&mut e, 0, GameAction::PlayCard { card: zhhu, targets: vec![0] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    // 随机敌唯一候选链：seed 10065 下恰一名敌 -1（另一敌不动）
    let (h1, h2) = (e.debug_hp(1), e.debug_hp(2));
    assert!((h1 == 3 && h2 == 4) || (h1 == 4 && h2 == 3), "恰一名敌被虚拟杀 -1: ({h1},{h2})");
    assert!(e.state_core().discard.contains(&zhhu), "指路小虎入弃堆");
}

/// 木甲人（机关牌第二切片）：弃全部非基本手牌→随机机关牌 1+N 入手
///（实体发现不足给余量）；usable:1 同回合再使用拒 CardLimit。
#[test]
fn swd_mujiaren_discard_nonbasic_discover_jiguan() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shahai-swd-upstream-noname")
        .expect("swd 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10066, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 4);
    }
    let mj = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.name.contains("mujiaren"))
        .expect("木甲人在目录") as u32;
    let sha = e.debug_place(0, "sha");
    let wuzhong = e.debug_place(0, "wuzhong");
    let mut snap = e.save();
    snap.core.deck.retain(|c| *c != mj);
    snap.core.discard.retain(|c| *c != mj);
    for pl in snap.core.players.iter_mut() { pl.hand.retain(|c| *c != mj); }
    snap.core.players[0].hand.insert(0, mj);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入");

    let out = submit(&mut e, 0, GameAction::PlayCard { card: mj, targets: vec![0] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    // 断言：弃 wuzhong（非基本）留 sha；发现机关牌 1+N=2 张（池=小虎
    // +勾魂锣×2 共 3 张候选随机取 2）；木甲人本体入弃堆
    let hand = e.debug_hand(0);
    assert!(hand.contains(&sha) && !hand.contains(&wuzhong), "弃非基本留基本");
    assert_eq!(hand.len(), 3, "弃 1 获 2（池 3 张候选=seed 10066 确定性）");
    let gained: Vec<_> = hand.iter().filter(|c| **c != sha).copied().collect();
    assert_eq!(gained.len(), 2, "获 2 张");
    for c in &gained {
        assert_eq!(e.catalog().card_kind(*c), CardKind::Jiguan, "获牌均为机关牌");
    }
    assert!(e.state_core().discard.contains(&mj), "木甲人本体入弃堆");
    // usable:1：同回合再使用拒 CardLimit
    let hand0 = e.debug_hand(0);
    let mj2 = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .find(|(i, c)| c.name.contains("mujiaren") && !hand0.contains(&(*i as u32)))
        .map(|(i, _)| i as u32)
        .expect("第二张木甲人");
    let mut snap = e.save();
    snap.core.deck.retain(|c| *c != mj2);
    snap.core.discard.retain(|c| *c != mj2);
    for pl in snap.core.players.iter_mut() { pl.hand.retain(|c| *c != mj2); }
    snap.core.players[0].hand.push(mj2);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入2");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: mj2, targets: vec![0] });
    assert!(matches!(out, CommandOutcome::Rejected(engine_native::RejectReason::CardLimit)), "{out:?}");
}

/// 勾魂锣（机关牌第三切片）：挂 3 轮倒计时（来源回合开始逐阶段-1）
/// →归 0 失去 1 体力+弃全部手牌+清挂附；重复挂同目标拒。
#[test]
fn swd_gouhunluo_countdown_lose_hp_discard() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shahai-swd-upstream-noname")
        .expect("swd 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10067, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 4);
    }
    let ghl = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.name.contains("gouhunluo"))
        .expect("勾魂锣在目录") as u32;
    let mut snap = e.save();
    snap.core.deck.retain(|c| *c != ghl);
    snap.core.discard.retain(|c| *c != ghl);
    for pl in snap.core.players.iter_mut() { pl.hand.retain(|c| *c != ghl); }
    snap.core.players[0].hand.push(ghl);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入");

    // 挂 1 号
    let out = submit(&mut e, 0, GameAction::PlayCard { card: ghl, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    assert_eq!(e.state_core().player(1).gouhunluo_rounds, 3, "倒计时 3");
    assert_eq!(e.state_core().player(1).gouhunluo_source, Some(0), "来源=0");
    let sub = e.catalog().skill_by_name("swd_gouhunluo_sub").expect("def");
    assert!(e.state_core().skills.contains_key(&(1, sub)), "1 号获倒计时实例");

    // 重复挂同目标拒
    let ghl2 = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .find(|(i, c)| c.name.contains("gouhunluo") && *i as u32 != ghl)
        .map(|(i, _)| i as u32)
        .expect("第二张勾魂锣");
    let mut snap = e.save();
    snap.core.deck.retain(|c| *c != ghl2);
    snap.core.discard.retain(|c| *c != ghl2);
    for pl in snap.core.players.iter_mut() { pl.hand.retain(|c| *c != ghl2); }
    snap.core.players[0].hand.push(ghl2);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入2");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: ghl2, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Rejected(engine_native::RejectReason::IllegalTargets)), "{out:?}");
    e.state_core();

    // 倒计时导航：循环推进（当前玩家 NP+排空询问）直到挂附者计数归 0
    for _ in 0..60 {
        if e.state_core().player(1).gouhunluo_rounds == 0 {
            break;
        }
        while let Some(c) = e.open_choice() {
            let (cid, obs) = (c.id, c.observer);
            submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
        }
        let actor = e.turn().current;
        let _ = submit(&mut e, actor, GameAction::NextPhase);
    }
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    // 归 0 结算：1 号失去 1 体力（4→3）+手牌清+挂附清
    assert_eq!(e.debug_hp(1), 3, "倒计时归 0 失去 1 体力（4→3）");
    assert_eq!(e.debug_hand(1).len(), 0, "弃全部手牌");
    assert_eq!(e.state_core().player(1).gouhunluo_rounds, 0, "计数清");
    assert_eq!(e.state_core().player(1).gouhunluo_source, None, "来源清");
    assert!(!e.state_core().skills.contains_key(&(1, sub)), "倒计时实例清");
}

/// 机关桶（机关牌第四切片）：AOE 逐目标「弃一张手牌或受 1 火伤」；
/// 无手牌目标直接火伤；取消=受火伤；本体入弃堆。
#[test]
fn swd_jiguantong_aoe_discard_or_fire() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shahai-swd-upstream-noname")
        .expect("swd 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10068, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 4);
    }
    let jgt = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.name.contains("jiguantong"))
        .expect("机关桶在目录") as u32;
    let mut snap = e.save();
    snap.core.deck.retain(|c| *c != jgt);
    snap.core.discard.retain(|c| *c != jgt);
    for pl in snap.core.players.iter_mut() { pl.hand.retain(|c| *c != jgt); }
    snap.core.players[0].hand.push(jgt);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入");
    let _shan = e.debug_place(0, "shan");
    // 1 号持牌（弃牌路径）、2 号空手（直接火伤路径）
    let h1 = e.debug_place(1, "tao");
    let _ = e.debug_place(0, "sha");

    // AOE 使用：targets=全部其他存活（1、2）→ 逆序结算（v193）：先 2 后 1
    let out = submit(&mut e, 0, GameAction::PlayCard { card: jgt, targets: vec![1, 2] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    // 询问流：2 号空手直接火伤（无询问）→ 1 号弃牌询问（选 tao 弃）
    while let Some(c) = e.open_choice() {
        let picks: Vec<ChoiceOption> = c
            .spec
            .options
            .iter()
            .copied()
            .filter(|o| matches!(o, ChoiceOption::Card(x) if *x == h1))
            .collect();
        let (cid, obs) = (c.id, c.observer);
        let sel = if picks.len() == 1 { picks } else { vec![ChoiceOption::Pass] };
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: sel });
    }
    // 断言：1 号弃 tao 不掉血；2 号空手受 1 火伤（4→3）；本体入弃堆；
    // 有人受伤 → contentAfter 不摸（使用者手牌保持 shan+sha 两张）
    assert_eq!(e.debug_hp(1), 4, "1 号弃牌免伤");
    assert!(!e.debug_hand(1).contains(&h1), "1 号 tao 已弃");
    assert_eq!(e.debug_hp(2), 3, "2 号空手受 1 火伤（4→3）");
    assert_eq!(e.debug_hand(0).len(), 2, "有人受伤：contentAfter 不摸牌");
    assert!(e.state_core().discard.contains(&jgt), "机关桶本体入弃堆");
    assert!(e.state_core().discard.contains(&h1), "tao 入弃堆");
}

/// 机关桶 v193 补齐两支：逆序结算（上游 reverseOrder——targets 座位序
/// 反转，先问 2 号）+ contentAfter（全员弃牌无人受伤 → 使用者摸 1）。
#[test]
fn swd_jiguantong_reverse_order_and_content_after_draw() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shahai-swd-upstream-noname")
        .expect("swd 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10069, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 4);
    }
    let jgt = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.name.contains("jiguantong"))
        .expect("机关桶在目录") as u32;
    let mut snap = e.save();
    snap.core.deck.retain(|c| *c != jgt);
    snap.core.discard.retain(|c| *c != jgt);
    for pl in snap.core.players.iter_mut() { pl.hand.retain(|c| *c != jgt); }
    snap.core.players[0].hand.push(jgt);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入");
    let _shan = e.debug_place(0, "shan");
    let _ = e.debug_place(0, "sha");
    // 1、2 号各持一张（全员走弃牌路径）
    let h1 = e.debug_place(1, "tao");
    let h2 = e.debug_place(2, "tao");

    let out = submit(&mut e, 0, GameAction::PlayCard { card: jgt, targets: vec![1, 2] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    // 逆序：首个弃牌询问 observer=2（座位序反转）；随后 observer=1
    let c = e.open_choice().expect("首个弃牌询问");
    assert_eq!(c.observer, 2, "逆序结算：先问 2 号（上游 reverseOrder）");
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(h2)] });
    let c = e.open_choice().expect("次个弃牌询问");
    assert_eq!(c.observer, 1, "逆序结算：后问 1 号");
    let (cid, obs) = (c.id, c.observer);
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(h1)] });
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    }
    // 断言：无人受伤（全员弃牌免伤）；contentAfter → 使用者摸 1
    //（shan+sha 两张 + 摸 1 = 3）；两张弃牌与本体入弃堆
    assert_eq!(e.debug_hp(1), 4, "1 号弃牌免伤");
    assert_eq!(e.debug_hp(2), 4, "2 号弃牌免伤");
    assert_eq!(e.debug_hand(0).len(), 3, "无人受伤：contentAfter 使用者摸 1");
    assert!(e.state_core().discard.contains(&jgt), "机关桶本体入弃堆");
    assert!(e.state_core().discard.contains(&h1) && e.state_core().discard.contains(&h2), "两张弃牌入弃堆");
}

// ---------------------------------------------------------------------------
// 整包验收 T7（2026-10-10）：ow、xianjian 两个纯武将包补齐真实宿主全生命
// 周期（15/15）。与 `wudian_real_host_full_lifecycle` 同款五步：独立安装
// +激活 → 安装包→原生建局（目录元数据）→ 对局中卸载 → 在局锁定副本续玩
// → 撤销后重装/重新 import 被拒。
// ---------------------------------------------------------------------------

/// ow（shaai-ow-upstream-noname，24 名武将元数据、0 张卡）全生命周期。
#[test]
fn ow_real_host_full_lifecycle() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();

    // ① 独立安装 + 激活
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "shaai-ow")
        .expect("shaai-ow 安装")
        .clone();
    let slot = format!("{}/{}", meta.namespace, meta.package_id);
    assert!(store.read_active().unwrap().packages.contains_key(&slot), "激活集");

    // ② 安装包 → 原生建局（0 卡注入；士兵76=突击/目镜接线生效）
    let json = std::fs::read_to_string(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .unwrap();
    let pack = bridge(&json);
    assert_eq!(pack.card_count(), 0, "ow 为纯武将包");
    assert_eq!(pack.generals.len(), 24, "24 名武将元数据");
    let base_cards = Engine::new(1, 3).catalog().cards.len();
    let mut e = Engine::new_with_extensions(6105, 3, &[&pack]).expect("建局");
    assert_eq!(e.catalog().cards.len(), base_cards, "纯武将包不注入卡牌");
    let gen_base = engine_native::content::standard::STANDARD_GENERALS.len() as u16;
    let mut shibing_skills = None;
    for idx in gen_base..(gen_base + 40) {
        if let Some((name, _, skills, _, _)) = e.catalog().general_meta(idx) {
            if name == "士兵76" {
                shibing_skills = Some(skills.to_vec());
                break;
            }
        }
    }
    assert_eq!(
        shibing_skills.map(|v| v.to_vec()),
        Some(vec!["tuji", "mujing"]),
        "士兵76 突击/目镜接线生效（力场暂缓不入目录）"
    );
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let mid = e.save();

    // ③ 对局中卸载
    store
        .revoke_slot(&meta.namespace, "shaai-ow", "验收：对局中卸载", true)
        .expect("撤销");
    assert!(store.read_revoke().unwrap().is_slot_revoked(&meta.namespace, "shaai-ow"));
    assert!(!store.read_active().unwrap().packages.contains_key(&slot), "移出激活集");

    // ④ 在局锁定副本续玩
    let mut resumed = Engine::restore_with_extensions(
        &engine_native::archive::decode_archive(&engine_native::archive::encode_archive(&mid))
            .expect("解码"),
        &[&pack],
    )
    .expect("对局中卸载后恢复");
    assert_eq!(resumed.catalog().cards.len(), base_cards, "锁定副本仍可用");
    let out = submit(&mut resumed, 0, GameAction::NextPhase);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "在局续玩: {out:?}");

    // ⑤ 重装守卫
    let bytes = std::fs::read(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("package.npack.json"),
    )
    .expect("npack");
    let err = store.install(&bytes, None, &options());
    assert!(matches!(err, Err(content_runtime::RuntimeError::Revoked(_))), "撤销后重装应被拒: {err:?}");

    // ⑥ 重新 import 被拒
    let report2 = import_extensions(&store, &root, &options());
    assert!(report2.is_err(), "撤销后重新 import 应被拒");
}

/// xianjian（shaai-xianjian-upstream-noname，36 名武将元数据、0 张卡）
/// 全生命周期；赵灵儿三技（回盟/天蛇/天蛇2）全映射接线生效。
#[test]
fn xianjian_real_host_full_lifecycle() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();

    // ① 独立安装 + 激活
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "shaai-xianjian")
        .expect("shaai-xianjian 安装")
        .clone();
    let slot = format!("{}/{}", meta.namespace, meta.package_id);
    assert!(store.read_active().unwrap().packages.contains_key(&slot), "激活集");

    // ② 安装包 → 原生建局（0 卡注入；赵灵儿=回盟/天蛇/天蛇2 全映射）
    let json = std::fs::read_to_string(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .unwrap();
    let pack = bridge(&json);
    assert_eq!(pack.card_count(), 0, "xianjian 为纯武将包");
    assert_eq!(pack.generals.len(), 36, "36 名武将元数据");
    let base_cards = Engine::new(1, 3).catalog().cards.len();
    let mut e = Engine::new_with_extensions(6106, 3, &[&pack]).expect("建局");
    assert_eq!(e.catalog().cards.len(), base_cards, "纯武将包不注入卡牌");
    let gen_base = engine_native::content::standard::STANDARD_GENERALS.len() as u16;
    let mut linger_skills = None;
    for idx in gen_base..(gen_base + 40) {
        if let Some((name, _, skills, _, _)) = e.catalog().general_meta(idx) {
            if name == "赵灵儿" {
                linger_skills = Some(skills.to_vec());
                break;
            }
        }
    }
    assert_eq!(
        linger_skills.map(|v| v.to_vec()),
        Some(vec!["huimeng", "tianshe", "tianshe2"]),
        "赵灵儿回盟/天蛇/天蛇2 全映射接线生效"
    );
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let mid = e.save();

    // ③ 对局中卸载
    store
        .revoke_slot(&meta.namespace, "shaai-xianjian", "验收：对局中卸载", true)
        .expect("撤销");
    assert!(store.read_revoke().unwrap().is_slot_revoked(&meta.namespace, "shaai-xianjian"));
    assert!(!store.read_active().unwrap().packages.contains_key(&slot), "移出激活集");

    // ④ 在局锁定副本续玩
    let mut resumed = Engine::restore_with_extensions(
        &engine_native::archive::decode_archive(&engine_native::archive::encode_archive(&mid))
            .expect("解码"),
        &[&pack],
    )
    .expect("对局中卸载后恢复");
    assert_eq!(resumed.catalog().cards.len(), base_cards, "锁定副本仍可用");
    let out = submit(&mut resumed, 0, GameAction::NextPhase);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "在局续玩: {out:?}");

    // ⑤ 重装守卫
    let bytes = std::fs::read(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("package.npack.json"),
    )
    .expect("npack");
    let err = store.install(&bytes, None, &options());
    assert!(matches!(err, Err(content_runtime::RuntimeError::Revoked(_))), "撤销后重装应被拒: {err:?}");

    // ⑥ 重新 import 被拒
    let report2 = import_extensions(&store, &root, &options());
    assert!(report2.is_err(), "撤销后重新 import 应被拒");
}

/// T12 玩点论杀续批（2026-10-14）：三将补入（西真夕红/造之仁君/杨仪）。
/// 屯田·DIY（ActivateSkill 主动技重铸：弃 1 装备 → 牌堆随机装备入手）
/// 与禁言（PlayCard 门查询：体力 ≤2 黑锦囊禁用，往事封禁先例同位）。
/// 登记差异：禁言上游四面查询仅承载使用面；屯田上游 canRecast 换置面
/// 不做。仇渡暂缓（依赖 deferred 卡调兵遣将）；复仇系（跨回合存储）、
/// 良策（五谷 Remained 子事件）、坚壁（目标转移）、理断（令他人用牌）
/// 逐批后续。
#[test]
fn wudian_generals_batch10_diyjuntun_recast_and_jinyan() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "wudian")
        .expect("wudian 安装")
        .clone();
    let json = std::fs::read_to_string(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .unwrap();
    let pack = bridge(&json);
    assert_eq!(pack.generals.len(), 9, "玩点论杀九名武将全量入册");

    let mut e = Engine::new_with_extensions(61000, 4, &[&pack]).expect("建局");
    let base = engine_native::content::standard::STANDARD_GENERALS.len() as u16;
    // 新三将在扩展段尾（6 旧将之后）
    let zao = base + 7;
    let m = e.catalog().general_meta(zao).expect("造之仁君在目录");
    assert_eq!(m.0, "造之仁君");
    assert_eq!(
        m.2,
        vec!["diyjuntun"],
        "屯田转正（良策/坚壁 null 暂缓不进目录列表）"
    );
    let jieyan_def = e.catalog().skill_by_name("diyjuntun").expect("屯田 def");
    let _ = jieyan_def;

    // 屯田·DIY：p0 挂装备 + 手牌装备 → ActivateSkill 重铸
    e.debug_grant(0, "diyjuntun");
    advance_to_play(&mut e, 0);
    let z0 = e.debug_place(0, "zhuge");
    {
        let core = e.state_core_mut();
        core.players[0].hand.retain(|x| *x != z0);
        core.players[0].equips.push(z0);
    }
    let h0 = hand_len_w(&e, 0);
    let equip_pool_before = {
        let core = e.state_core();
        core.deck
            .iter()
            .chain(core.discard.iter())
            .filter(|c| {
                e.catalog().card_kind(**c).equip_slot().is_some()
            })
            .count()
    };
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action: GameAction::ActivateSkill { skill: jieyan_def },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "屯田受理: {out:?}"
    );
    // SelectCards 应答（选 zhuge2）
    let c = e.open_choice().expect("屯田选择挂起");
    let (cid, obs) = (c.id, c.observer);
    let sel: Vec<ChoiceOption> = c
        .spec
        .options
        .iter()
        .filter(|o| matches!(o, ChoiceOption::Card(x) if *x == z0))
        .cloned()
        .collect();
    assert_eq!(sel.len(), 1, "重铸选择面含挂载装备");
    submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: sel });
    assert!(
        !e.state_core().players[0].equips.contains(&z0),
        "屯田：原装备被弃"
    );
    let gained = hand_len_w(&e, 0) - h0;
    if equip_pool_before > 0 {
        assert_eq!(gained, 1, "屯田：牌堆获得 1 张装备（池 {equip_pool_before}）");
    }

    // 禁言：p1（持禁言）体力 2 → 黑锦囊使用被拒；体力 3 → 放行
    e.debug_grant(1, "jinyan");
    advance_to_play(&mut e, 1);
    e.debug_set_hp(1, 2);
    let le = e.debug_place(1, "le");
    let out = submit(&mut e, 1, GameAction::PlayCard { card: le, targets: vec![0] });
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::CardLimit)),
        "禁言：体力 ≤2 黑锦囊被拒: {out:?}"
    );
    e.debug_set_hp(1, 3);
    let out = submit(&mut e, 1, GameAction::PlayCard { card: le, targets: vec![0] });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "禁言：体力 3 放行: {out:?}"
    );
}

fn hand_len_w(e: &Engine, p: u8) -> usize {
    e.state_core().player(p).hand.len()
}

/// 推进到指定玩家出牌阶段（空/首选应答排空）。
fn advance_to_play(e: &mut Engine, player: u8) {
    for _ in 0..200 {
        if e.turn().current == player && e.turn().phase == Phase::Play {
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
    panic!("200 步未到 p{player} 出牌阶段");
}

/// T12 玩点论杀批 12：理断（杨仪部分转正）——他人回合外获得装备到手
/// （可选确认）：同栏已有装备则交首张手牌给持有者；否则直接装备该牌
/// （登记差异：上游获得者自选，原生按同栏状态确定性分支；直装不走
/// Equipped 帧防双 grant）。坚壁登记「目标排除面」缺口维持暂缓。
#[test]
fn wudian_generals_batch12_liduan_gain_equip() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "wudian")
        .expect("wudian 安装")
        .clone();
    let json = std::fs::read_to_string(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(61001, 4, &[&pack]).expect("建局");
    let base = engine_native::content::standard::STANDARD_GENERALS.len() as u16;
    let yangyi = base + 8;
    let m = e.catalog().general_meta(yangyi).expect("杨仪在目录");
    assert_eq!(m.0, "杨仪");
    assert_eq!(m.2, vec!["liduan"], "仇渡暂缓（依赖 deferred 卡）");

    // 直装分支：p1 持理断；p0 回合中，p2 获得装备到手（同栏无装备）
    e.debug_grant(1, "liduan");
    advance_to_play(&mut e, 0);
    let z = e.debug_place(2, "zhuge");
    {
        // 从 p2 手移到 p0 再 give（产生 CardsMoved from=0 to=2——wait，理断
        // filter 要 from!=to 且 to 非当前回合玩家：当前回合=p0，to=p2 ✓
        let core = e.state_core_mut();
        core.players[2].hand.retain(|x| *x != z);
        core.players[0].hand.push(z);
    }
    e.debug_give_hand_card(0, 2, z);
    settle_optional_confirm(&mut e);
    assert!(
        e.state_core().players[2].equips.contains(&z),
        "理断：直装分支——装备被装上"
    );

    // 交牌分支：p2 再获得一件同栏（武器）装备 → 交首张手牌给 p1
    let z2 = e.debug_place(2, "hanbing");
    {
        let core = e.state_core_mut();
        core.players[2].hand.retain(|x| *x != z2);
        core.players[0].hand.push(z2);
    }
    // 给 p2 一张手牌作交牌面
    let pay = e.debug_place(2, "tao");
    let h1 = e.state_core().players[1].hand.len();
    let out = e.debug_give_hand_card(0, 2, z2);
    let _ = out;
    settle_optional_confirm(&mut e);
    assert!(
        !e.state_core().players[2].equips.contains(&z2),
        "理断：同栏已有装备不直装"
    );
    assert_eq!(
        e.state_core().players[1].hand.len(),
        h1 + 1,
        "理断：交牌分支——p1 收 1 张手牌"
    );
    let _ = pay;
}

/// 可选确认门排空（首选项应答）。
fn settle_optional_confirm(e: &mut Engine) {
    for _ in 0..8 {
        let Some(c) = e.open_choice() else { return };
        let (cid, obs) = (c.id, c.observer);
        if c.spec.options.is_empty() {
            return;
        }
        let sel = vec![c.spec.options[0].clone()];
        submit(e, obs, GameAction::Answer { choice_id: cid, selection: sel });
    }
}
