//! L28（雷霆万钧）独立复核 `571d828a-de8d-4c1a-b9cc-e06219f5006b` 提出的 12 项 FAIL 红字在先回归。
//! 每条断言均标注对应上游 `extension.js` / `character/extra.js` / `character/mobile.js` / `character/sp.js` 行号。

use super::l28_thunder_eight_match::*;
use engine_native::catalog::{CardKind, Suit};
use engine_native::event::Phase;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction};

fn clear_seat(e: &mut Engine, seat: u8) {
    let cards: Vec<u32> = e.state_core_mut().player_mut(seat).hand.drain(..).collect();
    let eqs: Vec<u32> = e.state_core_mut().player_mut(seat).equips.drain(..).collect();
    e.state_core_mut().discard.extend(cards);
    e.state_core_mut().discard.extend(eqs);
}

#[test]
fn l28_reviewer_reg_tianshu_chengye_tpys() {
    let p = pack();

    // 1) th_taipingyaoshu `list: []` (`extension.js:8968`) & th_tianshu 仅 phaseUseBegin (`extension.js:6914-6959`)
    let mut e = fresh(&p, 2901, "th_nanhualaoxian");
    let tp = card_named(&e, "th_taipingyaoshu");
    assert!(
        !e.state_core().deck.contains(&tp)
            && !(0..4).any(|s| e.state_core().player(s).hand.contains(&tp)),
        "th_taipingyaoshu must NOT be dealt into initial deck or opening hands (list: [])"
    );
    assert!(
        e.state_core().detached_cards.iter().any(|(c, _)| *c == tp),
        "th_taipingyaoshu must wait off-deck in detached_cards until first th_tianshu trigger"
    );
    assert!(
        matches!(activate(&mut e, 0, "th_tianshu"), CommandOutcome::Rejected(_)),
        "th_tianshu has no enable:'phaseUse' and must reject ActivateSkill (extension.js:6914)"
    );
    e.debug_set_turn(0, Phase::Draw, 1);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = open(&e, "th_tianshu:choose_target");
    let drop = e.state_core().player(0).hand[0];
    answer(&mut e, "th_tianshu:choose_target", vec![pl(2)]);
    answer(&mut e, "th_tianshu:discard_card", vec![cd(drop)]);
    drain(&mut e);
    assert!(
        e.state_core().player(2).equips.contains(&tp),
        "first th_tianshu instantiates off-deck th_taipingyaoshu onto target armor"
    );

    // 2) th_chengye `event.card.isCard` & `onremove` (`extension.js:6208, 6273-6276`)
    let mut e = fresh(&p, 2902, "th_mamidi");
    e.debug_grant(1, "武圣");
    e.debug_set_turn(1, Phase::Play, 1);
    clear_seat(&mut e, 1);
    clear_seat(&mut e, 2);
    let red_tao = card_by(&e, |c| {
        c.kind == CardKind::Tao && matches!(c.suit, Suit::Heart | Suit::Diamond)
    });
    give(&mut e, 1, red_tao);
    play(&mut e, 1, red_tao, vec![2]);
    drain(&mut e);
    assert_eq!(
        flag(&e, 0, "th_chengye_mask"),
        0,
        "converted card (use_as: Sha) must NOT satisfy useCardAfter event.card.isCard (extension.js:6208)"
    );

    let sha = card_by(&e, |c| c.kind == CardKind::Sha);
    take_out(&mut e, sha);
    e.state_core_mut()
        .vm_aux
        .piles
        .entry((0, "th_liujing".to_string()))
        .or_default()
        .push(sha);
    set_flag(&mut e, 0, "th_chengye_mask", 1 << 1);
    set_flag(&mut e, 0, "th_chengye_liujing_count", 1);
    e.assign_general_named(0, "th_caohua").expect("switch general while alive");
    assert!(
        e.state_core().discard.contains(&sha)
            && !e.state_core().vm_aux.piles.contains_key(&(0, "th_liujing".to_string())),
        "alive th_chengye onremove must flush th_liujing to discard pile (extension.js:6273-6276)"
    );
}

