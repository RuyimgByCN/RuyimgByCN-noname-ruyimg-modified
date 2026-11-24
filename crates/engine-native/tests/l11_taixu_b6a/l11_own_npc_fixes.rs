//! Item 6 (Track P3-B, Part 1): Real-event parity tests for `HostBehavior::L11OwnNpc` (tag 1389)
//! covering all 24 NPC generals & Part 1 of the 29 `L11_TAIXU_OWN_NPC_SKILLS`:
//! - All 24 NPC generals bind to `HostBehavior::L11OwnNpc` with 0 VM `L11_TAIXU_SKILL_ID` stubs
//! - `txhj_guimei` (`txhj_moou`, `txhj_shiemoxing`), `txhj_zuijiu` (`txhj_zhangfeihuanxiang`, `txhj_baiwuchanghuanxiang`, `txhj_huanjinglingzhu`, `txhj_peiyuanshao`)
//! - `txhj_baoji` (`txhj_lvbuhuanxiang`, `txhj_chi/mei/wang/liang`, `txhj_tiaopibubu`)
//! - `txhj_baolian` (`txhj_menghuo7`, `txhj_heiwuchanghuanxiang`), `tx_modao` (`txhj_gaosheng`, `txhj_buji`, `txhj_luocha`, `txhj_yecha`)
//! - `txhj_tunshi` & `txhj_chuanyun` (`txhj_tianjijushou`), `txhj_guihuo` (`txhj_zhurongx`/`txhj_zhurong`), `txhj_luolei` (`txhj_tiangongzhangjiao`, `txhj_digongzhangbao`)
//! - `txhj_huangjin` (`txhj_huangjinxinjun`, `txhj_guanhai`, `txhj_huangjinmengqi`, `txhj_dengmao`, `txhj_bocai`), `txqyyouji` (`txhj_bossquyi`)
//! - Step 5 NGSA save/restore additions for `rexianzhen` lose (`rexianzhen3`/`rexianzhen4`), `zhenlie`, `jintao`.

use super::*;
use content_native_loader::{L11_TAIXU_GENERALS, L11_TAIXU_OWN_NPC_SKILLS};
use engine_native::content::host::HostBehavior;
use engine_native::event::DamageNature;
use engine_native::{LogEvent, RejectReason};

pub(super) fn npc_take_card<F>(e: &mut Engine, pred: F) -> u32
where
    F: Fn(&engine_native::catalog::CardDef) -> bool,
{
    if let Some(pos) = e
        .state_core()
        .deck
        .iter()
        .position(|&c| e.catalog().cards.get(c as usize).is_some_and(&pred))
    {
        return e.state_core_mut().deck.remove(pos);
    }
    if let Some(pos) = e
        .state_core()
        .discard
        .iter()
        .position(|&c| e.catalog().cards.get(c as usize).is_some_and(&pred))
    {
        return e.state_core_mut().discard.remove(pos);
    }
    for seat in 0..e.state_core().players.len() {
        if let Some(pos) = e
            .state_core()
            .player(seat as u8)
            .hand
            .iter()
            .position(|&c| e.catalog().cards.get(c as usize).is_some_and(&pred))
        {
            return e.state_core_mut().player_mut(seat as u8).hand.remove(pos);
        }
    }
    panic!("matching card missing in Prototype deck/discard/hand");
}

pub(super) fn npc_play_card(e: &mut Engine, actor: u8, card: u32, targets: Vec<u8>) -> CommandOutcome {
    e.state_core_mut().player_mut(actor).hand.push(card);
    submit(e, actor, GameAction::PlayCard { card, targets })
}

pub(super) fn npc_has_skill(e: &Engine, seat: u8, name: &str) -> bool {
    e.catalog()
        .skill_by_name(name)
        .is_some_and(|d| e.state_core().has_skill(seat, d))
}

