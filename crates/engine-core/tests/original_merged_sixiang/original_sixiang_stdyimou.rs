//! Golden coverage for 标鲍信「毅谋」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, CardMoveReason, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult,
    DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const STDYIMOU: u32 = 4_237_148_705;
const STDMUTAO: u32 = 3_513_095_743;
const STD_BAOXIN: u32 = 2_419_830_352;
const SOURCE: &str = include_str!("../../../../content/original/ported/sixiang/stdyimou.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/sixiang/sixiang.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn inject(state: &mut GameState, owner: PlayerId, zone_kind: ZoneKind, def: u32) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: if zone_kind == ZoneKind::Equip {
            CardKind::Weapon
        } else {
            CardKind::Shan
        },
        suit: Suit::Club,
        rank: 6,
        weapon_range: u8::from(zone_kind == ZoneKind::Equip),
        zone,
        owner,
        face_up: zone_kind == ZoneKind::Equip,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(
    card_zone: Option<ZoneKind>,
) -> (Transaction, PlayerId, PlayerId, PlayerId, Option<CardId>) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package())
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 2_026_072_411, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let other = players[2];
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
        skill_def: SkillDefId::new(STDYIMOU),
    });
    run_effects(&mut tx, effects).unwrap();
    let card = card_zone.map(|zone| inject(tx.state_mut(), owner, zone, 138_001));
    (tx, owner, target, other, card)
}

fn damage(owner: PlayerId, source: PlayerId, amount: u32) -> DomainEvent {
    DomainEvent::DamageDealt {
        source,
        target: owner,
        amount,
        is_lose_hp: false,
        nature: 0,
    }
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
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
                cancelled: false,
            },
        },
    )
}

fn trigger(mut tx: Transaction, owner: PlayerId, source: PlayerId) -> (GameState, PipelineResult) {
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[damage(owner, source, 1)],
            &mut TriggerFireRecord::new(),
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let committed = tx.commit();
    assert_eq!(
        committed.output.choice.as_ref().unwrap().kind,
        ChoiceKind::Confirm
    );
    resolve(committed.state, owner, vec![1], 1)
}

#[test]
fn package_publishes_abi138_with_complete_baoxin() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert_eq!(package.skill_index.len(), 181);
    assert_eq!(package.character_index.len(), 101);
    assert!(package.skill_index.iter().any(|skill| {
        skill.skill_def_id == STDYIMOU && skill.name == "stdyimou" && !skill.locked
    }));
    let baoxin = package
        .character_index
        .iter()
        .find(|character| character.id == STD_BAOXIN)
        .unwrap();
    assert_eq!(baoxin.name_key, "std_baoxin");
    assert_eq!(baoxin.max_hp, 4);
    assert_eq!(baoxin.skills, [STDYIMOU, STDMUTAO]);

    let compiled = content_compiler::compile_source_named("stdyimou", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 138);
    let expected = &compiled.modules[0];
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
}

#[test]
fn damage_choice_selects_other_then_gives_current_hand_card() {
    let (tx, owner, target, other, card) = setup(Some(ZoneKind::Hand));
    let card = card.unwrap();
    let (selecting_target, target_output) = trigger(tx, owner, other);
    assert!(matches!(
        target_output.result,
        CommandResult::Accepted { .. }
    ));
    let target_choice = target_output.choice.as_ref().unwrap();
    assert_eq!(target_choice.kind, ChoiceKind::SelectTargets);
    assert_eq!(
        target_choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [target.raw(), other.raw()]
    );

    let (selecting_card, card_output) = resolve(selecting_target, owner, vec![target.raw()], 2);
    assert!(matches!(card_output.result, CommandResult::Accepted { .. }));
    assert_eq!(
        selecting_card.resolution.continuation_kind,
        ContinuationKind::GiveOneHandOrEquip
    );
    let card_choice = card_output.choice.as_ref().unwrap();
    assert_eq!(card_choice.kind, ChoiceKind::SelectCards);
    assert_eq!(
        card_choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [card.raw()]
    );

    let target_before = selecting_card.hand_count(target);
    let (done, given) = resolve(selecting_card, owner, vec![card.raw()], 3);
    assert!(matches!(given.result, CommandResult::Accepted { .. }));
    assert_eq!(done.cards.get(card).unwrap().owner, target);
    assert_eq!(done.hand_count(target), target_before + 1);
    assert!(given.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardsMoved {
            cards,
            reason: CardMoveReason::Give,
            ..
        } if cards == &vec![card]
    )));
}

#[test]
fn equipment_is_eligible_but_no_card_or_zero_damage_does_not_trigger() {
    let (tx, owner, target, other, equip) = setup(Some(ZoneKind::Equip));
    let equip = equip.unwrap();
    let (selecting_target, _) = trigger(tx, owner, other);
    let (selecting_card, card_output) = resolve(selecting_target, owner, vec![target.raw()], 4);
    assert_eq!(
        card_output
            .choice
            .as_ref()
            .unwrap()
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [equip.raw()]
    );
    let (done, _) = resolve(selecting_card, owner, vec![equip.raw()], 5);
    assert_eq!(done.cards.get(equip).unwrap().owner, target);
    assert_eq!(
        done.cards.get(equip).unwrap().zone,
        done.zones.find(ZoneKind::Hand, target).unwrap()
    );

    let (mut empty, empty_owner, empty_target, _, _) = setup(None);
    assert!(dispatch_for_events(
        &mut empty,
        &[damage(empty_owner, empty_target, 1)],
        &mut TriggerFireRecord::new(),
    )
    .unwrap()
    .is_empty());
    assert!(dispatch_for_events(
        &mut empty,
        &[damage(empty_owner, empty_target, 0)],
        &mut TriggerFireRecord::new(),
    )
    .unwrap()
    .is_empty());
}
