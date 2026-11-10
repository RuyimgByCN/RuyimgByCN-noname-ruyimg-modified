//! Batch 25 (Decision 34 Complete Closure): end-to-end integration and save/restore
//! verification for per-source remaining sub-skills, sub-generals, 14 HUD marks, and
//! standalone campaign state machines across `L29`, `L10`, `U06`, `U07`, `U08`, `L20`,
//! `L28`, `U02/L03`, `L11`, `L19`, and `L21`.

use content_native_loader::{
    bridge_pack, decision34_closure_extension_pack, longxue_full_unlock_report,
    verify_u08_standard_skill_bindings, DECISION34_STORAGE_DISPATCH_KEY, DIAOBINGQIANJIANG_ID,
    DUOQI_ID, HUANLE_KUWU_INTERACTIVE_ID, K_DANGGU_ID, K_MOWANG_ID, L20_MARK_DISPATCH_ID,
    L20_MARK_SPECS, L28_THUNDER_EXTRA_SKILLS, L28_THUNDER_GENERALS, MBCMFANGZHU_ID,
    MBCMJIUSHI_ID, MBJUEJIN_ID, TH_DUNSHI_ID, TH_JINGXIE_ID, TH_QIAOSI_ID, TH_SKILL_DISPATCH_ID,
    TH_TAIPINGYAOSHU_ID, TH_YEYAN_ID, YOUZHAN_ID, YXS_MENSHEN2_GUARD_ID,
};
use engine_native::event::DamageNature;
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand};
use rule_api::{EffectRequest, SkillId};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(2_500_000);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn repo_extensions_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

fn read_pack_lf(rel: &str) -> String {
    std::fs::read_to_string(repo_extensions_root().join(rel))
        .expect("read package cards.json")
        .replace("\r\n", "\n")
}

fn find_general(e: &Engine, name: &str) -> u16 {
    (0..e.catalog().general_count() as u16)
        .find(|&idx| e.catalog().general_meta(idx).is_some_and(|m| m.0 == name))
        .unwrap_or_else(|| panic!("general {name} not found in catalog"))
}

fn invoke_d34_op(e: &mut Engine, owner: u8, skill_name: &str, skill_id: u32) {
    let out = e.debug_commit_effect(
        skill_name,
        owner,
        EffectRequest::StorageSet {
            skill: SkillId::new(skill_id),
            key: DECISION34_STORAGE_DISPATCH_KEY,
            value: skill_id,
        },
    );
    assert!(
        !matches!(out, CommandOutcome::Rejected(_)),
        "Decision 34 effect dispatch ({skill_name}) rejected: {out:?}"
    );
}

fn flag(e: &Engine, p: u8, key: &str) -> u32 {
    e.state_core()
        .vm_aux
        .flags
        .get(&(p, key.to_string()))
        .copied()
        .unwrap_or(0)
}

fn set_flag(e: &mut Engine, p: u8, key: &str, val: u32) {
    if val == 0 {
        e.state_core_mut().vm_aux.flags.remove(&(p, key.to_string()));
    } else {
        e.state_core_mut()
            .vm_aux
            .flags
            .insert((p, key.to_string()), val);
    }
}

fn drain(e: &mut Engine) {
    for _ in 0..32 {
        let Some(c) = e.open_choice().cloned() else { break };
        let has_pass = c.spec.options.contains(&engine_native::ChoiceOption::Pass);
        let sel: Vec<engine_native::ChoiceOption> = if has_pass {
            vec![engine_native::ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
        };
        let _ = e.submit(GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::Answer {
                choice_id: c.id,
                selection: sel,
            },
        });
    }
}

fn next_phase(e: &mut Engine) {
    let actor = e.turn().current;
    let out = e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(e);
}

