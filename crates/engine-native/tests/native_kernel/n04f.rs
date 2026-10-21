use super::*;

/// P1/N04-f（ADR 0768）：快照关联校验——将槽越界拒绝、明置无将拒绝、
/// 牌重复归属拒绝；合法快照（含观星暂离态）不误拒。
#[test]
fn n04f_snapshot_association_validation() {
    // (1) 将槽 65535 + 明置 → 拒绝（原探针缺陷）
    let e = Engine::new(443, 4);
    let base = e.save();
    let mut bad = base.clone();
    bad.core.players[0].general_slots = [Some(u16::MAX), None];
    bad.core.players[0].revealed_generals = [true, false];
    assert!(Engine::restore(&bad).is_err(), "越界将槽 + 明置必须被拒绝");

    // (2) 手牌同时追加进弃牌堆 → 拒绝（牌重复归属）
    let mut bad2 = base.clone();
    let duplicate = bad2.core.players[0].hand[0];
    bad2.core.discard.push(duplicate);
    assert!(
        Engine::restore(&bad2).is_err(),
        "同一张牌在两个牌区必须被拒绝"
    );

    // (3) 明置引用空槽 → 拒绝
    let mut bad3 = base.clone();
    bad3.core.players[1].revealed_generals = [true, true];
    assert!(Engine::restore(&bad3).is_err(), "空槽明置必须被拒绝");

    // (4) 合法快照（含进行中的观星暂离态）不误拒：
    //     构造观星 Arrange 挂起 + 顶牌在机器内的存档照常恢复
    let mut g = Engine::new(901, 3);
    g.debug_grant(0, "观星");
    for p in 0..3 {
        g.debug_clear_hand(p);
    }
    let mut saved_mid: Option<engine_native::snap::StateSnapshot> = None;
    for _ in 0..128 {
        let t = g.turn();
        if t.current == 0 && t.phase == Phase::Judge {
            break;
        }
        match next_phase(&mut g) {
            CommandOutcome::Accepted { .. } => {}
            CommandOutcome::Pending { choice } => match choice.spec.prompt {
                "trigger_confirm" => {
                    answer(&mut g, ChoiceOption::Confirm);
                    if let Some(a) = g.open_choice() {
                        if a.spec.prompt == "guanxing_arrange" {
                            saved_mid = Some(g.save());
                            break;
                        }
                    }
                }
                other => panic!("unexpected {other}"),
            },
            o => panic!("walk: {o:?}"),
        }
    }
    if let Some(snap) = saved_mid {
        assert!(
            Engine::restore(&snap).is_ok(),
            "观星暂离态（顶牌在机器内）的合法快照不得被误拒"
        );
    }
}
