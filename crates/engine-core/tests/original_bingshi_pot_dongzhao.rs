//! Golden coverage for BINGSHI 势董昭「妙略」「迎驾」与「瞒天过海」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::run_effects;
use engine_core::query::card_ignored_for_hand_limit;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const SPMIAOLVE: u32 = 2_347_949_240;
const SPYINGJIA: u32 = 3_012_325_144;
const SPYINGJIA_DRAW: u32 = 3_612_952_432;
const MANTIANGUOHAI: u32 = 3_442_665_819;
const SOURCE: &str = include_str!("../../../content/original/ported/bingshi/pot_dongzhao.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup(def: u32, seed: u64) -> (GameState, PlayerId, Vec<PlayerId>) {
    let package = content_compiler::compile_source(SOURCE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let order = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    clear_hands(&mut tx);
    (tx.commit().state, owner, order)
}

fn clear_hands(tx: &mut Transaction) {
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let effects = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter_map(|player| {
            let cards = tx.state().hand_cards(player);
            (!cards.is_empty()).then(|| EffectRequest::MoveCards {
                cards,
                to: discard,
                from: tx.state().zones.find(ZoneKind::Hand, player),
            })
        })
        .collect();
    run_effects(tx, effects).unwrap();
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, serial: u32) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(539_000 + serial),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn skill(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == def)
        .unwrap()
        .id
}

fn resolve(
    state: GameState,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let revision = state.revision;
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: choice,
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn package_publishes_dongzhao_and_mantianguohai_at_abi539() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    assert_eq!(package.character_index.len(), 36);
    assert_eq!(package.skill_index.len(), 103);
    assert_eq!(package.card_index.len(), 1);
    assert_eq!(package.card_index[0].card_def_id, MANTIANGUOHAI);
    assert_eq!(package.card_index[0].kind, "mantianguohai");
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.name_key == "pot_dongzhao")
            .unwrap()
            .skills,
        [SPMIAOLVE, SPYINGJIA]
    );
}

#[test]
fn miaolve_creates_distinct_suits_and_resolves_damage_reward() {
    let (state, owner, _) = setup(SPMIAOLVE, 5391);
    let mut tx = Transaction::begin(state);
    let skill = skill(tx.state(), owner, SPMIAOLVE);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveSpmiaolve {
            player: owner,
            action: 0,
            skill,
        }],
    )
    .unwrap();
    let cards: Vec<_> = tx
        .state()
        .hand_cards(owner)
        .into_iter()
        .filter(|card| tx.state().cards.get(*card).unwrap().kind == CardKind::TrickMantianguohai)
        .collect();
    assert_eq!(cards.len(), 2);
    assert_ne!(
        tx.state().cards.get(cards[0]).unwrap().suit,
        tx.state().cards.get(cards[1]).unwrap().suit
    );
    assert!(cards
        .iter()
        .all(|card| card_ignored_for_hand_limit(tx.state(), owner, *card)));
    let before = tx.state().hand_count(owner);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveSpmiaolve {
                player: owner,
                action: 1,
                skill,
            }],
        ),
        Err(engine_core::effect::EffectError::ChoiceRequested)
    ));
    let (state, output) = resolve(tx.commit().state, vec![0], false, 5392);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), before + 2);
}

#[test]
fn yingjia_requires_a_repeated_trick_then_queues_one_draw_turn() {
    let (state, owner, order) = setup(SPYINGJIA, 5393);
    let target = order.into_iter().find(|player| *player != owner).unwrap();
    let mut tx = Transaction::begin(state);
    let cost = inject(&mut tx, owner, CardKind::Other, 1);
    let trick = inject(&mut tx, owner, CardKind::TrickGuohe, 2);
    let owner_skill = skill(tx.state(), owner, SPYINGJIA);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::ResolveSpyingjia {
                player: owner,
                action: 0,
                helper_def: SkillDefId::new(SPYINGJIA_DRAW),
                card: CardId::NONE,
                card_kind: 0,
                target: PlayerId::NONE,
                skill: owner_skill,
            },
            EffectRequest::ResolveSpyingjia {
                player: owner,
                action: 1,
                helper_def: SkillDefId::new(SPYINGJIA_DRAW),
                card: trick,
                card_kind: CardKind::TrickGuohe.tag(),
                target: PlayerId::NONE,
                skill: owner_skill,
            },
            EffectRequest::ResolveSpyingjia {
                player: owner,
                action: 1,
                helper_def: SkillDefId::new(SPYINGJIA_DRAW),
                card: trick,
                card_kind: CardKind::TrickGuohe.tag(),
                target: PlayerId::NONE,
                skill: owner_skill,
            },
        ],
    )
    .unwrap();
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveSpyingjia {
                player: owner,
                action: 2,
                helper_def: SkillDefId::new(SPYINGJIA_DRAW),
                card: CardId::NONE,
                card_kind: 0,
                target: PlayerId::NONE,
                skill: owner_skill,
            }],
        ),
        Err(engine_core::effect::EffectError::ChoiceRequested)
    ));
    let (state, _) = resolve(tx.commit().state, vec![cost.raw()], false, 5394);
    let (state, output) = resolve(state, vec![target.raw()], false, 5395);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.extra_turn_queue, [target]);
    assert!(!state.hand_cards(owner).contains(&cost));
    let helper = skill(&state, target, SPYINGJIA_DRAW);
    let before = state.hand_count(target);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveSpyingjia {
            player: target,
            action: 3,
            helper_def: SkillDefId::new(SPYINGJIA_DRAW),
            card: CardId::NONE,
            card_kind: 0,
            target: PlayerId::NONE,
            skill: helper,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(target), before + 2);
    assert!(tx.state().skills.get(helper).is_none());
}

#[test]
fn mantianguohai_gains_then_gives_each_successful_target_in_seat_order() {
    let (state, actor, order) = setup(SPMIAOLVE, 5396);
    let targets: Vec<_> = order
        .into_iter()
        .filter(|player| *player != actor)
        .collect();
    let mut tx = Transaction::begin(state);
    let card = inject(&mut tx, actor, CardKind::TrickMantianguohai, 10);
    let gift_a = inject(&mut tx, actor, CardKind::Other, 11);
    let gift_b = inject(&mut tx, actor, CardKind::Other, 12);
    let take_a = inject(&mut tx, targets[0], CardKind::Other, 13);
    let take_b = inject(&mut tx, targets[1], CardKind::Other, 14);
    let state = tx.commit().state;
    let (state, output) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(5397),
            kind: CommandKind::PlayCard {
                card,
                targets: targets.clone(),
                card_targets: Vec::new(),
            },
        },
    );
    assert!(state.has_pending_choice(), "{:?}", output.result);
    let (state, _) = resolve(state, vec![take_a.raw()], false, 5398);
    let (state, _) = resolve(state, vec![take_b.raw()], false, 5399);
    let (state, _) = resolve(state, vec![gift_a.raw()], false, 5400);
    let (state, output) = resolve(state, vec![gift_b.raw()], false, 5401);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert!(state.hand_cards(actor).contains(&take_a));
    assert!(state.hand_cards(actor).contains(&take_b));
    assert!(state.hand_cards(targets[0]).contains(&gift_a));
    assert!(state.hand_cards(targets[1]).contains(&gift_b));
}
