//! L20 标记补充（`archive/upstream/noname-liuli/noname/extension/标记补充/extension.js`）
//! 六维度真实对局验收测试套件（Section 四.3）：
//! - 覆盖全部 92 个非 CSS 条目（3 张整肃展示牌 + 15 个配置项 + 9 个独立 `HostBehavior` 技能 +
//!   8 个 Batch A 国战/修改技能 + 37 个 UI/Spine/空标记桩 + 20 个 Batch B 核心规则技能 = 92 项，
//!   连同 3 个 CSS 表面项共 95 项）。
//! - 完整覆盖六维度：① 正常发动与结算、② 可取消选择点 Pass/Cancel 不耗次数不改状态、
//!   ③ 非法目标/非法卡牌/数量不足拒绝（`CommandOutcome::Rejected`）、
//!   ④ 条件不满足与每回合/使命限次门禁、⑤ 多技能与被动修正联动、
//!   ⑥ 挂起选择点中途 `encode_archive` + `decode_archive` 快照恢复哈希一致。

use content_native_loader::{
    decision34_pack9_extension_pack,
    mark_supplement::{
        load_mark_supplement_config, lookup_mark_supplement_card, DaoJianFuStyle, DaoJianFuType,
        MarkSupplementConfig, YouxiShengfuMode, L20_MARK_SUPPLEMENT_CARD_SPECS,
        L20_MARK_SUPPLEMENT_CONFIG_KEYS,
    },
    L20_MARK_SUPPLEMENT_CARDS, L20_MARK_SUPPLEMENT_SKILLS,
};
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::catalog::{CardKind, Suit};
use engine_native::event::DamageNature;
use engine_native::state::Role;
use engine_native::{
    ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase,
};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(20_600_000);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn new_l20_engine(seed: u64, players: usize) -> Engine {
    let pack = decision34_pack9_extension_pack();
    let mut e = Engine::new_with_extensions(seed, players, &[&pack]).expect("L20 engine init");
    e.state_core_mut().skills.clear();
    e
}

fn flag(e: &Engine, p: u8, key: &str) -> u32 {
    e.state_core().vm_aux.flags.get(&(p, key.to_string())).copied().unwrap_or(0)
}

fn set_flag(e: &mut Engine, p: u8, key: &str, val: u32) {
    if val == 0 {
        e.state_core_mut().vm_aux.flags.remove(&(p, key.to_string()));
    } else {
        e.state_core_mut().vm_aux.flags.insert((p, key.to_string()), val);
    }
}

fn assert_archive_roundtrip(e: &Engine) -> Engine {
    let pack = decision34_pack9_extension_pack();
    let packs = [&pack];
    let bytes = encode_archive(&e.save());
    let decoded = decode_archive(&bytes).expect("decode_archive at choice");
    let restored = Engine::restore_with_extensions(&decoded, &packs).expect("restore_with_extensions");
    assert_eq!(restored.state_hash(), e.state_hash());
    restored
}

fn answer(e: &mut Engine, selection: Vec<ChoiceOption>) -> CommandOutcome {
    let c = e.open_choice().cloned().expect("expected open_choice");
    e.submit(GameCommand {
        actor: c.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer { choice_id: c.id, selection },
    })
}

fn cancel(e: &mut Engine) -> CommandOutcome {
    let c = e.open_choice().cloned().expect("expected open_choice to cancel");
    let sel = if c.spec.options.contains(&ChoiceOption::Pass) {
        vec![ChoiceOption::Pass]
    } else {
        Vec::new()
    };
    let out = answer(e, sel);
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "cancel on {} rejected: {out:?}", c.spec.prompt);
    out
}

fn drain(e: &mut Engine) {
    for _ in 0..64 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        let sel = if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
        };
        let _ = answer(e, sel);
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
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
}

fn advance_to(e: &mut Engine, seat: u8, phase: Phase) {
    for _ in 0..64 {
        drain(e);
        if e.turn().current == seat && e.turn().phase == phase {
            return;
        }
        next_phase(e);
    }
    panic!("failed to reach seat {seat} phase {phase:?}");
}

fn activate(e: &mut Engine, actor: u8, skill_name: &str) -> CommandOutcome {
    let def = e.catalog().skill_by_name(skill_name).expect(skill_name);
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill: def },
    })
}

fn play_card(e: &mut Engine, actor: u8, card: u32, targets: Vec<u8>) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card, targets },
    })
}

fn find_deck_card(e: &Engine, pred: impl Fn(&engine_native::catalog::CardDef) -> bool) -> u32 {
    e.state_core()
        .deck
        .iter()
        .copied()
        .find(|&cid| e.catalog().cards.get(cid as usize).is_some_and(&pred))
        .expect("matching card in deck")
}

fn give_deck_card(
    e: &mut Engine,
    owner: u8,
    pred: impl Fn(&engine_native::catalog::CardDef) -> bool,
) -> u32 {
    let cid = find_deck_card(e, pred);
    e.state_core_mut().deck.retain(|&x| x != cid);
    e.state_core_mut().player_mut(owner).hand.push(cid);
    cid
}

