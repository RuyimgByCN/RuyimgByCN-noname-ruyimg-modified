//! Golden coverage for 桃黄忠「益壮」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const TYYIZHUANG: u32 = 3_442_202_518;
const XINLIEGONG: u32 = 3_272_111_610;
const TY_HUANGZHONG: u32 = 4_203_825_178;
const SOURCE: &str = include_str!("../../../../content/original/ported/offline/tyyizhuang.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/offline/offline.npack.json");

fn setup() -> (Transaction, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("tyyizhuang-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, TYYIZHUANG as u64, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let other = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TYYIZHUANG),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    for player in [owner, other] {
        for card in tx.state().hand_cards(player) {
            tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Other;
        }
    }
    (tx, owner, other)
}

fn inject_judge(tx: &mut Transaction, owner: PlayerId, def: u32) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Judge, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Other,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn phase_prepare(player: PlayerId) -> DomainEvent {
    DomainEvent::TurnStateChanged {
        current: player,
        phase: TurnPhaseTag::Prepare,
    }
}

fn fire_prepare(tx: &mut Transaction, owner: PlayerId) -> Result<(), EffectError> {
    match dispatch_for_events(tx, &[phase_prepare(owner)], &mut TriggerFireRecord::new()) {
        Ok(effects) => run_effects(tx, effects).map(|_| ()),
        Err(error) => Err(error),
    }
}

fn resolve(
    state: GameState,
    actor: PlayerId,
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
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn package_publishes_complete_character_without_legacy_js() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    assert!(package.manifest.dependencies.iter().any(|dependency| {
        dependency.namespace == "official"
            && dependency.package_id == "shenhua"
            && dependency.version == "0.1.0"
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == TY_HUANGZHONG)
        .unwrap();
    assert_eq!(character.skills, [XINLIEGONG, TYYIZHUANG]);
    assert!(package.skill_index.iter().any(|skill| {
        skill.skill_def_id == TYYIZHUANG && skill.name == "tyyizhuang" && !skill.locked
    }));
    let serialized = serde_json::to_string(&package).unwrap();
    assert!(!serialized.contains(".js"));
    assert!(!serialized.contains("async content"));
}

#[test]
fn empty_judgment_zone_suppresses_the_optional_trigger() {
    let (mut tx, owner, _) = setup();
    fire_prepare(&mut tx, owner).unwrap();
    assert!(!tx.state().has_pending_choice());
}

#[test]
fn declining_preserves_hp_and_judgment_cards() {
    let (mut tx, owner, _) = setup();
    let judged = inject_judge(&mut tx, owner, 99_001);
    let hp = tx.state().players.get(owner).unwrap().hp;
    assert_eq!(
        fire_prepare(&mut tx, owner),
        Err(EffectError::ChoiceRequested)
    );
    let committed = tx.commit();
    let (state, result) = resolve(committed.state, owner, vec![], true, 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, hp);
    let judge = state.zones.find(ZoneKind::Judge, owner).unwrap();
    assert_eq!(state.zones.get(judge).unwrap().cards, [judged]);
}

#[test]
fn accepting_damages_owner_then_discards_only_current_judgment_cards() {
    let (mut tx, owner, _) = setup();
    let first = inject_judge(&mut tx, owner, 99_011);
    let second = inject_judge(&mut tx, owner, 99_012);
    let hand = tx.state().hand_cards(owner)[0];
    let hp = tx.state().players.get(owner).unwrap().hp;
    assert_eq!(
        fire_prepare(&mut tx, owner),
        Err(EffectError::ChoiceRequested)
    );
    let committed = tx.commit();
    let (state, result) = resolve(committed.state, owner, vec![1], false, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, hp - 1);
    let judge = state.zones.find(ZoneKind::Judge, owner).unwrap();
    assert!(state.zones.get(judge).unwrap().cards.is_empty());
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for card in [first, second] {
        assert_eq!(state.cards.get(card).unwrap().zone, discard);
    }
    assert!(state.hand_cards(owner).contains(&hand));
}

#[test]
fn judgment_discard_waits_until_the_self_damage_dying_save_finishes() {
    let (mut tx, owner, saver) = setup();
    let judged = inject_judge(&mut tx, owner, 99_021);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 1;
    let tao = tx.state().hand_cards(saver)[0];
    tx.state_mut().cards.get_mut(tao).unwrap().kind = CardKind::Tao;

    assert_eq!(
        fire_prepare(&mut tx, owner),
        Err(EffectError::ChoiceRequested)
    );
    let committed = tx.commit();
    let (dying, accepted) = resolve(committed.state, owner, vec![1], false, 3);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(
        dying.resolution.continuation_kind,
        ContinuationKind::DyingSave
    );
    assert_eq!(
        dying.cards.get(judged).unwrap().zone,
        dying.zones.find(ZoneKind::Judge, owner).unwrap()
    );

    let choice = accepted.choice.unwrap();
    assert_eq!(choice.observer, saver);
    let mut pending = Transaction::begin(dying);
    let late = inject_judge(&mut pending, owner, 99_022);
    let dying = pending.commit().state;
    let (state, saved) = resolve(dying, saver, vec![1], false, 4);
    assert!(matches!(saved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, 1);
    assert!(!state.has_pending_choice());
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for card in [judged, late] {
        assert_eq!(state.cards.get(card).unwrap().zone, discard);
    }
    assert_eq!(state.cards.get(tao).unwrap().zone, discard);
}
