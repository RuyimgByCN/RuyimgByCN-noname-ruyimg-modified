//! N01: OL界朱然「胆守」四选项、每阶段一次、结算后弃置序号张牌。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DANSHOU_SRC: &str =
    include_str!("../../../../content/original/ported/onlyOL/ol_jie_danshou.skill");

fn setup() -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("ol-danshou-n01", DANSHOU_SRC).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 2_026_102_501, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&compiled)
        .unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let other = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(3_750_129_445),
    });
    effects.push(EffectRequest::DrawCards { player: owner, count: 5 });
    effects.push(EffectRequest::DrawCards { player: other, count: 3 });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, other)
}

fn skill_instance(state: &GameState, actor: PlayerId) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| {
            skill.owner == actor && skill.def == engine_api::SkillDefId::new(3_750_129_445)
        })
        .expect("danshou skill instance")
        .id
}

fn start(state: GameState, actor: PlayerId) -> (GameState, Vec<u32>) {
    let skill = skill_instance(&state, actor);
    let mut tx = Transaction::begin(state);
    let _ = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveOlJieDanshou {
            player: actor,
            skill,
        }],
    );
    let committed = tx.commit();
    assert!(committed.state.has_pending_choice());
    let options = committed
        .output
        .choice
        .as_ref()
        .map(|choice| choice.options.iter().map(|option| option.token).collect())
        .unwrap_or_default();
    (committed.state, options)
}

fn choose_result(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
) -> (GameState, CommandResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (state, outcome) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(9),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    );
    (state, outcome.result)
}

fn choose(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
) -> GameState {
    let (state, result) = choose_result(state, actor, selections, cancelled);
    assert!(matches!(result, CommandResult::Accepted { .. }), "{result:?}");
    state
}

#[test]
fn danshou_damage_then_discards_option_number() {
    let (state, owner, other) = setup();
    let hp = state.players.get(other).unwrap().hp;
    let hand_before = state.hand_count(owner);
    let (state, options) = start(state, owner);
    assert_eq!(options, vec![1, 2, 3, 4]);

    let state = choose(state, owner, vec![3], false);
    assert!(state.has_pending_choice(), "option 3 asks for a target");
    let state = choose(state, owner, vec![other.raw()], false);
    assert_eq!(state.players.get(other).unwrap().hp, hp - 1);
    assert!(state.has_pending_choice(), "option 3 asks for one discard");
    let own = state.zones.get(state.zones.find(engine_core::state::ZoneKind::Hand, owner).unwrap()).unwrap().cards.clone();
    let state = choose(state, owner, vec![own[0].raw()], false);
    assert!(!state.has_pending_choice());
    assert_eq!(
        state.hand_count(owner),
        hand_before - 1,
        "option 3 discards exactly one card"
    );

    let (state, options) = start(state, owner);
    assert_eq!(
        options,
        vec![1, 2, 4],
        "option 3 is used once per play phase"
    );
}

#[test]
fn danshou_draw_option_draws_both_and_discards_two() {
    let (state, owner, other) = setup();
    let owner_before = state.hand_count(owner);
    let other_before = state.hand_count(other);
    let (state, _) = start(state, owner);
    let state = choose(state, owner, vec![4], false);
    let state = choose(state, owner, vec![other.raw()], false);
    assert_eq!(state.hand_count(owner), owner_before + 2);
    assert_eq!(state.hand_count(other), other_before + 2);
    let own = state.zones.get(state.zones.find(engine_core::state::ZoneKind::Hand, owner).unwrap()).unwrap().cards.clone();
    let state = choose(state, owner, vec![own[0].raw(), own[1].raw()], false);
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(owner), owner_before);
}

#[test]
fn danshou_publishes_ol_jie_zhuran_character_and_phase_reset_trigger() {
    const PACKAGE: &[u8] =
        include_bytes!("../../../../content/original/ported/onlyOL/onlyOL.npack.json");
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let character = package
        .character_index
        .iter()
        .find(|character| character.name_key == "ol_jie_zhuran")
        .expect("ol_jie_zhuran");
    assert_eq!(character.id, 2_606_884_808);
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.group, "wu");
    assert_eq!(character.skills, [3_750_129_445]);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == 3_750_129_445)
        .expect("danshou phase trigger");
    assert_eq!(trigger.event, "phase_play");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(
        package
            .active_index
            .iter()
            .any(|active| active.skill_def_id == 3_750_129_445),
        "danshou active index"
    );
}

