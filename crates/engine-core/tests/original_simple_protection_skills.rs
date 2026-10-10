//! Golden coverage for simple temporary damage and phase protection skills.

use engine_api::{Command, CommandKind, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId};
use engine_core::effect::run_effects;
use engine_core::intent::{resolve_damage_intent, DamageIntent};
use engine_core::query::skip_judge_phase;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const XUANMU2: u32 = 2_561_421_451;
const XINTIANXIANG4: u32 = 3_553_506_611;
const XINFU_PDGYINGSHI2: u32 = 2_785_649_758;
const YINGBIAN: &[u8] =
    include_bytes!("../../../content/original/ported/yingbian/yingbian.npack.json");
const SHENHUA: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");
const MOBILE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

fn isolated(package: &[u8], skill: u32, seed: u64) -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(package).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
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
    let owner = engine.state().turn.current;
    let other = engine
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
        .map(|active| EffectRequest::RemoveSkill { skill: active.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, other)
}

fn damage(
    state: &engine_core::state::GameState,
    source: PlayerId,
    target: PlayerId,
) -> DamageIntent {
    resolve_damage_intent(
        state,
        DamageIntent {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
            cancelled: false,
        },
    )
}

#[test]
fn temporary_protection_skills_cover_received_dealt_and_judge_phase_rules() {
    let (tx, owner, other) = isolated(YINGBIAN, XUANMU2, 20_260_816);
    assert!(damage(tx.state(), other, owner).cancelled);
    assert!(!damage(tx.state(), owner, other).cancelled);

    let (tx, owner, other) = isolated(SHENHUA, XINTIANXIANG4, 20_260_817);
    assert!(damage(tx.state(), other, owner).cancelled);
    assert!(damage(tx.state(), owner, other).cancelled);

    let (tx, owner, _) = isolated(MOBILE, XINFU_PDGYINGSHI2, 20_260_818);
    assert!(skip_judge_phase(tx.state(), owner));
}
