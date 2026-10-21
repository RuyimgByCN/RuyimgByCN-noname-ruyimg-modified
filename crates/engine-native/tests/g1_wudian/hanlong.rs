use super::*;

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
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
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
    assert_eq!(e.debug_hp(1), 2, "非满血不加成（3→2）");
    // 技能仍在
    let def = e
        .catalog()
        .skill_by_name(engine_native::content::host::WUDIAN_CIQIU_KEY)
        .expect("def");
    assert!(e.state_core().has_skill(0, def), "技能未被移除");

    // ② 满血目标（回满 4）：+1 → 3 伤（2→? 2-3 → 濒死）——伤 3 ≥ 体力 2？
    // 上游口径：加后 num(3) >= player.hp(目标受伤害前体力 2) → 濒死即死
    e.debug_set_hp(1, 2);
    // 注意满血判断：目标 hp==max_hp 才触发——先把 max_hp 压到 2
    e.debug_set_max_hp(1, 2);
    let sha = e.debug_place(0, "sha");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
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
    assert!(!e.debug_alive(1), "伤 3 ≥ 体力 2：濒死即死（跳过救援）");
    // 技能已移除：再对满血目标不加成
    e.debug_set_hp(2, 4);
    let sha = e.debug_place(0, "sha");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
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
    assert_eq!(e.debug_hp(2), 3, "技能移除后不加成（4→3）");
}