#[test]
fn danshou_discard_target_card_then_discards_one_own() {
    let (state, owner, other) = setup();
    let other_before = state.hand_count(other);
    let owner_before = state.hand_count(owner);
    let (state, _) = start(state, owner);
    let state = choose(state, owner, vec![1], false);
    let state = choose(state, owner, vec![other.raw()], false);
    assert!(state.has_pending_choice(), "option 1 asks for a target card");
    let other_hand = state
        .zones
        .get(state.zones.find(engine_core::state::ZoneKind::Hand, other).unwrap())
        .unwrap()
        .cards
        .clone();
    let state = choose(state, owner, vec![other_hand[0].raw()], false);
    assert_eq!(state.hand_count(other), other_before - 1);
    assert!(state.has_pending_choice(), "option 1 asks for one own discard");
    let state = choose(state, owner, vec![], true);
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(owner), owner_before);
}

#[test]
fn danshou_gain_target_card_then_discards_two() {
    let (state, owner, other) = setup();
    let owner_before = state.hand_count(owner);
    let other_before = state.hand_count(other);
    let (state, _) = start(state, owner);
    let state = choose(state, owner, vec![2], false);
    let state = choose(state, owner, vec![other.raw()], false);
    let other_hand = state
        .zones
        .get(state.zones.find(engine_core::state::ZoneKind::Hand, other).unwrap())
        .unwrap()
        .cards
        .clone();
    let gained = other_hand[0];
    let state = choose(state, owner, vec![gained.raw()], false);
    assert_eq!(state.hand_count(other), other_before - 1);
    let own_hand = state
        .zones
        .get(state.zones.find(engine_core::state::ZoneKind::Hand, owner).unwrap())
        .unwrap()
        .cards
        .clone();
    assert!(own_hand.contains(&gained));
    assert!(state.has_pending_choice(), "option 2 asks for two own discards");
    let state = choose(state, owner, vec![own_hand[0].raw(), own_hand[1].raw()], false);
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(owner), owner_before - 1);
}

#[test]
fn danshou_used_option_is_illegal_not_silently_ignored() {
    let (state, owner, other) = setup();
    let (state, _) = start(state, owner);
    let state = choose(state, owner, vec![3], false);
    let state = choose(state, owner, vec![other.raw()], false);
    let state = choose(state, owner, vec![], true);
    assert!(!state.has_pending_choice());

    let (state, options) = start(state, owner);
    assert!(!options.contains(&3));
    let (state, result) = choose_result(state, owner, vec![3], false);
    assert!(
        matches!(result, CommandResult::Rejected { .. }),
        "used option must be rejected, got {result:?}"
    );
    assert!(state.has_pending_choice(), "rejection keeps the choice open");
}

#[test]
fn danshou_is_usable_through_activate_skill_in_play_phase() {
    let (mut state, owner, _) = setup();
    state.turn.phase = engine_core::state::Phase::Play;
    let skill = skill_instance(&state, owner);
    let revision = state.revision;
    let (state, outcome) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(11),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![],
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(outcome.result, CommandResult::Accepted { .. }),
        "activate danshou: {:?}",
        outcome.result
    );
    assert!(state.has_pending_choice());
    assert_eq!(
        outcome
            .events
            .iter()
            .filter(|event| matches!(event, engine_api::DomainEvent::ChoiceRequested { .. }))
            .count(),
        1
    );
}

#[test]
fn danshou_phase_play_trigger_clears_used_options() {
    let (mut state, owner, other) = setup();
    assert_eq!(state.turn.current, owner, "setup starts owner's turn");
    state.turn.phase = engine_core::state::Phase::Play;
    let (state, _) = start(state, owner);
    let state = choose(state, owner, vec![3], false);
    let state = choose(state, owner, vec![other.raw()], false);
    let state = choose(state, owner, vec![], true);
    let skill = skill_instance(&state, owner);
    assert_eq!(state.skills.storage_get(skill, 3), 1);

    let mut state = state;
    state.turn.phase = engine_core::state::Phase::Prepare;
    let mut state = state;
    for key in 12..16 {
        let revision = state.revision;
        let (next, outcome) = apply_command(
            state,
            Command {
                actor: owner,
                expected_revision: ExpectedRevision(revision),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::EndPhase,
            },
        );
        assert!(
            matches!(outcome.result, CommandResult::Accepted { .. }),
            "advance to play: {:?}",
            outcome.result
        );
        state = next;
        if state.turn.phase == engine_core::state::Phase::Play {
            break;
        }
    }
    assert_eq!(state.turn.phase, engine_core::state::Phase::Play);
    assert_eq!(state.skills.storage_get(skill, 3), 0);
    let (_, options) = start(state, owner);
    assert_eq!(options, vec![1, 2, 3, 4]);
}

