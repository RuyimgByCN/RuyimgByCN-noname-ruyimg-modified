//! U01 末焱（2026-10-17）。
//!
//! 从真实安装的商店副本装载刘冲。末焱在一轮内首次判定阶段开始展示至多
//! 3 张牌，一名角色获得其中至少一张；之后持有者受伤时，若该角色存活且
//! 手牌数不大于获得张数，伤害 +1。取消、手牌超限、目标死亡、同一轮的
//! 下一次判定、牌库与弃牌堆皆空都不产生增幅。
//!
//! 角色表仍只有刘冲。其余 3D 武将和技能不在这个包里，本测试不把来源
//! 计成整包。

use std::path::PathBuf;

use content_runtime::{import_extensions, InstallOptions, PackageStore, RuntimeError};
use content_schema::ContentPackage;
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::content::extension::ExtensionPack;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

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

fn submit(engine: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    engine.submit(GameCommand {
        actor,
        expected_revision: engine.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action,
    })
}

fn accept(engine: &mut Engine, actor: u8, action: GameAction) {
    match submit(engine, actor, action) {
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
        other => panic!("command rejected: {other:?}"),
    }
}

fn store_character(store: &PackageStore) -> (ExtensionPack, String) {
    let dir = store
        .root()
        .join("packages")
        .join("upstream-noname__d3-select")
        .join("1.0.0");
    let npack = std::fs::read(dir.join("package.json")).expect("商店副本 package.json");
    let package = ContentPackage::from_json_bytes(&npack).expect("解析商店副本");
    assert!(
        package.manifest.dependencies.is_empty(),
        "依赖：不声明外部扩展"
    );
    let json = std::fs::read_to_string(dir.join("assets/character.json")).expect("商店副本角色");
    let bridged = content_native_loader::bridge_pack(&json).expect("商店副本可装载");
    assert_eq!(bridged.pack.def.name, "d3-upstream-noname");
    assert_eq!(bridged.pack.general_count(), 1, "角色表仍只有刘冲");
    let names: Vec<&str> = bridged
        .pack
        .generals
        .iter()
        .map(|general| general.name)
        .collect();
    assert_eq!(names, vec!["ddd刘冲"]);
    assert!(
        !json.contains("ddd_qianzhao"),
        "牵招不在包内，不能把元数据名单当成技能"
    );
    assert_eq!(
        bridged.pack.generals[0].skills,
        ["dddjinggou", "dddmoyan"].as_slice()
    );
    (bridged.pack, json)
}

fn engine_with(pack: &ExtensionPack, seed: u64, grant: bool) -> Engine {
    let mut engine = Engine::new_with_extensions(seed, 3, &[pack]).expect("建局");
    if grant {
        engine.debug_grant(0, "dddmoyan");
    }
    engine
}

fn top_cards(engine: &Engine, n: usize) -> Vec<u32> {
    let deck = &engine.save().core.deck;
    assert!(deck.len() >= n, "牌库不足");
    deck[deck.len() - n..].iter().rev().copied().collect()
}

fn arm_boundary(engine: &mut Engine) {
    engine.debug_set_turn(2, Phase::End, engine.turn().round);
    for player in 0..3 {
        engine.debug_clear_hand(player);
    }
}

fn open_moyan(engine: &mut Engine) -> Vec<u32> {
    arm_boundary(engine);
    let shown = top_cards(engine, 3);
    accept(engine, 2, GameAction::NextPhase);
    let choice = engine.open_choice().expect("末焱应挂起选人");
    assert_eq!(choice.spec.prompt, "d3_moyan_target");
    assert_eq!(choice.observer, 0);
    assert!(choice.spec.allow_cancel);
    assert!(choice.spec.options.contains(&ChoiceOption::Player(0)));
    assert!(choice.spec.options.contains(&ChoiceOption::Player(1)));
    assert!(
        engine.log().iter().any(|event| matches!(
            event,
            engine_native::LogEvent::CardsRevealed { cards, .. } if cards == &shown
        )),
        "展示的牌是牌库顶三张"
    );
    shown
}

fn answer(engine: &mut Engine, selection: Vec<ChoiceOption>) {
    let choice = engine.open_choice().expect("应有挂起选择").clone();
    accept(
        engine,
        choice.observer,
        GameAction::Answer {
            choice_id: choice.id,
            selection,
        },
    );
}

fn reload(bytes: &[u8], pack: &ExtensionPack) -> Engine {
    Engine::restore_with_extensions(&decode_archive(bytes).expect("decode"), &[pack])
        .expect("restore")
}

