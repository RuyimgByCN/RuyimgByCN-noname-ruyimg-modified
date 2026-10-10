//! Golden coverage for TW 宗预「御严」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWYUYAN: u32 = 3_341_156_482;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twyuyan.skill");

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, rank: u8) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_370_000 + u32::from(rank)),
        kind,
        suit: Suit::Diamond,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(
    gift_rank: Option<u8>,
    seed: u64,
) -> (GameState, PlayerId, PlayerId, CardId, Option<CardId>) {
    let package = content_compiler::compile_source_named("twyuyan-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 370);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (source, owner) = (players[0], players[1]);
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
        skill_def: SkillDefId::new(TWYUYAN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = source;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().players.get_mut(source).unwrap().hp = 4;
    tx.state_mut().players.get_mut(owner).unwrap().hp = 3;
    let sha = inject(&mut tx, source, CardKind::Sha, 5);
    let gift = gift_rank.map(|rank| inject(&mut tx, source, CardKind::Tao, rank));
    (tx.commit().state, source, owner, sha, gift)
}

fn play(state: GameState, source: PlayerId, owner: PlayerId, sha: CardId, key: u64) -> GameState {
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor: source,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![owner],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn higher_rank_gift_keeps_target_and_no_eligible_card_cancels_it() {
    let (state, source, owner, sha, gift) = setup(Some(8), 2_026_080_110);
    let hp = state.players.get(owner).unwrap().hp;
    let state = play(state, source, owner, sha, 1);
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let gift = gift.unwrap();
    let (state, result) = apply_command(
        state,
        Command {
            actor: source,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![gift.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(state.hand_cards(owner).contains(&gift));
    assert_eq!(state.players.get(owner).unwrap().hp, hp - 1);

    let (state, source, owner, sha, _) = setup(Some(2), 2_026_080_111);
    let hp = state.players.get(owner).unwrap().hp;
    let state = play(state, source, owner, sha, 3);
    assert_eq!(state.players.get(owner).unwrap().hp, hp);
    assert!(!state.has_pending_choice());
}
