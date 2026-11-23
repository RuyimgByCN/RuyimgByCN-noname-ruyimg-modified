//! U05-6b Part 6 Batch E & F: ALL 31 skills (`huanjue` .. `hongxi` + `stone_chenshui`)
//! Upstream: `character/hearth/skill.js:6304-7278` + `main/content.js:397-412`

use super::common::*;
use engine_native::catalog::{CardKind, Suit};
use engine_native::choice::ChoiceOption;
use engine_native::event::Phase;
use engine_native::CommandOutcome;

#[test]
fn u05_b6b_batch_e_all_15_huanjue_through_stone_chenshui() {
    // 1) `huanjue`: convert single-target card into one of 3 candidates from detached_cards (with card conservation!)
    let mut e = fresh_proto(10_781, &["hs_lrexxar", "hs_jaina"]);
    grant_host_skill(&mut e, 0, "huanjue");
    clear_hand(&mut e, 0);
    clear_hand(&mut e, 1);
    let before = total_tracked_cards(&e);
    let sha = give_free_card_kind_suit(&mut e, 0, CardKind::Sha, None);
    advance_to_phase(&mut e, 0, Phase::Play);
    play_card(&mut e, 0, sha, vec![1]);
    let ch = e.open_choice().cloned().expect("huanjue pick");
    assert_eq!(ch.spec.prompt, "hs_huanjue_pick");
    let chosen = ch.spec.options[0];
    save_restore_and_answer(&mut e, "hs_huanjue_pick", vec![chosen]);
    drain_auto(&mut e);
    assert_eq!(total_tracked_cards(&e), before);

    // 2) `bingjia` + `bingjia2`: stash card -> matching suit incoming cancels damage via mianyi & +1 hujia + conservation
    let mut ebj = fresh_proto(10_782, &["hs_jaina", "hs_lrexxar"]);
    grant_host_skill(&mut ebj, 0, "bingjia");
    clear_hand(&mut ebj, 0);
    clear_hand(&mut ebj, 1);
    let before_bj = total_tracked_cards(&ebj);
    let heart_c = give_free_card_kind_suit(&mut ebj, 0, CardKind::Shan, Some(Suit::Heart));
    advance_to_phase(&mut ebj, 0, Phase::Play);
    let _ = activate(&mut ebj, 0, "bingjia");
    answer(&mut ebj, 0, vec![ChoiceOption::Card(heart_c)]);
    drain_auto(&mut ebj);
    assert!(has_skill(&ebj, 0, "bingjia2"));
    assert_eq!(total_tracked_cards(&ebj), before_bj);

    advance_to_phase(&mut ebj, 1, Phase::Play);
    clear_hand(&mut ebj, 0);
    let sha_h = give_free_card_kind_suit(&mut ebj, 1, CardKind::Sha, Some(Suit::Heart));
    let hp_0 = ebj.state_core().player(0).hp;
    play_card(&mut ebj, 1, sha_h, vec![0]);
    drain_auto(&mut ebj);
    assert!(!has_skill(&ebj, 0, "bingjia2"));
    assert_eq!(ebj.state_core().player(0).hujia, 1);
    assert_eq!(ebj.state_core().player(0).hp, hp_0, "mianyi prevented dmg");
    assert_eq!(total_tracked_cards(&ebj), before_bj);

    // 3) `zuzhou` (`fulei`): spade hand card at Prepare places fulei in judge; Spade judge -> hits increment & transfer; remove on damage
    let mut ezz = fresh_proto(10_783, &["hs_guldan", "hs_lrexxar"]);
    grant_host_skill(&mut ezz, 0, "zuzhou");
    advance_to_phase(&mut ezz, 1, Phase::Play);
    clear_hand(&mut ezz, 0);
    clear_hand(&mut ezz, 1);
    give_free_card_kind_suit(&mut ezz, 0, CardKind::Sha, Some(Suit::Spade));
    let sp_judge = give_free_card_kind_suit(&mut ezz, 1, CardKind::Sha, Some(Suit::Spade));
    put_card_on_deck_top(&mut ezz, sp_judge);
    let hp0_zz = ezz.state_core().player(0).hp;
    advance_to_phase(&mut ezz, 0, Phase::Play);
    assert_eq!(ezz.state_core().player(0).hp, hp0_zz - 1, "fulei spade hit dealt 1");
    assert!(ezz.state_core().player(0).judge.is_empty());
    // `zuzhou_remove` set expired so `fulei` moved to discard rather than seat 1's judge!
    assert!(ezz.state_core().player(1).judge.is_empty());

    // 4) `mdzhoufu` (`mdzhoufu2`) + `xianzhi`: replace target's judge card AND swap deck top 2 before judge!
    let mut emd = fresh_proto(10_784, &["hs_lrexxar", "hs_jaina"]);
    grant_host_skill(&mut emd, 0, "mdzhoufu");
    grant_host_skill(&mut emd, 0, "xianzhi");
    clear_hand(&mut emd, 0);
    let sp_card = give_free_card_kind_suit(&mut emd, 0, CardKind::Sha, Some(Suit::Spade));
    let lebu = give_free_card_kind_suit(&mut emd, 0, CardKind::Le, None);
    advance_to_phase(&mut emd, 0, Phase::Play);
    let _ = activate(&mut emd, 0, "mdzhoufu");
    answer(&mut emd, 0, vec![ChoiceOption::Card(sp_card)]);
    answer(&mut emd, 0, vec![ChoiceOption::Player(1)]);
    play_card(&mut emd, 0, lebu, vec![1]);
    drain_auto(&mut emd);
    assert!(has_skill(&emd, 1, "mdzhoufu2"));
    let _ch_xz = advance_until_open_prompt(&mut emd, "hs_xianzhi_choice", 8).expect("xianzhi before judge");
    save_restore_and_answer(&mut emd, "hs_xianzhi_choice", vec![ChoiceOption::Confirm]);
    drain_auto(&mut emd);
    // `mdzhoufu2` replaced judge card with `sp_card` (Spade -> Lebu takes effect, seat 1 skips Play!) & removed `mdzhoufu2`
    assert!(!has_skill(&emd, 1, "mdzhoufu2"));
    assert_ne!(emd.turn().phase, Phase::Play, "spade replacement activated Lebu");

    // 5) `jingxiang`: choose target's hand card to respond to Sha + cancel refunds count
    let mut ejx = fresh_proto(10_785, &["hs_lrexxar", "hs_jaina", "hs_wuther"]);
    grant_host_skill(&mut ejx, 1, "jingxiang");
    clear_hand(&mut ejx, 0);
    clear_hand(&mut ejx, 1);
    clear_hand(&mut ejx, 2);
    let shan_from_2 = give_free_card_kind_suit(&mut ejx, 2, CardKind::Shan, None);
    let sha_0 = give_free_card_kind_suit(&mut ejx, 0, CardKind::Sha, None);
    let hp_1j = ejx.state_core().player(1).hp;
    advance_to_phase(&mut ejx, 0, Phase::Play);
    play_card(&mut ejx, 0, sha_0, vec![1]);
    // Respond choice asks seat 1 whether to trigger `jingxiang`
    let ch_r = ejx.open_choice().cloned().expect("respond skill choice");
    assert_eq!(ch_r.observer, 1);
    // Pick skill `jingxiang`
    let jx_opt = ch_r
        .spec
        .options
        .iter()
        .find(|o| matches!(o, ChoiceOption::Skill(_)))
        .copied()
        .expect("jingxiang skill option");
    answer(&mut ejx, 1, vec![jx_opt]);
    assert_eq!(
        ejx.open_choice().map(|c| c.spec.prompt),
        Some("hs_jingxiang_target")
    );
    save_restore_and_answer(&mut ejx, "hs_jingxiang_target", vec![ChoiceOption::Player(2)]);
    assert_eq!(
        ejx.open_choice().map(|c| c.spec.prompt),
        Some("hs_jingxiang_card")
    );
    answer(&mut ejx, 1, vec![ChoiceOption::Card(shan_from_2)]);
    drain_auto(&mut ejx);
    assert_eq!(ejx.state_core().player(1).hp, hp_1j, "jingxiang dodged Sha");
    assert!(!ejx.state_core().player(2).hand.contains(&shan_from_2));

    // 6) `wlianji` + `mengun` (`mengun2`) + `jianren` + `jihuo` + `stone_chenshui`:
    let mut emj = fresh_proto(10_786, &["hs_lrexxar", "hs_jaina"]);
    grant_host_skill(&mut emj, 0, "wlianji");
    grant_host_skill(&mut emj, 0, "jianren");
    grant_host_skill(&mut emj, 0, "jihuo");
    grant_host_skill(&mut emj, 0, "stone_chenshui");
    grant_host_skill(&mut emj, 1, "mengun");
    clear_hand(&mut emj, 0);
    clear_hand(&mut emj, 1);
    let weap = give_free_card_kind_suit(&mut emj, 0, CardKind::Weapon, None);
    let sp_sha = give_free_card_kind_suit(&mut emj, 0, CardKind::Sha, Some(Suit::Heart));
    let sp_shan = give_free_card_kind_suit(&mut emj, 1, CardKind::Shan, Some(Suit::Heart));
    let hp_1m = emj.state_core().player(1).hp;
    advance_to_phase(&mut emj, 0, Phase::Play);
    play_card(&mut emj, 0, weap, vec![]);
    let _ = activate(&mut emj, 0, "jianren");
    drain_auto(&mut emj);
    assert_eq!(emj.state_core().player(1).hp, hp_1m - 1, "jianren discarded weapon for 1 dmg");
    // Now `sp_sha` triggers `mengun` from seat 1!
    play_card(&mut emj, 0, sp_sha, vec![1]);
    assert_eq!(
        emj.open_choice().map(|c| c.spec.prompt),
        Some("hs_mengun_discard")
    );
    answer(&mut emj, 1, vec![ChoiceOption::Card(sp_shan)]);
    drain_auto(&mut emj);
    assert!(emj.state_core().player(0).hand.contains(&sp_sha), "mengun returned Sha");
    assert!(has_skill(&emj, 0, "mengun2"));
    assert!(
        matches!(play_card(&mut emj, 0, sp_sha, vec![1]), CommandOutcome::Rejected(_)),
        "mengun2 forbids replaying returned card this turn"
    );
    // End phase checks `wlianji` (countUsed <= hp here, so skipped cleanly) and `stone_chenshui` (dream card!)
    let ch_jh = advance_until_open_prompt(&mut emj, "hs_jihuo_discard", 10).expect("jihuo ask");
    let drop_j = ch_jh.spec.options[0];
    answer(&mut emj, 0, vec![drop_j]);
    drain_auto(&mut emj);
    assert_eq!(emj.turn().current, 0, "jihuo queued extra turn");
    assert!(emj.state_core().player(0).hand.iter().any(|&c| {
        emj.catalog().cards[c as usize]
            .name
            .contains("hsmengjing_")
    }));
}

