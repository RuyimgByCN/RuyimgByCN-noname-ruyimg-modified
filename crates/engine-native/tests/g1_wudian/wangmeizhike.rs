use super::*;

/// 分支①：目标为最小体力（已受伤）→ 回复 1；非最小手牌 → 不摸牌。
#[test]
fn wangmeizhike_recovers_min_hp_damaged_target() {
    let (mut e, pack) = setup();
    let wm = wangmei_id(&e, 0);
    e.debug_place(0, "shan");
    e.debug_place(0, "shan");
    e.debug_place(1, "nullify"); // 触发无懈窗口
    e.debug_place(2, "shan");
    let mut snap = e.save();
    snap.core.players[0].hp = 1; // 最小体力且已受伤
    snap.core.players[1].hp = 4;
    snap.core.players[2].hp = 4;
    snap.core.deck.retain(|c| *c != wm);
    snap.core.players[0].hand.push(wm);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let hand_before = e.debug_hand(0).len();
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: wm,
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
    pass_nullify(&mut e);
    assert_eq!(e.debug_hp(0), 2, "最小体力且已受伤 → 回复 1");
    assert_eq!(
        e.debug_hand(0).len(),
        hand_before - 1,
        "非最小手牌不摸牌（仅打出的牌离手）"
    );
}

/// 分支②：目标非最小体力 → 不回复；最小手牌 → 摸 2。
#[test]
fn wangmeizhike_draws_min_hand_target() {
    let (mut e, pack) = setup();
    let wm = wangmei_id(&e, 0);
    e.debug_set_hp(0, 4);
    e.debug_set_hp(1, 1); // 体力最小者为目标之外
    e.debug_set_hp(2, 4);
    e.debug_place(1, "nullify"); // 触发无懈窗口
    let mut snap = e.save();
    snap.core.deck.retain(|c| *c != wm);
    snap.core.players[0].hand.push(wm);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: wm,
            targets: vec![0],
        },
    );
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_nullify(&mut e);
    assert_eq!(e.debug_hp(0), 4, "非最小体力不回复");
    assert_eq!(
        e.debug_hand(0).len(),
        2,
        "打出 1 张后为最小手牌 → 摸 2（打出的不计）"
    );
}
