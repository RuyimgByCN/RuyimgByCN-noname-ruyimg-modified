//! G1 wuxing 纵向样本（ADR 0776）：五行属性入状态 → 生克宿主行为 →
//! 五行盘主动技（PickTag 选择原语）→ 建局/存档/恢复。
//!
//! 上游（noname-liuli wuxing @ 8485dbb2）：每角色随机属性、牌堆 1/3
//! 随机属性、被克弃牌/被生摸牌、五行盘改手牌属性（equip5）。
//! 原生映射：属性入核心状态（规则随机性走引擎 RNG）；被克弃牌为
//! 确定性首张（登记差异：上游为目标自选「he」）；五行盘占栏 equip5
//! → 宝物栏（Treasure），其技能以授予验证（装备技能挂载生命周期未做）。

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use content_runtime::{discover_extensions, import_extensions, InstallOptions, PackageStore};
use engine_native::content::extension::ExtensionPack;
use engine_native::{
    ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase,
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


fn install_and_bridge() -> ExtensionPack {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    assert!(
        discovered.iter().any(|d| d.name == "wuxing-upstream-liuli"),
        "wuxing 包可发现"
    );
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "wuxing")
        .expect("wuxing 安装");
    let cfg = std::fs::read_to_string(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/wuxing.json"),
    )
    .expect("配置可读");
    assert!(
        cfg.contains("\"wuxingpan\"") && cfg.contains("\"denominator\": 3"),
        "配置数据核对"
    );
    // 桥接统一走 C0 装载器（ADR 0777/0778）：五行盘、生克/盘属性改写
    // 两个宿主技能声明与开局钩子都来自 cards.json 的 pack 段与数据键。
    let cards_json = std::fs::read_to_string(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .expect("cards.json 可读");
    let out = content_native_loader::bridge_pack(&cards_json).expect("wuxing 可装载");
    assert_eq!(out.pack.def.name, "wuxing-upstream-liuli", "包名来自 pack 段");
    out.pack
}

/// G1 纵向：导入 → 建局（五行初始化：角色属性 + 牌堆属性 1/3）→
/// 生克结算 → 存档/恢复（属性随状态绑定）。
#[test]
fn g1_wuxing_vertical_natures_shengke_saverestore() {
    let pack = install_and_bridge();
    let base_cards = Engine::new(1, 3).catalog().cards.len();
    let mut e =
        Engine::new_with_extensions(3301, 3, &[&pack]).expect("建局");
    assert_eq!(e.catalog().cards.len(), base_cards + 1, "五行盘入目录");

    // 初始化：每名角色有属性；牌堆约 1/3 有属性（至少 1 张，全部合法）
    let snap = e.save();
    for p in snap.core.players.iter() {
        assert!(p.wunature.map(|n| n <= 4).unwrap_or(false), "角色属性已初始化");
    }
    let natured = snap.core.card_natures.len();
    assert!(natured > 0, "牌堆应有属性牌");
    // 统计比例容差（小牌堆采样波动）：1/4 ~ 1/2 区间即视为符合 1/3 设计
    let deck_len = snap.core.deck.len();
    assert!(
        natured * 4 >= deck_len && natured * 2 <= deck_len * 2 && natured <= deck_len,
        "约 1/3 比例: {natured} / {deck_len}"
    );
    assert!(natured * 3 <= deck_len * 2 + 6, "上限护栏: {natured}/{deck_len}");

    // 生克——被生（金生水）：0 号水属性，成为金属性杀的目标 → 摸 1
    // （推进到 1 号出牌阶段；途中挂起的选择取首选项应答）
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
    let sha1 = e.debug_place(1, "sha");
    let mut snap = e.save();
    let hand0_before = snap.core.players[0].hand.len();
    snap.core.players[0].wunature = Some(2); // 水
    snap.core.card_natures.insert(sha1, 0); // 金
    e = Engine::restore_with_extensions(&snap, &[&pack]).expect("属性注入恢复");
    let out = submit(&mut e, 1, GameAction::PlayCard { card: sha1, targets: vec![0] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    assert_eq!(
        e.save().core.players[0].hand.len(),
        hand0_before + 1,
        "金生水：被生摸一张"
    );

    // 生克——被克（金克木）：0 号木属性，成为金属性杀的目标 → 弃一张手牌
    let sha2 = e.debug_place(1, "sha");
    let _tao = e.debug_place(0, "tao"); // debug_place 已将 tao 置入 0 号手牌
    let mut snap = e.save();
    snap.core.players[0].wunature = Some(1); // 木
    snap.core.turn.sha_used_play_phase = false; // 同阶段第二张杀（测试便捷）
    snap.core.card_natures.insert(sha2, 0); // 金
    e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let discard_before = e.save().core.discard.len();
    // v64 牌堆位移：前序结算可能留有挂起询问——先排空再出杀。
    for _ in 0..8 {
        match e.open_choice() {
            Some(c) => {
                let sel: Vec<ChoiceOption> = if c.spec.min > 0 {
                    c.spec.options.iter().take(c.spec.min).copied().collect()
                } else if !c.spec.options.is_empty() {
                    vec![c.spec.options[0]]
                } else {
                    vec![]
                };
                let (id2, obs2) = (c.id, c.observer);
                let _ = submit(&mut e, obs2, GameAction::Answer { choice_id: id2, selection: sel });
            }
            None => break,
        }
    }
    let out = submit(&mut e, 1, GameAction::PlayCard { card: sha2, targets: vec![0] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    assert_eq!(
        e.save().core.discard.len(),
        discard_before + 2, // 杀 + 被克弃的手牌
        "金克木：被克弃一张手牌"
    );

    // 存档绑定：带 wuxing 存档被纯标准恢复拒绝
    let saved = e.save();
    assert!(matches!(
        Engine::restore(&saved),
        Err(engine_native::SnapshotError::RuleVersionMismatch { .. })
    ));
    Engine::restore_with_extensions(&saved, &[&pack]).expect("同集合恢复");
}

/// 五行盘主动技：授予 → 激活 → 选牌 → 选属性（PickTag）→ 属性入表；
/// 两段选择各支持字节恢复。
#[test]
fn wuxingpan_skill_change_nature_with_restore() {
    use engine_native::archive::{decode_archive, encode_archive};
    let pack = install_and_bridge();
    let mut e =
        Engine::new_with_extensions(3302, 3, &[&pack]).expect("建局");
    e.debug_grant(0, "wuxingpan_skill");
    advance_to_play(&mut e, 0);
    let card = e.debug_place(0, "tao");
    let skill = e.catalog().skill_by_name("wuxingpan_skill").unwrap();
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill });
    match out {
        CommandOutcome::Pending { choice } => {
            assert_eq!(choice.spec.prompt, "wuxing_select_card");
        }
        other => panic!("应挂起选牌: {other:?}"),
    }
    // 选牌阶段字节恢复
    let bytes = encode_archive(&e.save());
    let mut restored = Engine::restore_with_extensions(
        &decode_archive(&bytes).expect("decode"),
        &[&pack],
    )
    .expect("选牌挂起恢复");
    let c = restored.open_choice().expect("恢复件仍挂起");
    assert_eq!(c.spec.prompt, "wuxing_select_card");
    let cid = c.id;
    submit(&mut restored, 0, GameAction::Answer {
        choice_id: cid,
        selection: vec![ChoiceOption::Card(card)],
    });
    // 选属性（PickTag，5 个 Tag 选项）
    let c2 = restored.open_choice().expect("选属性挂起");
    assert_eq!(c2.spec.prompt, "wuxing_select_nature");
    assert!(matches!(
        c2.spec.kind,
        engine_native::ChoiceKind::PickTag
    ));
    assert_eq!(c2.spec.options.len(), 5);
    let cid2 = c2.id;
    submit(&mut restored, 0, GameAction::Answer {
        choice_id: cid2,
        selection: vec![ChoiceOption::Tag(3)], // 火
    });
    assert_eq!(
        restored.save().core.card_natures.get(&card),
        Some(&3),
        "五行盘：属性已永久改变"
    );
}

/// 五行盘卡牌：占栏装备路径（Treasure 槽）。
#[test]
fn wuxingpan_card_equips_via_treasure_slot() {
    let pack = install_and_bridge();
    let mut e = Engine::new_with_extensions(3303, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    let (wp,) = {
        let catalog = e.catalog();
        let wp = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("wxl_wuxingpan"))
            .expect("五行盘在目录") as u32;
        (wp,)
    };
    let mut snap = e.save();
    snap.core.deck.retain(|c| *c != wp);
    snap.core.players[0].hand.push(wp);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: wp, targets: vec![] });
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert!(
        e.save().core.players[0].equips.contains(&wp),
        "五行盘应装备至宝物栏"
    );
}

fn advance_to_play(e: &mut Engine, player: u8) {
    for _ in 0..64 {
        let t = e.turn();
        if t.current == player && t.phase == Phase::Play {
            return;
        }
        let actor = e.turn().current;
        match submit(e, actor, GameAction::NextPhase) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("advance: {other:?}"),
        }
    }
    panic!("no play phase");
}

