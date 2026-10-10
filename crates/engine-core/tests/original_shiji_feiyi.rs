//! Golden for SHIJI 费祎「生息 / 谏喻」语义（ABI 425）。
//!
//! 生息：结束阶段本回合用过牌且未造成伤害时，可选获得智囊或摸牌（复用 TW 生息）。
//! 谏喻：出牌阶段限一次选 2 名角色获喻标记；出牌阶段喻标记角色用牌指定另一喻标记
//! 角色为目标时，目标摸 1。

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const FYJIANYU: u32 = 2_248_481_428;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/shiji/shiji.npack.json");

fn setup(players: u8) -> (Transaction, PlayerId, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(8_112),
            kind: CommandKind::StartGame {
                player_count: players,
                rng_seed: 20_260_812,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let seats = engine.state().players.alive_by_seat();
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
        skill_def: SkillDefId::new(FYJIANYU),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, seats)
}

fn resolve_choice(
    state: GameState,
    actor: PlayerId,
    key: u64,
    selections: Vec<u32>,
    cancelled: bool,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    next
}

/// 谏喻 mode 0：授予 2 名目标喻标记。
#[test]
fn fyjianyu_grants_marks_to_two_targets() {
    let (mut tx, owner, seats) = setup(3);
    let others: Vec<_> = seats.into_iter().filter(|p| *p != owner).collect();
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.def.raw() == FYJIANYU && s.owner == owner)
        .map(|s| s.id)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveFyjianyu {
            player: owner,
            source: PlayerId::NONE,
            target: PlayerId::NONE,
            targets: others.clone(),
            skill,
        }],
    )
    .unwrap();
    for target in &others {
        assert!(
            tx.state()
                .players
                .get(*target)
                .unwrap()
                .marks
                .iter()
                .any(|(mark, count)| *mark == FYJIANYU && *count == 1),
            "目标应获得喻标记"
        );
    }
}

/// 谏喻 mode 1：喻标记来源指定喻标记目标 → 目标摸 1。
#[test]
fn fyjianyu_marked_target_draws() {
    let (mut tx, owner, seats) = setup(3);
    let others: Vec<_> = seats.into_iter().filter(|p| *p != owner).collect();
    let source = others[0];
    let target = others[1];
    // 手动给 source 和 target 喻标记。
    for p in [source, target] {
        run_effects(
            &mut tx,
            vec![EffectRequest::AddMark {
                player: p,
                mark: engine_api::MarkDefId::new(FYJIANYU),
                delta: 1,
            }],
        )
        .unwrap();
    }
    let hand_before = tx.state().hand_count(target);
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.def.raw() == FYJIANYU && s.owner == owner)
        .map(|s| s.id)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveFyjianyu {
            player: owner,
            source,
            target,
            targets: Vec::new(),
            skill,
        }],
    )
    .unwrap();
    assert_eq!(
        tx.state().hand_count(target),
        hand_before + 1,
        "喻标记目标应摸 1 张"
    );
    let _ = CardId::NONE;
}
