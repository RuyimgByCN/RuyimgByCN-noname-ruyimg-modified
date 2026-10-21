use super::*;

/// T12 swd deferred 消解批 113：灵矢（计亭 `swd_jiting`）——装备区每有一张
/// 牌攻击范围 +2；有武器或防具时【杀】不可闪避；有马时摸牌阶段额外摸一
/// 张；有宝物时回合内可额外使用一张【杀】。四臂正例 + 反例。
#[test]
fn swd_deferred_lingshi_range_direct_hit_draw_and_sha_limit() {
    let root = repo_extensions_root();
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
    let mut e = Engine::new_with_extensions(63113, 4, &[&pack]).expect("建局");
    e.debug_grant(0, "lingshi");
    advance_to_play(&mut e, 0);

    // ① 攻击范围：无装备时 0→2 座位距 2 > 攻击范围 1 → 杀 2 号非法。
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sha_a = e.debug_place(0, "sha");
    let out = play(&mut e, sha_a, vec![2]);
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::IllegalTargets)),
        "灵矢反例：无装备时座位距 2 超攻击范围（{out:?}）"
    );

    // ② 挂 1 件无射程装备（马）：攻击范围 +2 → 距离 2-2=1（下限）→ 杀 2
    // 号合法（无灵矢时距离 2 > 攻击范围 1，见①）。
    inject_equip_swd(&mut e, &pack, 0, CardKind::HorsePlus);
    e.debug_clear_hand(0);
    let sha_b = e.debug_place(0, "sha");
    let out = play(&mut e, sha_b, vec![2]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "灵矢：1 件装备 → 攻击范围 +2 覆盖座位距 2（{out:?}）"
    );
    // 收束本杀（目标放弃闪/无闪）。
    for _ in 0..8 {
        let Some(c) = e.open_choice() else { break };
        let (cid, obs) = (c.id, c.observer);
        let _ = submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Pass],
            },
        );
    }

    // ③ 有武器时【杀】不可闪避：1 号持闪仍照常受伤、闪未被打出。
    inject_equip_swd(&mut e, &pack, 0, CardKind::Chilongya);
    {
        let mut snap = e.save();
        snap.core.turn.sha_used_play_phase = false;
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("重置杀计数");
    }
    e.debug_clear_hand(1);
    let shan = e.debug_place(1, "shan");
    let shan_before = e.debug_hand(1).len();
    e.debug_place(0, "sha");
    let sha_c = e.debug_hand(0)[0];
    let out = play(&mut e, sha_c, vec![1]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    let mut saw_shan_ask = false;
    for _ in 0..8 {
        let Some(c) = e.open_choice() else { break };
        if c.spec.prompt == "respond_shan" {
            saw_shan_ask = true;
        }
        let (cid, obs) = (c.id, c.observer);
        let _ = submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Pass],
            },
        );
    }
    assert!(!saw_shan_ask, "灵矢：有武器 → 不出闪询问（直中）");
    assert_eq!(e.debug_hp(1), 3, "灵矢：直中 1 号（4→3）");
    assert_eq!(e.debug_hand(1).len(), shan_before, "灵矢：闪未被消耗");
    assert!(e.debug_hand(1).contains(&shan), "灵矢：闪仍在手");

    // ④ 有马时摸牌阶段额外摸一张（0 号判定阶段 → 摸牌阶段增量 3）。
    e.debug_set_turn(0, Phase::Judge, e.turn().round);
    let h_judge = e.debug_hand(0).len();
    let out = submit(&mut e, 0, GameAction::NextPhase);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert_eq!(
        e.debug_hand(0).len(),
        h_judge + 3,
        "灵矢：有马 → 摸牌阶段 2+1"
    );

    // ⑤ 有宝物时回合内可额外使用第二张杀（每回合一次额度）。
    inject_equip_swd(&mut e, &pack, 0, CardKind::Qinglonglingzhu);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let first = e.debug_place(0, "sha");
    let out = play(&mut e, first, vec![1]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "第一张杀 {out:?}"
    );
    for _ in 0..8 {
        let Some(c) = e.open_choice() else { break };
        let (cid, obs) = (c.id, c.observer);
        let _ = submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Pass],
            },
        );
    }
    let second = e.debug_place(0, "sha");
    let out = play(&mut e, second, vec![1]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "灵矢：宝物 → 第二张杀可用（{out:?}）"
    );
    for _ in 0..8 {
        let Some(c) = e.open_choice() else { break };
        let (cid, obs) = (c.id, c.observer);
        let _ = submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Pass],
            },
        );
    }
    // 反例：额度已用 → 第三张杀超限。
    let third = e.debug_place(0, "sha");
    let out = play(&mut e, third, vec![1]);
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::ShaLimit)),
        "灵矢：额度用尽 → 第三张杀被拒（{out:?}）"
    );
}