#[test]
fn l29_longxue_mbqianlong_unlocks_and_executes_all_four_skills_with_save_restore() {
    let longxue_json = read_pack_lf("longxue-upstream-liuli/assets/cards.json");
    let longxue_pack = bridge_pack(&longxue_json).expect("longxue pack").pack;
    let d34_pack = decision34_closure_extension_pack();
    let mut e = Engine::new_with_extensions(2026110901, 4, &[&longxue_pack, &d34_pack])
        .expect("new_with_extensions");
    let caomao = find_general(&e, "mb_caomao");
    e.debug_assemble_general_def(0, caomao);
    e.debug_assemble_general_def(1, caomao); // Another Wei general with mbweitong -> opening marks = 60

    next_phase(&mut e);
    assert!(
        e.state_core().player(0).mbqianlong_marks >= 60,
        "opening marks with mbweitong ally must be >= 60"
    );
    let qz = e.catalog().skill_by_name("mbcmqingzheng").expect("mbcmqingzheng");
    let js = e.catalog().skill_by_name("mbcmjiushi").expect("mbcmjiushi");
    let fz = e.catalog().skill_by_name("mbcmfangzhu").expect("mbcmfangzhu");
    let jj = e.catalog().skill_by_name("mbjuejin").expect("mbjuejin");

    assert!(e.state_core().skills.contains_key(&(0, qz)), "25 unlocked mbcmqingzheng");
    assert!(e.state_core().skills.contains_key(&(0, js)), "50 unlocked mbcmjiushi");

    // Deal 3 damage from seat 0 to seat 2 (+45 marks -> reaches 99 cap, unlocking fangzhu & juejin)
    let _ = e.debug_deal_damage(Some(0), 2, 3, DamageNature::Normal);
    drain(&mut e);
    assert_eq!(e.state_core().player(0).mbqianlong_marks, 99);
    assert!(e.state_core().skills.contains_key(&(0, fz)), "75 unlocked mbcmfangzhu");
    assert!(e.state_core().skills.contains_key(&(0, jj)), "99 unlocked mbjuejin");
    assert_eq!(
        longxue_full_unlock_report(99),
        "marks:99|unlocked:mbcmqingzheng,mbcmjiushi,mbcmfangzhu,mbjuejin"
    );

    // 1. Execute mbcmjiushi: face-up -> turns over & sets jiu_active; face-down -> flips back & gains trick
    assert!(!e.state_core().player(0).face_down);
    invoke_d34_op(&mut e, 0, "mbcmjiushi", MBCMJIUSHI_ID);
    assert!(e.state_core().player(0).face_down);
    assert!(e.state_core().turn.jiu_active);
    assert_eq!(flag(&e, 0, "mbcmjiushi_jiu_used"), 1);

    invoke_d34_op(&mut e, 0, "mbcmjiushi", MBCMJIUSHI_ID);
    assert!(!e.state_core().player(0).face_down);
    assert_eq!(flag(&e, 0, "mbcmjiushi_trick_gained"), 1);

    // 2. Execute mbcmfangzhu via REAL RuleVm Damage trigger: dealing 1 damage to seat 0 (marks == 99 >= 75)
    // automatically triggers mbcmfangzhu#d34 in RuleVm, flipping target 2 and drawing lost_hp cards!
    e.debug_set_hp(0, 2); // taking 1 damage brings hp to 1 (max_hp = 3 => lost_hp = 2)
    set_flag(&mut e, 0, "mbcmfangzhu_target", 2);
    let hand2_before = e.debug_hand(2).len();
    let _ = e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    drain(&mut e);
    assert!(
        e.state_core().player(2).face_down,
        "mbcmfangzhu must trigger via RuleVm on Damage to mb_caomao"
    );
    assert_eq!(flag(&e, 0, "mbcmfangzhu_drawn"), 2);
    assert_eq!(e.debug_hand(2).len(), hand2_before + 2);

    // 3. Execute mbjuejin: limited 1/game -> +2 hujia & moves shan/tao/jiu into "xiang" pile
    e.debug_place(1, "shan");
    invoke_d34_op(&mut e, 0, "mbjuejin", MBJUEJIN_ID);
    assert_eq!(flag(&e, 0, "mbjuejin_active"), 1);
    assert_eq!(e.state_core().player(0).hujia, 2);
    assert!(flag(&e, 0, "mbjuejin_xiang_count") >= 1);

    // Save & restore + binary archive roundtrip verification
    let snap = e.save();
    let bytes = engine_native::archive::encode_archive(&snap);
    let decoded = engine_native::archive::decode_archive(&bytes).expect("decode_archive");
    let restored = Engine::restore_with_extensions(&decoded, &[&longxue_pack, &d34_pack])
        .expect("restore_with_extensions");
    assert_eq!(restored.state_core().player(0).mbqianlong_marks, 99);
    assert_eq!(restored.state_core().player(0).hujia, 2);
    assert_eq!(flag(&restored, 0, "mbjuejin_active"), 1);
}