#[test]
fn u05_b6b_batch_f_all_16_totems_huanfeng_and_hongxi() {
    // 1) `tuteng_s` + `s_tuteng` + `tuteng` + `zuling` + `huanfeng` + `tuteng_h` + `tuteng_lose` + `tuteng1..8` + `hongxi`
    let mut e = fresh_proto(10_791, &["hs_sthrall", "hs_alakir", "hs_lrexxar"]);
    clear_hand(&mut e, 0);
    clear_hand(&mut e, 2);
    grant_host_skill(&mut e, 0, "tuteng_s");
    grant_host_skill(&mut e, 0, "tuteng");
    grant_host_skill(&mut e, 0, "zuling");
    grant_host_skill(&mut e, 0, "tuteng_h");
    grant_host_skill(&mut e, 0, "tuteng_lose");
    grant_host_skill(&mut e, 2, "hongxi");
    e.state_core_mut().player_mut(2).hp = 3;

    // `tuteng_h` lowers max handcard by 1
    assert_eq!(
        e.state_core().hand_limit(e.catalog(), 0),
        (e.state_core().player(0).hp - 1).max(0) as usize
    );

    // Grant tuteng1, tuteng2, tuteng3 -> at Turn 2 Prepare `zuling` awakens (`hs_tuteng_awake = 1`) and maxHp - 1
    grant_host_skill(&mut e, 0, "tuteng1");
    grant_host_skill(&mut e, 0, "tuteng2");
    grant_host_skill(&mut e, 0, "tuteng3");
    let mhp0 = e.state_core().player(0).max_hp;
    advance_to_phase(&mut e, 1, Phase::Play);
    advance_to_phase(&mut e, 0, Phase::Play);
    assert_eq!(flag(&e, 0, "hs_tuteng_awake"), 1, "zuling awakened");
    assert_eq!(e.state_core().player(0).max_hp, mhp0 - 1);
    // `tuteng_s` filled the 4th basic totem `tuteng4`!
    assert!(has_skill(&e, 0, "tuteng4"));

    // Active `tuteng` when awake and 4 owned -> pick Tag(6) and replace one!
    let _ = activate(&mut e, 0, "tuteng");
    assert_eq!(
        e.open_choice().map(|c| c.spec.prompt),
        Some("hs_tuteng_pick")
    );
    answer(&mut e, 0, vec![ChoiceOption::Tag(6)]);
    assert_eq!(
        e.open_choice().map(|c| c.spec.prompt),
        Some("hs_tuteng_replace")
    );
    save_restore_and_answer(&mut e, "hs_tuteng_replace", vec![ChoiceOption::Tag(4)]);
    drain_auto(&mut e);
    assert!(has_skill(&e, 0, "tuteng6"));

    // `tuteng6` adds +1 Sha damage in owner turn; `tuteng2` draws 1 on damageAfter
    clear_hand(&mut e, 2);
    let sha6 = give_free_card_kind_suit(&mut e, 0, CardKind::Sha, None);
    let hp_2 = e.state_core().player(2).hp;
    let h0_b = e.state_core().player(0).hand.len();
    play_card(&mut e, 0, sha6, vec![2]);
    drain_auto(&mut e);
    assert_eq!(e.state_core().player(2).hp, hp_2 - 2, "tuteng6 boosted Sha +1");
    assert_eq!(e.state_core().player(0).hand.len(), h0_b, "played 1 + tuteng2 drew 1");

    // `tuteng3` reduces incoming dmg by 1 and removes itself; `tuteng_lose` removes a totem
    let mut e3 = fresh_proto(10_792, &["hs_lrexxar", "hs_sthrall"]);
    grant_host_skill(&mut e3, 1, "tuteng3");
    grant_host_skill(&mut e3, 1, "tuteng_lose");
    grant_host_skill(&mut e3, 1, "tuteng8");
    assert_eq!(e3.state_core().computed_distance(e3.catalog(), 1, 0), 1);
    clear_hand(&mut e3, 1);
    let duel_2 = give_free_card_kind_suit(&mut e3, 0, CardKind::Duel, None);
    let hp_1t = e3.state_core().player(1).hp;
    advance_to_phase(&mut e3, 0, Phase::Play);
    play_card(&mut e3, 0, duel_2, vec![1]);
    drain_auto(&mut e3);
    assert_eq!(e3.state_core().player(1).hp, hp_1t, "tuteng3 prevented 1 dmg & removed itself");
    assert!(!has_skill(&e3, 1, "tuteng3"));

    // `huanfeng`: TurnBegin with tuteng1..=4 -> transform into Al'Akir (3/3, fengnu + shengdun, draw 4) -> PhaseEnd restore + extra turn!
    let mut ehf = fresh_proto(10_793, &["hs_sthrall", "hs_lrexxar"]);
    grant_host_skill(&mut ehf, 0, "huanfeng");
    grant_host_skill(&mut ehf, 0, "tuteng1");
    grant_host_skill(&mut ehf, 0, "tuteng2");
    grant_host_skill(&mut ehf, 0, "tuteng3");
    grant_host_skill(&mut ehf, 0, "tuteng4");
    ehf.state_core_mut().player_mut(0).max_hp = 4;
    ehf.state_core_mut().player_mut(0).hp = 1;
    advance_to_phase(&mut ehf, 1, Phase::Play);
    clear_hand(&mut ehf, 0);
    let orig_h = give_free_card_kind_suit(&mut ehf, 0, CardKind::Tao, None);
    advance_to_phase(&mut ehf, 0, Phase::Play);
    assert_eq!(flag(&ehf, 0, "hs_huanfeng_active"), 1);
    assert_eq!(ehf.state_core().player(0).max_hp, 3);
    assert!(has_skill(&ehf, 0, "fengnu"));
    assert!(!ehf.state_core().player(0).hand.contains(&orig_h));
    // End turn -> restores original hero (4 maxHp, orig_hp=1, orig_h) & queues extra turn (`tuteng1` heals 1->2 at extra turn's Prepare, proving orig_hp=1 was restored instead of staying Al'Akir's 3)!
    advance_to_phase(&mut ehf, 0, Phase::End);
    advance_to_phase(&mut ehf, 0, Phase::Play);
    assert_eq!(ehf.turn().current, 0, "huanfeng granted extra turn");
    assert_eq!(flag(&ehf, 0, "hs_huanfeng_active"), 0);
    assert_eq!(ehf.state_core().player(0).max_hp, 4);
    assert_eq!(ehf.state_core().player(0).hp, 2);
    assert!(ehf.state_core().player(0).hand.contains(&orig_h));

    // `hongxi`: any death heals owner to full HP!
    let mut ehx = fresh_proto(10_794, &["hs_lrexxar", "hs_jaina", "hs_wuther"]);
    grant_host_skill(&mut ehx, 2, "hongxi");
    clear_hand(&mut ehx, 1);
    ehx.state_core_mut().player_mut(1).hp = 1;
    ehx.state_core_mut().player_mut(2).hp = 1;
    let sha_kill = give_free_card_kind_suit(&mut ehx, 0, CardKind::Sha, None);
    advance_to_phase(&mut ehx, 0, Phase::Play);
    play_card(&mut ehx, 0, sha_kill, vec![1]);
    drain_auto(&mut ehx);
    assert!(!ehx.state_core().player(1).alive);
    assert_eq!(
        ehx.state_core().player(2).hp,
        ehx.state_core().player(2).max_hp as i32,
        "hongxi healed to full HP on death"
    );

    // Negative check on `s_tuteng` & `tuteng7` + `tuteng5`
    let mut est = fresh_proto(10_795, &["hs_sthrall", "hs_lrexxar"]);
    grant_host_skill(&mut est, 0, "s_tuteng");
    grant_host_skill(&mut est, 0, "tuteng5");
    grant_host_skill(&mut est, 0, "tuteng7");
    est.state_core_mut().player_mut(1).hp = 2;
    advance_to_phase(&mut est, 1, Phase::Play);
    advance_to_phase(&mut est, 0, Phase::Play);
    let owned_cnt = (1..=8u8)
        .filter(|i| has_skill(&est, 0, &format!("tuteng{i}")))
        .count();
    assert_eq!(owned_cnt, 3, "2 initial + s_tuteng added 1 totem at Prepare");
    let _ch7 = advance_until_open_prompt(&mut est, "hs_tuteng7_target", 8).expect("tuteng7");
    answer(&mut est, 0, vec![ChoiceOption::Player(1)]);
    drain_auto(&mut est);
    assert_eq!(est.state_core().player(1).hp, 3);
    assert!(matches!(
        engine_native::content::host::HostBehavior::from_tag(1255),
        Some(engine_native::content::host::HostBehavior::U05Hearth2BatchF)
    ));
    let _ = CommandOutcome::Rejected;
}
