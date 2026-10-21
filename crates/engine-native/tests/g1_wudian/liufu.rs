use super::*;

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

    // ② 他人（1 号回合）杀 0 号 → 使用者被问：Pass=杀无效+杀回手
    advance_loop(&mut e, 1);
    e.debug_grant(1, "咆哮"); // 两次出杀驱动
    let sha = e.debug_place(1, "sha");
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
    let c = e.open_choice().expect("筑城守卫询问挂起");
    assert_eq!(
        (c.spec.prompt, c.observer),
        (engine_native::content::host::WUDIAN_ZHUCHENG_GUARD_ASK, 1)
    );
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Pass],
        },
    );
    assert_eq!(e.debug_hp(0), 2, "杀无效不掉血");
    assert!(e.debug_hand(1).contains(&sha), "杀归还使用者手牌");

    // ③ 再杀 → 使用者弃 2 张（首张自选+余代抽）→ 杀照常结算
    let sha = e.debug_place(1, "sha");
    let _ = e.debug_place(1, "wuzhong");
    let _ = e.debug_place(1, "duel");
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
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![pick],
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
    for _ in 0..8 {
        if let Some(c) = e.open_choice() {
            assert_eq!(c.spec.prompt, "trigger_confirm", "收回确认挂起");
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
    assert!(e.state_core().player(0).zhucheng.is_empty(), "标记收回清空");
    let h0 = e.debug_hand(0);
    assert!(
        h0.contains(&37) && h0.contains(&5),
        "收回的 2 张标记入手（37/5）"
    );
}
