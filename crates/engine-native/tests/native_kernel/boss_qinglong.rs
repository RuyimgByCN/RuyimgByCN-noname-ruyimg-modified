use super::*;

/// 四象系续片（规则 v234）：雷击（青龙 releiji 界雷击）——打出闪后座
/// 位序下一名其他存活角色判定：黑桃 2 雷伤、梅花持有者回 1+其 1 雷伤、
/// 其他无事；万箭闪响应同触发；非持有者对照不触发。目标=座位序首名
/// 确定性近似（登记）。青木2（boss_qingmu2）=关卡流程面登记不落内核。
#[test]
fn boss_qinglong_releiji_thunder_judge() {
    // ① 梅花判定：青龙回 1、攻击者受 1 雷伤
    let mut e = Engine::new_boss_stage(54001, 4, 1, 3, 6, 4, 0);
    let qinglong_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 10) as u16; // BOSS_GENERALS[10] 青龙
    e.debug_set_general_def(1, qinglong_def);
    e.debug_set_hp(1, 4);
    let rdef = e.catalog().skill_by_name("雷击").expect("雷击入目录");
    let sdef = e.catalog().skill_by_name("神裔").expect("神裔入目录");
    assert!(e.state_core().skills.contains_key(&(1, rdef)));
    assert!(e.state_core().skills.contains_key(&(1, sdef)));
    let run_club = |seed: u64| {
        let mut e = Engine::new_boss_stage(seed, 4, 1, 3, 6, 4, 0);
        e.debug_set_general_def(1, qinglong_def);
        e.debug_set_hp(1, 3);
        advance_to_play(&mut e, 2);
        e.debug_clear_hand(2);
        e.debug_clear_hand(1);
        let sha = e.debug_place(2, "sha");
        let shan = e.debug_place(1, "shan");
        play(&mut e, sha, vec![1]);
        let c = e.open_choice().expect("闪询问挂起");
        assert_eq!(c.observer, 1);
        e.debug_set_deck_top_by(|def| def.suit == engine_native::catalog::Suit::Club);
        answer(&mut e, ChoiceOption::Card(shan));
        drain_choices(&mut e);
        e
    };
    let e = run_club(54001);
    assert_eq!(e.debug_hp(1), 4, "梅花：雷击持有者回复 1");
    assert_eq!(
        e.debug_hp(2),
        3,
        "梅花：座位序下一名（2 号=攻击者）受 1 雷伤"
    );

    // ② 黑桃判定：目标受 2 点雷伤、持有者不回
    let e = {
        let mut e = Engine::new_boss_stage(54002, 4, 1, 3, 6, 4, 0);
        e.debug_set_general_def(1, qinglong_def);
        e.debug_set_hp(1, 4);
        advance_to_play(&mut e, 2);
        e.debug_clear_hand(2);
        e.debug_clear_hand(1);
        let sha = e.debug_place(2, "sha");
        let shan = e.debug_place(1, "shan");
        play(&mut e, sha, vec![1]);
        let c = e.open_choice().expect("闪询问挂起");
        assert_eq!(c.observer, 1);
        e.debug_set_deck_top_by(|def| def.suit == engine_native::catalog::Suit::Spade);
        answer(&mut e, ChoiceOption::Card(shan));
        drain_choices(&mut e);
        e
    };
    assert_eq!(e.debug_hp(1), 4, "黑桃：持有者不回复");
    assert_eq!(e.debug_hp(2), 2, "黑桃：目标受 2 点雷伤");

    // ③ 红桃判定：无事
    let e = {
        let mut e = Engine::new_boss_stage(54003, 4, 1, 3, 6, 4, 0);
        e.debug_set_general_def(1, qinglong_def);
        e.debug_set_hp(1, 4);
        advance_to_play(&mut e, 2);
        e.debug_clear_hand(2);
        e.debug_clear_hand(1);
        let sha = e.debug_place(2, "sha");
        let shan = e.debug_place(1, "shan");
        play(&mut e, sha, vec![1]);
        let c = e.open_choice().expect("闪询问挂起");
        assert_eq!(c.observer, 1);
        e.debug_set_deck_top_by(|def| def.suit == engine_native::catalog::Suit::Heart);
        answer(&mut e, ChoiceOption::Card(shan));
        drain_choices(&mut e);
        e
    };
    assert_eq!(e.debug_hp(1), 4, "红桃：无事");
    assert_eq!(e.debug_hp(2), 4, "红桃：无事");

    // ④ 对照（非持有者响应闪）：不触发雷击
    let mut e = Engine::new(54004, 4);
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    let sha = e.debug_place(0, "sha");
    let shan = e.debug_place(1, "shan");
    play(&mut e, sha, vec![1]);
    let _c = e.open_choice().expect("闪询问挂起");
    e.debug_set_deck_top_by(|def| def.suit == engine_native::catalog::Suit::Spade);
    answer(&mut e, ChoiceOption::Card(shan));
    drain_choices(&mut e);
    assert_eq!(e.debug_hp(2), 4, "非持有者：无雷击");
    assert_eq!(e.debug_hp(1), 4, "闪抵消杀伤害");

    // ⑤ 万箭闪响应同触发：青龙闪万箭 → 梅花判定 → 2 号雷伤 1+万箭伤 1
    let mut e = Engine::new_boss_stage(54005, 4, 1, 3, 6, 4, 0);
    e.debug_set_general_def(1, qinglong_def);
    e.debug_set_hp(1, 3);
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    e.debug_clear_hand(3);
    e.debug_set_hp(2, 4);
    e.debug_set_hp(3, 4);
    let wj = e.debug_place(0, "archery");
    let shan = e.debug_place(1, "shan");
    play(&mut e, wj, vec![]);
    let c = e.open_choice().expect("万箭闪询问挂起");
    assert_eq!(c.observer, 1);
    e.debug_set_deck_top_by(|def| def.suit == engine_native::catalog::Suit::Club);
    answer(&mut e, ChoiceOption::Card(shan));
    drain_choices(&mut e);
    assert_eq!(e.debug_hp(1), 4, "万箭闪免伤+梅花回复 1");
    assert_eq!(e.debug_hp(2), 2, "2 号：雷击雷伤 1+万箭 1（无闪受 1）");
    assert_eq!(e.debug_hp(3), 3, "3 号：万箭 1（无闪）");
    assert_zone_conservation(&e);

    // ⑥ 存档往返
    let e = run_club(54006);
    let snap = e.save();
    let r = Engine::restore(&snap).expect("往返");
    assert!(
        r.state_core().skills.contains_key(&(1, rdef)),
        "雷击实例往返"
    );
    assert_zone_conservation(&r);
}