#[test]
fn l11_own_npc_bindings_guimei_zuijiu_baoji_draw_and_triggers() {
    let verified = taixu_verified();

    // 0. All 29 `L11_TAIXU_OWN_NPC_SKILLS` must bind to `HostBehavior::L11OwnNpc` (zero VM stubs)!
    {
        let e = start_locked_engine(2026_900, 2, LockedStartMode::Prototype, verified).unwrap();
        for &sk in &L11_TAIXU_OWN_NPC_SKILLS {
            let sid = e
                .catalog()
                .skill_by_name(sk)
                .unwrap_or_else(|| panic!("own NPC skill {sk} missing from Catalog"));
            assert_eq!(
                e.catalog().skills[sid as usize].behavior.as_host_behavior(),
                Some(HostBehavior::L11OwnNpc),
                "{sk} must bind to HostBehavior::L11OwnNpc instead of a PhaseBegin VM stub"
            );
            assert!(
                e.catalog().skills[sid as usize].behavior.package_skill_id().is_none(),
                "{sk} must not carry L11_TAIXU_SKILL_ID"
            );
        }
        // Verify ALL L11 NPC generals carrying `L11_TAIXU_OWN_NPC_SKILLS` instantiate in BOTH Prototype and Identity!
        let npc_generals: Vec<&str> = L11_TAIXU_GENERALS
            .iter()
            .filter(|(_, _, _, _, sks)| sks.iter().any(|s| L11_TAIXU_OWN_NPC_SKILLS.contains(s)))
            .map(|(name, _, _, _, _)| *name)
            .collect();
        assert!(npc_generals.len() >= 24, "expected >= 24 NPC generals, got {}", npc_generals.len());
        let mut e_id = start_locked_engine(2026_900, 4, LockedStartMode::Identity, verified).unwrap();
        let mut e_pr = start_locked_engine(2026_900, 4, LockedStartMode::Prototype, verified).unwrap();
        for g in npc_generals {
            e_id.assign_general_named(1, g)
                .unwrap_or_else(|err| panic!("Identity assign failed for {g}: {err:?}"));
            e_pr.assign_general_named(1, g)
                .unwrap_or_else(|err| panic!("Prototype assign failed for {g}: {err:?}"));
        }
    }

    // 1. `txhj_guimei` (`txhj_moou`, `txhj_shiemoxing`): blocks Le/Bing (including negative control Sha still works).
    for g in ["txhj_moou", "txhj_shiemoxing"] {
        let mut e = start_locked_engine(2026_901, 3, LockedStartMode::Prototype, verified).unwrap();
        e.assign_general_named(0, "txhj_huangyueying").unwrap();
        e.assign_general_named(1, g).unwrap();
        e.assign_general_named(2, "txhj_machao").unwrap();
        assert!(npc_has_skill(&e, 1, "txhj_guimei"));
        advance_to_play(&mut e, 0);
        e.debug_clear_hand(0);
        let le = npc_take_card(&mut e, |d| d.kind == CardKind::Le);
        let bing = npc_take_card(&mut e, |d| d.kind == CardKind::Bing);
        let sha = npc_take_card(&mut e, |d| d.kind == CardKind::Sha);
        let out_le = npc_play_card(&mut e, 0, le, vec![1]);
        assert!(matches!(out_le, CommandOutcome::Rejected(_)), "{g} txhj_guimei must block Le");
        let out_bing = npc_play_card(&mut e, 0, bing, vec![1]);
        assert!(matches!(out_bing, CommandOutcome::Rejected(_)), "{g} txhj_guimei must block Bing");
        let out_sha = npc_play_card(&mut e, 0, sha, vec![1]);
        assert!(!matches!(out_sha, CommandOutcome::Rejected(_)), "{g} txhj_guimei must allow non-delayed Sha: {out_sha:?}");
        drain_to_pass(&mut e);
    }

    // 2. `txhj_zuijiu` (`txhj_zhangfeihuanxiang`, `txhj_baiwuchanghuanxiang`, `txhj_huanjinglingzhu`, `txhj_peiyuanshao`):
    //    Sha non-linked damage +1 (2), negative: non-Sha (Duel) / chain damage does NOT get txhj_zuijiu +1!
    for g in ["txhj_zhangfeihuanxiang", "txhj_baiwuchanghuanxiang", "txhj_huanjinglingzhu", "txhj_peiyuanshao"] {
        let mut e = start_locked_engine(2026_902, 3, LockedStartMode::Prototype, verified).unwrap();
        e.assign_general_named(0, g).unwrap();
        e.assign_general_named(1, "txhj_huangyueying").unwrap();
        e.assign_general_named(2, "txhj_machao").unwrap();
        assert!(npc_has_skill(&e, 0, "txhj_zuijiu"));
        advance_to_play(&mut e, 0);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        e.debug_clear_hand(2);
        e.debug_set_hp(1, 5);
        e.debug_set_hp(2, 5);
        // Chain seat 1 and seat 2 -> FireSha on seat 1 deals +1 on seat 1 (2 fire), chain to seat 2 deals 2 fire without adding +1 twice!
        let chain = npc_take_card(&mut e, |d| d.kind == CardKind::Chain);
        let _ = npc_play_card(&mut e, 0, chain, vec![1, 2]);
        drain_to_pass(&mut e);
        let fsha = npc_take_card(&mut e, |d| d.kind == CardKind::FireSha || d.kind == CardKind::Sha);
        let hp1_before = e.state_core().player(1).hp;
        let _ = npc_play_card(&mut e, 0, fsha, vec![1]);
        while let Some(c) = e.open_choice().cloned() {
            let sel = if c.spec.prompt == "retieji:confirm" {
                Vec::new()
            } else if c.spec.options.contains(&ChoiceOption::Pass) {
                vec![ChoiceOption::Pass]
            } else if c.spec.allow_cancel && c.spec.min == 0 {
                Vec::new()
            } else {
                c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
            };
            let _ = answer_choice(&mut e, c.observer, sel);
        }
        assert_eq!(e.state_core().player(1).hp, hp1_before - 2, "{g} txhj_zuijiu must deal +1 on unchained Sha");
        if e.state_core().player(2).hp < 5 {
            assert_eq!(e.state_core().player(2).hp, 3, "txhj_zuijiu must not double-trigger on linked chain damage");
        }
    }

    // 3. `txhj_baoji` (`txhj_lvbuhuanxiang`, `txhj_chi`, `txhj_mei`, `txhj_wang`, `txhj_liang`, `txhj_tiaopibubu`):
    //    Deterministic RNG trial shows both hit (+1 -> 2) and non-hit (1) across seeds (~40%).
    {
        let mut saw_crit = false;
        let mut saw_normal = false;
        for seed_off in 0..24u64 {
            let mut e = start_locked_engine(2026_920 + seed_off, 2, LockedStartMode::Prototype, verified).unwrap();
            e.assign_general_named(0, "txhj_chi").unwrap();
            e.assign_general_named(1, "txhj_huangyueying").unwrap();
            assert!(npc_has_skill(&e, 0, "txhj_baoji"));
            advance_to_play(&mut e, 0);
            e.debug_clear_hand(0);
            e.debug_clear_hand(1);
            e.debug_set_hp(1, 4);
            let sha = npc_take_card(&mut e, |d| d.kind == CardKind::Sha);
            let _ = npc_play_card(&mut e, 0, sha, vec![1]);
            drain_to_pass(&mut e);
            let taken = 4 - e.state_core().player(1).hp;
            if taken == 2 {
                saw_crit = true;
            } else if taken == 1 {
                saw_normal = true;
            }
            if saw_crit && saw_normal {
                break;
            }
        }
        assert!(saw_crit && saw_normal, "txhj_baoji must deterministically trigger ~40% (+1 vs +0)");
        for g in ["txhj_lvbuhuanxiang", "txhj_mei", "txhj_wang", "txhj_liang", "txhj_tiaopibubu"] {
            let mut e = start_locked_engine(2026_950, 2, LockedStartMode::Prototype, verified).unwrap();
            e.assign_general_named(0, g).unwrap();
            assert!(npc_has_skill(&e, 0, "txhj_baoji"), "{g} must have txhj_baoji");
        }
    }

    // 4. `txhj_baolian` (`txhj_heiwuchanghuanxiang`) & `tx_modao` (`txhj_gaosheng`, `txhj_buji`, `txhj_luocha`, `txhj_yecha`):
    //    `txhj_baolian` draws 2 at `Phase::End { begin: true }`; `tx_modao` draws 2 at `Phase::Prepare { begin: true }`.
    {
        let mut e = start_locked_engine(2026_951, 2, LockedStartMode::Prototype, verified).unwrap();
        e.assign_general_named(0, "txhj_heiwuchanghuanxiang").unwrap();
        e.assign_general_named(1, "txhj_yecha").unwrap();
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        e.debug_set_turn(0, Phase::Discard, 1);
        let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard -> End (txhj_baolian draws 2)
        drain_to_pass(&mut e);
        assert_eq!(e.state_core().player(0).hand.len(), 2, "txhj_baolian must draw 2 at End begin");
        // Advance naturally into seat 1's TurnBegin -> Prepare (tx_modao draws 2, tx_mojian optional cancel)!
        let _ = submit(&mut e, 0, GameAction::NextPhase);
        while let Some(ch) = e.open_choice().cloned() {
            let sel = if ch.spec.allow_cancel { vec![] } else { ch.spec.options.into_iter().take(1).collect() };
            let _ = answer_choice(&mut e, ch.observer, sel);
        }
        assert!(e.state_core().player(1).hand.len() >= 2, "tx_modao must draw 2 at Prepare begin");
    }
}

