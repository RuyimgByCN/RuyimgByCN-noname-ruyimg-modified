//! Batch L11-3 Group B (`extension_buff.js:231..431`, `extension_skill.js:2038..2529`) real-path tests:
//! `dunjiazhoufa`, `ziyideyongzhan`, `yanhuozhiren`, `jingleizhiren`, `yingjicuoshijia`,
//! `yingjicuoshiyi`, `xihuozhikai`, `dingleizhikai`, `yueyingdejiqiao`, `taipingzhoufa`,
//! `baijujianchu`, `juejingzhice`, `xumingshenyao`, `wuweizhile`, `yuanduandilue`,
//! `jiezizhidao`, `huangtiannizhuan`, `dongchazhiyan`, `mingjingzhiyan`, `juxiangzhifeng`.

use super::common::*;
use engine_native::catalog::{CardKind, Suit};
use engine_native::view::HandView;
use engine_native::{ChoiceOption, CommandOutcome, GameAction, Phase};

#[test]
fn l11_b3_dunjiazhoufa_and_taipingzhoufa_and_leijimu() {
    // `extension_skill.js:2038..2057` (`_buff_txhj_dunjiazhoufa`): weapon + elemental Sha -> directHit; Normal Sha respondable!
    // `extension_skill.js:2247..2268` (`_buff_txhj_taipingzhoufa`): elemental Sha (incl. `txhj_leijimu`) +1 damage!
    let mut e = start_versus(311, &[&[0], &[1]]);
    grant_skill(&mut e, 0, "_buff_txhj_dunjiazhoufa");
    grant_skill(&mut e, 0, "_buff_txhj_taipingzhoufa");
    grant_skill(&mut e, 0, engine_native::vocabulary::names::PAO_XIAO);

    // Without weapon: FireSha deals +1 via taipingzhoufa, but dunjiazhoufa is inactive -> target CAN dodge with Shan!
    e.debug_set_turn(0, Phase::Play, 1);
    let _shan1 = place_kind(&mut e, 1, CardKind::Shan);
    let fsha0 = place_kind(&mut e, 0, CardKind::FireSha);
    let _ = play_card(&mut e, 0, fsha0, vec![1]);
    let ch = expect_pending(&e, "respond_shan");
    let _ = answer(&mut e, &ch, vec![ChoiceOption::Card(_shan1)]);
    drain(&mut e);
    assert_eq!(e.state_core().player(1).hp, 6);

    // With normal weapon:
    // (a) Normal Sha -> NOT elemental -> NO taipingzhoufa +1 and STILL respondable by Shan!
    equip_kind(&mut e, 0, CardKind::Weapon);
    let _shan2 = place_kind(&mut e, 1, CardKind::Shan);
    let nsha = place_kind(&mut e, 0, CardKind::Sha);
    let _ = play_card(&mut e, 0, nsha, vec![1]);
    let ch2 = expect_pending(&e, "respond_shan");
    let _ = answer(&mut e, &ch2, vec![ChoiceOption::Pass]);
    drain(&mut e);
    assert_eq!(
        e.state_core().player(1).hp,
        5,
        "Normal Sha must remain respondable and NOT gain taipingzhoufa +1"
    );

    // (b) FireSha with weapon -> BOTH dunjiazhoufa (no Bagua/Shan prompt!) AND taipingzhoufa (+1 => 2 dmg!) apply!
    equip_kind(&mut e, 1, CardKind::Armor);
    let fsha1 = place_kind(&mut e, 0, CardKind::FireSha);
    top_deck_suit(&mut e, Suit::Heart);
    let out_f = play_card(&mut e, 0, fsha1, vec![1]);
    assert!(
        matches!(out_f, CommandOutcome::Accepted { .. }),
        "FireSha with weapon + dunjiazhoufa must directHit skipping Bagua and respond_shan: {out_f:?}"
    );
    assert_eq!(e.state_core().player(1).hp, 3);

    // (c) `txhj_leijimu` converts Normal Sha to LeiSha -> `taipingzhoufa` adds +1 (and `dunjiazhoufa` skips dodge)!
    let _ = equip_named(&mut e, 0, "txhj_leijimu");
    let _shan3 = place_kind(&mut e, 1, CardKind::Shan);
    let sha_conv = place_kind(&mut e, 0, CardKind::Sha);
    let _ = play_card(&mut e, 0, sha_conv, vec![1]);
    let leijimu_c = expect_pending(&e, "txhj_leijimu:confirm");
    let _ = answer(&mut e, &leijimu_c, vec![ChoiceOption::Confirm]);
    drain(&mut e);
    assert_eq!(
        e.state_core().player(1).hp,
        1,
        "txhj_leijimu-converted LeiSha must trigger taipingzhoufa (+1 => 2 dmg)"
    );
}

