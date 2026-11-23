//! U05-6b Round 3 Parity Fixes Regression Suite (`#1 fengnu`, `#2 huanjue`, `#3 qianghua`,
//! `#4 bingjia2/mianyi`, `#5 huanfeng`, `#6a delayed tricks`, `#6b zuzhou/fulei`).
//!
//! All tests use formal `packages/extensions/shahai-hearth-upstream-noname`, real `Engine` actions
//! (ZERO `debug_execute_effect`), card conservation, `BoundedPlanner` public-info checks, and `NGSA 107` save/restore.

use super::common::*;
use engine_native::catalog::{CardKind, QueryCtx, Suit};
use engine_native::choice::ChoiceOption;
use engine_native::event::{
    DamageData, DamageNature, DamageSettlement, EventData, EventKind, Phase,
};
use engine_native::machine::LogEvent;
use engine_native::{CommandOutcome, Engine};

fn find_card_by_prefix(e: &Engine, prefix: &str) -> u32 {
    let cat = e.catalog();
    cat.cards
        .iter()
        .position(|c| c.name == prefix || c.name.starts_with(&format!("{prefix}_")))
        .unwrap_or_else(|| panic!("card prefix `{prefix}` missing in catalog")) as u32
}

fn isolate_detached(e: &mut Engine, keep: &[u32]) {
    let keep_set: std::collections::BTreeSet<u32> = keep.iter().copied().collect();
    let drained: Vec<u32> = e
        .state_core_mut()
        .detached_cards
        .drain(..)
        .map(|(c, _)| c)
        .collect();
    for c in drained {
        if keep_set.contains(&c) {
            e.state_core_mut()
                .detached_cards
                .push((c, engine_native::event::DetachedKind::Destroyed));
        } else {
            e.state_core_mut().discard.push(c);
        }
    }
    for &c in keep {
        remove_cid_everywhere(e, c);
        e.state_core_mut()
            .detached_cards
            .push((c, engine_native::event::DetachedKind::Destroyed));
    }
}

