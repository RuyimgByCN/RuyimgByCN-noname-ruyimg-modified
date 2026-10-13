//! O2 全量合并第二批（2026-10-14）：家族 faen 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_faen_family {
//! Golden coverage for the four legacy「法恩」variants and typed character-state filters.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, MarkDefId,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const OLDFAEN: u32 = 3_151_099_816;
const REFAEN: u32 = 3_292_191_694;
const DCFAEN: u32 = 2_290_375_740;
const FAEN: u32 = 2_817_547_947;
const OLD: &[u8] = include_bytes!("../../../content/original/ported/old/old.npack.json");
const REFRESH: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");
const YIJIANG: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

fn isolated(package_bytes: &[u8], skill_def: u32) -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(package_bytes).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(skill_def as u64),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: skill_def as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let players = engine.state().players.alive_by_seat();
    let (owner, target) = (players[0], players[1]);
    let mut tx = Transaction::begin(engine.state().clone());
    let mut setup: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    setup.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, setup).unwrap();
    (tx, owner, target)
}

fn accept(tx: Transaction, owner: PlayerId, key: u64) -> engine_core::state::GameState {
    let (state, pipeline) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    state
}

fn assert_trigger(package: &content_schema::ContentPackage, skill_def: u32, expected_filter: &str) {
    assert!(package.manifest.rule_vm_abi_version >= 27);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == skill_def)
        .unwrap();
    assert_eq!(trigger.event, "character_state_changed");
    assert_eq!(trigger.filter, expected_filter);
    assert!(trigger.optional);
}

#[test]
fn committed_packages_declare_all_four_faen_variants() {
    let old = content_schema::ContentPackage::from_json_bytes(OLD).unwrap();
    assert_trigger(
        &old,
        OLDFAEN,
        "owner_alive_event_player_turn_changed_or_became_linked",
    );
    let refresh = content_schema::ContentPackage::from_json_bytes(REFRESH).unwrap();
    assert_trigger(
        &refresh,
        REFAEN,
        "owner_alive_event_player_turn_changed_or_became_linked",
    );
    assert_trigger(
        &refresh,
        DCFAEN,
        "owner_alive_event_player_became_face_up_or_linked",
    );
    let yijiang = content_schema::ContentPackage::from_json_bytes(YIJIANG).unwrap();
    assert_trigger(
        &yijiang,
        FAEN,
        "owner_alive_event_player_became_face_up_or_linked",
    );
}

#[test]
fn oldfaen_fires_for_either_turn_direction_and_only_when_becoming_linked() {
    let (mut tx, owner, target) = isolated(OLD, OLDFAEN);
    let before = tx.state().hand_count(target);
    assert_eq!(
        run_effects(&mut tx, vec![EffectRequest::TurnOver { player: target }]).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = accept(tx, owner, 1);
    assert!(state.players.get(target).unwrap().turned_over);
    assert_eq!(state.hand_count(target), before + 1);

    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(&mut tx, vec![EffectRequest::TurnOver { player: target }]).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = accept(tx, owner, 2);
    assert!(!state.players.get(target).unwrap().turned_over);
    assert_eq!(state.hand_count(target), before + 2);

    let chain = MarkDefId::new(engine_core::content::MARK_CHAIN);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::AddMark {
                player: target,
                mark: chain,
                delta: 1,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = accept(tx, owner, 3);
    assert_eq!(state.hand_count(target), before + 3);

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::RemoveMark {
            player: target,
            mark: chain,
            amount: 1,
        }],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());
    assert_eq!(tx.state().hand_count(target), before + 3);
}

#[test]
fn dcfaen_only_fires_when_becoming_face_up_or_linked() {
    let (mut tx, owner, target) = isolated(REFRESH, DCFAEN);
    let before = tx.state().hand_count(target);
    run_effects(&mut tx, vec![EffectRequest::TurnOver { player: target }]).unwrap();
    assert!(tx.state().players.get(target).unwrap().turned_over);
    assert!(!tx.state().has_pending_choice());

    assert_eq!(
        run_effects(&mut tx, vec![EffectRequest::TurnOver { player: target }]).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = accept(tx, owner, 4);
    assert_eq!(state.hand_count(target), before + 1);

    let chain = MarkDefId::new(engine_core::content::MARK_CHAIN);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::AddMark {
                player: target,
                mark: chain,
                delta: 1,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = accept(tx, owner, 5);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::RemoveMark {
            player: target,
            mark: chain,
            amount: 1,
        }],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());
    assert_eq!(tx.state().hand_count(target), before + 2);
}
}
