//! Golden coverage for TW 乐就「催进」.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const CUIJIN: u32 = 2_582_559_793;
const SOURCE: &str = include_str!("../../../../content/original/ported/tw/cuijin.skill");

fn inject(state: &mut GameState, owner: PlayerId, kind: CardKind, rank: u8) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_395_000 + u32::from(rank)),
        kind,
        suit: Suit::Spade,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn setup(
    with_shan: bool,
) -> (
    GameState,
    PlayerId,
    PlayerId,
    CardId,
    CardId,
    Option<CardId>,
) {
    let package = content_compiler::compile_source_named("cuijin-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 395);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 2, 2_026_080_215, "identity-standard").unwrap();
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
            skill_def: SkillDefId::new(CUIJIN),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let sha = inject(tx.state_mut(), owner, CardKind::Sha, 7);
    let cost = inject(tx.state_mut(), owner, CardKind::Other, 8);
    let shan = with_shan.then(|| inject(tx.state_mut(), target, CardKind::Shan, 9));
    (tx.commit().state, owner, target, sha, cost, shan)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    )
}

fn play_and_accept(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    sha: CardId,
    cost: CardId,
) -> (GameState, PipelineResult) {
    let (state, played) = apply_command(
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
    // 上游 cuijin 的 cost 是单个 chooseToDiscard（选牌即可取消），无独立确认段。
    assert_eq!(played.choice.unwrap().kind, ChoiceKind::SelectCards);
    resolve(state, owner, vec![cost.raw()], 2)
}

#[test]
fn accepted_cuijin_adds_one_damage_and_clears_exact_sha_tracking() {
    let (state, owner, target, sha, cost, _) = setup(false);
    let target_hp = state.players.get(target).unwrap().hp;
    let owner_hp = state.players.get(owner).unwrap().hp;
    let (state, result) = play_and_accept(state, owner, target, sha, cost);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 2);
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp);
    let skill = state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == CUIJIN)
        .unwrap();
    assert!(skill.state_blob.is_empty());
}

#[test]
fn dodged_cuijin_draws_then_damages_the_sha_user() {
    let (state, owner, target, sha, cost, shan) = setup(true);
    let owner_hp = state.players.get(owner).unwrap().hp;
    let owner_hand = state.hand_count(owner);
    let (state, response) = play_and_accept(state, owner, target, sha, cost);
    let shan = shan.unwrap();
    assert_eq!(response.choice.unwrap().kind, ChoiceKind::SelectCards);
    let (state, result) = resolve(state, target, vec![shan.raw()], 4);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp - 1);
    assert_eq!(state.hand_count(owner), owner_hand - 1);
}
