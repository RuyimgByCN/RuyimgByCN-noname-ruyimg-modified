//! Item 6 (Track P3-B, Part 2): Real-event parity tests for the remaining `L11OwnNpc` skills:
//! - `txhj_beiming` (`txhj_niutouhuanxiang`, `txhj_mamianhuanxiang`) & `txhj_wangzu` (`txhj_duanwei`, `txhj_songxian`, `txhj_zumao`)
//! - `txhj_juhun` (`txhj_hunduntongren`, `txhj_tongrenjumo`), `txhj_zhuhun` & `txhj_wangyuan` & `txhj_shiyuan` (`txhj_huanyanzhangjiaoshi`, `txhj_changyuanshibing`)
//! - `txhj_kuangbao` (`txhj_tianjijushou`), `txhj_guimen` (`txhj_huangjinyushix`), `txhj_zhouzu` (`txhj_huangjinxinjun`, `txhj_guanhai`)
//! - `txhj_yaoshu` (`txhj_huangjinyaoshi`), `txhj_zhoufa` (`txhj_huangjinyushi`), `txhj_renwang` & `txhj_renfang` (`txhj_huanyanzhangliang`)
//! - `txhj_dizhou` & `txhj_didun` (`txhj_huanyanzhangbao`), `txhj_zhoulei` (`txhj_huanyanzhangjiaolei`)
//! - `txhj_guiyan` (`txhj_huanyanzhangjiaohuo`), `txhj_xiefeng` (`txhj_huanyanzhangjiaofeng`).

use super::*;
use super::l11_own_npc_fixes::{npc_has_skill, npc_play_card, npc_take_card};
use engine_native::catalog::Suit;
use engine_native::event::DamageNature;
use engine_native::LogEvent;

