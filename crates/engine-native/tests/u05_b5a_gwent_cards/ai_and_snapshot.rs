//! U05-5a AI 公开信息决策策略与 `u05_vanished` 洗牌防漏/全牌守恒专项测试。

use super::common::*;
use engine_native::ai::BoundedPlanner;
use engine_native::catalog::CardKind;
use engine_native::{ChoiceOption, CommandOutcome, GameAction, Phase, RejectReason};

#[test]
fn u05_b5a_ai_public_info_choices_prefer_expected_options() {
    let (v, e0) = setup_gwent_locked(5550);
    let planner = BoundedPlanner::new(32);

    // 1. gw_poxiao_branch: 友方无减益仅敌方有判定牌选 Tag(1) 摸牌；己方有天气时选 Tag(0) 清除
    let px = find_card(&e0, "gw_poxiao");
    let qpd = find_card(&e0, "gw_qinpendayu");
    let le = card_of_kind(&e0, CardKind::Le, None);
    let mut e1 = inject_hand(&e0, &v, &[px], 0);
    e1 = inject_judge(&e1, &v, le, 1);
    submit(&mut e1, 0, GameAction::PlayCard { card: px, targets: vec![] });
    pass_window(&mut e1);
    assert_eq!(e1.open_choice().unwrap().spec.prompt, "gw_poxiao_branch");
    let plan1 = e1.plan_action(0, &planner);
    assert!(
        matches!(plan1.action, GameAction::Answer { ref selection, .. } if selection == &vec![ChoiceOption::Tag(1)]),
        "仅敌方挂判定牌时，AI 破晓应选 Tag(1) 摸铜法术: {:?}",
        plan1.action
    );

    let mut e2 = inject_hand(&e0, &v, &[qpd, px], 0);
    submit(&mut e2, 0, GameAction::PlayCard { card: qpd, targets: vec![0] });
    pass_window(&mut e2);
    submit(&mut e2, 0, GameAction::PlayCard { card: px, targets: vec![] });
    pass_window(&mut e2);
    let plan2 = e2.plan_action(0, &planner);
    assert!(
        matches!(plan2.action, GameAction::Answer { ref selection, .. } if selection == &vec![ChoiceOption::Tag(0)]),
        "自身挂倾盆大雨时，AI 破晓应选 Tag(0) 自我净化: {:?}",
        plan2.action
    );

    // 2. gw_zuihouyuanwang_discard: 优先弃最低价值手牌（如【杀】低于【桃】）
    let zuihou = find_card(&e0, "gw_zuihouyuanwang");
    let tao = card_of_kind(&e0, CardKind::Tao, None);
    let shas = find_cards_matching(&e0, |d| d.kind == CardKind::Sha, 2);
    let mut e = inject_hand(&e0, &v, &[zuihou, tao, shas[0]], 0);
    submit(&mut e, 0, GameAction::PlayCard { card: zuihou, targets: vec![] });
    pass_window(&mut e);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "gw_zuihouyuanwang_discard");
    let plan = e.plan_action(0, &planner);
    let sel = match plan.action {
        GameAction::Answer { selection, .. } => selection,
        other => panic!("expected Answer from planner, got {other:?}"),
    };
    assert!(!sel.contains(&ChoiceOption::Card(tao)), "AI 不应优先弃置高价值【桃】");

    // 3. gwmaoxian_gaier_branch: 自身受伤选 Tag(0) 增上限，满血选 Tag(1) 降敌上限
    let gaier = find_card(&e0, "gwmaoxian_gaier");
    let mut e_hurt = inject_hand(&e0, &v, &[gaier], 0);
    e_hurt.debug_set_hp(0, 2);
    submit(&mut e_hurt, 0, GameAction::PlayCard { card: gaier, targets: vec![0] });
    pass_window(&mut e_hurt);
    let plan_g = e_hurt.plan_action(0, &planner);
    assert!(
        matches!(plan_g.action, GameAction::Answer { ref selection, .. } if selection == &vec![ChoiceOption::Tag(0)]),
        "AI 受伤选 Tag(0): {:?}",
        plan_g.action
    );

    // 4. gw_huangjiashenpan_pick: PickTag 选法术（不得返回空 ChoiceOption::Card 导致被拒）
    let huangjia = find_card(&e0, "gw_huangjiashenpan");
    let mut e_hj = inject_hand(&e0, &v, &[huangjia], 0);
    submit(&mut e_hj, 0, GameAction::PlayCard { card: huangjia, targets: vec![] });
    pass_window(&mut e_hj);
    assert_eq!(e_hj.open_choice().unwrap().spec.prompt, "gw_huangjiashenpan_pick");
    let plan_hj = e_hj.plan_action(0, &planner);
    assert!(
        matches!(
            plan_hj.action,
            GameAction::Answer { ref selection, .. }
                if matches!(selection.as_slice(), [ChoiceOption::Tag(_)])
        ),
        "AI 皇家审判应选 ChoiceOption::Tag: {:?}",
        plan_hj.action
    );
    let out = submit(&mut e_hj, 0, plan_hj.action);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "AI 皇家审判选牌提交必须成功: {out:?}"
    );
    pass_window(&mut e_hj);
    assert_eq!(e_hj.debug_hand(0).len(), 1);
    assert_zone_conservation(&e_hj);
}

#[test]
fn u05_b5a_vanished_cards_never_reenter_discard_on_reshuffle() {
    let (v, e0) = setup_gwent_locked(5551);
    let ansha = find_card(&e0, "gw_ansha");
    let youer = find_card(&e0, "gw_youer");
    let sha = card_of_kind(&e0, CardKind::Sha, None);

    let mut e = inject_hand(&e0, &v, &[youer, ansha], 0);
    e = inject_hand(&e, &v, &[sha], 1);
    e.debug_set_hp(1, 1);

    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: youer, targets: vec![1] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: ansha, targets: vec![] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    assert_ne!(e.turn().phase, Phase::Play);
    assert!(is_vanished(&e, youer) && is_vanished(&e, ansha));

    // 清空 deck 仅留 1 张触发洗牌，确认 vanish 牌不会混入 deck 或 discard
    let mut snap = e.save();
    let keep = snap.core.deck.pop();
    snap.core.discard.extend(std::mem::take(&mut snap.core.deck));
    if let Some(k) = keep {
        snap.core.deck.push(k);
    }
    let mut e = content_native_loader::restore_locked_engine(&snap, &v).unwrap();
    advance_to_seat_play(&mut e, 2);
    let core = e.state_core();
    assert!(!core.deck.contains(&youer) && !core.deck.contains(&ansha));
    assert!(!core.discard.contains(&youer) && !core.discard.contains(&ansha));
    assert!(is_vanished(&e, youer) && is_vanished(&e, ansha));
    assert_zone_conservation(&e);

    // 验证不可直接打出的衍生牌保持 Reject
    let gw_dieyi = find_card(&e0, "gw_dieyi");
    let mut e_rej = inject_hand(&e0, &v, &[gw_dieyi], 0);
    assert!(matches!(
        submit(&mut e_rej, 0, GameAction::PlayCard { card: gw_dieyi, targets: vec![] }),
        CommandOutcome::Rejected(RejectReason::WrongPhase)
    ));
}