/// 末焱正反例。整包计数不因本测试上升。
#[test]
fn u01_moyan_from_real_install() {
    let source = repo_extensions_root().join("d3-upstream-noname");
    let npack_bytes = std::fs::read(source.join("package.npack.json")).expect("读 npack");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &repo_extensions_root(), &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|item| item.package_id == "d3-select" && item.namespace == "upstream-noname")
        .expect("3D精选已安装");
    assert_eq!(meta.version, "1.0.0");
    assert_eq!(
        meta.content_hash,
        content_runtime::content_hash_hex(&npack_bytes),
        "商店 content_hash 来自真实 npack 字节"
    );
    assert!(
        report
            .installed
            .iter()
            .any(|item| item.package_id == "huanle-cards"),
        "组合：同一次导入还有其他包"
    );
    match store.activate_version("upstream-noname", "d3-select", "9.9.9") {
        Err(RuntimeError::NotFound(_)) => {}
        other => panic!("未安装版本应 NotFound: {other:?}"),
    }

    let (pack, _) = store_character(&store);
    let base = Engine::new(1, 3).catalog().general_count();
    let probe = Engine::new_with_extensions(17101, 3, &[&pack]).expect("建局");
    assert_eq!(
        probe.catalog().general_count(),
        base + 1,
        "组合：刘冲接在标准武将之后"
    );
    assert!(
        probe.debug_has_skill(0, "dddmoyan") == false,
        "未授予时座位上没有末焱"
    );

    let mut gain = engine_with(&pack, 17102, true);
    let shown = open_moyan(&mut gain);
    let suspended = encode_archive(&gain.save());
    let mut gain = reload(&suspended, &pack);
    assert_eq!(
        gain.open_choice().expect("恢复后仍在选人").spec.prompt,
        "d3_moyan_target"
    );
    answer(&mut gain, vec![ChoiceOption::Player(1)]);
    let picking = gain.open_choice().expect("获得者选牌");
    assert_eq!(picking.spec.prompt, "d3_moyan_gain");
    assert_eq!(picking.observer, 1);
    assert_eq!(picking.spec.min, 1);
    assert_eq!(picking.spec.max, shown.len());
    assert!(!picking.spec.allow_cancel);
    let mid_gain = encode_archive(&gain.save());
    let mut gain = reload(&mid_gain, &pack);
    answer(
        &mut gain,
        vec![ChoiceOption::Card(shown[0]), ChoiceOption::Card(shown[1])],
    );
    assert!(gain.open_choice().is_none(), "两段选择结束后不应再挂起");
    assert!(gain.debug_hand(1).contains(&shown[0]));
    assert!(gain.debug_hand(1).contains(&shown[1]));
    assert!(!gain.debug_hand(1).contains(&shown[2]));
    assert!(
        gain.save().core.discard.contains(&shown[2]),
        "未选中的牌弃置"
    );
    let gained = encode_archive(&gain.save());

    let mut boosted = reload(&gained, &pack);
    let hp = boosted.debug_hp(0);
    accept_damage(&mut boosted);
    assert_eq!(boosted.debug_hp(0), hp - 2, "手牌数不大于获得张数时伤害 +1");

    let mut crowded = reload(&gained, &pack);
    {
        let mut snap = crowded.save();
        let extra = snap.core.deck.pop().expect("extra");
        snap.core.players[1].hand.push(extra);
        crowded = reload(&encode_archive(&snap), &pack);
    }
    let hp = crowded.debug_hp(0);
    accept_damage(&mut crowded);
    assert_eq!(crowded.debug_hp(0), hp - 1, "手牌数大于获得张数时不增幅");

    let mut dead = reload(&gained, &pack);
    {
        let mut snap = dead.save();
        snap.core.players[1].alive = false;
        dead = reload(&encode_archive(&snap), &pack);
    }
    let hp = dead.debug_hp(0);
    accept_damage(&mut dead);
    assert_eq!(dead.debug_hp(0), hp - 1, "获得者已死亡时不增幅");

    let mut same_round = reload(&gained, &pack);
    assert_eq!(same_round.turn().round, 2);
    for _ in 0..16 {
        if same_round.turn().current == 1
            && same_round.turn().phase == Phase::Judge
            && same_round.turn().round == 2
            && same_round.open_choice().is_none()
        {
            break;
        }
        if same_round.open_choice().is_some() {
            let prompt = same_round.open_choice().unwrap().spec.prompt;
            assert_ne!(prompt, "d3_moyan_target", "同一轮不应再次发动末焱");
            let choice = same_round.open_choice().unwrap().clone();
            let selection = if choice.spec.allow_cancel {
                vec![]
            } else {
                choice
                    .spec
                    .options
                    .iter()
                    .take(choice.spec.min.max(1))
                    .copied()
                    .collect()
            };
            answer(&mut same_round, selection);
        } else {
            let actor = same_round.turn().current;
            accept(&mut same_round, actor, GameAction::NextPhase);
        }
    }
    assert_eq!(same_round.turn().current, 1);
    assert_eq!(same_round.turn().phase, Phase::Judge);
    assert_eq!(same_round.turn().round, 2);
    assert!(same_round.open_choice().is_none());

    let mut expired = reload(&gained, &pack);
    expired.debug_set_turn(2, Phase::End, 2);
    for player in 0..3 {
        expired.debug_clear_hand(player);
    }
    accept(&mut expired, 2, GameAction::NextPhase);
    assert_eq!(expired.turn().round, 3);
    assert_eq!(
        expired.open_choice().expect("新一轮再次发动").spec.prompt,
        "d3_moyan_target"
    );
    answer(&mut expired, vec![]);
    assert!(expired.open_choice().is_none(), "取消后不再选牌");
    let hp = expired.debug_hp(0);
    accept_damage(&mut expired);
    assert_eq!(expired.debug_hp(0), hp - 1, "新一轮取消后旧增幅已清除");

    let mut cancelled = engine_with(&pack, 17103, true);
    let shown = open_moyan(&mut cancelled);
    answer(&mut cancelled, vec![]);
    assert!(cancelled.open_choice().is_none());
    for card in &shown {
        assert!(
            cancelled.save().core.discard.contains(card),
            "取消则展示牌全部弃置"
        );
        assert!(!cancelled.debug_hand(1).contains(card));
    }
    let hp = cancelled.debug_hp(0);
    accept_damage(&mut cancelled);
    assert_eq!(cancelled.debug_hp(0), hp - 1, "取消不留下增幅");

    let mut bare = engine_with(&pack, 17104, false);
    bare.debug_set_turn(2, Phase::End, 1);
    accept(&mut bare, 2, GameAction::NextPhase);
    assert!(bare.open_choice().is_none(), "未授予末焱时回合边界不选人");
    assert_eq!(bare.turn().round, 2);

    let mut empty = engine_with(&pack, 17105, true);
    {
        let mut snap = empty.save();
        let deck = std::mem::take(&mut snap.core.deck);
        let discard = std::mem::take(&mut snap.core.discard);
        snap.core.table.extend(deck);
        snap.core.table.extend(discard);
        empty = reload(&encode_archive(&snap), &pack);
    }
    empty.debug_set_turn(2, Phase::End, 1);
    accept(&mut empty, 2, GameAction::NextPhase);
    assert!(empty.open_choice().is_none(), "牌库与弃牌堆皆空时不发动");

    let mut one = engine_with(&pack, 17106, true);
    for player in 0..3 {
        one.debug_clear_hand(player);
    }
    let only = {
        let mut snap = one.save();
        let card = snap.core.deck.pop().expect("one");
        let rest = std::mem::take(&mut snap.core.deck);
        let discard = std::mem::take(&mut snap.core.discard);
        snap.core.table.extend(rest);
        snap.core.table.extend(discard);
        snap.core.deck.push(card);
        one = reload(&encode_archive(&snap), &pack);
        card
    };
    one.debug_set_turn(2, Phase::End, 1);
    accept(&mut one, 2, GameAction::NextPhase);
    answer(&mut one, vec![ChoiceOption::Player(0)]);
    let picking = one.open_choice().expect("只剩一张仍要选择");
    assert_eq!(picking.spec.max, 1);
    assert_eq!(picking.spec.options, vec![ChoiceOption::Card(only)]);
    answer(&mut one, vec![ChoiceOption::Card(only)]);
    assert!(one.debug_hand(0).contains(&only), "持有者可以获得给自己");
    let hp = one.debug_hp(0);
    accept_damage(&mut one);
    assert_eq!(
        one.debug_hp(0),
        hp - 2,
        "自己获得 1 张且手牌数为 1 时伤害 +1"
    );
}

fn accept_damage(engine: &mut Engine) {
    match engine.debug_deal_damage(Some(1), 0, 1, engine_native::event::DamageNature::Normal) {
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
        other => panic!("damage rejected: {other:?}"),
    }
}