#[test]
fn l11_own_npc_beiming_wangzu_juhun_shiyuan_wangyuan_and_zhuhun() {
    let verified = taixu_verified();

    // 1. `txhj_beiming` (`txhj_niutouhuanxiang`) & `txhj_wangzu` (`txhj_duanwei`, `txhj_songxian`, `txhj_zumao`):
    //    Killer discards all hand cards on kill; off-turn Play death aborts killer's Play phase immediately!
    {
        let mut e = start_locked_engine(2026_960, 3, LockedStartMode::Prototype, verified).unwrap();
        e.assign_general_named(0, "txhj_huangyueying").unwrap();
        e.assign_general_named(1, "txhj_niutouhuanxiang").unwrap();
        e.assign_general_named(2, "txhj_duanwei").unwrap();
        assert!(npc_has_skill(&e, 1, "txhj_beiming"));
        assert!(npc_has_skill(&e, 2, "txhj_wangzu"));
        advance_to_play(&mut e, 0);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        e.debug_clear_hand(2);
        e.debug_set_hp(1, 1);
        e.debug_set_hp(2, 1);
        let duel1 = npc_take_card(&mut e, |d| d.kind == CardKind::Duel && d.name != "txhj_jiedao");
        let extra = npc_take_card(&mut e, |_| true);
        e.state_core_mut().player_mut(0).hand.push(extra);
        let out_duel = npc_play_card(&mut e, 0, duel1, vec![1]);
        assert!(!matches!(out_duel, CommandOutcome::Rejected(_)), "duel1: {out_duel:?}");
        if let Some(c) = e.open_choice().cloned() {
            let _ = answer_choice(&mut e, c.observer, vec![ChoiceOption::Pass]);
        }
        drain_to_pass(&mut e);
        assert!(!e.state_core().player(1).alive, "seat 1 died");
        assert!(e.state_core().player(0).hand.is_empty(), "txhj_beiming must force killer to discard all hand");

        // Now kill seat 2 (`txhj_duanwei`, `txhj_wangzu`) with Duel during seat 0's Play phase -> aborts seat 0's Play phase!
        let duel2 = npc_take_card(&mut e, |d| d.kind == CardKind::Duel && d.name != "txhj_jiedao");
        let out_duel2 = npc_play_card(&mut e, 0, duel2, vec![2]);
        assert!(!matches!(out_duel2, CommandOutcome::Rejected(_)), "duel2: {out_duel2:?}");
        if let Some(c) = e.open_choice().cloned() {
            let _ = answer_choice(&mut e, c.observer, vec![ChoiceOption::Pass]);
        }
        drain_to_pass(&mut e);
        assert!(!e.state_core().player(2).alive, "seat 2 died");
        assert_ne!(e.state_core().turn.phase, Phase::Play, "txhj_wangzu must end killer Play phase");
    }

    // 2. `txhj_juhun` (`txhj_hunduntongren`): end turn (`PhaseAfter` arms), ally dies -> revived hp=3 & draw=3 on owner's next TurnBegin!
    {
        let mut e = start_locked_engine(2026_961, 4, LockedStartMode::Prototype, verified).unwrap();
        e.state_core_mut().mode = GameMode::Versus {
            teams: vec![vec![0, 2], vec![1, 3]],
            substitutes: vec![vec![], vec![]],
        };
        e.assign_general_named(0, "txhj_hunduntongren").unwrap();
        e.assign_general_named(1, "txhj_huangyueying").unwrap();
        e.assign_general_named(2, "txhj_yecha").unwrap();
        e.assign_general_named(3, "txhj_machao").unwrap();
        assert!(npc_has_skill(&e, 0, "txhj_juhun"));
        e.debug_set_turn(0, Phase::End, 1);
        let _ = submit(&mut e, 0, GameAction::NextPhase); // seat 0 TurnEnded -> TurnPhaseAfter arms juhun -> seat 1 TurnBegin!
        drain_to_pass(&mut e);
        // Seat 1 kills seat 2 (seat 0's ally) via Duel during seat 1's Play phase (avoiding bazhen Sha dodge):
        advance_to_play(&mut e, 1);
        e.debug_clear_hand(1);
        e.debug_clear_hand(2);
        e.debug_set_hp(2, 1);
        let duel = npc_take_card(&mut e, |d| d.kind == CardKind::Duel && d.name != "txhj_jiedao");
        let _ = npc_play_card(&mut e, 1, duel, vec![2]);
        drain_to_pass(&mut e);
        assert!(!e.state_core().player(2).alive, "ally seat 2 died while juhun armed");

        // Advance to seat 0's next TurnBegin -> revives ally seat 2 at hp=3 and draws 3!
        e.debug_set_turn(3, Phase::End, 1);
        let _ = submit(&mut e, 3, GameAction::NextPhase);
        while let Some(c) = e.open_choice().cloned() {
            let _ = answer_choice(&mut e, c.observer, vec![]);
        }
        assert!(e.state_core().player(2).alive, "txhj_juhun must revive dead ally at owner TurnBegin");
        assert_eq!(e.state_core().player(2).hp, 3, "txhj_juhun revives to hp 3");
        assert_eq!(e.state_core().player(2).hand.len(), 3, "txhj_juhun draws 3 cards on revive");
    }

    // 3. `txhj_shiyuan`, `txhj_wangyuan`, `txhj_zhuhun` (`txhj_huanyanzhangjiaoshi`):
    //    - `txhj_shiyuan` dying skips rescue -> immediate Death; +1 damage dealt; killing enemy revives them as ally `txhj_changyuanshibing` (hp 3/3, draw 4)!
    //    - `txhj_wangyuan`: any death gives owner +1 maxHp, recover 1, +1 elemental bonus!
    //    - `txhj_zhuhun`: dead ally queued -> active revives as `txhj_changyuanshibing` (3/3, draw 4)!
    {
        let mut e = start_locked_engine(2026_962, 4, LockedStartMode::Prototype, verified).unwrap();
        e.state_core_mut().mode = GameMode::Versus {
            teams: vec![vec![0, 2], vec![1, 3]],
            substitutes: vec![vec![], vec![]],
        };
        e.assign_general_named(0, "txhj_huanyanzhangjiaoshi").unwrap();
        e.assign_general_named(1, "txhj_huangyueying").unwrap();
        e.assign_general_named(2, "txhj_yecha").unwrap();
        e.assign_general_named(3, "txhj_changyuanshibing").unwrap();
        for sk in ["txhj_shiyuan", "txhj_wangyuan", "txhj_zhuhun"] {
            assert!(npc_has_skill(&e, 0, sk), "seat 0 missing {sk}");
        }
        // Seat 3 (`txhj_changyuanshibing`, hp=2 < seat 1 hp=3) has a Tao in hand, yet on lethal `leiji` (+`txhj_shiyuan`) it MUST die immediately without DyingSave ask!
        advance_to_play(&mut e, 0);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        e.debug_clear_hand(2);
        e.debug_clear_hand(3);
        let tao = npc_take_card(&mut e, |d| d.kind == CardKind::Tao);
        e.state_core_mut().player_mut(3).hand.push(tao);
        e.debug_set_hp(1, 3);
        e.debug_set_hp(3, 2);
        e.debug_set_hp(0, 5); // below max_hp=12
        let wz = npc_take_card(&mut e, |d| d.kind == CardKind::Wuzhong && d.name == "wuzhong");
        let _ = npc_play_card(&mut e, 0, wz, vec![]);
        // `leiji` picks lowest-HP enemy seat 3 (2+1=3 dmg) -> Dying -> txhj_shiyuan skips rescue -> Death!
        // `txhj_wangyuan` triggers (+1 max_hp -> 13, recover 1 -> 6, mark -> 1) AND `txhj_shiyuan` converts & revives seat 3 as ally `txhj_changyuanshibing`!
        drain_to_pass(&mut e);
        assert!(e.state_core().player(3).alive, "txhj_shiyuan must revive slain enemy as txhj_changyuanshibing");
        assert!(e.state_core().is_friend_of(e.catalog(), 0, 3), "revived txhj_changyuanshibing joins killer side");
        assert_eq!(e.state_core().player(3).max_hp, 3);
        assert_eq!(e.state_core().player(3).hp, 3);
        assert_eq!(e.state_core().player(3).hand.len(), 4);
        assert_eq!(e.state_core().player(0).max_hp, 13, "txhj_wangyuan gains +1 max_hp");
        assert_eq!(e.state_core().player(0).hp, 6, "txhj_wangyuan recovers 1 hp");

        // Now seat 1 kills ally seat 2 (`txhj_yecha`) via Duel -> seat 0 `txhj_zhuhun` queues seat 2 -> active revives seat 2 as `txhj_changyuanshibing`!
        advance_to_play(&mut e, 1);
        e.debug_clear_hand(1);
        e.debug_clear_hand(2);
        e.debug_set_hp(2, 1);
        let duel_2 = npc_take_card(&mut e, |d| d.kind == CardKind::Duel && d.name != "txhj_jiedao");
        let _ = npc_play_card(&mut e, 1, duel_2, vec![2]);
        drain_to_pass(&mut e);
        assert!(!e.state_core().player(2).alive);
        assert_eq!(e.state_core().player(0).max_hp, 14, "txhj_wangyuan mark now 2");

        // Seat 0 uses FireSha (elemental!) on enemy seat 1 -> 1 (base) + 1 (shiyuan) + 2 (wangyuan mark=2) = 4 Fire damage!
        advance_to_play(&mut e, 0);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        e.debug_set_hp(3, 10); // keep seat 3 alive if leiji targets lowest-HP hostile
        e.debug_set_hp(1, 10);
        e.debug_set_hujia(1, 0);
        let fsha = npc_take_card(&mut e, |d| d.kind == CardKind::FireSha);
        let log_fsha = e.log().len();
        let _ = npc_play_card(&mut e, 0, fsha, vec![1]);
        drain_to_pass(&mut e);
        assert!(
            e.log()[log_fsha..].iter().any(|ev| matches!(ev, LogEvent::DamageDealt { target: 1, amount: 4, card: Some(c), .. } if *c == fsha)),
            "shiyuan(+1)+wangyuan(+2) on FireSha = 4 total dmg"
        );
        assert_eq!(e.state_core().player(1).hp, 1, "leiji(+5) + FireSha(+4) = 9 elemental dmg");

        // Activate `txhj_zhuhun` to revive dead ally seat 2 as `txhj_changyuanshibing` (3/3, draw 4)!
        let zh_id = e.catalog().skill_by_name("txhj_zhuhun").unwrap();
        let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: zh_id });
        assert!(!matches!(out, CommandOutcome::Rejected(_)), "txhj_zhuhun activate: {out:?}");
        let ch = e.open_choice().cloned().expect("txhj_zhuhun:choose_target");
        assert_eq!(ch.spec.prompt, "txhj_zhuhun:choose_target");
        let snap_zh = encode_archive(&e.save());
        let mut e_zh = restore_locked_engine(&decode_archive(&snap_zh).unwrap(), verified).unwrap();
        let _ = answer_choice(&mut e, 0, vec![ChoiceOption::Player(2)]);
        let _ = answer_choice(&mut e_zh, 0, vec![ChoiceOption::Player(2)]);
        drain_to_pass(&mut e);
        drain_to_pass(&mut e_zh);
        assert_eq!(e.state_hash(), e_zh.state_hash());
        assert!(e.state_core().player(2).alive);
        assert_eq!(e.state_core().player(2).max_hp, 3);
        assert_eq!(e.state_core().player(2).hand.len(), 4);

        // `txhj_shiyuan` dying-save skip MUST be blocked by `txhj_feilong_skill_blank` & `rechanyuan`:
        let mut eb = start_locked_engine(2026_9610, 2, LockedStartMode::Prototype, verified).unwrap();
        eb.assign_general_named(0, "txhj_huangyueying").unwrap();
        eb.assign_general_named(1, "txhj_changyuanshibing").unwrap();
        advance_to_play(&mut eb, 0);
        eb.debug_clear_hand(0);
        eb.debug_clear_hand(1);
        eb.debug_set_hp(1, 1);
        let tao_save = npc_take_card(&mut eb, |d| d.kind == CardKind::Tao);
        eb.state_core_mut().player_mut(1).hand.push(tao_save);
        eb.debug_grant(1, "txhj_feilong_skill_blank");
        let sha_k = npc_take_card(&mut eb, |d| d.kind == CardKind::Sha);
        let _ = npc_play_card(&mut eb, 0, sha_k, vec![1]);
        let ask_d = eb.open_choice().cloned().expect("txhj_feilong_skill_blank must allow DyingSave Tao prompt");
        assert_eq!(ask_d.observer, 1);
        assert!(ask_d.spec.options.contains(&ChoiceOption::Card(tao_save)));
        let _ = answer_choice(&mut eb, 1, vec![ChoiceOption::Card(tao_save)]);
        drain_to_pass(&mut eb);
        assert!(eb.state_core().player(1).alive && eb.state_core().player(1).hp == 1);

        // `l10_rechanyuan` (`hp == 1`) must also seal `txhj_shiyuan` so DyingSave Tao prompt opens:
        let mut ec = start_locked_engine(2026_9611, 2, LockedStartMode::Prototype, verified).unwrap();
        ec.assign_general_named(0, "txhj_huangyueying").unwrap();
        ec.assign_general_named(1, "txhj_changyuanshibing").unwrap();
        advance_to_play(&mut ec, 0);
        ec.debug_clear_hand(0);
        ec.debug_clear_hand(1);
        ec.debug_set_hp(1, 1);
        let tao_c = npc_take_card(&mut ec, |d| d.kind == CardKind::Tao);
        ec.state_core_mut().player_mut(1).hand.push(tao_c);
        ec.state_core_mut().vm_aux.flags.insert((1, "l10_rechanyuan".to_string()), 1);
        let sha_c = npc_take_card(&mut ec, |d| d.kind == CardKind::Sha);
        let _ = npc_play_card(&mut ec, 0, sha_c, vec![1]);
        let ask_c = ec.open_choice().cloned().expect("rechanyuan (hp=1) must seal txhj_shiyuan and allow DyingSave Tao prompt");
        assert_eq!(ask_c.observer, 1);
        assert!(ask_c.spec.options.contains(&ChoiceOption::Card(tao_c)));
        let _ = answer_choice(&mut ec, 1, vec![ChoiceOption::Card(tao_c)]);
        drain_to_pass(&mut ec);
        assert!(ec.state_core().player(1).alive && ec.state_core().player(1).hp == 1);
    }
}

