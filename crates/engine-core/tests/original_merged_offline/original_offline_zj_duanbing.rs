//! Golden for OFFLINE 智将丁奉「短兵」rewritten from legacy JavaScript semantics.

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
    RULE_VM_ABI_VERSION,
};
use engine_core::effect::run_effects;
use engine_core::query::{query, sha_response_count_for, QueryContext};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const ZJ_DINGFENG: u32 = 3_741_059_626;
const ZJ_DUANBING: u32 = 2_843_523_405;
const ZJ_FENXUN: u32 = 3_362_692_932;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/offline/offline.npack.json");

fn setup() -> (Transaction, PlayerId, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(ZJ_DUANBING as u64),
            kind: CommandKind::StartGame {
                player_count: 4,
                rng_seed: ZJ_DUANBING as u64,
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
        skill_def: SkillDefId::new(ZJ_DUANBING),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in &seats {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 20;
        participant.max_hp = 20;
    }
    (tx, owner, seats)
}

fn distance(tx: &Transaction, from: PlayerId, to: PlayerId) -> i32 {
    match query(
        tx.state(),
        &QueryRequest::Distance { from, to },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("unexpected distance response: {other:?}"),
    }
}

fn damage(tx: &mut Transaction, source: PlayerId, target: PlayerId) {
    run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
}

#[test]
fn package_closes_zj_dingfeng_and_publishes_both_generic_rules() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    assert!(package.character_index.iter().any(|character| {
        character.id == ZJ_DINGFENG && character.skills == [ZJ_DUANBING, ZJ_FENXUN]
    }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == ZJ_DUANBING
            && trigger.filter
                == "owner_source_target_distance_le_one_first_damage_to_target_this_turn"
    }));
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == ZJ_DUANBING
            && modifier.target == "sha_response_count_if_source_owner_target_distance_le_one"
    }));
}

#[test]
fn nearby_sha_needs_two_shan_and_only_first_damage_per_target_draws() {
    let (mut tx, owner, seats) = setup();
    let near: Vec<_> = seats
        .iter()
        .copied()
        .filter(|target| *target != owner && distance(&tx, owner, *target) == 1)
        .collect();
    let far = seats
        .iter()
        .copied()
        .find(|target| distance(&tx, owner, *target) > 1)
        .unwrap();
    assert_eq!(sha_response_count_for(tx.state(), owner, near[0], 1), 2);
    assert_eq!(sha_response_count_for(tx.state(), owner, far, 1), 1);
    assert_eq!(sha_response_count_for(tx.state(), near[0], owner, 1), 1);

    let hand = tx.state().hand_count(owner);
    damage(&mut tx, owner, near[0]);
    assert_eq!(tx.state().hand_count(owner), hand + 1);
    damage(&mut tx, owner, near[0]);
    assert_eq!(tx.state().hand_count(owner), hand + 1);
    damage(&mut tx, owner, near[1]);
    assert_eq!(tx.state().hand_count(owner), hand + 2);
    damage(&mut tx, owner, far);
    assert_eq!(tx.state().hand_count(owner), hand + 2);
}
