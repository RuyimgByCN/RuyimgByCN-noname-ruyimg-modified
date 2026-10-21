//! Golden coverage for 殃众（sp2 赵忠）来源弃牌失体（ABI 460）.
//!
//! 拥有者造成或受到伤害后，伤害来源可弃置两张牌，令受伤角色失去 1 点体力。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const YANGZHONG: u32 = 4_088_525_000;
const SOURCE: &str = include_str!("../../../../content/original/ported/sp2/yangzhong.skill");

fn drive_choice(
    state: engine_core::state::GameState,
    actor: PlayerId,
    selection: u32,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![selection],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "choice must be accepted"
    );
    next
}

#[test]
fn damage_source_discards_two_then_target_loses_hp() {
    let package = content_compiler::compile_source_named("yangzhong-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 460);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 3, 2_026_080_115, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, source, _third) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(YANGZHONG),
    });
    run_effects(&mut tx, effects).unwrap();
    // Give the source two discardable hand cards.
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, source)
        .expect("source hand zone");
    let mut cards = Vec::new();
    for rank in 1..=2u32 {
        let card = tx.state_mut().cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(9_700),
            kind: CardKind::Sha,
            suit: engine_api::Suit::Heart,
            rank: rank as u8,
            weapon_range: 0,
            zone: hand,
            owner: source,
            face_up: false,
        });
        tx.state_mut()
            .zones
            .get_mut(hand)
            .expect("hand zone")
            .cards
            .push(card);
        cards.push(card);
    }
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    let source_hand_before = tx.state().hand_count(source);

    // Source deals 1 damage to the owner: source confirms, discards 2, owner loses 1 HP.
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    );
    assert!(matches!(result, Err(EffectError::ChoiceRequested)));
    let mut state = tx.commit().state;
    state = drive_choice(state, source, 1); // optional confirm by the damage source
                                            // Discard choice selects both cards in one command.
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state,
        Command {
            actor: source,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![cards[0].raw(), cards[1].raw()],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "discard choice must be accepted"
    );
    let state = next;
    assert_eq!(
        state.players.get(owner).unwrap().hp,
        owner_hp - 2,
        "damage target loses 1 HP on top of the 1 damage"
    );
    assert_eq!(
        state.hand_count(source),
        source_hand_before - 2,
        "damage source discards two cards"
    );
    assert!(!state.has_pending_choice());
}