#[test]
fn l11_b3_ziyideyongzhan_yanhuozhiren_jingleizhiren_xihuozhikai_dingleizhikai() {
    // `ziyideyongzhan`: ONLY 2nd Sha in Play with weapon deals +1 dmg (no extra Sha count or target!);
    // `yanhuozhiren` / `jingleizhiren`: weapon + non-chain Fire/Thunder +1 dmg (+ chiyanzhenhunqin combo!);
    // `xihuozhikai` / `dingleizhikai`: armor + Fire/Thunder damage -1 (2->1, 1->0).
    let mut e = start_versus(312, &[&[0], &[1, 2]]);
    grant_skill(&mut e, 0, "_buff_txhj_ziyideyongzhan");
    grant_skill(&mut e, 0, "_buff_txhj_yanhuozhiren");
    grant_skill(&mut e, 0, "_buff_txhj_jingleizhiren");
    grant_skill(&mut e, 0, engine_native::vocabulary::names::PAO_XIAO);
    grant_skill(&mut e, 1, "_buff_txhj_xihuozhikai");
    grant_skill(&mut e, 2, "_buff_txhj_dingleizhikai");

    e.debug_set_turn(0, Phase::Play, 1);
    equip_std_kind(&mut e, 0, CardKind::Weapon);
    // Target 1 has NO armor yet, so xihuozhikai inactive!
    // 1st Sha: NO ziyideyongzhan (+1) -> 1 dmg
    let s1 = place_kind(&mut e, 0, CardKind::Sha);
    let _ = play_card(&mut e, 0, s1, vec![1]);
    drain(&mut e);
    assert_eq!(e.state_core().player(1).hp, 5);

    // 2nd Sha: ziyideyongzhan (+1 => 2 dmg)!
    let s2 = place_kind(&mut e, 0, CardKind::Sha);
    let _ = play_card(&mut e, 0, s2, vec![1]);
    drain(&mut e);
    assert_eq!(e.state_core().player(1).hp, 3, "2nd Sha with weapon gets ziyi +1 dmg");

    // 3rd Sha: NO ziyideyongzhan (+1) -> 1 dmg!
    let s3 = place_kind(&mut e, 0, CardKind::Sha);
    let _ = play_card(&mut e, 0, s3, vec![1]);
    drain(&mut e);
    assert_eq!(e.state_core().player(1).hp, 2, "3rd Sha must NOT get ziyi +1 dmg");

    // Equip armor on seat 1 & seat 2: Fire damage 2 (1 + yanhuozhiren) - xihuozhikai 1 = 1!
    // Chained damage onto seat 2 (with armor & dingleizhikai): chain carries 1 fire dmg WITHOUT yanhuozhiren re-adding +1!
    e.debug_set_hp(1, 6);
    e.debug_set_hp(2, 6);
    equip_std_kind(&mut e, 1, CardKind::Armor);
    equip_std_kind(&mut e, 2, CardKind::Armor);
    e.state_core_mut().player_mut(1).linked = true;
    e.state_core_mut().player_mut(2).linked = true;
    e.debug_set_turn(0, Phase::Play, 2);
    e.state_core_mut()
        .vm_aux
        .flags
        .remove(&(0, "l11_phase_sha_count".to_string()));
    let fsha = place_kind(&mut e, 0, CardKind::FireSha);
    let _ = play_card(&mut e, 0, fsha, vec![1]);
    drain(&mut e);
    assert_eq!(e.state_core().player(1).hp, 5);
    assert_eq!(e.state_core().player(2).hp, 5);

    // Thunder damage 2 (1 + jingleizhiren) - dingleizhikai (armor!) 1 = 1!
    e.debug_set_turn(0, Phase::Play, 3);
    e.state_core_mut()
        .vm_aux
        .flags
        .remove(&(0, "l11_phase_sha_count".to_string()));
    let lsha = place_kind(&mut e, 0, CardKind::LeiSha);
    let _ = play_card(&mut e, 0, lsha, vec![2]);
    drain(&mut e);
    assert_eq!(e.state_core().player(2).hp, 4);

    // `txhj_chiyanzhenhunqin` (`priority 5` -> Fire) + `yanhuozhiren` (`priority 4` -> +1): Normal Sha becomes 2 Fire dmg!
    let _ = equip_named(&mut e, 0, "txhj_chiyanzhenhunqin");
    e.debug_set_hp(1, 6);
    e.state_core_mut().player_mut(1).equips.clear(); // no armor
    e.debug_set_turn(0, Phase::Play, 4);
    e.state_core_mut()
        .vm_aux
        .flags
        .remove(&(0, "l11_phase_sha_count".to_string()));
    let sha_fire = place_kind(&mut e, 0, CardKind::Sha);
    let _ = play_card(&mut e, 0, sha_fire, vec![1]);
    drain(&mut e);
    assert_eq!(
        e.state_core().player(1).hp,
        4,
        "chiyanzhenhunqin -> Fire + yanhuozhiren -> 2 fire damage"
    );
}

