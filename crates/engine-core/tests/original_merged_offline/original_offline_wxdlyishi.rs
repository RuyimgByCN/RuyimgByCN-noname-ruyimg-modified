//! Golden coverage for OFFLINE 文心雕龙「遗世」(wxdlyishi).
//! Once per game, the owner picks any other living players; each draws the owner's
//! current lost HP. A follow-up choice grants wxdlfangzhu to one other player.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, SkillId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const WXDLYISHI: u32 = 3_108_822_644;
const WXDLFANGZHU: u32 = 2_892_907_842;
const SOURCE: &str = include_str!("../../../../content/original/ported/offline/wxdlyishi.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/offline/offline.npack.json");

fn setup() -> (GameState, PlayerId, Vec<PlayerId>, SkillId) {
    let package = content_compiler::compile_source_named("wxdlyishi-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 20_260_815_91, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let others: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in tx.state().players.alive_by_seat() {
        effects.push(EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(WXDLYISHI),
    });
    run_effects(&mut tx, effects).unwrap();
    {
        let owner_state = tx.state_mut().players.get_mut(owner).unwrap();
        owner_state.max_hp = 6;
        owner_state.hp = 3;
    }
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == WXDLYISHI)
        .unwrap();
    (tx.commit().state, owner, others, skill)
}

fn activate(
    state: GameState,
    owner: PlayerId,
    targets: &[PlayerId],
    skill: SkillId,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: targets.to_vec(),
                card_targets: vec![],
            },
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
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
    )
}

fn has_def(state: &GameState, player: PlayerId, def: u32) -> bool {
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == def)
        })
}

#[test]
fn compiled_source_and_npack_publish_the_yishi_contract() {
    let compiled = content_compiler::compile_source_named("wxdlyishi", SOURCE).unwrap();
    let active = compiled
        .active_index
        .iter()
        .find(|active| active.skill_def_id == WXDLYISHI)
        .expect("wxdlyishi active");
    assert!(active.once_per_game);
    assert_eq!(active.window, "play_phase");
    let targets = active.targets.as_ref().unwrap();
    assert_eq!((targets.min, targets.max), (1, 255));
    assert!(!targets.allow_self);

    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == WXDLYISHI && skill.name == "wxdlyishi"));
    let pack_active = committed
        .active_index
        .iter()
        .find(|active| active.skill_def_id == WXDLYISHI)
        .expect("npack wxdlyishi active");
    assert!(pack_active.once_per_game);
    let actual = committed
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .expect("npack module for wxdlyishi");
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn selected_others_draw_owner_lost_hp_then_one_gains_fangzhu() {
    let (state, owner, others, skill) = setup();
    assert_eq!(others.len(), 2);
    let before: Vec<_> = others.iter().map(|p| state.hand_count(*p)).collect();
    let (state, result) = activate(state, owner, &others, skill, 1);
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{:?}",
        result.result
    );
    for (player, start) in others.iter().zip(before) {
        assert_eq!(state.hand_count(*player), start + 3);
    }
    assert!(state.has_pending_choice());
    let (state, granted) = resolve(state, owner, vec![others[0].raw()], 2);
    assert!(
        matches!(granted.result, CommandResult::Accepted { .. }),
        "{:?}",
        granted.result
    );
    assert!(has_def(&state, others[0], WXDLFANGZHU));
    assert!(!has_def(&state, others[1], WXDLFANGZHU));
}

#[test]
fn npack_loads_wxdlyishi() {
    let mut engine = engine_core::Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.load_package(&pkg).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_815_92,
            ruleset: "identity-standard".into(),
        },
    });
    assert!(engine
        .state()
        .content
        .skills()
        .any(|def| def.skill_def.raw() == WXDLYISHI));
}
