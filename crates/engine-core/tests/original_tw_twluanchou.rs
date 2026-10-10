//! Golden coverage for TW 桥公「乱筹/共患」.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWLUANCHOU: u32 = 2_306_392_424;
const TWGONGHUAN: u32 = 3_503_235_693;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twluanchou.skill");

#[test]
fn resets_two_holders_and_redirects_only_once_with_original_damage_fields() {
    let package = content_compiler::compile_source_named("twluanchou-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 374);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 2_026_080_115, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, victim, source) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TWLUANCHOU),
    });
    run_effects(&mut tx, effects).unwrap();
    let main = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == TWLUANCHOU)
        .unwrap()
        .id;
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveTwluanchou {
            player: owner,
            helper_skill_def: TWGONGHUAN,
            action: 0,
            targets: vec![owner, victim],
            source: PlayerId::NONE,
            target: PlayerId::NONE,
            amount: 0,
            nature: 0,
            card: engine_api::CardId::NONE,
            card_kind: 0,
            card_flags: 0,
            skill: main,
        }],
    )
    .unwrap();
    let holders: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .filter(|skill| skill.def.raw() == TWGONGHUAN)
        .map(|skill| skill.owner)
        .collect();
    assert_eq!(holders, [owner, victim]);
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    let victim_hp = tx.state().players.get(victim).unwrap().hp;
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source,
            target: victim,
            amount: 2,
            is_lose_hp: false,
            nature: rule_api::damage_nature::FIRE,
            card: engine_api::CardId::NONE,
            card_kind: 0,
            card_flags: 0,
        }],
    );
    // 共患当前为 filter 门控的自动重定向（无确认 choice）。
    assert!(result.is_ok());
    let state = tx.commit().state;
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp - 2);
    assert_eq!(state.players.get(victim).unwrap().hp, victim_hp);
    assert!(!state.has_pending_choice());

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source,
            target: victim,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::THUNDER,
            card: engine_api::CardId::NONE,
            card_kind: 0,
            card_flags: 0,
        }],
    )
    .unwrap();
    let state = tx.commit().state;
    assert_eq!(state.players.get(victim).unwrap().hp, victim_hp - 1);
    assert!(!state.has_pending_choice());
}
