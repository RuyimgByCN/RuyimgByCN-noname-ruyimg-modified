//! Golden coverage for 标董允「秉正」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const STDBINGZHENG: u32 = 3_798_362_737;
const STD_DONGYUN: u32 = 2_711_327_184;
const STDDULIANG: u32 = 2_231_518_236;
const MOVE_PROBE: u32 = 4_200_000_127;
const SOURCE: &str = include_str!("../../../../content/original/ported/sixiang/stdbingzheng.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/sixiang/sixiang.npack.json");

fn phase_end(player: PlayerId) -> DomainEvent {
    DomainEvent::TurnStateChanged {
        current: player,
        phase: TurnPhaseTag::End,
    }
}

fn setup(with_move_probe: bool) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("stdbingzheng-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    if with_move_probe {
        let probe = content_compiler::compile_source_named(
            "stdbingzheng-move-probe",
            r#"
skill move_probe(4200000127) locked {
  trigger cards_moved owner_from_field_empty {
    effect draw_cards(self, 1);
  }
}
"#,
        )
        .unwrap();
        std::sync::Arc::make_mut(&mut state.content)
            .load_package(&probe)
            .unwrap();
    }
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_724, "identity-standard").unwrap();
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
    effects.extend(
        players
            .iter()
            .map(|player| EffectRequest::DiscardAllHandEquipJudge { player: *player }),
    );
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(STDBINGZHENG),
    });
    if with_move_probe {
        effects.push(EffectRequest::AddSkill {
            player: target,
            skill_def: SkillDefId::new(MOVE_PROBE),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    for player in players {
        let participant = tx.state_mut().players.get_mut(player).unwrap();
        participant.hp = 4;
        participant.max_hp = 4;
    }
    tx.state_mut().turn.current = owner;
    (tx, owner, target, other)
}

fn inject(tx: &mut Transaction, owner: PlayerId, zone_kind: ZoneKind, def: u32) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: if zone_kind == ZoneKind::Equip {
            CardKind::Armor
        } else {
            CardKind::Other
        },
        suit: Suit::Club,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

fn offer(mut tx: Transaction, owner: PlayerId) -> GameState {
    assert_eq!(
        dispatch_for_events(&mut tx, &[phase_end(owner)], &mut TriggerFireRecord::new(),),
        Err(EffectError::ChoiceRequested)
    );
    tx.commit().state
}

#[test]
fn package_publishes_abi127_skill_and_complete_std_dongyun() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert_eq!(package.skill_index.len(), 181);
    assert_eq!(package.character_index.len(), 101);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == STDBINGZHENG)
        .unwrap();
    assert_eq!(trigger.event, "phase_end");
    assert_eq!(
        trigger.filter,
        "owner_is_target_any_alive_self_discardable_hand_equip"
    );
    assert!(trigger.optional);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == STD_DONGYUN)
        .unwrap();
    assert_eq!(character.name_key, "std_dongyun");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [STDBINGZHENG, STDDULIANG]);
}

#[test]
fn accepted_target_self_discards_then_owner_loses_hp_when_hand_differs() {
    let (mut tx, owner, target, _) = setup(false);
    let card = inject(&mut tx, target, ZoneKind::Hand, 127_001);
    tx.state_mut().players.get_mut(target).unwrap().hp = 2;
    let offered = offer(tx, owner);

    let (state, result) = resolve(offered, owner, vec![1], false, 1);
    let target_choice = result.choice.unwrap();
    assert_eq!(target_choice.kind, ChoiceKind::SelectTargets);
    assert_eq!(target_choice.observer, owner);
    assert_eq!(
        target_choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [target.raw()]
    );

    let (state, result) = resolve(state, owner, vec![target.raw()], false, 2);
    let card_choice = result.choice.unwrap();
    assert_eq!(card_choice.kind, ChoiceKind::SelectCards);
    assert_eq!(card_choice.observer, target);
    assert_eq!(
        card_choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [card.raw()]
    );

    let (resolved, result) = resolve(state, target, vec![card.raw()], false, 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(resolved.players.get(owner).unwrap().hp, 3);
    assert_eq!(resolved.hand_count(target), 0);
    let discard = resolved
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(resolved.cards.get(card).unwrap().zone, discard);
}

#[test]
fn post_discard_check_waits_for_nested_movement_triggers() {
    let (mut tx, owner, target, _) = setup(true);
    let equipment = inject(&mut tx, target, ZoneKind::Equip, 127_010);
    tx.state_mut().players.get_mut(target).unwrap().hp = 1;
    let offered = offer(tx, owner);
    let (state, _) = resolve(offered, owner, vec![1], false, 10);
    let (state, _) = resolve(state, owner, vec![target.raw()], false, 11);
    let (resolved, result) = resolve(state, target, vec![equipment.raw()], false, 12);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(resolved.hand_count(target), 1);
    assert_eq!(resolved.players.get(owner).unwrap().hp, 4);
}

#[test]
fn decline_and_no_eligible_target_preserve_state_without_extra_choices() {
    let (mut tx, owner, target, _) = setup(false);
    let card = inject(&mut tx, target, ZoneKind::Hand, 127_020);
    let offered = offer(tx, owner);
    let (declined, result) = resolve(offered, owner, vec![0], false, 20);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(declined.cards.get(card).unwrap().owner, target);
    assert_eq!(declined.players.get(owner).unwrap().hp, 4);

    let (mut empty, empty_owner, _, _) = setup(false);
    assert_eq!(
        dispatch_for_events(
            &mut empty,
            &[phase_end(empty_owner)],
            &mut TriggerFireRecord::new(),
        )
        .unwrap(),
        Vec::new()
    );
    assert!(!empty.state().has_pending_choice());
}
