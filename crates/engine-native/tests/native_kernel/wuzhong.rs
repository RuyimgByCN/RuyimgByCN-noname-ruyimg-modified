use super::*;

/// P2/N08 缺牌种批①（规则 v62，2026-10-07）：无中生有（♥7/8/9/11，
/// 使用后摸 2）/桃园结义（♥1，所有其他角色回复 1）——上游 standard
/// list 花色点数原样落位，锦囊均经无懈窗口（可被无懈）。
#[test]
fn wuzhong_taoyuan_trick_behaviors() {
    // ① 无中生有：使用后摸 2，牌入弃牌堆。
    let mut e = Engine::new(120, 3);
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let wz = e.debug_place(0, "wuzhong");
    let hand0 = e.debug_hand(0).len();
    let out = submit_cmd(
        &mut e,
        0,
        GameAction::PlayCard {
            card: wz,
            targets: vec![],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    // 无懈窗口排空（放弃）。
    for _ in 0..8 {
        match e.open_choice() {
            Some(c) if c.spec.prompt == "nullify_window" => {
                let (id2, obs2) = (c.id, c.observer);
                submit_cmd(
                    &mut e,
                    obs2,
                    GameAction::Answer {
                        choice_id: id2,
                        selection: vec![ChoiceOption::Pass],
                    },
                );
            }
            _ => break,
        }
    }
    assert_eq!(
        e.debug_hand(0).len(),
        hand0 - 1 + 2,
        "无中生有：摸 2（弃牌+摸2）"
    );
    assert!(!e.save().core.table.contains(&wz), "无中生有离场");

    // ② 桃园结义：所有其他受伤角色回 1（含满血者跳过），使用者不变。
    let mut e = Engine::new(121, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(1, 2); // 受伤 → 回 1 → 3
    e.debug_set_hp(2, 4); // 满血 → 不变
    e.debug_set_hp(3, 3); // 受伤 → 回 1 → 4
    let ty = e.debug_place(0, "taoyuan");
    let out = submit_cmd(
        &mut e,
        0,
        GameAction::PlayCard {
            card: ty,
            targets: vec![],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    for _ in 0..8 {
        match e.open_choice() {
            Some(c) if c.spec.prompt == "nullify_window" => {
                let (id2, obs2) = (c.id, c.observer);
                submit_cmd(
                    &mut e,
                    obs2,
                    GameAction::Answer {
                        choice_id: id2,
                        selection: vec![ChoiceOption::Pass],
                    },
                );
            }
            _ => break,
        }
    }
    assert_eq!(e.debug_hp(1), 3, "桃园：1 号回 1");
    assert_eq!(e.debug_hp(2), 4, "桃园：满血 2 号不变");
    assert_eq!(e.debug_hp(3), 4, "桃园：3 号回 1");
    assert_eq!(e.debug_hp(0), 4, "桃园：使用者不在回复范围");
    assert!(!e.save().core.table.contains(&ty), "桃园离场");
}
