//! Golden for YINGBIAN 贾充「凶竖」语义（ABI 436）。
//!
//! 凶竖：其他角色出牌阶段开始时弃置 X 张牌（X=凶竖标记数，每轮递增）并展示其
//! 一张手牌，预测其是否会使用该牌；该出牌阶段结束时预测对则其受 1 点伤害，
//! 错则获得该牌。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const XIONGSHU: u32 = 3_731_458_110;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yingbian/yingbian.npack.json");

fn setup() -> (Transaction, PlayerId, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(XIONGSHU as u64),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_816,
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
        skill_def: SkillDefId::new(XIONGSHU),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, seats)
}

fn skill_instance(tx: &Transaction, owner: PlayerId) -> engine_api::SkillId {
    tx.state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.def.raw() == XIONGSHU && s.owner == owner)
        .map(|s| s.id)
        .unwrap()
}

fn resolve_choice(
    state: GameState,
    actor: PlayerId,
    key: u64,
    selections: Vec<u32>,
    cancelled: bool,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "resolve failed: {:?}",
        result.result
    );
    next
}

fn xiongshu(
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> EffectRequest {
    EffectRequest::ResolveXiongshu {
        player,
        target,
        card: CardId::NONE,
        action: 0,
        stage,
        skill,
    }
}

/// 预测“会使用”且牌已离开目标区域 → 目标受 1 点伤害。
#[test]
fn xiongshu_correct_prediction_damages() {
    let (mut tx, owner, seats) = setup();
    let target = seats.into_iter().find(|p| *p != owner).unwrap();
    tx.state_mut().turn.current = target;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = skill_instance(&tx, owner);
    // 目标手牌第一张作为展示牌。
    let shown = tx.state().hand_cards(target)[0];
    // stage 0（标记 0 → 直接推进）→ stage 1 开选牌。
    run_effects(&mut tx, vec![xiongshu(owner, target, 0, skill)]).unwrap_err();
    let state = resolve_choice(tx.commit().state, owner, 1, vec![shown.raw()], false);
    // stage 2 预测：0 会使用。
    let state = resolve_choice(state, owner, 2, vec![0], false);
    assert_eq!(state.resolution.pending_choice_id, 0, "预测后无进一步选择");
    let mut tx = Transaction::begin(state.clone());
    // 模拟该牌被使用：移入弃牌堆（离开目标区域）。
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    {
        let c = tx.state_mut().cards.get_mut(shown).unwrap();
        c.zone = discard;
    }
    // stage 3 结算：目标受 1 点伤害。
    let hp_before = tx.state().players.get(target).unwrap().hp;
    run_effects(&mut tx, vec![xiongshu(owner, target, 3, skill)]).unwrap();
    assert_eq!(
        tx.state().players.get(target).unwrap().hp,
        hp_before - 1,
        "预测正确应造成 1 点伤害"
    );
    // 预测状态已清空。
    assert!(
        tx.state()
            .skills
            .get(skill)
            .is_some_and(|instance| instance.state_blob.is_empty()),
        "预测状态应清空"
    );
}

/// 预测“会使用”但牌仍在目标手牌 → 预测错 → 拥有者获得该牌。
#[test]
fn xiongshu_wrong_prediction_gains_card() {
    let (mut tx, owner, seats) = setup();
    let target = seats.into_iter().find(|p| *p != owner).unwrap();
    tx.state_mut().turn.current = target;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = skill_instance(&tx, owner);
    let shown = tx.state().hand_cards(target)[0];
    run_effects(&mut tx, vec![xiongshu(owner, target, 0, skill)]).unwrap_err();
    let state = resolve_choice(tx.commit().state, owner, 1, vec![shown.raw()], false);
    // 预测：0 会使用（但牌实际未用）。
    let state = resolve_choice(state, owner, 2, vec![0], false);
    let mut tx = Transaction::begin(state.clone());
    let hand_before = tx.state().hand_count(owner);
    // stage 3 结算：牌仍在目标手牌 → 预测错 → 获得该牌。
    run_effects(&mut tx, vec![xiongshu(owner, target, 3, skill)]).unwrap();
    assert_eq!(
        tx.state().hand_count(owner),
        hand_before + 1,
        "预测错应获得该牌"
    );
    assert!(
        tx.state().hand_cards(owner).contains(&shown),
        "应获得展示的那张牌"
    );
}

/// 奸回（ABI 437）：连续同一来源伤害 → 来源强制弃一张牌；连续同一受伤者 → 摸一张。
#[test]
fn jianhui_consecutive_source_discard_and_same_target_draw() {
    const JIANHUI: u32 = 4_269_285_584;
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(JIANHUI as u64),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: 20_260_817,
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
        skill_def: SkillDefId::new(JIANHUI),
    });
    run_effects(&mut tx, effects).unwrap();
    let attacker = seats.iter().copied().find(|p| *p != owner).unwrap();
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.def.raw() == JIANHUI && s.owner == owner)
        .map(|s| s.id)
        .unwrap();
    // 第一段：来源=attacker 对 owner 造成伤害 → 记录（无摸牌/弃牌）。
    let hand_before_first = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveJianhui {
            player: owner,
            source: attacker,
            target: owner,
            skill,
        }],
    )
    .unwrap();
    assert_eq!(
        tx.state().hand_count(owner),
        hand_before_first,
        "首次无奖励"
    );
    // 第二段：同一来源再次对 owner 造成伤害 → 来源强制弃一张（开 choice）。
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveJianhui {
            player: owner,
            source: attacker,
            target: owner,
            skill,
        }],
    )
    .err();
    assert_eq!(
        err,
        Some(EffectError::ChoiceRequested),
        "连续同来源应开弃牌 choice"
    );
    // 解决来源的强制弃牌（选其一张手牌）。
    let attacker_card = tx.state().hand_cards(attacker)[0];
    let state = resolve_choice(
        tx.commit().state,
        attacker,
        3,
        vec![attacker_card.raw()],
        false,
    );
    let mut tx = Transaction::begin(state.clone());
    // 同一来源对另一目标（第三次）→ 受伤者不同，来源视角：连续同受伤者才摸。
    let other = seats
        .iter()
        .copied()
        .find(|p| *p != owner && *p != attacker)
        .unwrap();
    let hand_before = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveJianhui {
            player: owner,
            source: attacker,
            target: other,
            skill,
        }],
    )
    .unwrap();
    assert_eq!(
        tx.state().hand_count(owner),
        hand_before,
        "来源视角受伤者不同不摸牌"
    );
}
