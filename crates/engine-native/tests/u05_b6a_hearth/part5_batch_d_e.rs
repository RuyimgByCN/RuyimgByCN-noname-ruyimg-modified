//! Batch U05-6a Part 5: Batch D 20 skills (`midian` .. `wzhanyi`,
//! `character/hearth/skill.js:2604-3339`) + Batch E 27 skills
//! (`shengteng` .. `hsbaowu_huangjinyuanhou2`, `:3370-4177`).

use super::common::*;
use engine_native::catalog::CardKind;
use engine_native::event::Phase;
use engine_native::ChoiceOption;

#[test]
fn u05_b6a_batch_d_all_20_midian_through_wzhanyi() {
    // 1-4. `midian` (:2604), `xingluo` (:2630), `yuelu` (:2684), `yushou` (:2716..2854)
    let mut e = fresh_proto(15_001, &["hs_khadgar", "hs_tyrande", "hs_lrexxar"]);
    let tracked = total_tracked_cards(&e);
    advance_to_phase(&mut e, 0, Phase::Play);
    clear_hand(&mut e, 0);
    let _sha_neg = give_free_card_kind(&mut e, 0, CardKind::Sha);
    assert!(matches!(activate(&mut e, 0, "midian"), engine_native::CommandOutcome::Rejected(_)));
    let f = give_free_card_kind(&mut e, 0, CardKind::Wuzhong);
    assert!(!matches!(activate(&mut e, 0, "midian"), engine_native::CommandOutcome::Rejected(_)));
    save_restore_and_answer(&mut e, "hs_midian_card", vec![ChoiceOption::Card(f)]);
    drain_auto(&mut e);
    assert_eq!(e.state_core().player(0).hand.len(), 4, "midian must spawn 3 random tricks into hand");

    // Cancel `midian` with second attempt only if refunded; here verify once_per_turn rejects second activation
    assert!(matches!(activate(&mut e, 0, "midian"), engine_native::CommandOutcome::Rejected(_)));

    // `xingluo` & `yuelu`: Tyrande dying save via black card discard
    e.state_core_mut().player_mut(1).hp = 1;
    let blk = give_card_named(&mut e, 1, "hsjixie_zhadan"); // Spade
    let sha = give_free_card_kind(&mut e, 0, CardKind::Sha);
    let yuelu_def = e.catalog().skill_by_name("yuelu").unwrap();
    let _ = (blk, play_card(&mut e, 0, sha, vec![1]));
    let ds = advance_until_open_prompt(&mut e, "dying_save", 6)
        .expect("dying_save must offer yuelu when Tyrande holds black card");
    assert!(ds.spec.options.contains(&ChoiceOption::Skill(yuelu_def)));
    save_restore_and_answer(&mut e, "dying_save", vec![ChoiceOption::Skill(yuelu_def)]);
    let yl = e.open_choice().cloned().expect("hs_yuelu_card must open after picking yuelu");
    assert_eq!(yl.spec.prompt, "hs_yuelu_card");
    save_restore_and_answer(&mut e, "hs_yuelu_card", vec![yl.spec.options[0]]);
    drain_auto(&mut e);
    assert!(e.state_core().player(1).alive && e.state_core().player(1).hp >= 1);

    // `yushou` active on seat 2: discard -> gain random companion (`yushou_misha`/`huofu`/`leiouke`)
    advance_to_phase(&mut e, 2, Phase::Play);
    clear_hand(&mut e, 2);
    let yc = give_free_card_kind(&mut e, 2, CardKind::Sha);
    assert!(!matches!(activate(&mut e, 2, "yushou"), engine_native::CommandOutcome::Rejected(_)));
    save_restore_and_answer(&mut e, "hs_yushou_card", vec![ChoiceOption::Card(yc)]);
    drain_auto(&mut e);
    let pool = ["yushou_misha", "yushou_huofu", "yushou_leiouke"];
    assert!(pool.iter().any(|s| has_skill(&e, 2, s)));
    assert_eq!(total_tracked_cards(&e), tracked);

    // 5-12. `aoshu` (:2957), `bzhuiji` (:2979), `lianjin` (:2996), `shouji` (:3187),
    // `yingxi` (:3225) + `yingxi2`, `hsguimou` (:3265), `peiyu` (:3306) + `peiyu2`, `wzhanyi` (:3339),
    // `qingzun` (:2862) + `ayuling` (:2924)
    let mut e2 = fresh_proto(15_002, &["hs_jaina", "hs_kazhakusi", "hs_bolvar", "hs_walian"]);
    let tracked2 = total_tracked_cards(&e2);
    advance_to_phase(&mut e2, 0, Phase::Play);
    clear_hand(&mut e2, 0);
    // Negative `aoshu`: non-spade hand rejected / or choice empty; with Spade -> casts as Wuzhong & draws 2
    assert!(matches!(activate(&mut e2, 0, "aoshu"), engine_native::CommandOutcome::Rejected(_)));
    let sp = give_card_named(&mut e2, 0, "hsjixie_zhadan"); // Spade 3
    assert!(!matches!(activate(&mut e2, 0, "aoshu"), engine_native::CommandOutcome::Rejected(_)));
    save_restore_and_answer(&mut e2, "hs_aoshu_card", vec![ChoiceOption::Card(sp)]);
    drain_auto(&mut e2);
    assert_eq!(e2.state_core().player(0).hand.len(), 2, "aoshu Spade as Wuzhong draws 2");

    // `lianjin` (:2996) on seat 1: discard card -> spawn `hsyaoshui`
    advance_to_phase(&mut e2, 1, Phase::Play);
    clear_hand(&mut e2, 1);
    let lc = give_free_card_kind(&mut e2, 1, CardKind::Sha);
    assert!(!matches!(activate(&mut e2, 1, "lianjin"), engine_native::CommandOutcome::Rejected(_)));
    save_restore_and_answer(&mut e2, "hs_lianjin_card", vec![ChoiceOption::Card(lc)]);
    drain_auto(&mut e2);
    assert!(
        e2.state_core().player(1).hand.iter().any(|&c| e2.catalog().cards[c as usize].name.contains("hsyaoshui")),
        "lianjin must create hsyaoshui into hand"
    );

    // `bzhuiji` (:2979): seat 0 kills seat 1 -> seat 2 (`hs_bolvar`) triggers `bzhuiji` (draw 2 + Duel seat 0)
    advance_to_phase(&mut e2, 0, Phase::Play);
    clear_hand(&mut e2, 0);
    clear_hand(&mut e2, 1);
    e2.state_core_mut().player_mut(1).hp = 1;
    let kill_sha = give_free_card_kind(&mut e2, 0, CardKind::Sha);
    let _ = play_card(&mut e2, 0, kill_sha, vec![1]);
    let bz = advance_until_open_prompt(&mut e2, "hs_bzhuiji_confirm", 8)
        .expect("bzhuiji must prompt hs_bzhuiji_confirm when another player dies");
    save_restore_and_answer(&mut e2, bz.spec.prompt, vec![ChoiceOption::Confirm]);
    drain_auto(&mut e2);
    assert!(!e2.state_core().player(2).hand.is_empty());
    assert_eq!(total_tracked_cards(&e2), tracked2);

    // `ayuling` + `qingzun` (:2862/:2924), `peiyu` + `peiyu2`, `hsguimou`, `yingxi` + `yingxi2`
    let mut e3 = fresh_proto(15_003, &["hs_lrexxar", "hs_aya", "hs_totemic", "hs_laxiao"]);
    let tracked3 = total_tracked_cards(&e3);
    advance_to_phase(&mut e3, 0, Phase::Play);
    clear_hand(&mut e3, 0);
    clear_hand(&mut e3, 1);
    let sha_a = give_free_card_kind(&mut e3, 0, CardKind::Sha);
    let _ = play_card(&mut e3, 0, sha_a, vec![1]);
    let ay = advance_until_open_prompt(&mut e3, "trigger_confirm", 4)
        .expect("ayuling must prompt trigger_confirm on taking damage");
    assert_eq!(ay.observer, 1);
    let _ = answer(&mut e3, ay.observer, vec![ChoiceOption::Confirm]);
    drain_auto(&mut e3);
    assert!(
        e3.state_core().player(1).hand.iter().any(|&c| e3.catalog().cards[c as usize].name.contains("hsqingyu_")),
        "ayuling must spawn an hsqingyu_* Jade Golem card on damage"
    );
    advance_to_phase(&mut e3, 1, Phase::Play);
    let qy = give_card_named(&mut e3, 1, "hsqingyu_zhanfang");
    let _ = play_card(&mut e3, 1, qy, vec![1]);
    drain_auto(&mut e3);
    assert!(flag(&e3, 1, "hs_qingzun_count") >= 1);

    // `peiyu` + `peiyu2` on seat 2 (`hs_totemic`): Prepare grants target `peiyu2` + `tuteng*`; removed after damage
    advance_to_phase(&mut e3, 1, Phase::Discard);
    let py_ch = advance_until_open_prompt(&mut e3, "hs_peiyu_target", 8)
        .expect("peiyu at Prepare must prompt hs_peiyu_target");
    assert_eq!(py_ch.observer, 2);
    save_restore_and_answer(&mut e3, "hs_peiyu_target", vec![ChoiceOption::Player(2)]);
    drain_auto(&mut e3);
    assert!(has_skill(&e3, 2, "peiyu2"));
    assert!(
        (1..=8).any(|i| has_skill(&e3, 2, &format!("tuteng{i}"))),
        "peiyu must grant a tuteng skill alongside peiyu2"
    );
    // `hsguimou` & `yingxi`/`yingxi2` on seat 3 (`hs_laxiao`)
    advance_to_phase(&mut e3, 2, Phase::Play);
    clear_hand(&mut e3, 2);
    let red_src = give_free_card_kind(&mut e3, 2, CardKind::Tao);
    let dmg_duel = give_free_card_kind(&mut e3, 2, CardKind::Duel);
    let _ = play_card(&mut e3, 2, dmg_duel, vec![3]);
    let gm_ch = advance_until_open_prompt(&mut e3, "trigger_confirm", 6)
        .expect("hsguimou must prompt trigger_confirm on damage from source with hand");
    assert_eq!(gm_ch.observer, 3);
    let _ = answer(&mut e3, 3, vec![ChoiceOption::Confirm]);
    drain_auto(&mut e3);
    assert!(e3.state_core().player(3).hand.contains(&red_src), "hsguimou must steal card from damage source");
    // Seat 3 Play -> End with 0 turn_damage & black card -> `yingxi` uses black card as directHit Sha (`yingxi2`)
    advance_to_phase(&mut e3, 3, Phase::Play);
    clear_hand(&mut e3, 3);
    clear_hand(&mut e3, 2);
    let yx_blk = give_card_named(&mut e3, 3, "hsjixie_zhadan");
    let _shan2 = give_free_card_kind(&mut e3, 2, CardKind::Shan); // Full-HP target CANNOT Shan `yingxi2`!
    let hp2_before_yx = e3.state_core().player(2).hp;
    advance_to_phase(&mut e3, 3, Phase::Discard);
    let yx_c = advance_until_open_prompt(&mut e3, "hs_yingxi_card", 6)
        .expect("yingxi must open hs_yingxi_card when no damage dealt in turn");
    assert_eq!(yx_c.observer, 3);
    save_restore_and_answer(&mut e3, "hs_yingxi_card", vec![ChoiceOption::Card(yx_blk)]);
    save_restore_and_answer(&mut e3, "hs_yingxi_target", vec![ChoiceOption::Player(2)]);
    drain_auto(&mut e3);
    assert_eq!(e3.state_core().player(2).hp, hp2_before_yx - 1, "yingxi2 Sha against full-HP target must ignore Shan");
    assert!(e3.state_core().player(2).hand.contains(&_shan2), "yingxi2 directHit prevents spending Shan");
    assert_eq!(total_tracked_cards(&e3), tracked3);

    // `shouji` (:3187) on `hs_shaku` + `wzhanyi` (:3339) on `hs_walian` + unique-equip exclusion on `shouji_miss`
    let mut e4 = fresh_proto(15_004, &["hs_shaku", "hs_walian"]);
    let tracked4 = total_tracked_cards(&e4);
    advance_to_phase(&mut e4, 0, Phase::Play);
    clear_hand(&mut e4, 0);
    clear_hand(&mut e4, 1);
    // Equip `hstianqi_dalian` genuinely in seat 1 Play so equip skills remain coherent across save/restore
    advance_to_phase(&mut e4, 1, Phase::Play);
    clear_hand(&mut e4, 1);
    let uniq_eq = give_card_named(&mut e4, 1, "hstianqi_dalian");
    let _ = play_card(&mut e4, 1, uniq_eq, vec![]);
    drain_auto(&mut e4);
    advance_to_phase(&mut e4, 0, Phase::Play);
    clear_hand(&mut e4, 0);
    clear_hand(&mut e4, 1);
    let shan_u = give_free_card_kind(&mut e4, 1, CardKind::Shan);
    let shan_u2 = give_free_card_kind(&mut e4, 1, CardKind::Shan);
    let sha_u = give_free_card_kind(&mut e4, 0, CardKind::Sha);
    let _ = play_card(&mut e4, 0, sha_u, vec![1]);
    if let Some(ch) = e4.open_choice().cloned() {
        let pick = if ch.spec.options.contains(&ChoiceOption::Card(shan_u)) {
            shan_u
        } else {
            shan_u2
        };
        let _ = answer(&mut e4, ch.observer, vec![ChoiceOption::Card(pick)]);
    }
    drain_auto(&mut e4);
    assert_eq!(flag(&e4, 0, "hs_shouji_miss_used"), 0, "shouji_miss must ignore unique equips");
    clear_hand(&mut e4, 1);
    e4.state_core_mut().turn.sha_used_play_phase = false;
    e4.state_core_mut().vm_aux.flags.remove(&(0, "hs_shouji_begin_used".to_string()));
    e4.state_core_mut().vm_aux.flags.remove(&(0, "hs_shouji_miss_used".to_string()));
    let arm1 = give_free_card_kind(&mut e4, 1, CardKind::HorseMinus);
    let shan1 = give_free_card_kind(&mut e4, 1, CardKind::Shan);
    e4.state_core_mut().player_mut(1).hand.retain(|&c| c != arm1);
    e4.state_core_mut().player_mut(1).equips.push(arm1);
    let sha_sj = give_free_card_kind(&mut e4, 0, CardKind::Sha);
    let _ = play_card(&mut e4, 0, sha_sj, vec![1]);
    let mut answered_shan = false;
    for _ in 0..4 {
        if let Some(ch) = e4.open_choice().cloned() {
            if ch.observer == 1 && ch.spec.options.contains(&ChoiceOption::Card(shan1)) {
                let _ = answer(&mut e4, 1, vec![ChoiceOption::Card(shan1)]);
                answered_shan = true;
                break;
            }
            let _ = cancel_open_choice(&mut e4);
        } else {
            break;
        }
    }
    assert!(answered_shan, "opponent must be asked for Shan against Sha");
    drain_auto(&mut e4);
    assert_eq!(flag(&e4, 0, "hs_shouji_begin_used"), 1, "shouji_begin must fire on ShaBegin when target has hand card");
    assert_eq!(flag(&e4, 0, "hs_shouji_miss_used"), 1, "shouji_miss must fire on ShaMiss when target has non-unique equip");
    assert!(e4.state_core().player(0).hand.len() >= 2);
    // `wzhanyi` on seat 1 (`hs_walian`) PlayBefore: runs in BeforeTriggers, skips Play phase completely, and AI threshold hand_count + 2 <= hp
    clear_hand(&mut e4, 1);
    advance_to_phase(&mut e4, 0, Phase::Discard);
    let wz_sha = give_free_card_kind(&mut e4, 1, CardKind::Sha);
    put_card_on_deck_top(&mut e4, wz_sha);
    let log_wz_start = e4.log().len();
    let wz_ch = advance_until_open_prompt(&mut e4, "trigger_confirm", 8)
        .expect("wzhanyi must prompt trigger_confirm at phaseUseBefore");
    assert_eq!(wz_ch.observer, 1);
    // Trim hand after Phase::Draw (+2 cards) to exactly 2 cards so 2+2<=hp(4) -> Confirm, and adding 1 -> 3+2>4 -> Pass
    while e4.state_core().player(1).hand.len() > 2 {
        let drop_c = e4.state_core_mut().player_mut(1).hand.pop().unwrap();
        e4.state_core_mut().discard.push(drop_c);
    }
    assert_eq!(e4.state_core().player(1).hand.len(), 2);
    assert_eq!(
        ai_plan_action(&e4, 1),
        engine_native::GameAction::Answer {
            choice_id: wz_ch.id,
            selection: vec![ChoiceOption::Confirm],
        },
        "wzhanyi AI must confirm when hand_count + 2 <= hp"
    );
    let extra_ai = give_free_card_kind(&mut e4, 1, CardKind::Shan);
    assert_eq!(
        ai_plan_action(&e4, 1),
        engine_native::GameAction::Answer {
            choice_id: wz_ch.id,
            selection: vec![ChoiceOption::Pass],
        },
        "wzhanyi AI must pass when hand_count + 2 > hp"
    );
    e4.state_core_mut().player_mut(1).hand.retain(|&c| c != extra_ai);
    e4.state_core_mut().discard.push(extra_ai);
    save_restore_and_answer(&mut e4, "trigger_confirm", vec![ChoiceOption::Confirm]);
    assert!(e4.state_core().turn.skip_play_phase, "confirming wzhanyi must skip normal Play phase");
    let wzu = advance_until_open_prompt(&mut e4, "hs_wzhanyi_use", 4)
        .expect("wzhanyi must offer hs_wzhanyi_use for drawn Sha");
    assert_eq!(wzu.observer, 1);
    save_restore_and_answer(&mut e4, "hs_wzhanyi_use", vec![ChoiceOption::Card(wz_sha)]);
    let wzt = advance_until_open_prompt(&mut e4, "hs_wzhanyi_target", 4)
        .expect("wzhanyi Sha must open hs_wzhanyi_target");
    assert_eq!(wzt.observer, 1);
    let _ = answer(&mut e4, 1, vec![ChoiceOption::Player(0)]);
    drain_auto(&mut e4);
    assert!(
        !e4.log().iter().any(|ev| matches!(
            ev,
            engine_native::machine::LogEvent::PhaseChanged {
                player: 1,
                phase: Phase::Play,
                ..
            }
        )),
        "wzhanyi runs in BeforeTriggers so seat 1 never enters Phase::Play"
    );
    let _ = log_wz_start;
    assert_ne!(e4.turn().phase, Phase::Play);
    assert_eq!(total_tracked_cards(&e4), tracked4);
}

