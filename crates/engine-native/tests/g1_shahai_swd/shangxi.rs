use super::*;

/// T12 swd 武将面批㉟（凤钰袭西/轩辕剑仙花间/慕月星坠；fzhenwei/pozhou/
/// xuanyuan/lingxian/shouyin deferred）：三将行为面 + 反例。
#[test]
fn swd_generals_b75_shangxi_huajian_xingzhui() {
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
    let mut e = Engine::new_with_extensions(62043, 4, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    // ① 袭西（凤钰）：出牌阶段开始弃一张牌 → 距离内体力≥自己者受 1
    //（进入 Play 的帧窗口——手动推进到 Draw 停，单步 NextPhase 入 Play
    // 后立即取窗，防 advance_to_play 自动应答吞窗）
    e.debug_grant(0, "shangxi");
    submit(&mut e, 0, GameAction::NextPhase);
    submit(&mut e, 0, GameAction::NextPhase);
    submit(&mut e, 0, GameAction::NextPhase);
    for _ in 0..16 {
        let c = e.turn().current;
        let _ = submit(&mut e, c, GameAction::NextPhase);
    }
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        if p != 0 {
            e.debug_clear_hand(p);
        }
    }
    e.debug_set_hp(0, 2); // 自身 hp 2：候选须 hp≥2（p1 hp4 ✓ p3 hp4 ✓）
    let d1 = e.debug_place(0, "tao");
    e.debug_set_hp(1, 4);
    e.debug_set_hp(3, 4);
    // 推进到 p0 的 Draw 停（下一 NextPhase 即入 Play 触发袭西窗）
    submit(&mut e, 0, GameAction::NextPhase);
    submit(&mut e, 0, GameAction::NextPhase);
    submit(&mut e, 0, GameAction::NextPhase);
    for _ in 0..16 {
        let c = e.turn().current;
        if c == 0 && e.state_core().turn.phase == engine_native::event::Phase::Judge {
            break;
        }
        let _ = submit(&mut e, c, GameAction::NextPhase);
    }
    submit(&mut e, 0, GameAction::NextPhase); // Judge→Draw（摸牌自动结算）
    submit(&mut e, 0, GameAction::NextPhase); // Draw→Play：袭西窗开
                                              // 弃牌窗 → 目标窗
    let c = e.open_choice().expect("袭西弃牌窗");
    assert_eq!(c.spec.prompt, "shangxi_discard");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(d1)],
        },
    );
    let c = e.open_choice().expect("袭西目标窗");
    assert_eq!(c.spec.prompt, "shangxi_target");
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
    assert_eq!(e.debug_hp(1), 3, "袭西：p1 受 1（4→3）");
    assert_eq!(e.debug_hp(3), 4, "袭西：p3 未选不变");

    // ② 花间（轩辕剑仙）：出牌阶段结束弃一张牌 → 虚拟杀
    e.debug_grant(0, "huajian");
    next_p0_play_fn(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(1, 4);
    let d2 = e.debug_place(0, "tao");
    // 推进到出牌阶段结束（Play→Discard 的 Play AfterTriggers 窗）
    submit(&mut e, 0, GameAction::NextPhase);
    let c = e.open_choice().expect("花间弃牌窗");
    assert_eq!(c.spec.prompt, "huajian_discard");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(d2)],
        },
    );
    let c = e.open_choice().expect("花间目标窗");
    assert_eq!(c.spec.prompt, "huajian_target");
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
    assert_eq!(e.debug_hp(1), 3, "花间：虚拟杀 1 伤（4→3）");

    // ③ 星坠（慕月）：弃一张牌 → 目标弃同类别或受 1
    e.debug_grant(0, "xingzhui");
    next_p0_play_fn(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let anchor_tao = e.debug_place(0, "tao"); // 基本牌锚
    let vic_tao = e.debug_place(1, "tao");
    let vic_shan = e.debug_place(1, "shan");
    e.debug_set_hp(1, 4);
    let def_xz = e.catalog().skill_by_name("xingzhui").expect("星坠在目录");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_xz });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "星坠发动 {out:?}"
    );
    let c = e.open_choice().expect("星坠弃牌窗");
    assert_eq!(c.spec.prompt, "xingzhui_discard");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(anchor_tao)],
        },
    );
    let c = e.open_choice().expect("星坠目标窗");
    assert_eq!(c.spec.prompt, "xingzhui_target");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    let c = e.open_choice().expect("星坠同类别窗");
    assert_eq!(c.spec.prompt, "xingzhui_same_type");
    assert_eq!(c.observer, 1);
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(vic_tao)],
        },
    );
    settle_optional_confirm(&mut e);
    assert!(!e.debug_hand(1).contains(&vic_tao), "星坠：目标弃同类别牌");
    assert!(e.debug_hand(1).contains(&vic_shan), "星坠：其余手牌保留");
}
