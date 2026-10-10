//! Golden coverage for TW 伏完「谋溃」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWMOUKUI: u32 = 3_462_273_565;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twfuwan.skill");

fn inject(state: &mut GameState, owner: PlayerId, kind: CardKind, def: u32) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn choose(state: GameState, actor: PlayerId, selection: u32, key: u64) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![selection],
                cancelled: false,
            },
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
fn backwater_draws_discards_then_target_retaliates_after_nonlethal_sha() {
    let package = content_compiler::compile_source_named("tw-fuwan-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 393);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 2_026_080_212, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target) = (players[0], players[1]);
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
            skill_def: SkillDefId::new(TWMOUKUI),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let sha = inject(tx.state_mut(), owner, CardKind::Sha, 99_031);
    let owner_spare = inject(tx.state_mut(), owner, CardKind::Other, 99_032);
    let target_card = inject(tx.state_mut(), target, CardKind::Other, 99_033);
    let target_hp = tx.state().players.get(target).unwrap().hp;
    let state = tx.commit().state;
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![target],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));

    let state = choose(state, owner, 3, 2);
    assert_eq!(state.resolution.pending_observer, owner.raw());
    let state = choose(state, owner, target_card.raw(), 3);
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 1);
    assert_eq!(state.resolution.pending_observer, target.raw());
    let state = choose(state, target, owner_spare.raw(), 4);
    assert!(!state.has_pending_choice());
    assert_eq!(
        state
            .zones
            .get(state.cards.get(owner_spare).unwrap().zone)
            .unwrap()
            .kind,
        ZoneKind::DiscardPile
    );
}