#[test]
fn l28_reviewer_reg_caohua_shen_zhouyu() {
    let p = pack();

    // 3a) th_caiyi 仅 phaseJieshuBegin (`extension.js:4421`) + opt3 门控 (`4424-4426`) + 阴0 Damage (`951`) + 阴1 弃牌询问 + 无虚构 forbid_cards
    let mut e = fresh(&p, 2903, "th_caohua");
    assert!(
        matches!(activate(&mut e, 0, "th_caiyi"), CommandOutcome::Rejected(_)),
        "th_caiyi has no enable:'phaseUse' (extension.js:4420)"
    );
    clear_seat(&mut e, 0);
    e.debug_set_turn(0, Phase::Discard, 1);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = open(&e, "th_caiyi:choose_target");
    answer(&mut e, "th_caiyi:choose_target", vec![pl(1)]);
    let _ = open(&e, "th_caiyi:choose_option");
    assert!(
        rejected(&mut e, vec![tag(3)]),
        "initial th_caiyiyang [1,1,1,2] forbids opt 3 until one of 0..=2 is removed"
    );
    answer(&mut e, "th_caiyi:choose_option", vec![tag(1)]);

    // 下一回合触发阴面：先删阴 2，再令 1 号获上上签免疫 Damage，阴 0 必须走 Damage 被免疫且选阴 3 不置 th_caiyi_forbid_cards
    next_turn(&mut e, 2);
    clear_seat(&mut e, 0);
    e.debug_set_turn(0, Phase::Discard, 2);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    answer(&mut e, "th_caiyi:choose_target", vec![pl(1)]);
    answer(&mut e, "th_caiyi:choose_option", vec![tag(2)]); // removes yin opt 2, unlocking yin opt 3
    e.state_core_mut().player_mut(1).face_down = false;

    next_turn(&mut e, 3);
    clear_seat(&mut e, 0);
    e.debug_set_turn(0, Phase::Discard, 3);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    answer(&mut e, "th_caiyi:choose_target", vec![pl(1)]);
    answer(&mut e, "th_caiyi:choose_option", vec![tag(3)]); // random removed yang opt (opt 1 = draw 3)

    // 再到阴面：验证 yin opt 3 无 th_caiyi_forbid_cards，且 yin opt 0 为 Damage（受上上签保护）
    next_turn(&mut e, 4);
    clear_seat(&mut e, 0);
    e.debug_set_turn(0, Phase::Discard, 4);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    answer(&mut e, "th_caiyi:choose_target", vec![pl(1)]);
    answer(&mut e, "th_caiyi:choose_option", vec![tag(3)]); // random removed yin opt (opt 2)
    assert_eq!(
        flag(&e, 1, "th_caiyi_forbid_cards"),
        0,
        "yin opt 3 must NOT fabricate th_caiyi_forbid_cards (extension.js:967-978)"
    );
    e.state_core_mut().player_mut(1).face_down = false;
    next_turn(&mut e, 5); // skip yang turn cleanly:
    clear_seat(&mut e, 0);
    e.debug_set_turn(0, Phase::Discard, 5);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    answer(&mut e, "th_caiyi:choose_target", vec![pl(1)]);
    answer(&mut e, "th_caiyi:choose_option", vec![tag(0)]);
    next_turn(&mut e, 6);
    set_flag(&mut e, 1, "th_tiansuan2_0_prevent_damage", 1);
    let hp1_before = hp(&e, 1);
    clear_seat(&mut e, 0);
    e.debug_set_turn(0, Phase::Discard, 6);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    answer(&mut e, "th_caiyi:choose_target", vec![pl(1)]);
    answer(&mut e, "th_caiyi:choose_option", vec![tag(0)]);
    drain(&mut e);
    assert_eq!(
        hp(&e, 1),
        hp1_before,
        "yin opt 0 must emit Damage (preventable by th_tiansuan2_0), NOT LoseHp (extension.js:951)"
    );

    // 3b) th_guili 插入完整额外回合 (`extension.js:4501-4513` insertPhase)
    let mut e = fresh(&p, 2904, "th_caohua");
    hand_turn_to(&mut e, 0, 1);
    answer(&mut e, "th_guili:choose_target", vec![pl(1)]);
    clear_seat(&mut e, 0);
    // 确保自己的 yang && yin 不触发额外回合
    set_flag(&mut e, 0, "th_caiyi_yang_used_turn", 1);
    set_flag(&mut e, 0, "th_caiyi_yin_used_turn", 1);
    e.debug_set_turn(0, Phase::End, 1);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    drain(&mut e);
    assert_eq!(
        e.state_core().turn.current,
        1,
        "owner's yang&&yin must NOT grant extra turn; control hands to seat 1"
    );
    clear_seat(&mut e, 1);
    let h0_pre_extra = hand(&e, 0);
    advance_to(&mut e, Phase::End);
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    drain(&mut e);
    assert_eq!(
        e.state_core().turn.current,
        0,
        "th_guili grants seat 0 a full extra turn right after seat 1's damage-free turn"
    );
    assert_eq!(
        e.state_core().turn.round,
        1,
        "inserted extra turn must NOT bump round prematurely"
    );
    advance_to(&mut e, Phase::Play);
    assert_eq!(
        hand(&e, 0),
        h0_pre_extra + 2,
        "inserted extra turn executes Draw phase (+2 cards) before Play"
    );
    clear_seat(&mut e, 0);
    advance_to(&mut e, Phase::End);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    drain(&mut e);
    assert_eq!(
        e.state_core().turn.current,
        2,
        "after inserted extra turn ends, turn resumes seamlessly at seat 2"
    );

    // 4) th_shen_zhouyu 默认 ["th_yeyan", "pro_qinyin"] (`extension.js:3670`, `extra.js:8728-8815`)
    let mut e = fresh(&p, 2905, "th_shen_zhouyu");
    assert!(
        has_skill(&e, 0, "pro_qinyin"),
        "th_shen_zhouyu must carry pro_qinyin by default (extension.js:3670)"
    );
    activate(&mut e, 0, "th_yeyan");
    answer(&mut e, "th_yeyan:choose_mode", vec![tag(0)]);
    answer(&mut e, "th_yeyan:choose_targets", vec![pl(1)]);
    drain(&mut e);
    assert_eq!(flag(&e, 0, "th_yeyan_used"), 1);
    let red_c = card_by(&e, |c| matches!(c.suit, Suit::Heart | Suit::Diamond));
    let blk_c = card_by(&e, |c| matches!(c.suit, Suit::Spade | Suit::Club));
    clear_seat(&mut e, 0);
    give(&mut e, 0, red_c);
    give(&mut e, 0, blk_c);
    e.debug_set_hp(0, 1); // 3 cards with hp=1 forces discarding 2 (red_c + blk_c)
    let extra: Vec<u32> = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .filter(|(i, c)| (*i as u32) != red_c && (*i as u32) != blk_c && !c.off_deck)
        .take(1)
        .map(|(i, _)| i as u32)
        .collect();
    for c in extra {
        give(&mut e, 0, c);
    }
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play -> Discard
    answer(&mut e, "discard_to_limit", vec![cd(red_c), cd(blk_c)]);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard -> End
    // Discard -> End: trigger pro_qinyin
    let c = open(&e, "pro_qinyin:choose_mode");
    assert!(
        c.spec.options.contains(&tag(2)),
        "pro_qinyin offers Reset Yeyan (tag 2) when th_yeyan is used (extra.js:8788)"
    );
    let max_hp_before = e.state_core().player(0).max_hp;
    answer(&mut e, "pro_qinyin:choose_mode", vec![tag(2)]);
    assert_eq!(e.state_core().player(0).max_hp, max_hp_before - 1);
    assert_eq!(flag(&e, 0, "th_yeyan_used"), 0);
}