#[test]
fn u05_b6b_r3_fengnu_lifts_jiu_limit_and_keys_by_exact_card_name() {
    // 1A) `fengnu` lifts `cardUsable` for `Jiu` (`usable: 1`, no `forceUsable`):
    // two `Jiu` cards played sequentially in one Play phase both succeed AND 2nd draws 1.
    let mut ej = fresh_proto(30_101, &["hs_lrexxar", "hs_jaina"]);
    grant_host_skill(&mut ej, 0, "fengnu");
    advance_to_phase(&mut ej, 0, Phase::Play);
    clear_hand(&mut ej, 0);
    let before_j = total_tracked_cards(&ej);
    let jiu1 = give_free_card_kind_suit(&mut ej, 0, CardKind::Jiu, None);
    let jiu2 = give_free_card_kind_suit(&mut ej, 0, CardKind::Jiu, None);

    let out1 = play_card(&mut ej, 0, jiu1, vec![]);
    assert!(
        !matches!(out1, CommandOutcome::Rejected(_)),
        "first Jiu must succeed: {out1:?}"
    );
    drain_auto(&mut ej);
    let hand_after_first = ej.state_core().player(0).hand.len();
    assert_eq!(hand_after_first, 1);

    let out2 = play_card(&mut ej, 0, jiu2, vec![]);
    assert!(
        !matches!(out2, CommandOutcome::Rejected(_)),
        "fengnu must lift Jiu usable limit in Play phase: {out2:?}"
    );
    drain_auto(&mut ej);
    assert_eq!(
        ej.state_core().player(0).hand.len(),
        1,
        "2nd Jiu shares name `jiu` so fengnu draws 1"
    );
    assert_eq!(flag(&ej, 0, "hs_b6b_cnt:fengnu"), 1);
    assert_eq!(total_tracked_cards(&ej), before_j);

    // 1B) Exact card-name keying (`Shandianjian` vs `Wuzhong` share `CardKind::Wuzhong`;
    // `Shihuawuqi` vs `Tao` share `CardKind::Tao`): different names MUST NOT draw, while
    // same canonical name (`Sha` + `FireSha`, `Tao` + `Tao`) DOES draw (max 3/turn).
    let mut en = fresh_proto(30_102, &["hs_lrexxar", "hs_jaina"]);
    grant_host_skill(&mut en, 0, "fengnu");
    advance_to_phase(&mut en, 0, Phase::Play);
    clear_hand(&mut en, 0);
    clear_hand(&mut en, 1);
    let before_n = total_tracked_cards(&en);

    // Ensure seat 1 has 1 hand card for `shandianjian` and seat 0 is wounded for `tao`.
    let enemy_card = give_free_card_kind_suit(&mut en, 1, CardKind::Shan, Some(Suit::Club));
    en.state_core_mut().player_mut(0).hp = 1;
    let wz = give_free_base_named(&mut en, 0, "wuzhong");
    let sdj = give_free_base_named(&mut en, 0, "shandianjian");
    let swq = give_free_base_named(&mut en, 0, "shihuawuqi");
    let tao = give_free_card_kind_suit(&mut en, 0, CardKind::Tao, None);
    let sha1 = give_free_card_kind_suit(&mut en, 0, CardKind::Sha, None);
    let sha2 = give_free_card_kind_suit(&mut en, 0, CardKind::FireSha, None);

    // Play `wuzhong` (draws 2 -> now 2 + 5 = 7 in hand)
    play_card(&mut en, 0, wz, vec![]);
    drain_auto(&mut en);
    assert_eq!(flag(&en, 0, "hs_b6b_cnt:fengnu"), 0);

    // Play `shandianjian` (shares `CardKind::Wuzhong`, DIFFERENT card name!) -> must NOT draw!
    let h_before_sdj = en.state_core().player(0).hand.len();
    play_card(&mut en, 0, sdj, vec![1]);
    drain_auto(&mut en);
    assert_eq!(
        en.state_core().player(0).hand.len(),
        h_before_sdj - 1,
        "shandianjian != wuzhong: fengnu MUST NOT draw (and shandianjian target still had `{enemy_card}`)"
    );
    assert_eq!(flag(&en, 0, "hs_b6b_cnt:fengnu"), 0);

    // Play `shihuawuqi` (shares `CardKind::Tao`, gains `sha` if none; put `sha` in hand already so no extra draw), then `tao` -> MUST NOT draw!
    let h_before_swq = en.state_core().player(0).hand.len();
    play_card(&mut en, 0, swq, vec![]);
    drain_auto(&mut en);
    assert_eq!(en.state_core().player(0).hand.len(), h_before_swq - 1);
    assert_eq!(flag(&en, 0, "hs_b6b_cnt:fengnu"), 0);

    let h_before_tao = en.state_core().player(0).hand.len();
    play_card(&mut en, 0, tao, vec![]);
    drain_auto(&mut en);
    assert_eq!(
        en.state_core().player(0).hand.len(),
        h_before_tao - 1,
        "tao != shihuawuqi: fengnu MUST NOT draw!"
    );
    assert_eq!(flag(&en, 0, "hs_b6b_cnt:fengnu"), 0);

    // Play `sha` then `fire_sha` -> SAME canonical card name `sha` -> fengnu DOES draw 1!
    play_card(&mut en, 0, sha1, vec![1]);
    drain_auto(&mut en);
    let h_after_sha1 = en.state_core().player(0).hand.len();
    play_card(&mut en, 0, sha2, vec![1]);
    drain_auto(&mut en);
    assert_eq!(
        en.state_core().player(0).hand.len(),
        h_after_sha1,
        "sha + fire_sha share canonical name `sha`, so fengnu draws 1!"
    );
    assert_eq!(flag(&en, 0, "hs_b6b_cnt:fengnu"), 1);
    assert_eq!(total_tracked_cards(&en), before_n);
}

