//! NC07 / GT-012（规则 v325）：炼妖壶壶仓（`lianyaohu_store`）顺序进入
//! 规范摘要。反例（v324 及更早）：两种壶序的存档摘要相同，但炼化选项
//! 按壶序生成、结算丢弃序也随之不同——同摘要不同语义。修复后摘要必须
//! 区分，且归档往返、炼化结算与日志轨迹同步区分；v324 及更早旧档按
//! 版本门禁拒绝（存档 v105、全局原生 ABI 不变）。
use engine_native::archive::decode_archive;
use engine_native::content::extension::ExtensionPack;
use engine_native::snap::SnapshotError;
use engine_native::{
    ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase,
};

fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    let out = e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: e.revision() + 1,
        action,
    });
    assert!(
        !matches!(out, CommandOutcome::Rejected(_)),
        "unexpected rejection: {out:?}"
    );
    out
}

fn load_pack() -> ExtensionPack {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let json = std::fs::read_to_string(
        root.join("../../packages/extensions/shahai-swd-upstream-noname/assets/cards.json"),
    )
    .unwrap();
    content_native_loader::bridge_pack(&json).unwrap().pack
}

/// 与 2026-10-28 基线捕获同布阵：0 号进 Play、授炼化、牌堆顶两张入壶仓。
fn jar_fixture() -> (Engine, ExtensionPack) {
    let pack = load_pack();
    let mut e = Engine::new_with_extensions(3504, 4, &[&pack]).unwrap();
    for _ in 0..8 {
        if e.turn().phase == Phase::Play {
            break;
        }
        submit(&mut e, 0, GameAction::NextPhase);
    }
    assert_eq!(e.turn().phase, Phase::Play);
    e.debug_grant(0, "lianyaohu_lianhua");
    let first = e.state_core_mut().deck.pop().unwrap();
    let second = e.state_core_mut().deck.pop().unwrap();
    e.state_core_mut().players[0].lianyaohu_store = vec![first, second];
    (e, pack)
}

fn activate_lianhua(e: &mut Engine) {
    let def = e.catalog().skill_by_name("lianyaohu_lianhua").unwrap();
    let out = submit(e, 0, GameAction::ActivateSkill { skill: def });
    assert!(
        matches!(out, CommandOutcome::Pending { .. }),
        "lianhua must open the jar pick window: {out:?}"
    );
}

fn answer_open(e: &mut Engine, selection: Vec<ChoiceOption>) {
    let choice = e.open_choice().cloned().expect("choice open");
    assert_eq!(choice.spec.prompt, "lianhua_pick");
    submit(
        e,
        choice.observer,
        GameAction::Answer {
            choice_id: choice.id,
            selection,
        },
    );
}

#[test]
fn different_jar_orders_have_different_hashes_and_trajectories() {
    let (engine, pack) = jar_fixture();
    let left = engine.save();
    let [first, second] = left.core.players[0].lianyaohu_store[..] else {
        panic!("jar fixture must hold two cards");
    };
    let mut right = left.clone();
    right.core.players[0].lianyaohu_store.reverse();

    let mut a = Engine::restore_with_extensions(&left, &[&pack]).unwrap();
    let mut b = Engine::restore_with_extensions(&right, &[&pack]).unwrap();
    assert_eq!(a.state_core().players[0].lianyaohu_store, vec![first, second]);
    assert_eq!(b.state_core().players[0].lianyaohu_store, vec![second, first]);
    // 反例的反转（v324 缺陷态为相等）：壶序不同摘要必须不同。
    assert_ne!(a.state_hash(), b.state_hash());

    // 归档往返不抹平差异，也不改变各自摘要。
    let ra = Engine::restore_with_extensions(&a.save(), &[&pack]).unwrap();
    let rb = Engine::restore_with_extensions(&b.save(), &[&pack]).unwrap();
    assert_eq!(ra.state_hash(), a.state_hash());
    assert_eq!(rb.state_hash(), b.state_hash());
    assert_ne!(ra.state_hash(), rb.state_hash());
    assert_eq!(
        ra.state_core().players[0].lianyaohu_store,
        a.state_core().players[0].lianyaohu_store
    );
    assert_eq!(
        rb.state_core().players[0].lianyaohu_store,
        b.state_core().players[0].lianyaohu_store
    );

    // 完整轨迹：炼化选项按各自壶序展示。
    activate_lianhua(&mut a);
    activate_lianhua(&mut b);
    let opts_a = a.open_choice().unwrap().spec.options.clone();
    let opts_b = b.open_choice().unwrap().spec.options.clone();
    assert_eq!(opts_a, vec![ChoiceOption::Card(first), ChoiceOption::Card(second)]);
    assert_eq!(opts_b, vec![ChoiceOption::Card(second), ChoiceOption::Card(first)]);
    // 各按自己窗口展示的顺序作答：结算丢弃序跟随壶序。
    answer_open(&mut a, opts_a);
    answer_open(&mut b, opts_b);
    assert!(a.state_core().players[0].lianyaohu_store.is_empty());
    assert!(b.state_core().players[0].lianyaohu_store.is_empty());
    let da = &a.state_core().discard;
    let db = &b.state_core().discard;
    assert_eq!(&da[da.len() - 2..], &[first, second]);
    assert_eq!(&db[db.len() - 2..], &[second, first]);
    assert_ne!(a.state_hash(), b.state_hash());
}

#[test]
fn v324_and_earlier_archives_are_rejected_by_rule_version() {
    let ev = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/operations/evidence/2026-10-28-nc07-lianyaohu-hash");
    // 普通档：标准目录版本直接拒绝。
    for (name, version) in [("native-v323.ngsa", 323u64), ("native-v324.ngsa", 324u64)] {
        let snap = decode_archive(&std::fs::read(ev.join(name)).unwrap()).unwrap();
        assert_eq!(snap.ruleset_version, version);
        assert!(matches!(
            Engine::restore(&snap),
            Err(SnapshotError::RuleVersionMismatch { snapshot, catalog })
                if snapshot == version && catalog == 330
        ));
    }
    // 壶仓档：目录版本 = 标准版本 ^（内容锁 | 高位）。v324 壶仓档正是
    // 「壶序未入摘要」时代的产物，升号后不能被静默接受。
    let pack = load_pack();
    let catalog_version = Engine::new_with_extensions(3504, 4, &[&pack])
        .unwrap()
        .ruleset_version();
    for (name, version) in [("lianyaohu-v323.ngsa", 323u64), ("lianyaohu-v324.ngsa", 324u64)] {
        let snap = decode_archive(&std::fs::read(ev.join(name)).unwrap()).unwrap();
        // 壶仓档与当前目录的差异只来自标准版本位（内容锁互相抵消）。
        assert_eq!(snap.ruleset_version ^ catalog_version, version ^ 330);
        assert!(matches!(
            Engine::restore_with_extensions(&snap, &[&pack]),
            Err(SnapshotError::RuleVersionMismatch { snapshot, catalog })
                if snapshot == snap.ruleset_version && catalog == catalog_version
        ));
    }
}