#[test]
fn l28_reviewer_reg_bushi_gushe() {
    let p = pack();

    // 5) th_bushi (`extension.js:4208, 4330, 4349, 4381`): 初始 Spade/Diamond/Club/Heart、24排列、全牌种无限次（含酒）、gain无牌不摸
    let mut e = fresh(&p, 2906, "th_lukai");
    let d1 = card_by(&e, |c| c.kind == CardKind::Jiu && c.suit == Suit::Spade);
    give(&mut e, 0, d1);
    play(&mut e, 0, d1, Vec::new());
    drain(&mut e);
    give(&mut e, 0, d1);
    assert!(
        !matches!(submit(&mut e, 0, GameAction::PlayCard { card: d1, targets: Vec::new() }), CommandOutcome::Rejected(_)),
        "th_bushi default s0=Spade must allow unlimited Jiu uses (extension.js:4349)"
    );
    hand_turn_to(&mut e, 0, 2);
    let opt = open(&e, "th_bushi:choose_posture");
    assert_eq!(opt.spec.options.len(), 24, "th_bushi must allow all 4! = 24 suit permutations (extension.js:4224-4320)");
    // 选择非循环排列 `[Heart, Diamond, Club, Spade]` (设在某索引 > 3) 并测 gain_suit_card 牌堆/弃牌堆均无时不摸牌
    let non_cyclic = opt.spec.options[5]; // rank >= 4
    answer(&mut e, "th_bushi:choose_posture", vec![non_cyclic]);
    let s3_idx = flag(&e, 0, "th_bushi_s3");
    let matches_s3 = |s: Suit| -> bool {
        match s3_idx {
            0 => s == Suit::Spade,
            1 => s == Suit::Heart,
            2 => s == Suit::Club,
            _ => s == Suit::Diamond,
        }
    };
    let cat = e.catalog().clone();
    e.state_core_mut().deck.retain(|&c| !matches_s3(cat.cards[c as usize].suit));
    e.state_core_mut().discard.retain(|&c| !matches_s3(cat.cards[c as usize].suit));
    advance_to(&mut e, Phase::Play);
    clear_seat(&mut e, 0);
    advance_to(&mut e, Phase::End);
    drain(&mut e);
    assert_eq!(
        hand(&e, 0),
        0,
        "th_bushi gain_suit_card must NOT fall back to drawing when suit missing in deck/discard (extension.js:4381-4383)"
    );

    // 6) th_gushe (`extension.js:8219`) 与 rejici (`sp.js:16971`) 严格分账
    let mut e = fresh(&p, 2907, "th_wanglang");
    set_flag(&mut e, 0, "th_gushe_raosuo", 4);
    let mid0 = card_by(&e, |c| c.rank == 3);
    let mid1 = card_by(&e, |c| c.rank == 5);
    give(&mut e, 0, mid0);
    give(&mut e, 1, mid1);
    activate(&mut e, 0, "th_gushe");
    answer(&mut e, "th_gushe:choose_targets", vec![pl(1)]);
    answer(&mut e, "th_gushe:pindian_card", vec![cd(mid0)]);
    answer(&mut e, "th_gushe:target_pindian_card", vec![cd(mid1)]);
    // th_gushe_raosuo=4 不能触发 rejici 加点，王朗 3 < 5 仍输，因此问 owner_discard（非 target_discard）
    let own_disc = open(&e, "th_gushe:owner_discard");
    answer(&mut e, "th_gushe:owner_discard", Vec::new());
    assert_eq!(own_disc.observer, 0);
    assert_eq!(
        flag(&e, 0, "th_gushe_raosuo"),
        5,
        "th_gushe_raosuo must NOT fire rejici rank boost (extension.js:8219, sp.js:16971)"
    );
}

