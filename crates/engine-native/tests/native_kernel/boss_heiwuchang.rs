use super::*;

/// 阴间系黑无常片（规则 v256）：鬼计（弃己判定区首张）+太平（摸牌+2）
/// +索命（横置全部未链接异侧）+吸行（对首个已链接异侧 1 雷伤+回 1）。
/// 黑无常条目（[鬼计,太平,索命,吸行]，hp9；bianshen4 已处置）。
#[test]
fn boss_heiwuchang_guiji_taiping_suoming_xixing() {
    let mut e = Engine::new_boss_stage(76001, 4, 1, 3, 6, 4, 0);
    let hw_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 33) as u16; // BOSS_GENERALS[33] 黑无常
    e.debug_set_general_def(1, hw_def);
    e.debug_set_hp(1, 9);
    assert_eq!(
        e.catalog().general_meta(hw_def).map(|g| g.0),
        Some("黑无常"),
        "黑无常入目录"
    );
    for n in ["鬼计", "太平", "索命", "吸行"] {
        let d = e.catalog().skill_by_name(n).expect(n);
        assert!(e.state_core().skills.contains_key(&(1, d)), "{n} 入目录");
    }

    // 布阵：给 1 号判定区放一张乐（鬼计目标）；清全场手牌
    let _le = e.debug_place_judge(1, "le");
    for t in [0u8, 1, 2, 3] {
        e.debug_clear_hand(t);
    }
    e.debug_set_hp(0, 4);
    e.debug_set_hp(2, 4);
    e.debug_set_hp(3, 4);

    // ① 鬼计：推进到 1 号判定锚——先弃己判定区乐（judge 空→乐不生效）
    //    再过太平/索命/吸行流程（吸行无已链接目标→不触发）
    for _ in 0..12 {
        if e.turn().current == 1 && e.turn().phase == Phase::Draw {
            break;
        }
        if let Some(c) = e.open_choice().cloned() {
            if c.spec.prompt == "dying_save" {
                answer(&mut e, ChoiceOption::Pass);
                drain_choices(&mut e);
            } else {
                answer_cards(&mut e, &c);
            }
            continue;
        }
        match next_phase(&mut e) {
            CommandOutcome::Accepted { .. } => {}
            CommandOutcome::Suspended => {}
            CommandOutcome::Pending { choice } => {
                let is_dying = choice.spec.prompt == "dying_save";
                if is_dying {
                    answer(&mut e, ChoiceOption::Pass);
                    drain_choices(&mut e);
                } else {
                    answer_cards(&mut e, &choice);
                }
            }
            CommandOutcome::Rejected(r) => panic!("np rejected {r:?}"),
        }
    }
    assert!(e.debug_judge(1).is_empty(), "鬼计：己判定区牌已弃");
    // 太平：1 号摸牌阶段 +2（基础 2+太平 2=4）
    assert_eq!(e.debug_hand(1).len(), 4, "太平：摸牌 +2（2+2）");
    // 索命：End 横置全部异侧 → 0/2/3 linked（从 Draw 走 Play→Discard→End）
    for _ in 0..6 {
        if e.turn().current == 1 && e.turn().phase == Phase::End {
            break;
        }
        if let Some(c) = e.open_choice().cloned() {
            if c.spec.prompt == "dying_save" {
                answer(&mut e, ChoiceOption::Pass);
                drain_choices(&mut e);
            } else {
                answer_cards(&mut e, &c);
            }
            continue;
        }
        next_phase(&mut e);
    }
    assert!(e.state_core().player(0).linked, "索命：0 号横置");
    assert!(e.state_core().player(2).linked, "索命：2 号横置");
    assert!(e.state_core().player(3).linked, "索命：3 号横置");
    assert!(!e.state_core().player(1).linked, "本体不横置");
    // 前置掉血（条目 hp9=上限，为下回合吸行回复可见性）
    e.debug_set_hp(1, 8);

    // ② 吸行：次回合判定锚——对座位序首个已链接异侧（0 号）1 雷伤+回 1
    next_phase(&mut e); // End→0 Judge（索命已过）
    drain_choices(&mut e);
    // 0 号回合走完，回到 1 号判定锚
    for _ in 0..24 {
        if e.turn().current == 1 && e.turn().phase == Phase::Judge {
            break;
        }
        if let Some(c) = e.open_choice().cloned() {
            if c.spec.prompt == "dying_save" {
                answer(&mut e, ChoiceOption::Pass);
                drain_choices(&mut e);
            } else {
                answer_cards(&mut e, &c);
            }
            continue;
        }
        match next_phase(&mut e) {
            CommandOutcome::Accepted { .. } => {}
            CommandOutcome::Suspended => {}
            CommandOutcome::Pending { choice } => {
                let is_dying = choice.spec.prompt == "dying_save";
                if is_dying {
                    answer(&mut e, ChoiceOption::Pass);
                    drain_choices(&mut e);
                } else {
                    answer_cards(&mut e, &choice);
                }
            }
            CommandOutcome::Rejected(r) => panic!("np rejected {r:?}"),
        }
    }
    assert_eq!(e.debug_hp(0), 3, "吸行：0 号受 1 雷伤（4-1）");
    assert_zone_conservation(&e);

    // ③ 存档往返
    let snap = e.save();
    let r = Engine::restore(&snap).expect("往返");
    for n in ["鬼计", "太平", "索命", "吸行"] {
        let d = r.catalog().skill_by_name(n).expect(n);
        assert!(r.state_core().skills.contains_key(&(1, d)), "{n} 实例往返");
    }
    assert_zone_conservation(&r);
}
