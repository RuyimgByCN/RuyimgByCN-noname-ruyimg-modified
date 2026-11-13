//! U01 3D精选（`d3-upstream-noname`，31 将 + 64 技）6D 真实对局事件路径验收测试。
//!
//! 覆盖：
//! 1. `u01_all_31_generals_and_64_host_skills_registered_and_zhangkai_hp`：
//!    31 将（含 `ddd_zhangkai` 3/4 体力上限，`character.js:128-134`）与 64 技全部注册为原生 `HostSkillBehavior`。
//! 2. `u01_batch_a_real_match_6d_verification`：
//!    Batch A 代表技能（`dddyuanzhen`、`dddjuxian`、`ddddiedang`、`dddjinggou`）6 维度真实对局检验。
//! 3. `u01_batch_b_real_match_6d_verification`：
//!    Batch B 代表技能（`dddxuyu`、`dddxuxiao`、`dddquche`、`ddddongcha` + `dddzhijie`）6 维度真实对局检验。
//! 4. `u01_batch_c_and_d_real_match_6d_verification`：
//!    Batch C & D 代表技能（`dddqice`、`dddfenji`、`dddxiaheng`、`dddyouxue`、`dddduanbing`、`dddxuanlun` + `dddlanghuai`、`dddbailei`）6 维度真实对局检验。

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use content_native_loader::{
    bridge_pack, decision34_pack9_for_coexistence, resolve_u01_3d_gallery, U01_DDD_GENERALS,
    U01_DDD_SKILLS,
};
use engine_native::ai::BoundedPlanner;
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::catalog::{CardKind, Suit};
use engine_native::content::extension::ExtensionPack;
use engine_native::event::Phase;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand};

static DEDUP: AtomicU64 = AtomicU64::new(3_100_000);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action,
    })
}

fn repo_extensions_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

fn build_u01_packs() -> (ExtensionPack, ExtensionPack) {
    let d3_json = std::fs::read_to_string(
        repo_extensions_root().join("d3-upstream-noname/assets/character.json"),
    )
    .expect("read d3-upstream-noname character.json")
    .replace("\r\n", "\n");
    let d3_pack = bridge_pack(&d3_json).expect("bridge d3 character.json").pack;
    let p9_pack = decision34_pack9_for_coexistence(&[&d3_pack]);
    (d3_pack, p9_pack)
}

fn find_general(e: &Engine, name: &str) -> u16 {
    (0..e.catalog().general_count() as u16)
        .find(|&idx| e.catalog().general_meta(idx).is_some_and(|m| m.0 == name))
        .unwrap_or_else(|| panic!("general {name} not found in catalog"))
}

fn find_card_by<F: Fn(u32, &engine_native::catalog::CardDef) -> bool>(e: &Engine, pred: F) -> u32 {
    e.catalog()
        .cards
        .iter()
        .enumerate()
        .find_map(|(idx, def)| pred(idx as u32, def).then_some(idx as u32))
        .expect("matching card in catalog")
}

fn remove_card_from_all_zones(e: &mut Engine, card: u32) {
    e.state_core_mut().deck.retain(|&c| c != card);
    e.state_core_mut().discard.retain(|&c| c != card);
    e.state_core_mut().table.retain(|&c| c != card);
    for ids in e.state_core_mut().vm_aux.piles.values_mut() {
        ids.retain(|&c| c != card);
    }
    for p in 0..e.state_core().players.len() as u8 {
        e.state_core_mut().player_mut(p).hand.retain(|&c| c != card);
        e.state_core_mut().player_mut(p).equips.retain(|&c| c != card);
        e.state_core_mut().player_mut(p).judge.retain(|&c| c != card);
    }
}

fn put_in_hand(e: &mut Engine, seat: u8, card: u32) {
    remove_card_from_all_zones(e, card);
    e.state_core_mut().player_mut(seat).hand.push(card);
}

fn clear_hand(e: &mut Engine, seat: u8) {
    let cards: Vec<u32> = e.state_core_mut().player_mut(seat).hand.drain(..).collect();
    e.state_core_mut().deck.extend(cards);
}

fn truncate_hand(e: &mut Engine, seat: u8, keep: usize) {
    if e.state_core().player(seat).hand.len() > keep {
        let extra: Vec<u32> = e.state_core_mut().player_mut(seat).hand.drain(keep..).collect();
        e.state_core_mut().deck.extend(extra);
    }
}

fn clear_equips(e: &mut Engine, seat: u8) {
    let cards: Vec<u32> = e.state_core_mut().player_mut(seat).equips.drain(..).collect();
    e.state_core_mut().deck.extend(cards);
}

fn clear_judge(e: &mut Engine, seat: u8) {
    let cards: Vec<u32> = e.state_core_mut().player_mut(seat).judge.drain(..).collect();
    e.state_core_mut().deck.extend(cards);
}

fn clear_discard(e: &mut Engine) {
    let cards: Vec<u32> = e.state_core_mut().discard.drain(..).collect();
    e.state_core_mut().deck.extend(cards);
}

fn flag(e: &Engine, p: u8, key: &str) -> u32 {
    e.state_core()
        .vm_aux
        .flags
        .get(&(p, key.to_string()))
        .copied()
        .unwrap_or(0)
}

fn drain(e: &mut Engine) {
    for _ in 0..48 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        let has_pass = c.spec.options.contains(&ChoiceOption::Pass);
        let sel: Vec<ChoiceOption> = if has_pass {
            vec![ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
        };
        let _ = submit(
            e,
            c.observer,
            GameAction::Answer {
                choice_id: c.id,
                selection: sel,
            },
        );
    }
}

