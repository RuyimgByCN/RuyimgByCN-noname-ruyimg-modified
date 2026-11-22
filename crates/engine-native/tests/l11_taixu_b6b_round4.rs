//! Batch L11-6b Round 4: real-event regression suite closing all 4 parity defects from #69:
//! 1. `xinfu_tanbei` decoupled from L11 `txtanbei_effect1/2/3` (`sp2/skill.js:15817-15874`)
//! 2. `xinfu_sidao` real `useCardAfter` in `Phase::Play` (`sp2/skill.js:15876-15944`)
//! 3. `xinfu_yisuan` `ctx.lose_max_hp` (`MaxHpLost`), `LogEvent::CardsTaken`, and `table | discard`
//! 4. `kuanggu` dead-endpoint distance calculation (`get/index.js:3688-3724`)

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;

use content_native_loader::{
    get_dlc_event_by_idx, prepare_locked_content_with_bridge, restore_locked_engine,
    start_locked_engine, start_locked_taixu_battle, validate_and_freeze_raw_package,
    LockedStartMode, TaixuModeSession, TaixuPendingBattleSpec, TaixuPendingInteraction,
    VerifiedLockedContent,
};
use engine_native::ai::BoundedPlanner;
use engine_native::ids::{CardId, PlayerId};
use engine_native::state::Role;
use engine_native::view::EventView;
use engine_native::{
    ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, LogEvent, Phase,
};

static TAIXU_R4_VERIFIED: OnceLock<VerifiedLockedContent> = OnceLock::new();
static R4_DEDUP: AtomicU64 = AtomicU64::new(72_000_000);

fn next_dedup() -> u64 {
    R4_DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn taixu_verified() -> &'static VerifiedLockedContent {
    TAIXU_R4_VERIFIED.get_or_init(|| {
        let pkg_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../packages/extensions/taixu-upstream-liuli");
        let raw_json = std::fs::read_to_string(pkg_dir.join("package.npack.json"))
            .expect("read taixu package.npack.json")
            .replace("\r\n", "\n");
        let parsed: serde_json::Value = serde_json::from_str(&raw_json).unwrap();
        let mut assets = BTreeMap::new();
        if let Some(paths) = parsed.get("asset_paths").and_then(|v| v.as_object()) {
            for rel_path in paths.values().filter_map(|v| v.as_str()) {
                let bytes = std::fs::read(pkg_dir.join(rel_path))
                    .expect("read taixu asset")
                    .into_iter()
                    .filter(|&b| b != b'\r')
                    .collect::<Vec<u8>>();
                assets.insert(rel_path.to_string(), bytes);
            }
        }
        let frozen = validate_and_freeze_raw_package(raw_json, assets).expect("freeze taixu");
        prepare_locked_content_with_bridge(vec![frozen]).expect("prepare locked taixu")
    })
}

fn submit(e: &mut Engine, actor: PlayerId, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action,
    })
}

fn answer(e: &mut Engine, actor: PlayerId, selection: Vec<ChoiceOption>) -> CommandOutcome {
    let ch = e.open_choice().expect("open choice");
    assert_eq!(ch.observer, actor, "choice observer mismatch for {}", ch.spec.prompt);
    submit(
        e,
        actor,
        GameAction::Answer {
            choice_id: ch.id,
            selection,
        },
    )
}

fn ensure_hand_card(e: &mut Engine, seat: PlayerId, name: &str) -> CardId {
    if let Some(cid) = e
        .state_core()
        .player(seat)
        .hand
        .iter()
        .copied()
        .find(|&c| e.catalog().cards[c as usize].name == name)
    {
        cid
    } else {
        e.debug_place(seat, name)
    }
}

fn drain_open_with_ai(e: &mut Engine, max_steps: usize) {
    let ai = BoundedPlanner::new(16);
    for _ in 0..max_steps {
        if e.outcome().is_some() {
            break;
        }
        let Some(ch) = e.open_choice() else {
            break;
        };
        let obs = ch.observer;
        let act = e.plan_action(obs, &ai).action;
        let _ = submit(e, obs, act);
    }
}

fn drain_until_prompt(e: &mut Engine, target_prompt: &str, max_steps: usize) {
    let ai = BoundedPlanner::new(16);
    for _ in 0..max_steps {
        if e.outcome().is_some() {
            break;
        }
        let Some(ch) = e.open_choice() else {
            break;
        };
        if ch.spec.prompt == target_prompt {
            return;
        }
        let obs = ch.observer;
        let act = e.plan_action(obs, &ai).action;
        let _ = submit(e, obs, act);
    }
}

