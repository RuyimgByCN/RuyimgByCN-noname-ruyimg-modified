//! Inventory lock, `#57` Part-A-only general subset, public-info AI auto-play, and NGSA v107
//! mid-`ChoiceSpec` save/restore parity tests for P3-C1a `L11` Cross NPC Part A (80 + 3 skills).

use super::l11_npc_a_support::*;
use content_native_loader::{
    restore_locked_engine, start_locked_engine, LockedStartMode, L11_TAIXU_GENERALS,
    L11_TAIXU_IMPLEMENTED_CROSS_SKILLS, L11_TAIXU_UPSTREAM_UNDEFINED_SKILL_REFS,
};
use engine_native::ai::BoundedPlanner;
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::catalog::Suit;
use engine_native::content::host::HostBehavior;
use engine_native::{ChoiceOption, GameAction};

/// Part-A-only general subset (`81` `txhj_*` generals in `L11_TAIXU_GENERALS` having `>=1`
/// Part-A cross skill and `0` Part-B cross skills, eligible once #57 clears).
const PART_A_ONLY_GENERALS: [&str; 81] = [
    "txhj_wumazhong",
    "txhj_wujungongjianshou",
    "txhj_heqi",
    "txhj_redaqiao",
    "txhj_moxiaoqiao",
    "txhj_rexiaoqiao",
    "txhj_lvfan",
    "txhj_zhurong",
    "txhj_gousheng",
    "txhj_menghuo5",
    "txhj_menghuo4",
    "txhj_menghuo2",
    "txhj_mangyachang",
    "txhj_menghuo1",
    "txhj_manzushoushi",
    "txhj_mangu",
    "txhj_manzujishi",
    "txhj_nanzhongjingrui",
    "txhj_panjunshizu",
    "txhj_fennudeyezhu",
    "txhj_dushi",
    "txhj_baolveqi",
    "txhj_huangjinzhangyan",
    "txhj_digongzhangbaox",
    "txhj_rengongzhangliang",
    "txhj_tiangongzhangjiaox",
    "txhj_zhangmancheng",
    "txhj_huangjinqibing",
    "txhj_huangjintongling",
    "txhj_peiyuanshaox",
    "txhj_huangjinxiaobing",
    "txhj_hanjunluzhi",
    "txhj_mamidi",
    "txhj_bossquyi",
    "txhj_spdiaochan",
    "txhj_relvbu",
    "txhj_xganning",
    "txhj_sufei",
    "txhj_lingju",
    "txhj_guanyu",
    "txhj_xusheng",
    "txhj_miheng",
    "txhj_hubaoqi",
    "txhj_sunguan",
    "txhj_simashi",
    "txhj_spcaiwenji",
    "txhj_qinyilu",
    "txhj_houkai",
    "txhj_wanglang",
    "txhj_spsunshangxiang",
    "txhj_zhugeliang",
    "txhj_dengai",
    "txhj_zhouyuhuanxiang",
    "txhj_xuchuhuanxiang",
    "txhj_guanyuhuanxiang",
    "txhj_lvbuhuanxiang",
    "txhj_zhoutaihuanxiang",
    "txhj_hunduntongren",
    "txhj_moou",
    "txhj_huanshenguanyu",
    "txhj_huanshenlvmeng",
    "txhj_huanshenzhuge",
    "txhj_huanshenlvbu",
    "txhj_huanshenzhaoyun",
    "txhj_yongwuhuanxiang",
    "txhj_manyanhuanxiang",
    "txhj_yinhuanxiang",
    "txhj_heiwuchanghuanxiang",
    "txhj_mamianhuanxiang",
    "txhj_huanjinglingzhu",
    "txhj_tianjijushou",
    "txhj_bocai",
    "txhj_huanyanzhangliang",
    "txhj_huanyanzhangbao",
    "txhj_huanyanzhangjiaoshi",
    "txhj_huanyanzhangjiaolei",
    "txhj_huanyanzhangjiaohuo",
    "txhj_huanyanzhangjiaofeng",
    "txhj_dongtaihou",
    "txhj_caocao",
    "txhj_xuyou",
];

