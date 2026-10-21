//! Golden coverage for TW 周处「除害」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWCHUHAI: u32 = 3_782_702_310;
const SOURCE: &str = include_str!("../../../../content/original/ported/tw/twchuhai.skill");

fn inject(tx: &mut Transaction, owner: PlayerId, zone_kind: ZoneKind, rank: u8) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_372_000 + u32::from(rank)),
        kind: CardKind::Tao,
        suit: Suit::Club,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn resolve(state: GameState, actor: PlayerId, card: CardId, key: u64) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
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
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn success_disables_judge_zone_collects_clockwise_and_discards_each_new_card() {
    let package = content_compiler::compile_source_named("twchuhai-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 372);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 3, 2_026_080_113, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, first, second) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TWCHUHAI),
    });
    run_effects(&mut tx, effects).unwrap();
    let mission = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == TWCHUHAI)
        .unwrap()
        .id;
    let judge = inject(&mut tx, owner, ZoneKind::Judge, 1);
    let first_card = inject(&mut tx, first, ZoneKind::Hand, 2);
    let second_card = inject(&mut tx, second, ZoneKind::Equip, 3);
    tx.state_mut().turn.record_dying_caused(owner, first);
    tx.state_mut().turn.record_dying_caused(owner, second);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveTwchuhai {
            player: owner,
            action: 0,
            stage: 0,
            index: 0,
            event_cards: Vec::new(),
            skill: mission,
        }],
    );
    assert!(matches!(
        result,
        Err(engine_core::effect::EffectError::ChoiceRequested)
    ));
    let mut state = tx.commit().state;

    state = resolve(state, first, first_card, 1);
    state = resolve(state, owner, first_card, 2);
    state = resolve(state, second, second_card, 3);
    state = resolve(state, owner, second_card, 4);

    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(state.disabled_judge_zones.contains(&owner));
    assert_eq!(state.cards.get(judge).unwrap().zone, discard);
    assert!(state
        .skills
        .sorted_live()
        .iter()
        .any(|skill| skill.id == mission));
    assert_eq!(state.cards.get(first_card).unwrap().zone, discard);
    assert_eq!(state.cards.get(second_card).unwrap().zone, discard);
    assert!(!state.has_pending_choice());
}
