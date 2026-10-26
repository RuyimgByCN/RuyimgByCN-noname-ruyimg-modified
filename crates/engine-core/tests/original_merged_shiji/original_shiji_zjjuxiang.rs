//! Golden for SHIJI 周姬「拒降」的脱离濒死限定技语义。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const ZJJUXIANG: u32 = 2_215_213_817;
const RESHENGXI: u32 = 2_846_213_948;
const XINJUXIANG: u32 = 3_450_353_814;
const XINGUIXIU: u32 = 3_238_150_014;
/// Concatenate every committed shiji source in pack order (alphabetical),
/// mirroring `content-cli pack` input concatenation so the logic hash matches
/// the committed npack regardless of future roster growth.
fn shiji_sources() -> String {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../content/original/ported/shiji");
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "skill"))
        .map(|entry| entry.path())
        .collect();
    files.sort();
    files
        .iter()
        .map(|path| {
            let mut source = std::fs::read_to_string(path).unwrap();
            source.push('\n');
            source
        })
        .collect()
}

const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/shiji/shiji.npack.json");

#[test]
fn saved_other_player_can_trigger_damage_then_max_hp_draw() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut compiled = content_compiler::compile_source_named("shiji", &shiji_sources()).unwrap();
    compiled.character_index = committed.character_index.clone();
    compiled.manifest.version = committed.manifest.version.clone();
    compiled.manifest.dependencies = committed.manifest.dependencies.clone();
    compiled.manifest.logic_hash = compiled.compute_logic_hash();
    assert_eq!(committed.manifest.logic_hash, compiled.manifest.logic_hash);
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == ZJJUXIANG));

    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_714,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(ZJJUXIANG),
        }],
    )
    .unwrap();
    {
        let target = tx.state_mut().players.get_mut(target).unwrap();
        target.hp = 0;
        target.max_hp = 4;
        target.dying = true;
    }
    let hand = tx.state().hand_count(owner);

    let paused = run_effects(
        &mut tx,
        vec![EffectRequest::RecoverHp {
            source: target,
            player: target,
            amount: 2,
        }],
    )
    .unwrap_err();
    assert_eq!(paused, EffectError::ChoiceRequested);

    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, 1);
    assert_eq!(state.hand_count(owner), hand + 4);
    assert!(!state
        .skills
        .sorted_live()
        .iter()
        .any(|skill| skill.def.raw() == ZJJUXIANG));

    let mut tx = Transaction::begin(engine.state().clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(XINJUXIANG),
        }],
    )
    .unwrap();
    {
        let target = tx.state_mut().players.get_mut(target).unwrap();
        target.hp = 0;
        target.max_hp = 4;
        target.dying = true;
    }
    let hand = tx.state().hand_count(owner);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::RecoverHp {
                source: target,
                player: target,
                amount: 2,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, 1);
    assert_eq!(state.hand_count(owner), hand);
    assert!(!state
        .skills
        .sorted_live()
        .iter()
        .any(|skill| skill.def.raw() == XINJUXIANG));
}

#[test]
fn reshengxi_draws_only_when_the_owner_dealt_no_damage_this_turn() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_714_2,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;

    let prepare = || {
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
            skill_def: SkillDefId::new(RESHENGXI),
        });
        run_effects(&mut tx, effects).unwrap();
        tx
    };

    let event = DomainEvent::TurnStateChanged {
        current: owner,
        phase: TurnPhaseTag::End,
    };
    let mut peaceful = prepare();
    let hand = peaceful.state().hand_count(owner);
    assert_eq!(
        dispatch_for_events(
            &mut peaceful,
            std::slice::from_ref(&event),
            &mut TriggerFireRecord::new(),
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let (state, result) = apply_command(
        peaceful.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(peaceful.state().revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(peaceful.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand + 2);

    let mut violent = prepare();
    violent
        .state_mut()
        .turn
        .set_flag(engine_core::state::turn_flag::DAMAGE_DEALT, 1);
    let effects =
        dispatch_for_events(&mut violent, &[event], &mut TriggerFireRecord::new()).unwrap();
    assert!(effects.is_empty());
    assert!(!violent.state().has_pending_choice());
}

#[test]
fn xinguixiu_uses_the_odd_draw_and_even_wounded_recover_branches() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_828),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_828,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;

    for (hp, max_hp, expected_draws, expected_hp) in
        [(3, 3, 1, 3), (2, 3, 0, 3), (2, 2, 0, 2), (3, 4, 1, 3)]
    {
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
            skill_def: SkillDefId::new(XINGUIXIU),
        });
        run_effects(&mut tx, effects).unwrap();
        {
            let owner_state = tx.state_mut().players.get_mut(owner).unwrap();
            owner_state.hp = hp;
            owner_state.max_hp = max_hp;
        }
        let hand = tx.state().hand_count(owner);
        run_effects(
            &mut tx,
            vec![EffectRequest::SetTurnState {
                current: owner,
                phase: engine_core::state::Phase::End.to_u8(),
            }],
        )
        .unwrap();
        assert_eq!(tx.state().hand_count(owner), hand + expected_draws);
        assert_eq!(tx.state().players.get(owner).unwrap().hp, expected_hp);
    }
}
