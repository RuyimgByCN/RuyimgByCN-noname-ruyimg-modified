//! Golden semantics for EXTRA 神甘宁「魄袭」「劫营」。

use engine_api::{
    CardDefId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    ObserverKind, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::projection::project_observer;
use engine_core::query::{query, QueryContext};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const POXI: u32 = 3_008_932_273;
const POXI_LIMIT: u32 = 2_484_371_675;
const JIEYING: u32 = 2_400_922_959;
const JIEYING_MARK: u32 = 2_172_383_355;
const SHEN_GANNING: u32 = 3_481_630_433;
const EXTRA: &[u8] = include_bytes!("../../../content/original/ported/extra/extra.npack.json");
const STANDARD: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

fn setup(player_count: u8) -> (GameState, Vec<PlayerId>) {
    let standard = content_schema::ContentPackage::from_json_bytes(STANDARD).unwrap();
    let extra = content_schema::ContentPackage::from_json_bytes(EXTRA).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&standard).unwrap();
    engine.load_package(&extra).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(POXI as u64),
            kind: CommandKind::StartGame {
                player_count,
                rng_seed: POXI as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let mut key = 1;
    while engine.state().has_pending_choice() {
        let choice = engine.last_choice().unwrap().clone();
        assert!(matches!(
            engine.submit(Command {
                actor: choice.observer,
                expected_revision: ExpectedRevision(engine.state().revision),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id: choice.choice_id,
                    selections: Vec::new(),
                    cancelled: true,
                },
            }),
            CommandResult::Accepted { .. }
        ));
        key += 1;
    }
    let players = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let old = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, old).unwrap();
    (tx.state().clone(), players)
}

fn skill(state: &GameState, owner: PlayerId, def: u32) -> SkillId {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == def)
        .unwrap()
}

fn clear_hand(tx: &mut Transaction, player: PlayerId) {
    let cards = tx.state().hand_cards(player);
    if cards.is_empty() {
        return;
    }
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards,
            to: discard,
            from: Some(hand),
        }],
    )
    .unwrap();
}

fn inject(tx: &mut Transaction, owner: PlayerId, suit: Suit, rank: u8) -> engine_api::CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: CardDefId::new(99_000 + u32::from(rank)),
        kind: CardKind::Other,
        suit,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn resolve_choice(state: GameState, actor: PlayerId, cards: Vec<u32>, key: u64) -> GameState {
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: choice,
                selections: cards,
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn package_closes_shen_ganning_with_initial_hp_and_abi() {
    let package = content_schema::ContentPackage::from_json_bytes(EXTRA).unwrap();
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == SHEN_GANNING)
        .unwrap();
    assert_eq!((character.initial_hp, character.max_hp), (3, 6));
    assert_eq!(character.skills, [POXI, JIEYING]);
    content_runtime::check_engine_compat(
        &package.manifest,
        &content_runtime::tooling_host_versions(),
    )
    .unwrap();
    assert!(
        package
            .modifier_index
            .iter()
            .any(|modifier| modifier.skill_def_id == JIEYING
                && modifier.target == "global_draw_count")
    );
    assert!(package
        .modifier_index
        .iter()
        .any(|modifier| modifier.skill_def_id == JIEYING
            && modifier.target == "global_sha_use_limit"));
}

#[test]
fn poxi_reveals_target_hand_and_zero_own_cards_loses_max_hp() {
    let (state, players) = setup(2);
    let owner = players[0];
    let target = players[1];
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    clear_hand(&mut tx, owner);
    clear_hand(&mut tx, target);
    tx.state_mut().players.get_mut(owner).unwrap().max_hp = 6;
    let cards = [
        inject(&mut tx, target, Suit::Spade, 1),
        inject(&mut tx, target, Suit::Heart, 2),
        inject(&mut tx, target, Suit::Club, 3),
        inject(&mut tx, target, Suit::Diamond, 4),
    ];
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(POXI),
        }],
    )
    .unwrap();
    let poxi = skill(tx.state(), owner, POXI);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDrltPoxi {
                player: owner,
                target,
                hand_limit_skill_def: SkillDefId::new(POXI_LIMIT),
                stage: 0,
                own_count: 0,
                skill: poxi,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::ChooseDrltPoxiCards
    );
    let snapshot = project_observer(tx.state(), ObserverKind::Player(owner));
    assert!(cards
        .iter()
        .all(|card| snapshot.known_cards.iter().any(|known| known.id == *card)));
    let state = resolve_choice(
        tx.state().clone(),
        owner,
        cards.iter().map(|card| card.raw()).collect(),
        20,
    );
    assert_eq!(state.players.get(owner).unwrap().max_hp, 5);
    assert_eq!(state.hand_count(target), 0);
}