#[test]
fn l11_b3_yingjicuoshijia_yi_optional_and_ai_and_tieji_seal() {
    // `extension_skill.js:2137..2183`: optional (`frequent: true`), triggers on losing Armor (equip OR hand),
    // supports Decline/Confirm, respects `tieji_sealed`, and AI declines `cuoshijia` when full HP.
    let mut e = start_versus(313, &[&[0], &[1]]);
    grant_skill(&mut e, 0, "buff_txhj_yingjicuoshijia"); // intentionally test bare buff_* variant too!
    grant_skill(&mut e, 0, "_buff_txhj_yingjichuoshiyi");

    // 1. Snatch Armor equip from injured seat 0 -> triggers BOTH optional prompts ONCE each!
    equip_std_kind(&mut e, 0, CardKind::Armor);
    e.debug_clear_hand(0);
    e.debug_set_hp(0, 4);
    e.debug_set_turn(1, Phase::Play, 1);
    let snatch = place_kind(&mut e, 1, CardKind::Snatch);
    let _ = play_card(&mut e, 1, snatch, vec![0]);

    // Optional triggers prompt `trigger_confirm`; AI should Confirm both (injured cuoshijia + draw-2 cuoshiyi)
    let c1 = expect_pending(&e, "trigger_confirm");
    assert!(matches!(ai_plan(&e, 0), GameAction::Answer { ref selection, .. } if selection == &[ChoiceOption::Confirm]));
    let _ = answer(&mut e, &c1, vec![ChoiceOption::Confirm]);
    let c2 = expect_pending(&e, "trigger_confirm");
    assert!(matches!(ai_plan(&e, 0), GameAction::Answer { ref selection, .. } if selection == &[ChoiceOption::Confirm]));
    let _ = answer(&mut e, &c2, vec![ChoiceOption::Confirm]);
    drain(&mut e);
    assert_eq!(e.state_core().player(0).hp, 5, "yingjicuoshijia recovers 1 HP (no double-fire)");
    assert_eq!(e.state_core().player(0).hand.len(), 2, "yingjicuoshiyi draws 2 cards (no double-fire)");

    // 2. At full HP, AI refuses `yingjicuoshijia` (Pass); manual Pass works too:
    equip_std_kind(&mut e, 0, CardKind::Armor);
    e.debug_clear_hand(0);
    e.debug_set_hp(0, 6);
    let dis = place_kind(&mut e, 1, CardKind::Dismantle);
    let _ = play_card(&mut e, 1, dis, vec![0]);
    let d1 = expect_pending(&e, "trigger_confirm");
    // Depending on queue order, if cuoshijia is first, AI must Pass at full HP
    if let Some(engine_native::view::ChoiceView::Own {
        source_skill: Some(sk),
        ..
    }) = e.choice_view_for(Some(0))
    {
        if sk.contains("cuoshijia") || sk.contains("chuoshijia") {
            assert!(matches!(
                ai_plan(&e, 0),
                GameAction::Answer { ref selection, .. } if selection == &[ChoiceOption::Pass]
            ));
        }
    }
    let _ = answer(&mut e, &d1, vec![ChoiceOption::Pass]);
    drain(&mut e);
    assert_eq!(e.state_core().player(0).hand.len(), 0);

    // 3. `tieji_sealed` disables optional `yingjicuoshijia` / `yingjicuoshiyi`
    equip_std_kind(&mut e, 0, CardKind::Armor);
    e.debug_clear_hand(0);
    e.debug_set_hp(0, 4);
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((0, "fz_new_tieji_target_sealed".to_string()), 1);
    let dis2 = place_kind(&mut e, 1, CardKind::Dismantle);
    let out = play_card(&mut e, 1, dis2, vec![0]);
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "expected Accepted, got {out:?}");
    assert_eq!(e.state_core().player(0).hp, 4);
    assert_eq!(e.state_core().player(0).hand.len(), 0);
    e.state_core_mut()
        .vm_aux
        .flags
        .remove(&(0, "fz_new_tieji_target_sealed".to_string()));

    // 4. Loss of Armor from HAND via `hand_guohe`, `hand_snatch`, `hand_equip`, and `hand_discard`
    // MUST trigger BOTH `yingjicuoshijia` & `yingjicuoshiyi` ONCE each (never 0, never double-fire):
    // (a) `hand_guohe`:
    let _h_armor1 = place_std_kind(&mut e, 0, CardKind::Armor);
    let dis_h = place_kind(&mut e, 1, CardKind::Dismantle);
    let _ = play_card(&mut e, 1, dis_h, vec![0]);
    let hg1 = expect_pending(&e, "trigger_confirm");
    let _ = answer(&mut e, &hg1, vec![ChoiceOption::Confirm]);
    let hg2 = expect_pending(&e, "trigger_confirm");
    let _ = answer(&mut e, &hg2, vec![ChoiceOption::Confirm]);
    assert!(e.open_choice().is_none(), "hand_guohe must NOT double-fire");
    assert_eq!(e.state_core().player(0).hp, 5, "hand_guohe recovers 1 HP");
    assert_eq!(e.state_core().player(0).hand.len(), 2, "hand_guohe draws 2 cards");

    // (b) `hand_snatch`:
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_set_hp(0, 4);
    let _h_armor2 = place_std_kind(&mut e, 0, CardKind::Armor);
    let sn_h = place_kind(&mut e, 1, CardKind::Snatch);
    let _ = play_card(&mut e, 1, sn_h, vec![0]);
    let hs1 = expect_pending(&e, "trigger_confirm");
    let _ = answer(&mut e, &hs1, vec![ChoiceOption::Confirm]);
    let hs2 = expect_pending(&e, "trigger_confirm");
    let _ = answer(&mut e, &hs2, vec![ChoiceOption::Confirm]);
    assert!(e.open_choice().is_none(), "hand_snatch must NOT double-fire");
    assert_eq!(e.state_core().player(0).hp, 5, "hand_snatch recovers 1 HP");
    assert_eq!(e.state_core().player(0).hand.len(), 2, "hand_snatch draws 2 cards");

    // (c) `hand_equip`: equipping Armor from hand onto empty armor slot triggers BOTH once:
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_set_hp(0, 4);
    e.debug_set_turn(0, Phase::Play, 2);
    let eq_hand = place_std_kind(&mut e, 0, CardKind::Armor);
    let _ = play_card(&mut e, 0, eq_hand, Vec::new());
    let he1 = expect_pending(&e, "trigger_confirm");
    let _ = answer(&mut e, &he1, vec![ChoiceOption::Confirm]);
    let he2 = expect_pending(&e, "trigger_confirm");
    let _ = answer(&mut e, &he2, vec![ChoiceOption::Confirm]);
    assert!(e.open_choice().is_none(), "hand_equip must NOT double-fire");
    assert_eq!(e.state_core().player(0).hp, 5, "hand_equip recovers 1 HP");
    assert_eq!(e.state_core().player(0).hand.len(), 2, "hand_equip draws 2 cards");

    // (d) `hand_discard` in Discard phase: discarding Armor from hand triggers BOTH once (not 2x from CardsLost + CardsDiscarded!):
    e.state_core_mut().player_mut(0).equips.clear();
    e.debug_clear_hand(0);
    e.debug_set_hp(0, 1);
    let d_armor = place_std_kind(&mut e, 0, CardKind::Armor);
    let _d_junk = place_kind(&mut e, 0, CardKind::Sha);
    let _ = next_phase(&mut e); // Phase::Discard (hand=2, limit=1 -> discard_to_limit)
    let disc_pick = expect_pending(&e, "discard_to_limit");
    let _ = answer(&mut e, &disc_pick, vec![ChoiceOption::Card(d_armor)]);
    let hd1 = expect_pending(&e, "trigger_confirm");
    let _ = answer(&mut e, &hd1, vec![ChoiceOption::Confirm]);
    let hd2 = expect_pending(&e, "trigger_confirm");
    let _ = answer(&mut e, &hd2, vec![ChoiceOption::Confirm]);
    drain(&mut e);
    assert_eq!(e.state_core().player(0).hp, 2, "hand_discard recovers 1 HP (once, not 2)");
    assert_eq!(e.state_core().player(0).hand.len(), 3, "hand_discard keeps 1 + draws 2 (once, not 4)");
}

