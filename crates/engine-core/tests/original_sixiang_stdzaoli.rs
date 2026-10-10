//! Golden coverage for 标孙翊「躁厉」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const STD_SUNYI: u32 = 2_572_349_242;
const STDZAOLI: u32 = 2_735_843_050;
const MOVEMENT_HELPER: u32 = 4_000_000_011;
const DRAW_HELPER: u32 = 4_000_000_012;
const SOURCE: &str = include_str!("../../../content/original/ported/sixiang/stdzaoli.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn phase_prepare(player: PlayerId) -> DomainEvent {
    DomainEvent::TurnStateChanged {
        current: player,
        phase: TurnPhaseTag::Prepare,
    }
}

fn setup(with_barrier_helpers: bool) -> (Transaction, PlayerId) {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 2_026_072_410, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package()).unwrap();
    if with_barrier_helpers {
        let helpers = content_compiler::compile_source(
            "skill stdzaoli_movement_test(4000000011) locked { trigger cards_moved owner_from_hand_empty { effect lose_hp(self, 1); } } skill stdzaoli_draw_test(4000000012) locked { trigger cards_drawn owner_is_target { effect recover_hp(self, 1); } }",
        )
        .unwrap();
        std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&helpers).unwrap();
    }
    let owner = tx.state().players.alive_by_seat()[0];
    let players = tx.state().players.alive_by_seat();
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
        skill_def: SkillDefId::new(STDZAOLI),
    });
    if with_barrier_helpers {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(MOVEMENT_HELPER),
        });
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DRAW_HELPER),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    let participant = tx.state_mut().players.get_mut(owner).unwrap();
    participant.hp = 4;
    participant.max_hp = 4;
    (tx, owner)
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
        suit: Suit::Heart,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn dispatch_prepare(
    tx: &mut Transaction,
    owner: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let effects =
        dispatch_for_events(tx, &[phase_prepare(owner)], &mut TriggerFireRecord::new()).unwrap();
    run_effects(tx, effects)
}

fn resolve_zone(state: GameState, owner: PlayerId, zone: u32) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![zone],
                cancelled: false,
            },
        },
    )
}

fn zone_cards(state: &GameState, owner: PlayerId, kind: ZoneKind) -> Vec<CardId> {
    let zone = state.zones.find(kind, owner).unwrap();
    state.zones.get(zone).unwrap().cards.clone()
}

#[test]
fn package_publishes_abi133_skill_and_complete_std_sunyi() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert_eq!(package.skill_index.len(), 181);
    assert_eq!(package.character_index.len(), 101);
    assert!(package.skill_index.iter().any(|skill| {
        skill.skill_def_id == STDZAOLI && skill.name == "stdzaoli" && skill.locked
    }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == STDZAOLI
            && trigger.event == "phase_prepare"
            && trigger.filter == "owner_is_target"
            && !trigger.optional
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == STD_SUNYI)
        .unwrap();
    assert_eq!(character.name_key, "std_sunyi");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "wu");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [STDZAOLI]);

    let compiled = content_compiler::compile_source_named("stdzaoli", SOURCE).unwrap();
    for expected in &compiled.modules {
        let actual = package
            .modules
            .iter()
            .find(|module| module.name == expected.name)
            .unwrap();
        assert_eq!(actual.logic_hash, expected.logic_hash);
    }
}

#[test]
fn two_nonempty_zones_force_a_zone_choice_and_only_discard_that_zone() {
    let (mut tx, owner) = setup(false);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 2;
    let hand_a = inject(&mut tx, owner, ZoneKind::Hand, 133_001);
    let hand_b = inject(&mut tx, owner, ZoneKind::Hand, 133_002);
    let armor = inject(&mut tx, owner, ZoneKind::Equip, 133_003);

    assert_eq!(
        dispatch_prepare(&mut tx, owner),
        Err(EffectError::ChoiceRequested)
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::ChooseDiscardAllHandOrEquipDrawLostHpThenLoseHp
    );
    let committed = tx.commit();
    let choice = committed.output.choice.unwrap();
    assert_eq!(choice.kind, ChoiceKind::Custom);
    assert!(!choice.allow_cancel);
    assert_eq!((choice.min_count, choice.max_count), (1, 1));
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [1, 2]
    );

    let (state, result) = resolve_zone(committed.state, owner, 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(owner).unwrap().hp, 1);
    assert_eq!(zone_cards(&state, owner, ZoneKind::Hand).len(), 4);
    assert_eq!(zone_cards(&state, owner, ZoneKind::Equip), [armor]);
    let discard = zone_cards(&state, PlayerId::NONE, ZoneKind::DiscardPile);
    assert!(discard.contains(&hand_a));
    assert!(discard.contains(&hand_b));
}

#[test]
fn discard_and_draw_triggers_finish_before_lost_hp_is_counted_and_final_hp_is_lost() {
    let (mut tx, owner) = setup(true);
    let hand = inject(&mut tx, owner, ZoneKind::Hand, 133_011);

    let events = dispatch_prepare(&mut tx, owner).unwrap();
    assert!(!tx.state().has_pending_choice());
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 3);
    assert_eq!(zone_cards(tx.state(), owner, ZoneKind::Hand).len(), 2);
    assert!(zone_cards(tx.state(), PlayerId::NONE, ZoneKind::DiscardPile).contains(&hand));

    let movement_loss = events
        .iter()
        .position(|event| {
            matches!(
                event,
                DomainEvent::DamageDealt {
                    target,
                    is_lose_hp: true,
                    ..
                } if *target == owner
            )
        })
        .unwrap();
    let draw = events
        .iter()
        .position(|event| {
            matches!(
                event,
                DomainEvent::CardsDrawn { player, cards }
                    if *player == owner && cards.len() == 2
            )
        })
        .unwrap();
    let recovery = events
        .iter()
        .position(|event| {
            matches!(
                event,
                DomainEvent::HpRecovered { player, amount, .. }
                    if *player == owner && *amount == 1
            )
        })
        .unwrap();
    let final_loss = events
        .iter()
        .rposition(|event| {
            matches!(
                event,
                DomainEvent::DamageDealt {
                    target,
                    is_lose_hp: true,
                    ..
                } if *target == owner
            )
        })
        .unwrap();
    assert!(movement_loss < draw);
    assert!(draw < recovery);
    assert!(recovery < final_loss);
}

#[test]
fn no_discardable_hand_or_equipment_cards_is_a_noop() {
    let (mut tx, owner) = setup(false);
    let before_hp = tx.state().players.get(owner).unwrap().hp;
    let before_hand = zone_cards(tx.state(), owner, ZoneKind::Hand);
    let events = dispatch_prepare(&mut tx, owner).unwrap();
    assert!(!events.iter().any(|event| {
        matches!(
            event,
            DomainEvent::CardsMoved { .. }
                | DomainEvent::CardsDrawn { .. }
                | DomainEvent::DamageDealt { .. }
        )
    }));
    assert!(!tx.state().has_pending_choice());
    assert_eq!(tx.state().players.get(owner).unwrap().hp, before_hp);
    assert_eq!(zone_cards(tx.state(), owner, ZoneKind::Hand), before_hand);
}