#[test]
fn l10_shichangshi_k_danggu_and_k_mowang_roster_lifecycle_and_save_restore() {
    let epic_json = read_pack_lf("epic-cards-upstream-liuli/assets/cards.json");
    let epic_pack = bridge_pack(&epic_json).expect("epic pack").pack;
    let d34_pack = decision34_closure_extension_pack();
    let mut e = Engine::new_with_extensions(2026110902, 2, &[&epic_pack, &d34_pack])
        .expect("new_with_extensions");
    let scs = find_general(&e, "shichangshi");
    e.debug_assemble_general_def(0, scs);
    assert_eq!(e.state_core().player(0).max_hp, 1);

    // Real RuleVm TurnBegin trigger: advancing from GameStart into seat 0's TurnBegin
    // automatically triggers k_danggu#d34 in RuleVm!
    next_phase(&mut e);
    assert_eq!(flag(&e, 0, "k_danggu_roster_mask"), 0x3FF);
    assert_eq!(flag(&e, 0, "k_danggu_active_eunuch"), 1);
    assert_eq!(flag(&e, 0, "k_danggu_triggered"), 1);

    // Real RuleVm Dying trigger: dealing 1 fatal damage to shichangshi (seat 0, 1 HP -> 0 HP)
    // emits EventData::Dying, which automatically triggers k_mowang#d34 in RuleVm,
    // retiring eunuch #1, reviving player 0 to 1 HP, clearing dying, and rotating to eunuch #2!
    let _ = e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    drain(&mut e);
    assert_eq!(flag(&e, 0, "k_danggu_roster_mask"), 0x3FE);
    assert_eq!(flag(&e, 0, "k_danggu_active_eunuch"), 2);
    assert_eq!(flag(&e, 0, "k_mowang_revived"), 1);
    assert_eq!(e.state_core().player(0).hp, 1);
    assert!(!e.state_core().player(0).dying);

    let snap = e.save();
    let bytes = engine_native::archive::encode_archive(&snap);
    let decoded = engine_native::archive::decode_archive(&bytes).expect("decode_archive");
    let restored = Engine::restore_with_extensions(&decoded, &[&epic_pack, &d34_pack])
        .expect("restore_with_extensions");
    assert_eq!(flag(&restored, 0, "k_danggu_roster_mask"), 0x3FE);
    assert_eq!(flag(&restored, 0, "k_danggu_active_eunuch"), 2);
}

