//! Batch L11-4 Group D (`L11BuffsD` tag 737 + remaining 8 `L11BuffHelpersB` tag 738) real-path tests.

use super::common::*;
use engine_native::catalog::{CardKind, Suit};
use engine_native::{ChoiceOption, CommandOutcome, GameAction, Phase};

#[test]
fn l11_b4_tianxiawushuang_jijianchangge_huangtianzhinu_and_shenyizhidao() {
    // 1. `tianxiawushuang` (`2886`): Normal Sha +1 dmg; FireSha / LeiSha NOT boosted!
    let mut e = start_versus(420, &[&[0], &[1]]);
    grant_skill(&mut e, 0, "buff_txhj_tianxiawushuang");
    grant_skill(&mut e, 0, "_buff_txhj_tianxiawushuang");
    grant_skill(&mut e, 0, engine_native::vocabulary::names::PAO_XIAO);
    e.debug_set_turn(0, Phase::Play, 1);

    let sha_n = place_kind(&mut e, 0, CardKind::Sha);
    let _ = play_card(&mut e, 0, sha_n, vec![1]);
    drain(&mut e);
    assert_eq!(e.state_core().player(1).hp, 4, "Normal Sha +1 dmg via tianxiawushuang (6->4)");

    let sha_f = place_kind(&mut e, 0, CardKind::FireSha);
    let _ = play_card(&mut e, 0, sha_f, vec![1]);
    drain(&mut e);
    assert_eq!(e.state_core().player(1).hp, 3, "FireSha must NOT get tianxiawushuang +1");

    // 2. `jijianchangge` (`2907`): outside own turn responding Sha draws 1; own turn Play does NOT!
    grant_skill(&mut e, 0, "_buff_txhj_jijianchangge");
    e.debug_clear_hand(0);
    let sha_resp = place_kind(&mut e, 0, CardKind::Sha);
    e.debug_set_turn(1, Phase::Play, 1);
    let nanman = place_kind(&mut e, 1, CardKind::SavageAttack);
    let _ = play_card(&mut e, 1, nanman, Vec::new());
    let aoe_r = expect_pending(&e, "aoe_response");
    let _ = answer(&mut e, &aoe_r, vec![ChoiceOption::Card(sha_resp)]);
    drain(&mut e);
    assert_eq!(e.state_core().player(0).hand.len(), 1, "off-turn Sha response draws 1");

    // 3. `huangtianzhinu` (`2928`): elemental damage randomly discards 1 `he` card from target!
    grant_skill(&mut e, 0, "_buff_txhj_huangtianzhinu");
    e.debug_set_turn(0, Phase::Play, 2);
    let _t_card = place_kind(&mut e, 1, CardKind::Tao);
    let sha_l = place_kind(&mut e, 0, CardKind::LeiSha);
    let _ = play_card(&mut e, 0, sha_l, vec![1]);
    drain(&mut e);
    assert_eq!(e.state_core().player(1).hand.len(), 0, "elemental damage discards 1 he card");

    // 4. `shenyizhidao` (`2969`): normal Tao AND Dying Tao rescue heal 1 + 1 = 2 HP!
    let mut es = start_versus(421, &[&[0], &[1]]);
    grant_skill(&mut es, 0, "buff_txhj_shenyizhidao");
    grant_skill(&mut es, 0, "_buff_txhj_shenyizhidao");
    es.debug_set_hp(0, 2);
    es.debug_set_turn(0, Phase::Play, 1);
    let tao0 = place_kind(&mut es, 0, CardKind::Tao);
    let _ = play_card(&mut es, 0, tao0, Vec::new());
    drain(&mut es);
    assert_eq!(es.state_core().player(0).hp, 4, "normal Tao recovers 2 via shenyizhidao");

    // Dying save Tao also heals +1 (= 2 HP)!
    es.debug_set_hp(1, 1);
    let tao_save = place_kind(&mut es, 0, CardKind::Tao);
    let sha_kill = place_kind(&mut es, 0, CardKind::Sha);
    let _ = play_card(&mut es, 0, sha_kill, vec![1]);
    let ds = expect_pending(&es, "dying_save");
    let _ = answer(&mut es, &ds, vec![ChoiceOption::Card(tao_save)]);
    drain(&mut es);
    assert_eq!(es.state_core().player(1).hp, 2, "Dying Tao rescue heals 2 HP via shenyizhidao");
}

