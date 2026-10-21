//! Golden coverage for KEY 棒球 / 妙变 / 听弦 / 念力 rewritten from honest noops
//! (听弦/念力 noop 注记系误标，本批一并修正)。零新原语批次（ABI 651 不变）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const SASAMI_BAOQIU: u32 = 3_822_498_729;
const SASAMI_MIAOBIAN: u32 = 3_097_071_788;
const SASAMI_GONGQING: u32 = 4_263_354_141;
const SASAMI_FUNAN: u32 = 2_807_672_918;
const SHIORIMIYUKI_TINGXIAN: u32 = 2_995_156_497;
const SHIZURU_NIANLI: u32 = 2_833_421_901;

const BAOQIU_SRC: &str =
    include_str!("../../../../content/original/ported/key/sasami_baoqiu.skill");
const MIAOBIAN_SRC: &str =
    include_str!("../../../../content/original/ported/key/sasami_miaobian.skill");
const TINGXIAN_SRC: &str =
    include_str!("../../../../content/original/ported/key/shiorimiyuki_tingxian.skill");
const NIANLI_SRC: &str =
    include_str!("../../../../content/original/ported/key/shizuru_nianli.skill");
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/key/key.npack.json");

fn bootstrap(source: &str, name: &str, skill_def: u32) -> (Transaction, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named(name, source).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(skill_def), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&compiled)
        .unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
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
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, other)
}

fn has_skill(state: &GameState, owner: PlayerId, def: u32) -> bool {
    state
        .players
        .get(owner)
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

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let (state, pipe) = apply_command(
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
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    state
}

#[test]
fn baoqiu_source_and_npack_publish_attack_range() {
    let compiled = content_compiler::compile_source_named("sasami_baoqiu", BAOQIU_SRC).unwrap();
    assert!(compiled.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == SASAMI_BAOQIU
            && modifier.target == "attack_range"
            && modifier.mode == "add"
            && modifier.value == 2
    }));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == SASAMI_BAOQIU
            && modifier.target == "attack_range"
            && modifier.value == 2
    }));
}

#[test]
fn miaobian_source_and_npack_publish_threshold_grants() {
    let compiled = content_compiler::compile_source_named("sasami_miaobian", MIAOBIAN_SRC).unwrap();
    let events: Vec<String> = compiled
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == SASAMI_MIAOBIAN)
        .map(|trigger| trigger.event.clone())
        .collect();
    assert_eq!(events, vec!["hp_lost", "hp_recovered", "game_started"]);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(
        pack.trigger_index
            .iter()
            .filter(|trigger| trigger.skill_def_id == SASAMI_MIAOBIAN)
            .count(),
        3
    );
}

#[test]
fn miaobian_grants_skills_by_hp_thresholds() {
    let (mut tx, owner, _) = bootstrap(MIAOBIAN_SRC, "miaobian-test", SASAMI_MIAOBIAN);
    // hp = 2：应获公清（≤3）与赴难（≤2），未到棒球（≤1）。
    tx.state_mut().players.get_mut(owner).unwrap().hp = 2;
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source: owner,
            target: owner,
            amount: 1,
            is_lose_hp: true,
            nature: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;
    assert!(has_skill(&state, owner, SASAMI_GONGQING));
    assert!(has_skill(&state, owner, SASAMI_FUNAN));
    assert!(!has_skill(&state, owner, SASAMI_BAOQIU));

    // hp = 1 再失体力：三技齐授。
    let mut tx = Transaction::begin(state);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 1;
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source: owner,
            target: owner,
            amount: 1,
            is_lose_hp: true,
            nature: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;
    assert!(has_skill(&state, owner, SASAMI_GONGQING));
    assert!(has_skill(&state, owner, SASAMI_FUNAN));
    assert!(has_skill(&state, owner, SASAMI_BAOQIU));
}

#[test]
fn tingxian_source_and_npack_publish_play_begin_choice() {
    let compiled =
        content_compiler::compile_source_named("shiorimiyuki_tingxian", TINGXIAN_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == SHIORIMIYUKI_TINGXIAN)
        .expect("tingxian trigger");
    assert_eq!(trigger.event, "phase_play");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == SHIORIMIYUKI_TINGXIAN && trigger.event == "phase_play"
    }));
}

#[test]
fn tingxian_draws_chosen_count_and_recovers() {
    let (mut tx, owner, _) = bootstrap(TINGXIAN_SRC, "tingxian-test", SHIORIMIYUKI_TINGXIAN);
    let player = tx.state().players.get(owner).unwrap();
    let max_hp = player.max_hp as i32;
    let hand_before = tx.state().hand_count(owner);
    assert!(max_hp >= 2);
    tx.state_mut().players.get_mut(owner).unwrap().hp = max_hp - 2;
    let mut tx = tx;
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::TurnStateChanged {
                current: owner,
                phase: TurnPhaseTag::Play,
            }],
            &mut TriggerFireRecord::new(),
        ),
        Err(engine_core::effect::EffectError::ChoiceRequested)
    );
    // optional confirm → choose(1,2,3) 选 2：摸 2 张并回复 1。
    let state = resolve(tx.commit().state, owner, vec![1], 2);
    let state = resolve(state, owner, vec![2], 3);
    assert_eq!(state.hand_count(owner), hand_before + 2);
    assert_eq!(state.players.get(owner).unwrap().hp, max_hp - 1);
}

#[test]
fn nianli_source_and_npack_publish_suit_view_as() {
    let compiled = content_compiler::compile_source_named("shizuru_nianli", NIANLI_SRC).unwrap();
    let mut pairs: Vec<(String, String, String)> = compiled
        .view_as_index
        .iter()
        .filter(|entry| entry.skill_def_id == SHIZURU_NIANLI)
        .map(|entry| {
            (
                entry.as_kind.clone(),
                entry.window.clone(),
                entry.filter.clone(),
            )
        })
        .collect();
    pairs.sort();
    assert_eq!(
        pairs,
        vec![
            ("sha".into(), "play_phase".into(), "diamond".into()),
            ("sha".into(), "response".into(), "diamond".into()),
            ("shan".into(), "response".into(), "club".into()),
            ("tao".into(), "play_phase".into(), "heart".into()),
            ("tao".into(), "response".into(), "heart".into()),
            ("wuxie".into(), "response".into(), "spade".into()),
        ]
    );
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(
        pack.view_as_index
            .iter()
            .filter(|entry| entry.skill_def_id == SHIZURU_NIANLI)
            .count(),
        6
    );
    assert!(pack.view_as_index.iter().any(|entry| {
        entry.skill_def_id == SHIZURU_NIANLI
            && entry.as_kind == "sha"
            && entry.filter == "diamond"
            && entry.nature.as_deref() == Some("thunder")
    }));
}
