use super::*;

// ---------------------------------------------------------------------------
// 装备栏与距离/攻击范围修饰（阶段 2 扩面第一项）
// ---------------------------------------------------------------------------

/// ±1 马与武器射程：距离修饰、射程覆盖、同栏替换。
#[test]
fn equip_horses_weapon_range_and_replacement() {
    let mut e = Engine::new(101, 4);
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(2);
    let sha = e.debug_place(0, "sha");

    // (1) 基础：0→2 座距 2 > 射程 1 → 非法
    match play(&mut e, sha, vec![2]) {
        CommandOutcome::Rejected(RejectReason::IllegalTargets) => {}
        other => panic!("{other:?}"),
    }

    // (2) -1 马：距离 2-1=1 → 合法并命中（2 已清手，无闪）
    let hm = e.debug_place(0, "horse_minus");
    assert!(matches!(
        play(&mut e, hm, vec![]),
        CommandOutcome::Accepted { .. }
    ));
    assert!(matches!(
        play(&mut e, sha, vec![2]),
        CommandOutcome::Accepted { .. }
    ));
    assert_eq!(hp(&e, 2), 3);

    // (3) 武器：射程 2 → 再装后即便 2 号出 +1 马（距离 2+1-1=2）也能打到
    let w2 = e.debug_move_named_card_to_hand(0, "weapon2");
    assert!(matches!(
        play(&mut e, w2, vec![]),
        CommandOutcome::Accepted { .. }
    ));

    advance_to_play(&mut e, 2);
    let hp_card = e.debug_place(2, "horse_plus");
    assert!(matches!(
        play(&mut e, hp_card, vec![]),
        CommandOutcome::Accepted { .. }
    ));

    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(2);
    let sha2 = e.debug_place(0, "sha");
    // 0→2 = 2 + 1(+马) - 1(-马) = 2 ≤ 武器射程 2 → 合法
    assert!(matches!(
        play(&mut e, sha2, vec![2]),
        CommandOutcome::Accepted { .. }
    ));
    assert_eq!(hp(&e, 2), 2);

    // (4) 同栏替换：weapon3 换下 weapon2（旧牌入弃牌堆）
    e.debug_clear_hand(0);
    let w3 = e.debug_move_named_card_to_hand(0, "weapon3");
    assert!(matches!(
        play(&mut e, w3, vec![]),
        CommandOutcome::Accepted { .. }
    ));
    assert!(log_has(&e, |l| matches!(
        l,
        LogEvent::Equipped {
            card,
            replaced: Some(old),
            ..
        } if *card == w3 && *old == w2
    )));
}

/// 座位距离只计存活玩家：1 号死亡后 0→2 由 2 变 1。
#[test]
fn equip_distance_skips_dead_players() {
    let mut e = Engine::new(103, 4);
    e.debug_set_hp(1, 1);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sha = e.debug_place(0, "sha");
    // 无人持桃 → 1 号死亡
    play(&mut e, sha, vec![1]);
    assert!(!e.debug_alive(1));
    // 0→2：存活座位 {0,2,3} 相邻 → 距离 1；但杀已限一次 → 换 3 号位观察
    // 直接用距离断言（经公开查询路径的合法目标校验）
    advance_to_play(&mut e, 2);
    let sha_c = e.debug_place(2, "sha");
    // 2→0 现在也是存活相邻（距离 1）→ 合法
    assert!(matches!(
        play(&mut e, sha_c, vec![0]),
        CommandOutcome::Accepted { .. }
    ));
    assert_eq!(hp(&e, 0), 3);
}

/// 防具占栏但无保护效果（明确登记为缺失，防止隐性当成已实现）。
#[test]
fn armor_bagua_judgement_semantics() {
    // 八卦阵（ADR 0729）：受击者装备防具 → 出杀结算先做防具判定——
    // 红桃视为已出闪（伤害免除、不出闪询问），否则照常出闪询问。
    // (1) 判定红桃：伤害免除，ArmorJudged{success:true}
    let mut e = Engine::new(105, 4);
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    // 守方 1 号在自己的回合装备防具
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(1);
    let armor = e.debug_place(1, "armor");
    assert!(matches!(
        play(&mut e, armor, vec![]),
        CommandOutcome::Accepted { .. }
    ));
    advance_to_play(&mut e, 0);
    e.debug_set_deck_top_by(|def| def.suit.is_heart());
    let sha = e.debug_place(0, "sha");
    let out = play(&mut e, sha, vec![1]);
    // 判定命中：无出闪询问，杀直接结算
    assert!(
        matches!(out, CommandOutcome::Accepted { .. }),
        "红桃判定应免除伤害且无出闪询问: {out:?}"
    );
    assert_eq!(hp(&e, 1), 4, "八卦阵判定命中应免除伤害");
    assert!(log_has(&e, |l| matches!(
        l,
        LE::ArmorJudged {
            player: 1,
            success: true,
            ..
        }
    )));
    assert_eq!(discard_count(&e, sha), 1, "杀照常弃置");
    assert_zone_conservation(&e);

    // (2) 判定非红桃：照常进入出闪询问；出闪后无伤害
    let mut e2 = Engine::new(105, 4);
    advance_to_play(&mut e2, 0);
    e2.debug_clear_hand(0);
    e2.debug_clear_hand(1);
    advance_to_play(&mut e2, 1);
    e2.debug_clear_hand(1);
    let armor2 = e2.debug_place(1, "armor");
    let _ = play(&mut e2, armor2, vec![]);
    advance_to_play(&mut e2, 0);
    e2.debug_set_deck_top_by(|def| def.suit.is_spade() && !(2..=9).contains(&def.rank));
    let shan = e2.debug_place(1, "shan");
    let sha2 = e2.debug_place(0, "sha");
    let out = play(&mut e2, sha2, vec![1]);
    assert!(
        matches!(out, CommandOutcome::Pending { .. }),
        "判定未命中应照常进入出闪询问: {out:?}"
    );
    assert!(log_has(&e2, |l| matches!(
        l,
        LE::ArmorJudged {
            player: 1,
            success: false,
            ..
        }
    )));
    let c = e2.open_choice().unwrap();
    let cid = c.id;
    let out = e2.submit(GameCommand {
        actor: 1,
        expected_revision: e2.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::Answer {
            choice_id: c.id,
            selection: vec![ChoiceOption::Card(shan)],
        },
    });
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    let _ = cid;
    assert_eq!(hp(&e2, 1), 4, "判定未命中 + 出闪 → 无伤害");

    // (3) 无防具对照：不产生 ArmorJudged
    let mut e3 = Engine::new(105, 4);
    advance_to_play(&mut e3, 0);
    e3.debug_clear_hand(0);
    e3.debug_clear_hand(1);
    let sha3 = e3.debug_place(0, "sha");
    let _ = play(&mut e3, sha3, vec![1]);
    assert!(!log_has(&e3, |l| matches!(l, LE::ArmorJudged { .. })));
}
