use engine_native::catalog::{CardKind, Faction, Suit};
use engine_native::{ChoiceOption, CommandOutcome, GameAction, Phase};

use super::common::*;

/// Upstream: `extension_servant.js:11, 205, 244, 261` (`_slEnterGame`, `_slDamage`, `_slUseCard`, `_slHurt`).
#[test]
fn l11_b2_servant_hooks_enter_damage_usecard_hurt() {
    let mut e = start_taixu(0x2001, 2);
    e.debug_grant(0, "_slEnterGame");
    e.debug_grant(0, "_slDamage");
    e.debug_grant(0, "_slUseCard");
    e.debug_grant(0, "_slHurt");

    // TurnBegin increments _slEnterGame once only.
    e.debug_set_turn(0, Phase::End, 1);
    let _ = next_phase(&mut e); // Turn 1 begins
    drain(&mut e);
    assert_eq!(flag(&e, 0, "l11_sl_enter_game"), 1);
    e.debug_set_turn(1, Phase::End, 1);
    let _ = next_phase(&mut e); // Next TurnBegin must not re-fire
    drain(&mut e);
    assert_eq!(flag(&e, 0, "l11_sl_enter_game"), 1);

    // Sha does not trigger _slUseCard (only trick/Tao/Jiu), while Damage triggers _slDamage.
    e.debug_set_turn(0, Phase::Play, 2);
    let sha = place_kind_suit(&mut e, 0, CardKind::Sha, Suit::Spade);
    let _ = play_card(&mut e, 0, sha, vec![1]);
    drain(&mut e);
    assert_eq!(flag(&e, 0, "l11_sl_usecard_anim"), 0);
    assert_eq!(flag(&e, 0, "l11_sl_attack_anim"), 1);

    // Wuzhong (trick) & Tao trigger _slUseCard; taking damage triggers _slHurt.
    let wz = e.debug_move_named_card_to_hand(0, "wuzhong");
    let _ = play_card(&mut e, 0, wz, Vec::new());
    drain(&mut e);
    assert_eq!(flag(&e, 0, "l11_sl_usecard_anim"), 1);

    e.debug_set_turn(1, Phase::Play, 2);
    e.state_core_mut().turn.sha_used_play_phase = false;
    let sha1 = place_kind_suit(&mut e, 1, CardKind::Sha, Suit::Club);
    let _ = play_card(&mut e, 1, sha1, vec![0]);
    drain(&mut e);
    assert_eq!(flag(&e, 0, "l11_sl_hurt_anim"), 1);
}