#[test]
fn u06_kuwu_interactive_and_u07_wudian_diaobingqianjiang_duoqi_youzhan() {
    let wudian_json = read_pack_lf("wudian-upstream-noname/assets/cards.json");
    let wudian_pack = bridge_pack(&wudian_json).expect("wudian pack").pack;
    let d34_pack = decision34_closure_extension_pack();
    let mut e = Engine::new_with_extensions(2026110903, 4, &[&wudian_pack, &d34_pack])
        .expect("new_with_extensions");

    // 1. U06 kuwu interactive: chain guard -> skipped
    set_flag(&mut e, 0, "kuwu_from_chain", 1);
    invoke_d34_op(&mut e, 0, "huanle_kuwu_interactive", HUANLE_KUWU_INTERACTIVE_ID);
    assert_eq!(flag(&e, 0, "kuwu_skipped_from_chain"), 1);
    set_flag(&mut e, 0, "kuwu_from_chain", 0);

    // U06 kuwu interactive: discard 1 hand card branch
    let hand1_before = e.debug_hand(1).len();
    invoke_d34_op(&mut e, 0, "huanle_kuwu_interactive", HUANLE_KUWU_INTERACTIVE_ID);
    drain(&mut e);
    assert_eq!(e.debug_hand(1).len(), hand1_before - 1);
    assert!(flag(&e, 1, "kuwu_discarded_he") > 0);

    // U06 kuwu interactive: choose lose 1 HP branch
    let hp1_before = e.debug_hp(1);
    set_flag(&mut e, 1, "kuwu_choose_lose_hp", 1);
    invoke_d34_op(&mut e, 0, "huanle_kuwu_interactive", HUANLE_KUWU_INTERACTIVE_ID);
    drain(&mut e);
    assert_eq!(flag(&e, 1, "kuwu_lost_hp"), 1);
    assert_eq!(e.debug_hp(1), hp1_before - 1);
    set_flag(&mut e, 1, "kuwu_choose_lose_hp", 0);

    // 2. U07 `diaobingqianjiang` (`玩点论杀/card/card.js:4-142`):
    //    `contentBefore` draws 1 card (`card.js:15`), reveals `ceil(alive / 2)` market cards (`card.js:41`),
    //    swaps across seats (`card.js:87-94`), and returns chosen market card(s) to deck top (`card.js:130`).
    invoke_d34_op(&mut e, 0, "diaobingqianjiang", DIAOBINGQIANJIANG_ID);
    assert_eq!(flag(&e, 0, "diaobingqianjiang_resolved"), 1);
    assert!(flag(&e, 0, "diaobingqianjiang_swapped") >= 1);
    assert!(flag(&e, 0, "diaobingqianjiang_returned_top") >= 1);

    // 3. U07 `duoqi` (`玩点论杀/character/skill.js:109-159`):
    //    - Blocked on owner's own turn (`skill.js:112`: `_status.currentPhase === player`)
    //    - Blocked when `zhucheng` is empty (`skill.js:115`)
    //    - Off-turn with `zhucheng` card: discards 1 `zhucheng` card and sets `skip_play_phase = true` (`skill.js:151`), 0 draw!
    assert_eq!(e.turn().current, 0);
    invoke_d34_op(&mut e, 0, "duoqi", DUOQI_ID);
    assert_eq!(flag(&e, 0, "duoqi_blocked_self_turn"), 1);
    assert_eq!(flag(&e, 0, "duoqi_triggered"), 0);

    invoke_d34_op(&mut e, 1, "duoqi", DUOQI_ID);
    assert_eq!(flag(&e, 1, "duoqi_blocked_no_zhucheng"), 1);

    let zc_card = e.state_core_mut().deck.pop().expect("deck card for zhucheng");
    e.state_core_mut().player_mut(1).zhucheng.push(zc_card);
    let hand1_pre_duoqi = e.debug_hand(1).len();
    invoke_d34_op(&mut e, 1, "duoqi", DUOQI_ID);
    drain(&mut e);
    assert_eq!(flag(&e, 1, "duoqi_triggered"), 1);
    assert!(e.state_core().player(1).zhucheng.is_empty());
    assert!(e.state_core().discard.contains(&zc_card));
    assert!(e.state_core().turn.skip_play_phase);
    assert_eq!(
        e.debug_hand(1).len(),
        hand1_pre_duoqi,
        "duoqi (`玩点论杀/character/skill.js:109-159`) must not draw cards"
    );
    e.state_core_mut().turn.skip_play_phase = false;

    // 4. U07 `youzhan` (`玩点论杀/character/skill.js:422-447` & `card/card.js:169-202`):
    //    - Blocked when owner has no equip card in `he`
    //    - With equip card in `he`: discards equip card, cancels incoming Sha (`youdishenru_nullified_sha = 1`),
    //      and when attacker refuses/lacks Sha (`youdishenru_refuse_sha = 1`), deals 1 counter-damage with 0 self-draw!
    let mut moved_to_disc = Vec::new();
    let mut kept_hand = Vec::new();
    for c in e.debug_hand(0) {
        if e.catalog().card_kind(c).is_equip() {
            moved_to_disc.push(c);
        } else {
            kept_hand.push(c);
        }
    }
    moved_to_disc.extend(e.state_core().player(0).equips.iter().copied());
    e.state_core_mut().player_mut(0).hand = kept_hand;
    e.state_core_mut().player_mut(0).equips.clear();
    e.state_core_mut().discard.extend(moved_to_disc);
    invoke_d34_op(&mut e, 0, "youzhan", YOUZHAN_ID);
    assert_eq!(flag(&e, 0, "youzhan_blocked_no_equip"), 1);

    let equip_cid = e
        .state_core()
        .deck
        .iter()
        .chain(e.state_core().discard.iter())
        .copied()
        .find(|&c| e.catalog().card_kind(c).is_equip())
        .expect("equip card in deck/discard");
    e.state_core_mut().deck.retain(|&c| c != equip_cid);
    e.state_core_mut().discard.retain(|&c| c != equip_cid);
    e.state_core_mut().player_mut(0).hand.push(equip_cid);
    set_flag(&mut e, 1, "youdishenru_refuse_sha", 1);
    let hand0_pre_youzhan = e.debug_hand(0).len();
    let hp1_pre_youzhan = e.debug_hp(1);
    invoke_d34_op(&mut e, 0, "youzhan", YOUZHAN_ID);
    drain(&mut e);
    assert_eq!(flag(&e, 0, "youzhan_triggered"), 1);
    assert_eq!(flag(&e, 0, "youdishenru_nullified_sha"), 1);
    assert_eq!(e.debug_hp(1), hp1_pre_youzhan - 1);
    assert_eq!(
        e.debug_hand(0).len(),
        hand0_pre_youzhan - 1,
        "youzhan discards 1 equip card without drawing cards"
    );

    // Save/restore + binary archive card conservation verification
    let snap = e.save();
    let bytes = engine_native::archive::encode_archive(&snap);
    let decoded = engine_native::archive::decode_archive(&bytes).expect("decode_archive");
    let restored = Engine::restore_with_extensions(&decoded, &[&wudian_pack, &d34_pack])
        .expect("restore_with_extensions");
    assert_eq!(flag(&restored, 0, "diaobingqianjiang_resolved"), 1);
    assert_eq!(flag(&restored, 1, "duoqi_triggered"), 1);
    assert_eq!(flag(&restored, 0, "youzhan_triggered"), 1);
}

