//! Golden coverage for MOBILE 丁原「备诛」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, MarkDefId, ObserverKind, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::{canonical_state_hash, project_observer};
use rule_api::EffectRequest;

const DINGYUAN: u32 = 2_181_570_673;
const BEIZHU: u32 = 2_346_585_902;
const SOURCE: &str = include_str!("../../../content/original/ported/mobile/beizhu.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    other: PlayerId,
    skill: SkillId,
}

fn setup(seed: u64) -> Setup {
    let package = content_compiler::compile_source_named("beizhu-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let other = players[2];
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let removed = tx
        .state()
        .zones
        .find(ZoneKind::Removed, PlayerId::NONE)
        .unwrap();
    let draw_cards = tx.state().zones.get(draw).unwrap().cards.clone();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.extend([
        EffectRequest::MoveCards {
            cards: draw_cards,
            to: removed,
            from: Some(draw),
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(BEIZHU),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == BEIZHU)
        .unwrap();
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        target,
        other,
        skill,
    }
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    def: u32,
    kind: CardKind,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn activate(setup: Setup, target: PlayerId, key: u64) -> (GameState, PipelineResult) {
    let revision = setup.state.revision;
    apply_command(
        setup.state,
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill: setup.skill,
                targets: vec![target],
                card_targets: Vec::new(),
            },
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn package_publishes_abi324_skill_and_complete_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 324);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == BEIZHU && skill.name == "beizhu"));
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == BEIZHU)
        .unwrap();
    assert_eq!(active.uses_per_turn, Some(1));
    assert!(active.targets.as_ref().is_some_and(|targets| {
        (targets.min, targets.max, targets.allow_self) == (1, 1, false)
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == DINGYUAN)
        .unwrap();
    assert_eq!(character.name_key, "dingyuan");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "qun");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [BEIZHU]);
}

#[test]
fn target_contract_rejects_self_and_empty_hand_atomically() {
    for self_target in [true, false] {
        let setup = setup(2_026_073_201 + u64::from(self_target));
        let target = if self_target {
            setup.owner
        } else {
            setup.target
        };
        let before = canonical_state_hash(&setup.state);
        let (state, result) = activate(setup, target, 1);
        assert!(matches!(result.result, CommandResult::Rejected { .. }));
        assert_eq!(canonical_state_hash(&state), before);
    }
}

#[test]
fn no_sha_branch_privately_reveals_discards_and_can_cancel_or_grant_draw_pile_sha() {
    let mut cancel = setup(2_026_073_203);
    let target = cancel.target;
    let owner = cancel.owner;
    let other = cancel.other;
    let hidden = inject(
        &mut cancel.state,
        target,
        ZoneKind::Hand,
        99_201,
        CardKind::Other,
    );
    let draw_sha = inject(
        &mut cancel.state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        99_202,
        CardKind::Sha,
    );
    let (state, activated) = activate(cancel, target, 2);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    let choice = activated.choice.unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert_eq!(choice.observer, owner);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [hidden.raw()]
    );
    let owner_view = project_observer(&state, ObserverKind::Player(owner));
    assert!(owner_view
        .known_cards
        .iter()
        .any(|card| card.id == hidden && card.def.is_some()));
    let other_view = project_observer(&state, ObserverKind::Player(other));
    assert!(other_view
        .known_cards
        .iter()
        .all(|card| card.id != hidden || card.def.is_none()));

    let (state, discarded) = resolve(state, owner, vec![hidden.raw()], false, 3);
    assert!(matches!(discarded.result, CommandResult::Accepted { .. }));
    let grant = discarded.choice.unwrap();
    assert_eq!(grant.kind, ChoiceKind::Custom);
    assert!(grant.allow_cancel);
    let (state, result) = resolve(state, owner, Vec::new(), true, 4);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.cards.get(draw_sha).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DrawPile, PlayerId::NONE)
            .unwrap()
    );

    let mut grant = setup(2_026_073_204);
    let target = grant.target;
    let owner = grant.owner;
    let hidden = inject(
        &mut grant.state,
        target,
        ZoneKind::Hand,
        99_203,
        CardKind::Other,
    );
    let draw_sha = inject(
        &mut grant.state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        99_204,
        CardKind::ShaThunder,
    );
    let (state, _) = activate(grant, target, 5);
    let (state, _) = resolve(state, owner, vec![hidden.raw()], false, 6);
    let (state, result) = resolve(state, owner, vec![1], false, 7);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let target_hand = state.zones.find(ZoneKind::Hand, target).unwrap();
    assert_eq!(state.cards.get(draw_sha).unwrap().zone, target_hand);
    assert_eq!(state.cards.get(draw_sha).unwrap().owner, target);
}