#[test]
fn danshou_draw_option_can_target_self_and_draws_twice() {
    let (state, owner, _) = setup();
    let before = state.hand_count(owner);
    let (state, _) = start(state, owner);
    let state = choose(state, owner, vec![4], false);
    let state = choose(state, owner, vec![owner.raw()], false);
    assert_eq!(state.hand_count(owner), before + 4);
    let own = state
        .zones
        .get(state.zones.find(engine_core::state::ZoneKind::Hand, owner).unwrap())
        .unwrap()
        .cards
        .clone();
    let state = choose(state, owner, vec![own[0].raw(), own[1].raw()], false);
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(owner), before + 2);
}

#[test]
fn danshou_requires_at_least_one_own_card() {
    let (state, owner, _) = setup();
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        }],
    )
    .unwrap();
    let state = tx.commit().state;
    assert_eq!(state.hand_count(owner), 0);
    let skill = skill_instance(&state, owner);
    let mut tx = Transaction::begin(state);
    let _ = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveOlJieDanshou {
            player: owner,
            skill,
        }],
    );
    let committed = tx.commit();
    assert!(committed.output.choice.is_none());
    assert!(!committed.state.has_pending_choice());
}

#[test]
fn danshou_storage_survives_snapshot_roundtrip() {
    let (state, owner, other) = setup();
    let (state, _) = start(state, owner);
    let state = choose(state, owner, vec![3], false);
    let state = choose(state, owner, vec![other.raw()], false);
    let state = choose(state, owner, vec![], true);
    let skill = skill_instance(&state, owner);
    assert_eq!(state.skills.storage_get(skill, 3), 1);

    let snapshot = engine_core::snapshot::Snapshot::capture(&state);
    let bytes = snapshot.to_bytes().unwrap();
    let decoded = engine_core::snapshot::Snapshot::from_bytes(&bytes).unwrap();
    let restored = decoded.restore().unwrap();
    let restored_skill = skill_instance(&restored, owner);
    assert_eq!(restored.skills.storage_get(restored_skill, 3), 1);
    let (_, options) = start(restored, owner);
    assert_eq!(options, vec![1, 2, 4]);
}

#[test]
fn danshou_pending_choice_continuation_survives_snapshot() {
    let (state, owner, other) = setup();
    let (state, _) = start(state, owner);
    assert!(state.has_pending_choice());
    let snapshot = engine_core::snapshot::Snapshot::capture(&state);
    let bytes = snapshot.to_bytes().unwrap();
    let decoded = engine_core::snapshot::Snapshot::from_bytes(&bytes).unwrap();
    let restored = decoded.restore().unwrap();
    assert!(restored.has_pending_choice());
    let hp = restored.players.get(other).unwrap().hp;
    let state = choose(restored, owner, vec![3], false);
    let state = choose(state, owner, vec![other.raw()], false);
    assert_eq!(state.players.get(other).unwrap().hp, hp - 1);
    let state = choose(state, owner, vec![], true);
    assert!(!state.has_pending_choice());
}

#[test]
fn danshou_discard_option_can_target_self() {
    let (state, owner, _) = setup();
    let before = state.hand_count(owner);
    let (state, _) = start(state, owner);
    let (state, result) = choose_result(state, owner, vec![1], false);
    assert!(matches!(result, CommandResult::Accepted { .. }));
    let (state, result) = choose_result(state, owner, vec![owner.raw()], false);
    assert!(
        matches!(result, CommandResult::Accepted { .. }),
        "option 1 may target self: {result:?}"
    );
    let own = state
        .zones
        .get(state.zones.find(engine_core::state::ZoneKind::Hand, owner).unwrap())
        .unwrap()
        .cards
        .clone();
    let state = choose(state, owner, vec![own[0].raw()], false);
    let state = choose(state, owner, vec![], true);
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(owner), before - 1);
}

#[test]
fn danshou_gain_option_rejects_self_target() {
    let (state, owner, _) = setup();
    let (state, _) = start(state, owner);
    let state = choose(state, owner, vec![2], false);
    let (state, result) = choose_result(state, owner, vec![owner.raw()], false);
    assert!(
        matches!(result, CommandResult::Rejected { .. }),
        "option 2 is other-only: {result:?}"
    );
    assert!(state.has_pending_choice());
}

