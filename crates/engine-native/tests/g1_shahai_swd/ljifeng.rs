use super::*;

/// T12 swd 武将面批㉘（卢承轩疾风/慕容诗断义/小雪乐跃；降龙/孤兴/唤魂/
/// 待行 deferred）：三行为行为面 + 反例。
#[test]
fn swd_generals_b68_ljifeng_duanyi_yinyue() {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
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
        std::path::PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(62036, 4, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    // ① 疾风（卢承轩）：锁定——杀目标上限 +X、距离 -X（X=已损失体力）
    e.debug_grant(0, "ljifeng");
    e.debug_set_hp(0, 2); // maxHp 4 → 损失 2
    let sha1 = e.debug_place(0, "sha");
    // 距离面：p2 对座距离 2 → 疾风 -2 → 1 → 入攻击范围
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha1,
            targets: vec![2],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(2), 3, "疾风：对座 p2 入范围（距离 -2）受 1");
    // 目标面上限：1+X=3——双目标 [1,3] 合法（新回合防杀限一次）
    next_p0_play_fn(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(0, 2);
    e.debug_set_hp(1, 4);
    e.debug_set_hp(3, 4);
    let sha1b = e.debug_place(0, "sha");
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha1b,
            targets: vec![1, 3],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(1), 3, "疾风：双目标 p1 受 1");
    assert_eq!(e.debug_hp(3), 3, "疾风：双目标 p3 受 1");
    // 反例：满血（X=0）时对座出界（新回合）
    next_p0_play_fn(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(0, 4);
    let sha1c = e.debug_place(0, "sha");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha1c,
            targets: vec![2],
        },
    );
    assert!(
        !matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "疾风反例：满血对座出界拒 {out:?}"
    );

    // ② 断义（慕容诗）：弃两张杀 → 1 伤 + 目标随机弃 X=已损失 he
    e.debug_grant(0, "duanyi");
    next_p0_play_fn(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let s_a = e.debug_place(0, "sha");
    let s_b = e.debug_place(0, "sha");
    e.debug_place(1, "tao");
    e.debug_place(1, "shan");
    e.debug_place(1, "tao");
    e.debug_set_hp(1, 2); // maxHp 4 → X=2
    let def_dy = e.catalog().skill_by_name("duanyi").expect("断义在目录");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_dy });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "断义发动 {out:?}"
    );
    let c = e.open_choice().expect("断义弃牌窗");
    assert_eq!(c.spec.prompt, "duanyi_discard");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(s_a), ChoiceOption::Card(s_b)],
        },
    );
    let c = e.open_choice().expect("断义目标窗");
    assert_eq!(c.spec.prompt, "duanyi_target");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(1), 1, "断义：1 伤（2→1）");
    assert_eq!(e.debug_hand(1).len(), 1, "断义：随机弃 X=2（3→1）");
    assert!(
        !e.debug_hand(0).contains(&s_a) && !e.debug_hand(0).contains(&s_b),
        "断义：两杀已弃"
    );
    // 反例：手牌杀 ≤1 发动拒
    next_p0_play_fn(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let _one = e.debug_place(0, "sha");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_dy });
    assert!(
        !matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "断义反例：杀不足 2 拒 {out:?}"
    );

    // ③ 乐跃（小雪）：回复结算后——回复者非自己且其手牌≥自己则各摸 1
    e.debug_grant(0, "yinyue");
    next_p0_play_fn(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // p1 受伤 2/4，手牌 3；p0 手牌 3——分支 1：p1 手牌≥p0 → 各摸 1
    // （基线在推进到 p1 回合后重建）
    // 推进到 p1 回合（出杀须回合属主——桃同）
    submit(&mut e, 0, GameAction::NextPhase);
    submit(&mut e, 0, GameAction::NextPhase);
    submit(&mut e, 0, GameAction::NextPhase);
    for _ in 0..16 {
        let c = e.turn().current;
        if c == 1 && e.state_core().turn.phase == engine_native::event::Phase::Play {
            break;
        }
        let _ = submit(&mut e, c, GameAction::NextPhase);
    }
    advance_to_play(&mut e, 1);
    for p in 0..4 {
        if p != 1 {
            e.debug_clear_hand(p);
        }
    }
    // 重建双方手牌基线（清手后）：p1 3 张、p0 3 张
    e.debug_set_hp(1, 2);
    let tao1 = e.debug_place(1, "tao");
    e.debug_place(1, "shan");
    e.debug_place(1, "tao");
    e.debug_place(0, "tao");
    e.debug_place(0, "shan");
    e.debug_place(0, "tao");
    let h0 = e.debug_hand(0).len();
    let h1 = e.debug_hand(1).len();
    // p1 出桃自回（hp 2→3）→ 乐跃确认 → 各摸 1
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: tao1,
            targets: vec![],
        },
    );
    let c = e.open_choice().expect("乐跃确认窗");
    assert_eq!((c.spec.prompt, c.observer), ("trigger_confirm", 0));
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hand(0).len(), h0 + 1, "乐跃：p0 摸 1（分支 1）");
    assert_eq!(e.debug_hand(1).len(), h1, "乐跃：p1 出 1 摸 1（手牌不变）");
}