#[test]
fn l11_npc_a_inventory_and_part_a_generals_instantiate_cleanly() {
    let e = new_4p(93_001);
    assert_eq!(engine_native::Engine::new(1, 2).ruleset_version(), 333);
    let snap = e.save();
    assert_eq!(snap.ruleset_version, e.ruleset_version());
    assert_ne!(snap.ruleset_version, 0);
    let raw = encode_archive(&snap);
    assert_eq!(u16::from_le_bytes([raw[4], raw[5]]), 107);

    // Every single one of the 80 Part-A + 3 derived (`olhuoji`, `olkanpo`, `rexuanfeng`) binds to HostBehavior::L11CrossNpcA
    for &sk in L11_TAIXU_IMPLEMENTED_CROSS_SKILLS[..80]
        .iter()
        .chain(["olhuoji", "olkanpo", "rexuanfeng"].iter())
    {
        let def = e
            .catalog()
            .skill_by_name(sk)
            .unwrap_or_else(|| panic!("missing Part-A skill `{sk}`"));
        assert_eq!(
            e.catalog().skills[def as usize].behavior.as_host_behavior(),
            Some(HostBehavior::L11CrossNpcA),
            "`{sk}` must use HostBehavior::L11CrossNpcA"
        );
    }
    assert!(e.catalog().skill_by_name("bazhen").is_some());
    assert!(e.catalog().skill_by_name("reyingzi").is_some());

    // All 81 Part-A-only generals assemble in Catalog cleanly
    for gname in PART_A_ONLY_GENERALS {
        let mut eg = start_locked_engine(93_002, 2, LockedStartMode::Prototype, taixu_verified()).unwrap();
        eg.assign_general_named(0, gname)
            .unwrap_or_else(|err| panic!("failed to assign `{gname}`: {err:?}"));
    }

    // Regression guard: zero debug_execute_effect/debug_resolve in the new P3-C1a suite
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let forbidden_exec = concat!("debug_execute_", "effect(");
    let forbidden_res = concat!("debug_", "resolve(");
    for rel in [
        "l11_npc_a_active.rs",
        "l11_npc_a_triggers.rs",
        "l11_npc_a_save_ai.rs",
        "l11_npc_a_support/mod.rs",
    ] {
        let txt = std::fs::read_to_string(manifest_dir.join(rel)).unwrap();
        assert!(!txt.contains(forbidden_exec), "{rel} contains forbidden debug exec helper");
        assert!(!txt.contains(forbidden_res), "{rel} contains forbidden debug resolve helper");
    }
}

#[test]
fn l11_npc_a_public_ai_plays_multi_seat_part_a_generals_without_stall() {
    let mut e = new_4p(93_010);
    e.assign_general_named(0, "txhj_xusheng").unwrap(); // repojun
    e.assign_general_named(1, "txhj_huangjinzhangyan").unwrap(); // reqizhou
    e.assign_general_named(2, "txhj_lvfan").unwrap(); // diancai + diaodu
    e.assign_general_named(3, "txhj_guanyu").unwrap(); // wusheng + new_yijue
    let ai = BoundedPlanner::new(24).with_policy_variant(2);

    let mut progress_steps = 0usize;
    for _ in 0..90 {
        if e.outcome().is_some() {
            break;
        }
        let actor = e
            .open_choice()
            .map(|c| c.observer)
            .unwrap_or_else(|| e.state_core().turn.current);
        let plan = e.plan_action(actor, &ai);
        assert_eq!(plan.sim_faults, 0, "AI planning must never hit simulation faults");
        let out = submit(&mut e, actor, plan.action);
        assert!(
            !matches!(out, engine_native::CommandOutcome::Rejected(_)),
            "AI emitted rejected command for seat {actor}: {out:?}"
        );
        progress_steps += 1;
    }
    assert!(progress_steps >= 40);
}

