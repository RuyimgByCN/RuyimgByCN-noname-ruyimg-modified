//! Golden for 神话再临「血裔」：仅主公生效，其他群势力角色每名令手牌上限 +2。

use content_schema::PackageCharacterEntry;
use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::query::{query, QueryContext};
use engine_core::state::Identity;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const XUEYI: u32 = 3_178_128_297;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/xueyi.skill");

fn character(id: u32, group: &str) -> PackageCharacterEntry {
    PackageCharacterEntry {
        id,
        name_key: format!("test_character_{id}"),
        max_hp: 4,
        initial_hp: 0,
        skills: vec![],
        sex: "male".into(),
        group: group.into(),
        is_lord: false,
        clans: vec![],
        tags: vec![],
        sort_key: String::new(),
        portrait_asset: None,
        skin_assets: vec![],
    }
}

fn hand_limit(tx: &Transaction, player: PlayerId) -> i32 {
    match query(
        tx.state(),
        &QueryRequest::HandLimit { player },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("expected hand-limit integer, got {other:?}"),
    }
}

#[test]
fn xueyi_is_lord_only_and_counts_other_alive_qun_players_twice() {
    let mut package = content_compiler::compile_source_named("shenhua-xueyi", SOURCE).unwrap();
    package.character_index = vec![
        character(9_207_001, "wei"),
        character(9_207_002, "qun"),
        character(9_207_003, "qun"),
        character(9_207_004, "shu"),
    ];
    package.manifest.logic_hash = package.compute_logic_hash();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == XUEYI && skill.locked && skill.lord_only));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_714),
            kind: CommandKind::StartGame {
                player_count: 4,
                rng_seed: 20_260_714,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    let mut tx = Transaction::begin(engine.state().clone());
    let seats = tx.state().players.alive_by_seat();
    let lord = seats
        .iter()
        .copied()
        .find(|player| tx.state().players.get(*player).unwrap().identity == Identity::Lord)
        .unwrap();
    let mut others = seats.iter().copied().filter(|player| *player != lord);
    let qun_one = others.next().unwrap();
    let qun_two = others.next().unwrap();
    let nonlord = others.next().unwrap();

    tx.state_mut().players.get_mut(lord).unwrap().character_def = 9_207_001;
    tx.state_mut()
        .players
        .get_mut(qun_one)
        .unwrap()
        .character_def = 9_207_002;
    tx.state_mut()
        .players
        .get_mut(qun_two)
        .unwrap()
        .character_def = 9_207_003;
    tx.state_mut()
        .players
        .get_mut(nonlord)
        .unwrap()
        .character_def = 9_207_004;

    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: lord,
                skill_def: SkillDefId::new(XUEYI),
            },
            EffectRequest::AddSkill {
                player: nonlord,
                skill_def: SkillDefId::new(XUEYI),
            },
        ],
    )
    .unwrap();

    let lord_hp = tx.state().players.get(lord).unwrap().hp;
    let nonlord_hp = tx.state().players.get(nonlord).unwrap().hp;
    assert_eq!(hand_limit(&tx, lord), lord_hp + 4);
    assert_eq!(hand_limit(&tx, nonlord), nonlord_hp);

    tx.state_mut().players.get_mut(qun_two).unwrap().alive = false;
    assert_eq!(hand_limit(&tx, lord), lord_hp + 2);
}