#[test]
fn l11_b3_baijujianchu_juejingzhice_xumingshenyao() {
    // `baijujianchu` (`2271`): ONLY Normal Sha damage discards 1 target 'he' (NOT FireSha / Duel);
    // `juejingzhice` (`2295`): ONLY after last Shan (hand empty) draws 2;
    // `xumingshenyao` (`2318`): ONLY dying self-Tao (`hp <= 0`, `player == owner`) heals extra +1 (`0 -> 2`),
    // NOT ally-Tao on dying owner and NOT healthy self-Tao!
    let mut e = start_versus(314, &[&[0, 2], &[1]]);
    grant_skill(&mut e, 0, "_buff_txhj_baijujianchu");
    grant_skill(&mut e, 1, "_buff_txhj_juejingzhice");
    grant_skill(&mut e, 0, "_buff_txhj_xumingshenyao");

    // 1. `juejingzhice`: seat 1 has 2 cards [Shan, Tao] -> plays Shan (hand len 1 != 0) -> NO draw!
    e.debug_set_turn(0, Phase::Play, 1);
    let shan_a = place_kind(&mut e, 1, CardKind::Shan);
    let _tao_keep = place_kind(&mut e, 1, CardKind::Tao);
    let sha1 = place_kind(&mut e, 0, CardKind::Sha);
    let _ = play_card(&mut e, 0, sha1, vec![1]);
    let ch1 = expect_pending(&e, "respond_shan");
    let _ = answer(&mut e, &ch1, vec![ChoiceOption::Card(shan_a)]);
    drain(&mut e);
    assert_eq!(
        e.state_core().player(1).hand.len(),
        1,
        "juejingzhice must NOT trigger when hand still has cards"
    );

    // Clear seat 1's hand down to ONLY 1 Shan -> plays that last Shan -> draws 2!
    e.debug_clear_hand(1);
    let shan_b = place_kind(&mut e, 1, CardKind::Shan);
    let sha2 = place_kind(&mut e, 0, CardKind::Sha);
    e.debug_set_turn(0, Phase::Play, 2);
    e.state_core_mut().turn.sha_used_play_phase = false;
    let _ = play_card(&mut e, 0, sha2, vec![1]);
    let ch2 = expect_pending(&e, "respond_shan");
    let _ = answer(&mut e, &ch2, vec![ChoiceOption::Card(shan_b)]);
    drain(&mut e);
    assert_eq!(
        e.state_core().player(1).hand.len(),
        2,
        "juejingzhice must draw 2 when Shan leaves hand empty"
    );

    // (b) `Wanjian` (ArcheryAttack) response using last Shan MUST NOT trigger `juejingzhice` (`shanEnd` only)!
    let mut e_wj = start_versus(3141, &[&[0, 2], &[1]]);
    grant_skill(&mut e_wj, 1, "_buff_txhj_juejingzhice");
    let wj_shan = place_kind(&mut e_wj, 1, CardKind::Shan);
    e_wj.debug_set_turn(0, Phase::Play, 1);
    let wanjian = place_kind(&mut e_wj, 0, CardKind::ArcheryAttack);
    let _ = play_card(&mut e_wj, 0, wanjian, Vec::new());
    let wj_resp = expect_pending(&e_wj, "aoe_response");
    let _ = answer(&mut e_wj, &wj_resp, vec![ChoiceOption::Card(wj_shan)]);
    drain(&mut e_wj);
    assert_eq!(
        e_wj.state_core().player(1).hand.len(),
        0,
        "juejingzhice must NOT draw on ArcheryAttack (Wanjian) Shan response"
    );

    // (c) Negative: Hujia teammate helping target with last Shan MUST NOT trigger teammate's `juejingzhice`!
    let mut e_hj = start_versus(3142, &[&[0], &[1, 2]]);
    let wei_name = e_hj
        .catalog()
        .standard_generals
        .iter()
        .find(|g| g.faction == engine_native::catalog::Faction::Wei)
        .expect("wei general")
        .name;
    e_hj.debug_set_general(1, wei_name);
    e_hj.debug_set_general(2, wei_name);
    grant_skill(&mut e_hj, 1, engine_native::vocabulary::names::HU_JIA);
    grant_skill(&mut e_hj, 2, "_buff_txhj_juejingzhice");
    let hj_shan = place_kind(&mut e_hj, 2, CardKind::Shan);
    e_hj.debug_set_turn(0, Phase::Play, 1);
    let sha_hj = place_kind(&mut e_hj, 0, CardKind::Sha);
    let _ = play_card(&mut e_hj, 0, sha_hj, vec![1]);
    let c_hj_help = expect_pending(&e_hj, "hujia_respond");
    assert_eq!(c_hj_help.observer, 2);
    let _ = answer(&mut e_hj, &c_hj_help, vec![ChoiceOption::Card(hj_shan)]);
    drain(&mut e_hj);
    assert_eq!(
        e_hj.state_core().player(2).hand.len(),
        0,
        "juejingzhice must NOT draw when teammate responds with Shan via Hujia"
    );

    // 2. `baijujianchu`:
    // (a) FireSha damage does NOT discard target 'he' (`event.nature` truthy!)
    e.debug_set_turn(0, Phase::Play, 3);
    e.state_core_mut().turn.sha_used_play_phase = false;
    let fsha = place_kind(&mut e, 0, CardKind::FireSha);
    let _ = play_card(&mut e, 0, fsha, vec![1]);
    drain(&mut e);
    assert_eq!(e.state_core().player(1).hp, 5);
    assert_eq!(
        e.state_core().player(1).hand.len(),
        2,
        "baijujianchu must NOT discard on elemental Sha damage"
    );
    // (b) Normal Sha damage DOES discard 1 from target 'he'
    e.debug_set_turn(0, Phase::Play, 4);
    e.state_core_mut().turn.sha_used_play_phase = false;
    let nsha = place_kind(&mut e, 0, CardKind::Sha);
    let _ = play_card(&mut e, 0, nsha, vec![1]);
    drain(&mut e);
    assert_eq!(e.state_core().player(1).hp, 4);
    assert_eq!(
        e.state_core().player(1).hand.len(),
        1,
        "baijujianchu must randomDiscard 1 on Normal Sha damage"
    );

    // 3. `xumingshenyao`:
    // (a) Healthy seat 0 (hp 4 < 6) uses Tao on self -> recovers ONLY 1 HP (4 -> 5)!
    e.debug_set_hp(0, 4);
    let tao_healthy = place_kind(&mut e, 0, CardKind::Tao);
    let _ = play_card(&mut e, 0, tao_healthy, Vec::new());
    drain(&mut e);
    assert_eq!(
        e.state_core().player(0).hp,
        5,
        "healthy self-Tao must recover only 1 HP (no xumingshenyao)"
    );

    // (b) Dying seat 0 (hp 1 -> takes 1 dmg -> hp 0):
    // First seat 0 self-rescues with own Tao (`player == owner && hp <= 0`) -> heals 1 (xuming) + 1 (Tao) = 2 HP!
    e.debug_set_hp(0, 1);
    let tao_self = place_kind(&mut e, 0, CardKind::Tao);
    e.debug_set_turn(1, Phase::Play, 5);
    e.state_core_mut().turn.sha_used_play_phase = false;
    let sha_kill = place_kind(&mut e, 1, CardKind::Sha);
    let _ = play_card(&mut e, 1, sha_kill, vec![0]);
    let dying_c = expect_pending(&e, "dying_save");
    let _ = answer(&mut e, &dying_c, vec![ChoiceOption::Card(tao_self)]);
    drain(&mut e);
    assert_eq!(
        e.state_core().player(0).hp,
        2,
        "dying self-Tao with xumingshenyao must heal +1 extra (0 -> 2)"
    );

    // (c) Dying seat 0 rescued by ally seat 2's Tao (`u.player != owner`) -> heals ONLY 1 HP (0 -> 1)!
    e.debug_clear_hand(0);
    let _tao_ally = place_kind(&mut e, 2, CardKind::Tao);
    e.debug_set_hp(0, 1);
    e.debug_set_turn(1, Phase::Play, 6);
    e.state_core_mut().turn.sha_used_play_phase = false;
    let sha_kill2 = place_kind(&mut e, 1, CardKind::Sha);
    let _ = play_card(&mut e, 1, sha_kill2, vec![0]);
    let dying_ally = expect_pending(&e, "dying_save");
    assert_eq!(dying_ally.observer, 2);
    let _ = answer(&mut e, &dying_ally, vec![ChoiceOption::Card(_tao_ally)]);
    drain(&mut e);
    assert_eq!(
        e.state_core().player(0).hp,
        1,
        "ally Tao on dying owner must NOT trigger xumingshenyao (+1 only, 0 -> 1)"
    );
}