#[test]
fn l11_b4_luoshenjuezhang_mowangdejianiang_and_hannu() {
    // 1. `luoshenjuezhang` + `txhj_luoshenjuezhang` + `txhj_luoshen` (`2950, 3458, 3476`):
    //    Once per round when responding Shan -> draw 1 and attach `txhj_luoshen`; reset at Round 2 TurnBegin!
    let mut e = start_versus(422, &[&[0], &[1]]);
    grant_skill(&mut e, 0, "_buff_txhj_luoshenjuezhang");
    e.debug_set_turn(1, Phase::End, 0);
    advance_until_turn_phase(&mut e, 0, Phase::Draw);
    assert!(has_skill(&e, 0, "txhj_luoshenjuezhang"), "Round 1 TurnBegin grants txhj_luoshenjuezhang");

    e.debug_clear_hand(0);
    e.debug_set_turn(1, Phase::Play, 1);
    grant_skill(&mut e, 1, engine_native::vocabulary::names::PAO_XIAO);
    let shan_a = place_kind(&mut e, 0, CardKind::Shan);
    let shan_b = place_kind(&mut e, 0, CardKind::Shan);
    let sha_1 = place_kind(&mut e, 1, CardKind::Sha);
    let sha_2 = place_kind(&mut e, 1, CardKind::Sha);
    let _ = play_card(&mut e, 1, sha_1, vec![0]);
    let r1 = expect_pending(&e, "respond_shan");
    let _ = answer(&mut e, &r1, vec![ChoiceOption::Card(shan_a)]);
    drain(&mut e);
    assert!(has_skill(&e, 0, "txhj_luoshen"), "1st Shan attaches txhj_luoshen lock");
    assert_eq!(e.state_core().player(0).hand.len(), 2, "1st Shan draws 1 (shan_b + 1 drawn)");

    // 2nd Shan in same Round 1 must NOT draw!
    let _ = play_card(&mut e, 1, sha_2, vec![0]);
    let r2 = expect_pending(&e, "respond_shan");
    let _ = answer(&mut e, &r2, vec![ChoiceOption::Card(shan_b)]);
    drain(&mut e);
    assert_eq!(e.state_core().player(0).hand.len(), 1, "2nd Shan in same round draws nothing");

    // Round 2 TurnBegin resets `txhj_luoshen`, enabling draw again on next Shan!
    advance_until_turn_phase(&mut e, 0, Phase::Draw);
    assert!(!has_skill(&e, 0, "txhj_luoshen"), "new round clears txhj_luoshen");
    e.debug_clear_hand(0);
    let shan_c = place_kind(&mut e, 0, CardKind::Shan);
    e.debug_set_turn(1, Phase::Play, 2);
    let sha_3 = place_kind(&mut e, 1, CardKind::Sha);
    let _ = play_card(&mut e, 1, sha_3, vec![0]);
    let r3 = expect_pending(&e, "respond_shan");
    let _ = answer(&mut e, &r3, vec![ChoiceOption::Card(shan_c)]);
    drain(&mut e);
    assert_eq!(e.state_core().player(0).hand.len(), 1, "Round 2 Shan triggers luoshenjuezhang again");

    // 2. `mowangdejianiang` + `txhj_mowangdejianiang` + `txhj_jianiang_clear` + `hannu` (`2988, 3013, 3492, 3526`):
    let mut em = start_versus(423, &[&[0], &[1]]);
    grant_skill(&mut em, 0, "_buff_txhj_mowangdejianiang");
    grant_skill(&mut em, 0, "_buff_txhj_hannu");
    grant_skill(&mut em, 0, engine_native::vocabulary::names::PAO_XIAO);
    em.debug_set_turn(0, Phase::Play, 1);

    let jiu = place_kind(&mut em, 0, CardKind::Jiu);
    let _ = play_card(&mut em, 0, jiu, Vec::new());
    drain(&mut em);
    assert!(
        em.state_core().player(0).hand.iter().any(|&c| em.catalog().card_kind(c).is_sha_like()),
        "hannu gains a Sha after Jiu resolves"
    );
    assert_eq!(flag(&em, 0, "l11_jianiang_stacks"), 1);

    // Next Sha consumes Jiu (+1 dmg only = 2, NOT +1+1=3 because sha_used_jiu excludes it):
    let sha_drunk = place_kind(&mut em, 0, CardKind::Sha);
    let _ = play_card(&mut em, 0, sha_drunk, vec![1]);
    drain(&mut em);
    assert_eq!(em.state_core().player(1).hp, 4, "drunk Sha gets +1 from Jiu only (6->4)");

    // Following Sha in same turn (`sha_used_jiu == 0`): gets +1 stack from mowangdejianiang (= 2 dmg)!
    let sha_plain = place_kind(&mut em, 0, CardKind::Sha);
    let _ = play_card(&mut em, 0, sha_plain, vec![1]);
    drain(&mut em);
    assert_eq!(em.state_core().player(1).hp, 2, "2nd Sha persists +1 Jiu stack via mowangdejianiang (4->2)");

    // Phase::End / TurnEnded clears `l11_jianiang_stacks` back to 0!
    advance_until_turn_phase(&mut em, 1, Phase::Play);
    assert_eq!(flag(&em, 0, "l11_jianiang_stacks"), 0, "jianiang_clear resets stacks after turn ends");
    assert!(!has_skill(&em, 0, "txhj_mowangdejianiang"));
}