#[test]
fn l11_own_npc_kuangbao_guimen_mahou_renwang_renfang_dizhou_didun_zhoulei_guiyan_xiefeng() {
    let verified = taixu_verified();

    // 4. `txhj_kuangbao`: 7 rounds (rounds 2..=8) add=1..7 -> bonus=1 -> +1 damage; Dying resets add=0 & debuff=1!
    {
        let mut e = start_locked_engine(2026_963, 2, LockedStartMode::Prototype, verified).unwrap();
        e.assign_general_named(0, "txhj_tianjijushou").unwrap();
        e.assign_general_named(1, "txhj_huangyueying").unwrap();
        e.debug_set_hp(1, 20);
        // Trigger 7 real round starts cleanly across rounds 2..=8 via seat 1 TurnEnd -> seat 0 TurnBegin:
        for round in 1..=7u32 {
            e.debug_clear_hand(1);
            e.debug_set_turn(1, Phase::End, round);
            let _ = submit(&mut e, 1, GameAction::NextPhase);
            drain_to_pass(&mut e);
        }
        assert_eq!(
            e.state_core().vm_aux.flags.get(&(0, "l11_kuangbao_bonus".to_string())).copied().unwrap_or(0),
            1,
            "txhj_kuangbao must gain +1 damage bonus after 7 rounds"
        );
        advance_to_play(&mut e, 0);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        e.debug_set_hp(1, 4);
        let sha = npc_take_card(&mut e, |d| d.kind == CardKind::Sha);
        let _ = npc_play_card(&mut e, 0, sha, vec![1]);
        drain_to_pass(&mut e);
        assert_eq!(e.state_core().player(1).hp, 2, "txhj_kuangbao bonus 1 deals 2 damage");
        // Dying resets add=0 and sets debuff=1 (skips next round's add)!
        advance_to_play(&mut e, 1);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        e.debug_set_hp(0, 1);
        let tao = npc_take_card(&mut e, |d| d.kind == CardKind::Tao);
        e.state_core_mut().player_mut(0).hand.push(tao);
        let sha_kb = npc_take_card(&mut e, |d| d.kind == CardKind::Sha);
        let _ = npc_play_card(&mut e, 1, sha_kb, vec![0]);
        if let Some(c) = e.open_choice().cloned() {
            assert_eq!(c.spec.prompt, "dying_save");
            let _ = answer_choice(&mut e, 0, vec![ChoiceOption::Card(tao)]);
        }
        drain_to_pass(&mut e);
        assert_eq!(e.state_core().vm_aux.flags.get(&(0, "l11_kuangbao_debuff".to_string())).copied().unwrap_or(0), 1);
        e.debug_set_turn(1, Phase::End, 9);
        let _ = submit(&mut e, 1, GameAction::NextPhase);
        drain_to_pass(&mut e);
        assert_eq!(
            e.state_core().vm_aux.flags.get(&(0, "l11_kuangbao_add".to_string())).copied().unwrap_or(0),
            0,
            "txhj_kuangbao Dying debuff must block next round add"
        );
    }

    // 5. `txhj_guimen` (`txhj_huangjinyushix`): discarding Spade in Discard triggers OrdinaryJudge; |rank diff| <= 1 deals 2 Thunder!
    {
        let mut e = start_locked_engine(2026_964, 2, LockedStartMode::Prototype, verified).unwrap();
        e.assign_general_named(0, "txhj_huangjinyushix").unwrap();
        e.assign_general_named(1, "txhj_huangyueying").unwrap();
        assert!(npc_has_skill(&e, 0, "txhj_guimen"));
        e.debug_set_turn(0, Phase::Play, 1);
        e.debug_clear_hand(0);
        e.debug_set_hp(0, 1);
        e.debug_set_hp(1, 5);
        let sp6 = npc_take_card(&mut e, |d| d.suit == Suit::Spade && d.rank == 6);
        let non_sp = npc_take_card(&mut e, |d| d.suit != Suit::Spade);
        e.state_core_mut().player_mut(0).hand.extend([sp6, non_sp]);
        let _j5 = e.debug_set_deck_top_by(|d| d.rank == 5); // |6 - 5| = 1 -> hit!
        let _ = submit(&mut e, 0, GameAction::NextPhase); // Play -> Discard -> prompts discard_to_limit
        let dch = e.open_choice().cloned().expect("discard_to_limit");
        assert_eq!(dch.spec.prompt, "discard_to_limit");
        let _ = answer_choice(&mut e, 0, vec![ChoiceOption::Card(sp6)]);
        let gch = e.open_choice().cloned().expect("txhj_guimen:choose_target");
        assert_eq!(gch.spec.prompt, "txhj_guimen:choose_target");
        let snap_gm = encode_archive(&e.save());
        let mut e_gm = restore_locked_engine(&decode_archive(&snap_gm).unwrap(), verified).unwrap();
        let _ = answer_choice(&mut e, 0, vec![ChoiceOption::Player(1)]);
        let _ = answer_choice(&mut e_gm, 0, vec![ChoiceOption::Player(1)]);
        drain_to_pass(&mut e);
        drain_to_pass(&mut e_gm);
        assert_eq!(e.state_hash(), e_gm.state_hash());
        assert_eq!(e.state_core().player(1).hp, 3, "txhj_guimen must deal 2 Thunder dmg");
    }

    // 6. `txhj_zhouzu`, `txhj_yaoshu`, `txhj_zhoufa`: interactive mahou skills + expiry + NGSA save/restore!
    {
        let mut e = start_locked_engine(2026_965, 2, LockedStartMode::Prototype, verified).unwrap();
        e.assign_general_named(0, "txhj_huangjinyaoshi").unwrap(); // txhj_yaoshu
        e.assign_general_named(1, "txhj_huangjinyushi").unwrap(); // txhj_zhoufa
        advance_to_play(&mut e, 0);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        e.debug_set_hp(0, 6);
        e.debug_set_hp(1, 6);
        let ys_id = e.catalog().skill_by_name("txhj_yaoshu").unwrap();
        let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: ys_id });
        let _ = answer_choice(&mut e, 0, vec![ChoiceOption::Player(1)]);
        // Mid-duration NGSA save/restore!
        let snap_dur = encode_archive(&e.save());
        let mut e_res = restore_locked_engine(&decode_archive(&snap_dur).unwrap(), verified).unwrap();
        let _ = answer_choice(&mut e, 0, vec![ChoiceOption::Tag(0)]); // 1回合
        let _ = answer_choice(&mut e_res, 0, vec![ChoiceOption::Tag(0)]);
        drain_to_pass(&mut e);
        drain_to_pass(&mut e_res);
        assert_eq!(e.state_hash(), e_res.state_hash());

        // Advance to end of turn -> yaoshu_mahou triggers -> target 1 gains 1 stack of txhj_yaoshu_effect!
        e.debug_set_turn(0, Phase::End, 1);
        let _ = submit(&mut e, 0, GameAction::NextPhase);
        drain_to_pass(&mut e);
        assert_eq!(
            e.state_core().vm_aux.flags.get(&(1, "l11_yaoshu_effect_stacks".to_string())).copied().unwrap_or(0),
            1
        );
        // On seat 1 Play, seat 1 activates `txhj_zhoufa` (dur=1 -> 1 Thunder dmg on TurnPhaseEnd), and plays a Sha which MUST be nullified by `txhj_yaoshu_effect`!
        advance_to_play(&mut e, 1);
        let sha_blocked = npc_take_card(&mut e, |d| d.kind == CardKind::Sha);
        let _ = npc_play_card(&mut e, 1, sha_blocked, vec![0]);
        drain_to_pass(&mut e);
        assert_eq!(e.state_core().player(0).hp, 6, "txhj_yaoshu must nullify target card");
        let zf_id = e.catalog().skill_by_name("txhj_zhoufa").unwrap();
        let _ = submit(&mut e, 1, GameAction::ActivateSkill { skill: zf_id });
        let _ = answer_choice(&mut e, 1, vec![ChoiceOption::Player(0)]);
        let _ = answer_choice(&mut e, 1, vec![ChoiceOption::Tag(0)]); // 1回合 -> 1 Thunder dmg
        drain_to_pass(&mut e);
        e.debug_set_turn(1, Phase::End, 2);
        let _ = submit(&mut e, 1, GameAction::NextPhase);
        drain_to_pass(&mut e);
        assert_eq!(e.state_core().player(0).hp, 5, "txhj_zhoufa must deal dur (1) Thunder dmg");

        // `txhj_yaoshu` effect stack MUST nullify dying `【桃】` saves (`txhj_huangjinyaoshi`)!
        let mut ey = start_locked_engine(2026_9650, 2, LockedStartMode::Prototype, verified).unwrap();
        ey.assign_general_named(0, "txhj_huangjinyaoshi").unwrap();
        ey.assign_general_named(1, "txhj_huangyueying").unwrap();
        advance_to_play(&mut ey, 0);
        ey.debug_clear_hand(0);
        ey.debug_clear_hand(1);
        let ys_id = ey.catalog().skill_by_name("txhj_yaoshu").unwrap();
        let _ = submit(&mut ey, 0, GameAction::ActivateSkill { skill: ys_id });
        let _ = answer_choice(&mut ey, 0, vec![ChoiceOption::Player(1)]);
        let _ = answer_choice(&mut ey, 0, vec![ChoiceOption::Tag(0)]); // 1回合 -> 1 stack
        drain_to_pass(&mut ey);
        ey.debug_set_turn(0, Phase::End, 1);
        let _ = submit(&mut ey, 0, GameAction::NextPhase);
        drain_to_pass(&mut ey);
        assert_eq!(
            ey.state_core().vm_aux.flags.get(&(1, "l11_yaoshu_effect_stacks".to_string())).copied().unwrap_or(0),
            1
        );
        advance_to_play(&mut ey, 0);
        ey.debug_clear_hand(0);
        ey.debug_clear_hand(1);
        ey.debug_set_hp(1, 1);
        let tao1 = npc_take_card(&mut ey, |d| d.kind == CardKind::Tao);
        let tao2 = npc_take_card(&mut ey, |d| d.kind == CardKind::Tao);
        ey.state_core_mut().player_mut(1).hand.extend([tao1, tao2]);
        let sha_l = npc_take_card(&mut ey, |d| d.kind == CardKind::Sha);
        let log_y = ey.log().len();
        let _ = npc_play_card(&mut ey, 0, sha_l, vec![1]);
        // 1st dying Tao is nullified (stack 1 -> 0); 2nd dying Tao saves seat 1 (`hp = 1`):
        let d1 = ey.open_choice().cloned().expect("1st dying Tao ask");
        assert_eq!(d1.observer, 1);
        let _ = answer_choice(&mut ey, 1, vec![ChoiceOption::Card(tao1)]);
        assert!(
            ey.log()[log_y..].iter().any(|ev| matches!(ev, LogEvent::CardNullified { player: 1, card } if *card == tao1)),
            "txhj_yaoshu_effect must nullify dying Tao"
        );
        assert_eq!(
            ey.state_core().vm_aux.flags.get(&(1, "l11_yaoshu_effect_stacks".to_string())).copied().unwrap_or(0),
            0,
            "dying Tao must consume txhj_yaoshu_effect stack"
        );
        let d2 = ey.open_choice().cloned().expect("seat 1 still dying -> 2nd dying Tao ask");
        assert_eq!(d2.observer, 1);
        let _ = answer_choice(&mut ey, 1, vec![ChoiceOption::Card(tao2)]);
        drain_to_pass(&mut ey);
        assert!(ey.state_core().player(1).alive && ey.state_core().player(1).hp == 1);

        // Also test `txhj_zhouzu` (`txhj_guanhai`): dur=1 -> target discards 1 + 1 Thunder dmg!
        let mut ez = start_locked_engine(2026_966, 2, LockedStartMode::Prototype, verified).unwrap();
        ez.assign_general_named(0, "txhj_guanhai").unwrap();
        ez.assign_general_named(1, "txhj_huangjinmengqi").unwrap();
        advance_to_play(&mut ez, 0);
        ez.debug_clear_hand(1);
        ez.debug_set_hp(1, 4);
        let td = npc_take_card(&mut ez, |d| d.kind == CardKind::Shan);
        ez.state_core_mut().player_mut(1).hand.push(td);
        let zz_id = ez.catalog().skill_by_name("txhj_zhouzu").unwrap();
        let _ = submit(&mut ez, 0, GameAction::ActivateSkill { skill: zz_id });
        let _ = answer_choice(&mut ez, 0, vec![ChoiceOption::Player(1)]);
        let _ = answer_choice(&mut ez, 0, vec![ChoiceOption::Tag(0)]);
        drain_to_pass(&mut ez);
        ez.debug_set_turn(0, Phase::End, 1);
        let _ = submit(&mut ez, 0, GameAction::NextPhase);
        let zd = ez.open_choice().cloned().expect("txhj_zhouzu:discard_cards");
        assert_eq!(zd.spec.prompt, "txhj_zhouzu:discard_cards");
        let _ = answer_choice(&mut ez, 1, vec![ChoiceOption::Card(td)]);
        drain_to_pass(&mut ez);
        assert!(ez.state_core().player(1).hand.is_empty());
        assert_eq!(ez.state_core().player(1).hp, 3, "txhj_zhouzu must discard dur and deal 1 Thunder dmg");
    }

    // 7. `txhj_renwang` + `txhj_renfang` (`txhj_huanyanzhangliang`):
    //    Used card moves into `"txhj_renwang"` pile -> hand limit +len & sha limit +ceil(len/5); 36 "方" -> `txhj_renfang` deals 36 Thunder dmg!
    {
        let mut e = start_locked_engine(2026_967, 2, LockedStartMode::Prototype, verified).unwrap();
        e.assign_general_named(0, "txhj_huanyanzhangliang").unwrap();
        e.assign_general_named(1, "txhj_huangyueying").unwrap();
        advance_to_play(&mut e, 0);
        e.debug_clear_hand(0);
        e.debug_set_hp(0, 1);
        let tao = npc_take_card(&mut e, |d| d.kind == CardKind::Tao);
        let _ = npc_play_card(&mut e, 0, tao, vec![]);
        drain_to_pass(&mut e);
        assert_eq!(
            e.state_core().vm_aux.piles.get(&(0, "txhj_renwang".to_string())).map_or(0, |v| v.len()),
            1,
            "txhj_renwang must collect used card entering discard into pile"
        );
        // hp=2 + 1 fang = hand limit 3; sha limit 1 + ceil(1/5) = 2!
        assert_eq!(e.state_core().hand_limit(e.catalog(), 0), 3);
        let sha1 = npc_take_card(&mut e, |d| d.kind == CardKind::Sha);
        let sha2 = npc_take_card(&mut e, |d| d.kind == CardKind::Sha);
        let _ = npc_play_card(&mut e, 0, sha1, vec![1]);
        drain_to_pass(&mut e);
        let out2 = npc_play_card(&mut e, 0, sha2, vec![1]);
        assert!(!matches!(out2, CommandOutcome::Rejected(_)), "txhj_renwang ceil(1/5)=+1 Sha must allow 2nd Sha");
        drain_to_pass(&mut e);

        // Fill to 36 total cards into `txhj_renwang` via deck IDs, then activate `txhj_renfang`!
        let cur_fang = e.state_core().vm_aux.piles[&(0, "txhj_renwang".to_string())].len();
        let extra_fill: Vec<u32> = e.state_core_mut().deck.drain(..(36 - cur_fang)).collect();
        e.state_core_mut()
            .vm_aux
            .piles
            .entry((0, "txhj_renwang".to_string()))
            .or_default()
            .extend(extra_fill);
        assert_eq!(e.state_core().vm_aux.piles[&(0, "txhj_renwang".to_string())].len(), 36);
        let rf_id = e.catalog().skill_by_name("txhj_renfang").unwrap();
        let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: rf_id });
        let ch = e.open_choice().cloned().expect("txhj_renfang:choose_duration");
        assert_eq!(ch.spec.prompt, "txhj_renfang:choose_duration");
        let _ = answer_choice(&mut e, 0, vec![ChoiceOption::Tag(0)]); // 1回合
        assert_eq!(e.state_core().vm_aux.piles.get(&(0, "txhj_renwang".to_string())).map_or(0, |v| v.len()), 0, "36 方 spent");
        e.debug_set_turn(0, Phase::End, 2);
        let _ = submit(&mut e, 0, GameAction::NextPhase);
        let tgt = e.open_choice().cloned().expect("txhj_renfang:choose_target");
        assert_eq!(tgt.spec.prompt, "txhj_renfang:choose_target");
        e.debug_clear_hand(1);
        e.debug_set_hp(1, 5);
        let _ = answer_choice(&mut e, 0, vec![ChoiceOption::Player(1)]);
        drain_to_pass(&mut e);
        assert!(!e.state_core().player(1).alive, "txhj_renfang must deal 36 Thunder dmg and kill enemy");
    }

    // 8. `txhj_dizhou` & `txhj_didun` (`txhj_huanyanzhangbao`) + `txhj_zhoulei` (`txhj_huanyanzhangjiaolei`)
    //    + `txhj_guiyan` (`txhj_huanyanzhangjiaohuo`) + `txhj_xiefeng` (`txhj_huanyanzhangjiaofeng`).
    {
        // 8a. `txhj_huanyanzhangbao`: `txhj_dizhou` expand + enemy card suit match -> Spade judge cancels & loses 1 HP; `txhj_didun` black rejudge + draw 1 on black old card!
        let mut eb = start_locked_engine(2026_968, 2, LockedStartMode::Prototype, verified).unwrap();
        eb.assign_general_named(0, "txhj_huanyanzhangbao").unwrap();
        eb.assign_general_named(1, "txhj_huangyueying").unwrap();
        eb.debug_clear_hand(0);
        let sp_c = npc_take_card(&mut eb, |d| d.suit == Suit::Spade);
        let blk_rw = npc_take_card(&mut eb, |d| d.suit == Suit::Spade);
        eb.state_core_mut().player_mut(0).hand.extend([sp_c, blk_rw]);
        eb.debug_set_turn(0, Phase::End, 1);
        let _ = submit(&mut eb, 0, GameAction::NextPhase); // End -> TurnPhaseEnd -> txhj_dizhou expand
        let dz = eb.open_choice().cloned().expect("txhj_dizhou:choose_card");
        assert_eq!(dz.spec.prompt, "txhj_dizhou:choose_card");
        let _ = answer_choice(&mut eb, 0, vec![ChoiceOption::Card(sp_c)]);
        drain_to_pass(&mut eb);
        assert_eq!(eb.state_core().vm_aux.piles[&(0, "txhj_dizhou".to_string())], vec![sp_c]);

        // Enemy plays a Spade Sha while deck top is Heart/Red (replaced by `txhj_didun` with blk_rw? No: test deck top Spade 5 -> cancels + loses 1 HP; AND test `txhj_didun` with Black old judge!)
        advance_to_play(&mut eb, 1);
        eb.debug_clear_hand(1);
        eb.debug_set_hp(0, 6);
        eb.debug_set_hp(1, 5);
        let sp_sha = npc_take_card(&mut eb, |d| d.kind == CardKind::Sha && d.suit == Suit::Spade);
        let filler = npc_take_card(&mut eb, |_| true);
        eb.state_core_mut().player_mut(1).hand.push(filler);
        let old_blk = eb.debug_set_deck_top_by(|d| d.suit == Suit::Club && d.rank == 4);
        let _ = npc_play_card(&mut eb, 1, sp_sha, vec![0]);
        // `txhj_didun` prompts `"gui_dao_rewrite"` with blk_rw (Spade) -> gain old_blk (Club=Black -> draw 1!) & final judge is Spade -> cancels Sha + random-discards filler!
        if let Some(rw) = eb.open_choice().cloned() {
            if rw.spec.prompt == "gui_dao_rewrite" {
                let snap_dz = encode_archive(&eb.save());
                let mut eb_dz = restore_locked_engine(&decode_archive(&snap_dz).unwrap(), verified).unwrap();
                let _ = answer_choice(&mut eb, 0, vec![ChoiceOption::Card(blk_rw)]);
                let _ = answer_choice(&mut eb_dz, 0, vec![ChoiceOption::Card(blk_rw)]);
                drain_to_pass(&mut eb);
                drain_to_pass(&mut eb_dz);
                assert_eq!(eb.state_hash(), eb_dz.state_hash());
            }
        }
        drain_to_pass(&mut eb);
        assert_eq!(eb.state_core().player(0).hp, 6, "txhj_dizhou Spade cancel must prevent Sha");
        assert!(eb.state_core().player(0).hand.contains(&old_blk), "txhj_didun gains old judge card");
        assert!(eb.state_core().player(0).hand.len() >= 2, "txhj_didun draws 1 when old judge card was black");
        assert!(eb.state_core().player(1).hand.is_empty(), "txhj_dizhou black random-discards user card");

        // 8b. `txhj_huanyanzhangjiaolei`: `txhj_zhoulei` fires on enemy CardsDrawn with diff >= 5 -> 1 Thunder dmg + mark 1!
        let mut el = start_locked_engine(2026_969, 2, LockedStartMode::Prototype, verified).unwrap();
        el.assign_general_named(0, "txhj_huanyanzhangjiaolei").unwrap();
        el.assign_general_named(1, "txhj_huangyueying").unwrap();
        el.debug_clear_hand(0);
        el.debug_clear_hand(1);
        let cs: Vec<u32> = el.state_core_mut().deck.drain(..4).collect();
        el.state_core_mut().player_mut(1).hand.extend(cs);
        el.debug_set_hp(1, 4);
        el.debug_set_turn(1, Phase::Judge, 1);
        let _ = submit(&mut el, 1, GameAction::NextPhase); // Judge -> Draw (seat 1 hand becomes 6 >= 5 -> txhj_zhoulei deals 1 Thunder!)
        drain_to_pass(&mut el);
        assert_eq!(el.state_core().player(1).hp, 3, "txhj_zhoulei must deal 1 Thunder dmg");
        assert_eq!(el.state_core().vm_aux.flags.get(&(0, "l11_zhoulei_marks".to_string())).copied(), Some(1));

        // 8c. `txhj_huanyanzhangjiaohuo`: `txhj_guiyan` grants `ranshang` on Round 1 + Fire/+1 dmg, then Round 2 virtual Huogong!
        let mut eh = start_locked_engine(2026_970, 2, LockedStartMode::Prototype, verified).unwrap();
        eh.assign_general_named(0, "txhj_huanyanzhangjiaohuo").unwrap();
        eh.assign_general_named(1, "txhj_huangyueying").unwrap();
        eh.debug_set_hp(0, 4);
        eh.debug_set_hp(1, 5);
        eh.debug_set_turn(1, Phase::End, 1);
        let _ = submit(&mut eh, 1, GameAction::NextPhase); // transitions into seat 0 Prepare -> txhj_guiyan!
        let gch = eh.open_choice().cloned().expect("txhj_guiyan:choose_target");
        assert_eq!(gch.spec.prompt, "txhj_guiyan:choose_target");
        let snap_gy = encode_archive(&eh.save());
        let mut eh_gy = restore_locked_engine(&decode_archive(&snap_gy).unwrap(), verified).unwrap();
        let _ = answer_choice(&mut eh, 0, vec![ChoiceOption::Player(1)]);
        let _ = answer_choice(&mut eh_gy, 0, vec![ChoiceOption::Player(1)]);
        drain_to_pass(&mut eh);
        drain_to_pass(&mut eh_gy);
        assert_eq!(eh.state_hash(), eh_gy.state_hash());
        assert!(npc_has_skill(&eh, 1, "ranshang"), "txhj_guiyan must grant ranshang");
        // Deal Normal Sha from seat 0 vs linked target -> `txhj_guiyan` converts all damage to Fire (+1),
        // unlinking target and dealing +1 Fire damage cleanly!
        // Also test Round 2 choosing same target (now holding `ranshang`) emits virtual Huogong!
        advance_to_play(&mut eh, 0);
        eh.debug_clear_hand(0);
        eh.debug_clear_hand(1);
        eh.debug_set_hp(1, 10);
        eh.state_core_mut().player_mut(1).linked = true;
        let sha_h = npc_take_card(&mut eh, |d| d.kind == CardKind::Sha);
        let logs_before = eh.log().len();
        let _ = npc_play_card(&mut eh, 0, sha_h, vec![1]);
        drain_to_pass(&mut eh);
        assert!(!eh.state_core().player(1).linked, "txhj_guiyan Fire damage must break iron chain");
        assert!(
            eh.log()[logs_before..].iter().any(|ev| matches!(ev, LogEvent::DamageDealt { target: 1, card: Some(c), amount: 2, .. } if *c == sha_h)),
            "txhj_guiyan Normal Sha -> Fire(+1) must deal 2 dmg"
        );
        // Round 2: target 1 already has `ranshang` -> `txhj_guiyan` fires virtual Huogong!
        let f_card = npc_take_card(&mut eh, |d| d.suit == Suit::Heart);
        let t_card = npc_take_card(&mut eh, |d| d.suit == Suit::Heart);
        eh.debug_clear_hand(0);
        eh.debug_clear_hand(1);
        eh.state_core_mut().player_mut(0).hand.push(f_card);
        eh.state_core_mut().player_mut(1).hand.push(t_card);
        eh.debug_set_hp(0, 3);
        eh.debug_set_hp(1, 5);
        eh.debug_set_turn(1, Phase::End, 2);
        let _ = submit(&mut eh, 1, GameAction::NextPhase);
        let _gch2 = eh.open_choice().cloned().expect("txhj_guiyan round 2 target");
        let _ = answer_choice(&mut eh, 0, vec![ChoiceOption::Player(1)]);
        let mut saw_huogong = false;
        while let Some(c) = eh.open_choice().cloned() {
            if c.spec.prompt == "huogong:show_card" {
                saw_huogong = true;
                let _ = answer_choice(&mut eh, 1, vec![ChoiceOption::Card(t_card)]);
            } else if c.spec.prompt == "huogong:discard_card" {
                let _ = answer_choice(&mut eh, 0, vec![ChoiceOption::Card(f_card)]);
            } else {
                let _ = answer_choice(&mut eh, c.observer, vec![ChoiceOption::Pass]);
            }
        }
        assert!(saw_huogong, "txhj_guiyan on existing ranshang must fire virtual Huogong");
        assert!(eh.state_core().player(1).hp <= 3, "txhj_guiyan virtual Huogong must deal boosted Fire dmg");
        let _ = DamageNature::Fire;

        // 8d. `txhj_huanyanzhangjiaofeng`: `txhj_xiefeng` discards 2 cards -> draws 1 -> shields Normal dmg, does NOT shield Thunder dmg!
        let mut ef = start_locked_engine(2026_971, 2, LockedStartMode::Prototype, verified).unwrap();
        ef.assign_general_named(0, "txhj_huanyanzhangjiaofeng").unwrap();
        ef.assign_general_named(1, "txhj_shiemoxing").unwrap();
        ef.debug_clear_hand(0);
        let x1 = npc_take_card(&mut ef, |_| true);
        let x2 = npc_take_card(&mut ef, |_| true);
        ef.state_core_mut().player_mut(0).hand.extend([x1, x2]);
        ef.debug_set_turn(0, Phase::End, 1);
        let _ = submit(&mut ef, 0, GameAction::NextPhase); // End -> TurnPhaseAfter -> txhj_xiefeng discards 2 & draws 1 -> stacks=2; then seat 1 Prepare -> decline luolei
        let xf = ef.open_choice().cloned().expect("txhj_xiefeng:discard_cards");
        assert_eq!(xf.spec.prompt, "txhj_xiefeng:discard_cards");
        let _ = answer_choice(&mut ef, 0, vec![ChoiceOption::Card(x1), ChoiceOption::Card(x2)]);
        while let Some(c) = ef.open_choice().cloned() {
            let sel = if c.spec.allow_cancel { vec![] } else { vec![ChoiceOption::Pass] };
            let _ = answer_choice(&mut ef, c.observer, sel);
        }
        assert_eq!(ef.state_core().player(0).hand.len(), 1, "txhj_xiefeng draws 1 after discard");
        assert_eq!(ef.state_core().vm_aux.flags.get(&(0, "l11_xiefeng_stacks".to_string())).copied(), Some(2));
        advance_to_play(&mut ef, 1);
        ef.debug_clear_hand(0);
        ef.debug_clear_hand(1);
        ef.debug_set_hp(0, 5);
        let sha_n = npc_take_card(&mut ef, |d| d.kind == CardKind::Sha);
        let _ = npc_play_card(&mut ef, 1, sha_n, vec![0]);
        drain_to_pass(&mut ef);
        assert_eq!(ef.state_core().player(0).hp, 5, "txhj_xiefeng must prevent non-Thunder Normal Sha dmg");
        // Advance to seat 1 next Prepare -> `txhj_luolei` (Thunder 1) MUST penetrate `txhj_xiefeng` even with stacks > 0!
        ef.debug_set_turn(0, Phase::End, 2);
        let _ = submit(&mut ef, 0, GameAction::NextPhase);
        let ll = ef.open_choice().cloned().expect("txhj_luolei:choose_target");
        assert_eq!(ll.spec.prompt, "txhj_luolei:choose_target");
        let _ = answer_choice(&mut ef, 1, vec![ChoiceOption::Player(0)]);
        drain_to_pass(&mut ef);
        assert!(ef.state_core().vm_aux.flags.get(&(0, "l11_xiefeng_stacks".to_string())).copied().unwrap_or(0) > 0);
        assert_eq!(ef.state_core().player(0).hp, 4, "txhj_xiefeng must NOT prevent Thunder dmg");
    }
}
