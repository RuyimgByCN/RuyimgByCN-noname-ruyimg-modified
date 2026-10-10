//! Golden coverage for COLLAB「子鼠」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLZISHU: u32 = 2_483_133_612;
const OL_ZISHU: u32 = 3_877_208_473;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/collab/collab.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    others: Vec<PlayerId>,
    skill: SkillId,
}

fn inject_hand(state: &mut GameState, owner: PlayerId, def: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn setup(seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let owner = tx.state().turn.current;
    let others = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect::<Vec<_>>();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(
        tx.state()
            .players
            .alive_by_seat()
            .into_iter()
            .map(|player| EffectRequest::DiscardFromHand {
                player,
                count: u32::MAX,
            }),
    );
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(OLZISHU),
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
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == OLZISHU)
        .unwrap();
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        others,
        skill,
    }
}

fn activate(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![target],
                card_targets: Vec::new(),
            },
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    choice_id: ChoiceId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
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
fn package_publishes_larger_hand_contract_and_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 694);
    assert_eq!(package.skill_index.len(), 152);
    assert_eq!(package.character_index.len(), 88);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == OLZISHU)
        .unwrap();
    assert_eq!(active.uses_per_turn, Some(1));
    let targets = active.targets.as_ref().unwrap();
    assert_eq!(
        (targets.min, targets.max, targets.allow_self),
        (1, 1, false)
    );
    assert!(targets.hand_gt_owner);
    assert!(package.character_index.iter().any(|character| {
        character.id == OL_ZISHU
            && character.name_key == "ol_zishu"
            && character.sex == "male"
            && character.group == "qun"
            && character.max_hp == 3
            && character.skills == [OLZISHU]
    }));
}

#[test]
fn mandatory_first_take_repeats_and_stops_on_a_maximum_hand_tie() {
    let mut setup = setup(2_026_072_316);
    let first = setup.others[0];
    let second = setup.others[1];
    inject_hand(&mut setup.state, setup.owner, 99_301);
    let first_cards = [
        inject_hand(&mut setup.state, first, 99_311),
        inject_hand(&mut setup.state, first, 99_312),
        inject_hand(&mut setup.state, first, 99_313),
    ];
    let second_cards = [
        inject_hand(&mut setup.state, second, 99_321),
        inject_hand(&mut setup.state, second, 99_322),
        inject_hand(&mut setup.state, second, 99_323),
        inject_hand(&mut setup.state, second, 99_324),
    ];

    let (state, activated) = activate(setup.state, setup.owner, first, setup.skill, 1);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    let first_card_choice = activated.choice.expect("mandatory first hand-card choice");
    assert!(!first_card_choice.allow_cancel);
    assert_eq!(
        first_card_choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        first_cards.map(CardId::raw)
    );

    let (state, first_resolved) = resolve(
        state,
        setup.owner,
        first_card_choice.choice_id,
        vec![first_cards[0].raw()],
        false,
        2,
    );
    assert!(matches!(
        first_resolved.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(state.hand_count(setup.owner), 2);
    assert_eq!(state.hand_count(first), 2);
    let target_choice = first_resolved.choice.expect("optional repeated target");
    assert!(target_choice.allow_cancel);
    assert_eq!(
        target_choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [second.raw()]
    );

    let (state, target_resolved) = resolve(
        state,
        setup.owner,
        target_choice.choice_id,
        vec![second.raw()],
        false,
        3,
    );
    let second_card_choice = target_resolved.choice.expect("second target hand card");
    assert!(!second_card_choice.allow_cancel);
    assert_eq!(
        second_card_choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        second_cards.map(CardId::raw)
    );

    let (state, second_resolved) = resolve(
        state,
        setup.owner,
        second_card_choice.choice_id,
        vec![second_cards[0].raw()],
        false,
        4,
    );
    assert!(matches!(
        second_resolved.result,
        CommandResult::Accepted { .. }
    ));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(setup.owner), 3);
    assert_eq!(state.hand_count(second), 3);
}

#[test]
fn repeated_choice_may_be_cancelled_and_initial_non_larger_target_is_rejected() {
    let mut setup = setup(2_026_072_317);
    let equal = setup.others[0];
    let larger = setup.others[1];
    inject_hand(&mut setup.state, setup.owner, 99_331);
    inject_hand(&mut setup.state, equal, 99_332);
    let larger_cards = [
        inject_hand(&mut setup.state, larger, 99_341),
        inject_hand(&mut setup.state, larger, 99_342),
        inject_hand(&mut setup.state, larger, 99_343),
        inject_hand(&mut setup.state, larger, 99_344),
    ];

    let before = canonical_state_hash(&setup.state);
    let (state, rejected) = activate(setup.state, setup.owner, equal, setup.skill, 10);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before);

    let (state, activated) = activate(state, setup.owner, larger, setup.skill, 11);
    let card_choice = activated.choice.expect("first take");
    let (state, taken) = resolve(
        state,
        setup.owner,
        card_choice.choice_id,
        vec![larger_cards[0].raw()],
        false,
        12,
    );
    let repeat = taken.choice.expect("repeat choice");
    let (state, cancelled) = resolve(state, setup.owner, repeat.choice_id, Vec::new(), true, 13);
    assert!(matches!(cancelled.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(setup.owner), 2);
    assert_eq!(state.hand_count(larger), 3);
}