fn assert_mid_choice_archive_and_ai(
    e: &Engine,
    packs: &[&ExtensionPack],
    answer_sel: Vec<ChoiceOption>,
) {
    let c = e.open_choice().expect("open choice for archive & AI check").clone();
    // ⑥ AI planner (`BoundedPlanner`) answers open `Pending` choices legally without reading unrevealed info
    let planner = BoundedPlanner::new(8);
    let planned = e.plan_action(c.observer, &planner);
    assert!(
        matches!(planned.action, GameAction::Answer { choice_id, .. } if choice_id == c.id),
        "AI planner must produce a legal Answer for open choice {}: {planned:?}",
        c.spec.prompt
    );

    // ⑤ Mid-choice archive save/restore preserves `state_hash()` before and after answering
    let bytes = encode_archive(&e.save());
    let decoded = decode_archive(&bytes).expect("decode mid-choice archive");
    let mut restored =
        Engine::restore_with_extensions(&decoded, packs).expect("restore mid-choice archive");
    assert_eq!(
        restored.state_hash(),
        e.state_hash(),
        "state_hash mismatch on mid-choice restore for {}",
        c.spec.prompt
    );

    let mut cloned =
        Engine::restore_with_extensions(&e.save(), packs).expect("clone engine for post-answer hash");
    let out1 = submit(
        &mut restored,
        c.observer,
        GameAction::Answer {
            choice_id: c.id,
            selection: answer_sel.clone(),
        },
    );
    let out2 = submit(
        &mut cloned,
        c.observer,
        GameAction::Answer {
            choice_id: c.id,
            selection: answer_sel,
        },
    );
    assert!(
        !matches!(out1, CommandOutcome::Rejected(_)) && !matches!(out2, CommandOutcome::Rejected(_)),
        "mid-choice answer rejected: {out1:?} / {out2:?}"
    );
    assert_eq!(
        restored.state_hash(),
        cloned.state_hash(),
        "state_hash mismatch after answering {}",
        c.spec.prompt
    );
}

/// 1. 验证 31 名 `ddd_*` 武将（含 `ddd_zhangkai` 3/4 体力上限，`character.js:128-134`）与
///    64 项 `ddd*` 技能全部注册为原生 `HostSkillBehavior`（`!s.behavior.is_vm_skill()`）。
#[test]
fn u01_all_31_generals_and_64_host_skills_registered_and_zhangkai_hp() {
    let (d3_pack, p9_pack) = build_u01_packs();
    let packs = [&d3_pack, &p9_pack];
    let mut e = Engine::new_with_extensions(310101, 3, &packs).expect("U01 engine");

    assert_eq!(U01_DDD_GENERALS.len(), 31);
    assert_eq!(U01_DDD_SKILLS.len(), 64);

    for &(gname, expected_fac, expected_gender, expected_max_hp, expected_skills) in &U01_DDD_GENERALS {
        let gid = find_general(&e, gname);
        let (_, fac, skills, max_hp, gender) = e.catalog().general_meta(gid).expect("general_meta");
        assert_eq!(fac, expected_fac, "faction mismatch for {gname}");
        assert_eq!(gender, expected_gender, "gender mismatch for {gname}");
        assert_eq!(max_hp, expected_max_hp, "max_hp mismatch for {gname}");
        for &sk in expected_skills {
            assert!(
                skills.contains(&sk),
                "general {gname} missing expected skill {sk} (got {skills:?})"
            );
        }
    }

    // `ddd_zhangkai` (`archive/upstream/noname/apps/core/extension/3D精选/character/character.js:128-134`):
    // `"3/4"` -> initial hp == 3, max_hp == 4.
    let zhangkai = find_general(&e, "ddd_zhangkai");
    e.assign_general(0, zhangkai).expect("assign ddd_zhangkai");
    assert_eq!(e.state_core().player(0).hp, 3, "ddd_zhangkai initial hp must be 3");
    assert_eq!(e.state_core().player(0).max_hp, 4, "ddd_zhangkai max_hp must be 4");

    for &sk in &U01_DDD_SKILLS {
        let def_id = e
            .catalog()
            .skill_by_name(sk)
            .unwrap_or_else(|| panic!("U01 skill {sk} missing from Catalog"));
        let sdef = &e.catalog().skills[def_id as usize];
        assert!(
            !sdef.behavior.is_vm_skill(),
            "U01 skill {sk} must be native HostSkillBehavior, not VM skill"
        );
        assert!(
            sdef.behavior.as_host_behavior().is_some(),
            "U01 skill {sk} must expose HostBehavior"
        );
    }

    let gal = resolve_u01_3d_gallery("ddd_caomao", 60, -40, 200).expect("valid ddd_caomao");
    assert_eq!((gal.yaw_deg, gal.pitch_deg, gal.zoom_pct), (45, -30, 150));
    assert!(resolve_u01_3d_gallery("unknown_general", 0, 0, 100).is_err());
}