/// 四象系首片（规则 v233）：神裔（青龙共有技）两面——①判定阶段延时
/// 锦囊恒不生效（乐不跳过出牌、闪电恒不命中传下家；非持有者对照照常
/// 生效）；②武将牌不能翻至背面（背→正照常翻回）。青龙条目（群/hp4/
/// [神裔]；releiji 雷击/boss_qingmu2 青木未落随下批）。
#[test]
fn boss_qinglong_shenyi_judge_void_and_never_flipped() {
    // ① 乐：黑桃判定本应跳过出牌 → 神裔使 1 号出牌阶段可达
    let mut e = Engine::new_boss_stage(53001, 4, 1, 3, 6, 4, 0);
    let qinglong_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 10) as u16; // BOSS_GENERALS[10] 青龙
    e.debug_set_general_def(1, qinglong_def);
    e.debug_set_hp(1, 4);
    assert_eq!(
        e.catalog().general_meta(qinglong_def).map(|g| g.0),
        Some("青龙"),
        "青龙入目录"
    );
    let sdef = e.catalog().skill_by_name("神裔").expect("神裔入目录");
    assert!(
        e.state_core().skills.contains_key(&(1, sdef)),
        "共有技装配即启用"
    );
    e.debug_place_judge(1, "le");
    let mut seen_1_play = false;
    for _ in 0..60 {
        let cur = e.turn().current;
        let ph = e.turn().phase;
        if cur == 1 && ph == Phase::Play {
            seen_1_play = true;
            break;
        }
        assert!(
            !(cur == 1 && ph == Phase::Discard),
            "神裔：黑桃判定乐不得跳过 1 号出牌阶段"
        );
        if cur == 0 && ph == Phase::End {
            // 1 的上家 End 压顶：判定牌=黑桃（本应使乐生效）
            e.debug_set_deck_top_by(|def| def.suit.is_spade() && def.kind == CardKind::Sha);
        }
        match next_phase(&mut e) {
            CommandOutcome::Accepted { .. } => {}
            CommandOutcome::Pending { choice } => answer_cards(&mut e, &choice),
            other => panic!("{other:?}"),
        }
    }
    assert!(seen_1_play, "神裔：出牌阶段可达");
    assert!(e.debug_judge(1).is_empty(), "乐已结算弃置");

    // ② 闪电：黑桃 2–9 判定本应 3 点雷伤 → 恒不命中、传 2 号判定区
    e.debug_place_judge(1, "shandian");
    for _ in 0..60 {
        if !e.debug_judge(2).is_empty() {
            break;
        }
        if e.turn().current == 0 && e.turn().phase == Phase::End {
            e.debug_set_deck_top_by(|def| def.suit.is_spade() && (2..=9).contains(&def.rank));
        }
        match next_phase(&mut e) {
            CommandOutcome::Accepted { .. } => {}
            CommandOutcome::Pending { choice } => answer_cards(&mut e, &choice),
            other => panic!("{other:?}"),
        }
    }
    assert!(
        !e.debug_judge(2).is_empty(),
        "神裔：闪电不命中 → 传 2 号判定区"
    );
    assert_eq!(e.debug_hp(1), 4, "神裔：闪电恒不命中");
    assert!(e.debug_judge(1).is_empty());

    // ③ 对照（非持有者）：标准局黑桃判定 → 乐照常跳过出牌
    let mut e2 = Engine::new(53100, 4);
    advance_to_play(&mut e2, 0);
    for p in 0..4 {
        e2.debug_clear_hand(p);
    }
    e2.debug_place_judge(1, "le");
    e2.debug_set_deck_top_by(|def| def.suit.is_spade() && def.kind == CardKind::Sha);
    // 推进到 1 号回合：乐生效则首回合不经过 Play 直达 Discard
    let mut seen_play = false;
    let mut reached_discard = false;
    for _ in 0..10 {
        let cur = e2.turn().current;
        let ph = e2.turn().phase;
        if cur == 1 && ph == Phase::Play {
            seen_play = true;
            break;
        }
        if cur == 1 && ph == Phase::Discard {
            reached_discard = true;
            break;
        }
        match next_phase(&mut e2) {
            CommandOutcome::Accepted { .. } => {}
            CommandOutcome::Pending { choice } => answer_cards(&mut e2, &choice),
            other => panic!("{other:?}"),
        }
    }
    assert!(
        reached_discard && !seen_play,
        "对照：非持有者黑桃判定 → 乐照常生效（跳过出牌直达弃牌）"
    );

    // ④ 神裔：不能翻至背面；背→正照常翻回
    assert!(!e.state_core().player(1).face_down);
    e.debug_flip(1);
    assert!(!e.state_core().player(1).face_down, "神裔：正→背取消");
    e.state_core_mut().player_mut(1).face_down = true;
    e.debug_flip(1);
    assert!(!e.state_core().player(1).face_down, "背→正照常翻回");

    // ⑤ 存档往返：神裔实例/翻面态/判定区一致
    let snap = e.save();
    let r = Engine::restore(&snap).expect("往返");
    assert!(
        r.state_core().skills.contains_key(&(1, sdef)),
        "神裔实例往返"
    );
    assert!(!r.state_core().player(1).face_down, "翻面态往返");
    assert_zone_conservation(&r);
    assert_zone_conservation(&e);
}
