//! `card/gwent/card.js` 12 张铜卡法术上游忠实性验证（`type: "spell"` 无无懈、过滤条件、天气邻居扩散、破晓分支等）。

use super::common::*;
use engine_native::catalog::CardKind;
use engine_native::{ChoiceOption, CommandOutcome, GameAction, LogEvent, RejectReason};

#[test]
fn u05_b5a_bronze_no_wuxie_and_filters() {
    // card/gwent/card.js:1456..1919 (`type: "spell"` 非锦囊不进无懈窗口)
    let (v, e0) = setup_gwent_locked(5501);
    let wuxie = card_of_kind(&e0, CardKind::Nullify, None);

    // 1. 奥祖之雷 (`card/gwent/card.js:1653`): target.hp >= player.hp
    let aozu = find_card(&e0, "gw_aozuzhilei");
    let mut e = inject_hand(&e0, &v, &[aozu], 0);
    e = inject_hand(&e, &v, &[wuxie], 1);
    e.debug_set_hp(1, 2); // < 0 号(4) → 拒绝
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: aozu, targets: vec![1] }),
        CommandOutcome::Rejected(RejectReason::IllegalTargets)
    ));
    e.debug_set_hp(1, 4); // == 4 → 允许，且手握无懈可击也不触发无懈询问！
    let out = submit(&mut e, 0, GameAction::PlayCard { card: aozu, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    assert!(
        e.open_choice().is_none_or(|c| c.spec.prompt != "respond_wuxie"),
        "type: spell 不触发无懈可击询问"
    );
    pass_window(&mut e);
    assert_eq!(e.debug_hp(1), 3, "奥祖之雷造成 1 点雷伤");
    assert!(
        e.log().iter().any(|l| matches!(
            l,
            LogEvent::DamageDealt {
                target: 1,
                amount: 1,
                ..
            }
        )),
        "奥祖之雷应记录伤害日志"
    );
    assert_eq!(e.debug_hand(1).len(), 2, "奥祖之雷目标摸 1");
    assert_zone_conservation(&e);

    // 2. 昆恩法印 (`card/gwent/card.js:1765`): 单目标含自己，!hasSkill('gw_kunenfayin')，只防普通伤害
    let kunen = find_card(&e0, "gw_kunenfayin");
    let aozu2 = find_card(&e0, "gw_aozuzhilei");
    let mut e = inject_hand(&e0, &v, &[kunen, aozu2], 0);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: kunen, targets: vec![0] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: kunen, targets: vec![0] }),
        CommandOutcome::Rejected(_)
    ));
    // 火/雷属性伤害穿透昆恩（card/gwent/skill.js:49 `!event.nature`）
    let mut e = inject_hand(&e, &v, &[aozu2], 0);
    e.debug_set_hp(0, 4);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: aozu2, targets: vec![0] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    assert_eq!(e.debug_hp(0), 3, "属性伤害不被昆恩法印防止");
    assert_zone_conservation(&e);
}

