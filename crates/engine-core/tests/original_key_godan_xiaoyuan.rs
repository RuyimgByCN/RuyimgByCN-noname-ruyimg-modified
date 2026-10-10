//! Golden tests for KEY「消元」rewritten from the legacy JavaScript semantics.

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const GODAN_XIAOYUAN: u32 = 2_543_675_799;
const GODAN_FEIQU: u32 = 3_490_863_084;
const SOURCE: &str = include_str!("../../../content/original/ported/key/godan_xiaoyuan.skill");
const FEIQU_SOURCE: &str = include_str!("../../../content/original/ported/key/godan_feiqu.skill");
const KEY: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId) {
    let source = format!("{FEIQU_SOURCE}\n{SOURCE}");
    let package = content_compiler::compile_source_named("godan_xiaoyuan_test", &source).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let source = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(
        [GODAN_FEIQU, GODAN_XIAOYUAN].map(|skill_def| EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill_def),
        }),
    );
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, source)
}

fn has_skill(state: &GameState, owner: PlayerId, skill_def: u32) -> bool {
    state
        .skills
        .sorted_live()
        .into_iter()
        .any(|skill| skill.owner == owner && skill.def.raw() == skill_def)
}

fn hp_decrease(is_lose_hp: bool, seed: u64) -> GameState {
    let (mut tx, owner, source) = setup(seed);
    {
        let player = tx.state_mut().players.get_mut(owner).unwrap();
        player.max_hp = 4;
        player.hp = 4;
    }
    let hand = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, 1);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 1);
    assert_eq!(tx.state().hand_count(owner), hand + 3);
    assert!(!has_skill(tx.state(), owner, GODAN_FEIQU));
    assert!(!has_skill(tx.state(), owner, GODAN_XIAOYUAN));
    tx.commit().state
}

#[test]
fn committed_package_contains_both_hp_decrease_triggers() {
    let committed = content_schema::ContentPackage::from_json_bytes(KEY).unwrap();
    let compiled = content_compiler::compile_source_named("godan_xiaoyuan", SOURCE).unwrap();
    let triggers: Vec<_> = committed
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == GODAN_XIAOYUAN)
        .map(|trigger| {
            (
                trigger.event.as_str(),
                trigger.filter.as_str(),
                trigger.optional,
            )
        })
        .collect();
    assert_eq!(
        triggers,
        [
            ("damage", "owner_is_target_hp_below_4", false),
            ("hp_lost", "owner_is_target_hp_below_4", false),
        ]
    );
    for expected in &compiled.modules {
        let actual = committed
            .modules
            .iter()
            .find(|module| module.name == expected.name)
            .unwrap();
        assert_eq!(actual.logic_hash, expected.logic_hash);
    }
}

#[test]
fn damage_below_four_awakens_once() {
    let _ = hp_decrease(false, 20_260_718_1);
}

#[test]
fn direct_hp_loss_below_four_awakens_once() {
    let _ = hp_decrease(true, 20_260_718_2);
}

#[test]
fn hp_equal_to_four_does_not_awaken() {
    let (mut tx, owner, source) = setup(20_260_718_3);
    {
        let player = tx.state_mut().players.get_mut(owner).unwrap();
        player.max_hp = 6;
        player.hp = 5;
    }
    let hand = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, 6);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 4);
    assert_eq!(tx.state().hand_count(owner), hand);
    assert!(has_skill(tx.state(), owner, GODAN_FEIQU));
    assert!(has_skill(tx.state(), owner, GODAN_XIAOYUAN));
}