#[test]
fn u08_hero_kill_31_standard_skills_and_menshen2_guard() {
    let yxs_json = read_pack_lf("hero-kill-upstream-noname/assets/cards.json");
    let yxs_pack = bridge_pack(&yxs_json).expect("yxs pack").pack;
    let d34_pack = decision34_closure_extension_pack();
    let mut e = Engine::new_with_extensions(2026110904, 4, &[&yxs_pack, &d34_pack])
        .expect("new_with_extensions");

    // Verify all 31 cross-package standard skills referenced by U08 exist in Catalog
    let count = verify_u08_standard_skill_bindings(e.catalog())
        .expect("all 31 U08 standard skills must resolve in Catalog");
    assert_eq!(count, 31);

    // Fabricated yxs_tafang_wave must not exist in Catalog
    assert!(e.catalog().skill_by_name("yxs_tafang_wave").is_none());

    // yxs_menshen2 guard (`英雄杀/extension.js:2307`) transfers 1 damage from protected seat 1 to granter seat 0
    let menshen2_def = e.catalog().skill_by_name("yxs_menshen2").expect("yxs_menshen2");
    e.state_core_mut()
        .vm_aux
        .marks
        .insert((1, u32::from(menshen2_def)), 0);
    let hp0_before = e.debug_hp(0);
    invoke_d34_op(&mut e, 1, "yxs_menshen2_guard", YXS_MENSHEN2_GUARD_ID);
    drain(&mut e);
    assert_eq!(flag(&e, 1, "yxs_menshen2_guarded"), 1);
    assert_eq!(flag(&e, 1, "yxs_menshen2_transferred_to"), 0);
    assert_eq!(e.debug_hp(0), hp0_before - 1);
}