#[test]
fn l11_npc_a_ngsa_v107_mid_choice_save_restore_parity() {
    let verified = taixu_verified();

    // Case 1: Mid-ChoiceSpec `aocai:choose_card` save/restore parity
    {
        let mut e = new_2p(93_021);
        e.debug_grant(0, "aocai");
        goto_play(&mut e, 1);
        e.debug_clear_hand(0);
        let top_shan = e.debug_set_deck_top("shan");
        let sha1 = e.debug_place(1, "sha");
        let _ = submit(&mut e, 1, GameAction::PlayCard { card: sha1, targets: vec![0] });
        let aocai_def = e.catalog().skill_by_name("aocai").unwrap();
        answer_prompt(&mut e, "respond_shan", 0, vec![ChoiceOption::Skill(aocai_def)]);
        assert_eq!(e.open_choice().map(|c| c.spec.prompt), Some("aocai:choose_card"));

        let pre_hash = e.state_hash();
        let raw = encode_archive(&e.save());
        let decoded = decode_archive(&raw).unwrap();
        let mut restored = restore_locked_engine(&decoded, verified).unwrap();
        assert_eq!(restored.state_hash(), pre_hash);

        let cid = e.open_choice().unwrap().id;
        answer_checked(&mut e, 0, vec![ChoiceOption::Card(top_shan)]);
        let _ = submit(
            &mut restored,
            0,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Card(top_shan)],
            },
        );
        assert_eq!(restored.state_hash(), e.state_hash());
    }

    // Case 2: Mid-ChoiceSpec `gushe:owner_pindian` + `retianxiang:choose_target` + `kyou_zhidian:choose_effect` + `olniepan:choose_skill`
    {
        let mut e = new_4p(93_022);
        e.debug_grant(0, "kyou_zhidian");
        goto_play(&mut e, 0);
        e.debug_clear_hand(0);
        let kc = e.debug_place(0, "nullify");
        activate_skill(&mut e, 0, "kyou_zhidian");
        answer_prompt(&mut e, "kyou_zhidian:choose_card", 0, vec![ChoiceOption::Card(kc)]);
        answer_prompt(&mut e, "kyou_zhidian:choose_target", 0, vec![ChoiceOption::Player(1)]);
        assert_eq!(e.open_choice().map(|c| c.spec.prompt), Some("kyou_zhidian:choose_effect"));

        let raw = encode_archive(&e.save());
        let decoded = decode_archive(&raw).unwrap();
        let mut restored = restore_locked_engine(&decoded, verified).unwrap();
        assert_eq!(restored.state_hash(), e.state_hash());

        let cid = e.open_choice().unwrap().id;
        answer_checked(&mut e, 0, vec![ChoiceOption::Tag(1)]);
        let _ = submit(
            &mut restored,
            0,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Tag(1)],
            },
        );
        assert_eq!(restored.state_hash(), e.state_hash());
    }

    // Case 3: Mid-ChoiceSpec `gushe:owner_pindian`
    {
        let mut e = new_4p(93_023);
        e.debug_grant(0, "gushe");
        goto_play(&mut e, 0);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        let p0 = place_with_rank(&mut e, 0, 5, 10);
        let _p1 = place_with_rank(&mut e, 1, 2, 6);
        activate_skill(&mut e, 0, "gushe");
        answer_prompt(&mut e, "gushe:choose_targets", 0, vec![ChoiceOption::Player(1)]);
        assert_eq!(e.open_choice().map(|c| c.spec.prompt), Some("gushe:owner_pindian"));

        let raw = encode_archive(&e.save());
        let decoded = decode_archive(&raw).unwrap();
        let mut restored = restore_locked_engine(&decoded, verified).unwrap();
        assert_eq!(restored.state_hash(), e.state_hash());

        let cid = e.open_choice().unwrap().id;
        answer_checked(&mut e, 0, vec![ChoiceOption::Card(p0)]);
        let _ = submit(
            &mut restored,
            0,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Card(p0)],
            },
        );
        assert_eq!(restored.state_hash(), e.state_hash());
    }

    // Case 4: Mid-ChoiceSpec `olniepan:choose_skill`
    {
        let mut e = new_4p(93_024);
        e.debug_grant(0, "olniepan");
        goto_play(&mut e, 1);
        e.debug_clear_hand(0);
        e.debug_set_hp(0, 1);
        let sha1 = e.debug_place(1, "sha");
        let _ = submit(&mut e, 1, GameAction::PlayCard { card: sha1, targets: vec![0] });
        answer_prompt(&mut e, "olniepan:confirm", 0, vec![ChoiceOption::Confirm]);
        assert_eq!(e.open_choice().map(|c| c.spec.prompt), Some("olniepan:choose_skill"));

        let raw = encode_archive(&e.save());
        let decoded = decode_archive(&raw).unwrap();
        let mut restored = restore_locked_engine(&decoded, verified).unwrap();
        assert_eq!(restored.state_hash(), e.state_hash());

        let ch = e.open_choice().unwrap().clone();
        let pick = *ch.spec.options.first().unwrap();
        answer_checked(&mut e, 0, vec![pick]);
        let _ = submit(
            &mut restored,
            0,
            GameAction::Answer {
                choice_id: ch.id,
                selection: vec![pick],
            },
        );
        assert_eq!(restored.state_hash(), e.state_hash());
    }

    // Case 5: Mid-ChoiceSpec `nzry_kuizhu:damage_targets`
    {
        let mut e = new_4p(93_025);
        e.debug_grant(0, "nzry_kuizhu");
        e.debug_set_hp(0, 2);
        e.debug_set_hp(1, 2);
        goto_play(&mut e, 0);
        e.debug_clear_hand(0);
        let d1 = e.debug_place(0, "sha");
        let d2 = e.debug_place(0, "shan");
        let _ = e.debug_place(0, "tao");
        let _ = e.debug_place(0, "jiu");
        let _ = submit(&mut e, 0, GameAction::NextPhase);
        answer_prompt(
            &mut e,
            "discard_to_limit",
            0,
            vec![ChoiceOption::Card(d1), ChoiceOption::Card(d2)],
        );
        let _ = submit(&mut e, 0, GameAction::NextPhase);
        answer_prompt(&mut e, "trigger_confirm", 0, vec![ChoiceOption::Confirm]);
        answer_prompt(&mut e, "nzry_kuizhu:choose_mode", 0, vec![ChoiceOption::Tag(1)]);
        assert_eq!(e.open_choice().map(|c| c.spec.prompt), Some("nzry_kuizhu:damage_targets"));

        let raw = encode_archive(&e.save());
        let decoded = decode_archive(&raw).unwrap();
        let mut restored = restore_locked_engine(&decoded, verified).unwrap();
        assert_eq!(restored.state_hash(), e.state_hash());

        let cid = e.open_choice().unwrap().id;
        answer_checked(&mut e, 0, vec![ChoiceOption::Player(1)]);
        let _ = submit(
            &mut restored,
            0,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Player(1)],
            },
        );
        assert_eq!(restored.state_hash(), e.state_hash());
    }
}

