//! Golden for sp2 batch-AG DSL rewrite:
//! 2 个技能——批量脚本误标为「audio-only / 待迁移」，上游 (sp2/skill.js) 在引用的
//! 行号处均有真实规则体：
//!   - lslixun（李肃）：受到伤害前防止之，改为获得等量「珠」标记
//!   - xingshen（邢氏）：受到伤害后摸 1（上游手牌最少时摸 2 分支未建模）

use engine_api::{
    Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const LSLIXUN: u32 = 4_094_034_720;
const XINGSHEN: u32 = 3_761_126_799;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp2/sp2.npack.json");

fn started(seed: u64) -> (GameState, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
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
    let state = engine.state().clone();
    let owner = state.turn.current;
    (state, owner)
}

fn with_skill(state: GameState, owner: PlayerId, def: u32) -> GameState {
    let mut tx = Transaction::begin(state);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

#[test]
fn package_publishes_sp2_batch_ag_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 562);
    let triggers: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| matches!(trigger.skill_def_id, LSLIXUN | XINGSHEN))
        .map(|trigger| {
            (
                trigger.skill_def_id,
                trigger.event.as_str(),
                trigger.filter.as_str(),
                trigger.optional,
            )
        })
        .collect();
    assert!(triggers.contains(&(LSLIXUN, "damage_before", "owner_is_target", false)));
    assert!(triggers.contains(&(XINGSHEN, "damage", "owner_is_target", false)));
    assert!(!package.modifier_index.iter().any(|modifier| {
        matches!(modifier.skill_def_id, LSLIXUN | XINGSHEN)
            && modifier.target == "hand_limit"
    }));
}

#[test]
fn lslixun_turns_damage_into_mark() {
    let (state, owner) = started(2_026_081_101);
    let state = with_skill(state, owner, LSLIXUN);
    let mut tx = Transaction::begin(state);
    let other = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let hp_before = tx.state().players.get(owner).unwrap().hp;
    let effects = run_effects(
        &mut tx,
        vec![EffectRequest::CommitDamage {
            damage_id: 0,
            source: other,
            target: owner,
            amount: 2,
            base_amount: 2,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
            card: engine_api::CardId::NONE,
            card_kind: 0,
            card_flags: 0,
        }],
    )
    .unwrap();
    assert!(
        effects
            .iter()
            .any(|event| matches!(event, DomainEvent::DamagePrevented { .. })),
        "lslixun must prevent the damage"
    );
    // hp 不变，标记 +2
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp_before);
    let marks = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .find(|(id, _)| *id == LSLIXUN)
        .map(|(_, count)| *count)
        .unwrap_or(0);
    assert_eq!(marks, 2, "lslixun must add 珠 marks equal to damage amount");
}

#[test]
fn xingshen_draws_one_after_damage() {
    let (state, owner) = started(2_026_081_102);
    let state = with_skill(state, owner, XINGSHEN);
    let before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    let other = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source: other,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().hand_count(owner), before + 1);
}
