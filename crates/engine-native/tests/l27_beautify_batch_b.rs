//! L27 选将美化 6 维实盘闭环测试（Part 2：Batch B 18 项技能/伴生钩子与多技能/修饰器联动）。
//!
//! 覆盖范围（Queue Item 2 Source 9 `L27 选将美化` 后 18 项）：
//! `bolan`（含 `bolan_g`）、`drlt_duorui`、`pro_drlt_duorui`、`xinfu_xionghuo`、`mbwuling`、
//! `sbqingzheng`、`spyanjiao`、`twxiafeng`、`twxingzhui`、`xingchong`、`dcxiaoxi`、`twfengji`、
//! `twharvestinori`、`twzhouhu`、`twzuhuo`、`twzhouzu`、`jishe2`、`new_zhixi_recover`。

use super::l27_beautify_match::*;
use engine_native::catalog::{CardKind, Suit};
use engine_native::event::{DamageNature, Phase};
use engine_native::rvm_bridge::modifiers::{
    eval_card_distance_unlimited, eval_card_target_canceled, eval_card_unrespondable,
    eval_damage_amount, eval_sha_use_limit,
};
use engine_native::GameAction;

#[test]
fn l27_batch_b_active_and_duorui_skills() {
    let p = pack();

    // 1. bolan + bolan_g: Play phase begin skill grant & companion active LoseHp skill grant
    {
        let mut e = fresh(&p, 2720, &[&["bolan_g"], &["bolan"]]);
        let hp0 = hp(&e, 0);
        activate(&mut e, 0, "bolan_g");
        assert!(rejected(&mut e, vec![tag(99)]));
        snapshot_roundtrip(&e, &p, vec![tag(0)]);
        answer(&mut e, "bolan:choose_skill", vec![tag(0)]); // zhiheng
        assert_eq!(hp(&e, 0), hp0 - 1);
        assert_eq!(flag(&e, 0, "bolan_temp_skill"), 1);
        // ④ bolan_g once per turn
        assert!(is_rejected(activate(&mut e, 0, "bolan_g")));

        // Seat 1 Draw -> Play begin -> triggers bolan:choose_skill
        e.debug_set_turn(1, Phase::Draw, 1);
        let _ = submit(&mut e, 1, GameAction::NextPhase);
        snapshot_roundtrip(&e, &p, vec![tag(1)]);
        answer(&mut e, "bolan:choose_skill", vec![tag(1)]); // qiangxi
        drain(&mut e);
        assert_eq!(flag(&e, 1, "bolan_temp_skill"), 2);
    }

    // 2. drlt_duorui + 3. pro_drlt_duorui: Play phase damage -> seal slot & steal skill
    {
        let mut e = fresh(
            &p,
            2721,
            &[&["drlt_duorui", "pro_drlt_duorui", "kuangcai"], &["sbzhaxiang"]],
        );
        let sha1 = card_by(&e, |_, c| c.kind == CardKind::Sha);
        let sha2 = card_by(&e, |id, c| c.kind == CardKind::Sha && id != sha1);
        give(&mut e, 0, sha1);
        give(&mut e, 0, sha2);
        play(&mut e, 0, sha1, vec![1]);
        while let Some(c) = e.open_choice().cloned() {
            match c.spec.prompt {
                "drlt_duorui:choose_slot" | "pro_drlt_duorui:choose_slot" => {
                    snapshot_roundtrip(&e, &p, vec![tag(0)]);
                    answer(&mut e, c.spec.prompt, vec![tag(0)]);
                }
                "drlt_duorui:choose_skill" | "pro_drlt_duorui:choose_skill" => {
                    answer(&mut e, c.spec.prompt, vec![tag(0)]);
                }
                _ => {
                    drain(&mut e);
                    break;
                }
            }
        }
        assert_eq!(flag(&e, 0, "drlt_duorui_sealed_slots_mask") & 1, 1);
        assert!(has_skill(&e, 0, "sbzhaxiang"));
        // ④ Target already has disabled skill -> 2nd Sha does not re-trigger drlt_duorui
        play(&mut e, 0, sha2, vec![1]);
        assert!(e.open_choice().is_none());
    }

    // 4. xinfu_xionghuo: baoli mark transfer, +1 damage passive, Play phase begin modes
    {
        let mut e = fresh(&p, 2722, &[&["xinfu_xionghuo"], &[]]);
        let _ = hand_turn_to(&mut e, 0, 1);
        drain(&mut e);
        assert_eq!(flag(&e, 0, "xinfu_xionghuo_marks"), 3);
        e.debug_set_turn(0, Phase::Play, 1);
        activate(&mut e, 0, "xinfu_xionghuo");
        snapshot_roundtrip(&e, &p, Vec::new());
        // ② Cancel leaves 3 marks
        answer(&mut e, "xinfu_xionghuo:choose_target", Vec::new());
        assert_eq!(flag(&e, 0, "xinfu_xionghuo_marks"), 3);

        activate(&mut e, 0, "xinfu_xionghuo");
        assert!(rejected(&mut e, vec![pl(0)]));
        answer(&mut e, "xinfu_xionghuo:choose_target", vec![pl(1)]);
        assert_eq!(flag(&e, 0, "xinfu_xionghuo_marks"), 2);
        assert_eq!(flag(&e, 1, "xinfu_xionghuo_marked"), 1);
        assert_eq!(
            eval_damage_amount(
                e.state_core(),
                e.catalog(),
                Some(0),
                1,
                None,
                DamageNature::Normal,
                1
            ),
            2
        );

        // Force mode 1 (Fire damage + cannot Sha Seat 0 this turn) when Seat 1 enters Play phase
        set_flag(&mut e, 0, "xinfu_xionghuo_force_mode", 1);
        let sha = card_by(&e, |_, c| c.kind == CardKind::Sha);
        give(&mut e, 1, sha);
        let hp1 = hp(&e, 1);
        e.debug_set_turn(1, Phase::Draw, 1);
        let _ = submit(&mut e, 1, GameAction::NextPhase);
        drain(&mut e);
        assert_eq!(hp(&e, 1), hp1 - 1);
        assert_eq!(flag(&e, 1, "xinfu_xionghuo_no_sha_turn"), 1);
        assert!(eval_card_target_canceled(
            e.state_core(),
            e.catalog(),
            1,
            0,
            sha,
            CardKind::Sha,
            DamageNature::Normal
        ));
        assert!(is_rejected(submit(
            &mut e,
            1,
            GameAction::PlayCard {
                card: sha,
                targets: vec![0]
            }
        )));
    }

    // 5. mbwuling: 5-spirit buffs, up to 2 activations/turn
    {
        let mut e = fresh(&p, 2723, &[&["mbwuling"], &[]]);
        activate(&mut e, 0, "mbwuling");
        snapshot_roundtrip(&e, &p, Vec::new());
        answer(&mut e, "mbwuling:choose_target", Vec::new());
        assert_eq!(flag(&e, 0, "mbwuling_used_turn"), 0);

        activate(&mut e, 0, "mbwuling");
        answer(&mut e, "mbwuling:choose_target", vec![pl(0)]);
        snapshot_roundtrip(&e, &p, vec![tag(0)]);
        answer(&mut e, "mbwuling:choose_order", vec![tag(0)]); // Fire spirit -> card damage +1
        assert_eq!(flag(&e, 0, "mbwuling_spirit"), 1);
        let sha = card_by(&e, |_, c| c.kind == CardKind::Sha);
        assert_eq!(
            eval_damage_amount(
                e.state_core(),
                e.catalog(),
                Some(0),
                1,
                Some(sha),
                DamageNature::Normal,
                1
            ),
            2
        );

        activate(&mut e, 0, "mbwuling");
        answer(&mut e, "mbwuling:choose_target", vec![pl(1)]);
        answer(&mut e, "mbwuling:choose_order", vec![tag(1)]); // Lightning spirit -> immune to Le/Bing
        assert_eq!(flag(&e, 1, "mbwuling_spirit"), 2);
        let le = card_by(&e, |_, c| c.kind == CardKind::Le);
        assert!(eval_card_target_canceled(
            e.state_core(),
            e.catalog(),
            0,
            1,
            le,
            CardKind::Le,
            DamageNature::Normal
        ));
        // ④ 3rd activation in the same turn is rejected (2/turn limit)
        assert!(is_rejected(activate(&mut e, 0, "mbwuling")));
    }

    // 6. sbqingzheng + 7. spyanjiao
    {
        let mut e = fresh(&p, 2724, &[&["spyanjiao"], &["sbqingzheng"]]);
        let s1 = card_by(&e, |_, c| c.suit == Suit::Spade);
        give(&mut e, 0, s1);
        activate(&mut e, 0, "spyanjiao");
        snapshot_roundtrip(&e, &p, Vec::new());
        answer(&mut e, "spyanjiao:choose_suit", Vec::new());
        assert!(!once_used(&e, 0, "spyanjiao"));

        activate(&mut e, 0, "spyanjiao");
        answer(&mut e, "spyanjiao:choose_suit", vec![tag(0)]);
        snapshot_roundtrip(&e, &p, vec![pl(1)]);
        answer(&mut e, "spyanjiao:choose_target", vec![pl(1)]);
        assert!(once_used(&e, 0, "spyanjiao"));
        assert_eq!(flag(&e, 0, "spyanjiao_draw_pending"), 1);

        clear_hand(&mut e, 0);
        clear_hand(&mut e, 1);
        set_flag(&mut e, 1, "sbjianxiong", 1); // need = 3 - 1 = 2 suits, or min(need, suits.len())
        let sp1 = card_by(&e, |_, c| c.suit == Suit::Spade);
        let sp2 = card_by(&e, |id, c| c.suit == Suit::Spade && id != sp1);
        let h1 = card_by(&e, |_, c| c.suit == Suit::Heart);
        give(&mut e, 1, sp1);
        give(&mut e, 1, sp2);
        give(&mut e, 0, h1);
        let hp0 = hp(&e, 0);
        e.debug_set_turn(1, Phase::Draw, 1);
        let _ = submit(&mut e, 1, GameAction::NextPhase);
        snapshot_roundtrip(&e, &p, vec![tag(0)]);
        answer(&mut e, "sbqingzheng:choose_suits", vec![tag(0)]); // Spade (2 cards)
        answer(&mut e, "sbqingzheng:choose_target", vec![pl(0)]);
        answer(&mut e, "sbqingzheng:choose_target_suit", vec![tag(1)]); // Heart (1 card)
        snapshot_roundtrip(&e, &p, vec![tag(1)]);
        answer(&mut e, "sbqingzheng:gain_mark", vec![tag(1)]);
        drain(&mut e);
        assert_eq!(hp(&e, 0), hp0 - 1);
        assert_eq!(flag(&e, 1, "sbjianxiong"), 2);
    }
}