/// 2. Batch A 代表技能（`dddyuanzhen`、`dddjuxian`、`ddddiedang`、`dddjinggou`）6D 真实对局检验。
#[test]
fn u01_batch_a_real_match_6d_verification() {
    let (d3_pack, p9_pack) = build_u01_packs();
    let packs = [&d3_pack, &p9_pack];
    let mut e = Engine::new_with_extensions(310102, 4, &packs).expect("U01 Batch A engine");

    // ---- (A1) `dddyuanzhen` 远振 (`archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:6-36`) ----
    let qianzhao = find_general(&e, "ddd_qianzhao");
    e.assign_general(0, qianzhao).expect("assign ddd_qianzhao");
    e.debug_set_turn(0, Phase::Play, 1);

    let le_card_1 = find_card_by(&e, |_, d| d.kind == CardKind::Le);
    let le_card_2 = find_card_by(&e, |id, d| id != le_card_1 && d.kind == CardKind::Le);
    put_in_hand(&mut e, 0, le_card_1);
    put_in_hand(&mut e, 0, le_card_2);

    // ② Negative guard: seat 0 -> seat 1 has distance 1, so `dddyuanzhen` must NOT trigger.
    assert_eq!(e.state_core().computed_distance(e.catalog(), 0, 1), 1);
    let out_dist1 = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: le_card_1,
            targets: vec![1],
        },
    );
    assert!(
        matches!(out_dist1, CommandOutcome::Accepted { .. }),
        "dddyuanzhen must not trigger at distance 1: {out_dist1:?}"
    );
    assert!(e.open_choice().is_none());

    // ① Positive trigger: seat 0 -> seat 2 in 4p has distance 2 (!= 1), triggering `"dddyuanzhen_discard"`.
    assert_eq!(e.state_core().computed_distance(e.catalog(), 0, 2), 2);
    let c2_discard = e.state_core().player(2).hand[0];
    let out_dist2 = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: le_card_2,
            targets: vec![2],
        },
    );
    let choice_yz = match out_dist2 {
        CommandOutcome::Pending { choice, .. } => choice,
        other => panic!("expected Pending dddyuanzhen_discard, got {other:?}"),
    };
    assert_eq!(choice_yz.spec.prompt, "dddyuanzhen_discard");
    assert_eq!(choice_yz.observer, 2);
    assert!(choice_yz.spec.allow_cancel);

    // ⑤ & ⑥ Mid-choice archive save/restore + AI planner check
    assert_mid_choice_archive_and_ai(&e, &packs, vec![ChoiceOption::Card(c2_discard)]);

    // ③ & ④ Choice A (target discards 1 card) vs Choice B / Cancel (target cancels -> owner draws 1)
    let mut e_yz_cancel =
        Engine::restore_with_extensions(&e.save(), &packs).expect("fork dddyuanzhen cancel");
    let owner_hand_before = e.state_core().player(0).hand.len();
    let t2_hand_before = e.state_core().player(2).hand.len();

    let _ = submit(
        &mut e,
        2,
        GameAction::Answer {
            choice_id: choice_yz.id,
            selection: vec![ChoiceOption::Card(c2_discard)],
        },
    );
    assert_eq!(e.state_core().player(2).hand.len(), t2_hand_before - 1);
    assert_eq!(e.state_core().player(0).hand.len(), owner_hand_before);

    let _ = submit(
        &mut e_yz_cancel,
        2,
        GameAction::Answer {
            choice_id: choice_yz.id,
            selection: Vec::new(),
        },
    );
    assert_eq!(e_yz_cancel.state_core().player(2).hand.len(), t2_hand_before);
    assert_eq!(
        e_yz_cancel.state_core().player(0).hand.len(),
        owner_hand_before + 1,
        "canceling dddyuanzhen_discard must let ddd_qianzhao draw 1 card"
    );
    assert_ne!(e.state_hash(), e_yz_cancel.state_hash());

    // ---- (A2) `dddjuxian` 举贤 (`archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:244-383`) ----
    let zhangcheng = find_general(&e, "ddd_zhangcheng");
    e.assign_general(0, zhangcheng).expect("assign ddd_zhangcheng");
    e.debug_set_turn(0, Phase::Play, 2);
    let juxian_def = e.catalog().skill_by_name("dddjuxian").expect("dddjuxian");

    let red_c0 = find_card_by(&e, |_, d| matches!(d.suit, Suit::Heart | Suit::Diamond));
    let red_c1 = find_card_by(&e, |id, d| {
        id != red_c0 && matches!(d.suit, Suit::Heart | Suit::Diamond)
    });
    let blk_deck = find_card_by(&e, |_, d| matches!(d.suit, Suit::Spade | Suit::Club));
    clear_hand(&mut e, 0);
    clear_equips(&mut e, 0);
    clear_hand(&mut e, 1);
    clear_equips(&mut e, 1);
    put_in_hand(&mut e, 0, red_c0);
    put_in_hand(&mut e, 1, red_c1);

    // ④ Cancel on `"dddjuxian_mode"` leaves state unchanged and does not consume use count.
    let out_jx_cancel = submit(&mut e, 0, GameAction::ActivateSkill { skill: juxian_def });
    let c_jx_cancel = match out_jx_cancel {
        CommandOutcome::Pending { choice, .. } => choice,
        other => panic!("expected Pending dddjuxian_mode, got {other:?}"),
    };
    assert_eq!(c_jx_cancel.spec.prompt, "dddjuxian_mode");
    assert_mid_choice_archive_and_ai(&e, &packs, vec![ChoiceOption::Tag(0)]);
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_jx_cancel.id,
            selection: Vec::new(),
        },
    );
    assert_eq!(flag(&e, 0, "dddjuxian_used"), 0);

    // ① & ③ Branch Tag(0): put `red_c0` on top of deck and gain `red_c1` from seat 1 (same color -> no ban).
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: juxian_def });
    let c_mode0 = e.open_choice().expect("dddjuxian_mode").clone();
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_mode0.id,
            selection: vec![ChoiceOption::Tag(0)],
        },
    );
    let c_card0 = e.open_choice().expect("dddjuxian_card").clone();
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_card0.id,
            selection: vec![ChoiceOption::Card(red_c0)],
        },
    );
    let c_tgt0 = e.open_choice().expect("dddjuxian_target").clone();
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_tgt0.id,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    assert_eq!(e.state_core().deck.last().copied(), Some(red_c0));
    assert!(e.state_core().player(0).hand.contains(&red_c1));
    assert_eq!(flag(&e, 0, "dddjuxian_top_used"), 1);
    assert_eq!(flag(&e, 0, "dddjuxian_ban"), 0);

    // Branch Tag(1): give `red_c1` to seat 1 and draw `blk_deck` from deck (color mismatch -> sets `dddjuxian_ban = 1`).
    remove_card_from_all_zones(&mut e, blk_deck);
    e.state_core_mut().deck.push(blk_deck);
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: juxian_def });
    let c_mode1 = e.open_choice().expect("dddjuxian_mode 2nd").clone();
    assert_eq!(
        c_mode1.spec.options,
        vec![ChoiceOption::Tag(1)],
        "Tag(0) already used this turn; only Tag(1) remains"
    );
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_mode1.id,
            selection: vec![ChoiceOption::Tag(1)],
        },
    );
    let c_card1 = e.open_choice().expect("dddjuxian_card 2nd").clone();
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_card1.id,
            selection: vec![ChoiceOption::Card(red_c1)],
        },
    );
    let c_tgt1 = e.open_choice().expect("dddjuxian_target 2nd").clone();
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_tgt1.id,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    assert!(e.state_core().player(1).hand.contains(&red_c1));
    assert!(e.state_core().player(0).hand.contains(&blk_deck));
    assert_eq!(flag(&e, 0, "dddjuxian_ban"), 1);

    // ② Negative guard: once `dddjuxian_ban == 1`, activating `dddjuxian` is rejected.
    assert!(matches!(
        submit(&mut e, 0, GameAction::ActivateSkill { skill: juxian_def }),
        CommandOutcome::Rejected(_)
    ));

    // ---- (A3) `ddddiedang` 跌宕 (`archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:802-860`) ----
    let sunliang = find_general(&e, "ddd_sunliang");
    e.assign_general(0, sunliang).expect("assign ddd_sunliang");
    e.debug_set_turn(0, Phase::Play, 3);
    let diedang_def = e.catalog().skill_by_name("ddddiedang").expect("ddddiedang");

    // When ending hand size is strictly between min (seat 1: 0 cards) and max (seat 2: 6 cards), state does NOT toggle.
    truncate_hand(&mut e, 0, 1);
    clear_hand(&mut e, 1);
    while e.state_core().player(2).hand.len() < 6 {
        let c = e.state_core_mut().deck.pop().expect("deck card");
        e.state_core_mut().player_mut(2).hand.push(c);
    }
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: diedang_def });
    let c_dd0 = e.open_choice().expect("ddddiedang_discard st=0").clone();
    assert_eq!(c_dd0.spec.prompt, "ddddiedang_discard");
    let disc_one = e.state_core().player(0).hand[0];
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_dd0.id,
            selection: vec![ChoiceOption::Card(disc_one)],
        },
    );
    assert_eq!(e.state_core().player(0).hand.len(), 3);
    assert_eq!(
        flag(&e, 0, "ddddiedang_state"),
        0,
        "ddddiedang must not toggle when hand size is neither max nor min"
    );

    // ② Negative guard: `ddddiedang` is `once_per_turn`, so activating again in the same turn is rejected.
    assert!(matches!(
        submit(&mut e, 0, GameAction::ActivateSkill { skill: diedang_def }),
        CommandOutcome::Rejected(_)
    ));

    // Next turn: advance through seat 0's `Phase::Prepare` so `once_per_turn` resets, then seat 0 ends with max hand size -> toggles `ddddiedang_state` to 1.
    e.debug_set_turn(3, Phase::End, 3);
    let _ = submit(&mut e, 3, GameAction::NextPhase);
    e.debug_set_turn(0, Phase::Play, 4);
    truncate_hand(&mut e, 2, 1);
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: diedang_def });
    let c_dd1 = e.open_choice().expect("ddddiedang_discard toggle").clone();
    let disc_two = e.state_core().player(0).hand[0];
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_dd1.id,
            selection: vec![ChoiceOption::Card(disc_two)],
        },
    );
    assert_eq!(flag(&e, 0, "ddddiedang_state"), 1);

    // ---- (A4) `dddjinggou` 精彀 (`archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:443-471`) ----
    let liuchong = find_general(&e, "ddd_liuchong");
    e.assign_general(0, liuchong).expect("assign ddd_liuchong");
    e.debug_set_turn(0, Phase::Play, 5);

    let w_range2_a = find_card_by(&e, |_, d| d.kind == CardKind::Weapon && d.range == 2);
    let w_range2_b = find_card_by(&e, |id, d| {
        id != w_range2_a && d.kind == CardKind::Weapon && d.range == 2
    });
    let w_range3 = find_card_by(&e, |_, d| d.kind == CardKind::Weapon && d.range == 3);

    // ② Negative guard: seat 1 has range 3 weapon (`w_range3`), seat 0 equips range 2 weapon (`w_range2_a`) -> 2 < 3 -> does NOT trigger.
    put_in_hand(&mut e, 0, w_range2_a);
    clear_equips(&mut e, 1);
    remove_card_from_all_zones(&mut e, w_range3);
    e.state_core_mut().player_mut(1).equips.push(w_range3);
    let out_jg_neg = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: w_range2_a,
            targets: Vec::new(),
        },
    );
    assert!(
        matches!(out_jg_neg, CommandOutcome::Accepted { .. }),
        "dddjinggou must not trigger when weapon range (2) < max range (3): {out_jg_neg:?}"
    );
    assert!(e.open_choice().is_none());

    // ① Tied-max-range positive trigger: seat 1 has range 2 weapon (`w_range2_b`), seat 0 equips range 2 weapon (`w_range2_a`), tying max range 2!
    put_in_hand(&mut e, 0, w_range2_a);
    clear_equips(&mut e, 1);
    remove_card_from_all_zones(&mut e, w_range2_b);
    e.state_core_mut().player_mut(1).equips.push(w_range2_b);
    let hp1_before = e.state_core().player(1).hp;
    let out_jg_pos = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: w_range2_a,
            targets: Vec::new(),
        },
    );
    let c_jg = match out_jg_pos {
        CommandOutcome::Pending { choice, .. } => choice,
        other => panic!("expected Pending d3_jinggou_target on tied max range, got {other:?}"),
    };
    assert_eq!(c_jg.spec.prompt, "d3_jinggou_target");
    assert_mid_choice_archive_and_ai(&e, &packs, vec![ChoiceOption::Player(1)]);
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_jg.id,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    drain(&mut e);
    assert_eq!(e.state_core().player(1).hp, hp1_before - 1);
}

