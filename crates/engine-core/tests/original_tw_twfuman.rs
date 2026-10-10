//! Golden coverage for TW 马忠「抚蛮」.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::effective_card_kind;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWFUMAN: u32 = 2_838_266_052;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twfuman.skill");

fn inject(state: &mut GameState, owner: PlayerId, kind: CardKind, rank: u8) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_394_000 + u32::from(rank)),
        kind,
        suit: Suit::Heart,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn submit(state: GameState, actor: PlayerId, kind: CommandKind, key: u64) -> GameState {
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{:?}",
        result.result
    );
    state
}

#[test]
fn gifted_physical_card_becomes_sha_and_damage_draws_two_after_resolution() {
    let package = content_compiler::compile_source_named("twfuman-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 394);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 2_026_080_214, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, user, target) = (players[0], players[1], players[2]);
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
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(TWFUMAN),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let gift = inject(tx.state_mut(), owner, CardKind::Tao, 7);
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == TWFUMAN)
        .unwrap()
        .id;
    let state = submit(
        tx.commit().state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![user],
            card_targets: vec![gift],
        },
        1,
    );
    assert_eq!(
        effective_card_kind(&state, user, state.cards.get(gift).unwrap()),
        CardKind::Sha
    );
    let owner_hand = state.hand_count(owner);
    let target_hp = state.players.get(target).unwrap().hp;
    let mut state = state;
    state.turn.current = user;
    state.turn.phase = Phase::Play;
    let state = submit(
        state,
        user,
        CommandKind::PlayCard {
            card: gift,
            targets: vec![target],
            card_targets: Vec::new(),
        },
        2,
    );
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 1);
    assert_eq!(state.hand_count(owner), owner_hand + 2);
    assert_eq!(
        effective_card_kind(&state, user, state.cards.get(gift).unwrap()),
        CardKind::Tao
    );
}
