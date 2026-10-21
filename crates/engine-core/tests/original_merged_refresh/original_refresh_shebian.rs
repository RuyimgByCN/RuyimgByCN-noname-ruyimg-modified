//! 第一百八十四批：refresh「设变」（OL夏侯渊 shebian）零新原语重写 + 新过滤器。
//! 上游：当你的武将牌翻面后（player turnOverEnd），可移动场上的一张装备牌。
//! 实现：character_state_changed + owner_turned_over（本批新注册，双向翻面，
//! 5 处注册零 ABI）→ optional 确认 → choose_move_field_card_to(self, self)。
//! 近似（登记债）：原语可选集含判定区、目标固定为自己。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const SHEBIAN: u32 = 3_980_781_998;
const OL_XIAHOUYUAN: u32 = 3_694_177_949;
const SOURCE: &str = include_str!("../../../../content/original/ported/refresh/shebian.skill");
const REFRESH: &[u8] =
    include_bytes!("../../../../content/original/ported/refresh/refresh.npack.json");

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("shebian-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&compiled)
        .unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let other = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(SHEBIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, other)
}

/// 给 `target` 装一匹防御马（可被移动的场牌——移动者自己没有防马时合法）。
fn equip_mount(state: &mut GameState, target: PlayerId, kind: CardKind) -> CardId {
    let mut tx = Transaction::begin(state.clone());
    let zone = tx.state().zones.find(ZoneKind::Equip, target).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_999),
        kind,
        suit: Suit::Spade,
        rank: 1,
        weapon_range: 0,
        zone,
        owner: target,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    *state = tx.commit().state;
    card
}

/// 派发翻面事件。optional 触发的 confirm 在 dispatch 期打开（Err(ChoiceRequested)
/// 由命令管线负责提交），此处手动提交并返回「confirm 是否打开」。
fn turn_over(state: GameState, who: PlayerId) -> (GameState, bool) {
    let mut tx = Transaction::begin(state);
    let confirmed = match dispatch_for_events(
        &mut tx,
        &[DomainEvent::PlayerTurnedOver {
            player: who,
            turned_over: true,
        }],
        &mut TriggerFireRecord::new(),
    ) {
        Err(EffectError::ChoiceRequested) => true,
        Ok(effects) => {
            run_effects(&mut tx, effects).unwrap();
            false
        }
        Err(other) => panic!("unexpected dispatch error: {other:?}"),
    };
    (tx.commit().state, confirmed)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let (next, result) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled,
            },
        },
    );
    (next, result)
}

#[test]
fn package_publishes_shebian_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(REFRESH).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 681);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == SHEBIAN)
        .unwrap();
    assert_eq!(trigger.event, "character_state_changed");
    assert_eq!(trigger.filter, "owner_turned_over");
    assert!(trigger.optional, "upstream shebian is a may-trigger");
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == OL_XIAHOUYUAN)
        .expect("ol_xiahouyuan in roster");
    assert!(character.skills.contains(&SHEBIAN));
}

#[test]
fn own_turn_over_opens_confirm_then_field_card_moves_to_owner() {
    let (mut state, owner, other) = setup(20_260_909_05);
    let horse = equip_mount(&mut state, other, CardKind::HorseDef);

    let (state, confirmed) = turn_over(state, owner);
    assert!(confirmed, "optional confirm must open");
    assert!(state.has_pending_choice());
    assert_eq!(
        state.resolution.continuation_kind,
        engine_core::state::ContinuationKind::OptionalTrigger,
    );

    // 段一：确认发动。
    let (state, result) = resolve(state, owner, vec![1], false, 20_260_909_51);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(
        state.has_pending_choice(),
        "SelectCards must follow confirm"
    );

    // 段二：选那张防马移到自己装备区。
    let (state, result) = resolve(state, owner, vec![horse.raw()], false, 20_260_909_52);
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "move rejected: {:?}",
        result.result
    );
    assert!(!state.has_pending_choice());
    let owner_equip = state.zones.find(ZoneKind::Equip, owner).unwrap();
    assert!(
        state.zones.get(owner_equip).unwrap().cards.contains(&horse),
        "horse must land in owner's equip zone"
    );
    let other_equip = state.zones.find(ZoneKind::Equip, other).unwrap();
    assert!(
        !state.zones.get(other_equip).unwrap().cards.contains(&horse),
        "horse must leave other's equip zone"
    );
}

#[test]
fn others_turn_over_does_not_trigger() {
    let (mut state, owner, other) = setup(20_260_909_06);
    let _ = equip_mount(&mut state, other, CardKind::HorseDef);

    let (state, confirmed) = turn_over(state, other);
    assert!(
        !confirmed && !state.has_pending_choice(),
        "owner filter must suppress other-player turn-overs"
    );
}