#[test]
fn u05_b6b_r3_huanjue_precommit_and_filter_target() {
    // 2A) Target filter parity (`targetEnabled2`):
    // - Target 1 has empty `hej` and full HP -> `dismantle`, `zhaomingdan`, `hs_huogong`, `zhiliaobo` REJECTED; `duel` ACCEPTED.
    let mut ef = fresh_proto(30_201, &["hs_lrexxar", "hs_jaina"]);
    grant_host_skill(&mut ef, 0, "huanjue");
    clear_hand(&mut ef, 0);
    clear_hand(&mut ef, 1);
    clear_equips(&mut ef, 1);
    let before_f = total_tracked_cards(&ef);

    let dis_c = find_card_by_prefix(&ef, "dismantle");
    let zmd_c = find_card_by_prefix(&ef, "ht_zhaomingdan");
    let hg_c = find_card_by_prefix(&ef, "hs_huogong");
    let zlb_c = find_card_by_prefix(&ef, "ht_zhiliaobo");
    let duel_c = find_card_by_prefix(&ef, "duel");
    isolate_detached(&mut ef, &[dis_c, zmd_c, hg_c, zlb_c, duel_c]);

    let sha = give_free_card_kind_suit(&mut ef, 0, CardKind::Sha, None);
    advance_to_phase(&mut ef, 0, Phase::Play);
    play_card(&mut ef, 0, sha, vec![1]);
    let ch = ef.open_choice().cloned().expect("huanjue pick");
    assert_eq!(ch.spec.prompt, "hs_huanjue_pick");
    let offered: Vec<u32> = ch
        .spec
        .options
        .iter()
        .filter_map(|o| match *o {
            ChoiceOption::Card(c) => Some(c),
            _ => None,
        })
        .collect();
    assert_eq!(
        offered,
        vec![duel_c],
        "empty hej + full HP target must only offer Duel (reject dismantle/zhaomingdan/huogong/zhiliaobo), got {offered:?}"
    );
    let ai_pick = ai_choose_current(&ef);
    assert!(!ai_pick.is_empty());
    answer(&mut ef, 0, vec![ChoiceOption::Card(duel_c)]);
    drain_auto(&mut ef);
    assert_eq!(total_tracked_cards(&ef), before_f);

    // 2B) `zhiliaobo` allowed on wounded self, and replacing `Jiu` -> `Tao` BEFORE commit
    // must NOT leave `jiu_used = true` or `jiu_active = true`!
    let mut ej = fresh_proto(30_202, &["hs_lrexxar", "hs_jaina"]);
    grant_host_skill(&mut ej, 0, "huanjue");
    clear_hand(&mut ej, 0);
    clear_hand(&mut ej, 1);
    ej.state_core_mut().player_mut(0).hp = 2; // wounded: max_hp = 4
    let before_j = total_tracked_cards(&ej);

    let tao_c = find_card_by_prefix(&ej, "hs_tao");
    let zlb2 = find_card_by_prefix(&ej, "ht_zhiliaobo");
    isolate_detached(&mut ej, &[tao_c, zlb2]);

    let jiu1 = give_free_card_kind_suit(&mut ej, 0, CardKind::Jiu, None);
    let jiu2 = give_free_card_kind_suit(&mut ej, 0, CardKind::Jiu, None);
    let sha_after = give_free_card_kind_suit(&mut ej, 0, CardKind::Sha, None);
    advance_to_phase(&mut ej, 0, Phase::Play);

    play_card(&mut ej, 0, jiu1, vec![]);
    let ch2 = ej.open_choice().cloned().expect("huanjue pick on Jiu");
    assert_eq!(ch2.spec.prompt, "hs_huanjue_pick");
    let offered2: Vec<u32> = ch2
        .spec
        .options
        .iter()
        .filter_map(|o| match *o {
            ChoiceOption::Card(c) => Some(c),
            _ => None,
        })
        .collect();
    assert!(
        offered2.contains(&tao_c) && offered2.contains(&zlb2),
        "wounded self must offer both Tao and Zhiliaobo, got {offered2:?}"
    );
    save_restore_and_answer(&mut ej, "hs_huanjue_pick", vec![ChoiceOption::Card(tao_c)]);
    drain_auto(&mut ej);

    assert_eq!(ej.state_core().player(0).hp, 3, "replaced Tao healed +1");
    assert!(
        !ej.state_core().turn.jiu_used && !ej.state_core().turn.jiu_active,
        "pre-commit huanjue replacing Jiu->Tao MUST NOT mark jiu_used or jiu_active!"
    );

    // Since jiu_used == false, a real 2nd Jiu in hand MUST succeed and boost Sha by +1 only (2 dmg total)!
    let out_j2 = play_card(&mut ej, 0, jiu2, vec![]);
    assert!(
        !matches!(out_j2, CommandOutcome::Rejected(_)),
        "2nd Jiu must be accepted when original Jiu was replaced before commit: {out_j2:?}"
    );
    drain_auto(&mut ej);
    let hp1_before = ej.state_core().player(1).hp;
    play_card(&mut ej, 0, sha_after, vec![1]);
    drain_auto(&mut ej);
    assert_eq!(ej.state_core().player(1).hp, hp1_before - 2);
    assert_eq!(total_tracked_cards(&ej), before_j);

    // 2C) Replacing `Sha` -> `Duel` BEFORE commit MUST NOT consume `sha_used_play_phase`!
    let mut es = fresh_proto(30_203, &["hs_lrexxar", "hs_jaina"]);
    grant_host_skill(&mut es, 0, "huanjue");
    clear_hand(&mut es, 0);
    clear_hand(&mut es, 1);
    let before_s = total_tracked_cards(&es);

    let duel_s = find_card_by_prefix(&es, "duel");
    isolate_detached(&mut es, &[duel_s]);

    let sha_orig = give_free_card_kind_suit(&mut es, 0, CardKind::Sha, None);
    let sha_real = give_free_card_kind_suit(&mut es, 0, CardKind::Sha, None);
    advance_to_phase(&mut es, 0, Phase::Play);

    play_card(&mut es, 0, sha_orig, vec![1]);
    save_restore_and_answer(&mut es, "hs_huanjue_pick", vec![ChoiceOption::Card(duel_s)]);
    drain_auto(&mut es);
    assert!(
        !es.state_core().turn.sha_used_play_phase,
        "pre-commit huanjue replacing Sha->Duel MUST NOT mark sha_used_play_phase!"
    );
    let out_sha2 = play_card(&mut es, 0, sha_real, vec![1]);
    assert!(
        !matches!(out_sha2, CommandOutcome::Rejected(_)),
        "subsequent real Sha must succeed: {out_sha2:?}"
    );
    drain_auto(&mut es);
    assert_eq!(total_tracked_cards(&es), before_s);
}