#[test]
fn danshou_discard_option_excludes_target_without_cards() {
    let (state, owner, other) = setup();
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: other,
            count: u32::MAX,
        }],
    )
    .unwrap();
    let state = tx.commit().state;
    assert_eq!(state.hand_count(other), 0);
    let (state, _) = start(state, owner);
    let state = choose(state, owner, vec![1], false);
    let (state, result) = choose_result(state, owner, vec![other.raw()], false);
    assert!(
        matches!(result, CommandResult::Rejected { .. }),
        "target with no discardable cards must be rejected: {result:?}"
    );
    assert!(state.has_pending_choice());
}

#[test]
fn danshou_discard_option_excludes_protected_equipment() {
    const GUARD_SRC: &str =
        "skill danshou_guard(3750129446) locked { modifier card_discard_protected(equip) override 1; }";
    let (mut state, owner, other) = setup();
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: other,
            count: u32::MAX,
        }],
    )
    .unwrap();
    state = tx.commit().state;
    let guard = content_compiler::compile_source_named("danshou-guard", GUARD_SRC).unwrap();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&guard)
        .unwrap();
    let equip_card = state
        .zones
        .get(state.zones.find(engine_core::state::ZoneKind::DrawPile, PlayerId::NONE).unwrap())
        .unwrap()
        .cards
        .iter()
        .copied()
        .find(|card| {
            state
                .cards
                .get(*card)
                .is_some_and(|card| card.kind.is_equip())
        })
        .expect("an equipment card in the draw pile");
    let equip_zone = state
        .zones
        .find(engine_core::state::ZoneKind::Equip, other)
        .expect("equipment zone");
    let draw_zone = state
        .zones
        .find(engine_core::state::ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: other,
                skill_def: SkillDefId::new(3_750_129_446),
            },
            EffectRequest::MoveCards {
                cards: vec![equip_card],
                to: equip_zone,
                from: Some(draw_zone),
            },
        ],
    )
    .unwrap();
    let state = tx.commit().state;
    let (state, _) = start(state, owner);
    let state = choose(state, owner, vec![1], false);
    let (state, result) = choose_result(state, owner, vec![other.raw()], false);
    assert!(
        matches!(result, CommandResult::Rejected { .. }),
        "protected equipment must not be a discard target: {result:?}"
    );
    assert!(state.has_pending_choice());
}

#[test]
fn danshou_gain_option_excludes_protected_cards() {
    const GUARD_SRC: &str =
        "skill danshou_gain_guard(3750129447) locked { modifier card_gain_protected(any) override 1; }";
    let (mut state, owner, other) = setup();
    let guard = content_compiler::compile_source_named("danshou-gain-guard", GUARD_SRC).unwrap();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&guard)
        .unwrap();
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: other,
            skill_def: SkillDefId::new(3_750_129_447),
        }],
    )
    .unwrap();
    let state = tx.commit().state;
    let (state, options) = start(state, owner);
    assert!(
        !options.contains(&2),
        "option 2 must be unavailable when every other target is gain-protected"
    );
    let (state, result) = choose_result(state, owner, vec![2], false);
    assert!(
        matches!(result, CommandResult::Rejected { .. }),
        "gain-protected-only option must be rejected: {result:?}"
    );
    assert!(state.has_pending_choice());
}

#[test]
fn danshou_damage_option_rejects_dead_target() {
    let (mut state, owner, other) = setup();
    state.players.get_mut(other).unwrap().alive = false;
    state.players.get_mut(other).unwrap().dying = false;
    let (state, _) = start(state, owner);
    let state = choose(state, owner, vec![3], false);
    let (state, result) = choose_result(state, owner, vec![other.raw()], false);
    assert!(
        matches!(result, CommandResult::Rejected { .. }),
        "dead target must be rejected: {result:?}"
    );
    assert!(state.has_pending_choice());
}

#[test]
fn danshou_draw_option_rejects_dead_target() {
    let (mut state, owner, other) = setup();
    state.players.get_mut(other).unwrap().alive = false;
    state.players.get_mut(other).unwrap().dying = false;
    let (state, _) = start(state, owner);
    let state = choose(state, owner, vec![4], false);
    let (state, result) = choose_result(state, owner, vec![other.raw()], false);
    assert!(
        matches!(result, CommandResult::Rejected { .. }),
        "dead draw target must be rejected: {result:?}"
    );
    assert!(state.has_pending_choice());
}

#[test]
fn danshou_damage_option_can_target_self() {
    let (state, owner, _) = setup();
    let hp = state.players.get(owner).unwrap().hp;
    let hand = state.hand_count(owner);
    let (state, _) = start(state, owner);
    let state = choose(state, owner, vec![3], false);
    let state = choose(state, owner, vec![owner.raw()], false);
    assert_eq!(state.players.get(owner).unwrap().hp, hp - 1);
    assert!(state.has_pending_choice());
    let state = choose(state, owner, vec![], true);
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(owner), hand);
}

