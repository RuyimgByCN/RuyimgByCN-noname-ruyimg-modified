//! Golden coverage for STANDARD「反间」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{ContinuationKind, GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const FANJIAN: u32 = 36;
const YINGZI: u32 = 20;
const ZHOUYU: u32 = 19;
const SOURCE: &str = include_str!("../../../content/standard/fanjian.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
}

fn setup(seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(FANJIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == FANJIAN)
        .unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        target,
        skill,
    }
}

fn activate(setup: &Setup, key: u64) -> (GameState, PipelineResult) {
    apply_command(
        setup.state.clone(),
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(setup.state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill: setup.skill,
                targets: vec![setup.target],
                card_targets: vec![],
            },
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selection: u32,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections: vec![selection],
                cancelled: false,
            },
        },
    )
}

#[test]
fn package_publishes_current_abi_active_and_complete_zhouyu() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 261);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == FANJIAN)
        .expect("fanjian active");
    assert_eq!(active.uses_per_turn, Some(1));
    assert!(active.hand_nonempty);
    assert_eq!(
        active
            .targets
            .as_ref()
            .map(|targets| (targets.min, targets.max, targets.allow_self)),
        Some((1, 1, false))
    );
    assert!(package
        .character_index
        .iter()
        .any(|character| { character.id == ZHOUYU && character.skills == [YINGZI, FANJIAN] }));
    let compiled = content_compiler::compile_source_named("fanjian", SOURCE).unwrap();
    assert!(compiled.modules.iter().all(|expected| {
        package
            .modules
            .iter()
            .any(|actual| actual.name == expected.name && actual.logic_hash == expected.logic_hash)
    }));
}

#[test]
fn target_guesses_then_gains_owner_hand_card_and_mismatch_deals_owner_damage() {
    let mut setup = setup(2_026_072_501);
    let card = setup.state.hand_cards(setup.owner)[0];
    setup.state.cards.get_mut(card).unwrap().suit = Suit::Spade;
    let target_card = setup.state.hand_cards(setup.target)[0];
    let hp = setup.state.players.get(setup.target).unwrap().hp;

    let (state, activated) = activate(&setup, 1);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::FanjianChooseSuitOrHand
    );
    assert_eq!(
        activated.choice.as_ref().map(|choice| {
            choice
                .options
                .iter()
                .map(|option| option.token)
                .collect::<Vec<_>>()
        }),
        Some(vec![2, 4, 3, 1])
    );

    let (state, guessed) = resolve(state, setup.target, 2, 2);
    assert!(matches!(guessed.result, CommandResult::Accepted { .. }));
    assert!(guessed.choice.as_ref().is_some_and(|choice| choice
        .options
        .iter()
        .any(|option| option.token == card.raw())));

    let (unchanged, forged) = resolve(state.clone(), setup.target, target_card.raw(), 3);
    assert!(matches!(forged.result, CommandResult::Rejected { .. }));
    assert_eq!(unchanged, state);

    let (state, gained) = resolve(state, setup.target, card.raw(), 4);
    assert!(matches!(gained.result, CommandResult::Accepted { .. }));
    assert!(state.hand_cards(setup.target).contains(&card));
    assert_eq!(state.players.get(setup.target).unwrap().hp, hp - 1);
    assert!(gained.events.iter().any(|event| matches!(
        event,
        DomainEvent::DamageDealt {
            source,
            target,
            amount: 1,
            ..
        } if *source == setup.owner && *target == setup.target
    )));

    setup.state = state;
    let (_, repeated) = activate(&setup, 5);
    assert!(matches!(repeated.result, CommandResult::Rejected { .. }));
}

#[test]
fn matching_owner_effective_suit_moves_card_without_damage() {
    let mut setup = setup(2_026_072_502);
    let card: CardId = setup.state.hand_cards(setup.owner)[0];
    setup.state.cards.get_mut(card).unwrap().suit = Suit::Diamond;
    let hp = setup.state.players.get(setup.target).unwrap().hp;
    let (state, _) = activate(&setup, 6);
    let (state, _) = resolve(state, setup.target, 4, 7);
    let (state, result) = resolve(state, setup.target, card.raw(), 8);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(state.hand_cards(setup.target).contains(&card));
    assert_eq!(state.players.get(setup.target).unwrap().hp, hp);
}