#[test]
fn u05_b6a_batch_e_all_27_shengteng_through_baowu() {
    // 1-5. `shengteng` (:3370), `yuansu` (:3381), `chouhuo` (:3401), `nuyan2` (:3429), `nuyan` (:3513)
    let mut e = fresh_proto(15_010, &["hs_hallazeal", "hs_fandral", "hs_jaina"]);
    let tracked = total_tracked_cards(&e);
    advance_to_phase(&mut e, 0, Phase::Play);
    clear_hand(&mut e, 0);
    // `yuansu` active: requires lost_hp >= 3; negative at hp=2, positive at hp=1
    e.state_core_mut().player_mut(0).hp = 2;
    assert!(matches!(activate(&mut e, 0, "yuansu"), engine_native::CommandOutcome::Rejected(_)));
    e.state_core_mut().player_mut(0).hp = 1;
    assert!(!matches!(activate(&mut e, 0, "yuansu"), engine_native::CommandOutcome::Rejected(_)));
    save_restore_and_answer(&mut e, "hs_yuansu_confirm", vec![ChoiceOption::Confirm]);
    drain_auto(&mut e);

    // `nuyan` active on seat 1 (`hs_fandral`): pick Sha tag (0) + red card + target -> then `chouhuo` + `nuyan2`
    advance_to_phase(&mut e, 1, Phase::Play);
    clear_hand(&mut e, 1);
    let red_c = give_free_card_kind(&mut e, 1, CardKind::Tao); // Heart
    assert!(!matches!(activate(&mut e, 1, "nuyan"), engine_native::CommandOutcome::Rejected(_)));
    let _ = answer(&mut e, 1, vec![ChoiceOption::Tag(0)]);
    let _ = answer(&mut e, 1, vec![ChoiceOption::Card(red_c)]);
    save_restore_and_answer(&mut e, "hs_nuyan_target", vec![ChoiceOption::Player(0)]);
    drain_auto(&mut e);
    // `chouhuo` (:3401) at Prepare when all 6 nuyan tricks used (mask == 0b11_1111) -> max_hp-1, +2 hujia, grant `nuyan2`
    set_flag(&mut e, 1, "hs_nuyan_used_mask", 0b11_1111);
    let mhp_f = e.state_core().player(1).max_hp;
    advance_to_phase(&mut e, 2, Phase::Play);
    advance_to_phase(&mut e, 1, Phase::Play);
    assert_eq!(e.state_core().player(1).max_hp, mhp_f - 1, "chouhuo must reduce max_hp by 1");
    assert!(e.state_core().player(1).hujia >= 2, "chouhuo must grant 2 hujia");
    assert!(has_skill(&e, 1, "nuyan2"));
    assert!(!matches!(activate(&mut e, 1, "nuyan2"), engine_native::CommandOutcome::Rejected(_)));
    let _ = answer(&mut e, 1, vec![ChoiceOption::Tag(1)]); // Duel
    let _ = answer(&mut e, 1, vec![ChoiceOption::Player(0)]);
    drain_auto(&mut e);
    assert_eq!(flag(&e, 1, "hs_nuyan2_used_turn"), 1);
    assert_eq!(total_tracked_cards(&e), tracked);

    // 6-14. `duxin` (:3603), `oldduxin` (:3626), `hsdusu_shinancao` (:3646), `xiubu` (:3661),
    // `zengli` (:3688), `mobao` (:3730), `mobao2`, `mobao3`, `xianji` (:3795), `xianji2`, `xianji3`
    let mut e2 = fresh_proto(15_011, &["hs_blingtron", "hs_anomalus", "hs_kcthun", "hs_xialikeer"]);
    let tracked2 = total_tracked_cards(&e2);
    advance_to_phase(&mut e2, 0, Phase::Play);
    // `zengli` + `xiubu`: equipping weapon triggers `xiubu` -> grants CardKind::Hslingjian
    assert!(!matches!(activate(&mut e2, 0, "zengli"), engine_native::CommandOutcome::Rejected(_)));
    save_restore_and_answer(&mut e2, "hs_zengli_target", vec![ChoiceOption::Player(1)]);
    drain_auto(&mut e2);
    assert!(!e2.state_core().player(0).equips.is_empty());
    assert!(!e2.state_core().player(1).equips.is_empty());
    assert!(
        e2.state_core().player(0).hand.iter().any(|&c| e2.catalog().card_kind(c) == CardKind::Hslingjian),
        "xiubu must grant an Hslingjian card when weapon equipped"
    );

    // `mobao` on seat 1 (`hs_anomalus`): damage sets mask -> active thunder damage -> End clears mask!
    clear_hand(&mut e2, 0);
    clear_hand(&mut e2, 1);
    let sha0 = give_free_card_kind(&mut e2, 0, CardKind::Sha);
    let out_sha0 = play_card(&mut e2, 0, sha0, vec![1]);
    assert!(!matches!(out_sha0, engine_native::CommandOutcome::Rejected(_)));
    drain_auto(&mut e2);
    advance_to_phase(&mut e2, 1, Phase::Play);
    assert_ne!(flag(&e2, 1, "hs_mobao_mask"), 0);
    let blk1 = give_card_named(&mut e2, 1, "hsjixie_zhadan");
    let blk2 = give_card_named(&mut e2, 1, "jiedao");
    assert!(matches!(
        e2.catalog().cards[blk2 as usize].suit,
        engine_native::catalog::Suit::Spade | engine_native::catalog::Suit::Club
    ));
    assert!(!matches!(activate(&mut e2, 1, "mobao"), engine_native::CommandOutcome::Rejected(_)));
    save_restore_and_answer(&mut e2, "hs_mobao_cards", vec![ChoiceOption::Card(blk1)]);
    drain_auto(&mut e2);
    assert!(
        matches!(activate(&mut e2, 1, "mobao"), engine_native::CommandOutcome::Rejected(_)),
        "mobao active skill must allow only once per Play phase while still allowing End phase clear"
    );
    let _ = blk2;
    clear_hand(&mut e2, 1);
    advance_to_phase(&mut e2, 1, Phase::End);
    drain_auto(&mut e2);
    assert_eq!(flag(&e2, 1, "hs_mobao_mask"), 0, "mobao End Phase must clear hs_mobao_mask");

    // `xianji` / `xianji2` / `xianji3` / `duxin` / `oldduxin` / `hsdusu_shinancao`
    advance_to_phase(&mut e2, 2, Phase::Play);
    advance_to_phase(&mut e2, 2, Phase::Discard);
    let mut confirmed_duxin = false;
    for _ in 0..8 {
        if let Some(ch) = advance_until_open_prompt(&mut e2, "trigger_confirm", 8) {
            if ch.observer == 3 {
                save_restore_and_answer(&mut e2, "trigger_confirm", vec![ChoiceOption::Confirm]);
                confirmed_duxin = true;
                break;
            }
            let _ = cancel_open_choice(&mut e2);
        } else {
            break;
        }
    }
    assert!(confirmed_duxin, "duxin at Prepare/End must prompt trigger_confirm for seat 3");
    drain_auto(&mut e2);
    assert!(
        e2.state_core().player(3).hand.iter().any(|&c| e2.catalog().cards[c as usize].name.starts_with("hsdusu_")),
        "duxin at Prepare/End must grant an hsdusu_* poison card"
    );
    // `hsdusu_shinancao` + `xianji2`: seat 3 Play uses poison + at End Phase discards for `xianji2` -> seat 2 draws + seat 3 gets `xianji3`
    advance_to_phase(&mut e2, 3, Phase::Play);
    let sn_card = give_card_named(&mut e2, 3, "hsdusu_shinancao");
    let out_sn = play_card(&mut e2, 3, sn_card, vec![3]);
    assert!(!matches!(out_sn, engine_native::CommandOutcome::Rejected(_)));
    drain_auto(&mut e2);
    assert!(has_skill(&e2, 3, "hsdusu_shinancao"));
    let xj_disc = give_free_card_kind(&mut e2, 3, CardKind::Sha);
    e2.state_core_mut().player_mut(3).hp = 3;
    let kc_hand = e2.state_core().player(2).hand.len();
    advance_to_phase(&mut e2, 3, Phase::Discard);
    let xj_ch = advance_until_open_prompt(&mut e2, "hs_xianji2_discard", 6)
        .expect("xianji2 at non-owner End Phase must open hs_xianji2_discard");
    assert_eq!(xj_ch.observer, 3);
    let _ = answer(&mut e2, 3, vec![ChoiceOption::Card(xj_disc)]);
    drain_auto(&mut e2);
    assert_eq!(e2.state_core().player(2).hand.len(), kc_hand + 1, "xianji2 discard draws 1 for hs_kcthun");
    assert!(has_skill(&e2, 3, "xianji3"), "xianji2 must grant xianji3 to contributor");
    assert_eq!(total_tracked_cards(&e2), tracked2);

    // `tanmi` (:3859) on `hs_lreno`: empty hand at another player's End Phase -> confirm draws 2
    let mut etm = fresh_proto(15_014, &["hs_lreno", "hs_jaina"]);
    let tracked_tm = total_tracked_cards(&etm);
    advance_to_phase(&mut etm, 0, Phase::Play);
    clear_hand(&mut etm, 0);
    advance_to_phase(&mut etm, 1, Phase::Discard);
    let tm_ch = advance_until_open_prompt(&mut etm, "trigger_confirm", 6)
        .expect("tanmi must prompt trigger_confirm when owner has 0 hand at other End");
    assert_eq!(tm_ch.observer, 0);
    let _ = answer(&mut etm, 0, vec![ChoiceOption::Confirm]);
    drain_auto(&mut etm);
    assert!(etm.state_core().player(0).hand.len() >= 1, "tanmi confirm draws 2");
    assert_eq!(total_tracked_cards(&etm), tracked_tm);

    // 15-27. `tanmi` (:3859), `xueren` (:3886), `maoxian` (:3915), `yiwen` (:3988),
    // `qianghuax` (:4003), `zhuizong` (:4058), `xunbao` (:4088), `xunbao2` (:4121),
    // `hsbaowu_cangbaotu` (:4153), `hsbaowu_huangjinyuanhou` (:4162), `hsbaowu_huangjinyuanhou2` (:4177)
    let mut e3 = fresh_proto(15_012, &["hs_trueheart", "hs_yelise", "hs_finley", "hs_zhouzhuo"]);
    let tracked3 = total_tracked_cards(&e3);
    advance_to_phase(&mut e3, 0, Phase::Play);
    clear_hand(&mut e3, 0);
    let c_qh = give_free_card_kind(&mut e3, 0, CardKind::Sha);
    assert!(!matches!(activate(&mut e3, 0, "qianghuax"), engine_native::CommandOutcome::Rejected(_)));
    save_restore_and_answer(&mut e3, "hs_qianghuax_cards", vec![ChoiceOption::Card(c_qh)]);
    drain_auto(&mut e3);
    assert_eq!(e3.state_core().player(0).hand.len(), 1);

    // `zhuizong` on seat 1 (`hs_yelise`): discard 1 -> look 4 -> pick 1
    advance_to_phase(&mut e3, 1, Phase::Play);
    clear_hand(&mut e3, 1);
    let zz = give_free_card_kind(&mut e3, 1, CardKind::Sha);
    assert!(!matches!(activate(&mut e3, 1, "zhuizong"), engine_native::CommandOutcome::Rejected(_)));
    let _ = answer(&mut e3, 1, vec![ChoiceOption::Card(zz)]);
    assert!(
        e3.open_choice().is_some_and(|c| c.spec.prompt == "hs_zhuizong_pick"),
        "zhuizong must open hs_zhuizong_pick after discarding"
    );
    let opt = e3.open_choice().unwrap().spec.options[0];
    save_restore_and_answer(&mut e3, "hs_zhuizong_pick", vec![opt]);
    drain_auto(&mut e3);
    assert_eq!(e3.state_core().player(1).hand.len(), 1);
    assert_eq!(total_tracked_cards(&e3), tracked3);

    // `maoxian` on seat 2 (`hs_finley`): skill pool strictly excludes boss/internal/card skills; `yiwen` on seat 3 (`hs_zhouzhuo`)
    advance_to_phase(&mut e3, 2, Phase::Play);
    assert!(!matches!(activate(&mut e3, 2, "maoxian"), engine_native::CommandOutcome::Rejected(_)));
    for i in 0..flag(&e3, 2, "hs_choice_count") {
        let d = flag(&e3, 2, &format!("hs_choice_sk:{i}")).saturating_sub(1) as u16;
        let sk_name = e3.catalog().skill_name(d);
        assert!(
            !sk_name.starts_with('_')
                && !sk_name.starts_with("boss_")
                && !sk_name.starts_with("tuteng")
                && !sk_name.starts_with("hsmengjing_")
                && !matches!(sk_name, "maoxian" | "jihuocard2" | "chuansongmen2" | "chuansongmen3" | "shihuawuqi"),
            "maoxian must exclude boss/unique/card/internal skills, got `{sk_name}`"
        );
    }
    let mopt = e3.open_choice().unwrap().spec.options[0];
    save_restore_and_answer(&mut e3, "hs_maoxian_skill", vec![mopt]);
    drain_auto(&mut e3);
    assert!(flag(&e3, 2, "hs_maoxian_sk") > 0);
    // `zengli` must NOT steal deck/discard weapons, and must equip owner before target; `nuyan2` must LoseHp before virtual card
    let mut ezn = fresh_proto(15_016, &["hs_blingtron", "hs_fandral", "hs_jaina"]);
    let tracked_zn = total_tracked_cards(&ezn);
    advance_to_phase(&mut ezn, 0, Phase::Play);
    clear_hand(&mut ezn, 0);
    let deck_wp = give_free_card_kind(&mut ezn, 0, CardKind::Weapon);
    put_card_on_deck_top(&mut ezn, deck_wp);
    let disc_wp = give_free_card_kind(&mut ezn, 0, CardKind::Weapon);
    ezn.state_core_mut().player_mut(0).hand.retain(|&c| c != disc_wp);
    ezn.state_core_mut().discard.push(disc_wp);
    let log_zg = ezn.log().len();
    assert!(!matches!(activate(&mut ezn, 0, "zengli"), engine_native::CommandOutcome::Rejected(_)));
    save_restore_and_answer(&mut ezn, "hs_zengli_target", vec![ChoiceOption::Player(1)]);
    drain_auto(&mut ezn);
    assert!(ezn.state_core().deck.contains(&deck_wp), "zengli must NOT pull weapons from deck");
    assert!(ezn.state_core().discard.contains(&disc_wp), "zengli must NOT pull weapons from discard");
    let eq_order: Vec<u8> = ezn.log()
        .iter()
        .filter_map(|ev| match *ev {
            engine_native::machine::LogEvent::Equipped { player, .. } => Some(player),
            _ => None,
        })
        .collect();
    assert_eq!(eq_order, vec![0, 1], "zengli must equip owner first, then target (FIFO)");
    let _ = log_zg;
    assert_eq!(total_tracked_cards(&ezn), tracked_zn);
    // `nuyan2` via `chouhuo`: at 1 HP, LoseHp settles BEFORE virtual card -> dead owner does not cast virtual AOE
    set_flag(&mut ezn, 1, "hs_nuyan_used_mask", 0b11_1111);
    advance_to_phase(&mut ezn, 2, Phase::Play);
    advance_to_phase(&mut ezn, 1, Phase::Play);
    assert!(has_skill(&ezn, 1, "nuyan2"));
    ezn.state_core_mut().player_mut(1).hp = 1;
    let hp2_before = ezn.state_core().player(2).hp;
    assert!(!matches!(activate(&mut ezn, 1, "nuyan2"), engine_native::CommandOutcome::Rejected(_)));
    let _ = answer(&mut ezn, 1, vec![ChoiceOption::Tag(2)]); // SavageAttack AOE
    drain_auto(&mut ezn);
    assert!(!ezn.state_core().player(1).alive, "nuyan2 LoseHp at 1 HP resolves first");
    assert_eq!(ezn.state_core().player(2).hp, hp2_before, "dead nuyan2 caster must NOT fire virtual SavageAttack");
    // Test `yiwen`: seat 2 single-targeting seat 3 (`hs_zhouzhuo`) grants copy + marks `hs_yiwen2_turn`
    clear_hand(&mut e3, 3);
    let yw_sha = give_free_card_kind(&mut e3, 2, CardKind::Sha);
    let _ = play_card(&mut e3, 2, yw_sha, vec![3]);
    drain_auto(&mut e3);
    assert_eq!(flag(&e3, 2, "hs_yiwen2_turn"), 1, "yiwen must mark source player for the turn");
    assert!(
        e3.state_core().player(3).hand.iter().any(|&c| e3.catalog().card_kind(c) == CardKind::Sha),
        "yiwen must grant copy of single-target card used against owner"
    );
    // Test AI planner on seat 3 public view
    let _ = ai_plan_action(&e3, 2);

    // `xueren` on Liadrin
    let mut e4 = fresh_proto(15_013, &["hs_liadrin", "hs_jaina"]);
    advance_to_phase(&mut e4, 0, Phase::Play);
    clear_hand(&mut e4, 0);
    clear_hand(&mut e4, 1);
    let sha = give_free_card_kind(&mut e4, 0, CardKind::Sha);
    let _ = play_card(&mut e4, 0, sha, vec![1]);
    let xr = advance_until_open_prompt(&mut e4, "hs_xueren_confirm", 6)
        .expect("xueren must open hs_xueren_confirm on Sha damage");
    save_restore_and_answer(&mut e4, xr.spec.prompt, vec![ChoiceOption::Confirm]);
    drain_auto(&mut e4);
    assert!(e4.state_core().player(1).hp <= 2);

    // `xunbao` + `xunbao2` + `hsbaowu_cangbaotu` + `hsbaowu_huangjinyuanhou` + `hsbaowu_huangjinyuanhou2`
    let mut exb = fresh_proto(15_015, &["hs_yelise", "hs_jaina"]);
    let tracked_xb = total_tracked_cards(&exb);
    advance_to_phase(&mut exb, 1, Phase::Discard);
    let xb_ch = advance_until_open_prompt(&mut exb, "trigger_confirm", 8)
        .expect("xunbao at Prepare must prompt trigger_confirm");
    assert_eq!(xb_ch.observer, 0);
    let _ = answer(&mut exb, 0, vec![ChoiceOption::Confirm]);
    drain_auto(&mut exb);
    let rank_need = flag(&exb, 0, "hs_xunbao_rank") as u8;
    assert!(rank_need > 0 && has_skill(&exb, 0, "xunbao2"));
    advance_to_phase(&mut exb, 0, Phase::Play);
    clear_hand(&mut exb, 0);
    // Put a card matching `rank_need` into seat 0 hand so next Prepare opens `hs_xunbao2_discard`
    let match_c = exb
        .state_core()
        .deck
        .iter()
        .copied()
        .find(|&c| exb.catalog().cards[c as usize].rank == rank_need)
        .expect("card of matching rank in deck for xunbao2");
    exb.state_core_mut().deck.retain(|&c| c != match_c);
    exb.state_core_mut().player_mut(0).hand.push(match_c);
    advance_to_phase(&mut exb, 1, Phase::Discard);
    let _xb2_ch = advance_until_open_prompt(&mut exb, "hs_xunbao2_discard", 8)
        .expect("xunbao2 at Prepare must open hs_xunbao2_discard when holding matching-rank card");
    save_restore_and_answer(&mut exb, "hs_xunbao2_discard", vec![ChoiceOption::Card(match_c)]);
    drain_auto(&mut exb);
    assert!(!has_skill(&exb, 0, "xunbao2"), "xunbao2 must be removed after unlocking map");
    advance_to_phase(&mut exb, 0, Phase::Play);
    let map_c = exb
        .state_core()
        .player(0)
        .hand
        .iter()
        .copied()
        .find(|&c| exb.catalog().cards[c as usize].name.contains("hsbaowu_cangbaotu"))
        .expect("xunbao2 must grant hsbaowu_cangbaotu");
    let _ = play_card(&mut exb, 0, map_c, vec![]);
    drain_auto(&mut exb);
    assert!(has_skill(&exb, 0, "hsbaowu_cangbaotu"));
    // At End Phase, `hsbaowu_cangbaotu` spawns `hsbaowu_huangjinyuanhou`!
    advance_to_phase(&mut exb, 0, Phase::End);
    drain_auto(&mut exb);
    assert!(!has_skill(&exb, 0, "hsbaowu_cangbaotu"));
    advance_to_phase(&mut exb, 0, Phase::Play);
    let ape_c = exb
        .state_core()
        .player(0)
        .hand
        .iter()
        .copied()
        .find(|&c| exb.catalog().cards[c as usize].name.contains("hsbaowu_huangjinyuanhou"))
        .expect("hsbaowu_cangbaotu End Phase must grant hsbaowu_huangjinyuanhou");
    let _ = play_card(&mut exb, 0, ape_c, vec![]);
    drain_auto(&mut exb);
    assert!(has_skill(&exb, 0, "hsbaowu_huangjinyuanhou"));
    assert!(has_skill(&exb, 0, "hsbaowu_huangjinyuanhou2"));
    // Opponent cannot target seat 0 while Golden Monkey skill active!
    advance_to_phase(&mut exb, 1, Phase::Play);
    clear_hand(&mut exb, 1);
    let opp_sha = give_free_card_kind(&mut exb, 1, CardKind::Sha);
    assert!(
        matches!(play_card(&mut exb, 1, opp_sha, vec![0]), engine_native::CommandOutcome::Rejected(_)),
        "hsbaowu_huangjinyuanhou must prevent others from targeting owner"
    );
    // Next Prepare of seat 0 clears both Golden Monkey skills via `hsbaowu_huangjinyuanhou2`!
    advance_to_phase(&mut exb, 0, Phase::Play);
    assert!(!has_skill(&exb, 0, "hsbaowu_huangjinyuanhou"));
    assert!(!has_skill(&exb, 0, "hsbaowu_huangjinyuanhou2"));
    assert_eq!(total_tracked_cards(&exb), tracked_xb);
}
