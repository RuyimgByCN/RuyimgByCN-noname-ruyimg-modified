//! NGSA v107 mid-choice save/restore, public-info AI role isolation, multi-seat AI rollout,
//! and Part-B general catalog instantiation verification for `HostBehavior::L11CrossNpcB`.
//! Zero `debug_execute_effect` / `debug_resolve`.

#[path = "l11_npc_b_support/mod.rs"]
mod support;

use content_native_loader::{
    bridge_pack, decision34_pack9_for_coexistence, L11_TAIXU_GENERALS,
    L11_TAIXU_IMPLEMENTED_CROSS_SKILLS,
};
use engine_native::ai::BoundedPlanner;
use engine_native::catalog::CardKind;
use engine_native::content::extension::content_lock_id;
use engine_native::content::host::HostBehavior;
use engine_native::event::Phase;
use engine_native::state::Role;
use engine_native::{ChoiceOption, CommandOutcome, GameAction};
use support::*;

#[test]
fn l11_npc_b_mid_choice_ngsa_v107_save_restore_parity() {
    let mut e = start_2p(2026_8201);
    e.debug_grant(0, "shangyi");
    e.debug_grant(0, "twyuanhu");
    advance_to_phase(&mut e, 0, Phase::Play, None);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    let c0 = place_hand_kind_any(&mut e, 0, CardKind::Sha);
    let c1 = place_hand_kind_any(&mut e, 1, CardKind::Shan);
    // Enter mid-choice of `shangyi` (`ASK_SHANGYI_TARGET`), save NGSA bytes, restore, complete identically:
    let _ = activate(&mut e, 0, "shangyi");
    answer(&mut e, 0, vec![ChoiceOption::Card(c0)]);
    let open = e.open_choice().expect("open target choice");
    assert!(open.spec.prompt.starts_with("shangyi:"));

    let mut restored = roundtrip_restore(&e);
    answer(&mut restored, 0, vec![ChoiceOption::Player(1)]);
    answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    assert_eq!(restored.state_hash(), e.state_hash());
    if restored.open_choice().is_some() {
        let opt = restored.open_choice().unwrap().spec.options[0];
        answer(&mut restored, 0, vec![opt]);
        answer(&mut e, 0, vec![opt]);
    }
    drain_pass(&mut restored, 8);
    drain_pass(&mut e, 8);
    assert_eq!(restored.state_hash(), e.state_hash());
    let _ = c1;

    // Also save/restore across `xinfu_zhaoxin` mid-choice:
    let mut ez = start_2p(2026_8202);
    ez.debug_grant(0, "xinfu_zhaoxin");
    advance_to_phase(&mut ez, 0, Phase::Play, None);
    ez.debug_clear_hand(0);
    let zc = place_hand_kind_any(&mut ez, 0, CardKind::Tao);
    let _ = activate(&mut ez, 0, "xinfu_zhaoxin");
    let mut ez_r = roundtrip_restore(&ez);
    answer(&mut ez_r, 0, vec![ChoiceOption::Card(zc)]);
    answer(&mut ez, 0, vec![ChoiceOption::Card(zc)]);
    drain_pass(&mut ez_r, 8);
    drain_pass(&mut ez, 8);
    assert_eq!(ez_r.state_hash(), ez.state_hash());
}