#[test]
fn l28_reviewer_reg_jingxie_shengong() {
    let p = pack();

    // 7) th_jingxie (`extension.js:7429-7803`, `mobile.js:460-575, 15042-15172`)
    let mut e = fresh(&p, 2908, "th_majun");
    clear_seat(&mut e, 0);
    let tp = card_named(&e, "th_taipingyaoshu");
    take_out(&mut e, tp);
    e.state_core_mut().player_mut(0).hand.push(tp);
    assert!(
        matches!(activate(&mut e, 0, "th_jingxie"), CommandOutcome::Rejected(_)),
        "th_jingxie whitelist must reject th_taipingyaoshu (extension.js:7439)"
    );
    let zhuge = card_by(&e, |c| c.kind == CardKind::Zhuge);
    let renwang = card_by(&e, |c| c.kind == CardKind::Renwang);
    give(&mut e, 0, zhuge);
    give(&mut e, 0, renwang);
    activate(&mut e, 0, "th_jingxie");
    answer(&mut e, "th_jingxie:choose_card", vec![cd(zhuge)]);
    play(&mut e, 0, zhuge, Vec::new());
    drain(&mut e);
    assert_eq!(
        e.state_core().attack_range(e.catalog(), 0),
        3,
        "rewrite_zhuge must raise attack range from 1 to 3 (mobile.js:557)"
    );
    activate(&mut e, 0, "th_jingxie");
    answer(&mut e, "th_jingxie:choose_card", vec![cd(renwang)]);
    play(&mut e, 0, renwang, Vec::new());
    drain(&mut e);
    clear_seat(&mut e, 1);
    e.debug_set_turn(1, Phase::Play, 1);
    let heart_sha = card_by(&e, |c| c.kind == CardKind::Sha && c.suit == Suit::Heart);
    give(&mut e, 1, heart_sha);
    let hp0_before = hp(&e, 0);
    let h0_cards: Vec<u32> = e.state_core_mut().player_mut(0).hand.drain(..).collect();
    e.state_core_mut().discard.extend(h0_cards);
    play(&mut e, 1, heart_sha, vec![0]);
    drain(&mut e);
    assert_eq!(
        hp(&e, 0),
        hp0_before,
        "rewrite_renwang must block Heart Sha as well as black Sha (mobile.js:15128)"
    );

    // 8) th_shengong (`extension.js:5163-5502`)：三类各1次、专属card_map、离场destroy+end摸牌
    let mut e = fresh(&p, 2909, "th_puyuan");
    clear_seat(&mut e, 0);
    let w1 = card_by(&e, |c| c.kind == CardKind::Qinggang);
    let w2 = card_by(&e, |c| c.kind == CardKind::Qilin);
    let a1 = card_by(&e, |c| c.kind == CardKind::Armor);
    let h1 = card_by(&e, |c| c.kind == CardKind::HorsePlus);
    for c in [w1, w2, a1, h1] {
        give(&mut e, 0, c);
    }
    activate(&mut e, 0, "th_shengong");
    answer(&mut e, "th_shengong:discard_card", vec![cd(w1)]);
    while e
        .open_choice()
        .is_some_and(|c| c.spec.prompt == "th_shengong:assist")
    {
        answer(&mut e, "th_shengong:assist", Vec::new());
    }
    assert!(
        open(&e, "th_shengong:choose_card").spec.options.len() >= 1,
        "th_shengong must offer exclusive card_map card selection (extension.js:5411)"
    );
    let card_choice = open(&e, "th_shengong:choose_card").spec.options[0];
    let ChoiceOption::Card(forged_w) = card_choice else {
        panic!("expected forged card option");
    };
    answer(&mut e, "th_shengong:choose_card", vec![card_choice]);
    answer(&mut e, "th_shengong:choose_target", vec![pl(1)]);
    assert!(
        e.state_core().player(1).equips.contains(&forged_w),
        "target equips forged exclusive card_map weapon"
    );
    assert_ne!(
        e.catalog().cards[forged_w as usize].name,
        "",
        "forged weapon exists in off-deck catalog"
    );
    // 第二次允许再锻造不同类装备（如防具 a1），但弃置同类的 w2 被拒绝
    activate(&mut e, 0, "th_shengong");
    assert!(
        rejected(&mut e, vec![cd(w2)]),
        "cannot forge a second equip1 in the same turn (extension.js:5188)"
    );
    answer(&mut e, "th_shengong:discard_card", vec![cd(a1)]);
    while e
        .open_choice()
        .is_some_and(|c| c.spec.prompt == "th_shengong:assist")
    {
        answer(&mut e, "th_shengong:assist", Vec::new());
    }
    let a_opt = open(&e, "th_shengong:choose_card").spec.options[0];
    answer(&mut e, "th_shengong:choose_card", vec![a_opt]);
    answer(&mut e, "th_shengong:choose_target", vec![pl(2)]);

    // 拆掉 seat 1 身上的专属装备：进 detached_cards（th_shengong_destroy），结束阶段摸牌（th_shengong_draw）
    let h1_cards: Vec<u32> = e.state_core_mut().player_mut(1).hand.drain(..).collect();
    e.state_core_mut().discard.extend(h1_cards);
    let d1 = card_by(&e, |c| c.kind == CardKind::Dismantle);
    give(&mut e, 0, d1);
    play(&mut e, 0, d1, vec![1]);
    drain(&mut e);
    assert!(
        e.state_core().detached_cards.iter().any(|(c, _)| *c == forged_w)
            && !e.state_core().discard.contains(&forged_w),
        "th_shengong_destroy moves forged equip out of discard pile into detached_cards"
    );
    let h0_pre_end = hand(&e, 0);
    advance_to(&mut e, Phase::End);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    drain(&mut e);
    assert!(
        hand(&e, 0) > h0_pre_end,
        "th_shengong_draw draws at phaseJieshuBegin for destroyed forged cards"
    );
}

