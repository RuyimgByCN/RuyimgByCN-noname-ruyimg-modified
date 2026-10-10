//! Golden coverage for 庞统「旧涅槃」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const PANGTONG: u32 = 3_966_656_508;
const LIANHUAN: u32 = 2_921_824_505;
const OLDNIEPAN: u32 = 3_136_864_229;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");

fn has_oldniepan(state: &GameState, owner: PlayerId) -> bool {
    state
        .skills
        .sorted_live()
        .into_iter()
        .any(|skill| skill.owner == owner && skill.def.raw() == OLDNIEPAN)
}

fn inject(state: &mut GameState, owner: PlayerId, zone: ZoneKind, def: u32) -> CardId {
    let zone_id = state.zones.find(zone, owner).unwrap();
    let kind = if zone == ZoneKind::Equip {
        CardKind::Weapon
    } else {
        CardKind::TrickLe
    };
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone: zone_id,
        owner,
        face_up: zone != ZoneKind::Hand,
    });
    state.zones.get_mut(zone_id).unwrap().cards.push(card);
    card
}

fn setup(seed: u64) -> (Transaction, PlayerId, Vec<CardId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let owner = tx.state().players.alive_by_seat()[0];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLDNIEPAN),
    });
    run_effects(&mut tx, effects).unwrap();

    let mut original_cards = tx.state().hand_cards(owner);
    original_cards.push(inject(tx.state_mut(), owner, ZoneKind::Equip, 93_001));
    original_cards.push(inject(tx.state_mut(), owner, ZoneKind::Judge, 93_002));
    let participant = tx.state_mut().players.get_mut(owner).unwrap();
    participant.hp = -1;
    participant.max_hp = 3;
    participant.turned_over = true;
    participant
        .marks
        .push((engine_core::content::MARK_CHAIN, 1));
    (tx, owner, original_cards)
}

fn offer(tx: &mut Transaction, owner: PlayerId) -> Result<Vec<EffectRequest>, EffectError> {
    dispatch_for_events(
        tx,
        &[DomainEvent::EnteredDying {
            player: owner,
            source: PlayerId::NONE,
        }],
        &mut TriggerFireRecord::new(),
    )
}

fn resolve(tx: &Transaction, owner: PlayerId, accept: bool, key: u64) -> GameState {
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![u32::from(accept)],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn package_publishes_complete_pangtong_and_oldniepan_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 671);
    assert_eq!(package.skill_index.len(), 201);
    assert_eq!(package.character_index.len(), 48);
    assert!(package.character_index.iter().any(|character| {
        character.id == PANGTONG
            && character.name_key == "pangtong"
            && character.max_hp == 3
            && character.skills == [LIANHUAN, OLDNIEPAN]
    }));
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == OLDNIEPAN)
        .unwrap();
    assert_eq!(
        (
            trigger.event.as_str(),
            trigger.filter.as_str(),
            trigger.optional
        ),
        ("dying", "owner_is_target", true)
    );
}

#[test]
fn accepting_discards_hej_restores_state_draws_three_recovers_to_three_and_consumes() {
    let (mut tx, owner, original_cards) = setup(2_026_072_302);
    assert_eq!(
        offer(&mut tx, owner).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let accepted = resolve(&tx, owner, true, 1);
    let participant = accepted.players.get(owner).unwrap();
    assert_eq!(participant.hp, 3);
    assert!(!participant.turned_over);
    assert!(!participant
        .marks
        .iter()
        .any(|(mark, count)| *mark == engine_core::content::MARK_CHAIN && *count > 0));
    assert!(!has_oldniepan(&accepted, owner));
    assert_eq!(accepted.hand_count(owner), 3);
    assert!(accepted
        .zones
        .get(accepted.zones.find(ZoneKind::Equip, owner).unwrap())
        .unwrap()
        .cards
        .is_empty());
    assert!(accepted
        .zones
        .get(accepted.zones.find(ZoneKind::Judge, owner).unwrap())
        .unwrap()
        .cards
        .is_empty());
    let discard = accepted
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(original_cards
        .into_iter()
        .all(|card| accepted.cards.get(card).unwrap().zone == discard));
}

#[test]
fn declining_preserves_cards_state_hp_and_limited_skill() {
    let (mut tx, owner, original_cards) = setup(2_026_072_303);
    assert_eq!(
        offer(&mut tx, owner).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let declined = resolve(&tx, owner, false, 2);
    let participant = declined.players.get(owner).unwrap();
    assert_eq!(participant.hp, -1);
    assert!(participant.turned_over);
    assert!(participant
        .marks
        .iter()
        .any(|(mark, count)| *mark == engine_core::content::MARK_CHAIN && *count > 0));
    assert!(has_oldniepan(&declined, owner));
    assert!(original_cards.into_iter().all(|card| {
        let zone = declined.cards.get(card).unwrap().zone;
        zone == declined.zones.find(ZoneKind::Hand, owner).unwrap()
            || zone == declined.zones.find(ZoneKind::Equip, owner).unwrap()
            || zone == declined.zones.find(ZoneKind::Judge, owner).unwrap()
    }));
}