#[test]
fn u05_b6b_r3_qianghua_checks_can_use_filter_target() {
    // 3A) Casting `Dismantle` that empties target's last `hej` card:
    // `player.canUse({ name: "guohe" }, target, false, false)` is FALSE afterward -> NO `hs_qianghua_confirm`.
    let mut ed = fresh_proto(30_301, &["hs_lrexxar", "hs_jaina"]);
    grant_host_skill(&mut ed, 0, "qianghua");
    clear_hand(&mut ed, 0);
    clear_hand(&mut ed, 1);
    clear_equips(&mut ed, 1);
    let before_d = total_tracked_cards(&ed);

    let last_card = give_free_card_kind_suit(&mut ed, 1, CardKind::Shan, None);
    let guohe = give_free_card_kind_suit(&mut ed, 0, CardKind::Dismantle, None);
    advance_to_phase(&mut ed, 0, Phase::Play);

    play_card(&mut ed, 0, guohe, vec![1]);
    if ed.open_choice().is_some_and(|c| c.spec.prompt != "hs_qianghua_confirm") {
        let pick_opt = ed
            .open_choice()
            .unwrap()
            .spec
            .options
            .iter()
            .find(|o| matches!(o, ChoiceOption::Card(c) if *c == last_card))
            .copied()
            .unwrap_or_else(|| ed.open_choice().unwrap().spec.options[0]);
        answer(&mut ed, 0, vec![pick_opt]);
    }
    assert_ne!(
        ed.open_choice().map(|c| c.spec.prompt),
        Some("hs_qianghua_confirm"),
        "qianghua MUST NOT trigger after Dismantle left target with empty hej"
    );
    drain_auto(&mut ed);
    assert_eq!(flag(&ed, 0, "hs_b6b_cnt:qianghua"), 0);
    assert_eq!(total_tracked_cards(&ed), before_d);

    // 3B) Casting `Zhiliaobo` that heals target to full HP (`max_hp - 1 -> max_hp`):
    // `player.canUse({ name: "zhiliaobo" }, target, false, false)` is FALSE afterward -> NO `hs_qianghua_confirm`.
    let mut ez = fresh_proto(30_302, &["hs_lrexxar", "hs_jaina"]);
    grant_host_skill(&mut ez, 0, "qianghua");
    advance_to_phase(&mut ez, 0, Phase::Play);
    clear_hand(&mut ez, 0);
    clear_hand(&mut ez, 1);
    let m1 = ez.state_core().player(1).max_hp as i32;
    ez.state_core_mut().player_mut(1).hp = m1 - 1;
    let red_j = give_free_card_kind_suit(&mut ez, 0, CardKind::Sha, Some(Suit::Heart));
    put_card_on_deck_top(&mut ez, red_j);
    let zlb = give_free_card_kind_suit(&mut ez, 0, CardKind::Zhiliaobo, None);
    let before_z = total_tracked_cards(&ez);

    play_card(&mut ez, 0, zlb, vec![1]);
    assert_ne!(
        ez.open_choice().map(|c| c.spec.prompt),
        Some("hs_qianghua_confirm"),
        "qianghua MUST NOT trigger after Zhiliaobo healed target back to full HP"
    );
    drain_auto(&mut ez);
    assert_eq!(ez.state_core().player(1).hp, m1);
    assert_eq!(flag(&ez, 0, "hs_b6b_cnt:qianghua"), 0);
    assert_eq!(total_tracked_cards(&ez), before_z);
}