#[test]
fn l11_b3_yueyingdejiqiao_wuweizhile_yuanduandilue_jiezizhidao_huangtiannizhuan() {
    // Tests judgeEnd hooks + mod filters:
    // - `yueyingdejiqiao` (`2226`): judge card in discard + equips non-empty -> draw 1 (skipped if TianDu gained it!)
    // - `wuweizhile` (`2337`): other player's non-heart Le on their turn -> hand limit -1 (respects tieji!)
    // - `yuanduandilue` (`2369`): Bingliang unlimited distance
    // - `jiezizhidao` (`2389`): ANY non-club Bing resolves -> draw 1 (respects tieji!)
    // - `huangtiannizhuan` (`2417`): gives enemies mark=1 on TurnBegin -> odd mark flips ShanDian! Even mark (2 allies) cancels flip!
    let mut e = start_versus(315, &[&[0], &[1, 2]]);
    grant_skill(&mut e, 0, "_buff_txhj_wuweizhile");
    grant_skill(&mut e, 0, "_buff_txhj_yuanduandilue");
    grant_skill(&mut e, 0, "_buff_txhj_jiezizhidao");
    grant_skill(&mut e, 0, "_buff_txhj_huangtiannizhuan");
    grant_skill(&mut e, 1, "_buff_txhj_yueyingdejiqiao");

    // Seat 0 TurnBegin triggers `yuanduandilue` & `huangtiannizhuan` (marks enemies 1 & 2, NOT ally 0!):
    e.debug_set_turn(2, Phase::End, 0);
    advance_until_turn_phase(&mut e, 0, Phase::Play);
    assert!(has_skill(&e, 0, "txhj_yuanduandilue"));
    assert!(!has_skill(&e, 0, "txhj_huangtiannizhuan"));
    assert!(has_skill(&e, 1, "txhj_huangtiannizhuan"));
    assert!(has_skill(&e, 2, "txhj_huangtiannizhuan"));
    assert_eq!(flag(&e, 1, "l11_huangtiannizhuan_marks"), 1);

    // Distance-2 Bingliang succeeds via `yuanduandilue` (on seat 2? wait 0..2 is dist 1, test Bingliang unlimited from afar in 4-seat below too; here use Le/Bing/ShanDian on seat 1!):
    let bing = place_kind(&mut e, 0, CardKind::Bing);
    let le = place_kind(&mut e, 0, CardKind::Le);
    let shandian = place_kind(&mut e, 0, CardKind::ShanDian);
    assert!(matches!(play_card(&mut e, 0, bing, vec![1]), CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    drain(&mut e);
    assert!(matches!(play_card(&mut e, 0, le, vec![1]), CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    drain(&mut e);
    assert!(matches!(play_card(&mut e, 0, shandian, Vec::new()), CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    drain(&mut e);
    // Give seat 0's ShanDian directly to seat 1 to test enemy flipped ShanDian!
    if let Some(sd) = e.state_core_mut().player_mut(0).judge.pop() {
        e.state_core_mut().player_mut(1).judge.push(sd);
    }

    // Equip Armor on seat 1 so `yueyingdejiqiao` qualifies; set deck top so:
    // Queue order in seat 1 JudgePhase: [Bing, Le, ShanDian]
    // Remember deck.pop() takes LAST element! Put ShanDian Heart first (popped last), Le Club middle, Bing Diamond last (popped first)!
    equip_kind(&mut e, 1, CardKind::Armor);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_set_hp(1, 6);
    let _c3 = e.debug_set_deck_top_by(|cd| cd.suit == Suit::Heart); // ShanDian: Heart (normally miss, REVERSED -> HIT 3 thunder dmg!)
    let _c2 = e.debug_set_deck_top_by(|cd| cd.suit == Suit::Club); // Le: Club (!heart -> skip play + wuweizhile -1!)
    let _c1 = e.debug_set_deck_top_by(|cd| cd.suit == Suit::Diamond); // Bing: Diamond (!club -> skip draw + jiezizhidao +1 for seat 0!)

    // Move to seat 1 Judge phase: note order of pop is _c1 (Bing), _c2 (Le), _c3 (ShanDian)
    e.debug_set_turn(1, Phase::Prepare, 2);
    let _ = next_phase(&mut e); // Phase::Prepare -> Phase::Judge
    drain(&mut e);

    // Verify all 3 delayed tricks fired + judge settled hooks:
    // - Seat 0 drew 1 from `jiezizhidao` (Diamond Bing!)
    assert_eq!(e.state_core().player(0).hand.len(), 1, "jiezizhidao must draw 1 on Diamond Bing");
    // - Seat 1 got `txhj_wuweizhile` (`hand_limit` = hp 3 - 1 = 2!)
    assert!(has_skill(&e, 1, "txhj_wuweizhile"));
    assert_eq!(e.state_core().player(1).hp, 3, "reversed ShanDian on Heart must hit enemy for 3!");
    assert_eq!(e.state_core().hand_limit(e.catalog(), 1), 2, "wuweizhile reduces hand limit by 1");
    // - Seat 1 drew 3 from `yueyingdejiqiao` (1 per judge card left in discard while equipped!)
    assert_eq!(
        e.state_core().player(1).hand.len(),
        3,
        "yueyingdejiqiao must draw 1 for each judge card remaining in discard while equipped"
    );

    // Negative check on `yueyingdejiqiao` with `tiandu`: when `tiandu` gains the judge card into hand, it is NOT in discard -> NO draw from `yueyingdejiqiao`!
    let mut e2 = start_versus(316, &[&[0], &[1, 2, 3]]);
    grant_skill(&mut e2, 0, "_buff_txhj_yuanduandilue");
    let bing_far = place_kind(&mut e2, 0, CardKind::Bing);
    e2.debug_set_turn(0, Phase::Play, 1);
    assert!(matches!(
        play_card(&mut e2, 0, bing_far, vec![2]),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    drain(&mut e2);
    assert!(e2.state_core().player(2).judge.contains(&bing_far), "yuanduandilue allows Bingliang at distance 2");

    grant_skill(&mut e2, 2, "_buff_txhj_yueyingdejiqiao");
    grant_skill(&mut e2, 2, engine_native::vocabulary::names::TIAN_DU);
    equip_kind(&mut e2, 2, CardKind::Armor);
    e2.debug_clear_hand(2);
    e2.debug_set_turn(2, Phase::Prepare, 2);
    let _ = next_phase(&mut e2); // Phase::Judge
    let td = expect_pending(&e2, "tian_du_gain");
    let _ = answer(&mut e2, &td, vec![ChoiceOption::Confirm]);
    drain(&mut e2);
    assert_eq!(
        e2.state_core().player(2).hand.len(),
        1,
        "when TianDu gains judge card (1 card), yueyingdejiqiao must NOT draw a 2nd card"
    );

    // Even-mark `huangtiannizhuan` (2 allies holding it -> marks=2, NOT reversed) + `ShanDian` miss wraps past `txhj_wenhejibei`!
    let mut e3 = start_versus(319, &[&[0, 3], &[1, 2]]);
    grant_skill(&mut e3, 0, "_buff_txhj_huangtiannizhuan");
    grant_skill(&mut e3, 3, "_buff_txhj_huangtiannizhuan");
    grant_skill(&mut e3, 2, "_buff_txhj_wenhejibei");
    e3.debug_set_turn(3, Phase::End, 0);
    advance_until_turn_phase(&mut e3, 0, Phase::Play);
    assert_eq!(flag(&e3, 1, "l11_huangtiannizhuan_marks"), 2);
    let sd3 = place_kind(&mut e3, 0, CardKind::ShanDian);
    e3.state_core_mut().player_mut(1).judge.push(sd3);
    e3.state_core_mut().player_mut(0).hand.retain(|&c| c != sd3);
    // Non-Spade-2..9 on seat 1 with EVEN marks (2): must MISS (not hit!), skip seat 2 (`wenhejibei`), and pass to seat 3!
    let _ = top_deck_suit(&mut e3, Suit::Heart);
    e3.debug_set_turn(1, Phase::Prepare, 2);
    let _ = next_phase(&mut e3); // Phase::Judge
    drain(&mut e3);
    assert_eq!(e3.state_core().player(1).hp, 6, "even huangtiannizhuan marks must NOT reverse ShanDian");
    assert!(e3.state_core().player(2).judge.is_empty(), "ShanDian miss must skip txhj_wenhejibei seat 2");
    assert!(e3.state_core().player(3).judge.contains(&sd3), "ShanDian miss must wrap around wenhejibei to seat 3");
}

#[test]
fn l11_b3_dongchazhiyan_mingjingzhiyan_kapaikeshi_and_juxiangzhifeng() {
    // 1. `dongchazhiyan` / `mingjingzhiyan` -> temp `txhj_kapaikeshi` reveals opponent hand in View,
    //    prompts `txhj_kapaikeshi:choose_card`, `save->restore->Answer`, AI picks visible high-value card, cleans up!
    // 2. `juxiangzhifeng` -> Nanman prompts `txhj_juxiangzhifeng:discard_card`,
    //    `save->restore->Answer` on BOTH discard & pass (directHit!), AI keeps sole Sha / passes when 0 Sha!
    let mut e = start_versus(317, &[&[0], &[1, 2]]);
    grant_skill(&mut e, 0, "_buff_txhj_dongchazhiyan");
    grant_skill(&mut e, 0, "_buff_txhj_mingjingzhiyan");
    grant_skill(&mut e, 0, "_buff_txhj_juxiangzhifeng");

    // Before using Snatch/Guohe, seat 1's hand MUST be hidden to seat 0:
    let target_sha = place_kind(&mut e, 1, CardKind::Sha);
    let target_tao = place_kind(&mut e, 1, CardKind::Tao);
    assert!(matches!(e.view_for(0).players[1].hand, HandView::Hidden));

    // Cast Snatch during seat 0's turn -> activates temp `txhj_kapaikeshi`!
    e.debug_set_turn(0, Phase::Play, 1);
    let snatch = place_kind(&mut e, 0, CardKind::Snatch);
    let _ = play_card(&mut e, 0, snatch, vec![1]);
    let ch = expect_pending(&e, "txhj_kapaikeshi:choose_card");
    // While prompt is open, seat 0 view MUST reveal seat 1's hand!
    assert!(
        matches!(e.view_for(0).players[1].hand, HandView::Own(ref h) if h.contains(&target_tao) && h.contains(&target_sha)),
        "txhj_kapaikeshi must expose target hand in view_for(0) while Snatch resolves"
    );
    // AI picks highest-value card (Tao over Sha) from visible view!
    assert_eq!(
        ai_plan(&e, 0),
        GameAction::Answer {
            choice_id: ch.id,
            selection: vec![ChoiceOption::Card(target_tao)],
        }
    );
    // Verify save -> restore -> Answer exact state hash equality!
    let mut e = save_restore_and_compare_pending(&e, &ch, vec![ChoiceOption::Card(target_tao)]);
    assert!(e.state_core().player(0).hand.contains(&target_tao));
    assert!(
        !has_skill(&e, 0, "txhj_kapaikeshi"),
        "temp txhj_kapaikeshi must be removed after Snatch settles"
    );
    assert!(
        matches!(e.view_for(0).players[1].hand, HandView::Hidden),
        "target hand must be hidden again after Snatch finishes"
    );

    // Same for Guohe (`mingjingzhiyan`) choosing equip/hand card cleanly!
    let guohe = place_std_kind(&mut e, 0, CardKind::Dismantle);
    let _ = play_card(&mut e, 0, guohe, vec![1]);
    let ch_g = expect_pending(&e, "txhj_kapaikeshi:choose_card");
    let _ = answer(&mut e, &ch_g, vec![ChoiceOption::Card(target_sha)]);
    drain(&mut e);
    assert!(!has_skill(&e, 0, "txhj_kapaikeshi"));

    // Negative: Snatch out-of-turn (e.g. if forced out-of-turn) or non-owner -> does NOT grant kapaikeshi.
    // Now test `buff_txhj_juxiangzhifeng` on `SavageAttack` (Nanman):
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    // Seat 1 has [Sha, Tao] (1 Sha + extra card -> AI could discard junk, wait Tao>Sha so AI keeps Sha or discards lowest non-Sha card):
    let s1_sha = place_kind(&mut e, 1, CardKind::Sha);
    let s1_wuxie = place_kind(&mut e, 1, CardKind::Nullify); // value 45, Sha value 35 -> wait, test explicit choice & AI!
    // Seat 2 has ONLY 1 [Sha] (sole Sha, no junk!) -> AI MUST Pass (refuse to discard its only Sha and still die anyway)!
    let s2_sha = place_kind(&mut e, 2, CardKind::Sha);
    let _ = (s1_sha, s2_sha);

    let nanman = place_kind(&mut e, 0, CardKind::SavageAttack);
    let _ = play_card(&mut e, 0, nanman, Vec::new());

    // Seat 1 prompted `txhj_juxiangzhifeng:discard_card` -> AI discards `s1_wuxie` (keeping sole `s1_sha`!)
    let jx1 = expect_pending(&e, "txhj_juxiangzhifeng:discard_card");
    assert_eq!(jx1.observer, 1);
    assert_eq!(
        ai_plan(&e, 1),
        GameAction::Answer {
            choice_id: jx1.id,
            selection: vec![ChoiceOption::Card(s1_wuxie)],
        }
    );
    // Verify save -> restore -> Answer when seat 1 discards `s1_wuxie`:
    let snap_disc = save_restore_and_compare_pending(&e, &jx1, vec![ChoiceOption::Card(s1_wuxie)]);
    let _ = snap_disc;
    // Continue live engine with `jx1` discard, then inspect `jx2` for Pass + save/restore!
    let _ = answer(&mut e, &jx1, vec![ChoiceOption::Card(s1_wuxie)]);

    // Seat 2 prompted `txhj_juxiangzhifeng:discard_card` (only has `s2_sha`): AI MUST Pass (`selection: []`)!
    let jx2 = expect_pending(&e, "txhj_juxiangzhifeng:discard_card");
    assert_eq!(jx2.observer, 2);
    assert_eq!(
        ai_plan(&e, 2),
        GameAction::Answer {
            choice_id: jx2.id,
            selection: Vec::new(),
        }
    );
    // Verify save -> restore -> Answer when seat 2 passes (marks directHit!):
    let e = save_restore_and_compare_pending(&e, &jx2, Vec::new());
    // Seat 1 discarded `s1_wuxie` AND `drain` in save_restore passed `aoe_response` (or could have played `s1_sha`),
    // while Seat 2 passed `jx2` -> Seat 2 suffered `aoe_direct_hit` -> NEVER asked `aoe_response` and took 1 damage STILL keeping `s2_sha`!
    assert_eq!(e.state_core().player(2).hp, 5);
    assert!(
        e.state_core().player(2).hand.contains(&s2_sha),
        "seat 2 passed juxiangzhifeng discard -> took 1 directHit damage cleanly without discarding sole Sha"
    );

    // Also confirm when seat 1 answers `s1_sha` to `aoe_response` after paying `jx1` discard, seat 1 takes 0 damage!
    let mut e_resp = start_versus(318, &[&[0], &[1]]);
    grant_skill(&mut e_resp, 0, "_buff_txhj_juxiangzhifeng");
    let r_sha = place_kind(&mut e_resp, 1, CardKind::Sha);
    let r_junk = place_kind(&mut e_resp, 1, CardKind::Le);
    e_resp.debug_set_turn(0, Phase::Play, 1);
    let nanman2 = place_kind(&mut e_resp, 0, CardKind::SavageAttack);
    let _ = play_card(&mut e_resp, 0, nanman2, Vec::new());
    let jx = expect_pending(&e_resp, "txhj_juxiangzhifeng:discard_card");
    let _ = answer(&mut e_resp, &jx, vec![ChoiceOption::Card(r_junk)]);
    let aoe_r = expect_pending(&e_resp, "aoe_response");
    let _ = answer(&mut e_resp, &aoe_r, vec![ChoiceOption::Card(r_sha)]);
    drain(&mut e_resp);
    assert_eq!(
        e_resp.state_core().player(1).hp,
        6,
        "after paying juxiangzhifeng discard, target responds with Sha to Nanman and takes 0 damage"
    );

    // Verify Flow::Yield ordering + trick-immune target skipping (`txhj_guofengyupao`):
    // Seat 1 has `_buff_txhj_yingjichuoshiyi` + Armor in hand; Seat 2 has junk.
    // When Seat 1 discards its Armor for `juxiangzhifeng`, `yingjichuoshiyi` trigger MUST run BEFORE Seat 2 is asked!
    let mut e_yield = start_versus(3181, &[&[0], &[1, 2]]);
    grant_skill(&mut e_yield, 0, "_buff_txhj_juxiangzhifeng");
    grant_skill(&mut e_yield, 1, "_buff_txhj_yingjichuoshiyi");
    let s1_arm = place_std_kind(&mut e_yield, 1, CardKind::Armor);
    let _s2_junk = place_kind(&mut e_yield, 2, CardKind::Tao);
    e_yield.debug_set_turn(0, Phase::Play, 1);
    let nm_yield = place_kind(&mut e_yield, 0, CardKind::SavageAttack);
    let _ = play_card(&mut e_yield, 0, nm_yield, Vec::new());
    let jx_y1 = expect_pending(&e_yield, "txhj_juxiangzhifeng:discard_card");
    assert_eq!(jx_y1.observer, 1);
    let _ = answer(&mut e_yield, &jx_y1, vec![ChoiceOption::Card(s1_arm)]);
    // Next choice MUST be seat 1's `trigger_confirm` (`yingjichuoshiyi`) BEFORE seat 2's `juxiangzhifeng:discard_card`!
    let c_arm = expect_pending(&e_yield, "trigger_confirm");
    assert_eq!(c_arm.observer, 1);
    let _ = answer(&mut e_yield, &c_arm, vec![ChoiceOption::Confirm]);
    assert_eq!(e_yield.state_core().player(1).hand.len(), 2);
    let jx_y2 = expect_pending(&e_yield, "txhj_juxiangzhifeng:discard_card");
    assert_eq!(jx_y2.observer, 2);
    drain(&mut e_yield);

    // Trick-immune target skipping:
    // (a) 3-seat where seat 1 equips `txhj_guofengyupao` and seat 2 does not -> Nanman skips seat 1 cleanly (only seat 2 prompted):
    let mut e_imm = start_versus(3182, &[&[0], &[1, 2]]);
    grant_skill(&mut e_imm, 0, "_buff_txhj_juxiangzhifeng");
    let _ = equip_named(&mut e_imm, 1, "txhj_guofengyupao");
    let _s1_keep = place_kind(&mut e_imm, 1, CardKind::Tao);
    let _s2_keep = place_kind(&mut e_imm, 2, CardKind::Tao);
    e_imm.debug_set_turn(0, Phase::Play, 1);
    let nm_imm = place_kind(&mut e_imm, 0, CardKind::SavageAttack);
    let _ = play_card(&mut e_imm, 0, nm_imm, Vec::new());
    let jx_imm = expect_pending(&e_imm, "txhj_juxiangzhifeng:discard_card");
    assert_eq!(
        jx_imm.observer, 2,
        "juxiangzhifeng must skip trick-immune seat 1 (txhj_guofengyupao) and go directly to seat 2"
    );
    drain(&mut e_imm);
    assert_eq!(e_imm.state_core().player(1).hp, 6);
    assert_eq!(e_imm.state_core().player(1).hand.len(), 1);

    // (b) 2-seat where sole opponent equips `txhj_guofengyupao` -> SavageAttack rejected (no legal targets):
    let mut e_imm2 = start_versus(3183, &[&[0], &[1]]);
    grant_skill(&mut e_imm2, 0, "_buff_txhj_juxiangzhifeng");
    let _ = equip_named(&mut e_imm2, 1, "txhj_guofengyupao");
    e_imm2.debug_set_turn(0, Phase::Play, 1);
    let nm_bad = place_kind(&mut e_imm2, 0, CardKind::SavageAttack);
    assert!(
        matches!(
            play_card(&mut e_imm2, 0, nm_bad, Vec::new()),
            CommandOutcome::Rejected { .. }
        ),
        "SavageAttack with all opponents trick-immune (txhj_guofengyupao) must be Rejected"
    );
}