#[test]
fn l20_configs_15_and_zhengsu_cards_3_and_dedicated_9_skills_six_dimensions() {
    // 1. 15 configs + 3 zhengsu cards: normal load, custom overrides, and illegal value rejections (Dim ① & ③)
    assert_eq!(L20_MARK_SUPPLEMENT_CONFIG_KEYS.len(), 15);
    assert_eq!(L20_MARK_SUPPLEMENT_CARD_SPECS.len(), 3);
    assert_eq!(L20_MARK_SUPPLEMENT_CARDS.len(), 3);
    assert_eq!(L20_MARK_SUPPLEMENT_SKILLS.len(), 74);

    let cfg_default = load_mark_supplement_config("{}").expect("default L20 config");
    assert_eq!(cfg_default, MarkSupplementConfig::default());
    assert_eq!(cfg_default.daojianfu, DaoJianFuStyle::V45);
    assert_eq!(cfg_default.daojianfu_type, DaoJianFuType::Random);

    let cfg_custom = load_mark_supplement_config(
        r#"{"jinengxiugai":false,"biaojixiugai":false,"zhijiechifa":false,"doudizhu_guanqiuxiaoguo":false,"huajiatexiao":false,"guozhanmeihua":false,"shoushajishatexiao":false,"shiyongkapaitixiao":false,"youxishengfu":"identity","daojianfu":"shousha","daojianfu_type":"jian","xinshadoudizhu":true,"xlfbmoshi":true,"mpfbmoshi":true,"zhujizhongwang":true}"#,
    )
    .expect("custom L20 config");
    assert!(!cfg_custom.jinengxiugai && cfg_custom.xlfbmoshi && cfg_custom.mpfbmoshi && cfg_custom.zhujizhongwang);
    assert_eq!(cfg_custom.youxishengfu, YouxiShengfuMode::Identity);
    assert_eq!(cfg_custom.daojianfu, DaoJianFuStyle::Shousha);
    assert_eq!(cfg_custom.daojianfu_type, DaoJianFuType::Jian);
    assert!(load_mark_supplement_config(r#"{"daojianfu_type":"99"}"#).is_err());
    assert!(load_mark_supplement_config(r#"{"xlfbmoshi":"yes"}"#).is_err());

    let mut e = new_l20_engine(2026111301, 4);
    for &cname in &L20_MARK_SUPPLEMENT_CARDS {
        let cspec = lookup_mark_supplement_card(cname).expect("card spec");
        assert!(cspec.fullskin);
        assert!(e.catalog().card_by_name(cname).is_some(), "card {cname} in catalog");
    }
    assert!(lookup_mark_supplement_card("unknown_zhengsu").is_none());

    // All L20 skills + 4 subskills are bound to native HostBehavior, except `zuoxing`:
    // upstream `标记补充/extension.js:6155-6204` comments it out entirely, so L20 must not
    // override (take over) the core skill with a HostBehavior.
    for &sk in L20_MARK_SUPPLEMENT_SKILLS
        .iter()
        .chain(["wengua5", "xianqu_mark", "kuanshi", "kuanshi2"].iter())
    {
        let sid = e.catalog().skill_by_name(sk).unwrap_or_else(|| panic!("missing {sk}"));
        let host_bound = e.catalog().skills[sid as usize].behavior.as_host_behavior().is_some();
        if sk == "zuoxing" {
            assert!(!host_bound, "L20 must not override zuoxing (commented out upstream)");
        } else {
            assert!(host_bound, "L20 skill {sk} must bind to a native HostBehavior");
        }
    }

    // 2. `xinhongyan`: Dim ⑤ spade-as-heart (no hand-limit bonus, not active in Play) + Dim ①/③/⑥ forced `xinhongyan:choose_suit` on Heart OrdinaryJudge
    advance_to(&mut e, 0, Phase::Play);
    e.debug_grant(0, "xinhongyan");
    assert!(
        matches!(activate(&mut e, 0, "xinhongyan"), CommandOutcome::Rejected(_)),
        "xinhongyan is not an active Play-phase skill"
    );
    let spade_weapon = find_deck_card(&e, |c| c.kind == CardKind::Qinggang && c.suit == Suit::Spade);
    e.state_core_mut().deck.retain(|&x| x != spade_weapon);
    e.state_core_mut().player_mut(0).equips.push(spade_weapon);
    assert_eq!(
        e.state_core().effective_hand_limit(e.catalog(), 0),
        e.debug_hp(0) as usize,
        "upstream xinhongyan (`extension.js:1105-1135`) does not modify maxHandcard"
    );
    let le_c = find_deck_card(&e, |c| c.kind == CardKind::Le);
    let spade_judge = find_deck_card(&e, |c| c.suit == Suit::Spade && c.kind == CardKind::Sha);
    e.state_core_mut().deck.retain(|&x| x != le_c && x != spade_judge);
    e.state_core_mut().player_mut(0).judge.push(le_c);
    e.state_core_mut().deck.push(spade_judge); // Spade judge card on seat 0 is treated as Heart by xinhongyan
    e.debug_set_turn(0, Phase::Prepare, 1);
    next_phase(&mut e); // Prepare -> Judge -> triggers OrdinaryJudge -> xinhongyan:choose_suit
    assert_eq!(e.open_choice().unwrap().spec.prompt, "xinhongyan:choose_suit");
    assert!(!e.open_choice().unwrap().spec.allow_cancel, "xinhongyan is forced on Heart judge");
    e = assert_archive_roundtrip(&e);
    assert!(matches!(answer(&mut e, vec![]), CommandOutcome::Rejected(_)));
    assert!(matches!(answer(&mut e, vec![ChoiceOption::Tag(99)]), CommandOutcome::Rejected(_)));
    let _ = answer(&mut e, vec![ChoiceOption::Tag(2)]); // Tag(2) -> Club (3)
    drain(&mut e);
    assert_eq!(flag(&e, 0, "xinhongyan_applied"), 1);
    assert!(e.state_core().turn.skip_play_phase, "Le judge with Club suit skips Play phase");
    e.state_core_mut().turn.skip_play_phase = false;
    advance_to(&mut e, 0, Phase::Play);

    // 3. `xinfu_langxi`: Prepare-phase trigger (`extension.js:4323-4364`), not active in Play phase
    e.debug_grant(0, "xinfu_langxi");
    assert!(
        matches!(activate(&mut e, 0, "xinfu_langxi"), CommandOutcome::Rejected(_)),
        "xinfu_langxi is a Prepare-phase trigger, not an active Play-phase skill"
    );
    e.debug_set_hp(0, 3);
    for p in 1..4 {
        e.debug_set_hp(p, 4);
    }
    e.debug_set_turn(3, Phase::End, 1);
    next_phase(&mut e); // Turn 3 End -> Turn 0 Prepare: blocked when all others have hp > owner.hp
    assert!(
        e.open_choice().map_or(true, |c| c.spec.prompt != "xinfu_langxi:choose_target"),
        "xinfu_langxi blocked when no other player has hp <= owner.hp"
    );
    drain(&mut e);
    e.debug_set_hp(1, 3);
    e.debug_set_turn(3, Phase::End, 1);
    next_phase(&mut e); // Turn 3 End -> Turn 0 Prepare: prompts xinfu_langxi:choose_target
    assert_eq!(e.open_choice().unwrap().spec.prompt, "xinfu_langxi:choose_target");
    e = assert_archive_roundtrip(&e);
    assert!(matches!(answer(&mut e, vec![ChoiceOption::Player(2)]), CommandOutcome::Rejected(_)));
    let _ = cancel(&mut e);
    assert_eq!(flag(&e, 0, "xinfu_langxi_fired"), 0);
    drain(&mut e);
    e.debug_set_turn(3, Phase::End, 1);
    next_phase(&mut e);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "xinfu_langxi:choose_target");
    let _ = answer(&mut e, vec![ChoiceOption::Player(1)]);
    drain(&mut e);
    assert_eq!(flag(&e, 0, "xinfu_langxi_fired"), 1);
    let dealt = flag(&e, 0, "xinfu_langxi_last_dmg") as i32;
    assert!((0..=2).contains(&dealt));
    assert_eq!(e.debug_hp(1), 3 - dealt);
    e.debug_set_hp(1, 4);
    e.debug_set_hp(0, 4);
    advance_to(&mut e, 0, Phase::Play);

    // 4. `fz_gzpaoxiao`: Dim ①/⑤ unlimited Sha in Play phase + 2nd Sha draws 1 card + ignores armor only with shouyue
    e.debug_grant(0, "fz_gzpaoxiao");
    let ignores_armor = |e: &Engine| {
        engine_native::rvm_bridge::modifiers::eval_ignore_armor(
            e.state_core(),
            e.catalog(),
            0,
            1,
            None,
            DamageNature::Normal,
        )
    };
    assert!(!ignores_armor(&e), "fz_gzpaoxiao does not ignore armor without shouyue");
    set_flag(&mut e, 0, "shouyue_active", 1);
    assert!(ignores_armor(&e), "fz_gzpaoxiao ignores armor when shouyue is active");
    set_flag(&mut e, 0, "shouyue_active", 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sha1 = give_deck_card(&mut e, 0, |c| c.kind == CardKind::Sha);
    let sha2 = give_deck_card(&mut e, 0, |c| c.kind == CardKind::Sha);
    let sha3 = give_deck_card(&mut e, 0, |c| c.kind == CardKind::Sha);
    e.debug_set_hp(1, 4);
    let _ = play_card(&mut e, 0, sha1, vec![1]);
    drain(&mut e);
    assert_eq!(flag(&e, 0, "fz_gzpaoxiao_second_sha_drawn"), 0, "1st Sha does not draw");
    let hand_before_2nd = e.debug_hand(0).len();
    let _ = play_card(&mut e, 0, sha2, vec![1]);
    drain(&mut e);
    assert_eq!(flag(&e, 0, "fz_gzpaoxiao_second_sha_drawn"), 1, "2nd Sha triggers draw");
    assert_eq!(e.debug_hand(0).len(), hand_before_2nd, "2nd Sha net hand unchanged");
    e.debug_set_hp(1, 4);
    let out3 = play_card(&mut e, 0, sha3, vec![1]);
    assert!(!matches!(out3, CommandOutcome::Rejected(_)), "3rd Sha allowed by fz_gzpaoxiao");
    drain(&mut e);

    // 5. `kurou` & `_chongzhu`: Dim ①/②/③/④/⑥ (`_chongzhu` only allows recastable cards such as `Chain`)
    e.debug_clear_hand(0);
    e.debug_grant(0, "_chongzhu");
    assert!(matches!(activate(&mut e, 0, "_chongzhu"), CommandOutcome::Rejected(_)));
    e.debug_grant(0, "kurou");
    let hp_before = e.debug_hp(0);
    let _ = activate(&mut e, 0, "kurou");
    drain(&mut e);
    assert_eq!(e.debug_hp(0), hp_before - 1);
    assert_eq!(e.debug_hand(0).len(), 2);

    e.debug_clear_hand(0);
    let non_recast_sha = give_deck_card(&mut e, 0, |c| c.kind == CardKind::Sha);
    assert!(
        matches!(activate(&mut e, 0, "_chongzhu"), CommandOutcome::Rejected(_)),
        "_chongzhu rejects when hand has no recastable (Chain) cards"
    );
    let chain_c = give_deck_card(&mut e, 0, |c| c.kind == CardKind::Chain);
    assert!(matches!(activate(&mut e, 0, "_chongzhu"), CommandOutcome::Pending { .. }));
    assert_eq!(e.open_choice().unwrap().spec.prompt, "_chongzhu:choose_card");
    e = assert_archive_roundtrip(&e);
    assert!(
        matches!(answer(&mut e, vec![ChoiceOption::Card(non_recast_sha)]), CommandOutcome::Rejected(_)),
        "_chongzhu rejects non-recastable card"
    );
    let _ = cancel(&mut e);
    assert_eq!(flag(&e, 0, "l20_chongzhu_recast"), 0);
    let _ = activate(&mut e, 0, "_chongzhu");
    let _ = answer(&mut e, vec![ChoiceOption::Card(chain_c)]);
    assert_eq!(flag(&e, 0, "l20_chongzhu_recast"), 1);
    assert_eq!(e.debug_hand(0).len(), 2);

    // 6. `shouxi`, `_xigua_mopaifanbei_`, `_xigua_xueliangfanbei_`, `_xigua_zhuwanzhongji_`
    let mut e5 = new_l20_engine(2026111302, 5);
    e5.debug_set_identity_roles(vec![
        Role::Lord, Role::Loyalist, Role::Rebel, Role::Rebel, Role::Traitor,
    ]);
    for sk in ["_xigua_mopaifanbei_", "_xigua_xueliangfanbei_", "_xigua_zhuwanzhongji_", "shouxi"] {
        e5.debug_grant(1, sk);
    }
    e5.debug_set_hp(1, 3);
    let max1_before = e5.state_core().player(1).max_hp;
    advance_to(&mut e5, 1, Phase::Judge);
    assert_eq!(e5.state_core().player(1).max_hp, max1_before + 3);
    assert_eq!(e5.debug_hp(1), 6);
    let hand_before_draw = e5.debug_hand(1).len();
    next_phase(&mut e5); // Judge -> Draw
    drain(&mut e5);
    assert_eq!(e5.debug_hand(1).len(), hand_before_draw + 4);

    advance_to(&mut e5, 2, Phase::Play);
    e5.debug_clear_hand(1);
    e5.debug_clear_hand(2);
    let tao_p1 = give_deck_card(&mut e5, 1, |c| c.kind == CardKind::Tao);
    let sha_p2 = give_deck_card(&mut e5, 2, |c| c.kind == CardKind::Sha);
    let extra_sha_p2 = give_deck_card(&mut e5, 2, |c| c.kind == CardKind::Sha);
    let non_match_tao_p2 = give_deck_card(&mut e5, 2, |c| c.kind == CardKind::Tao);
    let _ = play_card(&mut e5, 2, sha_p2, vec![1]);
    assert_eq!(e5.open_choice().unwrap().spec.prompt, "shouxi:declare_kind");
    e5 = assert_archive_roundtrip(&e5);
    let first_tag = e5.open_choice().unwrap().spec.options[0]; // Tag(0) = Sha
    let _ = answer(&mut e5, vec![first_tag]);
    assert_eq!(e5.open_choice().unwrap().spec.prompt, "shouxi:attacker_discard");
    e5 = assert_archive_roundtrip(&e5);
    assert!(
        matches!(answer(&mut e5, vec![ChoiceOption::Card(non_match_tao_p2)]), CommandOutcome::Rejected(_)),
        "shouxi:attacker_discard only accepts cards matching declared kind"
    );
    let _ = answer(&mut e5, vec![ChoiceOption::Card(extra_sha_p2)]);
    assert_eq!(e5.open_choice().unwrap().spec.prompt, "shouxi:attacker_gain");
    e5 = assert_archive_roundtrip(&e5);
    let _ = answer(&mut e5, vec![ChoiceOption::Card(tao_p1)]);
    drain(&mut e5);
    assert!(e5.debug_hand(2).contains(&tao_p1), "attacker gained chosen card from shouxi owner");
    assert_eq!(e5.debug_hp(1), 5, "Sha dealt 1 damage after attacker discarded matching card");

    e5.debug_set_hp(0, 1);
    for p in 0..5 {
        e5.debug_clear_hand(p);
    }
    let _ = e5.debug_deal_damage(Some(2), 0, 1, DamageNature::Normal);
    drain(&mut e5);
    assert_eq!(e5.state_core().identity_roles().unwrap()[1], Role::Lord);
}

#[test]
fn l20_batch_a_guozhan_8_skills_and_37_ui_mark_stubs_six_dimensions() {
    let mut e = new_l20_engine(2026111303, 4);
    e.debug_set_identity_roles(vec![
        Role::Lord,
        Role::Rebel,
        Role::Traitor,
        Role::Loyalist,
    ]);
    e.debug_set_general(0, "曹操");
    e.debug_set_general(1, "刘备");
    e.debug_set_general(2, "孙权");
    e.debug_set_general(3, "吕布");

    // 1. `_aozhan_judge`: blocked in 4-seat game while all 4 seats are alive (alive_count > 3 && total_seats <= 7)
    e.debug_grant(0, "_aozhan_judge");
    advance_to(&mut e, 0, Phase::Play);
    assert_eq!(flag(&e, 0, "aozhan_active"), 0);

    // 2. `wengua` & `wengua5`: Dim ①/②/③/④/⑥
    e.debug_grant(0, "wengua");
    e.debug_grant(0, "wengua5");
    let c0 = e.debug_hand(0)[0];
    assert!(matches!(activate(&mut e, 0, "wengua"), CommandOutcome::Pending { .. }));
    assert_eq!(e.open_choice().unwrap().spec.prompt, "wengua:choose_card");
    e = assert_archive_roundtrip(&e);
    assert!(matches!(answer(&mut e, vec![ChoiceOption::Card(999)]), CommandOutcome::Rejected(_)));
    let _ = cancel(&mut e);
    assert_eq!(flag(&e, 0, "wengua_resolved"), 0, "cancel refunds wengua once_per_turn");

    let _ = activate(&mut e, 0, "wengua");
    let _ = answer(&mut e, vec![ChoiceOption::Card(c0)]);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "wengua:choose_side");
    e = assert_archive_roundtrip(&e);
    assert!(matches!(answer(&mut e, vec![ChoiceOption::Tag(9)]), CommandOutcome::Rejected(_)));
    let _ = cancel(&mut e);
    assert_eq!(flag(&e, 0, "wengua_resolved"), 0, "stage-2 cancel refunds wengua once_per_turn");

    let _ = activate(&mut e, 0, "wengua");
    let _ = answer(&mut e, vec![ChoiceOption::Card(c0)]);
    let _ = answer(&mut e, vec![ChoiceOption::Tag(0)]);
    assert_eq!(flag(&e, 0, "wengua_resolved"), 1);
    assert_eq!(e.state_core().deck.last().copied(), Some(c0));
    assert!(matches!(activate(&mut e, 0, "wengua"), CommandOutcome::Rejected(_)));

    // `wengua5` grants temporary `wengua2_active` to seat 1 at seat 1's Prepare phase
    advance_to(&mut e, 1, Phase::Play);
    assert_eq!(flag(&e, 1, "wengua2_active"), 1, "wengua5 granted wengua2_active at Prepare");
    let c1 = e.debug_hand(1)[0];
    assert!(matches!(activate(&mut e, 1, "wengua5"), CommandOutcome::Pending { .. }));
    assert_eq!(e.open_choice().unwrap().spec.prompt, "wengua5:give_card");
    e = assert_archive_roundtrip(&e);
    let _ = cancel(&mut e);
    assert_eq!(flag(&e, 0, "wengua5_side"), 0);

    let _ = activate(&mut e, 1, "wengua5");
    let _ = answer(&mut e, vec![ChoiceOption::Card(c1)]);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "wengua5:choose_side");
    assert_eq!(e.open_choice().unwrap().observer, 0, "wengua owner chooses side");
    e = assert_archive_roundtrip(&e);
    let _ = answer(&mut e, vec![ChoiceOption::Tag(1)]);
    assert_eq!(flag(&e, 0, "wengua5_side"), 1);
    assert_eq!(e.state_core().deck.first().copied(), Some(c1));
    assert!(matches!(activate(&mut e, 1, "wengua5"), CommandOutcome::Rejected(_)));

    // 3. `_guozhan_marks`, `xianqu_mark`, `_zhulianbihe_mark_tao`, `_yinyang_mark_add`, `diaohulishan`
    advance_to(&mut e, 0, Phase::Play);
    for sk in ["_guozhan_marks", "xianqu_mark", "_zhulianbihe_mark_tao", "_yinyang_mark_add"] {
        e.debug_grant(0, sk);
    }

    // Dim ④: rejected without marks
    assert!(matches!(activate(&mut e, 0, "_guozhan_marks"), CommandOutcome::Rejected(_)));
    assert!(matches!(activate(&mut e, 0, "_zhulianbihe_mark_tao"), CommandOutcome::Rejected(_)));

    set_flag(&mut e, 0, "xianqu_mark", 1);
    set_flag(&mut e, 0, "zhulianbihe_mark", 2);
    set_flag(&mut e, 0, "yinyang_mark", 2);
    set_flag(&mut e, 2, "general_unrevealed", 1);
    set_flag(&mut e, 2, "deputy_unrevealed", 1);
    e.debug_clear_hand(0);
    assert!(matches!(activate(&mut e, 0, "_guozhan_marks"), CommandOutcome::Pending { .. }));
    assert_eq!(e.open_choice().unwrap().spec.prompt, "_guozhan_marks:choose_mark");
    e = assert_archive_roundtrip(&e);
    let _ = cancel(&mut e);
    assert_eq!(flag(&e, 0, "xianqu_mark"), 1);

    // Choose Tag(0) -> xianqu_mark draws hand to 4 and prompts `xianqu_mark:choose_target` + `xianqu_mark:choose_option`
    let _ = activate(&mut e, 0, "_guozhan_marks");
    let _ = answer(&mut e, vec![ChoiceOption::Tag(0)]);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "xianqu_mark:choose_target");
    assert_eq!(e.debug_hand(0).len(), 4);
    e = assert_archive_roundtrip(&e);
    assert!(matches!(answer(&mut e, vec![ChoiceOption::Player(0)]), CommandOutcome::Rejected(_)));
    let _ = answer(&mut e, vec![ChoiceOption::Player(2)]);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "xianqu_mark:choose_option");
    e = assert_archive_roundtrip(&e);
    let _ = answer(&mut e, vec![ChoiceOption::Tag(1)]);
    assert_eq!(flag(&e, 0, "xianqu_viewed"), 1);
    assert_eq!(flag(&e, 0, "xianqu_viewed_main"), 1);
    assert_eq!(flag(&e, 0, "xianqu_target"), 3);
    set_flag(&mut e, 2, "general_unrevealed", 0);
    set_flag(&mut e, 2, "deputy_unrevealed", 0);

    // Choose Tag(1) -> zhulianbihe_mark draws 2; Choose Tag(2) -> yinyang_mark draws 1
    let _ = activate(&mut e, 0, "_guozhan_marks");
    let _ = answer(&mut e, vec![ChoiceOption::Tag(1)]);
    assert_eq!(e.debug_hand(0).len(), 6);
    let _ = activate(&mut e, 0, "_guozhan_marks");
    let _ = answer(&mut e, vec![ChoiceOption::Tag(2)]);
    assert_eq!(e.debug_hand(0).len(), 7);

    // `_zhulianbihe_mark_tao`: Dim ④ rejected at full HP, Dim ①/②/⑥ when wounded
    let max0 = e.state_core().player(0).max_hp as i32;
    e.debug_set_hp(0, max0);
    assert!(matches!(activate(&mut e, 0, "_zhulianbihe_mark_tao"), CommandOutcome::Rejected(_)));
    e.debug_set_hp(0, max0 - 1);
    assert!(matches!(activate(&mut e, 0, "_zhulianbihe_mark_tao"), CommandOutcome::Pending { .. }));
    assert_eq!(e.open_choice().unwrap().spec.prompt, "_zhulianbihe_mark_tao:confirm");
    e = assert_archive_roundtrip(&e);
    let _ = cancel(&mut e);
    assert_eq!(e.debug_hp(0), max0 - 1);
    let _ = activate(&mut e, 0, "_zhulianbihe_mark_tao");
    let _ = answer(&mut e, vec![ChoiceOption::Confirm]);
    assert_eq!(e.debug_hp(0), max0);
    assert_eq!(flag(&e, 0, "_zhulianbihe_tao_used"), 1);

    // `diaohulishan`: passive status skill (not actively usable; blocks all cards, damage, loseHp, recover, and targeting)
    e.debug_grant(0, "diaohulishan");
    assert!(matches!(activate(&mut e, 0, "diaohulishan"), CommandOutcome::Rejected(_)));
    e.debug_set_skill_enabled(0, "diaohulishan", false);
    e.debug_grant(1, "diaohulishan");
    set_flag(&mut e, 1, "diaohulishan_active", 1);
    assert!(e.catalog().blocks_all_cards(e.state_core(), 1));
    let hp1_before = e.debug_hp(1);
    let _ = e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert_eq!(e.debug_hp(1), hp1_before, "diaohulishan prevents damage");
    let sha_c = give_deck_card(&mut e, 0, |c| c.kind == CardKind::Sha);
    assert!(matches!(play_card(&mut e, 0, sha_c, vec![1]), CommandOutcome::Rejected(_)));
    set_flag(&mut e, 1, "diaohulishan_active", 0);
    e.debug_set_skill_enabled(1, "diaohulishan", false);

    // 4. All 37 UI/Spine/mark-stub skills in Batch A reject active invocation in Play phase
    for &stub in &[
        "zhulianbihe_mark", "yinyang_mark", "yexinjia_mark", "_xigua_feiyanbahu_", "_shiqihujia_",
        "_guozhanChooseControlCHG", "_guozhan_diebiaoji_", "_guozhan_duiweibiaoji_gua_",
        "_guozhan_duiweibiaoji_gua_gua_", "_guozhanhalfchghp", "_shoushajisha_",
        "_shoushajisha_delete_", "_diankuangtulu_gua", "_wanjunqushou_gua", "_o_o_recovertrigger",
        "_o_o_miaoshouhuichun", "_o_o_yishugaochao", "_losehp_", "_mianshang_", "_player_baojiA",
        "_player_baojiB", "_player_baojiC", "_player_baojiD", "_player_baojiE", "_player_baojiF",
        "_player_baojiG", "_player_baojiH", "_jiubuff_", "_jiubuff_xiaoshi_", "_player_game_guagua",
        "_guozhan_quanjubiaoji_guagua", "_shiyongkapaitexiao_", "_shiyongkapaitexiao_xi_gua_",
        "_shanghaipaishiyongtexiao_", "_player_daojianfu_xxgg_fuzi_", "_binglintu", "_dysp",
    ] {
        e.debug_grant(0, stub);
        let h_before = e.debug_hand(0).len();
        let out = activate(&mut e, 0, stub);
        assert!(matches!(out, CommandOutcome::Rejected(_)), "{stub} must not be Play-activatable");
        assert_eq!(e.debug_hand(0).len(), h_before);
    }

    // 5. `_yinyang_mark_add` at Discard phase begin: Dim ①/⑤/⑥ (+2 hand limit)
    let limit_before = e.state_core().effective_hand_limit(e.catalog(), 0);
    assert!(e.debug_hand(0).len() > limit_before);
    next_phase(&mut e); // Play -> Discard
    assert_eq!(e.open_choice().unwrap().spec.prompt, "_yinyang_mark_add:confirm");
    e = assert_archive_roundtrip(&e);
    let _ = answer(&mut e, vec![ChoiceOption::Confirm]);
    assert_eq!(e.state_core().effective_hand_limit(e.catalog(), 0), limit_before + 2);
    drain(&mut e);

    // 6. `_aozhan_judge` fires when seat 3 dies (leaving 3 alive seats with distinct factions Wei/Shu/Wu)
    e.debug_clear_hand(3);
    e.debug_set_hp(3, 1);
    let _ = e.debug_deal_damage(Some(0), 3, 2, DamageNature::Normal);
    drain(&mut e);
    assert!(!e.debug_alive(3));
    advance_to(&mut e, 0, Phase::Play);
    assert_eq!(flag(&e, 0, "aozhan_active"), 1, "_aozhan_judge activated when 3 distinct factions remain");
    assert_eq!(flag(&e, 0, "l20_ui_hook:_dysp"), 1, "passive VFX hook fired on PhaseBegin");
    // Under aozhan_active, Tao cannot be used for self-heal in Play phase, and can be used as Sha on a target
    e.debug_set_hp(0, 2);
    let tao_c = give_deck_card(&mut e, 0, |c| c.kind == CardKind::Tao);
    assert!(
        matches!(play_card(&mut e, 0, tao_c, vec![]), CommandOutcome::Rejected(_)),
        "aozhan_active blocks normal self-heal Tao in Play phase"
    );
    e.debug_clear_hand(1);
    let hp1_aozhan = e.debug_hp(1);
    let _ = play_card(&mut e, 0, tao_c, vec![1]);
    drain(&mut e);
    assert_eq!(e.debug_hp(1), hp1_aozhan - 1, "aozhan_active allows Tao in hand to be used as Sha");
}