#[test]
fn u05_b5a_bronze_weather_zhihuan_zumo_wenyi_poxiao() {
    let (v, e0) = setup_gwent_locked(5502);

    // 1. 刺骨寒霜 + 蔽日浓雾 + 倾盆大雨 (`card/gwent/card.js:1456..1517`): 扩散给存活邻座，已持有者拒绝目标
    let qinpen = find_card(&e0, "gw_qinpendayu");
    let birin = find_card(&e0, "gw_birinongwu");
    let cigu = find_card(&e0, "gw_ciguhanshuang");
    let mut e = inject_hand(&e0, &v, &[qinpen, birin, cigu], 0);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: birin, targets: vec![0] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    // 0 号持有 gw_birinongwu 后出杀应受阻，且邻座（1 与 3 号）亦获蔽日浓雾
    let sha = card_of_kind(&e, CardKind::Sha, None);
    let mut e_sha = inject_hand(&e, &v, &[sha], 0);
    assert!(
        matches!(
            submit(&mut e_sha, 0, GameAction::PlayCard { card: sha, targets: vec![1] }),
            CommandOutcome::Rejected(_)
        ),
        "蔽日浓雾应阻止出杀"
    );
    // 重试指定已有浓雾的 1 号被拒绝
    let birin2 = find_cards_matching(&e0, |d| d.kind == CardKind::Birinongwu, 2)[1];
    let mut e_dup = inject_hand(&e, &v, &[birin2], 0);
    assert!(matches!(
        submit(&mut e_dup, 0, GameAction::PlayCard { card: birin2, targets: vec![1] }),
        CommandOutcome::Rejected(RejectReason::IllegalTargets)
    ));

    // 2. 置换军 (`card/gwent/card.js:1580`): target 受伤且手牌<maxHp 时循环 loseMaxHp
    let zhihuan = find_card(&e0, "gw_zhihuanjun");
    let mut e = inject_hand(&e0, &v, &[zhihuan], 0);
    e.debug_set_hp(1, 2); // max_hp=4, hand=0 < 4 → 循环失上限至 max_hp=2 或不再满足
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: zhihuan, targets: vec![1] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    assert_eq!(e.save().core.players[1].max_hp, 2, "置换军循环失体力上限至不受伤");
    assert_zone_conservation(&e);

    // 3. 祖墨手铠 (`card/gwent/card.js:1609`): hujia==0 && 已封印时不可选中；封印不禁用锁定技；燕子药水 / 暴雪药水
    let zumo = find_card(&e0, "gw_zumoshoukao");
    let yz = find_card(&e0, "gw_yanziyaoshui");
    let bx = find_card(&e0, "gw_baoxueyaoshui");
    let extra: Vec<_> = find_cards_matching(&e0, |d| d.kind == CardKind::Sha, 3);
    let mut e = inject_hand(&e0, &v, &[zumo, yz, bx], 0);
    e = inject_hand(&e, &v, &extra[..2], 2);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: zumo, targets: vec![1] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    let zumo2 = find_cards_matching(&e0, |d| d.kind == CardKind::Zumoshoukao, 2)[1];
    let mut e = inject_hand(&e, &v, &[zumo2], 0);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: zumo2, targets: vec![1] }),
        CommandOutcome::Rejected(RejectReason::IllegalTargets)
    ));
    // 燕子药水给当前手牌极少的 1 号摸 2，暴雪药水弃 2 号两手牌摸 1
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: yz, targets: vec![1] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    assert_eq!(e.debug_hand(1).len(), 2);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: bx, targets: vec![2] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "gw_baoxue_discard");
    let h2_opts: Vec<_> = e.open_choice().unwrap().spec.options.iter().take(2).copied().collect();
    let (mut e, out) = save_restore_answer(e, &v, 2, h2_opts);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    pass_window(&mut e);
    assert_eq!(e.debug_hand(2).len(), 1);
    assert_zone_conservation(&e);

    // 4. 瘟疫多最少体力平手全命中 + 破晓分支 0 存档恢复清天气与判定牌
    let wenyi = find_card(&e0, "gw_wenyi");
    let mut e = inject_hand(&e0, &v, &[wenyi], 0);
    e.debug_set_hp(1, 1);
    e.debug_set_hp(3, 1);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: wenyi, targets: vec![] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    assert!(e.debug_hp(1) < 1 && e.debug_hp(3) < 1, "平手最少体力全体命中");

    let px = find_card(&e0, "gw_poxiao");
    let mut e = inject_hand(&e0, &v, &[qinpen, px], 0);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: qinpen, targets: vec![1] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: px, targets: vec![] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "gw_poxiao_branch");
    let (e, out) = save_restore_answer(e, &v, 0, vec![ChoiceOption::Tag(0)]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    assert_eq!(e.open_choice().unwrap().spec.prompt, "gw_poxiao_targets");
    let (mut e, out) = save_restore_answer(e, &v, 0, vec![ChoiceOption::Player(1)]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    pass_window(&mut e);
    assert!(
        !e.save().core.skills.keys().any(|(o, d)| {
            *o == 1 && e.catalog().skills[*d as usize].name.contains("qinpen")
        }),
        "破晓分支 0 清除目标天气"
    );
    assert!(!is_vanished(&e, px), "铜法术破晓无 vanish，进入弃牌堆");
    assert_zone_conservation(&e);
}

#[test]
fn u05_b5a_round2_aozu_death_before_draw_and_baoxue_lifo() {
    let (v, e0) = setup_gwent_locked(5503);

    // #3: `gw_aozuzhilei` 对 1 血空手且牌堆顶为【桃】的目标：先受 1 雷伤死去，不得摸那张【桃】
    let aozu = find_card(&e0, "gw_aozuzhilei");
    let tao = card_of_kind(&e0, CardKind::Tao, None);
    let mut e = inject_hand(&e0, &v, &[aozu], 0);
    e.debug_set_hp(0, 1);
    e.debug_set_hp(1, 1);
    let mut snap = e.save();
    snap.core.deck.retain(|c| *c != tao);
    for p in &mut snap.core.players {
        p.hand.retain(|c| *c != tao);
        p.equips.retain(|c| *c != tao);
        p.judge.retain(|c| *c != tao);
    }
    snap.core.discard.retain(|c| *c != tao);
    snap.core.deck.push(tao);
    let mut e = content_native_loader::restore_locked_engine(&snap, &v).unwrap();
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: aozu, targets: vec![1] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    drain_default(&mut e);
    assert!(!e.debug_alive(1), "奥祖之雷先伤后摸：1 血目标应先死亡而非摸牌自救");
    assert!(e.debug_hand(1).is_empty(), "死后不再摸牌");
    assert_zone_conservation(&e);

    // #10 (`gw_baoxueyaoshui`): 先全弃两手牌触发【连营】摸 1，再执行暴雪药水摸 1（净得 2 张）
    let bx = find_card(&e0, "gw_baoxueyaoshui");
    let shas = find_cards_matching(&e0, |d| d.kind == CardKind::Sha, 2);
    let mut e_bx = grant_skill(&e0, &v, "连营", 2);
    e_bx = inject_hand(&e_bx, &v, &[bx], 0);
    e_bx = inject_hand(&e_bx, &v, &shas, 2);
    assert!(matches!(
        submit(&mut e_bx, 0, GameAction::PlayCard { card: bx, targets: vec![2] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e_bx);
    assert_eq!(e_bx.open_choice().unwrap().spec.prompt, "gw_baoxue_discard");
    let opts: Vec<_> = e_bx.open_choice().unwrap().spec.options.clone();
    let bx_id = e_bx.open_choice().unwrap().id;
    submit(&mut e_bx, 2, GameAction::Answer {
        choice_id: bx_id,
        selection: opts,
    });
    let ly_choice = e_bx.open_choice().cloned().expect("弃空手牌后应在暴雪摸牌前触发连营确认");
    assert_eq!(ly_choice.observer, 2);
    assert_eq!(ly_choice.spec.prompt, "trigger_confirm");
    submit(&mut e_bx, 2, GameAction::Answer {
        choice_id: ly_choice.id,
        selection: vec![ChoiceOption::Confirm],
    });
    pass_window(&mut e_bx);
    assert_eq!(e_bx.debug_hand(2).len(), 2, "连营摸 1 + 暴雪药水摸 1 = 2 张");
    assert_zone_conservation(&e_bx);
}
