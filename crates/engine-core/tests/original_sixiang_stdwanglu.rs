//! Golden coverage for 标张奋「望橹」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const STD_ZHANGFEN: u32 = 3_289_092_799;
const STDWANGLU: u32 = 3_938_702_468;
const SOURCE: &str = include_str!("../../../content/original/ported/sixiang/stdwanglu.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    empty: PlayerId,
    skill: engine_api::SkillId,
    equipment: CardId,
}

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn skill_id(state: &GameState, owner: PlayerId) -> engine_api::SkillId {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == STDWANGLU)
        .unwrap()
}

fn inject_equipment(
    state: &mut GameState,
    owner: PlayerId,
    kind: CardKind,
    weapon_range: u8,
) -> CardId {
    let zone = state.zones.find(ZoneKind::Equip, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(136_001 + u32::from(kind.tag())),
        kind,
        suit: Suit::Spade,
        rank: 5,
        weapon_range,
        zone,
        owner,
        face_up: true,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(kind: CardKind, weapon_range: u8) -> Setup {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 2_026_072_409, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package()).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let empty = players[2];
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
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(STDWANGLU),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let equipment = inject_equipment(tx.state_mut(), target, kind, weapon_range);
    let skill = skill_id(tx.state(), owner);
    Setup {
        state: tx.commit().state,
        owner,
        target,
        empty,
        skill,
        equipment,
    }
}

fn activate(
    setup: &Setup,
    state: GameState,
    target: PlayerId,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
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
    card: CardId,
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
                selections: vec![card.raw()],
                cancelled: false,
            },
        },
    )
}

#[test]
fn package_publishes_abi136_and_complete_std_zhangfen() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert_eq!(package.skill_index.len(), 181);
    assert_eq!(package.character_index.len(), 101);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == STDWANGLU)
        .unwrap();
    assert_eq!(active.window, "play_phase");
    assert_eq!(active.uses_per_turn, Some(1));
    let targets = active.targets.as_ref().unwrap();
    assert_eq!((targets.min, targets.max, targets.allow_self), (1, 1, true));
    assert!(targets.has_discardable_equip);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == STD_ZHANGFEN)
        .unwrap();
    assert_eq!(character.name_key, "std_zhangfen");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "wu");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [STDWANGLU]);

    let compiled = content_compiler::compile_source_named("stdwanglu", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 136);
    let expected = &compiled.modules[0];
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
}

#[test]
fn weapon_discard_draws_printed_range_after_card_movement() {
    let setup = setup(CardKind::Weapon, 3);
    let before = setup.state.hand_count(setup.target);
    let (offered, activation) = activate(&setup, setup.state.clone(), setup.target, 1);
    assert!(matches!(activation.result, CommandResult::Accepted { .. }));
    let choice = activation.choice.as_ref().unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert_eq!((choice.min_count, choice.max_count), (1, 1));
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [setup.equipment.raw()]
    );

    let (done, result) = resolve(offered, setup.owner, setup.equipment, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(done.hand_count(setup.target), before + 3);
    let discard = done
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(done.cards.get(setup.equipment).unwrap().zone, discard);
    let moved = result
        .events
        .iter()
        .position(|event| {
            matches!(event, DomainEvent::CardsMoved { cards, .. } if cards.contains(&setup.equipment))
        })
        .unwrap();
    let drawn = result
        .events
        .iter()
        .position(
            |event| matches!(event, DomainEvent::CardsDrawn { player, .. } if *player == setup.target),
        )
        .unwrap();
    assert!(moved < drawn);
}

#[test]
fn non_weapon_draws_nothing_and_target_without_discardable_equipment_is_rejected() {
    let setup = setup(CardKind::Armor, 0);
    let before_hash = canonical_state_hash(&setup.state);
    let (same, rejected) = activate(&setup, setup.state.clone(), setup.empty, 10);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before_hash);

    let before_hand = setup.state.hand_count(setup.target);
    let (offered, activation) = activate(&setup, setup.state.clone(), setup.target, 11);
    assert!(matches!(activation.result, CommandResult::Accepted { .. }));
    let (done, result) = resolve(offered, setup.owner, setup.equipment, 12);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(done.hand_count(setup.target), before_hand);
    assert!(!result.events.iter().any(
        |event| matches!(event, DomainEvent::CardsDrawn { player, .. } if *player == setup.target)
    ));
}
