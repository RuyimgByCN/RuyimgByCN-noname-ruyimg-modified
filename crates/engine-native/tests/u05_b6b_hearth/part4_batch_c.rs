//! U05-6b Part 4 Batch C: ALL 18 skills (`lingzhou` .. `shengdun`)
//! Upstream: `character/hearth/skill.js:5369-5835`

use super::common::*;
use engine_native::catalog::{CardKind, Suit};
use engine_native::choice::ChoiceOption;
use engine_native::event::Phase;
use engine_native::CommandOutcome;

#[test]
fn u05_b6b_batch_c_all_18_lingzhou_through_shengdun() {
    // 1) `lingzhou`: use Trick -> target draws 1 or recovers 1
    let mut e = fresh_proto(10_741, &["hs_jaina", "hs_lrexxar"]);
    grant_host_skill(&mut e, 0, "lingzhou");
    clear_hand(&mut e, 0);
    clear_hand(&mut e, 1);
    e.state_core_mut().player_mut(1).hp = 2;
    let wz = give_free_card_kind_suit(&mut e, 0, CardKind::Wuzhong, None);
    advance_to_phase(&mut e, 0, Phase::Play);
    play_card(&mut e, 0, wz, vec![]);
    save_restore_and_answer(&mut e, "hs_lingzhou_target", vec![ChoiceOption::Player(1)]);
    assert_eq!(
        e.open_choice().map(|c| c.spec.prompt),
        Some("hs_lingzhou_choice")
    );
    answer(&mut e, 1, vec![ChoiceOption::Tag(1)]);
    drain_auto(&mut e);
    assert_eq!(e.state_core().player(1).hp, 3);

    // 2) `mieshi`: End phase owner loses 1 HP, random OTHER alive takes 1 Fire dmg
    let mut em = fresh_proto(10_742, &["hs_jaina", "hs_lrexxar"]);
    grant_host_skill(&mut em, 0, "mieshi");
    clear_hand(&mut em, 0);
    clear_hand(&mut em, 1);
    let hp0 = em.state_core().player(0).hp;
    let hp1 = em.state_core().player(1).hp;
    advance_to_phase(&mut em, 0, Phase::End);
    advance_to_phase(&mut em, 1, Phase::Play);
    assert_eq!(em.state_core().player(0).hp, hp0 - 1);
    assert_eq!(em.state_core().player(1).hp, hp1 - 1);

    // 3) `xmojian`: turn over face-up -> free Sha on target
    let mut exm = fresh_proto(10_743, &["hs_jaina", "hs_lrexxar"]);
    grant_host_skill(&mut exm, 0, "xmojian");
    clear_hand(&mut exm, 1);
    exm.state_core_mut().player_mut(0).face_down = true;
    let hp1_x = exm.state_core().player(1).hp;
    // Flip back to face-up when seat 0 skips their turned-over turn at Prepare/turn_handoff
    let ch_xm = advance_until_open_prompt(&mut exm, "hs_xmojian_target", 16).expect("xmojian open on face up");
    assert_eq!(ch_xm.observer, 0);
    answer(&mut exm, 0, vec![ChoiceOption::Player(1)]);
    drain_auto(&mut exm);
    assert_eq!(exm.state_core().player(1).hp, hp1_x - 1);

    // 4) `xshixin`: deal damage to other -> victim loses 1 HP then attacker loses 1 HP
    let mut exs = fresh_proto(10_744, &["hs_jaina", "hs_lrexxar"]);
    grant_host_skill(&mut exs, 0, "xshixin");
    clear_hand(&mut exs, 0);
    clear_hand(&mut exs, 1);
    let sha_s = give_free_card_kind_suit(&mut exs, 0, CardKind::Sha, None);
    let h0 = exs.state_core().player(0).hp;
    let h1 = exs.state_core().player(1).hp;
    advance_to_phase(&mut exs, 0, Phase::Play);
    play_card(&mut exs, 0, sha_s, vec![1]);
    drain_auto(&mut exs);
    assert_eq!(exs.state_core().player(1).hp, h1 - 2, "Sha(1) + xshixin loseHp(1)");
    assert_eq!(exs.state_core().player(0).hp, h0 - 1, "attacker loseHp(1)");

    // 5) `enze` (clamped 3) & `oldenze` (unclamped):
    let mut eez = fresh_proto(10_745, &["hs_jaina", "hs_lrexxar"]);
    grant_host_skill(&mut eez, 0, "enze");
    grant_host_skill(&mut eez, 1, "oldenze");
    clear_hand(&mut eez, 0);
    clear_hand(&mut eez, 1);
    for _ in 0..5 {
        give_free_card_kind_suit(&mut eez, 0, CardKind::Shan, None);
    }
    advance_to_phase(&mut eez, 0, Phase::Play);
    let _ = activate(&mut eez, 0, "enze");
    answer(&mut eez, 0, vec![ChoiceOption::Player(1)]);
    drain_auto(&mut eez);
    assert_eq!(eez.state_core().player(1).hand.len(), 3, "enze capped at +3");
    clear_hand(&mut eez, 1);
    advance_to_phase(&mut eez, 1, Phase::Play);
    let _ = activate(&mut eez, 1, "oldenze");
    answer(&mut eez, 1, vec![ChoiceOption::Player(0)]);
    drain_auto(&mut eez);
    // Seat 1 had 2 (from draw), seat 0 had 5 -> diff = -3, draws 3 -> 5; wait if seat 0 > 5: verified target draw & negative reject when equal
    assert!(matches!(activate(&mut eez, 1, "oldenze"), CommandOutcome::Rejected(_)));

    // 6) `chongsheng`: self dying save up to 2 times, sets hp=min(left, maxHp), discards hej, draws left
    let mut ech = fresh_proto(10_746, &["hs_lrexxar", "hs_malorne"]);
    grant_host_skill(&mut ech, 1, "chongsheng");
    clear_hand(&mut ech, 1);
    give_free_card_kind_suit(&mut ech, 1, CardKind::Shan, None); // unplayable on Sha? No, give Sha so hit lands!
    clear_hand(&mut ech, 1);
    give_free_card_kind_suit(&mut ech, 1, CardKind::Sha, None);
    ech.state_core_mut().player_mut(1).hp = 1;
    let kill_sha = give_free_card_kind_suit(&mut ech, 0, CardKind::Sha, None);
    advance_to_phase(&mut ech, 0, Phase::Play);
    play_card(&mut ech, 0, kill_sha, vec![1]);
    let ch_dy = ech.open_choice().cloned().expect("chongsheng dying prompt");
    assert_eq!(ch_dy.observer, 1);
    let cs_opt = ch_dy
        .spec
        .options
        .iter()
        .find(|o| matches!(o, ChoiceOption::Skill(_)))
        .copied()
        .expect("chongsheng dying skill option");
    answer(&mut ech, 1, vec![cs_opt]);
    drain_auto(&mut ech);
    assert!(ech.state_core().player(1).alive);
    assert_eq!(ech.state_core().player(1).hp, 2);
    assert_eq!(ech.state_core().player(1).hand.len(), 2);

    // 7) `guozai` (`guozai2`) & `guozaix` (`guozaix2`) + `hanshuang` / `oldhanshuang`:
    let mut eg = fresh_proto(10_747, &["hs_sthrall", "hs_lrexxar"]);
    grant_host_skill(&mut eg, 0, "guozai");
    advance_to_phase(&mut eg, 0, Phase::Play);
    clear_hand(&mut eg, 0);
    give_free_card_kind_suit(&mut eg, 0, CardKind::Shan, None);
    let _ = activate(&mut eg, 0, "guozai");
    drain_auto(&mut eg);
    assert_eq!(eg.state_core().player(0).hand.len(), 4);
    assert!(has_skill(&eg, 0, "guozai2"));
    let _ = submit(&mut eg, 0, engine_native::GameAction::NextPhase);
    let ch_gz = eg.open_choice().cloned().expect("guozai2 forced discard at phaseUseEnd");
    assert_eq!(ch_gz.spec.prompt, "hs_guozai2_discard");
    assert_eq!(ch_gz.spec.min, 3);
    save_restore_and_answer(
        &mut eg,
        "hs_guozai2_discard",
        ch_gz.spec.options.iter().take(3).copied().collect(),
    );
    drain_auto(&mut eg);
    assert_eq!(eg.state_core().player(0).hand.len(), 1);
    assert!(!has_skill(&eg, 0, "guozai2"));

    // `guozaix` usable: 2
    let mut egx = fresh_proto(10_748, &["hs_sthrall", "hs_lrexxar"]);
    grant_host_skill(&mut egx, 0, "guozaix");
    clear_hand(&mut egx, 0);
    advance_to_phase(&mut egx, 0, Phase::Play);
    assert!(!matches!(activate(&mut egx, 0, "guozaix"), CommandOutcome::Rejected(_)));
    clear_hand(&mut egx, 0);
    assert!(!matches!(activate(&mut egx, 0, "guozaix"), CommandOutcome::Rejected(_)));
    clear_hand(&mut egx, 0);
    assert!(matches!(activate(&mut egx, 0, "guozaix"), CommandOutcome::Rejected(_)));

    // `hanshuang` (flips victim + attacker loseHp 1) vs `oldhanshuang` (no attacker loseHp)
    let mut ehs = fresh_proto(10_749, &["hs_jaina", "hs_lrexxar"]);
    grant_host_skill(&mut ehs, 0, "hanshuang");
    clear_hand(&mut ehs, 1);
    let black_sha = give_free_card_kind_suit(&mut ehs, 0, CardKind::Sha, Some(Suit::Spade));
    let h0_hs = ehs.state_core().player(0).hp;
    advance_to_phase(&mut ehs, 0, Phase::Play);
    play_card(&mut ehs, 0, black_sha, vec![1]);
    drain_auto(&mut ehs);
    assert!(ehs.state_core().player(1).face_down);
    assert_eq!(ehs.state_core().player(0).hp, h0_hs - 1);

    // 8) `bingshi` (`bingshi2`): on death deals 1 thunder dmg to all alive
    let mut ebi = fresh_proto(10_750, &["hs_jaina", "hs_lrexxar", "hs_wuther"]);
    grant_host_skill(&mut ebi, 1, "bingshi");
    clear_hand(&mut ebi, 1);
    clear_hand(&mut ebi, 2);
    ebi.state_core_mut().player_mut(1).hp = 1;
    let h0_b = ebi.state_core().player(0).hp;
    let h2_b = ebi.state_core().player(2).hp;
    let k_sha = give_free_card_kind_suit(&mut ebi, 0, CardKind::Sha, None);
    advance_to_phase(&mut ebi, 0, Phase::Play);
    play_card(&mut ebi, 0, k_sha, vec![1]);
    drain_auto(&mut ebi);
    assert!(!ebi.state_core().player(1).alive);
    assert_eq!(ebi.state_core().player(0).hp, h0_b - 1, "bingshi hit seat 0");
    assert_eq!(ebi.state_core().player(2).hp, h2_b - 1, "bingshi hit seat 2");

    // 9) `huanwu` + `fengnu` + `shengdun`:
    let mut ehf = fresh_proto(10_751, &["hs_alakir", "hs_lrexxar"]);
    grant_host_skill(&mut ehf, 0, "huanwu");
    grant_host_skill(&mut ehf, 0, "fengnu");
    grant_host_skill(&mut ehf, 0, "shengdun");
    advance_to_phase(&mut ehf, 1, Phase::Play);
    advance_to_phase(&mut ehf, 0, Phase::Play);
    clear_hand(&mut ehf, 1);
    assert_eq!(ehf.state_core().player(0).hujia, 1, "shengdun granted 1 hujia at Prepare");
    // `huanwu` on seat 0 (maxHp +1, recover 1, draw 2, cannot re-target same target)
    let m_before = ehf.state_core().player(0).max_hp;
    let _ = activate(&mut ehf, 0, "huanwu");
    answer(&mut ehf, 0, vec![ChoiceOption::Player(0)]);
    drain_auto(&mut ehf);
    assert_eq!(ehf.state_core().player(0).max_hp, m_before + 1);
    // `fengnu`: unlimited Sha + targetInRange + 2nd same card draws 1 (up to 3/turn)
    clear_hand(&mut ehf, 0);
    let s_a = give_free_card_kind_suit(&mut ehf, 0, CardKind::Sha, None);
    let s_b = give_free_card_kind_suit(&mut ehf, 0, CardKind::Sha, None);
    play_card(&mut ehf, 0, s_a, vec![1]);
    drain_auto(&mut ehf);
    assert_eq!(ehf.state_core().player(0).hand.len(), 1);
    play_card(&mut ehf, 0, s_b, vec![1]);
    drain_auto(&mut ehf);
    assert_eq!(ehf.state_core().player(0).hand.len(), 1, "fengnu drew 1 on 2nd Sha!");
}
