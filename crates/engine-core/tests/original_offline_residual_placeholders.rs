//! Golden for OFFLINE residual placeholders upgraded to behaviorful DSL:
//! - `ylygxiangxing_yingzi` (inherit 界英姿: draw+1, hand base = max_hp)
//! - `psqingsuan` / `tanhu3` published with real modifiers in the shipped npack

use engine_api::{
    Command, CommandKind, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::query::{query, QueryContext};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const PSQINGSUAN: u32 = 4_016_041_748;
const TANHU3: u32 = 4_019_583_040;
const YLYGXIANGXING_YINGZI: u32 = 3_190_971_665;
const YLYG_PANGTONG: u32 = 0; // filled at runtime from package

const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");
const SOURCE_YINGZI: &str =
    include_str!("../../../content/original/ported/offline/ylygxiangxing_yingzi.skill");

fn loaded(seed: u64) -> (Transaction, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(seed),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&package).unwrap();
    let owner = engine.state().turn.current;
    (Transaction::begin(engine.state().clone()), owner)
}

fn strip_skills(tx: &mut Transaction, owner: PlayerId) {
    let skills = tx.state().players.get(owner).unwrap().skills.clone();
    for skill in skills {
        run_effects(tx, vec![EffectRequest::RemoveSkill { skill }]).unwrap();
    }
}

fn add_skill(tx: &mut Transaction, owner: PlayerId, skill: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill),
        }],
    )
    .unwrap();
}

fn hand_limit(tx: &Transaction, owner: PlayerId) -> i32 {
    match query(
        tx.state(),
        &QueryRequest::HandLimit { player: owner },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("unexpected query response: {other:?}"),
    }
}

#[test]
fn package_publishes_former_placeholder_skills_with_modifiers() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();

    for (id, name) in [
        (PSQINGSUAN, "psqingsuan"),
        (TANHU3, "tanhu3"),
        (YLYGXIANGXING_YINGZI, "ylygxiangxing_yingzi"),
    ] {
        assert!(
            package
                .skill_index
                .iter()
                .any(|s| s.skill_def_id == id && s.name == name),
            "missing skill {name}"
        );
        assert!(
            package
                .modifier_index
                .iter()
                .any(|m| m.skill_def_id == id),
            "skill {name} must publish at least one modifier"
        );
    }

    // lord skill flag
    let qs = package
        .skill_index
        .iter()
        .find(|s| s.skill_def_id == PSQINGSUAN)
        .unwrap();
    assert!(qs.locked);
    assert!(qs.lord_only);

    // character closures still reference derivation skills
    let zhonghui = package
        .character_index
        .iter()
        .find(|c| c.name_key == "yj_zhonghui")
        .expect("yj_zhonghui");
    assert!(zhonghui.skills.contains(&PSQINGSUAN));

    let pangtong = package
        .character_index
        .iter()
        .find(|c| c.name_key == "ylyg_pangtong")
        .expect("ylyg_pangtong");
    assert!(
        package
            .skill_index
            .iter()
            .any(|s| s.name == "ylygxiangxing" && pangtong.skills.contains(&s.skill_def_id))
    );
    let _ = YLYG_PANGTONG;
}

#[test]
fn ylygxiangxing_yingzi_matches_reyingzi_draw_and_hand_base() {
    // compile_source path — exercises the refined .skill, not a reimplementation
    let compiled =
        content_compiler::compile_source_named("ylygxiangxing_yingzi-test", SOURCE_YINGZI)
            .unwrap();
    assert!(compiled.skill_index.iter().any(|s| {
        s.skill_def_id == YLYGXIANGXING_YINGZI && s.name == "ylygxiangxing_yingzi" && s.locked
    }));
    assert_eq!(
        compiled
            .modifier_index
            .iter()
            .filter(|m| m.skill_def_id == YLYGXIANGXING_YINGZI)
            .count(),
        2
    );

    let (mut tx, owner) = loaded(20_260_808_31);
    strip_skills(&mut tx, owner);

    // standardize hp so hand base = max_hp is observable
    {
        let p = tx.state_mut().players.get_mut(owner).unwrap();
        p.hp = 3;
        p.max_hp = 4;
    }

    let base_draw = engine_core::query::draw_count_for(tx.state(), owner);
    let base_hand = hand_limit(&tx, owner);

    add_skill(&mut tx, owner, YLYGXIANGXING_YINGZI);

    assert_eq!(
        engine_core::query::draw_count_for(tx.state(), owner),
        base_draw + 1,
        "draw_count +1 like reyingzi"
    );
    // hand_limit_base_max_hp override → limit uses max_hp as base (4) vs previous base
    let after = hand_limit(&tx, owner);
    assert_eq!(
        after, 4,
        "hand limit base becomes max_hp (4); was {base_hand} before skill"
    );
}

#[test]
fn psqingsuan_and_tanhu3_load_from_shipped_npack() {
    let (mut tx, owner) = loaded(20_260_808_32);
    strip_skills(&mut tx, owner);
    add_skill(&mut tx, owner, PSQINGSUAN);
    add_skill(&mut tx, owner, TANHU3);
    let live: Vec<_> = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .collect();
    assert_eq!(live.len(), 2);
    // Skills are live on the player after AddSkill — package path is the unit under test.
    let defs: Vec<u32> = live
        .into_iter()
        .filter_map(|sid| {
            tx.state()
                .skills
                .get(sid)
                .map(|s| s.def.raw())
        })
        .collect();
    assert!(defs.contains(&PSQINGSUAN));
    assert!(defs.contains(&TANHU3));
}