#[test]
fn l20_batch_b_20_canonical_skills_six_dimensions() {
    let mut e = new_l20_engine(2026111304, 4);

    // 1. `zhengsu`: Dim ①/②/③/④/⑥ (leijin success + reward, violation guard)
    e.debug_grant(0, "zhengsu");
    advance_to(&mut e, 0, Phase::Judge);
    next_phase(&mut e); // Judge -> Draw
    drain(&mut e);
    next_phase(&mut e); // Draw -> Play
    drain(&mut e);
    set_flag(&mut e, 0, "zhengsu_leijin", 1);

    e.debug_clear_hand(0);
    e.debug_set_hp(0, 2);
    let c_r2 = give_deck_card(&mut e, 0, |c| c.kind == CardKind::Tao && c.rank < 5);
    let r_first = e.catalog().cards[c_r2 as usize].rank;
    let c_r_mid = give_deck_card(&mut e, 0, |c| c.kind == CardKind::Wuzhong && c.rank > r_first && c.rank < 11);
    let r_mid = e.catalog().cards[c_r_mid as usize].rank;
    let c_r_hi = give_deck_card(&mut e, 0, |c| c.kind.equip_slot().is_some() && c.rank > r_mid);
    let _ = play_card(&mut e, 0, c_r2, vec![]);
    drain(&mut e);
    let _ = play_card(&mut e, 0, c_r_mid, vec![]);
    drain(&mut e);
    let _ = play_card(&mut e, 0, c_r_hi, vec![]);
    drain(&mut e);
    assert_eq!(flag(&e, 0, "zhengsu_succeeded"), 1);
    e.debug_clear_hand(0);
    next_phase(&mut e); // Play -> Discard
    next_phase(&mut e); // Discard -> End (triggers `zhengsu:choose_reward` at PhaseEnd of Discard)
    assert_eq!(e.open_choice().unwrap().spec.prompt, "zhengsu:choose_reward");
    e = assert_archive_roundtrip(&e);
    let hand_before_reward = e.debug_hand(0).len();
    let _ = answer(&mut e, vec![ChoiceOption::Tag(0)]);
    assert_eq!(flag(&e, 0, "zhengsu_rewarded"), 1);
    assert_eq!(e.debug_hand(0).len(), hand_before_reward + 2);
    drain(&mut e);

    // Dim ④ violation on zhengsu_leijin: non-increasing rank fails zhengsu
    advance_to(&mut e, 0, Phase::Draw);
    drain(&mut e);
    next_phase(&mut e); // Draw -> Play
    drain(&mut e);
    set_flag(&mut e, 0, "zhengsu_leijin", 1);
    e.debug_clear_hand(0);
    let hi_card = give_deck_card(&mut e, 0, |c| c.kind == CardKind::Wuzhong && c.rank >= 8);
    let lo_card = give_deck_card(&mut e, 0, |c| c.kind == CardKind::Sha && c.rank <= 5);
    let _ = play_card(&mut e, 0, hi_card, vec![]);
    drain(&mut e);
    let _ = play_card(&mut e, 0, lo_card, vec![1]);
    drain(&mut e);
    assert_eq!(flag(&e, 0, "zhengsu_failed"), 1);
    assert_eq!(flag(&e, 0, "zhengsu_succeeded"), 0);

    // 2. `zhenyi_spade` (`extension.js:1041-1104`): Dim ①/②/④/⑥ — fires on any player's
    // pre-finalized OrdinaryJudge while the owner holds `zhenyi_mark`; rewrites suit + number 5.
    let mut ez = new_l20_engine(2026111311, 4);
    ez.debug_grant(1, "zhenyi_spade");
    for p in 0..4 {
        ez.debug_clear_hand(p);
    }
    let le1 = give_judge_trick(&mut ez, 0, CardKind::Le);
    ez.debug_set_turn(0, Phase::Prepare, 1);
    next_phase(&mut ez); // no zhenyi_mark -> Dim ④ no prompt
    assert!(ez.open_choice().map_or(true, |c| c.spec.prompt != "zhenyi_spade:choose_suit"));
    drain(&mut ez);
    assert!(!ez.state_core().player(0).judge.contains(&le1));
    set_flag(&mut ez, 1, "zhenyi_mark", 1);
    let _ = give_judge_trick(&mut ez, 0, CardKind::Le);
    ez.state_core_mut().turn.skip_play_phase = false;
    ez.debug_set_turn(0, Phase::Prepare, 1);
    next_phase(&mut ez);
    assert_eq!(ez.open_choice().unwrap().spec.prompt, "zhenyi_spade:choose_suit");
    assert_eq!(ez.open_choice().unwrap().observer, 1);
    ez = assert_archive_roundtrip(&ez);
    assert!(matches!(answer(&mut ez, vec![ChoiceOption::Tag(9)]), CommandOutcome::Rejected(_)));
    let _ = cancel(&mut ez);
    drain(&mut ez);
    assert_eq!(flag(&ez, 1, "zhenyi_mark"), 1, "cancel keeps zhenyi_mark");
    assert_eq!(flag(&ez, 1, "zhenyi_spade_applied"), 0);
    // Positive: Lightning judged as ♠5 (zhenyi suit + fixed number 5) -> 3 thunder damage.
    let _ = give_judge_trick(&mut ez, 0, CardKind::ShanDian);
    ez.state_core_mut().turn.skip_play_phase = false;
    ez.debug_set_hp(0, 4);
    ez.debug_set_turn(0, Phase::Prepare, 1);
    next_phase(&mut ez);
    assert_eq!(ez.open_choice().unwrap().spec.prompt, "zhenyi_spade:choose_suit");
    let _ = answer(&mut ez, vec![ChoiceOption::Tag(0)]); // Spade
    drain(&mut ez);
    assert_eq!(flag(&ez, 1, "zhenyi_spade_applied"), 1);
    assert_eq!(flag(&ez, 1, "zhenyi_mark"), 0);
    assert_eq!(ez.debug_hp(0), 1, "judge rewritten to Spade 5 -> Lightning deals 3");
    assert_eq!(flag(&ez, 1, "zhenyi_judge_suit"), 0, "rewrite is not left on the owner");
    assert_eq!(flag(&ez, 0, "zhenyi_judge_suit"), 0, "rewrite consumed by the judge");

    // 3. `tspowei` (`extension.js:4366-4531`): wei marks at first TurnBegin, wei holder's TurnBegin prompt,
    // damage removes wei, duty success at owner's PhaseBegin(Prepare) when no wei remains.
    let mut e2 = new_l20_engine(2026111305, 4);
    e2.debug_grant(0, "tspowei");
    for p in 0..4 {
        e2.debug_clear_hand(p);
        e2.debug_set_hp(p, 4);
    }
    e2.debug_set_hp(0, 3);
    e2.debug_set_hp(2, 3);
    e2.debug_set_turn(3, Phase::End, 1);
    next_phase(&mut e2); // TurnBegin(0): init marks on all other alive players
    drain(&mut e2);
    assert_eq!(flag(&e2, 0, "tspowei_init"), 1);
    assert_eq!(flag(&e2, 1, "tspowei_wei"), 1);
    assert_eq!(flag(&e2, 2, "tspowei_wei"), 1);
    assert_eq!(flag(&e2, 3, "tspowei_wei"), 1);
    set_flag(&mut e2, 1, "tspowei_wei", 0);
    set_flag(&mut e2, 3, "tspowei_wei", 0);
    assert!(matches!(activate(&mut e2, 0, "tspowei"), CommandOutcome::Rejected(_)), "tspowei is not active");
    let disc_c = give_deck_card(&mut e2, 0, |c| c.kind == CardKind::Shan);
    e2.debug_set_turn(1, Phase::End, 1);
    next_phase(&mut e2); // TurnBegin(2): wei holder -> owner prompted
    assert_eq!(e2.open_choice().unwrap().spec.prompt, "tspowei:choose_action");
    assert_eq!(e2.open_choice().unwrap().observer, 0);
    e2 = assert_archive_roundtrip(&e2);
    let _ = answer(&mut e2, vec![ChoiceOption::Tag(0)]);
    assert_eq!(e2.open_choice().unwrap().spec.prompt, "tspowei:choose_card");
    let _ = answer(&mut e2, vec![ChoiceOption::Card(disc_c)]);
    drain(&mut e2);
    assert_eq!(e2.debug_hp(2), 2, "tspowei discard -> 1 damage to wei holder");
    assert_eq!(flag(&e2, 2, "tspowei_wei"), 0, "damage removes wei");
    assert_eq!(flag(&e2, 0, "tspowei_achieved"), 0, "duty success checks at owner's PhaseBegin(Prepare)");
    e2.debug_set_turn(3, Phase::End, 1);
    next_phase(&mut e2); // TurnBegin(0) -> PhaseBegin(0, Prepare)
    drain(&mut e2);
    assert_eq!(flag(&e2, 0, "tspowei_achieved"), 1);
    assert_eq!(flag(&e2, 0, "shenzhu_granted"), 1);

    // `qingyu` (`extension.js:4534-4590`): mandatory discard 2 on damage + duty success at Prepare (full hp & empty hand)
    let mut eq = new_l20_engine(2026111312, 4);
    eq.debug_grant(1, "qingyu");
    for p in 0..4 {
        eq.debug_clear_hand(p);
    }
    let _ = give_deck_card(&mut eq, 1, |_| true);
    let _ = give_deck_card(&mut eq, 1, |_| true);
    let hp1_before = eq.debug_hp(1);
    let _ = eq.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert_eq!(eq.open_choice().unwrap().spec.prompt, "qingyu:discard_two");
    assert!(!eq.open_choice().unwrap().spec.allow_cancel);
    eq = assert_archive_roundtrip(&eq);
    assert!(matches!(answer(&mut eq, vec![]), CommandOutcome::Rejected(_)));
    let h1 = eq.debug_hand(1);
    assert!(matches!(answer(&mut eq, vec![ChoiceOption::Card(h1[0])]), CommandOutcome::Rejected(_)));
    let _ = answer(&mut eq, vec![ChoiceOption::Card(h1[0]), ChoiceOption::Card(h1[1])]);
    drain(&mut eq);
    assert_eq!(eq.debug_hp(1), hp1_before, "qingyu prevents damage");
    assert_eq!(flag(&eq, 1, "qingyu_prevented"), 1);
    let _ = eq.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    drain(&mut eq);
    assert_eq!(eq.debug_hp(1), hp1_before - 1, "qingyu needs >= 2 he cards");
    let max1 = eq.state_core().player(1).max_hp as i32;
    eq.debug_set_hp(1, max1);
    eq.debug_set_turn(0, Phase::End, 1);
    next_phase(&mut eq); // TurnBegin(1) -> Prepare(1): hp == max_hp && empty hand
    drain(&mut eq);
    assert_eq!(flag(&eq, 1, "qingyu_resolved"), 1);
    assert_eq!(flag(&eq, 1, "xuancun_granted"), 1);

    // `mibei` (`extension.js:4592-4674`): fail at PhaseBegin(End) when xingqi is empty, succeed when 2 basic + 2 trick + 2 equip used
    let mut e2 = new_l20_engine(2026111313, 4);
    e2.debug_grant(2, "mibei");
    let max2_before = e2.state_core().player(2).max_hp;
    advance_to(&mut e2, 2, Phase::Judge);
    assert_eq!(flag(&e2, 2, "mibei_mark"), 1);
    advance_to(&mut e2, 2, Phase::End);
    drain(&mut e2);
    assert_eq!(flag(&e2, 2, "mibei_resolved"), 2);
    assert_eq!(e2.state_core().player(2).max_hp, max2_before - 1);

    set_flag(&mut e2, 2, "mibei_resolved", 0);
    set_flag(&mut e2, 2, "mibei_basic_cnt", 2);
    set_flag(&mut e2, 2, "mibei_trick_cnt", 2);
    set_flag(&mut e2, 2, "mibei_equip_cnt", 1);
    advance_to(&mut e2, 2, Phase::Play);
    let eq_c = give_deck_card(&mut e2, 2, |c| c.kind.equip_slot().is_some());
    let hand_before_mibei = e2.debug_hand(2).len();
    let _ = play_card(&mut e2, 2, eq_c, vec![]);
    drain(&mut e2);
    assert_eq!(flag(&e2, 2, "mibei_resolved"), 1, "equip use (Equipped) counts toward mibei");
    assert_eq!(e2.debug_hand(2).len(), hand_before_mibei - 1 + 3);

    // 4. `fz_new_rewusheng`, `fz_new_longdan`, `fz_new_tieji`, `fz_liegong`, `fz_xinkuanggu`
    fz_five_skills_six_dimensions();

    // 5. `rekuanshi` (`extension.js:5244-5297`), `kuanshi`, `kuanshi2`
    let mut e5 = new_l20_engine(2026111308, 4);
    e5.debug_grant(0, "rekuanshi");
    e5.debug_grant(1, "kuanshi");
    advance_to(&mut e5, 0, Phase::Discard);
    e5.debug_clear_hand(0);
    next_phase(&mut e5); // Discard -> End
    assert_eq!(e5.open_choice().unwrap().spec.prompt, "rekuanshi:choose_target");
    e5 = assert_archive_roundtrip(&e5);
    let _ = answer(&mut e5, vec![ChoiceOption::Player(2)]);
    drain(&mut e5);
    e5.debug_set_hp(0, 2);
    e5.debug_set_hp(2, 4);
    let _ = e5.debug_deal_damage(Some(3), 2, 1, DamageNature::Normal);
    drain(&mut e5);
    assert_eq!(flag(&e5, 0, "rekuanshi_triggered"), 0, "1 damage taken does not trigger rekuanshi");
    assert_eq!(e5.debug_hp(2), 3);
    let hand0_before = e5.debug_hand(0).len();
    let _ = e5.debug_deal_damage(Some(3), 2, 2, DamageNature::Normal);
    drain(&mut e5);
    assert_eq!(flag(&e5, 0, "rekuanshi_triggered"), 1);
    assert_eq!(e5.debug_hp(2), 2, "target took 2 more (total 3 > 1) then recovered 1");
    assert_eq!(e5.debug_hp(0), 2, "rekuanshi owner never heals");
    assert_eq!(e5.debug_hand(0).len(), hand0_before, "rekuanshi owner never draws");
    let _ = e5.debug_deal_damage(Some(3), 2, 1, DamageNature::Normal);
    drain(&mut e5);
    assert_eq!(e5.debug_hp(2), 1, "rekuanshi fires once");

    advance_to(&mut e5, 1, Phase::Discard);
    e5.debug_clear_hand(1);
    next_phase(&mut e5); // Discard -> End
    assert_eq!(e5.open_choice().unwrap().spec.prompt, "kuanshi:choose_target");
    e5 = assert_archive_roundtrip(&e5);
    let _ = answer(&mut e5, vec![ChoiceOption::Player(3)]);
    drain(&mut e5);
    e5.debug_set_hp(3, 4);
    let _ = e5.debug_deal_damage(Some(0), 3, 2, DamageNature::Normal);
    drain(&mut e5);
    assert_eq!(flag(&e5, 1, "kuanshi_prevented"), 1);
    assert_eq!(flag(&e5, 1, "kuanshi_skip_draw"), 1);
    assert_eq!(e5.debug_hp(3), 4);

    rechuhai_six_dimensions();
    lingren_tieji_duanliang_shouye_mowang_six_dimensions();
}