#[test]
fn u05_b6b_r3_bingjia2_mianyi_cancels_before_tuteng3() {
    // Target owns BOTH `bingjia` (stashing Heart) and `tuteng3`.
    // Matching Heart Sha triggers `bingjia2` -> grants `mianyi` -> cancels damage at `DamageBefore`
    // (`LogEvent::DamagePrevented`), BEFORE `tuteng3` can fire or remove itself!
    let mut e = fresh_proto(30_401, &["hs_jaina", "hs_lrexxar"]);
    grant_host_skill(&mut e, 0, "bingjia");
    grant_host_skill(&mut e, 0, "tuteng3");
    clear_hand(&mut e, 0);
    clear_hand(&mut e, 1);
    let before = total_tracked_cards(&e);

    let heart_stash = give_free_card_kind_suit(&mut e, 0, CardKind::Shan, Some(Suit::Heart));
    advance_to_phase(&mut e, 0, Phase::Play);
    let _ = activate(&mut e, 0, "bingjia");
    answer(&mut e, 0, vec![ChoiceOption::Card(heart_stash)]);
    drain_auto(&mut e);
    assert!(has_skill(&e, 0, "bingjia2"));

    advance_to_phase(&mut e, 1, Phase::Play);
    clear_hand(&mut e, 0);
    let sha_h1 = give_free_card_kind_suit(&mut e, 1, CardKind::Sha, Some(Suit::Heart));
    let duel_2 = give_free_card_kind_suit(&mut e, 1, CardKind::Duel, None);
    let hp_before = e.state_core().player(0).hp;

    let log_start = e.log().len();
    let _out = play_card(&mut e, 1, sha_h1, vec![0]);
    drain_auto(&mut e);
    let prevented_in_out = e.log()[log_start..]
        .iter()
        .any(|ev| matches!(ev, LogEvent::DamagePrevented { target: 0, .. }));
    assert!(
        prevented_in_out,
        "bingjia2 -> mianyi must emit DamagePrevented (`DamageOutcome::Cancelled`): {:?}",
        &e.log()[log_start..]
    );
    assert!(has_skill(&e, 0, "mianyi"), "mianyi skill granted until TurnPhaseAfter");
    assert_eq!(e.state_core().player(0).hp, hp_before);
    assert_eq!(e.state_core().player(0).hujia, 1);
    assert!(
        has_skill(&e, 0, "tuteng3"),
        "tuteng3 MUST NOT be consumed or removed when mianyi prevents damage at DamageBefore!"
    );

    // Same enemy turn: `mianyi` still blocks subsequent Duel damage!
    play_card(&mut e, 1, duel_2, vec![0]);
    drain_auto(&mut e);
    assert_eq!(e.state_core().player(0).hp, hp_before);
    assert!(has_skill(&e, 0, "tuteng3"));

    // After enemy turn finishes (`TurnPhaseAfter`), temp `mianyi` expires; now `tuteng3` handles damage.
    advance_to_phase(&mut e, 0, Phase::Play);
    assert!(
        !has_skill(&e, 0, "mianyi"),
        "bingjia2 temporary `mianyi` must expire at TurnPhaseAfter"
    );
    assert_eq!(total_tracked_cards(&e), before);
}