#[test]
fn l20_fourteen_hud_marks_and_l28_thunder_fourteen_authentic_generals_and_skills() {
    let d34_pack = decision34_closure_extension_pack();
    let mut e = Engine::new_with_extensions(2026110905, 4, &[&d34_pack])
        .expect("new_with_extensions");

    // 1. L20: Dispatch all 14 HUD marks (`标记补充/extension.js:116-129`) and verify persistence + niepan HP recovery
    e.debug_set_hp(0, 1);
    for (idx, &(mark_name, _zh, mark_id)) in L20_MARK_SPECS.iter().enumerate() {
        set_flag(&mut e, 0, "l20_mark_index", idx as u32);
        set_flag(&mut e, 0, "l20_mark_delta", 2);
        invoke_d34_op(&mut e, 0, "l20_mark_dispatch", L20_MARK_DISPATCH_ID);
        assert_eq!(
            e.state_core().vm_aux.marks.get(&(0, mark_id)).copied(),
            Some(2),
            "L20 mark {mark_name} must be recorded in vm_aux.marks"
        );
        assert_eq!(flag(&e, 0, &format!("l20_mark:{mark_name}")), 2);
    }
    assert_eq!(e.debug_hp(0), 3, "niepan mark dispatch recovers HP up to 3");

    // 2. L28: Verify all 14 authentic th_* generals (`雷霆万钧/extension.js:3657-3670`) exist in Catalog,
    // and all 11 previously fabricated th_* generals + 3 fabricated skills are absent.
    assert_eq!(L28_THUNDER_GENERALS.len(), 14);
    for &(g_name, ..) in &L28_THUNDER_GENERALS {
        let _ = find_general(&e, g_name);
    }
    for fabricated_general in [
        "th_sunshangxiang",
        "th_diaochan",
        "th_zhenji",
        "th_xiaoqiao",
        "th_daqiao",
        "th_caiwenji",
        "th_zhugeliang",
        "th_simayi",
        "th_luxun",
        "th_lvbu",
        "th_zhaoyun",
    ] {
        assert!(
            (0..e.catalog().general_count() as u16)
                .all(|idx| e.catalog().general_meta(idx).map(|m| m.0) != Some(fabricated_general)),
            "fabricated general {fabricated_general} must not exist in Catalog"
        );
    }
    for fabricated_skill in ["th_chongzou", "th_huanyin", "th_xuanlv"] {
        assert!(
            e.catalog().skill_by_name(fabricated_skill).is_none(),
            "fabricated skill {fabricated_skill} must not exist in Catalog"
        );
    }
    for &sk in &L28_THUNDER_EXTRA_SKILLS {
        assert!(
            e.catalog().skill_by_name(sk).is_some(),
            "authentic L28 skill {sk} (`雷霆万钧/extension.js`) must exist in Catalog"
        );
    }

    // 3. Test authentic L28 skills (including direct dispatch by `TH_DUNSHI_ID` when `executing` is empty):
    // - `th_dunshi` (`雷霆万钧/extension.js:3783`): prevents damage, loses 1 max_hp, unlocks virtue & draws
    let th_gn = find_general(&e, "th_guanning");
    e.debug_assemble_general_def(0, th_gn);
    let max_hp0_before = e.state_core().player(0).max_hp;
    set_flag(&mut e, 0, "th_dunshi_basic_bit", 1);
    set_flag(&mut e, 0, "th_dunshi_prevent_damage", 1);
    invoke_d34_op(&mut e, 0, "", TH_DUNSHI_ID);
    drain(&mut e);
    assert_eq!(flag(&e, 0, "th_dunshi_damage_prevented"), 1);
    assert_eq!(e.state_core().player(0).max_hp, max_hp0_before - 1);
    assert_eq!(flag(&e, 0, "th_dunshi_virtue_unlocked"), 1);

    // - `th_jingxie` (`雷霆万钧/extension.js:7429`): dying save restores HP to 1 and draws 1 card
    e.debug_set_hp(0, 0);
    e.state_core_mut().player_mut(0).dying = true;
    invoke_d34_op(&mut e, 0, "th_jingxie", TH_JINGXIE_ID);
    drain(&mut e);
    assert_eq!(flag(&e, 0, "th_jingxie_saved_to_1hp"), 1);
    assert_eq!(e.debug_hp(0), 1);
    assert!(!e.state_core().player(0).dying);

    // - `th_qiaosi` (`雷霆万钧/extension.js:7030`): water-wheel wins 1..=3 cards and gives/discards
    set_flag(&mut e, 0, "th_qiaosi_won_count", 2);
    set_flag(&mut e, 0, "th_qiaosi_give_other", 1);
    invoke_d34_op(&mut e, 0, "th_qiaosi", TH_QIAOSI_ID);
    drain(&mut e);
    assert_eq!(flag(&e, 0, "th_qiaosi_drawn"), 2);
    assert!(flag(&e, 0, "th_qiaosi_given") >= 1);

    // - `th_yeyan` (`雷霆万钧/extension.js:8636`): limited 1/game fire damage
    let hp1_before = e.debug_hp(1);
    invoke_d34_op(&mut e, 0, "th_yeyan", TH_YEYAN_ID);
    drain(&mut e);
    assert_eq!(flag(&e, 0, "th_yeyan_used"), 1);
    assert_eq!(e.debug_hp(1), hp1_before - 1);

    // - `th_taipingyaoshu` (`雷霆万钧/extension.js:6974, 8968`): prevents elemental damage; on lose draws 2 & loses 1 HP if hp > 1
    invoke_d34_op(&mut e, 0, "th_taipingyaoshu", TH_TAIPINGYAOSHU_ID);
    assert_eq!(flag(&e, 0, "th_taipingyaoshu_prevent_elemental"), 1);
    e.debug_set_hp(0, 3);
    let hand0_before = e.debug_hand(0).len();
    set_flag(&mut e, 0, "th_taipingyaoshu_on_lose", 1);
    invoke_d34_op(&mut e, 0, "th_taipingyaoshu", TH_TAIPINGYAOSHU_ID);
    drain(&mut e);
    assert_eq!(flag(&e, 0, "th_taipingyaoshu_lost_effect"), 1);
    assert_eq!(e.debug_hand(0).len(), hand0_before + 2);
    assert_eq!(e.debug_hp(0), 2);
}

