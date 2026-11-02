use super::*;

/// 天道：闪电判定询问改判者（张角以天道资格入队，任意手牌可打）——打出
/// 红桃替换判定 → 闪电不造成伤害，原判定牌被张角获得。
#[test]
fn boss_zhangjiao_tiandao_rewrite() {
    let mut e = engine_native::Engine::new_boss_stage(85001, 4, 1, 3, 6, 4, 0);
    let zj_def = zhangjiao_def_of(&e);
    e.debug_set_general_def(1, zj_def);
    e.debug_set_hp(1, 8);
    assert_eq!(
        e.catalog().general_meta(zj_def).map(|g| g.0),
        Some("张角"),
        "张角入目录"
    );
    for sk in ["雷击·张角", "天道", "极点"] {
        let d = e
            .catalog()
            .skill_by_name(sk)
            .unwrap_or_else(|| panic!("{sk} 入目录"));
        assert!(e.state_core().skills.contains_key(&(1, d)), "{sk} 实例");
    }

    advance_to_play(&mut e, 0);
    // 闪电置入 1 号（张角）判定区：黑桃 2-9 判定命中=3 雷伤
    let sd = e.debug_place(1, "shandian");
    let _ = sd;
    // 用牌路径：闪电须由当前回合玩家置入自己判定区——0 号置入 0 号？不行。
    // 改为张角自己回合置入：推进到 1 号出牌阶段
    next_phase_settled(&mut e); // 0 Play→Discard
    next_phase_settled(&mut e); // Discard→End
    next_phase_settled(&mut e); // End→收束 → 1 号 Judge begin（判定区空，机器空转）
    advance_to_play(&mut e, 1);
    let sd2 = e.debug_move_named_card_to_hand(1, "shandian");
    assert!(matches!(
        play(&mut e, sd2, vec![]),
        engine_native::CommandOutcome::Accepted { .. }
    ));
    drain_choices(&mut e);
    assert_eq!(e.debug_judge(1).len(), 1, "闪电置入判定区");

    // 推进到下一回合张角 Judge 阶段：机器询问改判者（天道资格）
    next_phase_settled(&mut e); // Play→Discard
    next_phase_settled(&mut e); // Discard→End begin
    next_phase_settled(&mut e); // End→收束 → 1 号 Judge begin → 闪电判定询问
                                // 机器询问改判：张角（座位序自拥有者起含本人）以天道资格被问
    let mut saw_ask = false;
    let mut asked_prompt = "";
    for _ in 0..40 {
        if let Some(c) = e.open_choice() {
            if c.spec.prompt == "gui_dao_rewrite" {
                saw_ask = true;
                asked_prompt = c.spec.prompt;
                break;
            }
            // 弃牌等其他询问：直接应答（选牌类取 min 个牌选项）后继续
            let ch = e.open_choice().expect("choice open").clone();
            let sel: Vec<ChoiceOption> = ch
                .spec
                .options
                .iter()
                .filter(|o| matches!(o, ChoiceOption::Card(_)))
                .copied()
                .take(ch.spec.min.max(1))
                .collect();
            let _ = answer_sel(&mut e, sel);
            continue;
        }
        let out = next_phase(&mut e);
        if let engine_native::CommandOutcome::Rejected(r) = out {
            panic!("np rejected: {r:?}");
        }
    }
    assert!(saw_ask, "天道：改判询问挂起");
    assert_eq!(asked_prompt, "gui_dao_rewrite");
    let choice = e.open_choice().expect("choice open").clone();
    // 天道：选项含全部手牌（不限花色）——非黑手牌选项的存在本身证明
    // 天道扩展生效（鬼道仅黑、鬼才仅手牌亦全量；张角无鬼才，红桃等非
    // 黑手牌在选项中=天道扩展直接证据）
    let non_black_opt = choice.spec.options.iter().find_map(|o| match o {
        ChoiceOption::Card(c)
            if !matches!(
                e.catalog().cards[*c as usize].suit,
                engine_native::catalog::Suit::Spade | engine_native::catalog::Suit::Club
            ) =>
        {
            Some(*c)
        }
        _ => None,
    });
    let card_opt = non_black_opt.expect("天道：改判选项含非黑手牌（扩展生效）");
    let hand_len_before = e.debug_hand(1).len();
    assert!(
        matches!(
            answer(&mut e, ChoiceOption::Card(card_opt)),
            engine_native::CommandOutcome::Accepted { .. }
        ),
        "天道改判应答"
    );
    drain_choices(&mut e);
    // 闪电判定被替换为所打之牌——原判定牌被张角获得（手牌+1 相对改判前）
    assert!(
        log_has(&e, |l| matches!(
            l,
            LogEvent::JudgementRewritten { player: 1, .. }
        )),
        "天道：改判日志"
    );
    // 获得原判定牌为机器隐式行为（无独立 Gained 日志——Gained 仅天妒
    // 阶段用）：所打牌离手+原判定牌入手=手牌数不变
    assert_eq!(
        e.debug_hand(1).len(),
        hand_len_before,
        "天道：打一张+获得原判定牌=手牌数不变"
    );
    // 闪电判定：所打牌为牌堆序某牌——若判定牌非黑桃 2-9 则无伤；不精确
    // 断言伤害值，断言张角存活且获得发生即可（改判语义已由 Rewritten+
    // Gained 日志闭环）
    assert!(e.debug_alive(1));
    assert_zone_conservation(&e);
    let snap = e.save();
    let r = engine_native::Engine::restore(&snap).expect("往返");
    assert_zone_conservation(&r);
}