/// 3. Batch B 代表技能（`dddxuyu`、`dddxuxiao`、`dddquche`、`ddddongcha` + `dddzhijie`）6D 真实对局检验。
#[test]
fn u01_batch_b_real_match_6d_verification() {
    let (d3_pack, p9_pack) = build_u01_packs();
    let packs = [&d3_pack, &p9_pack];
    let mut e = Engine::new_with_extensions(310103, 3, &packs).expect("U01 Batch B engine");

    // ---- (B1) `dddxuyu` 蓄誉 (`archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:2407-2422`) ----
    let xujing = find_general(&e, "ddd_xujing");
    e.assign_general(0, xujing).expect("assign ddd_xujing");
    e.debug_set_turn(0, Phase::Play, 1);

    let le1 = find_card_by(&e, |_, d| d.kind == CardKind::Le);
    let le2 = find_card_by(&e, |id, d| id != le1 && d.kind == CardKind::Le);
    put_in_hand(&mut e, 0, le1);
    put_in_hand(&mut e, 0, le2);

    let out_xy1 = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: le1,
            targets: vec![1],
        },
    );
    let c_xy1 = match out_xy1 {
        CommandOutcome::Pending { choice, .. } => choice,
        other => panic!("expected Pending dddxuyu_confirm, got {other:?}"),
    };
    assert_eq!(c_xy1.spec.prompt, "dddxuyu_confirm");
    assert_mid_choice_archive_and_ai(&e, &packs, vec![ChoiceOption::Confirm]);

    // ③ & ④ Pass/Cancel vs Confirm
    let mut e_xy_pass =
        Engine::restore_with_extensions(&e.save(), &packs).expect("fork dddxuyu pass");
    let h_before = e.state_core().player(0).hand.len();
    let _ = submit(
        &mut e_xy_pass,
        0,
        GameAction::Answer {
            choice_id: c_xy1.id,
            selection: vec![ChoiceOption::Pass],
        },
    );
    assert_eq!(e_xy_pass.state_core().player(0).hand.len(), h_before);
    assert_eq!(flag(&e_xy_pass, 0, "dddxuyu_must_discard"), 0);

    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_xy1.id,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    assert_eq!(e.state_core().player(0).hand.len(), h_before + 1);
    assert_eq!(flag(&e, 0, "dddxuyu_must_discard"), 1);

    // Second card used in same turn forces `"dddxuyu_discard"` (`allow_cancel == false`).
    let out_xy2 = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: le2,
            targets: vec![2],
        },
    );
    let c_xy2 = match out_xy2 {
        CommandOutcome::Pending { choice, .. } => choice,
        other => panic!("expected Pending dddxuyu_discard, got {other:?}"),
    };
    assert_eq!(c_xy2.spec.prompt, "dddxuyu_discard");
    assert!(!c_xy2.spec.allow_cancel);
    let to_disc = e.state_core().player(0).hand[0];
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_xy2.id,
            selection: vec![ChoiceOption::Card(to_disc)],
        },
    );
    assert_eq!(flag(&e, 0, "dddxuyu_must_discard"), 0);

    // ---- (B2) `dddxuxiao` 谑啸 (`archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:2333-2406`) ----
    let wuzhi = find_general(&e, "ddd_wuzhi");
    e.assign_general(0, wuzhi).expect("assign ddd_wuzhi");
    clear_discard(&mut e);
    clear_judge(&mut e, 1);
    clear_judge(&mut e, 2);

    let red_tao = find_card_by(&e, |_, d| {
        d.kind == CardKind::Tao && matches!(d.suit, Suit::Heart | Suit::Diamond)
    });
    let blk_sha = find_card_by(&e, |_, d| {
        d.kind == CardKind::Sha && matches!(d.suit, Suit::Spade | Suit::Club)
    });
    e.state_core_mut().player_mut(0).hp = 1;
    clear_hand(&mut e, 0);
    put_in_hand(&mut e, 0, red_tao);
    put_in_hand(&mut e, 0, blk_sha);
    e.debug_set_turn(0, Phase::Play, 2);

    let out_to_disc = submit(&mut e, 0, GameAction::NextPhase);
    let c_disc_phase = match out_to_disc {
        CommandOutcome::Pending { choice, .. } => choice,
        other => panic!("expected discard_to_limit choice, got {other:?}"),
    };
    assert_eq!(c_disc_phase.spec.prompt, "discard_to_limit");

    // ② Negative guard: discarding a RED basic card (`red_tao`) does NOT trigger `dddxuxiao`.
    let mut e_xx_neg =
        Engine::restore_with_extensions(&e.save(), &packs).expect("fork dddxuxiao neg");
    let _ = submit(
        &mut e_xx_neg,
        0,
        GameAction::Answer {
            choice_id: c_disc_phase.id,
            selection: vec![ChoiceOption::Card(red_tao)],
        },
    );
    assert!(
        e_xx_neg.open_choice().is_none(),
        "dddxuxiao must not trigger on red card discard"
    );

    // ① Positive trigger: discarding a BLACK basic card (`blk_sha`) triggers `"dddxuxiao_card"`.
    let out_xx = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_disc_phase.id,
            selection: vec![ChoiceOption::Card(blk_sha)],
        },
    );
    let c_xx_card = match out_xx {
        CommandOutcome::Pending { choice, .. } => choice,
        other => panic!("expected Pending dddxuxiao_card, got {other:?}"),
    };
    assert_eq!(c_xx_card.spec.prompt, "dddxuxiao_card");
    assert_mid_choice_archive_and_ai(&e, &packs, vec![ChoiceOption::Card(blk_sha)]);

    let t1_hand_before = e.state_core().player(1).hand.len();
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_xx_card.id,
            selection: vec![ChoiceOption::Card(blk_sha)],
        },
    );
    let c_xx_tgt = e.open_choice().expect("dddxuxiao_target").clone();
    assert_eq!(c_xx_tgt.spec.prompt, "dddxuxiao_target");
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_xx_tgt.id,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    assert!(e.state_core().player(1).judge.contains(&blk_sha));
    assert_eq!(
        e.state_core().declared_kinds.get(&blk_sha).copied(),
        Some(CardKind::Bing)
    );
    assert_eq!(e.state_core().player(1).hand.len(), t1_hand_before + 2);

    // ---- (B3) `dddquche` 驱车 (`archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:2706-2818`) ----
    let caomao = find_general(&e, "ddd_caomao");
    e.assign_general(0, caomao).expect("assign ddd_caomao");
    e.debug_set_turn(0, Phase::Play, 3);
    let quche_def = e.catalog().skill_by_name("dddquche").expect("dddquche");
    clear_hand(&mut e, 0);
    put_in_hand(&mut e, 0, red_tao);
    put_in_hand(&mut e, 0, blk_sha);
    clear_hand(&mut e, 1);

    // ④ Cancel on `"dddquche_red_card"` does not mark `dddquche_used`.
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: quche_def });
    let c_qc_cancel = e.open_choice().expect("dddquche_red_card").clone();
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_qc_cancel.id,
            selection: Vec::new(),
        },
    );
    assert_eq!(flag(&e, 0, "dddquche_used"), 0);

    // ① & ③ Give `red_tao` to seat 1, then discard `blk_sha` to emit virtual `Sha` against seat 1.
    let hp1_before_qc = e.state_core().player(1).hp;
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: quche_def });
    let c_qc_red = e.open_choice().expect("dddquche_red_card").clone();
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_qc_red.id,
            selection: vec![ChoiceOption::Card(red_tao)],
        },
    );
    let c_qc_tgt = e.open_choice().expect("dddquche_give_target").clone();
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_qc_tgt.id,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    let c_qc_blk = e.open_choice().expect("dddquche_black_sha").clone();
    assert_eq!(c_qc_blk.spec.prompt, "dddquche_black_sha");
    assert_mid_choice_archive_and_ai(&e, &packs, vec![ChoiceOption::Card(blk_sha)]);
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_qc_blk.id,
            selection: vec![ChoiceOption::Card(blk_sha)],
        },
    );
    drain(&mut e);
    assert_eq!(e.state_core().player(1).hp, hp1_before_qc - 1);
    // ② Negative guard: limited skill `dddquche` cannot be activated again once `dddquche_used == 1`.
    put_in_hand(&mut e, 0, red_tao);
    assert!(matches!(
        submit(&mut e, 0, GameAction::ActivateSkill { skill: quche_def }),
        CommandOutcome::Rejected(_)
    ));

    // ---- (B4) `ddddongcha` 洞察 + `dddzhijie` 智诫 (`archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:2851-3067`) ----
    let xinxianying = find_general(&e, "ddd_xinxianying");
    e.assign_general(0, xinxianying).expect("assign ddd_xinxianying");
    let zhijie_def = e.catalog().skill_by_name("dddzhijie").expect("dddzhijie");

    // ② Negative guard: `dddzhijie` rejected when `"ddd_jian"` pile has < 2 cards.
    e.debug_set_turn(0, Phase::Play, 4);
    assert!(matches!(
        submit(&mut e, 0, GameAction::ActivateSkill { skill: zhijie_def }),
        CommandOutcome::Rejected(_)
    ));

    // ① `ddddongcha` triggers on seat 1's `Phase::Prepare`: both seat 0 and seat 1 put 1 card into `"ddd_jian"`.
    clear_hand(&mut e, 0);
    clear_hand(&mut e, 1);
    put_in_hand(&mut e, 0, red_tao);
    put_in_hand(&mut e, 1, blk_sha);
    e.debug_set_turn(0, Phase::End, 4);
    let out_dc = submit(&mut e, 0, GameAction::NextPhase);
    let c_dc_own = match out_dc {
        CommandOutcome::Pending { choice, .. } => choice,
        other => panic!("expected Pending ddddongcha_owner_put, got {other:?}"),
    };
    assert_eq!(c_dc_own.spec.prompt, "ddddongcha_owner_put");
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_dc_own.id,
            selection: vec![ChoiceOption::Card(red_tao)],
        },
    );
    let c_dc_tgt = e.open_choice().expect("ddddongcha_target_put").clone();
    assert_eq!(c_dc_tgt.spec.prompt, "ddddongcha_target_put");
    assert_eq!(c_dc_tgt.observer, 1);
    let _ = submit(
        &mut e,
        1,
        GameAction::Answer {
            choice_id: c_dc_tgt.id,
            selection: vec![ChoiceOption::Card(blk_sha)],
        },
    );
    assert_eq!(
        e.state_core()
            .vm_aux
            .piles
            .get(&(0, "ddd_jian".to_string()))
            .map(Vec::len),
        Some(2)
    );

    // ③ Activate `dddzhijie` in seat 0's `Phase::Play`: discarding different-color pair sets `dddzhijie_viewas = 2` and draws 2.
    e.debug_set_turn(0, Phase::Play, 5);
    let out_zj = submit(&mut e, 0, GameAction::ActivateSkill { skill: zhijie_def });
    let c_zj = match out_zj {
        CommandOutcome::Pending { choice, .. } => choice,
        other => panic!("expected Pending dddzhijie_cards, got {other:?}"),
    };
    assert_eq!(c_zj.spec.prompt, "dddzhijie_cards");
    assert_mid_choice_archive_and_ai(&e, &packs, vec![ChoiceOption::Card(red_tao), ChoiceOption::Card(blk_sha)]);
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_zj.id,
            selection: vec![ChoiceOption::Card(red_tao), ChoiceOption::Card(blk_sha)],
        },
    );
    assert_eq!(flag(&e, 0, "dddzhijie_viewas"), 2);
    assert_eq!(e.state_core().player(0).hand.len(), 2);
}

