//! Golden coverage for TW 周处「果毅」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWGUOYI: u32 = 3_002_953_620;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twguoyi.skill");

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, rank: u8) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_371_000 + u32::from(rank)),
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

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
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
                selections,
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

fn play(state: GameState, source: PlayerId, card: CardId, target: PlayerId, key: u64) -> GameState {
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor: source,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card,
                targets: vec![target],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn choosing_both_branches_replays_the_second_card_after_original_resolution() {
    let package = content_compiler::compile_source_named("twguoyi-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 371);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 2_026_080_112, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (source, target) = (players[0], players[1]);
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
        player: source,
        skill_def: SkillDefId::new(TWGUOYI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = source;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().players.get_mut(source).unwrap().max_hp = 5;
    tx.state_mut().players.get_mut(source).unwrap().hp = 5;
    let tiesuo = inject(&mut tx, source, CardKind::TrickTiesuo, 1);
    let sha = inject(&mut tx, source, CardKind::Sha, 2);
    inject(&mut tx, source, CardKind::Tao, 3);
    inject(&mut tx, source, CardKind::Tao, 4);
    let discard = inject(&mut tx, target, CardKind::Tao, 5);
    inject(&mut tx, target, CardKind::Tao, 6);
    inject(&mut tx, target, CardKind::Tao, 7);
    inject(&mut tx, target, CardKind::Tao, 8);
    inject(&mut tx, target, CardKind::Tao, 9);
    let mut state = tx.commit().state;

    state = play(state, source, tiesuo, target, 1);
    // 当前实现：果毅触发后由目标直接选择分支（无独立 optional 确认段）。
    state = resolve(state, target, vec![0], 2); // choose hand-card lock
    assert!(state
        .players
        .get(target)
        .unwrap()
        .marks
        .iter()
        .any(|(mark, count)| *mark == engine_core::content::MARK_CHAIN && *count > 0));

    let hp = state.players.get(target).unwrap().hp;
    state = play(state, source, sha, target, 4);
    state = resolve(state, target, vec![1], 5); // choose discard branch
    assert_eq!(state.resolution.pending_card_replays.len(), 1);
    assert_eq!(
        state.resolution.pending_card_replays[0].kind,
        CardKind::Sha.tag()
    );
    assert_eq!(state.resolution.pending_card_replays[0].targets, [target]);
    assert_eq!(state.players.get(target).unwrap().hp, hp);
    state = resolve(state, target, vec![discard.raw()], 7);

    assert!(state.resolution.pending_card_replays.is_empty());
    assert_eq!(state.players.get(target).unwrap().hp, hp - 2);
    assert!(!state.hand_cards(target).contains(&discard));
    assert!(!state.has_pending_choice());
}