#[test]
fn l11_b4_guosejueyan_mingyundezhiyi_ximengzhili_and_hengxingwuji() {
    // 1. `gusoejueyan` / `_buff_txhj_guosejueyan` (`3035`, locked: false!): non-Heart Le judge draws 1; Heart / tieji-sealed does NOT!
    let mut e = start_versus(424, &[&[0], &[1]]);
    grant_skill(&mut e, 0, "buff_txhj_gusoejueyan");
    grant_skill(&mut e, 0, "_buff_txhj_guosejueyan");
    e.debug_set_turn(1, Phase::Play, 1);
    let le1 = place_kind(&mut e, 1, CardKind::Le);
    let _ = play_card(&mut e, 1, le1, vec![0]);
    drain(&mut e);
    top_deck_suit(&mut e, Suit::Spade);
    e.debug_clear_hand(0);
    e.debug_set_turn(0, Phase::Prepare, 2);
    let _ = next_phase(&mut e); // Phase::Judge
    drain(&mut e);
    assert_eq!(e.state_core().player(0).hand.len(), 1, "non-Heart Le judge draws 1 via guosejueyan");

    // Heart Le judge must NOT draw via guosejueyan:
    let le_h = place_kind(&mut e, 1, CardKind::Le);
    e.state_core_mut().player_mut(1).hand.retain(|&c| c != le_h);
    e.state_core_mut().player_mut(0).judge.push(le_h);
    top_deck_suit(&mut e, Suit::Heart);
    e.debug_clear_hand(0);
    e.debug_set_turn(0, Phase::Prepare, 3);
    let _ = next_phase(&mut e); // Phase::Judge
    drain(&mut e);
    assert_eq!(e.state_core().player(0).hand.len(), 0, "Heart Le judge does NOT draw");

    // Non-Heart Le while holder `0` is `tieji_sealed` (`locked: false`): must NOT draw!
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((0, "fz_new_tieji_target_sealed".to_string()), 1);
    let le_s = place_kind(&mut e, 1, CardKind::Le);
    e.state_core_mut().player_mut(1).hand.retain(|&c| c != le_s);
    e.state_core_mut().player_mut(0).judge.push(le_s);
    top_deck_suit(&mut e, Suit::Club);
    e.debug_set_turn(0, Phase::Prepare, 4);
    let _ = next_phase(&mut e); // Phase::Judge
    drain(&mut e);
    assert_eq!(e.state_core().player(0).hand.len(), 0, "tieji-sealed guosejueyan (`locked: false`) draws nothing");

    // Global skill (`extension_buff.js:752`): enemy's non-Heart Le judge draws 1 for buff holder 0
    // even while enemy 1 itself is `tieji_sealed`; sealing buff holder 0 blocks the global draw!
    e.state_core_mut()
        .vm_aux
        .flags
        .remove(&(0, "fz_new_tieji_target_sealed".to_string()));
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((1, "fz_new_tieji_target_sealed".to_string()), 1);
    let le_e1 = place_kind(&mut e, 0, CardKind::Le);
    e.state_core_mut().player_mut(0).hand.retain(|&c| c != le_e1);
    e.state_core_mut().player_mut(1).judge.push(le_e1);
    top_deck_suit(&mut e, Suit::Spade);
    e.debug_set_turn(1, Phase::Prepare, 5);
    let _ = next_phase(&mut e); // Seat 1 Phase::Judge
    drain(&mut e);
    assert_eq!(
        e.state_core().player(0).hand.len(),
        1,
        "enemy non-Heart Le judge triggers global guosejueyan draw for holder 0 regardless of enemy tieji seal"
    );

    e.state_core_mut()
        .vm_aux
        .flags
        .insert((0, "fz_new_tieji_target_sealed".to_string()), 1);
    let le_e2 = place_kind(&mut e, 0, CardKind::Le);
    e.state_core_mut().player_mut(0).hand.retain(|&c| c != le_e2);
    e.state_core_mut().player_mut(1).judge.push(le_e2);
    top_deck_suit(&mut e, Suit::Club);
    e.debug_set_turn(1, Phase::Prepare, 6);
    let _ = next_phase(&mut e); // Seat 1 Phase::Judge
    drain(&mut e);
    assert_eq!(
        e.state_core().player(0).hand.len(),
        1,
        "tieji-sealed holder 0 gets 0 cards even when enemy judges non-Heart Le"
    );

    // 2. `mingyundezhiyi` (`3058`): ShanDian links all unlinked enemies (skips allies & respects txminguangkai)!
    let mut em = start_versus(425, &[&[0, 3], &[1, 2]]);
    grant_skill(&mut em, 0, "_buff_txhj_mingyundezhiyi");
    let _ = equip_named(&mut em, 2, "txminguangkai");
    em.debug_set_hp(2, 5); // txminguangkai blocks linking while injured!
    em.debug_set_turn(0, Phase::Play, 1);
    let sd = place_kind(&mut em, 0, CardKind::ShanDian);
    let _ = play_card(&mut em, 0, sd, Vec::new());
    drain(&mut em);
    assert!(em.state_core().player(1).linked, "enemy 1 linked by mingyundezhiyi");
    assert!(!em.state_core().player(2).linked, "txminguangkai blocks mingyundezhiyi link on enemy 2");
    assert!(!em.state_core().player(3).linked, "ally 3 untouched by mingyundezhiyi");

    // 3. `ximengzhili` (`3092`): `txhj_yuanjiao` draws 3 + 1 = 4 for owner (and 1 for target)!
    let mut ex = start_versus(426, &[&[0], &[1]]);
    grant_skill(&mut ex, 0, "buff_txhj_ximengzhili");
    ex.debug_set_turn(0, Phase::Play, 1);
    let yj = place_named(&mut ex, 0, "txhj_yuanjiao");
    let _ = play_card(&mut ex, 0, yj, vec![1]);
    drain(&mut ex);
    assert_eq!(ex.state_core().player(1).hand.len(), 1);
    assert_eq!(ex.state_core().player(0).hand.len(), 4, "ximengzhili boosts txhj_yuanjiao draw 3->4");

    // 4. `hengxingwuji` (`3110`): standard Dismantle (`guohe`) draws 1 after resolving!
    //    `Nullify`-cancelled Dismantle and `txhj_caomu` do NOT grant `hengxingwuji`'s draw.
    grant_skill(&mut ex, 0, "_buff_txhj_hengxingwuji");
    ex.debug_clear_hand(0);
    let _target_h = place_kind(&mut ex, 1, CardKind::Tao);
    let guohe = place_std_kind(&mut ex, 0, CardKind::Dismantle);
    let _ = play_card(&mut ex, 0, guohe, vec![1]);
    drain(&mut ex);
    assert_eq!(ex.state_core().player(0).hand.len(), 1, "hengxingwuji draws 1 after standard Dismantle");

    // Nullify neutralizes target effect only; upstream `useCardEnd` STILL draws 1 via hengxingwuji:
    ex.debug_clear_hand(0);
    let target_h2 = place_kind(&mut ex, 1, CardKind::Tao);
    let wuxie = place_kind(&mut ex, 1, CardKind::Nullify);
    let guohe_w = place_std_kind(&mut ex, 0, CardKind::Dismantle);
    let _ = play_card(&mut ex, 0, guohe_w, vec![1]);
    let wuxie_ask = expect_pending(&ex, "nullify_window");
    let _ = answer(&mut ex, &wuxie_ask, vec![ChoiceOption::Card(wuxie)]);
    drain(&mut ex);
    assert!(ex.state_core().player(1).hand.contains(&target_h2), "Nullify protects target card from Dismantle");
    assert_eq!(ex.state_core().player(0).hand.len(), 1, "Nullify-neutralized Dismantle still fires useCardEnd draw 1");

    // `txhj_caomu` draws only its own 1 card (NO +1 from hengxingwuji):
    ex.debug_clear_hand(0);
    let caomu = place_named(&mut ex, 0, "txhj_caomu");
    let _ = play_card(&mut ex, 0, caomu, vec![1]);
    drain(&mut ex);
    assert_eq!(ex.state_core().player(0).hand.len(), 1, "txhj_caomu grants only its own 1 card, not hengxingwuji");
}