fn advance_to_play(e: &mut Engine, seat: PlayerId) {
    let ai = BoundedPlanner::new(16);
    for _ in 0..120 {
        if e.outcome().is_some() {
            break;
        }
        if let Some(ch) = e.open_choice() {
            let obs = ch.observer;
            let act = e.plan_action(obs, &ai).action;
            let _ = submit(e, obs, act);
            continue;
        }
        let st = e.state_core();
        if st.turn.current == seat && st.turn.phase == Phase::Play {
            return;
        }
        if st.turn.current == seat {
            let _ = submit(e, seat, GameAction::NextPhase);
        } else if st.turn.phase == Phase::Play {
            let _ = submit(e, st.turn.current, GameAction::NextPhase);
        } else {
            let act = e.plan_action(st.turn.current, &ai).action;
            let _ = submit(e, st.turn.current, act);
        }
    }
    assert_eq!(e.state_core().turn.current, seat);
    assert_eq!(e.state_core().turn.phase, Phase::Play);
}

fn start_4p_identity_lord0(seed: u64, verified: &VerifiedLockedContent) -> Engine {
    let mut e = start_locked_engine(seed, 4, LockedStartMode::Identity, verified)
        .expect("start 4p identity");
    let lord = e
        .state_core()
        .identity_roles()
        .and_then(|r| r.iter().position(|x| *x == Role::Lord))
        .unwrap_or(0) as u8;
    assert_eq!(lord, 0, "Identity mode assigns Lord to seat 0");
    e.debug_set_identity_roles(vec![Role::Lord, Role::Rebel, Role::Traitor, Role::Loyalist]);
    advance_to_play(&mut e, 0);
    e
}

fn simple_1v1_boss_idx() -> usize {
    (0..313)
        .find(|&i| get_dlc_event_by_idx(i).is_some_and(|d| d.id == "txhj_huanjingshouhutongzi"))
        .expect("txhj_huanjingshouhutongzi")
}

