use super::*;

/// 筑城双技（U07 技能批三，规则 v113）：刘馥——①阶段结束确认开关：
/// 无标记→牌堆顶取 max(1,maxHp-体力) 张存标记；有标记→收回入手。②守
/// 卫（zhucheng2 显式化）：成为杀目标且标记非空→使用者选择：弃 n 张
/// （首张自选+余随机代抽，登记差异）或杀无效（跳过结算+杀回手）。
#[test]
fn liufu_zhucheng_filter_recheck_and_sha_guard() {
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
            submit(
                &mut e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: vec![ChoiceOption::Confirm],
                },
            );
            break;
        }
        let actor = e.turn().current;
        let _ = submit(&mut e, actor, GameAction::NextPhase);
    }
    assert_eq!(e.state_core().player(0).zhucheng.len(), 2, "筑城标记 2 张");

    // ② 他人（1 号回合）杀 0 号：手/装不足 num=2 时直接取消，杀进弃牌堆不回手；
    // 足够 num=2 时弹出 wudian_zhucheng_guard（允许取消，杀同样进弃牌堆不回手）
    advance_loop(&mut e, 1);
    e.debug_clear_hand(1);
    e.debug_grant(1, "咆哮"); // 多次出杀驱动
    let sha_less = e.debug_place(1, "sha");
    let out = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha_less,
            targets: vec![0],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    let confirm_less = e.open_choice().expect("zhucheng2 trigger_confirm");
    assert_eq!(confirm_less.spec.prompt, "trigger_confirm");
    assert_eq!(confirm_less.observer, 0);
    let cid_less = confirm_less.id;
    submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: cid_less,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    assert!(e.open_choice().is_none(), "确认发动筑城且 he 不足筑城张数 2 时直接取消不弹窗");
    assert_eq!(e.debug_hp(0), 2, "杀无效不掉血");
    assert!(!e.debug_hand(1).contains(&sha_less), "杀取消不得退回使用者手牌");
    assert!(e.save().core.discard.contains(&sha_less), "被取消的杀应进弃牌堆");

    // 足够 num=2 张时，主动取消（空选）同样令杀无效且入弃牌堆不回手
    let sha_cancel = e.debug_place(1, "sha");
    let wz_keep = e.debug_place(1, "wuzhong");
    let dl_keep = e.debug_place(1, "duel");
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha_cancel,
            targets: vec![0],
        },
    );
    let confirm_ask = e.open_choice().expect("zhucheng2 trigger_confirm②");
    assert_eq!(confirm_ask.spec.prompt, "trigger_confirm");
    let cid_ask = confirm_ask.id;
    submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: cid_ask,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    let c = e.open_choice().expect("筑城守卫询问挂起");
    assert_eq!(
        (c.spec.prompt, c.observer, c.spec.min, c.spec.max, c.spec.allow_cancel),
        (
            engine_native::content::host::WUDIAN_ZHUCHENG_GUARD_ASK,
            1,
            2,
            2,
            true,
        )
    );
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![],
        },
    );
    assert_eq!(e.debug_hp(0), 2, "取消筑城弃牌时杀无效不掉血");
    assert!(!e.debug_hand(1).contains(&sha_cancel), "取消后原杀不得退回手牌");
    assert!(e.save().core.discard.contains(&sha_cancel));
    assert!(e.debug_hand(1).contains(&wz_keep) && e.debug_hand(1).contains(&dl_keep));

    // ③ 再杀 → 使用者自选任意 2 张弃置（无随机补弃）→ 杀照常结算
    let sha = e.debug_place(1, "sha");
    let keep_extra = e.debug_place(1, "tao");
    let out = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha,
            targets: vec![0],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    let confirm_ask2 = e.open_choice().expect("zhucheng2 trigger_confirm③");
    assert_eq!(confirm_ask2.spec.prompt, "trigger_confirm");
    let cid_ask2 = confirm_ask2.id;
    submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: cid_ask2,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    let c = e.open_choice().expect("筑城守卫询问挂起②");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(wz_keep), ChoiceOption::Card(dl_keep)],
        },
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
    assert_eq!(e.debug_hp(0), 1, "弃 2 放行：杀照常（2→1）");
    assert_eq!(
        e.debug_hand(1),
        vec![keep_extra],
        "仅自选弃置 [wz_keep, dl_keep]，keep_extra 保留且无随机补弃"
    );

    // ④ 开关收回：0 号回合结束阶段确认 → 标记收回入手
    let round0 = e.turn().round;
    for _ in 0..48 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play && e.turn().round > round0 {
            break;
        }
        if let Some(c) = e.open_choice() {
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
        let actor = e.turn().current;
        let _ = submit(&mut e, actor, GameAction::NextPhase);
    }
    e.debug_set_hp(0, 2);
    let marks = e.state_core().player(0).zhucheng.clone();
    // C331: the actual filter requires an empty mark pile; an inline gate cannot bypass it.
    for _ in 0..8 {
        if let Some(c) = e.open_choice().cloned() {
            assert_ne!(c.spec.prompt, "trigger_confirm", "ineligible zhucheng must not ask");
            let selection = c.spec.options.iter().take(c.spec.min).copied().collect();
            submit(&mut e, c.observer, GameAction::Answer { choice_id: c.id, selection });
        } else {
            let actor = e.turn().current;
            let _ = submit(&mut e, actor, GameAction::NextPhase);
        }
    }
    assert_eq!(e.state_core().player(0).zhucheng, marks, "failed current filter preserves the marks");
    let h0 = e.debug_hand(0);
    assert!(marks.iter().all(|card| !h0.contains(card)), "confirmation cannot recover marks through a false filter");
}