#[test]
fn l11_b4_shierqicejin_shierqicetui_and_pofuchenzhou_vs_fuzu() {
    // 1. `shierqicejin` (`3129, 3543`) & `shierqicetui` (`3159, 3567`) in Versus 2v2:
    let mut e = start_versus(427, &[&[0, 2], &[1, 3]]);
    grant_skill(&mut e, 0, "_buff_txhj_shierqicejin");
    grant_skill(&mut e, 0, "_buff_txhj_shierqicetui");
    e.debug_set_turn(3, Phase::End, 0);
    advance_until_turn_phase(&mut e, 0, Phase::Play);
    assert!(has_skill(&e, 2, "txhj_shierqicejin"), "ally marked txhj_shierqicejin");
    assert!(has_skill(&e, 1, "txhj_shierqicetui") && has_skill(&e, 3, "txhj_shierqicetui"));

    // Seat 0 plays Nanman -> ally 2 protected (no prompt / no damage!), enemies 1 & 3 hit!
    let _shan_unused = place_kind(&mut e, 2, CardKind::Sha);
    let nanman = place_kind(&mut e, 0, CardKind::SavageAttack);
    let _ = play_card(&mut e, 0, nanman, Vec::new());
    drain(&mut e);
    assert_eq!(e.state_core().player(2).hp, 6, "ally excluded from Seat 0 Nanman");
    assert_eq!(e.state_core().player(1).hp, 5, "enemy 1 hit by Nanman");

    // Seat 0 plays Taoyuan -> ally 2 healed, enemies 1 & 3 excluded!
    e.debug_set_hp(2, 4);
    let taoyuan = place_kind(&mut e, 0, CardKind::Taoyuan);
    let _ = play_card(&mut e, 0, taoyuan, Vec::new());
    drain(&mut e);
    assert_eq!(e.state_core().player(2).hp, 5, "ally 2 heals from Taoyuan");
    assert_eq!(e.state_core().player(1).hp, 5, "enemy 1 excluded by shierqicetui");

    // 2. `pofuchenzhou` (`extension_framework.js:10039`, `extension_skill.js:3189, 3588`):
    //    (a) Alone: Turn 1 TurnBegin trims 4->1 silently (no discard triggers!), Prepare +1, End +1!
    let mut ep = start_versus(428, &[&[0], &[1]]);
    grant_skill(&mut ep, 0, "_buff_txhj_pofuchenzhou");
    grant_skill(&mut ep, 0, "_buff_txhj_yingjichuoshijia"); // ensure silent trim does NOT trigger!
    for _ in 0..4 {
        let _ = place_kind(&mut ep, 0, CardKind::Armor);
    }
    ep.debug_set_turn(1, Phase::End, 0);
    advance_until_turn_phase(&mut ep, 0, Phase::Draw);
    // TurnBegin trimmed 4->1, Prepare drew +1, Draw drew +2 => hand == 4!
    assert_eq!(ep.state_core().player(0).hand.len(), 4, "pofuchenzhou sets opening=1, Prepare +1, Draw +2 (=4)");
    assert!(has_skill(&ep, 0, "txhj_pofuchenzhou"));
    // `pofuchenzhou` Phase::End (`begin: true`) draws 1!
    ep.debug_clear_hand(0);
    ep.debug_set_turn(0, Phase::Discard, 1);
    let _ = next_phase(&mut ep);
    drain(&mut ep);
    assert_eq!(
        ep.state_core().player(0).hand.len(),
        1,
        "txhj_pofuchenzhou draws 1 at Phase::End begin"
    );

    //    (b) Paired with `fuzu`: `fuzu` overrides opening hand to `max_hp` (6), + Prepare (+1) + Draw (+2) = 9!
    let mut ef = start_versus(429, &[&[0], &[1]]);
    grant_skill(&mut ef, 0, "_buff_txhj_pofuchenzhou");
    grant_skill(&mut ef, 0, "_buff_txhj_fuzu");
    for _ in 0..4 {
        let _ = place_kind(&mut ef, 0, CardKind::Sha);
    }
    ef.debug_set_turn(1, Phase::End, 0);
    advance_until_turn_phase(&mut ef, 0, Phase::Draw);
    assert_eq!(
        ef.state_core().player(0).hand.len(),
        9,
        "fuzu overrides pofuchenzhou opening hand to max_hp=6, plus Prepare +1 and Draw +2 = 9"
    );
}