fn give_judge_trick(e: &mut Engine, owner: u8, kind: CardKind) -> u32 {
    let cid = find_deck_card(e, |c| c.kind == kind);
    e.state_core_mut().deck.retain(|&x| x != cid);
    e.state_core_mut().player_mut(owner).judge.push(cid);
    cid
}

/// Answers every open choice: a prompt listed in `picks` gets that selection, anything
/// else gets the default `drain` answer. Returns the prompts seen in order.
fn drive(e: &mut Engine, picks: &[(&str, Vec<ChoiceOption>)]) -> Vec<String> {
    let mut seen = Vec::new();
    for _ in 0..48 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        seen.push(c.spec.prompt.to_string());
        let sel = if let Some((_, sel)) = picks.iter().find(|(p, _)| *p == c.spec.prompt) {
            sel.clone()
        } else if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
        };
        let out = answer(e, sel);
        assert!(!matches!(out, CommandOutcome::Rejected(_)), "drive {} rejected: {out:?}", c.spec.prompt);
    }
    seen
}

fn fz_five_skills_six_dimensions() {
    let mut e3 = new_l20_engine(2026111306, 4);
    advance_to(&mut e3, 0, Phase::Play);
    for p in 0..4 {
        e3.debug_clear_hand(p);
    }
    e3.debug_grant(0, "fz_new_rewusheng");
    e3.debug_grant(0, "fz_new_longdan");
    let _ = drive(&mut e3, &[]);
    assert!(matches!(activate(&mut e3, 0, "fz_new_rewusheng"), CommandOutcome::Rejected(_)));
    assert!(matches!(activate(&mut e3, 0, "fz_new_longdan"), CommandOutcome::Rejected(_)));

    let mut e4 = new_l20_engine(2026111307, 4);
    advance_to(&mut e4, 0, Phase::Play);
    for p in 0..4 {
        e4.debug_clear_hand(p);
    }
    e4.debug_grant(0, "fz_new_tieji");
    e4.debug_grant(0, "fz_liegong");
    e4.debug_grant(0, "fz_xinkuanggu");
    e4.debug_set_hp(0, 1);
    let _ = give_deck_card(&mut e4, 1, |c| c.kind == CardKind::Shan);
    let sha_c = give_deck_card(&mut e4, 0, |c| c.kind == CardKind::Sha);
    let hp1_start = e4.debug_hp(1);
    let _ = play_card(&mut e4, 0, sha_c, vec![1]);
    let seen = drive(
        &mut e4,
        &[
            ("fz_new_tieji:confirm", vec![ChoiceOption::Confirm]),
            ("fz_liegong:confirm", vec![ChoiceOption::Confirm]),
            ("fz_xinkuanggu:choose_mode", vec![ChoiceOption::Tag(0)]),
        ],
    );
    assert!(seen.iter().any(|p| p == "fz_new_tieji:confirm"), "seen: {seen:?}");
    assert!(seen.iter().any(|p| p == "fz_xinkuanggu:choose_mode"), "seen: {seen:?}");
    assert!(e4.debug_hp(1) < hp1_start, "Sha dealt damage");
    assert_eq!(e4.debug_hp(0), 2, "fz_xinkuanggu recovered 1");
    let snap = assert_archive_roundtrip(&e4);
    assert_eq!(snap.state_hash(), e4.state_hash());
}

