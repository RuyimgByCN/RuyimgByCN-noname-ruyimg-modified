//! Golden for XIANDING 马伶俐【硝引】（ABI 607）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DCXIAOYIN: u32 = 3_398_996_453;
const DCXIAOYIN_EFFECT: u32 = 3_398_996_454;
const SOURCE: &str = include_str!("../../../../content/original/ported/xianding/dcxiaoyin.skill");

fn add_to_draw(tx: &mut Transaction, suit: Suit, rank: u8) -> CardId {
    let zone = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(96_070 + u32::from(rank)),
        kind: CardKind::Sha,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner: PlayerId::NONE,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

#[test]
fn xiaoyin_gains_red_cards_and_places_black_card_on_the_chosen_target() {
    let package = content_compiler::compile_source_named("malingli-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DCXIAOYIN),
        }],
    )
    .unwrap();
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == DCXIAOYIN)
        .unwrap()
        .id;
    let black = add_to_draw(&mut tx, Suit::Club, 7);
    add_to_draw(&mut tx, Suit::Heart, 8);
    add_to_draw(&mut tx, Suit::Diamond, 9);
    let hand_before = tx.state().hand_count(owner);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveMalingli {
                player: owner,
                card: CardId::NONE,
                helper_skill_def: SkillDefId::new(DCXIAOYIN_EFFECT),
                action: 0,
                source: PlayerId::NONE,
                target: PlayerId::NONE,
                nature: 0,
                damage_id: 0,
                event_cards: Vec::new(),
                event_targets: Vec::new(),
                skill,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let revision = tx.state().revision;
    let choice_id = ChoiceId::new(tx.state().resolution.pending_choice_id);
    let (state, result) = apply_command(
        tx.commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(6071),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![target.raw()],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    assert_eq!(state.hand_count(owner), hand_before + 2);
    assert_eq!(
        state.skill_pile_cards(target, DCXIAOYIN_EFFECT),
        vec![black]
    );
    assert!(state
        .players
        .get(target)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == DCXIAOYIN_EFFECT)
        }));
}