#[test]
fn l11_b4_huangtianyili_zhizu_zhimin_and_youhu_mahou() {
    // 1. `huangtianyili` (Spade) + `huangtianzhizu` (Spade 5 overrides yili) + `huangtianzhimin` (judge in discard -> draw 1):
    let mut e = start_versus(430, &[&[0], &[1]]);
    grant_skill(&mut e, 0, "_buff_txhj_huangtianyili");
    grant_skill(&mut e, 0, "_buff_txhj_huangtianzhizu");
    grant_skill(&mut e, 0, "_buff_txhj_huangtianzhimin");

    // Club Q Le -> rewritten to Spade 5 -> fails Heart check (skips Play) AND huangtianzhimin draws 1!
    let le = place_kind(&mut e, 1, CardKind::Le);
    e.state_core_mut().player_mut(1).hand.retain(|&c| c != le);
    e.state_core_mut().player_mut(0).judge.push(le);
    let _ = top_deck_suit_rank(&mut e, Suit::Heart, 5); // even Heart is rewritten to Spade 5!
    e.debug_set_turn(0, Phase::Prepare, 2);
    let _ = next_phase(&mut e); // Phase::Judge
    drain(&mut e);
    assert_eq!(e.state_core().player(0).hand.len(), 1, "huangtianzhimin draws 1 when judge stays in discard");
    let _ = next_phase(&mut e); // Judge -> Draw
    assert_eq!(e.turn().phase, Phase::Draw);
    let _ = next_phase(&mut e); // Draw -> skips Play -> Discard!
    assert_eq!(e.turn().phase, Phase::Discard, "Heart Le rewritten to Spade 5 hits Le and skips Play!");

    // `huangtianyili` alone keeps original rank (Heart Q -> Spade Q misses ShanDian 2..9),
    // and `huangtianzhimin` does NOT draw when Tiandu picks up the judge card from discard!
    let mut ey = start_versus(432, &[&[0], &[1]]);
    grant_skill(&mut ey, 0, "_buff_txhj_huangtianyili");
    grant_skill(&mut ey, 0, "_buff_txhj_huangtianzhimin");
    grant_skill(&mut ey, 0, engine_native::vocabulary::names::TIAN_DU);
    let sd_y = place_kind(&mut ey, 1, CardKind::ShanDian);
    ey.state_core_mut().player_mut(1).hand.retain(|&c| c != sd_y);
    ey.state_core_mut().player_mut(0).judge.push(sd_y);
    let _ = top_deck_suit_rank(&mut ey, Suit::Heart, 12); // Q -> Spade Q (outside 2..9 -> misses ShanDian!)
    ey.debug_set_turn(0, Phase::Prepare, 2);
    let _ = next_phase(&mut ey); // Phase::Judge
    let td_ask = expect_pending(&ey, "tian_du_gain");
    let _ = answer(&mut ey, &td_ask, vec![ChoiceOption::Confirm]);
    drain(&mut ey);
    assert_eq!(ey.state_core().player(0).hp, 6, "huangtianyili Spade Q misses ShanDian 2..9");
    assert_eq!(
        ey.state_core().player(0).hand.len(),
        1,
        "when Tiandu takes judge card from discard, huangtianzhimin draws NO 2nd card"
    );

    // 2. `digongdeyouhu` (`1..=4` max), `tiangongdeyouhu` (`rem = x - 1`), `renigongdeyouhu` (`txhj_renigongdeyouhu:confirm` AI + save/restore):
    let mut em = start_versus_with_l27(431, &[&[0], &[1]]);
    grant_skill(&mut em, 0, "twxingzhui");
    grant_skill(&mut em, 0, "buff_txhj_digongdeyouhu");
    grant_skill(&mut em, 0, "buff_txhj_tiangongdeyouhu");
    grant_skill(&mut em, 0, "buff_txhj_renigongdeyouhu");
    em.debug_set_turn(0, Phase::Play, 1);

    let out = activate_skill(&mut em, 0, "twxingzhui");
    assert!(matches!(out, CommandOutcome::Pending { .. }));
    let turn_ask = expect_pending(&em, "twxingzhui:choose_turns");
    assert!(turn_ask.spec.options.contains(&ChoiceOption::Tag(4)), "digongdeyouhu unlocks Tag(4)");
    let _ = answer(&mut em, &turn_ask, vec![ChoiceOption::Tag(4)]);
    assert_eq!(flag(&em, 0, "twxingzhui_rem"), 3, "tiangongdeyouhu reduces rem 4 -> 3");

    let rc = expect_pending(&em, "txhj_renigongdeyouhu:confirm");
    assert_eq!(
        ai_plan(&em, 0),
        GameAction::Answer {
            choice_id: rc.id,
            selection: vec![ChoiceOption::Confirm],
        }
    );
    let em = save_restore_and_compare_pending_with(
        &em,
        &rc,
        vec![ChoiceOption::Confirm],
        taixu_with_l27_verified(),
    );
    assert_eq!(em.state_core().player(0).hand.len(), 1, "confirming renigongdeyouhu draws 1");
}
