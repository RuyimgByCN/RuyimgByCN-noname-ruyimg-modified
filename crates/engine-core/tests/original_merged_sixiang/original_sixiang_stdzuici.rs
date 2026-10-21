//! Golden coverage for 标孙邵「罪辞」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const STD_SUNSHAO: u32 = 4_214_291_567;
const STDDINGYI: u32 = 2_968_274_563;
const STDZUICI: u32 = 2_244_504_256;
const SOURCE: &str = include_str!("../../../../content/original/ported/sixiang/stdzuici.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/sixiang/sixiang.npack.json");

fn damage(owner: PlayerId, source: PlayerId) -> DomainEvent {
    DomainEvent::DamageDealt {
        source,
        target: owner,
        amount: 1,
        is_lose_hp: false,
        nature: 0,
    }
}

fn setup() -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("stdzuici-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 202);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_726, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, source, donor) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(STDZUICI),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, source, donor)
}

fn inject_field(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    def: u32,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selection: u32,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![selection],
                cancelled: false,
            },
        },
    )
}

fn accept_trigger(
    mut tx: Transaction,
    owner: PlayerId,
    source: PlayerId,
    key: u64,
) -> (GameState, PipelineResult) {
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[damage(owner, source)],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    resolve(tx.commit().state, owner, 1, key)
}

#[test]
fn committed_package_publishes_complete_sunshao_and_abi202_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert_eq!(package.skill_index.len(), 181);
    assert_eq!(package.character_index.len(), 101);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == STDZUICI
            && trigger.event == "damage"
            && trigger.filter == "owner_is_target_source_movable_field_card"
            && trigger.optional
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == STD_SUNSHAO)
        .unwrap();
    assert_eq!(character.skills, [STDDINGYI, STDZUICI]);
}

#[test]
fn equipment_and_delayed_trick_are_moved_to_the_fixed_damage_source() {
    for (zone_kind, kind, def, key) in [
        (ZoneKind::Equip, CardKind::Weapon, 99_001, 1),
        (ZoneKind::Judge, CardKind::TrickLe, 99_002, 3),
    ] {
        let (mut tx, owner, source, donor) = setup();
        let moved = inject_field(&mut tx, donor, zone_kind, kind, def);
        let (state, offered) = accept_trigger(tx, owner, source, key);
        assert!(matches!(offered.result, CommandResult::Accepted { .. }));
        assert_eq!(
            state.resolution.continuation_kind,
            ContinuationKind::ChooseMoveFieldCardTo
        );
        assert_eq!(offered.choice.unwrap().options[0].token, moved.raw());
        let (state, completed) = resolve(state, owner, moved.raw(), key + 1);
        assert!(matches!(completed.result, CommandResult::Accepted { .. }));
        assert!(!state.has_pending_choice());
        let destination = state.zones.find(zone_kind, source).unwrap();
        assert_eq!(state.cards.get(moved).unwrap().zone, destination);
        assert_eq!(state.cards.get(moved).unwrap().owner, source);
    }
}

#[test]
fn occupied_equipment_slot_and_duplicate_delayed_trick_are_not_movable() {
    let (mut tx, owner, source, donor) = setup();
    inject_field(&mut tx, source, ZoneKind::Equip, CardKind::Armor, 99_003);
    inject_field(&mut tx, donor, ZoneKind::Equip, CardKind::Armor, 99_004);
    inject_field(
        &mut tx,
        source,
        ZoneKind::Judge,
        CardKind::TrickBingliang,
        99_005,
    );
    inject_field(
        &mut tx,
        donor,
        ZoneKind::Judge,
        CardKind::TrickBingliang,
        99_006,
    );
    assert!(dispatch_for_events(
        &mut tx,
        &[damage(owner, source)],
        &mut TriggerFireRecord::new(),
    )
    .unwrap()
    .is_empty());
}
