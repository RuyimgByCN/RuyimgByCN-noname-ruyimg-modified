//! Golden coverage for Standard 骁果 rewritten from the legacy JavaScript semantics.

use content_compiler::compile_source_named;
use engine_api::{
    CardDefId, CardId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const STDXIAOGUO: u32 = 3_757_481_350;
const STD_YUEJIN: u32 = 3_041_549_837;
const SOURCE: &str = include_str!("../../../content/standard/stdxiaoguo.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    rank: u8,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(98_000 + u32::from(rank)),
        kind,
        suit: Suit::Club,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(with_equipment: bool) -> (GameState, PlayerId, PlayerId, CardId, Option<CardId>) {
    let package = compile_source_named("standard-stdxiaoguo-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_725_53, "identity-standard").unwrap();
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(STDXIAOGUO),
    });
    run_effects(&mut tx, effects).unwrap();
    let basic = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Sha, 2);
    inject(&mut tx, owner, ZoneKind::Hand, CardKind::Other, 3);
    let equipment =
        with_equipment.then(|| inject(&mut tx, target, ZoneKind::Hand, CardKind::Armor, 4));
    tx.state_mut().turn.current = target;
    tx.state_mut().turn.phase = Phase::End;
    (tx.commit().state, owner, target, basic, equipment)
}

fn dispatch_end(state: GameState, target: PlayerId) -> (GameState, engine_api::ChoiceRequest) {
    let mut tx = Transaction::begin(state);
    let mut fires = TriggerFireRecord::new();
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: target,
            phase: TurnPhaseTag::End,
        }],
        &mut fires,
    )
    .unwrap();
    assert_eq!(
        run_effects(&mut tx, effects),
        Err(EffectError::ChoiceRequested)
    );
    let committed = tx.commit();
    (committed.state, committed.output.choice.unwrap())
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    choice: engine_api::ChoiceRequest,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn package_contains_complete_standard_yuejin() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == STDXIAOGUO)
        .unwrap();
    assert_eq!(trigger.event, "phase_end");
    assert_eq!(
        trigger.filter,
        "owner_alive_other_phase_end_has_discardable_basic_hand"
    );
    assert!(!trigger.optional);
    let yuejin = package
        .character_index
        .iter()
        .find(|character| character.id == STD_YUEJIN)
        .unwrap();
    assert_eq!(yuejin.skills, [STDXIAOGUO]);
}

#[test]
fn basic_cost_then_target_discards_equipment_or_takes_damage() {
    let (state, owner, target, basic, equipment) = setup(true);
    let equipment = equipment.unwrap();
    let hp = state.players.get(target).unwrap().hp;
    let (state, cost) = dispatch_end(state, target);
    assert_eq!(cost.observer, owner);
    assert_eq!(cost.kind, ChoiceKind::SelectCards);
    assert!(cost
        .options
        .iter()
        .any(|option| option.token == basic.raw()));
    assert_eq!(cost.options.len(), 1);
    let (state, result) = resolve(state, owner, cost, vec![basic.raw()], false, 1);
    let consequence = result.choice.unwrap();
    assert_eq!(consequence.observer, target);
    assert!(consequence
        .options
        .iter()
        .any(|option| option.token == equipment.raw()));
    let (discarded, result) = resolve(
        state.clone(),
        target,
        consequence.clone(),
        vec![equipment.raw()],
        false,
        2,
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(discarded.players.get(target).unwrap().hp, hp);

    let (damaged, result) = resolve(state, target, consequence, Vec::new(), true, 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(damaged.players.get(target).unwrap().hp, hp - 1);
}

#[test]
fn declining_cost_or_missing_equipment_resolves_without_extra_choices() {
    let (state, owner, target, basic, _) = setup(false);
    let hp = state.players.get(target).unwrap().hp;
    let (state, cost) = dispatch_end(state, target);
    let (declined, result) = resolve(state.clone(), owner, cost.clone(), Vec::new(), true, 10);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(declined.players.get(target).unwrap().hp, hp);
    assert!(declined.hand_cards(owner).contains(&basic));

    let (damaged, result) = resolve(state, owner, cost, vec![basic.raw()], false, 11);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(damaged.players.get(target).unwrap().hp, hp - 1);
}
