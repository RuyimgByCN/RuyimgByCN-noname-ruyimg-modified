//! L28 雷霆万钧 Batch A 触发技的 6 维真实对局验收（2026-11-12）。
//! 共用 `l28_thunder_eight_match.rs` 的命令驱动与存档辅助函数；六维编号同该文件头注释。

use super::l28_thunder_eight_match::*;
use engine_native::catalog::{CardKind, Suit};
use engine_native::event::Phase;
use engine_native::{CommandOutcome, GameAction};

/// 3. Batch A 触发技：布势/莲华/栖肆/隐世/承业/遁世伤害防止与三选二/归离首回合选标与无伤额外阶段。
#[test]
fn l28_thunder_eight_batch_a_triggers() {
    let p = pack();
    // ---- th_bushi 布势（`extension.js:4203`）：讨无次数限制、杀使用摸2、迎被指定得止牌、止结束得牌 ----
    let mut e = fresh(&p, 2831, "th_lukai");
    let out = hand_turn_to(&mut e, 0, 1);
    assert!(
        matches!(out, CommandOutcome::Pending { .. }),
        "posture prompt opens: {out:?}"
    );
    open(&e, "th_bushi:choose_posture");
    assert!(
        rejected(&mut e, Vec::new()) && rejected(&mut e, vec![tag(4)]),
        "③ forced 0..4"
    );
    snapshot_roundtrip(&e, &p, vec![tag(1)]);
    answer(&mut e, "th_bushi:choose_posture", vec![tag(1)]);
    assert_eq!(
        (
            flag(&e, 0, "th_bushi_tao"),
            flag(&e, 0, "th_bushi_sha"),
            flag(&e, 0, "th_bushi_ying"),
            flag(&e, 0, "th_bushi_zhi"),
            flag(&e, 0, "th_bushi_unlimited_suit"),
        ),
        (1, 2, 3, 0, 2),
        "① posture 1 assigns 讨=Heart(1), 杀=Club(2), 迎=Diamond(3), 止=Spade(0)"
    );
    advance_to(&mut e, Phase::Play);
    // ⑤ 讨花色（Heart）无使用次数限制：同一出牌阶段连续使用 2 张红桃【杀】均合法，再出黑桃【杀】被次数限制拒绝
    let heart_shas: Vec<u32> = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .filter(|(_, c)| c.kind == CardKind::Sha && c.suit == Suit::Heart)
        .map(|(i, _)| i as u32)
        .take(2)
        .collect();
    give(&mut e, 0, heart_shas[0]);
    give(&mut e, 0, heart_shas[1]);
    play(&mut e, 0, heart_shas[0], vec![1]);
    drain(&mut e);
    play(&mut e, 0, heart_shas[1], vec![3]);
    drain(&mut e);
    let spade_sha = card_by(&e, |c| c.kind == CardKind::Sha && c.suit == Suit::Spade);
    give(&mut e, 0, spade_sha);
    assert!(
        matches!(
            submit(
                &mut e,
                0,
                GameAction::PlayCard {
                    card: spade_sha,
                    targets: vec![1]
                }
            ),
            CommandOutcome::Rejected(_)
        ),
        "④ non-讨 suit Sha is blocked after Sha use limit reached"
    );
    // ① 使用杀花色（Club）牌时摸 2 张牌
    let club_eq = card_by(&e, |c| {
        c.suit == Suit::Club
            && matches!(
                c.kind.equip_slot(),
                Some(
                    engine_native::catalog::EquipSlot::Weapon
                        | engine_native::catalog::EquipSlot::Armor
                )
            )
    });
    give(&mut e, 0, club_eq);
    let h0 = hand(&e, 0);
    play(&mut e, 0, club_eq, Vec::new());
    drain(&mut e);
    assert_eq!(
        hand(&e, 0),
        h0 + 1,
        "① using 杀-suit (Club) card spends 1 and draws 2 (net +1)"
    );
    // ① 结束阶段获得 1 张止花色（Spade）牌
    e.debug_set_turn(0, Phase::Discard, 2);
    let before: Vec<u32> = e.state_core().player(0).hand.clone();
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    drain(&mut e);
    let gained: Vec<u32> = e
        .state_core()
        .player(0)
        .hand
        .iter()
        .copied()
        .filter(|c| !before.contains(c))
        .collect();
    assert_eq!(gained.len(), 1, "① end phase gains one 止-suit card");
    assert_eq!(e.catalog().cards[gained[0] as usize].suit, Suit::Spade);
    // ① 成为迎花色（Diamond）牌的目标时获得 1 张止花色（Spade）牌
    set_flag(&mut e, 0, "th_zhongzhuang_mounted", 0);
    e.state_core_mut().turn.sha_used_play_phase = false;
    e.debug_set_turn(1, Phase::Play, 2);
    let dia_sha = card_by(&e, |c| c.kind == CardKind::Sha && c.suit == Suit::Diamond);
    give(&mut e, 1, dia_sha);
    let h0 = hand(&e, 0);
    play(&mut e, 1, dia_sha, vec![0]);
    drain(&mut e);
    assert_eq!(
        hand(&e, 0),
        h0 + 1,
        "① targeted by 迎-suit (Diamond) card gains 1 止-suit card"
    );

    // ---- shhlianhua 莲华（`extension.js:4584`）：成为【杀】目标摸 1，受伤再摸 1 ----
    let mut e = fresh(&p, 2832, "th_sunhanhua");
    e.debug_set_turn(1, Phase::Play, 1);
    let sha = card_by(&e, |c| c.kind == CardKind::Sha);
    give(&mut e, 1, sha);
    let (h0, hp0) = (hand(&e, 0), hp(&e, 0));
    play(&mut e, 1, sha, vec![0]);
    open(&e, "respond_shan");
    drain(&mut e);
    assert_eq!(
        (hand(&e, 0), hp(&e, 0)),
        (h0 + 2, hp0 - 1),
        "① targeted + damaged draws"
    );
    assert_eq!(flag(&e, 0, "th_shhlianhua_ward"), 1);

    // ---- olqisi 栖肆（`extension.js:5100`）：首回合开始装备两件；摸牌阶段选类别得装备 ----
    let mut e = fresh(&p, 2833, "th_puyuan");
    let _ = hand_turn_to(&mut e, 0, 1);
    let eq = e.state_core().player(0).equips.clone();
    assert_eq!(
        eq.len(),
        2,
        "① two different-slot equips at first turn begin"
    );
    assert_ne!(
        e.catalog().card_kind(eq[0]).equip_slot(),
        e.catalog().card_kind(eq[1]).equip_slot()
    );
    while e.open_choice().is_none() {
        let _ = submit(&mut e, 0, GameAction::NextPhase);
    }
    assert_eq!(e.state_core().turn.phase, Phase::Draw);
    assert!(rejected(&mut e, vec![tag(4)]), "③ slot 0..4");
    snapshot_roundtrip(&e, &p, vec![tag(1)]);
    let before: Vec<u32> = e.state_core().player(0).hand.clone();
    answer(&mut e, "olqisi:choose_slot", vec![tag(1)]);
    drain(&mut e);
    let got: Vec<u32> = e
        .state_core()
        .player(0)
        .hand
        .iter()
        .copied()
        .filter(|c| !before.contains(c))
        .collect();
    assert!(
        got.iter()
            .any(|&c| e.catalog().card_kind(c).equip_slot().is_some()),
        "① equip"
    );
    assert_eq!(flag(&e, 0, "th_olqisi_converted"), 1);
    let _ = hand_turn_to(&mut e, 0, 2);
    assert_eq!(
        e.state_core().player(0).equips,
        eq,
        "④ initial equip only once"
    );

    // ---- xinfu_pdgyingshi 隐世（`extension.js:3662`）：⑤ 不能成为延时锦囊目标；准备阶段隐居 ----
    let mut e = fresh(&p, 2834, "th_pangdegong");
    e.debug_set_turn(1, Phase::Play, 1);
    let le = card_by(&e, |c| c.kind == CardKind::Le);
    give(&mut e, 1, le);
    let out = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: le,
            targets: vec![0],
        },
    );
    assert!(
        matches!(out, CommandOutcome::Rejected(_)),
        "⑤ 乐不思蜀 cannot target 隐世"
    );
    assert!(
        e.state_core().player(0).judge.is_empty() && e.state_core().player(1).hand.contains(&le)
    );
    assert!(!matches!(
        submit(
            &mut e,
            1,
            GameAction::PlayCard {
                card: le,
                targets: vec![2]
            }
        ),
        CommandOutcome::Rejected(_)
    ));
    assert_eq!(flag(&e, 0, "th_yingshi_hermit"), 0);
    let _ = hand_turn_to(&mut e, 0, 1);
    assert_eq!(
        flag(&e, 0, "th_yingshi_hermit"),
        1,
        "① prepare phase hermit"
    );

    // ---- th_chengye 承业（`extension.js:6155`）：其他角色弃牌，按缺失“六经”收为“典” ----
    let mut e = fresh(&p, 2835, "th_mamidi");
    e.debug_set_turn(1, Phase::Play, 1);
    for _ in 0..4 {
        let c = e.state_core().deck[0];
        give(&mut e, 1, c);
    }
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    let c = open(&e, "discard_to_limit");
    let picked: Vec<u32> = e
        .state_core()
        .player(1)
        .hand
        .iter()
        .take(c.spec.min)
        .copied()
        .collect();
    let expect_mask = picked.iter().fold(0u32, |m, &x| {
        let k = e.catalog().card_kind(x);
        let ch = if k.is_sha_like() {
            0
        } else if k == CardKind::Shan {
            1
        } else if k == CardKind::Tao {
            2
        } else if k == CardKind::Jiu {
            3
        } else if k.equip_slot().is_some() {
            4
        } else {
            5
        };
        m | (1 << ch)
    });
    answer(
        &mut e,
        "discard_to_limit",
        picked.iter().map(|&x| cd(x)).collect(),
    );
    assert_eq!(
        flag(&e, 0, "th_chengye_mask"),
        expect_mask,
        "① one 典 per missing classic"
    );
    let dian = e.state_core().vm_aux.piles[&(0, "th_liujing".to_string())].clone();
    assert_eq!(dian.len() as u32, expect_mask.count_ones());
    assert!(
        dian.iter().all(|c| !e.state_core().discard.contains(c)),
        "典 leave the discard pile"
    );

    // ---- th_dunshi 遁世（`extension.js:3783`）：伤害前触发三选二（防止伤害授仁义礼智信 / 减上限摸X / 删牌名）----
    let mut e = fresh(&p, 2836, "th_guanning");
    activate(&mut e, 0, "th_dunshi");
    answer(&mut e, "th_dunshi:choose_basic", vec![tag(0)]);
    assert!(
        rejected(&mut e, vec![pl(0)]),
        "③ virtual sha cannot target self"
    );
    let hp1 = hp(&e, 1);
    answer(&mut e, "th_dunshi:choose_sha_target", vec![pl(1)]);
    let c = open(&e, "th_dunshi:damage_choice");
    assert_eq!(c.observer, 0, "dunshi owner chooses 2 of 3 options");
    assert!(rejected(&mut e, vec![tag(3)]), "③ option 0..=2");
    snapshot_roundtrip(&e, &p, vec![tag(0)]);
    // 第 1 项选 0：防止此伤害，并为当前回合角色选择 1 个“仁义礼智信”技能
    answer(&mut e, "th_dunshi:damage_choice", vec![tag(0)]);
    assert!(rejected(&mut e, vec![tag(6)]), "③ skill pool 0..6");
    snapshot_roundtrip(&e, &p, vec![tag(1)]);
    answer(&mut e, "th_dunshi:choose_skill", vec![tag(1)]);
    assert!(
        has_skill(&e, 0, "奸雄") && flag(&e, 0, "th_dunshi_gained_skill") == 2,
        "① granted jianxiong (奸雄)"
    );
    // 第 2 项从剩余 {1, 2} 中选 2：移除本次使用的基本牌名（杀，bit 0）
    assert!(
        rejected(&mut e, vec![tag(0)]),
        "③ already-chosen option 0 is excluded from second pick"
    );
    answer(&mut e, "th_dunshi:damage_choice", vec![tag(2)]);
    assert_eq!(
        (hp(&e, 1), flag(&e, 0, "th_dunshi_removed_mask")),
        (hp1, 0b0001),
        "① damage prevented (seat 1 HP unchanged) and sha removed from dunshi"
    );
    // 下一回合：杀（tag 0）已被移除不可再选；选酒（tag 3），出杀触发三选二选 {2, 1}（删酒 + 减1上限摸2）
    next_turn(&mut e, 1);
    activate(&mut e, 0, "th_dunshi");
    assert!(
        rejected(&mut e, vec![tag(0)]),
        "③ removed basic card (sha) cannot be used again"
    );
    answer(&mut e, "th_dunshi:choose_basic", vec![tag(3)]);
    let sha = card_by(&e, |c| c.kind == CardKind::Sha);
    give(&mut e, 0, sha);
    let h0 = hand(&e, 0);
    play(&mut e, 0, sha, vec![1]);
    // 跳过闪响应直达 DamageBefore 触发的 th_dunshi:damage_choice
    if e.open_choice().is_some_and(|o| o.spec.prompt == "respond_shan") {
        answer(&mut e, "respond_shan", vec![ engine_native::ChoiceOption::Pass ]);
    }
    answer(&mut e, "th_dunshi:damage_choice", vec![tag(2)]);
    answer(&mut e, "th_dunshi:damage_choice", vec![tag(1)]);
    drain(&mut e);
    assert_eq!(
        (
            e.state_core().player(0).max_hp,
            flag(&e, 0, "th_dunshi_removed_mask"),
            hand(&e, 0),
        ),
        (6, 0b1001, h0 - 1 + 2),
        "① opt 2+1 removes jiu (2 removed total), loses 1 max HP (7->6), and draws X=2 cards"
    );

    // ---- th_guili 归离（`extension.js:4475`）：首回合选其他角色；其每轮首回合未造伤则获得额外准备+出牌 ----
    let mut e = fresh(&p, 2837, "th_caohua");
    let _ = hand_turn_to(&mut e, 0, 1);
    open(&e, "th_guili:choose_target");
    assert!(
        rejected(&mut e, Vec::new()) && rejected(&mut e, vec![pl(0)]),
        "③ guili target is mandatory and must be another player"
    );
    snapshot_roundtrip(&e, &p, vec![pl(1)]);
    answer(&mut e, "th_guili:choose_target", vec![pl(1)]);
    assert_eq!(flag(&e, 0, "th_guili_target"), 2, "① seat 1 recorded as guili target");
    // 第 1 轮座位 1 首回合：座位 1 造成伤害 → 回合结束不触发归离（④）
    let _ = hand_turn_to(&mut e, 1, 1);
    advance_to(&mut e, Phase::Play);
    let sha = card_by(&e, |c| c.kind == CardKind::Sha);
    give(&mut e, 1, sha);
    play(&mut e, 1, sha, vec![2]);
    drain(&mut e);
    advance_to(&mut e, Phase::End);
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    drain(&mut e);
    assert_eq!(
        flag(&e, 0, "th_guili_extra_turn"),
        0,
        "④ guili does not fire when target dealt damage in their first turn of the round"
    );
    // 第 2 轮座位 1 首回合：未造成伤害 → 回合结束触发归离，曹华回合结束获得额外准备+出牌阶段（①）
    let _ = hand_turn_to(&mut e, 1, 2);
    advance_to(&mut e, Phase::End);
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    drain(&mut e);
    assert_eq!(
        (
            flag(&e, 0, "th_guili_extra_turn"),
            flag(&e, 0, "th_guili_pending_extra"),
        ),
        (1, 1),
        "① target ended first turn of round 2 with 0 damage → guili triggers"
    );
}