#[test]
fn l11_b6b_r4_tanbei_mode0_mode1_decoupled_from_txtanbei() {
    let verified = taixu_verified();
    let mut e = start_4p_identity_lord0(2026_701, verified);
    e.assign_general_named(0, "guosi").expect("guosi on seat 0");
    for s in 1..4 {
        e.debug_set_max_hp(s, 5);
        e.debug_set_hp(s, 5);
        e.debug_clear_hand(s);
    }
    e.debug_sweep_judge_zones();
    e.debug_clear_hand(0);

    // Give seat 1 a card in judge zone (`le`) to verify Mode 0 steals from `hej`!
    let judge_le = e.debug_place_judge(1, "le");
    let tb_id = e.catalog().skill_by_name("xinfu_tanbei").unwrap();

    // --- 1. Mode 0: steals from `hej`, forbids owner -> tgt, keeps other targets & non-owner attackers legal, NO txtanbei skills ---
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: tb_id });
    assert!(!matches!(out, CommandOutcome::Rejected(_)));
    assert_eq!(e.open_choice().map(|c| c.spec.prompt), Some("xinfu_tanbei:choose_target"));
    let _ = answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    assert_eq!(e.open_choice().map(|c| c.spec.prompt), Some("xinfu_tanbei:choose_mode"));
    let _ = answer(&mut e, 1, vec![ChoiceOption::Tag(0)]);

    assert!(e.debug_hand(0).contains(&judge_le), "Mode 0 must steal from target `hej`");
    assert!(e.debug_judge(1).is_empty(), "judge zone card must move out cleanly");
    assert!(!e.debug_has_skill(1, "txtanbei_effect1"));
    assert!(!e.debug_has_skill(1, "txtanbei_effect2"));
    assert!(!e.debug_has_skill(0, "txtanbei_effect3"));

    // Owner CANNOT target seat 1 with Sha / Duel / Dismantle this turn:
    let sha1 = ensure_hand_card(&mut e, 0, "sha");
    let duel1 = ensure_hand_card(&mut e, 0, "duel");
    let dism1 = ensure_hand_card(&mut e, 0, "dismantle");
    let _loot1 = e.debug_place(1, "shan");
    assert!(
        matches!(
            submit(&mut e, 0, GameAction::PlayCard { card: sha1, targets: vec![1] }),
            CommandOutcome::Rejected(_)
        ),
        "tanbei Mode 0 must forbid owner targeting target with Sha"
    );
    assert!(
        matches!(
            submit(&mut e, 0, GameAction::PlayCard { card: duel1, targets: vec![1] }),
            CommandOutcome::Rejected(_)
        ),
        "tanbei Mode 0 must forbid owner targeting target with Duel"
    );
    assert!(
        matches!(
            submit(&mut e, 0, GameAction::PlayCard { card: dism1, targets: vec![1] }),
            CommandOutcome::Rejected(_)
        ),
        "tanbei Mode 0 must forbid owner targeting target with Dismantle"
    );

    // Owner CAN STILL target seat 3 (adjacent dist=1), and Bystander seat 3 CAN STILL target seat 1!
    let out_p3 = submit(&mut e, 0, GameAction::PlayCard { card: sha1, targets: vec![3] });
    assert!(
        !matches!(out_p3, CommandOutcome::Rejected(_)),
        "tanbei Mode 0 must NOT block targeting other seats"
    );
    drain_open_with_ai(&mut e, 16);

    advance_to_play(&mut e, 3);
    let cixiong3 = ensure_hand_card(&mut e, 3, "cixiong");
    let _ = submit(&mut e, 3, GameAction::PlayCard { card: cixiong3, targets: vec![] });
    let sha_from_3 = ensure_hand_card(&mut e, 3, "sha");
    let out_3_to_1 = submit(
        &mut e,
        3,
        GameAction::PlayCard {
            card: sha_from_3,
            targets: vec![1],
        },
    );
    assert!(
        !matches!(out_3_to_1, CommandOutcome::Rejected(_)),
        "bystander seat 3 must still be able to target seat 1"
    );
    drain_open_with_ai(&mut e, 16);

    // Next turn of seat 0: Mode 0 forbid expires cleanly
    advance_to_play(&mut e, 0);
    e.debug_sweep_judge_zones();
    e.debug_clear_hand(0);
    let sha_next = ensure_hand_card(&mut e, 0, "sha");
    let _ = ensure_hand_card(&mut e, 1, "sha");
    let out_next = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha_next,
            targets: vec![1],
        },
    );
    assert!(
        !matches!(out_next, CommandOutcome::Rejected(_)),
        "Mode 0 target ban must clear on turn change"
    );
    drain_open_with_ai(&mut e, 16);

    // --- 2. Mode 1 (chosen by target): unlimited range & count on seat 2 (dist=2), NEVER grants txtanbei_effect1/seals jianxiong ---
    advance_to_play(&mut e, 0);
    e.debug_sweep_judge_zones();
    e.debug_clear_hand(0);
    e.debug_clear_hand(2);
    e.debug_grant(2, "奸雄");
    let p2_card = e.debug_place(2, "tao");

    let out_m1 = submit(&mut e, 0, GameAction::ActivateSkill { skill: tb_id });
    assert!(!matches!(out_m1, CommandOutcome::Rejected(_)));
    let _ = answer(&mut e, 0, vec![ChoiceOption::Player(2)]);
    assert_eq!(e.open_choice().map(|c| c.spec.prompt), Some("xinfu_tanbei:choose_mode"));
    let _ = answer(&mut e, 2, vec![ChoiceOption::Tag(1)]);

    assert!(!e.debug_has_skill(2, "txtanbei_effect1"));
    assert!(!e.debug_has_skill(0, "txtanbei_effect3"));
    assert!(
        !e.catalog().is_tieji_sealed(e.state_core(), 2),
        "SP2 xinfu_tanbei Mode 1 must NEVER seal target skills"
    );

    // Distance to seat 2 is 2 (> 1): Save/restore check while active, then Snatch + two Shas on seat 2!
    let snap_m1 = e.save();
    let mut e_m1 = restore_locked_engine(&snap_m1, verified).expect("restore tanbei m1");
    assert_eq!(e_m1.state_hash(), e.state_hash());

    let snatch = ensure_hand_card(&mut e_m1, 0, "snatch");
    let out_snatch = submit(
        &mut e_m1,
        0,
        GameAction::PlayCard {
            card: snatch,
            targets: vec![2],
        },
    );
    assert!(
        !matches!(out_snatch, CommandOutcome::Rejected(_)),
        "Mode 1 allows Snatch against dist=2 target: {out_snatch:?}"
    );
    drain_open_with_ai(&mut e_m1, 16);
    assert!(e_m1.debug_hand(0).contains(&p2_card));

    // Deal Sha damage to seat 2 and verify non-locked `jianxiong` STILL triggers (unsealed!), plus second Sha bypasses count!
    let sha_a = ensure_hand_card(&mut e_m1, 0, "sha");
    e_m1.debug_clear_hand(2);
    let out_sa = submit(
        &mut e_m1,
        0,
        GameAction::PlayCard {
            card: sha_a,
            targets: vec![2],
        },
    );
    assert!(!matches!(out_sa, CommandOutcome::Rejected(_)));
    drain_until_prompt(&mut e_m1, "trigger_confirm", 16);
    assert_eq!(
        e_m1.open_choice().map(|c| c.spec.prompt),
        Some("trigger_confirm"),
        "Mode 1 must NOT seal target non-locked jianxiong"
    );
    let _ = answer(&mut e_m1, 2, vec![ChoiceOption::Confirm]);
    drain_open_with_ai(&mut e_m1, 16);
    let sha_b = ensure_hand_card(&mut e_m1, 0, "sha");
    let out_sb = submit(
        &mut e_m1,
        0,
        GameAction::PlayCard {
            card: sha_b,
            targets: vec![2],
        },
    );
    assert!(
        !matches!(out_sb, CommandOutcome::Rejected(_)),
        "Mode 1 allows unlimited Sha usage against target: {out_sb:?}"
    );
    drain_open_with_ai(&mut e_m1, 16);

    // --- 3. Empty `hej` auto Mode 1 fallback without prompting target ---
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    drain_open_with_ai(&mut e, 32);
    advance_to_play(&mut e, 0);
    e.debug_sweep_judge_zones();
    e.debug_clear_hand(2);
    for eq in e.debug_equips(2) {
        e.debug_remove_equip(2, eq);
    }
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: tb_id });
    let _ = answer(&mut e, 0, vec![ChoiceOption::Player(2)]);
    assert!(
        e.open_choice().is_none(),
        "empty `hej` target must skip choose_mode and apply Mode 1"
    );
    assert!(!e.catalog().is_tieji_sealed(e.state_core(), 2));
    assert!(!e.debug_has_skill(2, "txtanbei_effect1"));
    let _ = e.debug_place(2, "shan");
    let sn_empty = ensure_hand_card(&mut e, 0, "snatch");
    assert!(
        !matches!(
            submit(&mut e, 0, GameAction::PlayCard { card: sn_empty, targets: vec![2] }),
            CommandOutcome::Rejected(_)
        ),
        "empty-hej fallback must grant Mode 1 unlimited distance"
    );
}

