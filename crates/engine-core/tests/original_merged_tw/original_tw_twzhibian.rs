//! Golden coverage for TW 宗预「直辩」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWZHIBIAN: u32 = 4_091_292_152;
const SOURCE: &str = include_str!("../../../../content/original/ported/tw/twzhibian.skill");

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    rank: u8,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_369_000 + u32::from(rank)),
        kind,
        suit: Suit::Club,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn choose(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
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

#[test]
fn win_backwater_moves_corresponding_zone_and_tie_or_loss_loses_hp() {
    let package = content_compiler::compile_source_named("twzhibian-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 369);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 2_026_080_109, "identity-standard").unwrap();
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TWZHIBIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().players.get_mut(owner).unwrap().hp = 2;
    let owner_high = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Sha, 13);
    let cost = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Armor, 2);
    let target_low = inject(&mut tx, target, ZoneKind::Hand, CardKind::Shan, 1);
    let judge = inject(&mut tx, target, ZoneKind::Judge, CardKind::TrickLe, 6);
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.def.raw() == TWZHIBIAN)
        .unwrap()
        .id;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwzhibian {
                player: owner,
                target: PlayerId::NONE,
                winner: PlayerId::NONE,
                stage: 0,
                skill
            }]
        ),
        Err(EffectError::ChoiceRequested)
    );
    let mut state = tx.commit().state;
    state = choose(state, owner, vec![target.raw()], 1);
    state = choose(state, owner, vec![owner_high.raw()], 2);
    state = choose(state, target, vec![target_low.raw()], 3);
    state = choose(state, owner, vec![3], 4);
    state = choose(state, owner, vec![cost.raw()], 5);
    state = choose(state, owner, vec![judge.raw()], 6);
    assert_eq!(state.players.get(owner).unwrap().hp, 3);
    assert!(state
        .zones
        .get(state.zones.find(ZoneKind::Judge, owner).unwrap())
        .unwrap()
        .cards
        .contains(&judge));

    let mut tx = Transaction::begin(state);
    let owner_low = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Sha, 1);
    let target_high = inject(&mut tx, target, ZoneKind::Hand, CardKind::Shan, 13);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwzhibian {
                player: owner,
                target: PlayerId::NONE,
                winner: PlayerId::NONE,
                stage: 0,
                skill
            }]
        ),
        Err(EffectError::ChoiceRequested)
    );
    let mut state = tx.commit().state;
    state = choose(state, owner, vec![target.raw()], 7);
    state = choose(state, owner, vec![owner_low.raw()], 8);
    state = choose(state, target, vec![target_high.raw()], 9);
    assert_eq!(state.players.get(owner).unwrap().hp, 2);
}