#[test]
fn fabricated_campaign_step_skills_absent_and_l28_extra_skills_roundtrip() {
    let d34_pack = decision34_closure_extension_pack();
    let mut e = Engine::new_with_extensions(2026110906, 4, &[&d34_pack])
        .expect("new_with_extensions");

    // 1. Verify fabricated campaign step skills were removed per Decision 34 Step 2
    for removed in [
        "boss_baonu_stage",
        "taixu_expedition_step",
        "doomsday_infection_step",
        "martial_formation_step",
        "yxs_tafang_wave",
    ] {
        assert!(
            e.catalog().skill_by_name(removed).is_none(),
            "fabricated step skill {removed} must be absent from Catalog"
        );
    }

    // 2. Test authentic L28 extra skills (`th_tianming` at `extension.js:3689`, `th_qinyin` at `extension.js:8608`)
    invoke_d34_op(&mut e, 0, "th_tianming", TH_SKILL_DISPATCH_ID);
    drain(&mut e);
    assert_eq!(flag(&e, 0, "th_skill:th_tianming"), 1);

    let hp1_before = e.debug_hp(1);
    invoke_d34_op(&mut e, 0, "th_qinyin", TH_SKILL_DISPATCH_ID);
    drain(&mut e);
    assert_eq!(flag(&e, 0, "th_skill:th_qinyin"), 1);
    assert_eq!(e.debug_hp(1), hp1_before - 1);

    // Save & restore roundtrip
    let snap = e.save();
    let restored = Engine::restore_with_extensions(&snap, &[&d34_pack])
        .expect("restore_with_extensions");
    assert_eq!(flag(&restored, 0, "th_skill:th_tianming"), 1);
    assert_eq!(flag(&restored, 0, "th_skill:th_qinyin"), 1);
}