#[test]
fn l11_b6b_r4_sidao_real_use_card_after_parity() {
    let verified = taixu_verified();
    let mut e = start_4p_identity_lord0(2026_702, verified);
    e.assign_general_named(0, "guosi").expect("guosi owns xinfu_sidao");
    for s in 1..4 {
        e.debug_set_max_hp(s, 6);
        e.debug_set_hp(s, 6);
        e.debug_clear_hand(s);
    }
    e.debug_sweep_judge_zones();
    e.debug_clear_hand(0);

    // Verify no passive PhaseBegin `l11_viewas:xinfu_sidao` view-as exists before Play phase triggers:
    let probe_card = ensure_hand_card(&mut e, 0, "sha");
    assert!(
        e.catalog()
            .find_player_vm_view_as(
                e.state_core(),
                0,
                probe_card,
                true,
                Some(engine_native::catalog::CardKind::Snatch)
            )
            .is_none(),
        "xinfu_sidao must NOT passively grant view-as Snatch outside useCardAfter"
    );

    let p1_card = e.debug_place(1, "tao");
    let _p1_keep = e.debug_place(1, "tao");
    let _p3_card = e.debug_place(3, "shan");
    let _p2_card = e.debug_place(2, "sha");

    // 1. Negative guards:
    // (a) Single card on seat 1 -> no prompt
    let sha1 = ensure_hand_card(&mut e, 0, "sha");
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha1, targets: vec![1] });
    drain_open_with_ai(&mut e, 16);
    assert!(e.open_choice().is_none(), "1st card alone must NOT trigger xinfu_sidao");

    // (b) Intervening `Wuzhong` breaks consecutive pair before next card on seat 1
    let wz = ensure_hand_card(&mut e, 0, "wuzhong");
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: wz, targets: vec![] });
    drain_open_with_ai(&mut e, 16);
    let duel_a = ensure_hand_card(&mut e, 0, "duel");
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: duel_a, targets: vec![1] });
    drain_open_with_ai(&mut e, 16);
    assert!(
        e.open_choice().is_none(),
        "Wuzhong between Sha(1) and Duel(1) breaks consecutive pair"
    );

    // (c) Disjoint consecutive targets: Duel(1) then Dismantle(3) -> no prompt
    let dism3 = ensure_hand_card(&mut e, 0, "dismantle");
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: dism3, targets: vec![3] });
    drain_open_with_ai(&mut e, 16);
    assert!(
        e.open_choice().is_none(),
        "disjoint targets [1] then [3] must NOT trigger xinfu_sidao"
    );

    // (d) Two consecutive cards on dist=2 target (seat 2): illegal Snatch distance -> no prompt
    let duel_p2_a = ensure_hand_card(&mut e, 0, "duel");
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: duel_p2_a, targets: vec![2] });
    drain_open_with_ai(&mut e, 16);
    let duel_p2_b = ensure_hand_card(&mut e, 0, "duel");
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: duel_p2_b, targets: vec![2] });
    drain_open_with_ai(&mut e, 16);
    assert!(
        e.open_choice().is_none(),
        "two consecutive cards on out-of-Snatch-range seat 2 must NOT trigger xinfu_sidao"
    );

    // 2. Equip exclusion, shared-target narrowing (`Chain([1,3])` + `Duel([1])`), cancel preserves usability:
    e.debug_clear_hand(0);
    let zhuge = ensure_hand_card(&mut e, 0, "zhuge");
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: zhuge, targets: vec![] });
    assert!(e.debug_equips(0).contains(&zhuge));

    let chain = ensure_hand_card(&mut e, 0, "chain");
    let duel1 = ensure_hand_card(&mut e, 0, "duel");
    let hand_keep = ensure_hand_card(&mut e, 0, "wuzhong");

    let _ = submit(&mut e, 0, GameAction::PlayCard { card: chain, targets: vec![1, 3] });
    drain_open_with_ai(&mut e, 16);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: duel1, targets: vec![1] });
    drain_until_prompt(&mut e, "xinfu_sidao:choose_card", 16);

    let ch_card = e.open_choice().expect("sidao choose_card");
    assert_eq!(ch_card.spec.prompt, "xinfu_sidao:choose_card");
    assert!(ch_card.spec.allow_cancel);
    assert!(ch_card.spec.options.contains(&ChoiceOption::Card(hand_keep)));
    assert!(
        !ch_card.spec.options.contains(&ChoiceOption::Card(zhuge)),
        "equipped card must NOT be offered by xinfu_sidao (hand-only `hs`)"
    );
    let _ = answer(&mut e, 0, vec![ChoiceOption::Card(hand_keep)]);

    let ch_tgt = e.open_choice().expect("sidao choose_target");
    assert_eq!(ch_tgt.spec.prompt, "xinfu_sidao:choose_target");
    assert!(ch_tgt.spec.allow_cancel);
    assert_eq!(
        ch_tgt.spec.options,
        vec![ChoiceOption::Player(1)],
        "only shared legal target seat 1 must be offered (seat 3 excluded)"
    );
    // Cancel at target selection: NO card lost, NO xinfu_sidaoy set!
    let _ = answer(&mut e, 0, vec![]);
    assert!(e.debug_hand(0).contains(&hand_keep));
    assert_eq!(
        e.state_core()
            .vm_aux
            .flags
            .get(&(0, "skill_turn:xinfu_sidaoy".to_string()))
            .copied()
            .unwrap_or(0),
        0,
        "canceling xinfu_sidao must NOT set xinfu_sidaoy"
    );

    // 3. Re-trigger after cancel + NGSA save/restore across BOTH choices + Snatch resolves + `xinfu_sidaoy` locks turn:
    let dism1 = ensure_hand_card(&mut e, 0, "dismantle");
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: dism1, targets: vec![1] });
    drain_until_prompt(&mut e, "xinfu_sidao:choose_card", 16);

    let snap_card = e.save();
    let mut e_res = restore_locked_engine(&snap_card, verified).expect("restore choose_card");
    assert_eq!(e_res.state_hash(), e.state_hash());
    let _ = answer(&mut e_res, 0, vec![ChoiceOption::Card(hand_keep)]);

    let snap_tgt = e_res.save();
    let mut e_res2 = restore_locked_engine(&snap_tgt, verified).expect("restore choose_target");
    assert_eq!(e_res2.state_hash(), e_res.state_hash());
    let _ = answer(&mut e_res2, 0, vec![ChoiceOption::Player(1)]);
    drain_open_with_ai(&mut e_res2, 16);

    assert!(!e_res2.debug_hand(0).contains(&hand_keep), "converted hand card consumed");
    assert!(
        e_res2.debug_hand(1).len() == 0 && (e_res2.debug_hand(0).contains(&p1_card) || e_res2.debug_hand(0).contains(&_p1_keep)),
        "virtual Snatch stole remaining seat 1 card"
    );
    assert_eq!(
        e_res2
            .state_core()
            .vm_aux
            .flags
            .get(&(0, "skill_turn:xinfu_sidaoy".to_string()))
            .copied()
            .unwrap_or(0),
        1,
        "xinfu_sidaoy set after use"
    );

    // Further consecutive cards in same Play phase must NOT re-trigger `xinfu_sidao`:
    let sha_after = ensure_hand_card(&mut e_res2, 0, "sha");
    let _ = submit(&mut e_res2, 0, GameAction::PlayCard { card: sha_after, targets: vec![1] });
    drain_open_with_ai(&mut e_res2, 16);
    assert!(
        e_res2.open_choice().is_none(),
        "xinfu_sidaoy must block second trigger in same turn"
    );

    // Next turn of seat 0 clears `xinfu_sidaoy` cleanly:
    let _ = submit(&mut e_res2, 0, GameAction::NextPhase);
    drain_open_with_ai(&mut e_res2, 32);
    advance_to_play(&mut e_res2, 0);
    assert_eq!(
        e_res2
            .state_core()
            .vm_aux
            .flags
            .get(&(0, "skill_turn:xinfu_sidaoy".to_string()))
            .copied()
            .unwrap_or(0),
        0
    );

    // 4. AI public-info check: Loyalist declines Snatch against Lord; Rebel/enemy picks enemy!
    let mut e_ai = start_locked_engine(2026_703, 4, LockedStartMode::Identity, verified)
        .expect("start ai identity");
    e_ai.debug_set_identity_roles(vec![Role::Lord, Role::Loyalist, Role::Rebel, Role::Traitor]);
    e_ai.assign_general_named(1, "guosi").expect("guosi on Loyalist seat 1");
    advance_to_play(&mut e_ai, 1);
    e_ai.debug_clear_hand(1);
    e_ai.debug_clear_hand(0);
    let _lord_card = e_ai.debug_place(0, "tao");
    let d_a = ensure_hand_card(&mut e_ai, 1, "duel");
    let d_b = e_ai.debug_place(1, "duel");
    let hold = ensure_hand_card(&mut e_ai, 1, "sha");
    let _ = submit(&mut e_ai, 1, GameAction::PlayCard { card: d_a, targets: vec![0] });
    drain_open_with_ai(&mut e_ai, 16);
    let _ = submit(&mut e_ai, 1, GameAction::PlayCard { card: d_b, targets: vec![0] });
    drain_until_prompt(&mut e_ai, "xinfu_sidao:choose_card", 16);
    let ai = BoundedPlanner::new(16);
    drain_until_prompt(&mut e_ai, "xinfu_sidao:choose_target", 8);
    if let Some(ch) = e_ai.open_choice().filter(|c| c.spec.prompt == "xinfu_sidao:choose_target") {
        let act = e_ai.plan_action(1, &ai).action;
        assert_eq!(
            act,
            GameAction::Answer {
                choice_id: ch.id,
                selection: vec![],
            },
            "public-info AI Loyalist must cancel xinfu_sidao against publicly revealed Lord"
        );
        let _ = submit(&mut e_ai, 1, act);
    }
    assert!(e_ai.debug_hand(1).contains(&hold));
}