#[test]
fn poxi_post_discard_branches_keep_the_exact_legacy_results() {
    let (state, players) = setup(2);
    let owner = players[0];
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    clear_hand(&mut tx, owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDrltPoxi {
            player: owner,
            target: players[1],
            hand_limit_skill_def: SkillDefId::new(POXI_LIMIT),
            stage: 1,
            own_count: 1,
            skill: SkillId::NONE,
        }],
    )
    .unwrap();
    assert_ne!(tx.state().turn.phase, Phase::Play);
    assert_eq!(
        query(
            tx.state(),
            &QueryRequest::HandLimit { player: owner },
            &mut QueryContext::new(),
        ),
        QueryResponse::Int(tx.state().players.get(owner).unwrap().hp.max(0) - 1)
    );

    tx.state_mut().players.get_mut(owner).unwrap().hp = 1;
    tx.state_mut().players.get_mut(owner).unwrap().max_hp = 4;
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDrltPoxi {
            player: owner,
            target: players[1],
            hand_limit_skill_def: SkillDefId::new(POXI_LIMIT),
            stage: 1,
            own_count: 3,
            skill: SkillId::NONE,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 2);
    let hand = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDrltPoxi {
            player: owner,
            target: players[1],
            hand_limit_skill_def: SkillDefId::new(POXI_LIMIT),
            stage: 1,
            own_count: 4,
            skill: SkillId::NONE,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand + 4);
}

#[test]
fn jieying_stacks_per_owner_transfers_and_collects_the_marked_hand() {
    let (state, players) = setup(3);
    let owner = players[0];
    let second_owner = players[1];
    let target = players[2];
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = owner;
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(JIEYING),
            },
            EffectRequest::AddSkill {
                player: second_owner,
                skill_def: SkillDefId::new(JIEYING),
            },
        ],
    )
    .unwrap();
    let jieying = skill(tx.state(), owner, JIEYING);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDrltJieying {
            player: owner,
            event_player: owner,
            mark: engine_api::MarkDefId::new(JIEYING_MARK),
            mode: 0,
            skill: jieying,
        }],
    )
    .unwrap();
    tx.state_mut().turn.phase = Phase::End;
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDrltJieying {
                player: owner,
                event_player: owner,
                mark: engine_api::MarkDefId::new(JIEYING_MARK),
                mode: 1,
                skill: jieying,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let state = resolve_choice(tx.state().clone(), owner, vec![target.raw()], 30);
    assert_eq!(engine_core::query::draw_count_for(&state, target), 4);
    assert_eq!(engine_core::query::sha_use_limit_for(&state, target), 3);
    assert_eq!(
        query(
            &state,
            &QueryRequest::HandLimit { player: target },
            &mut QueryContext::new(),
        ),
        QueryResponse::Int(state.players.get(target).unwrap().hp.max(0) + 2)
    );

    let mut tx = Transaction::begin(state);
    clear_hand(&mut tx, owner);
    clear_hand(&mut tx, target);
    inject(&mut tx, target, Suit::Heart, 7);
    inject(&mut tx, target, Suit::Club, 8);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDrltJieying {
            player: owner,
            event_player: target,
            mark: engine_api::MarkDefId::new(JIEYING_MARK),
            mode: 2,
            skill: jieying,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(target), 0);
    assert_eq!(tx.state().hand_count(owner), 2);
    assert_eq!(engine_core::query::draw_count_for(tx.state(), target), 2);
}