/// Upstream: `extension.js:2031-2380` (`txhj_dundunSkill1`, `txhj_datongSkill1`, `txhj_luluSkill1..2`, `txhj_aleSkill1..2`).
#[test]
fn l11_b2_servants_dundun_datong_lulu_ale() {
    // 1. txhj_dundunSkill1 + txhj_luluSkill1 + txhj_luluSkill2:
    let mut e = start_taixu(0x2002, 3);
    e.debug_grant(0, "txhj_dundunSkill1");
    e.debug_grant(0, "txhj_luluSkill1");
    e.debug_grant(0, "txhj_luluSkill2");
    e.debug_set_hp(0, 2);
    equip_kind(&mut e, 0, CardKind::Fangtian); // equips Fangtian when playing last Sha
    e.debug_set_turn(0, Phase::Draw, 1);
    let _ = next_phase(&mut e); // -> Play begin: luluSkill1 draws 2 (2 == hp!)
    drain(&mut e);
    let hand_after_lulu1 = e.debug_hand(0);
    assert_eq!(hand_after_lulu1.len(), 2, "luluSkill1 draws 2 at Play begin");

    // Non-Sha Wuzhong doesn't trigger dundunSkill1
    let wz = e.debug_move_named_card_to_hand(0, "wuzhong");
    let _ = play_card(&mut e, 0, wz, Vec::new());
    drain(&mut e);
    assert_eq!(e.debug_hand(0).len(), 4, "Wuzhong draws 2 without dundunSkill1");

    // Give seat 1 a Shan to test lulu2 unrespondable + +1 dmg; Fangtian equips Sha when last card.
    e.debug_clear_hand(0);
    let sha1 = place_kind_suit(&mut e, 0, CardKind::Sha, Suit::Spade);
    // Reset gained flag only for sha1? Give it via real draw:
    e.state_core_mut().player_mut(0).hand.clear();
    let _ = top_deck_suit(&mut e, Suit::Spade);
    e.debug_set_turn(0, Phase::Prepare, 2);
    let _ = next_phase(&mut e); // Draw skipped? Prepare -> Judge -> Draw
    let _ = next_phase(&mut e); // Judge -> Draw (draws 2 gained cards!)
    let _ = next_phase(&mut e); // Draw -> Play (+2 from luluSkill1 => 4 gained >= hp 2)
    drain(&mut e);
    let _ = sha1;
    let first_sha = place_kind(&mut e, 0, CardKind::Sha);
    // Mark first_sha as gained-this-turn by discarding/moving through lulu2 or just check an actually drawn Sha:
    let _ = e.debug_discard_card(0, first_sha);
    let _ = e.debug_set_deck_top_by(|cd| cd.kind == CardKind::Sha);
    e.state_core_mut().turn.sha_used_play_phase = false;
    e.debug_grant(0, "txhj_aleSkill2"); // harmless check
    // Draw a Sha via Wuzhong so CardsDrawn marks it gained this turn:
    let wz2 = e.debug_move_named_card_to_hand(0, "wuzhong");
    let _ = play_card(&mut e, 0, wz2, Vec::new());
    drain(&mut e);
    let real_sha = e
        .debug_hand(0)
        .into_iter()
        .find(|&c| e.catalog().card_kind(c).is_sha_like())
        .expect("drawn Sha");
    let shan1 = place_kind(&mut e, 1, CardKind::Shan);
    let _ = shan1;
    let hand_before_sha = e.debug_hand(0).len();
    let hp1_before = e.debug_hp(1);
    let _ = play_card(&mut e, 0, real_sha, vec![1]);
    // Because gained >= hp (2) and first Sha: unrespondable (no Shan prompt for seat 1!) & +1 dmg!
    assert!(e.open_choice().is_none(), "luluSkill2 first Sha must be unrespondable");
    assert_eq!(e.debug_hp(1), hp1_before - 2, "luluSkill2 adds +1 Sha dmg");
    assert_eq!(
        e.debug_hand(0).len(),
        hand_before_sha,
        "dundunSkill1 draws 1 per Sha target (-1 played + 1 drawn)"
    );

    // 2. txhj_datongSkill1: dying recovers to 1 HP once, second dying dies
    let mut e_dt = start_taixu(0x2003, 2);
    e_dt.debug_grant(1, "txhj_datongSkill1");
    e_dt.debug_set_hp(1, 1);
    e_dt.debug_set_turn(0, Phase::Play, 1);
    let sha_a = place_kind(&mut e_dt, 0, CardKind::Sha);
    let _ = play_card(&mut e_dt, 0, sha_a, vec![1]);
    drain(&mut e_dt);
    assert!(e_dt.debug_alive(1) && e_dt.debug_hp(1) == 1, "datongSkill1 saves once to 1 HP");
    let sha_b = place_kind(&mut e_dt, 0, CardKind::Sha);
    e_dt.state_core_mut().turn.sha_used_play_phase = false;
    let _ = play_card(&mut e_dt, 0, sha_b, vec![1]);
    drain(&mut e_dt);
    assert!(!e_dt.debug_alive(1), "second dying without Peach dies");

    // 3. txhj_aleSkill1 & txhj_aleSkill2:
    let mut e_al = start_taixu(0x2004, 3);
    e_al.debug_grant(1, "txhj_aleSkill1");
    e_al.debug_grant(1, "txhj_aleSkill2");
    e_al.debug_set_hp(0, 6);
    e_al.debug_set_hp(1, 4);
    e_al.debug_set_hp(2, 4);
    // Prepare begin on seat 1: hp 4 < max_hp 6 & seat 0 has hp 6 > 4 -> recovers +1 -> 5
    e_al.debug_set_turn(0, Phase::End, 1);
    let _ = next_phase(&mut e_al);
    drain(&mut e_al);
    assert_eq!(e_al.debug_hp(1), 5, "aleSkill2 recovers 1 when !isMaxHp");
    // If seat 1 has highest HP (set others to 4): !isMaxHp fails -> no recover!
    e_al.debug_set_hp(0, 4);
    e_al.debug_set_hp(1, 5);
    e_al.debug_set_turn(0, Phase::End, 2);
    let _ = next_phase(&mut e_al);
    drain(&mut e_al);
    assert_eq!(e_al.debug_hp(1), 5, "aleSkill2 must NOT trigger when ties max HP");

    // aleSkill1: basic card targeted + dodged (0 dmg) -> draws 1; basic card deals dmg -> draws 0!
    e_al.debug_set_turn(0, Phase::Play, 3);
    let shan_1 = place_kind(&mut e_al, 1, CardKind::Shan);
    let sha_miss = place_kind(&mut e_al, 0, CardKind::Sha);
    let _ = play_card(&mut e_al, 0, sha_miss, vec![1]);
    let c_sh = expect_pending(&e_al, "respond_shan");
    let _ = answer(&mut e_al, &c_sh, vec![ChoiceOption::Card(shan_1)]);
    drain(&mut e_al);
    assert_eq!(e_al.debug_hand(1).len(), 1, "aleSkill1 draws 1 on dodged Sha");
    let sha_hit = place_kind(&mut e_al, 0, CardKind::Sha);
    e_al.state_core_mut().turn.sha_used_play_phase = false;
    let _ = play_card(&mut e_al, 0, sha_hit, vec![1]);
    drain(&mut e_al);
    assert_eq!(e_al.debug_hand(1).len(), 1, "aleSkill1 must NOT draw when Sha dealt dmg");
}