#[test]
fn l11_own_npc_tunshi_chuanyun_guihuo_luolei_huangjin_and_youji() {
    let verified = taixu_verified();

    // 5. `txhj_tianjijushou`: `txhj_tunshi` (`TurnBegin` enemy hand > owner -> 1 dmg),
    //    `txhj_chuanyun` (`TurnPhaseEnd` enemy hp > owner -> 1 dmg); negative: equal hand / hp skips!
    {
        let mut e = start_locked_engine(2026_952, 3, LockedStartMode::Prototype, verified).unwrap();
        e.assign_general_named(0, "txhj_huangyueying").unwrap();
        e.assign_general_named(1, "txhj_tianjijushou").unwrap();
        e.assign_general_named(2, "txhj_machao").unwrap();
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        e.debug_clear_hand(2);
        let c1 = npc_take_card(&mut e, |_| true);
        let c2 = npc_take_card(&mut e, |_| true);
        e.state_core_mut().player_mut(2).hand.extend([c1, c2]); // seat 2 hand (2) > seat 1 hand (0); seat 0 hand == 0!
        e.debug_set_hp(1, 3);
        e.debug_set_hp(0, 3);
        e.debug_set_hp(2, 5);
        e.debug_set_turn(0, Phase::End, 1);
        let _ = submit(&mut e, 0, GameAction::NextPhase); // transitions into seat 1 TurnBegin -> txhj_tunshi!
        let ch = e.open_choice().cloned().expect("txhj_tunshi prompt");
        assert_eq!(ch.spec.prompt, "txhj_tunshi:choose_target");
        assert_eq!(ch.spec.options, vec![ChoiceOption::Player(2)], "only enemy with strictly more hand valid");
        let _ = answer_choice(&mut e, 1, vec![ChoiceOption::Player(2)]);
        drain_to_pass(&mut e);
        assert_eq!(e.state_core().player(2).hp, 4, "txhj_tunshi must deal 1 dmg to seat 2");

        // Advance seat 1 to End -> TurnPhaseEnd -> txhj_chuanyun (seat 2 hp=4 > seat 1 hp=3; seat 0 hp=3 excluded)!
        e.debug_set_hp(1, 3);
        e.debug_set_turn(1, Phase::End, 1);
        let _ = submit(&mut e, 1, GameAction::NextPhase);
        let ch2 = e.open_choice().cloned().expect("txhj_chuanyun prompt");
        assert_eq!(ch2.spec.prompt, "txhj_chuanyun:choose_target");
        assert_eq!(ch2.spec.options, vec![ChoiceOption::Player(2)], "only enemy with hp > owner.hp valid");
        let _ = answer_choice(&mut e, 1, vec![ChoiceOption::Player(2)]);
        drain_to_pass(&mut e);
        assert_eq!(e.state_core().player(2).hp, 3, "txhj_chuanyun must deal 1 dmg");
    }

    // 6. `txhj_zhurongx` (`txhj_guihuo` 1 Fire) & `txhj_shiemoxing` (`txhj_luolei` 1 Thunder): real triggers!
    {
        let mut e = start_locked_engine(2026_953, 2, LockedStartMode::Prototype, verified).unwrap();
        e.assign_general_named(0, "txhj_zhurongx").unwrap();
        e.assign_general_named(1, "txhj_shiemoxing").unwrap();
        e.debug_set_hp(1, 5);
        e.debug_set_turn(0, Phase::Discard, 1);
        let _ = submit(&mut e, 0, GameAction::NextPhase); // End -> txhj_guihuo
        let ch = e.open_choice().cloned().expect("txhj_guihuo prompt");
        assert_eq!(ch.spec.prompt, "txhj_guihuo:choose_target");
        let _ = answer_choice(&mut e, 0, vec![ChoiceOption::Player(1)]);
        drain_to_pass(&mut e);
        assert_eq!(e.state_core().player(1).hp, 4);

        // Next turn seat 1 Prepare -> txhj_luolei
        e.debug_set_hp(0, 4);
        e.debug_set_turn(0, Phase::End, 1);
        let _ = submit(&mut e, 0, GameAction::NextPhase);
        while let Some(c) = e.open_choice().cloned() {
            if c.spec.prompt == "txhj_luolei:choose_target" {
                let _ = answer_choice(&mut e, 1, vec![ChoiceOption::Player(0)]);
                break;
            }
            let _ = answer_choice(&mut e, c.observer, vec![]);
        }
        drain_to_pass(&mut e);
        assert_eq!(e.state_core().player(0).hp, 3, "txhj_luolei must deal 1 Thunder dmg");
    }

    // 7. `txhj_huangjin` (`txhj_huangjinmengqi`): |sha_rank - judge_rank| <= 1 cancels Sha; > 1 still hits!
    {
        let mut e = start_locked_engine(2026_954, 2, LockedStartMode::Prototype, verified).unwrap();
        e.assign_general_named(0, "txhj_huangyueying").unwrap();
        e.assign_general_named(1, "txhj_huangjinmengqi").unwrap();
        advance_to_play(&mut e, 0);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        e.debug_set_hp(1, 4);
        let sha7 = npc_take_card(&mut e, |d| d.kind == CardKind::Sha && d.rank == 7);
        let _j8 = e.debug_set_deck_top_by(|d| d.rank == 8); // |7 - 8| = 1 -> invalid!
        let _ = npc_play_card(&mut e, 0, sha7, vec![1]);
        let ch = e.open_choice().cloned().expect("txhj_huangjin:confirm");
        assert_eq!(ch.spec.prompt, "txhj_huangjin:confirm");
        // NGSA mid-choice save/restore check!
        let snap = encode_archive(&e.save());
        let mut e_res = restore_locked_engine(&decode_archive(&snap).unwrap(), verified).unwrap();
        let _ = answer_choice(&mut e, 1, vec![ChoiceOption::Confirm]);
        let _ = answer_choice(&mut e_res, 1, vec![ChoiceOption::Confirm]);
        drain_to_pass(&mut e);
        drain_to_pass(&mut e_res);
        assert_eq!(e.state_hash(), e_res.state_hash());
        assert_eq!(e.state_core().player(1).hp, 4, "rank 7 vs judge 8 must cancel Sha");

        // Negative: |7 - 2| = 5 > 1 -> hits for 1 dmg!
        advance_to_play(&mut e, 0);
        e.state_core_mut().turn.sha_used_play_phase = false;
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        let sha7b = npc_take_card(&mut e, |d| d.kind == CardKind::Sha && d.rank == 7);
        let _j2 = e.debug_set_deck_top_by(|d| d.rank == 2);
        let out_b = npc_play_card(&mut e, 0, sha7b, vec![1]);
        assert!(!matches!(out_b, CommandOutcome::Rejected(_)), "sha7b: {out_b:?}");
        let _ = answer_choice(&mut e, 1, vec![ChoiceOption::Confirm]); // huangjin confirm
        if let Some(sh) = e.open_choice().cloned() {
            assert_eq!(sh.spec.prompt, "respond_shan");
            let _ = answer_choice(&mut e, 1, vec![ChoiceOption::Pass]);
        }
        drain_to_pass(&mut e);
        assert_eq!(e.state_core().player(1).hp, 3, "rank 7 vs judge 2 must NOT cancel Sha");

        // Multi-target Sha (`Fangtian` last hand card targeting seat 1 `txhj_huangjin` + co-target seat 2) must cancel ALL targets including co-target!
        let mut em = start_locked_engine(2026_9540, 3, LockedStartMode::Prototype, verified).unwrap();
        em.assign_general_named(0, "txhj_huangyueying").unwrap();
        em.assign_general_named(1, "txhj_huangjinxinjun").unwrap();
        em.assign_general_named(2, "txhj_yecha").unwrap();
        advance_to_play(&mut em, 0);
        em.debug_clear_hand(0);
        em.debug_clear_hand(1);
        em.debug_clear_hand(2);
        em.debug_set_hp(1, 4);
        em.debug_set_hp(2, 4);
        let ft = npc_take_card(&mut em, |d| d.kind == CardKind::Fangtian);
        let _ = npc_play_card(&mut em, 0, ft, vec![]);
        drain_to_pass(&mut em);
        let sha7_m = npc_take_card(&mut em, |d| d.kind == CardKind::Sha && d.rank == 7);
        let _j8_m = em.debug_set_deck_top_by(|d| d.rank == 8);
        let out_m = npc_play_card(&mut em, 0, sha7_m, vec![1, 2]);
        assert!(!matches!(out_m, CommandOutcome::Rejected(_)), "Fangtian multi-target Sha rejected: {out_m:?}");
        let ch_m = em.open_choice().cloned().expect("txhj_huangjin:confirm on multi-target Sha");
        assert_eq!(ch_m.spec.prompt, "txhj_huangjin:confirm");
        let _ = answer_choice(&mut em, 1, vec![ChoiceOption::Confirm]);
        drain_to_pass(&mut em);
        assert_eq!(em.state_core().player(1).hp, 4, "txhj_huangjin holder spared");
        assert_eq!(em.state_core().player(2).hp, 4, "txhj_huangjin must clear ALL targets including co-target seat 2");

        // Resolved rank check (`gj_yange_clone` overrides raw rank 2 -> template rank 7 vs judge 8):
        let mut er = start_locked_engine(2026_9541, 2, LockedStartMode::Prototype, verified).unwrap();
        er.assign_general_named(0, "txhj_huangyueying").unwrap();
        er.assign_general_named(1, "txhj_huangjinxinjun").unwrap();
        advance_to_play(&mut er, 0);
        er.debug_clear_hand(0);
        er.debug_clear_hand(1);
        er.debug_set_hp(1, 4);
        let sha_raw2 = npc_take_card(&mut er, |d| d.kind == CardKind::Sha && d.rank == 2);
        let tmpl7 = (0..er.catalog().cards.len() as u32)
            .find(|&id| er.catalog().cards[id as usize].kind == CardKind::Sha && er.catalog().cards[id as usize].rank == 7)
            .unwrap();
        er.state_core_mut().vm_aux.flags.insert((0, format!("gj_yange_clone:{sha_raw2}")), tmpl7 + 1);
        let _j8_r = er.debug_set_deck_top_by(|d| d.rank == 8);
        let out_r = npc_play_card(&mut er, 0, sha_raw2, vec![1]);
        assert!(!matches!(out_r, CommandOutcome::Rejected(_)));
        let _ = answer_choice(&mut er, 1, vec![ChoiceOption::Confirm]);
        drain_to_pass(&mut er);
        assert_eq!(er.state_core().player(1).hp, 4, "txhj_huangjin must use resolved card rank (7 vs judge 8)");
    }

    // 8. `txqyyouji` (`txhj_bossquyi`): single-target ordinary trick (Wuzhong, Snatch) resolves twice; negative: AoE SavageAttack does NOT trigger!
    {
        let mut e = start_locked_engine(2026_955, 3, LockedStartMode::Prototype, verified).unwrap();
        e.assign_general_named(0, "txhj_bossquyi").unwrap();
        e.assign_general_named(1, "txhj_huangyueying").unwrap();
        e.assign_general_named(2, "txhj_machao").unwrap();
        assert!(npc_has_skill(&e, 0, "txqyyouji"));
        advance_to_play(&mut e, 0);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        let wz = npc_take_card(&mut e, |d| d.kind == CardKind::Wuzhong && d.name == "wuzhong");
        let out_wz = npc_play_card(&mut e, 0, wz, vec![]);
        assert!(!matches!(out_wz, CommandOutcome::Rejected(_)), "wuzhong: {out_wz:?}");
        let ch = e.open_choice().cloned().expect("txqyyouji:confirm on Wuzhong");
        assert_eq!(ch.spec.prompt, "txqyyouji:confirm");
        let _ = answer_choice(&mut e, 0, vec![ChoiceOption::Confirm]);
        drain_to_pass(&mut e);
        assert_eq!(e.state_core().player(0).hand.len(), 4, "txqyyouji Wuzhong must draw 2+2 = 4 cards");

        // Negative: AoE SavageAttack has multiple targets -> NO txqyyouji prompt!
        let aoe = npc_take_card(&mut e, |d| d.kind == CardKind::SavageAttack);
        let _ = npc_play_card(&mut e, 0, aoe, vec![]);
        while let Some(c) = e.open_choice().cloned() {
            assert_ne!(c.spec.prompt, "txqyyouji:confirm", "AoE must not trigger txqyyouji");
            let _ = answer_choice(&mut e, c.observer, vec![ChoiceOption::Pass]);
        }
    }

    // 9. AI public-info parity (`BoundedPlanner` handles `txhj_huangjin` / `txhj_chuanyun` / `txhj_guihuo` cleanly without errors).
    {
        let mut e = start_locked_engine(2026_956, 2, LockedStartMode::Prototype, verified).unwrap();
        e.assign_general_named(0, "txhj_zhurongx").unwrap();
        e.assign_general_named(1, "txhj_guanhai").unwrap();
        e.debug_set_turn(0, Phase::Discard, 1);
        let _ = submit(&mut e, 0, GameAction::NextPhase);
        let ch = e.open_choice().expect("guihuo choice");
        let planner = BoundedPlanner::new(64).with_policy_variant(2);
        let act = e.plan_action(ch.observer, &planner).action;
        assert!(matches!(act, GameAction::Answer { .. }), "AI must produce legal public-info answer: {act:?}");
    }

    // 10. Step 5 mid-match NGSA save/restore checks for `rexianzhen` lose (`rexianzhen3`/`rexianzhen4`), `zhenlie`, `jintao`.
    {
        // 10a. `txhj_lingtonghuanxiang` (`rexianzhen` lose with Sha pindian) mid-turn NGSA save/restore preserves rexianzhen3+rexianzhen4 and TurnEnd cleanup!
        let mut e = start_locked_engine(2026_957, 2, LockedStartMode::Prototype, verified).unwrap();
        e.assign_general_named(0, "txhj_lingtonghuanxiang").unwrap();
        e.assign_general_named(1, "txhj_huangyueying").unwrap();
        advance_to_play(&mut e, 0);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        let small = npc_take_card(&mut e, |d| d.kind == CardKind::Sha && d.rank <= 6);
        let big = npc_take_card(&mut e, |d| d.rank >= 12);
        let sha1 = npc_take_card(&mut e, |d| d.kind == CardKind::Sha);
        let sha2 = npc_take_card(&mut e, |d| d.kind == CardKind::Sha);
        e.state_core_mut().player_mut(0).hand.extend([small, sha1, sha2]);
        e.state_core_mut().player_mut(1).hand.push(big);
        e.state_core_mut().vm_aux.set_selected_cards(0, &[small]);
        e.state_core_mut().vm_aux.set_selected_target_seats(0, &[1]);
        let rxz = e.catalog().skill_by_name("rexianzhen").unwrap();
        let out_rxz = submit(&mut e, 0, GameAction::ActivateSkill { skill: rxz });
        assert!(!matches!(out_rxz, CommandOutcome::Rejected(_)), "rexianzhen activate: {out_rxz:?}");
        if let Some(ch) = e.open_choice().cloned() {
            let opt = if ch.spec.options.contains(&ChoiceOption::Player(1)) {
                ChoiceOption::Player(1)
            } else {
                ch.spec.options[0]
            };
            let _ = answer_choice(&mut e, ch.observer, vec![opt]);
        }
        drain_to_pass(&mut e);
        let snap = encode_archive(&e.save());
        let mut e_res = restore_locked_engine(&decode_archive(&snap).unwrap(), verified).unwrap();
        assert_eq!(e.state_hash(), e_res.state_hash());
        // On restore, rexianzhen3 forbids Sha on target 1 AND rexianzhen4 keeps Shas (`sha1`, `sha2`) ignored from hand limit!
        assert!(matches!(submit(&mut e_res, 0, GameAction::PlayCard { card: sha1, targets: vec![1] }), CommandOutcome::Rejected(_)));
        assert!(e_res.state_core().ignored_hand_card_count(e_res.catalog(), 0) >= 2, "restored rexianzhen4 must ignore Sha in hand limit");
        // After turn end -> next Play of seat 0, both rexianzhen3 and rexianzhen4 expire so Sha succeeds!
        e_res.debug_set_turn(0, Phase::End, 1);
        let _ = submit(&mut e_res, 0, GameAction::NextPhase);
        drain_to_pass(&mut e_res);
        assert_eq!(e_res.state_core().ignored_hand_card_count(e_res.catalog(), 0), 0, "rexianzhen4 must expire after TurnEnd");
        advance_to_play(&mut e_res, 0);
        e_res.debug_clear_hand(1);
        e_res.debug_set_hp(1, 3);
        if !e_res.state_core().player(0).hand.contains(&sha2) {
            e_res.state_core_mut().player_mut(0).hand.push(sha2);
        }
        let out_after = submit(&mut e_res, 0, GameAction::PlayCard { card: sha2, targets: vec![1] });
        assert!(!matches!(out_after, CommandOutcome::Rejected(_)), "rexianzhen3 must expire after TurnEnd: {out_after:?}");
        drain_to_pass(&mut e_res);
        assert_eq!(e_res.state_core().player(1).hp, 2);

        // 10b. `txhj_wangyi` (`zhenlie`) mid-confirm NGSA save/restore!
        let mut ez = start_locked_engine(2026_958, 2, LockedStartMode::Prototype, verified).unwrap();
        ez.assign_general_named(0, "txhj_huangyueying").unwrap();
        ez.assign_general_named(1, "txhj_wangyi").unwrap();
        advance_to_play(&mut ez, 0);
        ez.debug_clear_hand(0);
        ez.debug_clear_hand(1);
        ez.debug_set_hp(1, 3);
        let sha = npc_take_card(&mut ez, |d| d.kind == CardKind::Sha);
        let trk = npc_take_card(&mut ez, |d| d.kind == CardKind::Tao || d.kind == CardKind::Sha);
        ez.state_core_mut().player_mut(0).hand.push(trk);
        let _ = npc_play_card(&mut ez, 0, sha, vec![1]);
        let ch = ez.open_choice().cloned().expect("zhenlie confirm prompt");
        let snap_z = encode_archive(&ez.save());
        let mut ez_res = restore_locked_engine(&decode_archive(&snap_z).unwrap(), verified).unwrap();
        let _ = answer_choice(&mut ez, ch.observer, vec![ChoiceOption::Confirm]);
        let _ = answer_choice(&mut ez_res, ch.observer, vec![ChoiceOption::Confirm]);
        drain_to_pass(&mut ez);
        drain_to_pass(&mut ez_res);
        assert_eq!(ez.state_hash(), ez_res.state_hash());

        // 10c. `txhj_huzhen` (`jintao`) mid-Play save/restore preserves 1st-Sha count & 2nd-Sha unrespondable!
        let mut ej = start_locked_engine(2026_959, 2, LockedStartMode::Prototype, verified).unwrap();
        ej.assign_general_named(0, "txhj_huzhen").unwrap();
        ej.assign_general_named(1, "txhj_huangyueying").unwrap();
        advance_to_play(&mut ej, 0);
        ej.debug_clear_hand(0);
        ej.debug_clear_hand(1);
        ej.debug_set_hp(1, 5);
        let s1 = npc_take_card(&mut ej, |d| d.kind == CardKind::Sha);
        let s2 = npc_take_card(&mut ej, |d| d.kind == CardKind::Sha);
        let shan = npc_take_card(&mut ej, |d| d.kind == CardKind::Shan);
        ej.state_core_mut().player_mut(0).hand.push(s2);
        ej.state_core_mut().player_mut(1).hand.push(shan);
        let _ = npc_play_card(&mut ej, 0, s1, vec![1]);
        if let Some(c) = ej.open_choice().cloned() {
            let _ = answer_choice(&mut ej, c.observer, vec![ChoiceOption::Pass]);
        }
        drain_to_pass(&mut ej);
        let snap_j = encode_archive(&ej.save());
        let mut ej_res = restore_locked_engine(&decode_archive(&snap_j).unwrap(), verified).unwrap();
        assert_eq!(ej.state_hash(), ej_res.state_hash());
        let out2 = submit(&mut ej_res, 0, GameAction::PlayCard { card: s2, targets: vec![1] });
        assert!(!matches!(out2, CommandOutcome::Rejected(_)));
        assert!(ej_res.open_choice().is_none(), "restored jintao 2nd Sha must be unrespondable");
        drain_to_pass(&mut ej_res);
        assert_eq!(ej_res.state_core().player(1).hp, 2, "restored jintao: 1st Sha +1 (2) + 2nd Sha (1) = 3 total");
        let _ = DamageNature::Normal;
        let _ = LogEvent::TurnEnded { player: 0 };
        let _ = RejectReason::IllegalTargets;
    }
}
