use super::*;

/// P1/N10-a（ADR 0768）：私有选择应答对非观察者不可见——
/// 观星应答的牌 id 不得进入他人/观战者的 AI 事件流与表现投影。
#[test]
fn n10a_private_choice_answer_hidden_from_others() {
    use engine_native::view::EventView;
    let mut e = Engine::new(901, 3);
    e.debug_grant(0, "观星");
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let mut answered_card = None;
    for _ in 0..128 {
        let _actor = e.turn().current;
        match next_phase(&mut e) {
            CommandOutcome::Accepted { .. } => {}
            CommandOutcome::Pending { choice } => match choice.spec.prompt {
                "trigger_confirm" => {
                    assert_eq!(choice.observer, 0);
                    answer(&mut e, ChoiceOption::Confirm);
                    // 确认后落位询问在同一命令段内同步挂起
                    let arr = e.open_choice().expect("guanxing_arrange 应挂起");
                    assert_eq!(arr.spec.prompt, "guanxing_arrange");
                    let card = arr
                        .spec
                        .options
                        .iter()
                        .find_map(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        })
                        .expect("观星应有牌选项");
                    answered_card = Some(card);
                    answer_sel(&mut e, vec![ChoiceOption::Card(card)]);
                    break;
                }
                other => panic!("unexpected prompt: {other}"),
            },
            o => panic!("walk failed: {o:?}"),
        }
    }
    let card = answered_card.expect("观星应发生");
    // 他人 AI 事件流：不得出现该询问的私有应答内容
    for other in 1..3u8 {
        let events = e.ai_session(other).events();
        for ev in &events {
            if let EventView::Public(engine_native::LogEvent::ChoiceAnswered { id, options }) = ev {
                let _ = id;
                assert!(
                    !options.iter().any(|o| o.contains(&format!("card:{card}"))),
                    "座位 {other} 不得看到他人私有应答选项: {options:?}"
                );
            }
        }
        // 他人流中必须存在该询问的脱敏占位（审计可知有应答、不知内容）
        assert!(
            events
                .iter()
                .any(|ev| matches!(ev, EventView::ChoiceAnsweredHidden { id: _ })),
            "他人流应保留脱敏应答占位"
        );
    }
    // 观察者本人的 AI 流：应答内容完整可见（对账能力保留）
    let own = e.ai_session(0).events();
    assert!(
        own.iter().any(|ev| matches!(
            ev,
            EventView::Public(engine_native::LogEvent::ChoiceAnswered { options, .. })
                if options.iter().any(|o| o.contains(&format!("card:{card}")))
        )),
        "观察者本人应可见自己的应答内容"
    );
}