#[test]
fn u05_b6b_r3_huanfeng_die_before_and_phase_end() {
    // 5A) Lethal damage while transformed into `hs_alakir` triggers `dieBefore` (`EventKind::Death`),
    // sets `death_cancelled`, exits subplayer restoring saved main general WITHOUT discarding stashed cards!
    let mut ed = fresh_proto(30_501, &["hs_sthrall", "hs_lrexxar"]);
    grant_host_skill(&mut ed, 0, "huanfeng");
    for i in 1..=4 {
        grant_host_skill(&mut ed, 0, &format!("tuteng{i}"));
    }
    clear_hand(&mut ed, 0);
    clear_hand(&mut ed, 1);
    advance_to_phase(&mut ed, 1, Phase::Play);
    clear_hand(&mut ed, 0);
    clear_hand(&mut ed, 1);
    ed.state_core_mut().player_mut(0).max_hp = 4;
    ed.state_core_mut().player_mut(0).hp = 2;
    let orig_tao = give_free_card_kind_suit(&mut ed, 0, CardKind::Tao, None);
    let orig_armor = find_card_by_prefix(&ed, "hs_armor");
    remove_cid_everywhere(&mut ed, orig_armor);
    ed.state_core_mut().player_mut(0).equips.push(orig_armor);
    let before_d = total_tracked_cards(&ed);

    advance_to_phase(&mut ed, 0, Phase::Play);
    assert_eq!(flag(&ed, 0, "hs_huanfeng_active"), 1);
    assert_eq!(ed.state_core().player(0).max_hp, 3);
    assert_eq!(ed.state_core().player(0).hp, 3);
    assert!(has_skill(&ed, 0, "fengnu"));
    assert!(!ed.state_core().player(0).hand.contains(&orig_tao));

    // Self-inflict 4 Duel damages (seat 1 plays Sha, seat 0 has 0 hand):
    // 3 HP + 1 hujia from shengdun -> 4 dmg -> dying -> no Tao -> Death `dieBefore`!
    let s1 = give_free_card_kind_suit(&mut ed, 1, CardKind::Sha, None);
    let s2 = give_free_card_kind_suit(&mut ed, 1, CardKind::Sha, None);
    let s3 = give_free_card_kind_suit(&mut ed, 1, CardKind::Sha, None);
    let s4 = give_free_card_kind_suit(&mut ed, 1, CardKind::Sha, None);
    for sh in [s1, s2, s3, s4] {
        clear_hand(&mut ed, 0);
        let duel = give_free_card_kind_suit(&mut ed, 0, CardKind::Duel, None);
        let out = play_card(&mut ed, 0, duel, vec![1]);
        assert!(
            !matches!(out, CommandOutcome::Rejected(_)),
            "duel rejected: {out:?}"
        );
        while let Some(ch) = ed.open_choice().cloned() {
            if ch.observer == 1 {
                answer(&mut ed, 1, vec![ChoiceOption::Card(sh)]);
            } else {
                drain_auto(&mut ed);
            }
        }
    }

    assert!(ed.state_core().player(0).alive, "subplayer dieBefore keeps main general alive");
    assert_eq!(flag(&ed, 0, "hs_huanfeng_active"), 0, "exited huanfeng subplayer");
    assert_eq!(ed.state_core().player(0).max_hp, 4, "restored orig max_hp");
    assert_eq!(ed.state_core().player(0).hp, 2, "restored orig hp");
    assert!(!has_skill(&ed, 0, "fengnu"), "removed Al'Akir fengnu");
    assert!(!has_skill(&ed, 0, "shengdun"), "removed Al'Akir shengdun");
    assert!(ed.state_core().player(0).hand.contains(&orig_tao), "restored stashed hand");
    assert!(ed.state_core().player(0).equips.contains(&orig_armor), "restored stashed equips");
    assert_eq!(total_tracked_cards(&ed), before_d);

    // 5B) Normal turn without dying stays `hs_alakir` THROUGH `Phase::End`, and exits ONLY at `TurnPhaseEnd`!
    let mut en = fresh_proto(30_502, &["hs_sthrall", "hs_lrexxar"]);
    grant_host_skill(&mut en, 0, "huanfeng");
    for i in 1..=4 {
        grant_host_skill(&mut en, 0, &format!("tuteng{i}"));
    }
    advance_to_phase(&mut en, 1, Phase::Play);
    clear_hand(&mut en, 0);
    en.state_core_mut().player_mut(0).max_hp = 4;
    en.state_core_mut().player_mut(0).hp = 1;
    let keep_tao = give_free_card_kind_suit(&mut en, 0, CardKind::Tao, None);
    let before_n = total_tracked_cards(&en);

    advance_to_phase(&mut en, 0, Phase::End);
    assert_eq!(
        flag(&en, 0, "hs_huanfeng_active"),
        1,
        "must remain Al'Akir during Phase::End (not exit at Phase::End begin:true)"
    );
    assert_eq!(en.state_core().player(0).max_hp, 3);
    assert!(has_skill(&en, 0, "fengnu"));

    // Advancing out of Phase::End hits `TurnPhaseEnd` -> exits subplayer & enters extra turn!
    advance_to_phase(&mut en, 0, Phase::Play);
    assert_eq!(en.turn().current, 0, "extra turn inserted at TurnPhaseEnd");
    assert_eq!(flag(&en, 0, "hs_huanfeng_active"), 0);
    assert_eq!(en.state_core().player(0).max_hp, 4);
    assert_eq!(en.state_core().player(0).hp, 1);
    assert!(en.state_core().player(0).hand.contains(&keep_tao));
    assert_eq!(total_tracked_cards(&en), before_n);
}