#[test]
fn l27_batch_b_tw_and_companion_skills() {
    let p = pack();

    // 1. twxingzhui + 2. twxiafeng + 3. xingchong + 4. dcxiaoxi
    {
        let mut e = fresh(
            &p,
            2725,
            &[&["twxingzhui", "twxiafeng", "xingchong", "dcxiaoxi"], &[]],
        );
        // twxingzhui is Play-phase active skill
        activate(&mut e, 0, "twxingzhui");
        snapshot_roundtrip(&e, &p, vec![tag(2)]);
        answer(&mut e, "twxingzhui:choose_turns", vec![tag(2)]);
        assert_eq!(flag(&e, 0, "twxingzhui_rem"), 2);

        // Hand turn to Seat 0 -> TurnBegin/Prepare triggers xingchong
        let _ = hand_turn_to(&mut e, 0, 2);
        snapshot_roundtrip(&e, &p, vec![tag(1)]);
        answer(&mut e, "xingchong:choose_draw", vec![tag(1)]);
        let show_c = open(&e, "xingchong:show_cards").spec.options[0];
        snapshot_roundtrip(&e, &p, vec![show_c]);
        answer(&mut e, "xingchong:show_cards", vec![show_c]);
        drain(&mut e);
        assert_eq!(flag(&e, 0, "xingchong_tagged_count"), 1);

        // Seat 0 Draw -> Play begin with baonvezhi=2 -> triggers twxiafeng & dcxiaoxi
        set_flag(&mut e, 0, "baonvezhi", 2);
        set_flag(&mut e, 0, "baonvezhi_marks", 2);
        e.debug_set_turn(0, Phase::Draw, 2);
        let _ = submit(&mut e, 0, GameAction::NextPhase);
        while let Some(c) = e.open_choice().cloned() {
            match c.spec.prompt {
                "twxiafeng:choose_count" => {
                    snapshot_roundtrip(&e, &p, vec![tag(1)]);
                    answer(&mut e, "twxiafeng:choose_count", vec![tag(1)]);
                }
                "dcxiaoxi:choose_count" => {
                    snapshot_roundtrip(&e, &p, vec![tag(1)]);
                    answer(&mut e, "dcxiaoxi:choose_count", vec![tag(1)]);
                }
                "dcxiaoxi:choose_target" => {
                    answer(&mut e, "dcxiaoxi:choose_target", vec![pl(1)]);
                }
                "dcxiaoxi:choose_mode" => {
                    answer(&mut e, "dcxiaoxi:choose_mode", vec![tag(1)]); // deal 1 damage
                }
                _ => {
                    drain(&mut e);
                    break;
                }
            }
        }
        assert_eq!(flag(&e, 0, "twxiafeng_x"), 1);
        assert_eq!(flag(&e, 0, "baonvezhi"), 1);
        assert_eq!(e.state_core().player(0).max_hp, 4);
        assert!(
            eval_sha_use_limit(
                e.state_core(),
                e.catalog(),
                0,
                None,
                CardKind::Sha,
                DamageNature::Normal,
                1
            )
            .1
        );
        let sha = card_by(&e, |_, c| c.kind == CardKind::Sha);
        assert!(eval_card_distance_unlimited(
            e.state_core(),
            e.catalog(),
            0,
            sha,
            CardKind::Sha,
            DamageNature::Normal
        ));
        assert!(eval_card_unrespondable(
            e.state_core(),
            e.catalog(),
            0,
            1,
            sha,
            CardKind::Sha,
            DamageNature::Normal
        ));
    }

    // 5. twfengji + 6. twharvestinori + 7. twzhouhu + 8. twzuhuo + 9. twzhouzu
    {
        let mut e = fresh(
            &p,
            2726,
            &[
                &[
                    "twfengji",
                    "twharvestinori",
                    "twzhouhu",
                    "twzuhuo",
                    "twzhouzu",
                ],
                &[],
            ],
        );
        e.debug_set_hp(0, 3);
        let c1 = card_by(&e, |_, c| c.kind == CardKind::Sha);
        let black = card_by(&e, |id, c| {
            matches!(c.suit, Suit::Spade | Suit::Club) && id != c1
        });
        let red = card_by(&e, |id, c| {
            matches!(c.suit, Suit::Heart | Suit::Diamond) && id != c1 && id != black
        });
        let trick = card_by(&e, |id, c| {
            !c.kind.is_basic_kind()
                && c.kind != CardKind::Nullify
                && id != c1
                && id != black
                && id != red
        });
        give(&mut e, 0, c1);
        give(&mut e, 0, black);
        give(&mut e, 0, red);
        give(&mut e, 0, trick);

        // Seat 0 Draw -> Play begin triggers twfengji:choose_card
        e.debug_set_turn(0, Phase::Draw, 1);
        let _ = submit(&mut e, 0, GameAction::NextPhase);
        snapshot_roundtrip(&e, &p, vec![cd(c1)]);
        answer(&mut e, "twfengji:choose_card", vec![cd(c1)]);
        answer(&mut e, "twfengji:choose_turns", vec![tag(1)]);
        assert_eq!(flag(&e, 0, "twfengji_rem"), 1);

        activate(&mut e, 0, "twharvestinori");
        snapshot_roundtrip(&e, &p, vec![cd(black)]);
        answer(&mut e, "twharvestinori:discard_black", vec![cd(black)]);
        answer(&mut e, "twharvestinori:choose_turns", vec![tag(1)]);
        assert_eq!(flag(&e, 0, "twharvestinori_rem"), 1);

        activate(&mut e, 0, "twzhouhu");
        snapshot_roundtrip(&e, &p, vec![cd(red)]);
        answer(&mut e, "twzhouhu:discard_red", vec![cd(red)]);
        answer(&mut e, "twzhouhu:choose_turns", vec![tag(1)]);
        assert_eq!(flag(&e, 0, "twzhouhu_rem"), 1);

        activate(&mut e, 0, "twzuhuo");
        snapshot_roundtrip(&e, &p, vec![cd(trick)]);
        answer(&mut e, "twzuhuo:discard_nonbasic", vec![cd(trick)]);
        answer(&mut e, "twzuhuo:choose_turns", vec![tag(1)]);
        assert_eq!(flag(&e, 0, "twzuhuo_rem"), 1);

        activate(&mut e, 0, "twzhouzu");
        snapshot_roundtrip(&e, &p, vec![pl(1)]);
        answer(&mut e, "twzhouzu:choose_target", vec![pl(1)]);
        answer(&mut e, "twzhouzu:choose_turns", vec![tag(1)]);
        assert_eq!(flag(&e, 0, "twzhouzu_rem"), 1);

        // Seat 0 End phase end resolves twharvestinori (+2 cards), twzhouhu (+1 HP -> 4), twzuhuo (shield=1), twzhouzu (1 Thunder damage)
        let hp1 = hp(&e, 1);
        e.debug_set_turn(0, Phase::End, 1);
        let _ = submit(&mut e, 0, GameAction::NextPhase);
        drain(&mut e);
        assert_eq!(hp(&e, 0), 4);
        assert_eq!(hp(&e, 1), hp1 - 1);
        assert_eq!(flag(&e, 0, "twzuhuo_shields"), 1);
        assert_eq!(
            eval_damage_amount(
                e.state_core(),
                e.catalog(),
                Some(1),
                0,
                None,
                DamageNature::Normal,
                1
            ),
            0
        );

        // Hand turn to Seat 0 Prepare resolves twfengji (twfengji_rem -> 0)
        let _ = hand_turn_to(&mut e, 0, 2);
        drain(&mut e);
        assert_eq!(flag(&e, 0, "twfengji_rem"), 0);
    }

    // 10. jishe2 + 11. new_zhixi_recover companion cleanup at End phase end
    {
        let mut e = fresh(
            &p,
            2727,
            &[&["jishe2", "new_zhixi_recover", "new_zhixi_mark2"], &[]],
        );
        set_flag(&mut e, 0, "jishe2", 2);
        set_flag(&mut e, 0, "new_zhixi", 1);
        set_flag(&mut e, 0, "new_zhixi_mark2", 1);
        assert_eq!(e.state_core().hand_limit(e.catalog(), 0), 3);
        e.debug_set_turn(0, Phase::End, 1);
        let _ = submit(&mut e, 0, GameAction::NextPhase); // Phase::End end
        drain(&mut e);
        assert_eq!(flag(&e, 0, "jishe2"), 0);
        assert_eq!(flag(&e, 0, "new_zhixi"), 0);
        assert_eq!(flag(&e, 0, "new_zhixi_mark2"), 0);
        assert_eq!(e.state_core().hand_limit(e.catalog(), 0), 5);
    }
}