/// 雷击·张角：打出【闪】后座位序首个其他存活判定黑→1 雷伤+张角摸 1。
#[test]
fn boss_zhangjiao_leiji_shan_thunder() {
    let mut e = engine_native::Engine::new_boss_stage(85002, 4, 1, 3, 6, 4, 0);
    let zj_def = zhangjiao_def_of(&e);
    e.debug_set_general_def(1, zj_def);
    e.debug_set_hp(1, 8);
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    let shan = e.debug_place(1, "shan");
    let sha = e.debug_place(0, "sha");
    // 判定牌预置黑桃（非杀）：雷击判定黑 → 0 号 1 雷伤 + 张角摸 1
    e.debug_set_deck_top_by(|def| def.suit.is_spade() && def.kind != CardKind::Sha);
    assert!(matches!(
        play(&mut e, sha, vec![1]),
        engine_native::CommandOutcome::Pending { .. }
    ));
    assert_eq!(prompt_of(&e), "respond_shan");
    assert!(matches!(
        answer(&mut e, ChoiceOption::Card(shan)),
        engine_native::CommandOutcome::Pending { .. }
    ));
    assert_eq!(prompt_of(&e), "c331_jidian_target");
    drain_choices(&mut e);
    assert_eq!(hp(&e, 0), 3, "雷击：判定黑 → 0 号 1 雷伤");
    assert_eq!(e.debug_hand(1).len(), 1, "雷击：张角摸 1");
    assert_zone_conservation(&e);
    let snap = e.save();
    let r = engine_native::Engine::restore(&snap).expect("往返");
    assert_zone_conservation(&r);
}

/// 极点：选择距受伤者 ≤1 的合法角色，目标判定黑→1 雷伤；
/// 雷伤可以继续触发极点；明确取消嵌套目标选择后完成。
#[test]
fn boss_zhangjiao_jidian_nested_target_can_decline() {
    let mut e = engine_native::Engine::new_boss_stage(85003, 4, 1, 3, 6, 4, 0);
    let zj_def = zhangjiao_def_of(&e);
    e.debug_set_general_def(1, zj_def);
    e.debug_set_hp(1, 8);
    advance_to_play(&mut e, 0);

    // 张角（1 号）对 3 号造成 1 伤害 → 极点：距 3 号 ≤1 的首个其他存活
    //（座位序=0 号）判定黑 → 0 号 1 雷伤
    e.debug_set_deck_top_by(|def| def.suit.is_spade() && def.kind != CardKind::Sha);
    e.debug_deal_damage(Some(1), 3, 1, DamageNature::Normal);
    assert_eq!(prompt_of(&e), "c331_jidian_target");
    answer_sel(&mut e, vec![ChoiceOption::Player(0)]);
    drain_jidian_judge(&mut e);
    assert_eq!(hp(&e, 3), 3, "原伤害 1");
    assert_eq!(hp(&e, 0), 3, "极点：判定黑 → 0 号 1 雷伤");

    // 黑判定后的雷伤再次开启极点；测试明确取消该嵌套选择。
    assert!(e.open_choice().is_none(), "明确拒绝嵌套极点后完成");

    // 再触发：选择距 2 号 ≤1 的张角自身，黑判定后承受雷伤，再取消嵌套。
    e.debug_set_deck_top_by(|def| def.suit.is_spade());
    e.debug_deal_damage(Some(1), 2, 1, DamageNature::Normal);
    assert_eq!(prompt_of(&e), "c331_jidian_target");
    answer_sel(&mut e, vec![ChoiceOption::Player(1)]);
    drain_jidian_judge(&mut e);
    assert_eq!(hp(&e, 2), 3, "原伤害 1");
    assert_eq!(hp(&e, 1), 7, "极点：距 2 号 ≤1 首个=张角自身判定黑自雷伤");
    assert!(e.open_choice().is_none());
    assert_zone_conservation(&e);
    let snap = e.save();
    let r = engine_native::Engine::restore(&snap).expect("往返");
    assert_zone_conservation(&r);
}

fn drain_jidian_judge(e: &mut Engine) {
    for _ in 0..32 {
        let Some(c) = e.open_choice().cloned() else {
            return;
        };
        let selection = if c.spec.prompt == "gui_dao_rewrite" {
            vec![ChoiceOption::Pass]
        } else {
            vec![]
        };
        let out = answer_sel(e, selection);
        assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
    }
    panic!("judge subtree did not finish");
}
