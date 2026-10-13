//! O2 全量合并第二批（2026-10-14）：家族 consecutive 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_consecutive_card_draw {
//! Golden tests for legacy skills that compare a used card with the previous one.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const MOBILE_FENYIN: u32 = 3_469_868_183;
const IWASAWA_FENYIN: u32 = 2_317_205_314;
const JIANYING: u32 = 3_679_198_172;
const MOBILE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");
const KEY: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");
const YIJIANG: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

fn setup(package: &[u8], skill_def: u32) -> (Transaction, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(package).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == skill_def && !skill.locked));
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(skill_def as u64),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: skill_def as u64,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
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
        skill_def: SkillDefId::new(skill_def),
    });
    effects.push(EffectRequest::SetTurnState {
        current: owner,
        phase: Phase::Play.to_u8(),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner)
}

fn inject(tx: &mut Transaction, owner: PlayerId, suit: Suit, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(90_000 + u32::from(rank)),
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

fn use_card(
    tx: &mut Transaction,
    owner: PlayerId,
    suit: Suit,
    rank: u8,
) -> Result<(), EffectError> {
    let card = inject(tx, owner, suit, rank);
    run_effects(
        tx,
        vec![EffectRequest::UseCard {
            player: owner,
            card,
            targets: vec![],
        }],
    )
    .map(|_| ())
}

fn resolve(
    tx: Transaction,
    owner: PlayerId,
    accept: bool,
    key: u64,
) -> engine_core::state::GameState {
    let state = tx.state();
    let (state, result) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections: if accept { vec![1] } else { vec![] },
                cancelled: !accept,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

fn assert_fenyin(package: &[u8], skill_def: u32) {
    let (mut tx, owner) = setup(package, skill_def);
    let initial_hand = tx.state().hand_count(owner);
    use_card(&mut tx, owner, Suit::Spade, 7).unwrap();
    use_card(&mut tx, owner, Suit::Club, 8).unwrap();
    assert_eq!(tx.state().hand_count(owner), initial_hand);

    let paused = use_card(&mut tx, owner, Suit::Heart, 9).unwrap_err();
    assert_eq!(paused, EffectError::ChoiceRequested);
    let state = resolve(tx, owner, true, skill_def as u64 + 1);
    assert_eq!(state.hand_count(owner), initial_hand + 1);

    let mut tx = Transaction::begin(state);
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: other,
                phase: Phase::Prepare.to_u8(),
            },
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::Play.to_u8(),
            },
        ],
    )
    .unwrap();
    use_card(&mut tx, owner, Suit::Spade, 10).unwrap();
    assert_eq!(tx.state().hand_count(owner), initial_hand + 1);
}

#[test]
fn both_fenyin_versions_compare_color_with_the_previous_turn_card() {
    assert_fenyin(MOBILE, MOBILE_FENYIN);
    assert_fenyin(KEY, IWASAWA_FENYIN);
}

#[test]
fn jianying_matches_rank_or_suit_only_in_the_same_play_phase() {
    let (mut tx, owner) = setup(YIJIANG, JIANYING);
    let initial_hand = tx.state().hand_count(owner);
    use_card(&mut tx, owner, Suit::Spade, 7).unwrap();

    let paused = use_card(&mut tx, owner, Suit::Heart, 7).unwrap_err();
    assert_eq!(paused, EffectError::ChoiceRequested);
    let state = resolve(tx, owner, true, JIANYING as u64 + 1);
    assert_eq!(state.hand_count(owner), initial_hand + 1);

    let mut tx = Transaction::begin(state);
    let paused = use_card(&mut tx, owner, Suit::Heart, 8).unwrap_err();
    assert_eq!(paused, EffectError::ChoiceRequested);
    let state = resolve(tx, owner, false, JIANYING as u64 + 2);
    assert_eq!(state.hand_count(owner), initial_hand + 1);

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::End.to_u8(),
            },
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::Play.to_u8(),
            },
        ],
    )
    .unwrap();
    use_card(&mut tx, owner, Suit::Heart, 9).unwrap();
    assert_eq!(tx.state().hand_count(owner), initial_hand + 1);
}
}
