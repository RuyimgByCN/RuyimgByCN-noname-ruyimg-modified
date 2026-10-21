//! Golden coverage for OFFLINE SP李傕「飞熊」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const NS_LIJUE: u32 = 3_250_593_432;
const NSFEIXIONG: u32 = 3_328_516_321;
const NSCESUAN: u32 = 3_127_862_270;
const NSFEIXIONG_WIN: u32 = 3_810_557_850;
const NSFEIXIONG_LOSE: u32 = 4_078_033_373;
const SOURCE: &str = include_str!("../../../../content/original/ported/offline/nsfeixiong.skill");
const OFFLINE: &[u8] =
    include_bytes!("../../../../content/original/ported/offline/offline.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    owner_card: CardId,
    target_card: CardId,
}

fn inject(state: &mut GameState, owner: PlayerId, def: u32, rank: u8) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Other,
        suit: Suit::Spade,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> Option<engine_api::SkillId> {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .find_map(|id| {
            state
                .skills
                .get(*id)
                .is_some_and(|skill| skill.def.raw() == def)
                .then_some(*id)
        })
}

fn setup(owner_rank: u8, target_rank: u8, seed: u64) -> Setup {
    let package = content_compiler::compile_source_named("nsfeixiong-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
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
        skill_def: SkillDefId::new(NSFEIXIONG),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    for player in &players {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 4;
        participant.max_hp = 6;
    }
    let owner_card = inject(tx.state_mut(), owner, 99_301, owner_rank);
    let target_card = inject(tx.state_mut(), target, 99_302, target_rank);
    Setup {
        state: tx.commit().state,
        owner,
        target,
        owner_card,
        target_card,
    }
}

fn open_play_phase(state: GameState) -> GameState {
    let current = state.turn.current;
    let mut tx = Transaction::begin(state);
    match dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current,
            phase: TurnPhaseTag::Play,
        }],
        &mut TriggerFireRecord::new(),
    ) {
        Ok(effects) => assert_eq!(
            run_effects(&mut tx, effects).unwrap_err(),
            EffectError::ChoiceRequested
        ),
        Err(error) => assert_eq!(error, EffectError::ChoiceRequested),
    }
    tx.commit().state
}

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let (state, result) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

fn compare(setup: Setup, key: u64) -> GameState {
    let state = open_play_phase(setup.state);
    let state = resolve(state, setup.owner, vec![1], key);
    let state = resolve(state, setup.owner, vec![setup.target.raw()], key + 1);
    let state = resolve(state, setup.owner, vec![setup.owner_card.raw()], key + 2);
    resolve(state, setup.target, vec![setup.target_card.raw()], key + 3)
}

#[test]
fn offline_package_publishes_complete_ns_lijue_without_an_abi_bump() {
    let package = content_schema::ContentPackage::from_json_bytes(OFFLINE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    assert_eq!(package.character_index.len(), 385);
    assert_eq!(package.skill_index.len(), 851);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == NS_LIJUE)
        .unwrap();
    assert_eq!(character.max_hp, 6);
    assert_eq!(character.initial_hp, 4);
    assert_eq!(character.skills, [NSFEIXIONG, NSCESUAN]);
    for skill in [NSFEIXIONG, NSFEIXIONG_WIN, NSFEIXIONG_LOSE] {
        assert!(package
            .skill_index
            .iter()
            .any(|entry| entry.skill_def_id == skill));
    }
}

#[test]
fn strict_winner_damages_loser_and_tie_only_cleans_up_the_helper() {
    for (owner_rank, target_rank, owner_loss, target_loss, key) in
        [(13, 1, 0, 1, 10), (1, 13, 1, 0, 20), (7, 7, 0, 0, 30)]
    {
        let setup = setup(owner_rank, target_rank, 2_026_072_900 + key);
        let owner = setup.owner;
        let target = setup.target;
        let state = compare(setup, key);
        assert_eq!(state.players.get(owner).unwrap().hp, 4 - owner_loss);
        assert_eq!(state.players.get(target).unwrap().hp, 4 - target_loss);
        assert!(skill_id(&state, owner, NSFEIXIONG_WIN).is_none());
        assert!(skill_id(&state, owner, NSFEIXIONG_LOSE).is_none());
        assert!(!state.has_pending_choice());
    }
}