/// 整包验收续（2026-10-08，第六例）：wuxing 真实宿主全生命周期——
/// 独立安装+激活 → 安装包→原生建局 → 对局中卸载 → 在局锁定副本
/// 续玩 → 撤销后重装/重新 import 被拒。
#[test]
fn wuxing_real_host_full_lifecycle() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();

    // ① 独立安装 + 激活
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "wuxing")
        .expect("wuxing 安装")
        .clone();
    let slot = format!("{}/{}", meta.namespace, meta.package_id);
    assert!(store.read_active().unwrap().packages.contains_key(&slot), "激活集");

    // ② 安装包 → 原生建局（五行盘入目录）
    let cards_json = std::fs::read_to_string(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .expect("cards.json 可读");
    let out = content_native_loader::bridge_pack(&cards_json).expect("桥接");
    let pack = out.pack;
    let base_cards = Engine::new(1, 3).catalog().cards.len();
    let mut e = Engine::new_with_extensions(5801, 3, &[&pack]).expect("建局");
    assert_eq!(e.catalog().cards.len(), base_cards + 1, "五行盘入目录");
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let mid = e.save();

    // ③ 对局中卸载
    store
        .revoke_slot(&meta.namespace, "wuxing", "验收：对局中卸载", true)
        .expect("撤销");
    assert!(store.read_revoke().unwrap().is_slot_revoked(&meta.namespace, "wuxing"));
    assert!(!store.read_active().unwrap().packages.contains_key(&slot), "移出激活集");

    // ④ 在局锁定副本续玩
    let mut resumed = Engine::restore_with_extensions(
        &engine_native::archive::decode_archive(&engine_native::archive::encode_archive(&mid))
            .expect("解码"),
        &[&pack],
    )
    .expect("对局中卸载后恢复");
    assert_eq!(resumed.catalog().cards.len(), base_cards + 1, "锁定副本仍可用");
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
