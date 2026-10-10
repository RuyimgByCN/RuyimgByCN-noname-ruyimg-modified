//! Golden coverage for BINGSHI 势桓阶「共谋」「正朔」 and derived「奇策」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const POTGONGMOU: u32 = 3_894_451_488;
const POTZHENGSHUO: u32 = 2_346_184_405;
const QICE: u32 = 3_567_101_194;
const KANPO: u32 = 2_834_927_312;
const HUANJIE: u32 = 2_873_796_786;
const BINGSHI: &[u8] =
    include_bytes!("../../../content/original/ported/bingshi/bingshi.npack.json");
const YIJIANG: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");
const SHENHUA: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");

fn setup(defs: &[u32]) -> (Transaction, PlayerId, Vec<PlayerId>) {
    let mut engine = Engine::new();
    for bytes in [YIJIANG, SHENHUA, BINGSHI] {
        engine
            .load_package(&content_schema::ContentPackage::from_json_bytes(bytes).unwrap())
            .unwrap();
    }
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(529),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: 529,
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
    effects.extend(defs.iter().copied().map(|def| EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, order)
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

fn inject(tx: &mut Transaction, owner: PlayerId, serial: u32) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(529_000 + serial),
        kind: CardKind::Shan,
        suit: Suit::Heart,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn choose(state: GameState, selection: u32, key: u64) -> GameState {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let revision = state.revision;
    let choice_id = state.resolution.pending_choice_id;
    let (state, output) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(choice_id),
                selections: vec![selection],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(output.result, CommandResult::Accepted { .. }),
        "{:?}",
        output.result
    );
    state
}

#[test]
fn package_publishes_huanjie_and_cross_package_derivations_at_abi529() {
    let package = content_schema::ContentPackage::from_json_bytes(BINGSHI).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.id == HUANJIE)
            .unwrap()
            .skills,
        [POTGONGMOU, POTZHENGSHUO]
    );
    assert!(package
        .manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.package_id == "yijiang"));
    assert!(package
        .manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.package_id == "shenhua"));
}

#[test]
fn gongmou_swaps_whole_hands_and_grants_qice_and_kanpo_until_turn_end() {
    let (mut tx, owner, order) = setup(&[POTGONGMOU]);
    let target = order.into_iter().find(|role| *role != owner).unwrap();
    clear_hand(&mut tx, owner);
    clear_hand(&mut tx, target);
    let owner_card = inject(&mut tx, owner, 1);
    let target_card = inject(&mut tx, target, 2);
    let gongmou = skill(tx.state(), owner, POTGONGMOU);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePothuanjie {
                player: owner,
                target: PlayerId::NONE,
                stage: 0,
                qice_def: SkillDefId::new(QICE),
                kanpo_def: SkillDefId::new(KANPO),
                skill: gongmou,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, target.raw(), 1);
    assert_eq!(state.hand_cards(owner), [target_card]);
    assert_eq!(state.hand_cards(target), [owner_card]);
    assert_eq!(skill(&state, owner, QICE).raw() != 0, true);
    assert_eq!(skill(&state, target, KANPO).raw() != 0, true);
}

#[test]
fn zhengshuo_discards_every_hand_then_reshuffles_and_deals_four() {
    let (mut tx, owner, order) = setup(&[POTZHENGSHUO]);
    let before = tx.state().draw_reshuffle_count;
    let zhengshuo = skill(tx.state(), owner, POTZHENGSHUO);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolvePothuanjie {
            player: owner,
            target: PlayerId::NONE,
            stage: 2,
            qice_def: SkillDefId::new(QICE),
            kanpo_def: SkillDefId::new(KANPO),
            skill: zhengshuo,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().draw_reshuffle_count, before + 1);
    assert!(order
        .into_iter()
        .all(|player| tx.state().hand_count(player) == 4));
}

#[test]
fn qice_consumes_every_original_hand_card_as_one_ordinary_trick() {
    let (mut tx, owner, _) = setup(&[QICE]);
    clear_hand(&mut tx, owner);
    let materials = [
        inject(&mut tx, owner, 10),
        inject(&mut tx, owner, 11),
        inject(&mut tx, owner, 12),
    ];
    let qice = skill(tx.state(), owner, QICE);
    let state = tx.commit().state;
    let revision = state.revision;
    let (mut state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(20),
            kind: CommandKind::UseSkillCard {
                skill: qice,
                card_kind: CardKind::TrickTaoyuan.tag(),
                targets: Vec::new(),
                equip_slot: 0,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let mut key = 21;
    while state.has_pending_choice() {
        let actor = PlayerId::new(state.resolution.pending_observer);
        let revision = state.revision;
        let choice_id = state.resolution.pending_choice_id;
        let (next, output) = apply_command(
            state,
            Command {
                actor,
                expected_revision: ExpectedRevision(revision),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id: ChoiceId::new(choice_id),
                    selections: Vec::new(),
                    cancelled: true,
                },
            },
        );
        assert!(matches!(output.result, CommandResult::Accepted { .. }));
        state = next;
        key += 1;
    }
    assert_eq!(
        materials
            .iter()
            .map(|card| {
                state
                    .zones
                    .get(state.cards.get(*card).unwrap().zone)
                    .unwrap()
                    .kind
            })
            .collect::<Vec<_>>(),
        vec![ZoneKind::DiscardPile; materials.len()]
    );
    assert_eq!(state.hand_count(owner), 0);
}