/// 4. Batch C & D 代表技能（`dddqice`、`dddfenji`、`dddxiaheng`、`dddyouxue`、`dddduanbing`、
///    `dddxuanlun` + `dddlanghuai`、`dddbailei`）6D 真实对局检验。
#[test]
fn u01_batch_c_and_d_real_match_6d_verification() {
    let (d3_pack, p9_pack) = build_u01_packs();
    let packs = [&d3_pack, &p9_pack];
    let mut e = Engine::new_with_extensions(310104, 3, &packs).expect("U01 Batch C/D engine");

    // ---- (C1) `dddqice` 奇策 (`archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:3507-3568`) ----
    let liuye = find_general(&e, "ddd_liuye");
    e.assign_general(0, liuye).expect("assign ddd_liuye");
    e.debug_set_turn(2, Phase::End, 1);

    // ① Positive trigger on seat 0's `Phase::Prepare`: opens `"dddqice_confirm"`.
    let out_qc = submit(&mut e, 2, GameAction::NextPhase);
    let c_qc = match out_qc {
        CommandOutcome::Pending { choice, .. } => choice,
        other => panic!("expected Pending dddqice_confirm, got {other:?}"),
    };
    assert_eq!(c_qc.spec.prompt, "dddqice_confirm");
    assert_mid_choice_archive_and_ai(&e, &packs, vec![ChoiceOption::Confirm]);

    // ④ Pass/Cancel leaves `ddd_qice_disabled == 0`; Confirm draws 2 and sets `ddd_qice_disabled = 1`.
    let mut e_qc_pass =
        Engine::restore_with_extensions(&e.save(), &packs).expect("fork dddqice pass");
    let _ = submit(
        &mut e_qc_pass,
        0,
        GameAction::Answer {
            choice_id: c_qc.id,
            selection: vec![ChoiceOption::Pass],
        },
    );
    assert_eq!(flag(&e_qc_pass, 0, "ddd_qice_disabled"), 0);

    let h0_before = e.state_core().player(0).hand.len();
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_qc.id,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    assert_eq!(e.state_core().player(0).hand.len(), h0_before + 2);
    assert_eq!(flag(&e, 0, "ddd_qice_disabled"), 1);

    // ② Negative guard: while `ddd_qice_disabled == 1`, `Phase::Prepare` does NOT trigger `dddqice`.
    e.debug_set_turn(2, Phase::End, 2);
    let out_qc_blocked = submit(&mut e, 2, GameAction::NextPhase);
    assert!(matches!(out_qc_blocked, CommandOutcome::Accepted { .. }));
    assert!(e.open_choice().is_none());

    // ③ Playing a trick card (`Wuzhong`) re-enables `dddqice` (`ddd_qice_disabled = 0`).
    e.debug_set_turn(0, Phase::Play, 2);
    let wuzhong = find_card_by(&e, |_, d| d.kind == CardKind::Wuzhong);
    put_in_hand(&mut e, 0, wuzhong);
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: wuzhong,
            targets: Vec::new(),
        },
    );
    drain(&mut e);
    assert_eq!(flag(&e, 0, "ddd_qice_disabled"), 0);

    // ---- (C2) `dddfenji` 奋激 (`archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:3866-3943`) ----
    let zhaoang = find_general(&e, "ddd_zhaoang");
    e.assign_general(0, zhaoang).expect("assign ddd_zhaoang");
    clear_hand(&mut e, 1);
    e.debug_set_turn(0, Phase::Judge, 3);
    let out_fj = submit(&mut e, 0, GameAction::NextPhase);
    let c_fj_trick = match out_fj {
        CommandOutcome::Pending { choice, .. } => choice,
        other => panic!("expected Pending dddfenji_trick, got {other:?}"),
    };
    assert_eq!(c_fj_trick.spec.prompt, "dddfenji_trick");
    assert_eq!(
        c_fj_trick.spec.options,
        vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1), ChoiceOption::Tag(2)]
    );
    assert_mid_choice_archive_and_ai(&e, &packs, vec![ChoiceOption::Tag(0)]);

    let hp1_before_fj = e.state_core().player(1).hp;
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_fj_trick.id,
            selection: vec![ChoiceOption::Tag(0)],
        },
    );
    let c_fj_tgt = e.open_choice().expect("dddfenji_target").clone();
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_fj_tgt.id,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    drain(&mut e);
    assert_eq!(flag(&e, 0, "dddfenji_used_mask"), 1);
    assert_eq!(e.state_core().player(1).hp, hp1_before_fj - 1);

    // Next trigger on `Phase::Discard`: `Tag(0)` (Duel) is no longer available in `"dddfenji_trick"`.
    e.debug_set_turn(0, Phase::Play, 3);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let c_fj2 = e.open_choice().expect("dddfenji_trick on Discard").clone();
    assert_eq!(c_fj2.spec.prompt, "dddfenji_trick");
    assert_eq!(
        c_fj2.spec.options,
        vec![ChoiceOption::Tag(1), ChoiceOption::Tag(2)]
    );
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_fj2.id,
            selection: Vec::new(),
        },
    );
    drain(&mut e);

    // ---- (C3) `dddxiaheng` 狭横 (`archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:3944-4018`) ----
    let zhouchu = find_general(&e, "ddd_zhouchu");
    e.assign_general(0, zhouchu).expect("assign ddd_zhouchu");
    assert_eq!(e.state_core().player(0).max_hp, 5);
    put_in_hand(&mut e, 1, wuzhong);
    e.state_core_mut().player_mut(1).hp = 3;
    e.debug_set_turn(0, Phase::Draw, 4);
    let out_xh = submit(&mut e, 0, GameAction::NextPhase);
    let c_xh_t1 = match out_xh {
        CommandOutcome::Pending { choice, .. } => choice,
        other => panic!("expected Pending dddxiaheng_discard_target, got {other:?}"),
    };
    assert_eq!(c_xh_t1.spec.prompt, "dddxiaheng_discard_target");

    // ③ Choice A (`t1 = 1, t2 = 1`, neither is owner -> `max_hp` reduced from 5 to 4) vs Choice B (`t1 = 0, t2 = 1` -> `max_hp` stays 5).
    let mut e_xh_self =
        Engine::restore_with_extensions(&e.save(), &packs).expect("fork dddxiaheng self");
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_xh_t1.id,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    let c_xh_t2 = e.open_choice().expect("dddxiaheng_damage_target").clone();
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_xh_t2.id,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    drain(&mut e);
    assert_eq!(e.state_core().player(0).max_hp, 4);
    assert_eq!(flag(&e, 0, "dddxiaheng_same_target"), 1);

    let _ = submit(
        &mut e_xh_self,
        0,
        GameAction::Answer {
            choice_id: c_xh_t1.id,
            selection: vec![ChoiceOption::Player(0)],
        },
    );
    let c_xh_t2_self = e_xh_self
        .open_choice()
        .expect("dddxiaheng_damage_target self")
        .clone();
    let _ = submit(
        &mut e_xh_self,
        0,
        GameAction::Answer {
            choice_id: c_xh_t2_self.id,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    drain(&mut e_xh_self);
    assert_eq!(e_xh_self.state_core().player(0).max_hp, 5);
    assert_eq!(flag(&e_xh_self, 0, "dddxiaheng_same_target"), 0);

    // ---- (D1) `dddyouxue` 游学 (`archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:4975-5017`) ----
    let guanning = find_general(&e, "ddd_guanning");
    e.assign_general(0, guanning).expect("assign ddd_guanning");
    e.debug_set_turn(2, Phase::End, 4);
    let out_yx = submit(&mut e, 2, GameAction::NextPhase);
    let c_yx = match out_yx {
        CommandOutcome::Pending { choice, .. } => choice,
        other => panic!("expected Pending dddyouxue_target, got {other:?}"),
    };
    assert_eq!(c_yx.spec.prompt, "dddyouxue_target");
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_yx.id,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    assert_eq!(flag(&e, 0, "dddyouxue_holder"), 2);

    // Next Prepare: previous holder `Player(1)` is excluded from `"dddyouxue_target"`.
    e.debug_set_turn(2, Phase::End, 5);
    let _ = submit(&mut e, 2, GameAction::NextPhase);
    let c_yx2 = e.open_choice().expect("dddyouxue_target 2nd").clone();
    assert!(!c_yx2.spec.options.contains(&ChoiceOption::Player(1)));
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_yx2.id,
            selection: vec![ChoiceOption::Player(2)],
        },
    );
    assert_eq!(flag(&e, 0, "dddyouxue_holder"), 3);

    // ---- (D2) `dddduanbing` 短兵 (`archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:5131-5222`) ----
    let dingfeng = find_general(&e, "ddd_dingfeng");
    e.assign_general(0, dingfeng).expect("assign ddd_dingfeng");
    let duanbing_def = e.catalog().skill_by_name("dddduanbing").expect("dddduanbing");
    let blk_basic = find_card_by(&e, |_, d| {
        d.kind == CardKind::Sha && matches!(d.suit, Suit::Spade | Suit::Club)
    });
    clear_judge(&mut e, 0);
    clear_judge(&mut e, 1);
    clear_hand(&mut e, 1);
    e.state_core_mut().player_mut(1).hp = 3;
    put_in_hand(&mut e, 0, blk_basic);
    e.debug_set_turn(0, Phase::Play, 6);

    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: duanbing_def });
    let c_db_card = e.open_choice().expect("dddduanbing_card").clone();
    assert_eq!(c_db_card.spec.prompt, "dddduanbing_card");
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_db_card.id,
            selection: vec![ChoiceOption::Card(blk_basic)],
        },
    );
    let c_db_tgt = e.open_choice().expect("dddduanbing_target").clone();
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_db_tgt.id,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    // Virtual Sha deals 1 damage to seat 1 -> triggers `"dddduanbing_transfer"`.
    let c_db_xfer = e.open_choice().expect("dddduanbing_transfer").clone();
    assert_eq!(c_db_xfer.spec.prompt, "dddduanbing_transfer");
    assert_mid_choice_archive_and_ai(&e, &packs, vec![ChoiceOption::Confirm]);
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_db_xfer.id,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    drain(&mut e);
    assert!(e.state_core().player(0).judge.is_empty());
    assert!(e.state_core().player(1).judge.contains(&blk_basic));
    // ② Negative guard: cannot activate `dddduanbing` twice in the same turn.
    assert!(matches!(
        submit(&mut e, 0, GameAction::ActivateSkill { skill: duanbing_def }),
        CommandOutcome::Rejected(_)
    ));

    // ---- (D3) `dddxuanlun` 玄论 + `dddlanghuai` 朗怀 (`archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:5989-6235`) ----
    let xiahouxuan = find_general(&e, "ddd_xiahouxuan");
    e.assign_general(0, xiahouxuan).expect("assign ddd_xiahouxuan");
    clear_hand(&mut e, 0);
    clear_judge(&mut e, 0);
    e.state_core_mut().player_mut(0).hp = 3;
    let duel_xl = find_card_by(&e, |_, d| d.kind == CardKind::Duel);
    put_in_hand(&mut e, 1, duel_xl);
    e.debug_set_turn(1, Phase::Play, 7);

    let out_sha = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: duel_xl,
            targets: vec![0],
        },
    );
    let c_xl = match out_sha {
        CommandOutcome::Pending { choice, .. } => choice,
        other => panic!("expected Pending dddxuanlun_confirm, got {other:?}"),
    };
    assert_eq!(c_xl.spec.prompt, "dddxuanlun_confirm");
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_xl.id,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    assert_eq!(e.state_core().player(0).hand.len(), 4);
    assert_eq!(flag(&e, 0, "dddxuanlun_triggered"), 1);

    // At `Phase::End`, `dddxuanlun` opens `"dddxuanlun_end_mode"`: choose `Tag(1)` to set `dddxuanlun_del = 1`.
    e.debug_set_turn(1, Phase::Discard, 7);
    clear_hand(&mut e, 1);
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    let c_xl_end = e.open_choice().expect("dddxuanlun_end_mode").clone();
    assert_eq!(c_xl_end.spec.prompt, "dddxuanlun_end_mode");
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_xl_end.id,
            selection: vec![ChoiceOption::Tag(1)],
        },
    );
    assert_eq!(flag(&e, 0, "dddxuanlun_del"), 1);

    // At seat 0's `Phase::Draw`, `dddlanghuai` opens `"dddlanghuai_confirm"` with `allow_cancel == false`.
    clear_judge(&mut e, 0);
    e.debug_set_turn(0, Phase::Judge, 8);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let c_lh = e.open_choice().expect("dddlanghuai_confirm").clone();
    assert_eq!(c_lh.spec.prompt, "dddlanghuai_confirm");
    assert!(
        !c_lh.spec.allow_cancel,
        "dddxuanlun_del == 1 must force dddlanghuai_confirm (allow_cancel == false)"
    );
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_lh.id,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    assert_eq!(flag(&e, 0, "dddxuanlun_del"), 0);
    assert_eq!(flag(&e, 0, "dddlanghuai_state"), 1);

    // ---- (D4) `dddbailei` 败累 (`archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:6302-6365`) ----
    let zhangkai = find_general(&e, "ddd_zhangkai");
    e.assign_general(0, zhangkai).expect("assign ddd_zhangkai");
    e.state_core_mut().player_mut(1).hp = 1;
    e.state_core_mut().player_mut(2).hp = 2;
    put_in_hand(&mut e, 1, blk_basic);
    e.debug_set_turn(2, Phase::End, 8);

    let out_bl = submit(&mut e, 2, GameAction::NextPhase);
    let c_bl_tgt = match out_bl {
        CommandOutcome::Pending { choice, .. } => choice,
        other => panic!("expected Pending dddbailei_target, got {other:?}"),
    };
    assert_eq!(c_bl_tgt.spec.prompt, "dddbailei_target");
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_bl_tgt.id,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    let c_bl_mode = e.open_choice().expect("dddbailei_mode").clone();
    assert_eq!(c_bl_mode.spec.prompt, "dddbailei_mode");
    assert_mid_choice_archive_and_ai(&e, &packs, vec![ChoiceOption::Tag(0)]);

    // ③ Choice A (`Tag(0)`: gain 1 card, `dddbailei_lost == 0`) vs Choice B (`Tag(1)`: deal 1 damage to 1-HP target -> `dddbailei_lost = 1`).
    let mut e_bl_gain =
        Engine::restore_with_extensions(&e.save(), &packs).expect("fork dddbailei gain");
    let _ = submit(
        &mut e_bl_gain,
        0,
        GameAction::Answer {
            choice_id: c_bl_mode.id,
            selection: vec![ChoiceOption::Tag(0)],
        },
    );
    assert!(e_bl_gain.state_core().player(0).hand.contains(&blk_basic));
    assert_eq!(flag(&e_bl_gain, 0, "dddbailei_lost"), 0);

    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_bl_mode.id,
            selection: vec![ChoiceOption::Tag(1)],
        },
    );
    drain(&mut e);
    assert_eq!(flag(&e, 0, "dddbailei_lost"), 1);

    // ② Negative guard: once `dddbailei_lost == 1`, `dddbailei` no longer triggers on `Phase::Prepare`.
    e.debug_set_turn(2, Phase::End, 9);
    let out_bl_blocked = submit(&mut e, 2, GameAction::NextPhase);
    assert!(matches!(out_bl_blocked, CommandOutcome::Accepted { .. }));
    assert!(e.open_choice().is_none());
}