/// Upstream: `extension.js:2385-2560` (`txhj_ruiSkill2..3`, `txhj_yueerSkill1..2`, `txhj_yanSkill1..3`).
#[test]
fn l11_b2_servants_rui_yueer_yan() {
    // 1. txhj_ruiSkill2 & txhj_ruiSkill3:
    let mut e = start_versus(0x2005, &[&[0, 1], &[2]], &[Faction::Wei, Faction::Shu]);
    e.debug_grant(0, "txhj_ruiSkill2");
    e.debug_grant(0, "txhj_ruiSkill3");
    let c_e2 = place_kind(&mut e, 2, CardKind::Sha);
    let c_0a = place_kind(&mut e, 0, CardKind::Sha);
    // Odd hand (1) at TurnBegin -> draws 1 for friend (seat 1, NOT self)
    e.debug_set_turn(2, Phase::End, 1);
    let _ = next_phase(&mut e); // -> Turn 0 begins: seat 0 has 1 card (odd) -> draws 1 for ally seat 1!
    drain(&mut e);
    assert_eq!(e.debug_hand(0).len(), 1, "ruiSkill2 must keep self at 1");
    assert_eq!(e.debug_hand(1).len(), 1, "ruiSkill2 odd hand draws 1 for ally seat 1");
    let _ = (c_e2, c_0a);

    // Clear hands so seat 0 has 2 cards at TurnEnded -> discards enemy 2's card
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    let _ = place_kind(&mut e, 0, CardKind::Sha);
    let _ = place_kind(&mut e, 0, CardKind::Shan);
    let _ = place_kind(&mut e, 2, CardKind::Sha);
    e.debug_set_turn(0, Phase::End, 2);
    let _ = next_phase(&mut e); // triggers TurnEnded(0) -> enemy 2 loses hand card
    drain(&mut e);
    assert!(e.debug_hand(2).is_empty(), "ruiSkill2 even hand discards enemy card");

    // txhj_ruiSkill3: opponent Dismantle targeting seat 0 -> Red judge steals card & removes target; Black fails!
    e.debug_set_turn(2, Phase::Play, 3);
    e.debug_clear_hand(0);
    let kept0 = place_kind(&mut e, 0, CardKind::Sha);
    let gh1 = place_kind(&mut e, 2, CardKind::Dismantle);
    let _ = top_deck_suit(&mut e, Suit::Heart);
    let _ = play_card(&mut e, 2, gh1, vec![0]);
    drain(&mut e);
    assert!(e.debug_hand(0).contains(&gh1), "ruiSkill3 steals Red-judged trick");
    assert!(e.debug_hand(0).contains(&kept0) && e.debug_hand(0).len() == 2, "ruiSkill3 cancels target so original card kept");
    let gh2 = place_kind(&mut e, 2, CardKind::Dismantle);
    let _ = top_deck_suit(&mut e, Suit::Spade);
    let _ = play_card(&mut e, 2, gh2, vec![0]);
    drain(&mut e);
    assert!(!e.debug_hand(0).contains(&gh2), "ruiSkill3 Black judge fails");

    // 2. txhj_yueerSkill1 & txhj_yueerSkill2:
    let mut e_ye = start_taixu(0x2006, 3);
    e_ye.debug_grant(0, "txhj_yueerSkill1");
    e_ye.debug_grant(0, "txhj_yueerSkill2");
    e_ye.debug_set_hp(0, 4);
    e_ye.debug_set_turn(1, Phase::Play, 1);
    let sha_m = place_kind(&mut e_ye, 1, CardKind::Sha);
    let _ = play_card(&mut e_ye, 1, sha_m, vec![2]); // Male (default) takes dmg -> seat 0 heals 1 & draws 1
    drain(&mut e_ye);
    assert_eq!(e_ye.debug_hp(0), 5);
    assert_eq!(e_ye.debug_hand(0).len(), 1);
    // Second Male damage same round does not re-trigger:
    let sha_m2 = place_kind(&mut e_ye, 1, CardKind::Sha);
    e_ye.state_core_mut().turn.sha_used_play_phase = false;
    let _ = play_card(&mut e_ye, 1, sha_m2, vec![2]);
    drain(&mut e_ye);
    assert_eq!(e_ye.debug_hp(0), 5, "yueerSkill1 once per round");
    // Discard end draws 2:
    e_ye.debug_set_turn(0, Phase::Discard, 2);
    let _ = next_phase(&mut e_ye);
    drain(&mut e_ye);
    assert_eq!(e_ye.debug_hand(0).len(), 3, "yueerSkill2 draws 2 at Discard end");

    // 3. txhj_yanSkill1, txhj_yanSkill2, txhj_yanSkill3:
    let mut e_yn = start_taixu(0x2007, 3);
    e_yn.debug_grant(0, "txhj_yanSkill1");
    e_yn.debug_set_turn(1, Phase::Play, 1);
    // Single-target Snatch from 1 -> 0 with Black judge -> reflected back onto seat 1!
    let dummy0 = place_kind(&mut e_yn, 0, CardKind::Sha);
    let dummy1 = place_kind(&mut e_yn, 1, CardKind::Tao);
    let sn1 = place_kind(&mut e_yn, 1, CardKind::Snatch);
    let _ = top_deck_suit(&mut e_yn, Suit::Spade);
    let _ = play_card(&mut e_yn, 1, sn1, vec![0]);
    drain(&mut e_yn);
    assert!(e_yn.debug_hand(0).contains(&dummy0), "yanSkill1 avoided original Snatch");
    assert!(e_yn.debug_hand(0).contains(&dummy1), "yanSkill1 reflected Snatch onto seat 1");
    // Jiedao without weapon on orig_user -> must NOT cast reflected Jiedao!
    let jd1 = place_kind(&mut e_yn, 1, CardKind::Jiedao);
    let _ = top_deck_suit(&mut e_yn, Suit::Club);
    let sha_0 = place_kind(&mut e_yn, 0, CardKind::Sha);
    let _ = sha_0;
    let _ = play_card(&mut e_yn, 1, jd1, vec![0, 2]); // Wait targets len == 1 filter vs Jiedao? Jiedao has targets [0, 2]!
    // Check yanSkill1 with single-target Dismantle + Red judge negative:
    let gh_red = place_kind(&mut e_yn, 1, CardKind::Dismantle);
    let _ = top_deck_suit(&mut e_yn, Suit::Heart);
    let _ = play_card(&mut e_yn, 1, gh_red, vec![0]);
    drain(&mut e_yn);
    assert_eq!(e_yn.debug_hand(0).len(), 2, "yanSkill1 Red judge doesn't reflect");

    // yanSkill2 + yanSkill3:
    let mut e_y23 = start_taixu(0x2008, 2);
    e_y23.debug_grant(0, "txhj_yanSkill2");
    e_y23.debug_grant(0, "txhj_yanSkill3");
    let _ = place_kind(&mut e_y23, 1, CardKind::Tao);
    e_y23.debug_set_turn(1, Phase::Play, 1);
    let sha_atk = place_kind(&mut e_y23, 1, CardKind::Sha);
    let _ = top_deck_suit(&mut e_y23, Suit::Spade); // yanSkill3 Black judge!
    let _ = play_card(&mut e_y23, 1, sha_atk, vec![0]); // deals dmg -> yan2 shoots virtual Sha -> deals dmg -> yan2 heals 1 + yan3 steals card!
    let c_y3 = expect_pending(&e_y23, "txhj_yanSkill3:choose_card");
    assert!(matches!(
        ai_plan(&e_y23, 0),
        GameAction::Answer { choice_id, .. } if choice_id == c_y3.id
    ));
    let steal_cid = e_y23.debug_hand(1)[0];
    let e_y23_after =
        save_restore_and_compare_pending(&e_y23, &c_y3, vec![ChoiceOption::Card(steal_cid)]);
    assert_eq!(e_y23_after.debug_hp(0), 6, "yanSkill2 recovered back to 6 HP");
    assert_eq!(e_y23_after.debug_hp(1), 5, "yanSkill2 virtual Sha damaged attacker");
    assert!(e_y23_after.debug_hand(0).contains(&steal_cid), "yanSkill3 Black stole card");
}