#[test]
fn danshou_rejects_unknown_option_token() {
    let (state, owner, _) = setup();
    let (state, _) = start(state, owner);
    let (state, result) = choose_result(state, owner, vec![5], false);
    assert!(
        matches!(result, CommandResult::Rejected { .. }),
        "unknown option token must be rejected: {result:?}"
    );
    assert!(state.has_pending_choice());
}

#[test]
fn danshou_all_options_used_produces_no_choice() {
    let (mut state, owner, other) = setup();
    // Give both sides enough cards for the four option resolutions.
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::DrawCards { player: owner, count: 4 },
            EffectRequest::DrawCards { player: other, count: 2 },
        ],
    )
    .unwrap();
    state = tx.commit().state;

    let (state, _) = start(state, owner);
    let state = choose(state, owner, vec![1], false);
    let state = choose(state, owner, vec![owner.raw()], false);
    let target_card = state
        .zones
        .get(state.zones.find(engine_core::state::ZoneKind::Hand, owner).unwrap())
        .unwrap()
        .cards[0];
    let state = choose(state, owner, vec![target_card.raw()], false);
    let state = choose(state, owner, vec![], true);

    let (state, _) = start(state, owner);
    let state = choose(state, owner, vec![2], false);
    let state = choose(state, owner, vec![other.raw()], false);
    let target_card = state
        .zones
        .get(state.zones.find(engine_core::state::ZoneKind::Hand, other).unwrap())
        .unwrap()
        .cards[0];
    let state = choose(state, owner, vec![target_card.raw()], false);
    let state = choose(state, owner, vec![], true);

    let (state, _) = start(state, owner);
    let state = choose(state, owner, vec![3], false);
    let state = choose(state, owner, vec![owner.raw()], false);
    let state = choose(state, owner, vec![], true);

    let (state, _) = start(state, owner);
    let state = choose(state, owner, vec![4], false);
    let state = choose(state, owner, vec![owner.raw()], false);
    let state = choose(state, owner, vec![], true);

    let skill = skill_instance(&state, owner);
    let mut tx = Transaction::begin(state);
    let _ = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveOlJieDanshou {
            player: owner,
            skill,
        }],
    );
    let committed = tx.commit();
    assert!(committed.output.choice.is_none());
    assert!(!committed.state.has_pending_choice());
}

#[test]
fn danshou_new_and_old_npack_versions_keep_identity() {
    const NEW_PACKAGE: &[u8] =
        include_bytes!("../../../../content/original/ported/onlyOL/onlyOL.npack.json");
    const OLD_PACKAGE: &[u8] =
        include_bytes!("../../../../docs/operations/evidence/2026-10-24-up37-n01-zhuhuan/onlyOL-0.1.2.npack.json");
    let new = content_schema::ContentPackage::from_json_bytes(NEW_PACKAGE).unwrap();
    assert_eq!(new.manifest.version, "0.1.4");
    assert_eq!(new.manifest.rule_vm_abi_version, 757);
    assert!(new
        .character_index
        .iter()
        .any(|character| character.name_key == "ol_jie_zhuran"));
    let old = content_schema::ContentPackage::from_json_bytes(OLD_PACKAGE).unwrap();
    assert_eq!(old.manifest.version, "0.1.2");
    assert_eq!(old.manifest.rule_vm_abi_version, 754);
    assert!(!old
        .character_index
        .iter()
        .any(|character| character.name_key == "ol_jie_zhuran"));
}

#[test]
fn danshou_pending_discard_with_damage_history_survives_snapshot() {
    let (state, owner, other) = setup();
    let hp = state.players.get(other).unwrap().hp;
    let (state, _) = start(state, owner);
    let state = choose(state, owner, vec![3], false);
    let state = choose(state, owner, vec![other.raw()], false);
    assert!(state.has_pending_choice());
    assert!(!state.turn.damage_pair_counts.is_empty());

    let snapshot = engine_core::snapshot::Snapshot::capture(&state);
    let bytes = snapshot.to_bytes().unwrap();
    let decoded = engine_core::snapshot::Snapshot::from_bytes(&bytes).unwrap();
    let restored = decoded.restore().unwrap();
    assert!(restored.has_pending_choice());
    assert_eq!(restored.players.get(other).unwrap().hp, hp - 1);
    assert!(!restored.turn.damage_pair_counts.is_empty());
    let skill = skill_instance(&restored, owner);
    assert_eq!(restored.skills.storage_get(skill, 3), 1);

    let state = choose(restored, owner, vec![], true);
    assert!(!state.has_pending_choice());
}
