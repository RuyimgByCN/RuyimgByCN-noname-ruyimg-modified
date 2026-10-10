//! Golden coverage for BINGSHI 势太史慈「酣战」「战烈」「振锋」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const HANZHAN: u32 = 2_573_360_121;
const ZHANLIE: u32 = 3_829_642_123;
const ZHENFENG: u32 = 2_197_448_518;
const SOURCE: &str = include_str!("../../../content/original/ported/bingshi/pot_taishici.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup(seed: u64) -> (GameState, PlayerId, Vec<PlayerId>) {
    let package = content_compiler::compile_source(SOURCE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 4,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let order = engine.state().players.alive_by_seat();
    let owner = order[0];
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(
        [HANZHAN, ZHANLIE, ZHENFENG].map(|skill_def| EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill_def),
        }),
    );
    run_effects(&mut tx, effects).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let clears: Vec<_> = order
        .iter()
        .copied()
        .filter_map(|player| {
            let cards = tx.state().hand_cards(player);
            (!cards.is_empty()).then(|| EffectRequest::MoveCards {
                cards,
                to: discard,
                from: tx.state().zones.find(ZoneKind::Hand, player),
            })
        })
        .collect();
    run_effects(&mut tx, clears).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx.commit().state, owner, order)
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

fn inject_hand(tx: &mut Transaction, owner: PlayerId, serial: u32) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(548_000 + serial),
        kind: CardKind::Other,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn resolve(
    state: GameState,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let actor = PlayerId::new(state.resolution.pending_observer);
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn package_publishes_complete_pot_taishici_at_abi548() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    assert_eq!(package.character_index.len(), 36);
    assert_eq!(package.skill_index.len(), 103);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.name_key == "pot_taishici")
            .unwrap()
            .skills,
        [HANZHAN, ZHANLIE, ZHENFENG]
    );
}

#[test]
fn hanzhan_draws_each_side_to_max_hp_cap_three_then_duels() {
    let (state, owner, order) = setup(5481);
    let target = order[1];
    let target_hp = state.players.get(target).unwrap().hp;
    let (mut state, mut output) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(5482),
            kind: CommandKind::ActivateSkill {
                skill: skill(&state, owner, HANZHAN),
                targets: vec![target],
                card_targets: Vec::new(),
            },
        },
    );
    assert_eq!(state.hand_count(owner), 3);
    assert_eq!(state.hand_count(target), 3);
    let mut key = 5483;
    while state.has_pending_choice() {
        (state, output) = resolve(state, vec![0], false, key);
        key += 1;
    }
    assert!(
        matches!(output.result, CommandResult::Accepted { .. }),
        "{:?}",
        output.result
    );
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 1);
}

#[test]
fn zhanlie_spends_six_marks_for_damage_response_cost_and_post_sha_draw() {
    let (state, owner, order) = setup(5484);
    let target = order[1];
    let target_hp = state.players.get(target).unwrap().hp;
    let mut tx = Transaction::begin(state);
    let discard_card = inject_hand(&mut tx, target, 1);
    let zhanlie = skill(tx.state(), owner, ZHANLIE);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddMark {
            player: owner,
            mark: engine_api::MarkDefId::new(ZHANLIE),
            delta: 6,
        }],
    )
    .unwrap();
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePottaishici {
                player: owner,
                target: owner,
                action: 2,
                cards: Vec::new(),
                hanzhan_def: SkillDefId::new(HANZHAN),
                zhanlie_def: SkillDefId::new(ZHANLIE),
                skill: zhanlie,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let (state, output) = resolve(tx.commit().state, vec![target.raw()], false, 5485);
    assert!(
        matches!(output.result, CommandResult::Accepted { .. }),
        "target: {:?}",
        output.result
    );
    let (state, output) = resolve(state, vec![2, 4, 8], false, 5486);
    assert!(
        matches!(output.result, CommandResult::Accepted { .. }),
        "options: {:?}",
        output.result
    );
    let (mut state, mut output) = resolve(state, vec![discard_card.raw()], false, 5487);
    assert!(
        matches!(output.result, CommandResult::Accepted { .. }),
        "cost: {:?}",
        output.result
    );
    let mut key = 5488;
    while state.has_pending_choice() {
        (state, output) = resolve(state, Vec::new(), false, key);
        key += 1;
    }
    assert!(
        matches!(output.result, CommandResult::Accepted { .. }),
        "{:?}",
        output.result
    );
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 2);
    assert_eq!(state.hand_count(owner), 3);
    assert_eq!(state.hand_count(target), 0);
}

#[test]
fn zhenfeng_persists_independent_x_sources_for_both_skills() {
    let (state, owner, _) = setup(5490);
    let mut tx = Transaction::begin(state);
    let zhenfeng = skill(tx.state(), owner, ZHENFENG);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePottaishici {
                player: owner,
                target: owner,
                action: 4,
                cards: Vec::new(),
                hanzhan_def: SkillDefId::new(HANZHAN),
                zhanlie_def: SkillDefId::new(ZHANLIE),
                skill: zhenfeng,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let (state, _) = resolve(tx.commit().state, vec![1], false, 5491);
    let (state, output) = resolve(state, vec![11, 23], false, 5492);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state
            .skills
            .get(skill(&state, owner, HANZHAN))
            .unwrap()
            .state_blob[0],
        1
    );
    assert_eq!(
        state
            .skills
            .get(skill(&state, owner, ZHANLIE))
            .unwrap()
            .state_blob[0],
        3
    );
}
