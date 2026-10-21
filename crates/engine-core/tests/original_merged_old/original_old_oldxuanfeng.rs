//! Golden coverage for old「旋风」rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, SkillId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{turn_flag, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLDXUANFENG: u32 = 2_240_977_101;
const SOURCE: &str = include_str!("../../../../content/original/ported/old/oldxuanfeng.skill");
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/old/old.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
    target_hp: i32,
}

fn setup(clear_target_hand: bool) -> Setup {
    let package = content_compiler::compile_source_named("oldxuanfeng-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 2_026_072_810, "identity-standard").unwrap();
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLDXUANFENG),
    });
    if clear_target_hand {
        effects.push(EffectRequest::MoveCards {
            cards: tx.state().hand_cards(target),
            to: tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .unwrap(),
            from: tx.state().zones.find(ZoneKind::Hand, target),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = tx.state().players.get(owner).unwrap().skills[0];
    let target_hp = tx.state().players.get(target).unwrap().hp;
    Setup {
        state: tx.commit().state,
        owner,
        target,
        skill,
        target_hp,
    }
}

fn choose(state: GameState, actor: PlayerId, token: u32, key: u64) -> GameState {
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
                selections: vec![token],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

fn begin(setup: &Setup) -> GameState {
    let mut tx = Transaction::begin(setup.state.clone());
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveOldXuanfeng {
                player: setup.owner,
                target: PlayerId::NONE,
                card: engine_api::CardId::NONE,
                stage: 0,
                branch: 0,
                skill: setup.skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    choose(tx.commit().state, setup.owner, setup.target.raw(), 1)
}

#[test]
fn distance_one_target_can_take_direct_damage_or_an_uncounted_virtual_sha() {
    let direct = setup(false);
    let state = choose(begin(&direct), direct.owner, 2, 2);
    assert_eq!(
        state.players.get(direct.target).unwrap().hp,
        direct.target_hp - 1
    );

    let sha = setup(true);
    let state = choose(begin(&sha), sha.owner, 1, 3);
    assert_eq!(state.players.get(sha.target).unwrap().hp, sha.target_hp - 1);
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED), 0);
}

#[test]
fn package_publishes_oldxuanfeng_without_javascript() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 275);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == OLDXUANFENG && skill.name == "oldxuanfeng"));
    assert!(!serde_json::to_string(&package).unwrap().contains(".js"));
}