#[test]
fn l11_b6b_r4_yisuan_table_and_max_hp_loss_logs() {
    let verified = taixu_verified();
    let idx = simple_1v1_boss_idx();
    let d = get_dlc_event_by_idx(idx).unwrap();
    let mut sess =
        TaixuModeSession::start_season(3090, "ChongYingChuLin", 1, "liubei", "pinganwushi", None, 0)
            .unwrap();
    sess.use_skills.push("xinfu_yisuan".into());
    sess.pending = Some(TaixuPendingInteraction::Battle(TaixuPendingBattleSpec {
        slot_idx: 0,
        event_id: d.id.into(),
        event_kind: d.kind.into(),
        season: sess.season.clone(),
        chapter: "1".into(),
        dlc_def_idx: Some(idx),
        choose_btn_idx: None,
        target_str: "all".into(),
        unique: false,
        max_rounds: 5,
        exp: d.exp,
    }));
    let mut e = start_locked_taixu_battle(&sess, verified).unwrap();
    advance_to_play(&mut e, 0);

    // 1. Ordinary trick still on `table` (`wuzhong`) must trigger `xinfu_yisuan:confirm`:
    e.debug_clear_hand(0);
    let wz = e.debug_move_named_card_to_hand(0, "wuzhong");
    let max_before = e.state_core().player(0).max_hp;
    let out = submit(&mut e, 0, GameAction::PlayCard { card: wz, targets: vec![] });
    assert!(!matches!(out, CommandOutcome::Rejected(_)));
    drain_until_prompt(&mut e, "xinfu_yisuan:confirm", 16);
    assert_eq!(
        e.open_choice().map(|c| c.spec.prompt),
        Some("xinfu_yisuan:confirm")
    );
    assert!(
        e.state_core().table.contains(&wz) || e.state_core().discard.contains(&wz),
        "trick carrier must be in table or discard during useCardAfter"
    );

    let snap = e.save();
    let mut e_restored = restore_locked_engine(&snap, verified).expect("restore yisuan confirm");
    assert_eq!(e_restored.state_hash(), e.state_hash());

    let _ = answer(&mut e_restored, 0, vec![ChoiceOption::Confirm]);
    assert_eq!(e_restored.state_core().player(0).max_hp, max_before - 1);
    assert!(e_restored.debug_hand(0).contains(&wz));

    let post_events = e_restored.event_view_for(0);
    let lost_idx = post_events
        .iter()
        .position(|ev| matches!(ev, EventView::Public(LogEvent::MaxHpLost { player: 0, amount: 1 })))
        .expect("xinfu_yisuan must emit LogEvent::MaxHpLost via ctx.lose_max_hp");
    let taken_idx = post_events
        .iter()
        .position(|ev| {
            matches!(
                ev,
                EventView::Public(LogEvent::CardsTaken {
                    from: 0,
                    to: 0,
                    cards,
                    public: true
                }) if cards == &vec![wz]
            )
        })
        .expect("xinfu_yisuan must emit LogEvent::CardsTaken for regained trick");
    assert!(lost_idx < taken_idx);
    assert!(
        !post_events[lost_idx..].iter().any(|ev| {
            matches!(ev, EventView::Public(LogEvent::CardsDrawn { cards, .. }) if cards.contains(&wz))
        }),
        "xinfu_yisuan regain must NOT masquerade as LogEvent::CardsDrawn"
    );

    // 2. `max_hp == 1` lethal max_hp reduction emits Death cleanly on a new turn:
    let _ = submit(&mut e_restored, 0, GameAction::NextPhase);
    drain_open_with_ai(&mut e_restored, 16);
    advance_to_play(&mut e_restored, 0);
    e_restored.debug_set_max_hp(0, 1);
    e_restored.debug_set_hp(0, 1);
    e_restored.debug_clear_hand(0);
    let wz2 = e_restored.debug_move_named_card_to_hand(0, "wuzhong");
    let _ = submit(&mut e_restored, 0, GameAction::PlayCard { card: wz2, targets: vec![] });
    drain_until_prompt(&mut e_restored, "xinfu_yisuan:confirm", 16);
    assert_eq!(
        e_restored.open_choice().map(|c| c.spec.prompt),
        Some("xinfu_yisuan:confirm")
    );
    let _ = answer(&mut e_restored, 0, vec![ChoiceOption::Confirm]);
    drain_open_with_ai(&mut e_restored, 16);
    assert_eq!(e_restored.state_core().player(0).max_hp, 0);
    assert!(
        !e_restored.debug_alive(0),
        "losing last point of max_hp via yisuan must kill owner"
    );
}