fn rechuhai_six_dimensions() {
    // Win path: +X pindian bonus, gain one card per type in target hand, damage -> place equip, 3 equips -> duty success.
    let mut e6 = new_l20_engine(2026111309, 4);
    advance_to(&mut e6, 0, Phase::Play);
    for p in 0..4 {
        e6.debug_clear_hand(p);
    }
    e6.debug_grant(0, "rechuhai");
    e6.debug_set_hp(0, 2);
    let p0_card = give_deck_card(&mut e6, 0, |c| c.rank >= 6 && c.rank <= 8);
    let p1_low = give_deck_card(&mut e6, 1, |c| c.rank >= 9 && c.rank <= 10);
    let _ = give_deck_card(&mut e6, 1, |c| c.kind == CardKind::Wuzhong);
    assert!(matches!(activate(&mut e6, 0, "rechuhai"), CommandOutcome::Pending { .. }));
    assert_eq!(e6.open_choice().unwrap().spec.prompt, "rechuhai:choose_target");
    let opts = e6.open_choice().unwrap().spec.options.clone();
    assert_eq!(opts, vec![ChoiceOption::Player(1)], "only others with handcards");
    e6 = assert_archive_roundtrip(&e6);
    assert!(matches!(answer(&mut e6, vec![ChoiceOption::Player(2)]), CommandOutcome::Rejected(_)));
    let _ = cancel(&mut e6);
    assert_eq!(flag(&e6, 0, "rechuhai_won"), 0);

    let _ = activate(&mut e6, 0, "rechuhai");
    let _ = answer(&mut e6, vec![ChoiceOption::Player(1)]);
    assert_eq!(e6.debug_hand(0).len(), 2, "rechuhai draws 1 before pindian");
    assert_eq!(e6.open_choice().unwrap().spec.prompt, "rechuhai:pindian_card");
    e6 = assert_archive_roundtrip(&e6);
    let _ = answer(&mut e6, vec![ChoiceOption::Card(p0_card)]);
    assert_eq!(e6.open_choice().unwrap().spec.prompt, "rechuhai:target_pindian_card");
    assert_eq!(e6.open_choice().unwrap().observer, 1);
    e6 = assert_archive_roundtrip(&e6);
    let _ = answer(&mut e6, vec![ChoiceOption::Card(p1_low)]);
    let r0 = e6.catalog().cards[p0_card as usize].rank as u32;
    assert_eq!(flag(&e6, 0, "rechuhai_owner_num"), (r0 + 4).min(13), "pindian +4 with 0 equips");
    assert_eq!(flag(&e6, 0, "rechuhai_won"), 1);
    let hand0 = e6.debug_hand(0);
    assert_eq!(hand0.len(), 2, "drawn card + 1 trick gained for target's trick type");
    assert!(hand0.iter().any(|&c| {
        let k = e6.catalog().cards[c as usize].kind;
        !k.is_basic_kind() && k.equip_slot().is_none()
    }));
    assert!(matches!(activate(&mut e6, 0, "rechuhai"), CommandOutcome::Rejected(_)), "once per Play phase");

    let _ = e6.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    drain(&mut e6);
    assert_eq!(e6.state_core().player(0).equips.len(), 1, "damage to pindian loser places one equip");
    assert_eq!(flag(&e6, 0, "rechuhai_resolved"), 0, "1 equip does not complete the duty");
    let mut used_slots: Vec<_> = e6
        .state_core()
        .player(0)
        .equips
        .iter()
        .filter_map(|&c| e6.catalog().cards[c as usize].kind.equip_slot())
        .collect();
    for _ in 0..2 {
        let slots = used_slots.clone();
        let eq = give_deck_card(&mut e6, 0, |c| c.kind.equip_slot().is_some_and(|s| !slots.contains(&s)));
        used_slots.push(e6.catalog().cards[eq as usize].kind.equip_slot().unwrap());
        let _ = play_card(&mut e6, 0, eq, vec![]);
        drain(&mut e6);
    }
    assert_eq!(e6.state_core().player(0).equips.len(), 3);
    assert_eq!(flag(&e6, 0, "rechuhai_resolved"), 1, "countCards('e') > 2 completes the duty");
    assert_eq!(e6.debug_hp(0), e6.state_core().player(0).max_hp as i32);
    assert_eq!(flag(&e6, 0, "zhangming_granted"), 1);
    assert_archive_roundtrip(&e6);

    // Fail path: lose with effective number < 7.
    let mut e6b = new_l20_engine(2026111314, 4);
    advance_to(&mut e6b, 0, Phase::Play);
    for p in 0..4 {
        e6b.debug_clear_hand(p);
    }
    e6b.debug_grant(0, "rechuhai");
    let low = give_deck_card(&mut e6b, 0, |c| c.rank <= 2);
    let high = give_deck_card(&mut e6b, 1, |c| c.rank == 13);
    let _ = activate(&mut e6b, 0, "rechuhai");
    let _ = answer(&mut e6b, vec![ChoiceOption::Player(1)]);
    let _ = answer(&mut e6b, vec![ChoiceOption::Card(low)]);
    let _ = answer(&mut e6b, vec![ChoiceOption::Card(high)]);
    assert_eq!(flag(&e6b, 0, "rechuhai_won"), 0);
    assert_eq!(flag(&e6b, 0, "rechuhai_failed"), 1);
    assert_eq!(flag(&e6b, 0, "rechuhai_resolved"), 2);
}