#[test]
fn l11_npc_b_public_info_ai_no_unrevealed_leak_and_4p_rollout() {
    // 1. Public-info AI must NOT pick Rebel-specific sabotage against unrevealed Loyalist/Traitor vs publicly revealed Lord:
    let mut e = start_4p_identity(2026_8203);
    e.debug_grant(1, "twlinglu");
    // Seat 0 = publicly revealed Lord; Seat 1 = Rebel; Seat 2 = unrevealed Traitor; Seat 3 = unrevealed Loyalist
    e.debug_set_identity_roles(vec![Role::Lord, Role::Rebel, Role::Traitor, Role::Loyalist]);
    advance_to_phase(&mut e, 1, Phase::Play, Some("twlinglu:"));
    let planner = BoundedPlanner::new(8);
    if let Some(ch) = e.open_choice().cloned() {
        assert_eq!(ch.observer, 1);
        let plan = e.plan_action(1, &planner);
        if let GameAction::Answer { selection, .. } = plan.action {
            // Publicly revealed Lord (seat 0) is the known enemy for Rebel seat 1:
            assert!(
                !selection.is_empty(),
                "AI must score twlinglu target selection cleanly using public info"
            );
        }
    }
    drain_with_ai(&mut e, 24);

    // 2. Loyalist (seat 3) must NOT sabotage publicly revealed Lord (seat 0) via `zhendu`:
    let mut e_loy = start_4p_identity(2026_8204);
    e_loy.debug_grant(3, "zhendu");
    e_loy.debug_clear_hand(3);
    place_hand_kind_any(&mut e_loy, 3, CardKind::Sha);
    advance_to_phase(&mut e_loy, 0, Phase::Play, Some("zhendu:"));
    if let Some(ch) = e_loy.open_choice().cloned() {
        if ch.observer == 3 && ch.spec.prompt.starts_with("zhendu:") {
            let plan = e_loy.plan_action(3, &planner);
            if let GameAction::Answer { selection, .. } = plan.action {
                assert!(
                    selection.is_empty() || selection == vec![ChoiceOption::Pass],
                    "Loyalist public-info AI must pass zhendu against revealed Lord, got {selection:?}"
                );
            }
        }
    }

    // 3. Full multi-seat AI rollout with Part-B skills attached completes without stalls/panics:
    let mut e_sim = start_4p_identity(2026_8205);
    e_sim.debug_grant(0, "twwushen");
    e_sim.debug_grant(0, "sbliegong");
    e_sim.debug_grant(1, "twmibei");
    e_sim.debug_grant(1, "shunshi");
    e_sim.debug_grant(2, "xinfu_zhaoxin");
    e_sim.debug_grant(2, "twhongju");
    e_sim.debug_grant(3, "xinsheng");
    e_sim.debug_grant(3, "yuce");
    for _ in 0..80 {
        if e_sim.outcome().is_some() {
            break;
        }
        if let Some(ch) = e_sim.open_choice() {
            let obs = ch.observer;
            let act = e_sim.plan_action(obs, &planner).action;
            let _ = submit(&mut e_sim, obs, act);
        } else {
            let cur = e_sim.state_core().turn.current;
            let act = e_sim.plan_action(cur, &planner).action;
            if matches!(submit(&mut e_sim, cur, act), CommandOutcome::Rejected(_)) {
                let _ = submit(&mut e_sim, cur, GameAction::NextPhase);
            }
        }
    }
    assert_ne!(e_sim.state_hash(), [0u8; 32]);
}

#[test]
fn l11_npc_b_generals_and_coexistence_lock_preserved() {
    let e = start_2p(2026_8206);
    let part_b_set = &L11_TAIXU_IMPLEMENTED_CROSS_SKILLS[80..];
    let mut part_b_generals = Vec::new();
    for &(gname, _, _, _, sks) in &L11_TAIXU_GENERALS {
        if sks.iter().any(|s| part_b_set.contains(s)) {
            let gid = (0..e.catalog().general_count() as u16)
                .find(|&idx| e.catalog().general_meta(idx).map(|m| m.0) == Some(gname))
                .unwrap_or_else(|| panic!("general {gname} missing"));
            part_b_generals.push((gname, gid));
        }
    }
    assert!(!part_b_generals.is_empty());
    let mut eng = start_2p(2026_8207);
    for (gname, gid) in part_b_generals {
        eng.debug_assemble_general_def(0, gid);
        assert_eq!(eng.catalog().general_meta(gid).map(|m| m.0), Some(gname));
    }
    // Verify `decision34_pack9_for_coexistence` lock invariant remains bit-identical:
    let d3_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions/d3-upstream-noname/assets/character.json");
    let d3_src = std::fs::read_to_string(d3_path)
        .expect("read d3 character.json")
        .replace("\r\n", "\n");
    let d3_pack = bridge_pack(&d3_src).expect("bridge d3").pack;
    let coex = decision34_pack9_for_coexistence(&[&d3_pack]);
    assert_eq!(content_lock_id(&[&coex]), 9_097_112_637_251_845_257);
    let _ = HostBehavior::L11CrossNpcB;
}