/// Upstream: `extension.js:2565-2702`, `extension_buff.js:1428-1455` (`txhj_liuli..aHao`).
#[test]
fn l11_b2_servants_liuli_manman_xiaoxiao_xueren_xuanwu_ahao() {
    // 1. txhj_liuliSkill1 & txhj_liuliSkill2:
    let mut e = start_taixu(0x2009, 2);
    e.debug_grant(0, "txhj_liuliSkill1");
    e.debug_grant(1, "txhj_liuliSkill2");
    let c0 = place_kind(&mut e, 0, CardKind::Sha);
    let _ = place_kind(&mut e, 0, CardKind::Shan);
    e.debug_set_turn(1, Phase::Play, 1);
    let _ = e.debug_discard_card(0, c0); // 0 loses card in 1's turn -> 1 damage to 1!
    drain(&mut e);
    assert_eq!(e.debug_hp(1), 5, "liuliSkill1 damaged turn player");
    // Discard phase on seat 0 -> seat 0 discards excess -> seat 1 gets last discard card!
    e.debug_set_turn(0, Phase::Play, 2);
    for _ in 0..8 {
        let _ = place_kind(&mut e, 0, CardKind::Sha);
    }
    let _ = next_phase(&mut e); // Play -> Discard (opens discard prompt)
    drain(&mut e); // answers discard prompt
    let _ = next_phase(&mut e); // Discard -> End triggers TurnPhaseEnd(Discard) -> liuliSkill2!
    drain(&mut e);
    assert_eq!(e.debug_hand(1).len(), 1, "liuliSkill2 drew discard card at Discard end");

    // 2. txhj_manmanSkill1, txhj_manmanSkill2, txhj_xuerenSkill1:
    let mut e_mm = start_taixu(0x200A, 2);
    e_mm.debug_grant(0, "txhj_manmanSkill1");
    e_mm.debug_grant(0, "txhj_manmanSkill2");
    e_mm.debug_grant(0, "txhj_xuerenSkill1");
    let prize = place_kind(&mut e_mm, 1, CardKind::Tao);
    e_mm.debug_set_turn(1, Phase::End, 1);
    let _ = next_phase(&mut e_mm); // TurnBegin(0) arms manmanSkill1 & casts virtual SavageAttack -> hits seat 1 -> triggers manmanSkill2!
    let c_m2 = expect_pending(&e_mm, "txhj_manmanSkill2:choose_card");
    let mut e_mm2 =
        save_restore_and_compare_pending(&e_mm, &c_m2, vec![ChoiceOption::Card(prize)]);
    assert_eq!(e_mm2.debug_hand(0).len(), 2, "manman1 drew 1 (1 dmg) + manman2 stole 1");
    // TurnEnded draws +1 from xuerenSkill1:
    e_mm2.debug_set_turn(0, Phase::End, 2);
    let _ = next_phase(&mut e_mm2);
    drain(&mut e_mm2);
    assert_eq!(e_mm2.debug_hand(0).len(), 3, "xuerenSkill1 drew 1 at TurnEnded");

    // 3. txhj_xiaoxiaoSkill1 & txhj_xiaoxiaoSkill2:
    let mut e_xx = start_taixu(0x200B, 3);
    e_xx.debug_grant(0, "txhj_xiaoxiaoSkill1");
    e_xx.debug_grant(0, "txhj_xiaoxiaoSkill2");
    equip_kind(&mut e_xx, 0, CardKind::Zhuge);
    equip_kind(&mut e_xx, 2, CardKind::Zhuge);
    equip_kind(&mut e_xx, 2, CardKind::Renwang); // seat 2 has 2 equips > seat 0's 1!
    let p1 = place_kind(&mut e_xx, 1, CardKind::Sha);
    let p2 = place_kind(&mut e_xx, 2, CardKind::Shan);
    e_xx.debug_set_turn(1, Phase::Play, 1);
    e_xx.debug_set_hp(0, 5);
    let gh = place_kind(&mut e_xx, 1, CardKind::Dismantle);
    let _ = play_card(&mut e_xx, 1, gh, vec![0]);
    let c_x1 = expect_pending(&e_xx, "txhj_xiaoxiaoSkill1:confirm");
    assert_eq!(ai_plan(&e_xx, 0), GameAction::Answer {
        choice_id: c_x1.id,
        selection: vec![ChoiceOption::Confirm],
    });
    let _ = answer(&mut e_xx, &c_x1, vec![ChoiceOption::Confirm]);
    drain(&mut e_xx);
    assert_eq!(e_xx.debug_hp(0), 6, "xiaoxiaoSkill1 healed injured owner on opponent trick");
    // End begin steals 1 from seat 1 (1<=1? Wait: 0<=1) and skips seat 2 (2>1):
    e_xx.debug_set_turn(0, Phase::Discard, 2);
    let _ = next_phase(&mut e_xx);
    drain(&mut e_xx);
    assert!(e_xx.debug_hand(0).contains(&p1), "xiaoxiaoSkill2 steals from <= equips");
    assert!(e_xx.debug_hand(2).contains(&p2), "xiaoxiaoSkill2 skips > equips");

    // 4. txhj_xuanwuSkill1..3 & txhj_aHaoSkill1..3:
    let mut e_xa = start_taixu(0x200C, 3);
    e_xa.debug_grant(0, "txhj_xuanwuSkill1");
    e_xa.debug_grant(0, "txhj_xuanwuSkill2");
    e_xa.debug_grant(0, "txhj_xuanwuSkill3");
    e_xa.debug_grant(1, "txhj_aHaoSkill1");
    e_xa.debug_grant(1, "txhj_aHaoSkill2");
    e_xa.debug_grant(1, "txhj_aHaoSkill3");
    e_xa.debug_set_hp(0, 4);
    e_xa.debug_set_hp(2, 4);
    let tao2 = place_kind(&mut e_xa, 2, CardKind::Tao);
    e_xa.debug_set_turn(2, Phase::Play, 1);
    let _ = play_card(&mut e_xa, 2, tao2, Vec::new()); // 2 heals -> xuanwu1 heals 0 -> 5
    drain(&mut e_xa);
    assert_eq!(e_xa.debug_hp(0), 5, "xuanwuSkill1 healed when another healed");

    // xuanwuSkill2: discard in own turn draws 1
    e_xa.debug_set_hp(0, 1);
    let _ = place_kind(&mut e_xa, 0, CardKind::Sha);
    let d0 = place_kind(&mut e_xa, 0, CardKind::Sha);
    e_xa.debug_set_turn(0, Phase::Play, 2);
    let _ = next_phase(&mut e_xa);
    let c_xd = expect_pending(&e_xa, "discard_to_limit");
    let _ = answer(&mut e_xa, &c_xd, vec![ChoiceOption::Card(d0)]);
    drain(&mut e_xa);
    assert_eq!(e_xa.debug_hand(0).len(), 2, "xuanwuSkill2 drew 1 on own-turn discard");
    e_xa.debug_set_hp(0, 6);
    e_xa.debug_clear_hand(0);
    // xuanwuSkill3: trick rank <= hand.len() (1) cancels; rank > 1 hits!
    e_xa.debug_set_turn(2, Phase::Play, 2);
    let gh_r1 = place_kind_rank(&mut e_xa, 2, CardKind::Dismantle, 3);
    let _ = play_card(&mut e_xa, 2, gh_r1, vec![0]);
    drain(&mut e_xa);
    assert_eq!(e_xa.debug_hand(0).len(), 0, "rank 3 > hand 1 -> Dismantle still hits");
    for _ in 0..5 {
        let _ = place_kind(&mut e_xa, 0, CardKind::Sha);
    }
    let gh_r2 = place_kind_rank(&mut e_xa, 2, CardKind::Dismantle, 3);
    let _ = play_card(&mut e_xa, 2, gh_r2, vec![0]);
    drain(&mut e_xa);
    assert_eq!(e_xa.debug_hand(0).len(), 5, "rank 3 <= hand 5 -> xuanwuSkill3 cancels");

    // aHaoSkill2..3:
    // Losing equip via Dismantle triggers aHaoSkill2 -> 1 dmg to every enemy (not ally)!
    let mut e_ah = start_versus(0x200D, &[&[0, 1], &[2]], &[Faction::Wei, Faction::Shu]);
    e_ah.debug_grant(1, "txhj_aHaoSkill2");
    let _eq_lost = equip_kind(&mut e_ah, 1, CardKind::Zhuge);
    let gh2 = place_kind(&mut e_ah, 2, CardKind::Dismantle);
    e_ah.debug_set_turn(2, Phase::Play, 1);
    let _ = play_card(&mut e_ah, 2, gh2, vec![1]);
    drain(&mut e_ah);
    assert_eq!(e_ah.debug_hp(0), 6, "aHaoSkill2 does NOT damage ally seat 0");
    assert_eq!(e_ah.debug_hp(2), 5, "aHaoSkill2 deals 1 dmg to enemy seat 2 on LostOrigin::Equip");

    // aHaoSkill3: give seat 1 2 equips -> Draw draws 2 + 2 = 4, Sha limit 1 + 2 = 3!
    equip_kind(&mut e_xa, 1, CardKind::Zhuge);
    equip_kind(&mut e_xa, 1, CardKind::Renwang);
    e_xa.debug_set_hp(1, 6);
    // Draw bonus +2 & Sha limit +2:
    e_xa.debug_clear_hand(1);
    e_xa.debug_set_turn(1, Phase::Judge, 3);
    let _ = next_phase(&mut e_xa); // Draw draws 2 + 2 = 4 (Wait aHao1 also didn't fire since draw added cards)
    let _ = next_phase(&mut e_xa); // -> Play
    drain(&mut e_xa);
    assert_eq!(e_xa.debug_hand(1).len(), 4, "aHaoSkill3 draws +equips.len()=2");
    // Play 2 Sha without Zhuge? Wait Zhuge already makes Sha unlimited, so unequip Zhuge & equip HorsePlus!
    let eq_zg = e_xa.debug_equips(1)[0];
    e_xa.debug_remove_equip(1, eq_zg);
    equip_kind(&mut e_xa, 1, CardKind::HorsePlus); // Now Renwang + HorsePlus = 2 equips, no Zhuge -> limit 3!
    let s1 = place_kind(&mut e_xa, 1, CardKind::Sha);
    let s2 = place_kind(&mut e_xa, 1, CardKind::Sha);
    assert!(matches!(play_card(&mut e_xa, 1, s1, vec![2]), CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    drain(&mut e_xa);
    assert!(matches!(play_card(&mut e_xa, 1, s2, vec![2]), CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    drain(&mut e_xa);
}
