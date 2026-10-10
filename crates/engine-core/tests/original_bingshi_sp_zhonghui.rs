//! Golden coverage for BINGSHI 势钟会「俟资」「挟志」「狱难」「克昌」.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{run_effects, EffectError};
use engine_core::query::{query, sha_use_limit_for, QueryContext};
use engine_core::state::{turn_flag, GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const SIZI: u32 = 2_212_391_997;
const XIEZHI: u32 = 3_797_490_396;
const YUNAN: u32 = 3_604_534_562;
const KECHANG: u32 = 2_942_773_382;
const SOURCE: &str = include_str!("../../../content/original/ported/bingshi/sp_zhonghui.skill");
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
        [SIZI, XIEZHI, YUNAN, KECHANG].map(|skill_def| EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill_def),
        }),
    );
    run_effects(&mut tx, effects).unwrap();
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

fn resolve(state: GameState, selection: u32, key: u64) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor: PlayerId::new(state.resolution.pending_observer),
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections: vec![selection],
                cancelled: false,
            },
        },
    )
}

fn effect(
    player: PlayerId,
    target: PlayerId,
    action: u8,
    amount: u32,
    skill: engine_api::SkillId,
) -> EffectRequest {
    EffectRequest::ResolveMbzhonghui {
        player,
        target,
        action,
        amount,
        mbsizi_def: SkillDefId::new(SIZI),
        mbkechang_def: SkillDefId::new(KECHANG),
        skill,
    }
}

#[test]
fn package_publishes_complete_sp_zhonghui_at_abi553() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    assert_eq!(package.character_index.len(), 36);
    assert_eq!(package.skill_index.len(), 103);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.name_key == "sp_zhonghui")
            .unwrap()
            .skills,
        [SIZI, XIEZHI, YUNAN, KECHANG]
    );
}

#[test]
fn sizi_spends_charge_and_settles_sha_users() {
    let (state, owner, order) = setup(5531);
    let user = order[1];
    let sizi = skill(&state, owner, SIZI);
    let mut tx = Transaction::begin(state);
    run_effects(&mut tx, vec![effect(owner, owner, 1, 0, sizi)]).unwrap();
    assert!(matches!(
        run_effects(&mut tx, vec![effect(owner, owner, 0, 0, sizi)]),
        Err(EffectError::ChoiceRequested)
    ));
    let (state, output) = resolve(tx.commit().state, 2, 5532);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    let mut tx = Transaction::begin(state);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::sha_used_by_player(user), 1);
    let hp = tx.state().players.get(user).unwrap().hp;
    let hand = tx.state().hand_count(owner);
    run_effects(&mut tx, vec![effect(owner, owner, 2, 0, sizi)]).unwrap();
    assert_eq!(tx.state().hand_count(owner), hand + 2);
    assert_eq!(tx.state().players.get(user).unwrap().hp, hp - 1);
    assert_eq!(tx.state().skills.get(sizi).unwrap().state_blob[0], 1);
}

#[test]
fn xiezhi_overflow_grows_both_limits_and_yunan_upgrades_kechang() {
    let (state, owner, order) = setup(5533);
    let mut tx = Transaction::begin(state);
    let sizi = skill(tx.state(), owner, SIZI);
    let xiezhi = skill(tx.state(), owner, XIEZHI);
    let yunan = skill(tx.state(), owner, YUNAN);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddMark {
            player: owner,
            mark: engine_api::MarkDefId::new(SIZI),
            delta: 3,
        }],
    )
    .unwrap();
    let base_sha = sha_use_limit_for(tx.state(), owner);
    let base_hand = match query(
        tx.state(),
        &QueryRequest::HandLimit { player: owner },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("unexpected {other:?}"),
    };
    run_effects(&mut tx, vec![effect(owner, owner, 4, 3, xiezhi)]).unwrap();
    assert_eq!(sha_use_limit_for(tx.state(), owner), base_sha + 1);
    assert_eq!(
        query(
            tx.state(),
            &QueryRequest::HandLimit { player: owner },
            &mut QueryContext::new(),
        ),
        QueryResponse::Int(base_hand + 1)
    );
    assert_eq!(
        tx.state()
            .players
            .get(owner)
            .unwrap()
            .marks
            .iter()
            .find(|(mark, _)| *mark == SIZI)
            .unwrap()
            .1,
        4
    );

    run_effects(
        &mut tx,
        vec![
            effect(owner, order[1], 5, 0, yunan),
            effect(owner, order[2], 6, 0, yunan),
        ],
    )
    .unwrap();
    assert_eq!(tx.state().player_group(owner), Some("qun"));
    assert_eq!(
        tx.state()
            .skills
            .get(skill(tx.state(), owner, KECHANG))
            .unwrap()
            .state_blob,
        [1]
    );
    assert_eq!(tx.state().skills.get(sizi).unwrap().def.raw(), SIZI);
}