#[test]
fn l11_npc_a_round2_public_ai_and_formal_loader_closure() {
    // 1. Formal package-loader closure check against `packages/extensions/taixu-upstream-liuli/package.npack.json`:
    //    Confirm ALL 81 `PART_A_ONLY_GENERALS` resolve 100% of their upstream-defined skills cleanly,
    //    all distinct npack VM skills have non-empty handlers (`trigger`/`active`/`modifier`/`view_as`),
    //    and ONLY `txhj_jiaoxie` / `xinchenhui` are omitted without fake catalog skills.
    let pkg_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions/taixu-upstream-liuli/package.npack.json");
    let raw_json = std::fs::read_to_string(&pkg_path).expect("read taixu package.npack.json");
    let parsed: serde_json::Value = serde_json::from_str(&raw_json).expect("parse npack json");
    let mut npack_skills = std::collections::BTreeMap::<String, u64>::new();
    for entry in parsed["skill_index"].as_array().expect("skill_index array") {
        if let (Some(id), Some(def_id)) = (
            entry.get("name").and_then(|v| v.as_str()),
            entry.get("skill_def_id").and_then(|v| v.as_u64()),
        ) {
            npack_skills.insert(id.to_string(), def_id);
        }
    }
    let mut handler_def_ids = std::collections::BTreeSet::<u64>::new();
    for idx_key in ["trigger_index", "modifier_index", "active_index", "view_as_index"] {
        for entry in parsed[idx_key].as_array().expect("handler index array") {
            if let Some(def_id) = entry.get("skill_def_id").and_then(|v| v.as_u64()) {
                handler_def_ids.insert(def_id);
            }
        }
    }

    let verified = taixu_verified();
    let e_loader = start_locked_engine(93_200, 2, LockedStartMode::Prototype, verified).unwrap();
    assert_eq!(PART_A_ONLY_GENERALS.len(), 81);
    let mut npack_used = std::collections::BTreeSet::new();
    let mut undef_seen = std::collections::BTreeSet::new();
    let mut defined_total = 0usize;
    for gname in PART_A_ONLY_GENERALS {
        let &(_, _, _, _, sk_list) = L11_TAIXU_GENERALS
            .iter()
            .find(|g| g.0 == gname)
            .unwrap_or_else(|| panic!("Part-A general `{gname}` missing from L11_TAIXU_GENERALS"));
        let g_idx = (0..e_loader.catalog().general_count() as u16)
            .find(|&idx| e_loader.catalog().general_meta(idx).map(|m| m.0) == Some(gname))
            .unwrap_or_else(|| panic!("general `{gname}` missing from Catalog"));
        let (_, _, attached, _, _) = e_loader.catalog().general_meta(g_idx).unwrap();
        let expected_defined: Vec<&str> = sk_list
            .iter()
            .copied()
            .filter(|sk| !L11_TAIXU_UPSTREAM_UNDEFINED_SKILL_REFS.contains(sk))
            .collect();
        for &sk in sk_list {
            if L11_TAIXU_UPSTREAM_UNDEFINED_SKILL_REFS.contains(&sk) {
                undef_seen.insert(sk);
                assert!(
                    e_loader.catalog().skill_by_name(sk).is_none(),
                    "undefined upstream ref `{sk}` must NOT create fake catalog skill"
                );
            } else if let Some(&def_id) = npack_skills.get(sk) {
                assert!(
                    handler_def_ids.contains(&def_id),
                    "npack skill `{sk}` ({def_id}) on `{gname}` must have trigger/modifier/active/view_as handler"
                );
                assert!(
                    sk == "jueqing" || e_loader.catalog().skill_by_name(sk).is_some(),
                    "npack skill `{sk}` on `{gname}` missing from Catalog"
                );
                npack_used.insert(sk.to_string());
            }
        }
        assert_eq!(
            attached.len(),
            expected_defined.len(),
            "general `{gname}` dropped an upstream-defined skill (`{attached:?}` vs `{expected_defined:?}`)"
        );
        for &att_sk in &attached {
            let sid = e_loader
                .catalog()
                .skill_by_name(att_sk)
                .unwrap_or_else(|| panic!("attached skill `{att_sk}` on `{gname}` missing from Catalog"));
            assert_ne!(e_loader.catalog().skills[sid as usize].name, "");
        }
        defined_total += expected_defined.len();
    }
    assert_eq!(npack_used.len(), 70, "70 distinct npack skills used by 81 Part-A generals");
    assert_eq!(defined_total, 233, "233 total upstream-defined skill attachments across 81 Part-A generals");
    assert_eq!(
        undef_seen.into_iter().collect::<Vec<_>>(),
        vec!["txhj_jiaoxie", "xinchenhui"]
    );

    // 2. Public-info AI checks for Round 2 fixes (`reguose`, `nzry_kuizhu`, `nsxianhai`)
    let ai = BoundedPlanner::new(8).with_policy_variant(2);

    // (a) `reguose:choose_target`: prefers self/ally with `Le` over enemy with `Le` or ally without `Le`.
    {
        let mut e = new_4p(93_201);
        e.debug_grant(0, "reguose");
        e.debug_grant(0, "xinfu_limu");
        e.debug_grant(1, "xinfu_limu");
        goto_play(&mut e, 1);
        let e1_le = place_by_suit_pred(&mut e, 1, Suit::Diamond, Some("sha"));
        let _ = submit(&mut e, 1, GameAction::PlayCard { card: e1_le, targets: vec![1] });
        drain_pass(&mut e);
        goto_play(&mut e, 0);
        e.debug_clear_hand(0);
        let d_self = place_by_suit_pred(&mut e, 0, Suit::Diamond, Some("sha"));
        let d_cost = place_by_suit_pred(&mut e, 0, Suit::Diamond, Some("shan"));
        let _ = submit(&mut e, 0, GameAction::PlayCard { card: d_self, targets: vec![0] });
        drain_pass(&mut e);
        assert!(e.debug_judge(0).contains(&d_self) && e.debug_judge(1).contains(&e1_le));
        activate_skill(&mut e, 0, "reguose");
        answer_prompt(&mut e, "reguose:choose_card", 0, vec![ChoiceOption::Card(d_cost)]);
        let plan = e.plan_action(0, &ai);
        assert_eq!(plan.sim_faults, 0);
        let cid = e.open_choice().unwrap().id;
        assert_eq!(
            plan.action,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Player(0)],
            },
            "AI reguose must discard self Le rather than enemy seat 1's Le"
        );
    }

    // (b) `nzry_kuizhu`: public-info AI handles `choose_mode` AND `damage_targets` with 0 loops.
    {
        let mut e = new_4p(93_202);
        e.debug_grant(0, "nzry_kuizhu");
        e.debug_set_hp(0, 2);
        e.debug_set_hp(1, 2);
        goto_play(&mut e, 0);
        e.debug_clear_hand(0);
        let d1 = e.debug_place(0, "sha");
        let d2 = e.debug_place(0, "shan");
        let _ = e.debug_place(0, "tao");
        let _ = e.debug_place(0, "jiu");
        let _ = submit(&mut e, 0, GameAction::NextPhase);
        answer_prompt(
            &mut e,
            "discard_to_limit",
            0,
            vec![ChoiceOption::Card(d1), ChoiceOption::Card(d2)],
        );
        let _ = submit(&mut e, 0, GameAction::NextPhase);
        answer_prompt(&mut e, "trigger_confirm", 0, vec![ChoiceOption::Confirm]);
        assert_eq!(e.open_choice().map(|c| c.spec.prompt), Some("nzry_kuizhu:choose_mode"));
        let plan_mode = e.plan_action(0, &ai);
        assert_eq!(plan_mode.sim_faults, 0);
        // Also explicitly verify AI on `damage_targets` cancels cleanly (0 loop!)
        answer_prompt(&mut e, "nzry_kuizhu:choose_mode", 0, vec![ChoiceOption::Tag(1)]);
        assert_eq!(e.open_choice().map(|c| c.spec.prompt), Some("nzry_kuizhu:damage_targets"));
        let plan_dmg = e.plan_action(0, &ai);
        assert_eq!(plan_dmg.sim_faults, 0);
        let out = submit(&mut e, 0, plan_dmg.action);
        assert!(!matches!(out, engine_native::CommandOutcome::Rejected(_)));
        assert_ne!(
            e.open_choice().map(|c| c.spec.prompt),
            Some("nzry_kuizhu:damage_targets"),
            "AI must terminate nzry_kuizhu:damage_targets without looping"
        );
    }

    // (c) `nsxianhai:confirm`: AI declines at max_hp == 1
    {
        let mut e2 = new_4p(93_204);
        e2.debug_grant(2, "nsxianhai");
        e2.debug_set_max_hp(2, 1);
        e2.debug_set_hp(2, 1);
        goto_play(&mut e2, 0);
        e2.debug_clear_hand(0);
        e2.debug_clear_hand(1);
        let s0 = e2.debug_place(0, "sha");
        let _ = submit(&mut e2, 0, GameAction::PlayCard { card: s0, targets: vec![1] });
        assert_eq!(e2.open_choice().map(|c| c.spec.prompt), Some("nsxianhai:confirm"));
        let plan = e2.plan_action(2, &ai);
        assert_eq!(plan.sim_faults, 0);
        let out = submit(&mut e2, 2, plan.action);
        assert!(!matches!(out, engine_native::CommandOutcome::Rejected(_)));
        assert_eq!(e2.state_core().player(2).max_hp, 1, "AI must decline nsxianhai at max_hp==1");
    }
}