#[test]
fn l28_reviewer_reg_tiansuan() {
    let p = pack();

    // 9a) th_tiansuan 加权随机抽签（允许选 tag 0..=4 加权或跳过 tag 5）+ th_tiansuan2 死亡清理 onremove
    let mut e = fresh(&p, 2910, "th_zhouqun");
    let mut outcomes = Vec::new();
    for seed_offset in 0..8u64 {
        let mut e2 = fresh(&p, 2920 + seed_offset, "th_zhouqun");
        activate(&mut e2, 0, "th_tiansuan");
        answer(&mut e2, "th_tiansuan:choose_lot", vec![tag(4)]);
        answer(&mut e2, "th_tiansuan:choose_target", vec![pl(1)]);
        drain(&mut e2);
        outcomes.push(flag(&e2, 0, "th_tiansuan_lot"));
    }
    assert!(
        outcomes.iter().any(|&lot| lot != 5),
        "choosing weight tag 4 must still draw randomly via PRNG, NOT fix outcome=4: {outcomes:?}"
    );

    // th_tiansuan2 onremove：周群死亡时立即清除目标签与标记 (`extension.js:8027-8033`)
    activate(&mut e, 0, "th_tiansuan");
    answer(&mut e, "th_tiansuan:choose_lot", vec![tag(2)]);
    answer(&mut e, "th_tiansuan:choose_target", vec![pl(1)]);
    drain(&mut e);
    assert!(has_skill(&e, 0, "th_tiansuan2"));
    let target_lot_skill_present = ["th_tiansuan2_0", "th_tiansuan2_1", "th_tiansuan2_2", "th_tiansuan2_3", "th_tiansuan2_4"]
        .iter()
        .any(|s| has_skill(&e, 1, s));
    assert!(target_lot_skill_present);
    e.debug_set_hp(0, 1);
    clear_seat(&mut e, 0);
    clear_seat(&mut e, 2);
    clear_seat(&mut e, 3);
    e.debug_set_turn(1, Phase::Play, 1);
    let kill_sha = card_by(&e, |c| c.kind == CardKind::Sha);
    give(&mut e, 1, kill_sha);
    play(&mut e, 1, kill_sha, vec![0]);
    drain(&mut e);
    assert!(!e.state_core().player(0).alive);
    assert!(
        !["th_tiansuan2_0", "th_tiansuan2_1", "th_tiansuan2_2", "th_tiansuan2_3", "th_tiansuan2_4"]
            .iter()
            .any(|s| has_skill(&e, 1, s)),
        "Zhou Qun death must trigger th_tiansuan2 onremove and clear target's lot skills"
    );

    // 9b) th_tiansuan2_3 (damageBegin3 +1) 先于 th_tiansuan2_1/2 (damageBegin4 封顶1) 且火签不额外 +1
    let mut e = fresh(&p, 2911, "th_zhouqun");
    set_flag(&mut e, 1, "th_tiansuan2_3_extra_damage", 1);
    set_flag(&mut e, 1, "th_tiansuan2_1_cap_damage", 1);
    let sha = card_by(&e, |c| c.kind == CardKind::Sha);
    give(&mut e, 0, sha);
    clear_seat(&mut e, 1);
    let hp1 = hp(&e, 1);
    play(&mut e, 0, sha, vec![1]);
    drain(&mut e);
    assert_eq!(
        hp(&e, 1),
        hp1 - 1,
        "damageBegin3 (+1 -> 2) MUST resolve before damageBegin4 (cap to 1) -> net 1 dmg"
    );

    let mut e = fresh(&p, 2912, "th_zhouqun");
    set_flag(&mut e, 1, "th_tiansuan2_fire_nature", 1);
    let fsha = card_by(&e, |c| c.kind == CardKind::FireSha);
    give(&mut e, 0, fsha);
    clear_seat(&mut e, 1);
    let hp1 = hp(&e, 1);
    play(&mut e, 0, fsha, vec![1]);
    drain(&mut e);
    assert_eq!(
        hp(&e, 1),
        hp1 - 1,
        "th_tiansuan2_fire_nature must NOT deal extra +1 fire damage"
    );
}
