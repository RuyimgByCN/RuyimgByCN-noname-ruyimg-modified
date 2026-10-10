//! Golden coverage for standard 奸雄's physical damage-card gain contract.

use content_compiler::compile_source_named;
use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const JIANXIONG: u32 = 1;
const SOURCE: &str = include_str!("../../../content/standard/jianxiong.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId) {
    let package = compile_source_named("standard-jianxiong-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_725_1, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let source = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(JIANXIONG),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = 20;
    tx.state_mut().players.get_mut(owner).unwrap().max_hp = 20;
    (tx.commit().state, owner, source)
}

fn inject(state: &mut GameState, zone: ZoneKind, kind: CardKind, def: u32) -> CardId {
    let zone = state.zones.find(zone, PlayerId::NONE).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone,
        owner: PlayerId::NONE,
        face_up: true,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn deal_card_damage(
    state: GameState,
    source: PlayerId,
    target: PlayerId,
    card: CardId,
    kind: CardKind,
) -> (GameState, Result<Vec<engine_api::DomainEvent>, EffectError>) {
    let mut tx = Transaction::begin(state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
            card,
            card_kind: kind.tag(),
            card_flags: 0,
        }],
    );
    (tx.commit().state, result)
}

#[test]
fn package_and_runtime_gain_the_processing_damage_card() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 261);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == JIANXIONG
            && trigger.event == "damage"
            && trigger.filter == "owner_is_target_card_in_processing"
            && trigger.optional
    }));

    let (mut state, owner, source) = setup();
    let card = inject(&mut state, ZoneKind::Processing, CardKind::Sha, 170_001);
    let hand_before = state.hand_count(owner);
    let (state, result) = deal_card_damage(state, source, owner, card, CardKind::Sha);
    assert_eq!(result, Err(EffectError::ChoiceRequested));

    let (state, resolved) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand_before + 1);
    assert!(state.hand_cards(owner).contains(&card));
}

#[test]
fn cardless_or_already_discarded_damage_opens_no_choice() {
    let (state, owner, source) = setup();
    let hand_before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    let mut state = tx.commit().state;
    assert!(!state.has_pending_choice());

    let card = inject(&mut state, ZoneKind::DiscardPile, CardKind::Sha, 170_002);
    let (state, result) = deal_card_damage(state, source, owner, card, CardKind::Sha);
    assert!(result.is_ok());
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(owner), hand_before);
}