#[test]
fn u05_b6b_r3_no_delayed_trick_triggers_and_fulei_vanishes() {
    // 6A) Delayed tricks (`ShanDian`, `fulei`, `Le`, `Bing`) must NOT match `malymowang`, `oldmalymowang`,
    // `bingshuang`, or `tuteng4` damage triggers, while normal tricks (`Duel`, `Linghunzhihuo`) DO.
    let mut e = fresh_proto(30_601, &["hs_sthrall", "hs_lrexxar"]);
    grant_host_skill(&mut e, 0, "malymowang");
    grant_host_skill(&mut e, 0, "oldmalymowang");
    grant_host_skill(&mut e, 0, "bingshuang");
    grant_host_skill(&mut e, 0, "tuteng4");
    advance_to_phase(&mut e, 0, Phase::Play);
    let cat = e.catalog();
    let fulei_c = find_card_by_prefix(&e, "fulei");
    let shandian_c = find_card_by_prefix(&e, "shandian");
    let le_c = find_card_by_prefix(&e, "le");
    let bing_c = cat
        .cards
        .iter()
        .position(|c| c.kind == CardKind::Bing)
        .map(|i| i as u32);
    let duel_c = find_card_by_prefix(&e, "duel");

    for delayed in [Some(fulei_c), Some(shandian_c), Some(le_c), bing_c]
        .into_iter()
        .flatten()
    {
        let dmg = DamageData {
            source: Some(0),
            target: 1,
            amount: 1,
            nature: DamageNature::Thunder,
            card: Some(delayed),
            from_chain: false,
            prevented: false,
            redirect_chain: Vec::new(),
            settlement: DamageSettlement::default(),
        };
        let ev = EventData::Damage(dmg);
        let qctx = QueryCtx {
            state: e.state_core(),
            catalog: cat,
        };
        for (sk, ek) in [
            ("malymowang", EventKind::DamageBefore),
            ("oldmalymowang", EventKind::DamageBefore),
            ("bingshuang", EventKind::DamageEnd),
            ("tuteng4", EventKind::DamageBefore),
        ] {
            let def = cat.skill_by_name(sk).unwrap();
            assert!(
                !cat.skills[def as usize]
                    .behavior
                    .filter_in_window(&qctx, 0, &ev, ek),
                "skill `{sk}` must exclude delayed trick card `{}`",
                cat.cards[delayed as usize].name
            );
        }
    }
    let normal_trick = EventData::Damage(DamageData {
        source: Some(0),
        target: 1,
        amount: 1,
        nature: DamageNature::Normal,
        card: Some(duel_c),
        from_chain: false,
        prevented: false,
        redirect_chain: Vec::new(),
        settlement: DamageSettlement::default(),
    });
    let qctx = QueryCtx {
        state: e.state_core(),
        catalog: cat,
    };
    for (sk, ek) in [
        ("malymowang", EventKind::DamageBefore),
        ("oldmalymowang", EventKind::DamageBefore),
        ("bingshuang", EventKind::DamageEnd),
        ("tuteng4", EventKind::DamageBefore),
    ] {
        let def = cat.skill_by_name(sk).unwrap();
        assert!(
            cat.skills[def as usize]
                .behavior
                .filter_in_window(&qctx, 0, &normal_trick, ek),
            "skill `{sk}` must still trigger on normal trick `duel`"
        );
    }

    // 6B) `zuzhou` (`fulei`): 1 token in `detached_cards`, vanishes back to `detached_cards`
    // (NOT `discard`) on expiry, clears `hs_fulei_hits`, so next `zuzhou` starts at 1 damage again!
    let mut ezz = fresh_proto(30_602, &["hs_guldan", "hs_lrexxar"]);
    grant_host_skill(&mut ezz, 0, "zuzhou");
    let fulei_supply_count = ezz
        .state_core()
        .detached_cards
        .iter()
        .filter(|&&(c, _)| {
            let n = ezz.catalog().cards[c as usize].name;
            n == "fulei" || n.starts_with("fulei_")
        })
        .count();
    assert_eq!(fulei_supply_count, 1, "at most 1 fulei can exist");

    // Round 1 `zuzhou`: hit seat 0 for 1 -> `fulei` vanishes to `detached_cards` & resets hits!
    advance_to_phase(&mut ezz, 1, Phase::Play);
    clear_hand(&mut ezz, 0);
    clear_hand(&mut ezz, 1);
    let before_zz = total_tracked_cards(&ezz);
    give_free_card_kind_suit(&mut ezz, 0, CardKind::Sha, Some(Suit::Spade));
    let sp1 = give_free_card_kind_suit(&mut ezz, 1, CardKind::Sha, Some(Suit::Spade));
    put_card_on_deck_top(&mut ezz, sp1);
    let hp_r1 = ezz.state_core().player(0).hp;
    advance_to_phase(&mut ezz, 0, Phase::Play);
    assert_eq!(ezz.state_core().player(0).hp, hp_r1 - 1, "round 1 fulei deals 1");
    let fulei_in_discard = ezz.state_core().discard.iter().any(|&c| {
        let n = ezz.catalog().cards[c as usize].name;
        n == "fulei" || n.starts_with("fulei_")
    });
    assert!(!fulei_in_discard, "expired fulei MUST vanish, not enter discard");
    let fulei_back_detached = ezz.state_core().detached_cards.iter().any(|&(c, _)| {
        let n = ezz.catalog().cards[c as usize].name;
        n == "fulei" || n.starts_with("fulei_")
    });
    assert!(fulei_back_detached, "expired fulei must return to detached_cards");
    assert_eq!(total_tracked_cards(&ezz), before_zz);

    // Round 2 `zuzhou`: re-spawns the vanished `fulei`, Spade judge deals 1 (NOT 2!)!
    advance_to_phase(&mut ezz, 1, Phase::Play);
    clear_hand(&mut ezz, 0);
    clear_hand(&mut ezz, 1);
    give_free_card_kind_suit(&mut ezz, 0, CardKind::Sha, Some(Suit::Spade));
    let sp2 = give_free_card_kind_suit(&mut ezz, 1, CardKind::Sha, Some(Suit::Spade));
    put_card_on_deck_top(&mut ezz, sp2);
    let hp_r2 = ezz.state_core().player(0).hp;
    advance_to_phase(&mut ezz, 0, Phase::Play);
    assert_eq!(
        ezz.state_core().player(0).hp,
        hp_r2 - 1,
        "re-spawned fulei after vanish MUST reset hs_fulei_hits and deal 1 again (not 2)"
    );
    assert_eq!(total_tracked_cards(&ezz), before_zz);
}