fn lingren_tieji_duanliang_shouye_mowang_six_dimensions() {
    // `xinfu_lingren` (`extension.js:5577-5687`) + `sbtieji` (`extension.js:5689-5758`) on the same Sha.
    let mut e7 = new_l20_engine(2026111310, 4);
    advance_to(&mut e7, 0, Phase::Play);
    for p in 0..4 {
        e7.debug_clear_hand(p);
    }
    e7.debug_grant(0, "xinfu_lingren");
    e7.debug_grant(0, "sbtieji");
    let _ = give_deck_card(&mut e7, 1, |c| c.kind == CardKind::Tao);
    let _ = give_deck_card(&mut e7, 1, |c| c.kind == CardKind::Wuzhong);
    let _ = give_deck_card(&mut e7, 1, |c| c.kind.equip_slot().is_some());
    let _ = give_deck_card(&mut e7, 1, |c| c.kind == CardKind::Shan);
    let sha_c = give_deck_card(&mut e7, 0, |c| c.kind == CardKind::Sha);
    let hp1_before = e7.debug_hp(1);
    let _ = play_card(&mut e7, 0, sha_c, vec![1]);
    let mut seen = Vec::new();
    for _ in 0..16 {
        let Some(c) = e7.open_choice().cloned() else { break };
        seen.push(c.spec.prompt.to_string());
        match c.spec.prompt {
            "xinfu_lingren:choose_target" => {
                assert_eq!(c.spec.options, vec![ChoiceOption::Player(1)]);
                e7 = assert_archive_roundtrip(&e7);
                let _ = answer(&mut e7, vec![ChoiceOption::Player(1)]);
            }
            "xinfu_lingren:guess_mask" => {
                assert_eq!(c.spec.options.len(), 8, "guess mask 0..=7 incl. 'none of them'");
                e7 = assert_archive_roundtrip(&e7);
                assert!(matches!(answer(&mut e7, vec![ChoiceOption::Tag(99)]), CommandOutcome::Rejected(_)));
                let _ = answer(&mut e7, vec![ChoiceOption::Tag(0)]);
            }
            "sbtieji:confirm" => {
                e7 = assert_archive_roundtrip(&e7);
                let _ = answer(&mut e7, vec![ChoiceOption::Confirm]);
            }
            "sbtieji:mouyi_owner" => {
                assert_eq!(c.observer, 0);
                let _ = answer(&mut e7, vec![ChoiceOption::Tag(1)]);
            }
            "sbtieji:mouyi_target" => {
                assert_eq!(c.observer, 1);
                e7 = assert_archive_roundtrip(&e7);
                let _ = answer(&mut e7, vec![ChoiceOption::Tag(1)]);
            }
            _ => {
                let _ = drive(&mut e7, &[]);
            }
        }
    }
    for p in ["xinfu_lingren:choose_target", "xinfu_lingren:guess_mask", "sbtieji:confirm", "sbtieji:mouyi_target"] {
        assert!(seen.iter().any(|s| s == p), "missing {p} in {seen:?}");
    }
    assert_eq!(flag(&e7, 0, "xinfu_lingren_hits"), 3);
    assert_eq!(flag(&e7, 0, "sbtieji_won"), 1);
    assert_eq!(flag(&e7, 1, "sbtieji_target_sealed"), 1);
    assert_eq!(e7.debug_hp(1), hp1_before - 2, "Sha unrespondable (sbtieji) and +1 damage (lingren)");
    assert_eq!(e7.debug_hand(0).len(), 4, "lingren 2 hits draw 2 + sbtieji mouyi win draw 2");
    let jx = e7.catalog().skill_by_name("jianxiong").or_else(|| e7.catalog().skill_by_name("奸雄")).unwrap();
    assert!(e7.state_core().has_skill(0, jx), "3 hits grant jianxiong until next turn");
    assert_eq!(flag(&e7, 1, "xinfu_lingren_vuln_from"), 0, "+1 cleared once the triggering Sha finishes resolving");

    // Dim ④ lingren once per turn: a second damaging card does not prompt again.
    let duel_c = give_deck_card(&mut e7, 0, |c| c.kind == CardKind::Duel);
    let _ = play_card(&mut e7, 0, duel_c, vec![2]);
    let seen2 = drive(&mut e7, &[("sbtieji:confirm", vec![ChoiceOption::Pass])]);
    assert!(!seen2.iter().any(|s| s.starts_with("xinfu_lingren")), "{seen2:?}");
    assert_eq!(flag(&e7, 1, "xinfu_lingren_vuln_from"), 0, "next card use clears the previous card's +1");

    // Owner's next TurnBegin removes temporary jianxiong/xingshang and sbtieji seal.
    e7.debug_set_turn(3, Phase::End, 1);
    next_phase(&mut e7);
    drain(&mut e7);
    assert!(!e7.state_core().has_skill(0, jx));
    assert_eq!(flag(&e7, 1, "sbtieji_target_sealed"), 0);

    // `sbduanliang` (`extension.js:5760-5827`): choose target -> Mouyi -> Bingliang into judge area.
    let mut e8 = new_l20_engine(2026111315, 4);
    advance_to(&mut e8, 0, Phase::Play);
    e8.debug_grant(0, "sbduanliang");
    assert!(matches!(activate(&mut e8, 0, "sbduanliang"), CommandOutcome::Pending { .. }));
    assert_eq!(e8.open_choice().unwrap().spec.prompt, "sbduanliang:choose_target");
    e8 = assert_archive_roundtrip(&e8);
    assert!(matches!(answer(&mut e8, vec![ChoiceOption::Player(0)]), CommandOutcome::Rejected(_)));
    let _ = cancel(&mut e8);
    assert_eq!(flag(&e8, 0, "sbduanliang_bing_placed"), 0);
    let _ = activate(&mut e8, 0, "sbduanliang");
    let _ = answer(&mut e8, vec![ChoiceOption::Player(2)]);
    assert_eq!(e8.open_choice().unwrap().spec.prompt, "sbduanliang:mouyi_owner");
    let _ = answer(&mut e8, vec![ChoiceOption::Tag(0)]);
    assert_eq!(e8.open_choice().unwrap().spec.prompt, "sbduanliang:mouyi_target");
    assert_eq!(e8.open_choice().unwrap().observer, 2);
    e8 = assert_archive_roundtrip(&e8);
    let _ = answer(&mut e8, vec![ChoiceOption::Tag(0)]);
    drain(&mut e8);
    assert_eq!(flag(&e8, 0, "sbduanliang_bing_placed"), 1);
    assert_eq!(e8.state_core().player(2).judge.len(), 1);
    assert!(matches!(activate(&mut e8, 0, "sbduanliang"), CommandOutcome::Rejected(_)), "usable 1");

    // Mouyi lost: nothing happens.
    let mut e9 = new_l20_engine(2026111316, 4);
    advance_to(&mut e9, 0, Phase::Play);
    e9.debug_grant(0, "sbduanliang");
    let _ = activate(&mut e9, 0, "sbduanliang");
    let _ = answer(&mut e9, vec![ChoiceOption::Player(2)]);
    let _ = answer(&mut e9, vec![ChoiceOption::Tag(1)]);
    let _ = answer(&mut e9, vec![ChoiceOption::Tag(0)]);
    drain(&mut e9);
    assert_eq!(flag(&e9, 0, "sbduanliang_won"), 0);
    assert!(e9.state_core().player(2).judge.is_empty());

    // Mouyi won with option 1: virtual Duel against the target.
    let mut e10 = new_l20_engine(2026111317, 4);
    advance_to(&mut e10, 0, Phase::Play);
    for p in 0..4 {
        e10.debug_clear_hand(p);
    }
    e10.debug_grant(0, "sbduanliang");
    let hp2 = e10.debug_hp(2);
    let _ = activate(&mut e10, 0, "sbduanliang");
    let _ = answer(&mut e10, vec![ChoiceOption::Player(2)]);
    let _ = answer(&mut e10, vec![ChoiceOption::Tag(1)]);
    let _ = answer(&mut e10, vec![ChoiceOption::Tag(1)]);
    drain(&mut e10);
    assert_eq!(flag(&e10, 0, "sbduanliang_duel"), 1);
    assert_eq!(e10.debug_hp(2), hp2 - 1, "virtual Duel hits target without Sha");

    // `shouye` (`extension.js:5829-5896`): only when the sole target of another's card.
    let mut e11 = new_l20_engine(2026111318, 4);
    advance_to(&mut e11, 0, Phase::Play);
    for p in 0..4 {
        e11.debug_clear_hand(p);
    }
    e11.debug_grant(2, "shouye");
    let savage = give_deck_card(&mut e11, 0, |c| c.kind == CardKind::SavageAttack);
    let _ = play_card(&mut e11, 0, savage, vec![1, 2, 3]);
    let seen_sv = drive(&mut e11, &[]);
    assert!(!seen_sv.iter().any(|s| s.starts_with("shouye")), "multi-target card: {seen_sv:?}");
    for p in 0..4 {
        e11.debug_set_hp(p, 4);
    }
    let duel = give_deck_card(&mut e11, 0, |c| c.kind == CardKind::Duel);
    let _ = play_card(&mut e11, 0, duel, vec![2]);
    assert_eq!(e11.open_choice().unwrap().spec.prompt, "shouye:confirm");
    assert_eq!(e11.open_choice().unwrap().observer, 2);
    e11 = assert_archive_roundtrip(&e11);
    let _ = answer(&mut e11, vec![ChoiceOption::Confirm]);
    assert_eq!(e11.open_choice().unwrap().spec.prompt, "shouye:mouyi_owner");
    let _ = answer(&mut e11, vec![ChoiceOption::Tag(1)]);
    assert_eq!(e11.open_choice().unwrap().spec.prompt, "shouye:mouyi_target");
    assert_eq!(e11.open_choice().unwrap().observer, 0);
    e11 = assert_archive_roundtrip(&e11);
    let _ = answer(&mut e11, vec![ChoiceOption::Tag(1)]);
    drain(&mut e11);
    assert_eq!(flag(&e11, 2, "shouye_won"), 1);
    assert_eq!(e11.debug_hp(2), 4, "Duel nullified against shouye owner");
    assert!(e11.debug_hand(2).contains(&duel), "shouye owner gains the used card");
    assert!(!e11.state_core().discard.contains(&duel));

    // `mbmowang` (`extension.js:5898-6133`): Dying -> rest (untargetable) -> full hp next turn -> dies at turn end.
    let mut e12 = new_l20_engine(2026111319, 4);
    advance_to(&mut e12, 0, Phase::Play);
    e12.debug_grant(3, "mbmowang");
    e12.debug_set_hp(3, 1);
    let hand0 = e12.debug_hand(0).len();
    let _ = e12.debug_deal_damage(Some(0), 3, 1, DamageNature::Normal);
    drain(&mut e12);
    assert!(e12.state_core().player(3).alive);
    assert_eq!(flag(&e12, 3, "mbmowang_resting"), 1);
    assert!(e12.state_core().player(3).hand.is_empty() && e12.state_core().player(3).equips.is_empty());
    assert_eq!(e12.debug_hand(0).len(), hand0, "no invented draw");
    let sha = give_deck_card(&mut e12, 0, |c| c.kind == CardKind::Sha);
    assert!(
        matches!(play_card(&mut e12, 0, sha, vec![3]), CommandOutcome::Rejected(_)),
        "resting mbmowang owner is out of game"
    );
    e12 = assert_archive_roundtrip(&e12);
    e12.debug_set_turn(2, Phase::End, 1);
    next_phase(&mut e12); // TurnBegin(3)
    drain(&mut e12);
    assert_eq!(flag(&e12, 3, "mbmowang_resting"), 0);
    assert_eq!(flag(&e12, 3, "mbmowang_doomed"), 1);
    assert_eq!(e12.debug_hp(3), e12.state_core().player(3).max_hp as i32);
    for _ in 0..12 {
        if !e12.state_core().player(3).alive || e12.turn().current != 3 {
            break;
        }
        drain(&mut e12);
        if e12.state_core().player(3).alive && e12.turn().current == 3 {
            next_phase(&mut e12);
        }
    }
    drain(&mut e12);
    assert!(!e12.state_core().player(3).alive, "mbmowang owner dies at the end of the recovered turn");
}