#[test]
fn decision34_edge_cases_and_boundary_guards() {
    let d34_pack = decision34_closure_extension_pack();
    let mut e = Engine::new_with_extensions(2026110907, 4, &[&d34_pack])
        .expect("new_with_extensions");

    // 0. Daoxin threshold lock guards: calling mbcmjiushi (< 50), mbcmfangzhu (< 75), or mbjuejin (< 99)
    // when un-unlocked must early-return with mbqianlong_locked_guard == 1 and zero state mutation!
    assert_eq!(e.state_core().player(0).mbqianlong_marks, 0);
    invoke_d34_op(&mut e, 0, "mbcmjiushi", MBCMJIUSHI_ID);
    assert_eq!(flag(&e, 0, "mbqianlong_locked_guard"), 1);
    assert!(!e.state_core().player(0).face_down);

    set_flag(&mut e, 0, "mbqianlong_locked_guard", 0);
    invoke_d34_op(&mut e, 0, "mbcmfangzhu", MBCMFANGZHU_ID);
    assert_eq!(flag(&e, 0, "mbqianlong_locked_guard"), 1);
    assert!(!e.state_core().player(1).face_down);

    set_flag(&mut e, 0, "mbqianlong_locked_guard", 0);
    invoke_d34_op(&mut e, 0, "mbjuejin", MBJUEJIN_ID);
    assert_eq!(flag(&e, 0, "mbqianlong_locked_guard"), 1);
    assert_eq!(e.state_core().player(0).hujia, 0);

    // 1. mbjuejin limited 1/game idempotency once unlocked (mbqianlong_marks == 99):
    // first call grants +2 hujia; second call does not add extra hujia
    e.state_core_mut().player_mut(0).mbqianlong_marks = 99;
    invoke_d34_op(&mut e, 0, "mbjuejin", MBJUEJIN_ID);
    assert_eq!(e.state_core().player(0).hujia, 2);
    invoke_d34_op(&mut e, 0, "mbjuejin", MBJUEJIN_ID);
    assert_eq!(
        e.state_core().player(0).hujia,
        2,
        "mbjuejin is limited 1/game and must not grant hujia twice"
    );

    // 2. k_mowang exhaustion when last eunuch bit is retired, and post-exhaustion guard
    set_flag(&mut e, 0, "k_danggu_roster_mask", 0b0001);
    set_flag(&mut e, 0, "k_danggu_active_eunuch", 1);
    invoke_d34_op(&mut e, 0, "k_mowang", K_MOWANG_ID);
    assert_eq!(flag(&e, 0, "k_danggu_roster_mask"), 0);
    assert_eq!(flag(&e, 0, "k_mowang_exhausted"), 1);
    invoke_d34_op(&mut e, 0, "k_danggu", K_DANGGU_ID);
    assert_eq!(
        flag(&e, 0, "k_danggu_roster_mask"),
        0,
        "exhausted shichangshi roster must not re-initialize to 0x3FF"
    );

    // 3. yxs_menshen2_guard when unprotected (neither mark nor flag) or when granter is dead -> does not transfer damage
    let hp0_before = e.debug_hp(0);
    invoke_d34_op(&mut e, 1, "yxs_menshen2_guard", YXS_MENSHEN2_GUARD_ID);
    assert_eq!(
        flag(&e, 1, "yxs_menshen2_guarded"),
        0,
        "unprotected seat 1 must not default menshen2_granter to seat 0"
    );
    assert_eq!(e.debug_hp(0), hp0_before);

    set_flag(&mut e, 1, "yxs_menshen2_granter", 2);
    e.state_core_mut().player_mut(2).alive = false;
    invoke_d34_op(&mut e, 1, "yxs_menshen2_guard", YXS_MENSHEN2_GUARD_ID);
    assert_eq!(
        flag(&e, 1, "yxs_menshen2_guarded"),
        0,
        "dead granter cannot absorb menshen2 damage"
    );

    // 4. th_yeyan limited 1/game guard & th_dunshi removed basic card guard (`雷霆万钧/extension.js:3783, 8636`)
    let hp3_before = e.debug_hp(3);
    invoke_d34_op(&mut e, 0, "th_yeyan", TH_YEYAN_ID);
    drain(&mut e);
    let hp1_after_first = e.debug_hp(1);
    invoke_d34_op(&mut e, 0, "th_yeyan", TH_YEYAN_ID);
    drain(&mut e);
    assert_eq!(
        e.debug_hp(1),
        hp1_after_first,
        "th_yeyan is limited 1/game and must not deal fire damage a second time"
    );
    let _ = hp3_before;
}