#[test]
fn all_snapshotted_sha_are_forced_and_direct_damage_draws_per_point() {
    let mut setup = setup(2_026_073_205);
    let owner = setup.owner;
    let target = setup.target;
    for def in [99_205, 99_206] {
        inject(&mut setup.state, target, ZoneKind::Hand, def, CardKind::Sha);
    }
    for def in [99_207, 99_208] {
        inject(
            &mut setup.state,
            PlayerId::NONE,
            ZoneKind::DrawPile,
            def,
            CardKind::Other,
        );
    }
    let hp = setup.state.players.get(owner).unwrap().hp;
    let (state, result) = activate(setup, target, 8);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(owner).unwrap().hp, hp - 2);
    assert_eq!(state.hand_count(owner), 2);
    assert_eq!(state.hand_count(target), 0);
}

#[test]
fn queued_sha_skips_stale_cards_and_survives_main_skill_removal() {
    let mut setup = setup(2_026_073_206);
    let owner = setup.owner;
    let target = setup.target;
    let stale = inject(
        &mut setup.state,
        target,
        ZoneKind::Hand,
        99_211,
        CardKind::Other,
    );
    let sha = inject(
        &mut setup.state,
        target,
        ZoneKind::Hand,
        99_212,
        CardKind::Sha,
    );
    inject(
        &mut setup.state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        99_213,
        CardKind::Other,
    );
    let hp = setup.state.players.get(owner).unwrap().hp;
    let mut tx = Transaction::begin(setup.state);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::RemoveSkill { skill: setup.skill },
            EffectRequest::ResolveBeizhu {
                player: owner,
                target,
                stage: 2,
                cards: vec![stale, sha],
                skill: setup.skill,
            },
        ],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 1);
    assert_eq!(tx.state().hand_count(owner), 1);
    assert_eq!(tx.state().hand_cards(target), [stale]);
}

#[test]
fn elemental_chain_damage_does_not_draw_for_the_propagated_target() {
    let mut setup = setup(2_026_073_207);
    let owner = setup.owner;
    let target = setup.target;
    let other = setup.other;
    inject(
        &mut setup.state,
        target,
        ZoneKind::Hand,
        99_209,
        CardKind::ShaFire,
    );
    inject(
        &mut setup.state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        99_210,
        CardKind::Other,
    );
    let mut tx = Transaction::begin(setup.state);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddMark {
                player: owner,
                mark: MarkDefId::new(engine_core::content::MARK_CHAIN),
                delta: 1,
            },
            EffectRequest::AddMark {
                player: other,
                mark: MarkDefId::new(engine_core::content::MARK_CHAIN),
                delta: 1,
            },
        ],
    )
    .unwrap();
    setup.state = tx.commit().state;
    let owner_hp = setup.state.players.get(owner).unwrap().hp;
    let other_hp = setup.state.players.get(other).unwrap().hp;
    let (state, result) = activate(setup, target, 9);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp - 1);
    assert_eq!(state.players.get(other).unwrap().hp, other_hp - 1);
    assert_eq!(state.hand_count(owner), 1);
    assert_eq!(state.hand_count(other), 0);
}