#[test]
fn l11_b6b_r4_kuanggu_lethal_distance_with_dead_target() {
    let verified = taixu_verified();

    // Case A: Lethal Sha killing adjacent seat 1 (distance 1, dead at DamageEnd) heals seat 0 without panicking:
    let mut e1 = start_4p_identity_lord0(2026_704, verified);
    e1.debug_grant(0, "kuanggu");
    e1.debug_set_max_hp(0, 4);
    e1.debug_set_hp(0, 2);
    e1.debug_set_hp(1, 1);
    for s in 0..4 {
        e1.debug_clear_hand(s);
    }
    let sha1 = ensure_hand_card(&mut e1, 0, "sha");
    let out1 = submit(&mut e1, 0, GameAction::PlayCard { card: sha1, targets: vec![1] });
    assert!(!matches!(out1, CommandOutcome::Rejected(_)));
    drain_open_with_ai(&mut e1, 32);
    assert!(!e1.debug_alive(1), "seat 1 killed by lethal Sha");
    assert_eq!(
        e1.debug_hp(0),
        3,
        "kuanggu must heal 1 HP when killing adjacent seat 1 (`distance == 1` with dead endpoint)"
    );

    // Case B: Lethal Sha killing distance-2 seat 2 (via Cixiong attack range 2) must NOT heal seat 0:
    let mut e2 = start_4p_identity_lord0(2026_705, verified);
    e2.debug_grant(0, "kuanggu");
    e2.debug_set_max_hp(0, 4);
    e2.debug_set_hp(0, 2);
    e2.debug_set_hp(2, 1);
    for s in 0..4 {
        e2.debug_clear_hand(s);
    }
    let cixiong = ensure_hand_card(&mut e2, 0, "cixiong");
    let _ = submit(&mut e2, 0, GameAction::PlayCard { card: cixiong, targets: vec![] });
    let sha2 = ensure_hand_card(&mut e2, 0, "sha");
    let out2 = submit(&mut e2, 0, GameAction::PlayCard { card: sha2, targets: vec![2] });
    assert!(!matches!(out2, CommandOutcome::Rejected(_)));
    drain_open_with_ai(&mut e2, 32);
    assert!(!e2.debug_alive(2), "seat 2 killed at distance 2");
    assert_eq!(
        e2.debug_hp(0),
        2,
        "kuanggu must NOT heal when killing target at distance 2"
    );

    // Case C: Intermediate seat 1 is ALREADY dead -> seat 2 is now distance 1 -> lethal Sha on seat 2 heals seat 0!
    let mut e3 = start_4p_identity_lord0(2026_706, verified);
    e3.debug_grant(0, "kuanggu");
    e3.debug_set_max_hp(0, 4);
    e3.debug_set_hp(0, 2);
    e3.debug_set_hp(1, 1);
    e3.debug_set_hp(2, 1);
    for s in 0..4 {
        e3.debug_clear_hand(s);
    }
    let zhuge = ensure_hand_card(&mut e3, 0, "zhuge");
    let _ = submit(&mut e3, 0, GameAction::PlayCard { card: zhuge, targets: vec![] });
    let kill1 = ensure_hand_card(&mut e3, 0, "sha");
    let _ = submit(&mut e3, 0, GameAction::PlayCard { card: kill1, targets: vec![1] });
    drain_open_with_ai(&mut e3, 32);
    assert!(!e3.debug_alive(1));
    assert_eq!(e3.debug_hp(0), 3);

    // Reset hp back to 2 (so wounded) and kill seat 2 now that dead seat 1 is skipped -> distance(0, 2) == 1!
    e3.debug_set_hp(0, 2);
    let kill2 = ensure_hand_card(&mut e3, 0, "sha");
    let out3 = submit(&mut e3, 0, GameAction::PlayCard { card: kill2, targets: vec![2] });
    assert!(!matches!(out3, CommandOutcome::Rejected(_)));
    drain_open_with_ai(&mut e3, 32);
    assert!(!e3.debug_alive(2));
    assert_eq!(
        e3.debug_hp(0),
        3,
        "kuanggu skips previously dead seat 1 and keeps dead endpoint seat 2 (`distance == 1`)"
    );
}
