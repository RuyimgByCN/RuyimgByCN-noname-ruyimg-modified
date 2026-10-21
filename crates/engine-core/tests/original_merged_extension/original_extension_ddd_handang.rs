//! Golden for 3D精选 韩当【险袭】（ABI 617）。

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DDDXIANXI: u32 = 3_400_504_078;
const STATE_MAGIC: u32 = 0xDDD0_6170;
const SOURCE: &str = include_str!("../../../../content/original/ported/ddd/handang.skill");

fn setup(seed: u64) -> (GameState, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("ddd-handang", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
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
        player: players[0],
        skill_def: SkillDefId::new(DDDXIANXI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = players[0];
    tx.state_mut().turn.phase = Phase::Play;
    (tx.commit().state, players)
}

fn inject(state: &mut GameState, owner: PlayerId, kind: CardKind, serial: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_000 + serial),
        kind,
        suit: Suit::Club,
        rank: 8,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn skill(state: &GameState, owner: PlayerId) -> engine_api::SkillId {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|id| {
            state
                .skills
                .get(*id)
                .is_some_and(|skill| skill.def.raw() == DDDXIANXI)
        })
        .unwrap()
}

fn command(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn choose(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    command(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled,
        },
        key,
    )
}

fn track(
    state: &mut GameState,
    owner: PlayerId,
    card: CardId,
    targets: &[PlayerId],
) -> engine_api::SkillId {
    let skill = skill(state, owner);
    let blob = &mut state.skills.get_mut(skill).unwrap().state_blob;
    blob.clear();
    blob.extend_from_slice(&STATE_MAGIC.to_le_bytes());
    blob.extend_from_slice(&card.raw().to_le_bytes());
    blob.extend_from_slice(&(targets.len() as u32).to_le_bytes());
    for target in targets {
        blob.extend_from_slice(&target.raw().to_le_bytes());
    }
    skill
}

#[test]
fn package_exposes_exact_abi_and_trigger_contract() {
    let package = content_compiler::compile_source_named("ddd-handang", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 617);
    assert_eq!(package.skill_index.len(), 1);
    assert_eq!(package.trigger_index.len(), 2);
}

#[test]
fn xianxi_adds_multiple_sha_targets_while_ignoring_distance() {
    let (mut state, players) = setup(20_260_803);
    let owner = players[0];
    let sha = inject(&mut state, owner, CardKind::Sha, 1);
    let hp: Vec<_> = players
        .iter()
        .map(|player| state.players.get(*player).unwrap().hp)
        .collect();
    let (state, result) = command(
        state,
        owner,
        CommandKind::PlayCard {
            card: sha,
            targets: vec![players[1]],
            card_targets: Vec::new(),
        },
        10,
    );
    assert_eq!(result.choice.unwrap().observer, owner);
    let (state, result) = choose(state, owner, vec![1], false, 11);
    let choice = result.choice.unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectTargets);
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == players[2].raw()));
    let (state, result) = choose(
        state,
        owner,
        vec![players[2].raw(), players[3].raw()],
        false,
        12,
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    assert!(result.choice.is_none());
    assert!(state
        .skills
        .get(skill(&state, owner))
        .unwrap()
        .state_blob
        .is_empty());
    assert_eq!(state.players.get(players[1]).unwrap().hp, hp[1] - 1);
    assert_eq!(state.players.get(players[2]).unwrap().hp, hp[2] - 1);
    assert_eq!(state.players.get(players[3]).unwrap().hp, hp[3] - 1);
}

#[test]
fn xianxi_settlement_supports_discard_damage_and_cancel_draw_loss() {
    let (mut state, players) = setup(20_260_804);
    let owner = players[0];
    let card = inject(&mut state, owner, CardKind::Sha, 2);
    let cost = inject(&mut state, owner, CardKind::Shan, 3);
    let tracked = [players[2]];
    let skill = track(&mut state, owner, card, &tracked);
    let hp = state.players.get(players[2]).unwrap().hp;
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddXianxi {
                player: owner,
                card,
                action: 1,
                event_targets: tracked.to_vec(),
                damaged_targets: Vec::new(),
                skill,
            }],
        ),
        Err(engine_core::effect::EffectError::ChoiceRequested)
    );
    let state = tx.commit().state;
    let (state, result) = choose(state, owner, vec![cost.raw()], false, 20);
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    assert_eq!(state.players.get(players[2]).unwrap().hp, hp - 1);
    assert!(!state.hand_cards(owner).contains(&cost));

    let (mut state, players) = setup(20_260_805);
    let owner = players[0];
    let card = inject(&mut state, owner, CardKind::Sha, 4);
    inject(&mut state, owner, CardKind::Shan, 5);
    let tracked = [players[1], players[2]];
    let skill = track(&mut state, owner, card, &tracked);
    let hp = state.players.get(owner).unwrap().hp;
    let hand = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddXianxi {
                player: owner,
                card,
                action: 1,
                event_targets: tracked.to_vec(),
                damaged_targets: Vec::new(),
                skill,
            }],
        ),
        Err(engine_core::effect::EffectError::ChoiceRequested)
    );
    let state = tx.commit().state;
    let (state, result) = choose(state, owner, Vec::new(), true, 30);
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    assert_eq!(state.players.get(owner).unwrap().hp, hp - 1);
    assert_eq!(state.hand_count(owner), hand + 2);
}
