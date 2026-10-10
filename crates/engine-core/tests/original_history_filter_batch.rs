//! 第二百二十批：历史查询过滤器两件 + twyouye/mbyouye/mbsuwang 晋升行为测试。
//! `owner_alive_other_phase_end_not_damaged_owner_this_turn`：伤害对计数
//! （damage_pair_count(事件角色, owner)==0，跟踪面 ABI≥353 全开）。
//! `owner_alive_other_phase_end_used_card_on_owner_undamaged_this_turn`：
//! card_target_count（需过滤器 opt-in）+ owner 本回合未受伤。
//! 消费主面：place_top_deck_to_expansion(self, self_skill, def, 1)（ABI 685）。

use engine_api::{
    CardDefId, CardId, ChoiceId, DomainEvent, PlayerId, SkillDefId, Suit,
};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

fn inject_hand_card(state: GameState, owner: PlayerId, def: u32) -> (GameState, CardId) {
    let mut tx = Transaction::begin(state);
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Other,
        suit: Suit::Spade,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    (tx.commit().state, card)
}

/// 双过滤器直驱测试：合成 probe 技能挂双触发臂，验证伤害对计数与用牌计数门控。
#[test]
fn history_filters_gate_on_pair_counts() {
    let source = "skill probe_hist(9006001) {\n  trigger phase_end owner_alive_other_phase_end_not_damaged_owner_this_turn {\n    effect draw_cards(self, 1);\n  }\n  trigger phase_end owner_alive_other_phase_end_used_card_on_owner_undamaged_this_turn {\n    effect draw_cards(self, 2);\n  }\n}\n";
    let compiled = content_compiler::compile_source_named("hist-b220", source).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_926_01, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
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
        skill_def: SkillDefId::new(900_6001),
    });
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;
    let hand_before = state.hand_count(owner);

    // other 结束阶段：未伤害过 owner → 臂一开（摸 1）；未用牌指 owner → 臂二不开。
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: other,
            phase: engine_api::TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    )
    .expect("dispatch");
    let outcome = engine_core::effect::run_effects(&mut tx, effects);
    assert!(outcome.is_ok(), "both arms forced, no choice: {outcome:?}");
    let state = tx.commit().state;
    eprintln!("DBG after phase1 hand={}", state.hand_count(owner));
    assert_eq!(
        state.hand_count(owner),
        hand_before + 1,
        "arm 1 (not damaged) fired; arm 2 (no card use) silent"
    );

    // 模拟 other 对 owner 用牌：直录 card_target 计数后，臂二在下一结束阶段开。
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.record_card_target(other, owner);
    let state = tx.commit().state;
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: other,
            phase: engine_api::TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    )
    .expect("dispatch");
    let outcome = engine_core::effect::run_effects(&mut tx, effects);
    assert!(outcome.is_ok(), "{outcome:?}");
    let state = tx.commit().state;
    assert_eq!(
        state.hand_count(owner),
        hand_before + 1 + 1 + 2,
        "arm 2 fired after recorded card use (owner undamaged)"
    );

    // owner 受伤后：臂二未受伤门控关闭（臂一不受 owner 受伤影响，仍开）。
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.record_damage_taken(owner);
    let state = tx.commit().state;
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: other,
            phase: engine_api::TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    )
    .expect("dispatch");
    let outcome = engine_core::effect::run_effects(&mut tx, effects);
    assert!(outcome.is_ok(), "{outcome:?}");
    let state = tx.commit().state;
    assert_eq!(
        state.hand_count(owner),
        hand_before + 1 + 1 + 2 + 1,
        "arm 1 only (+1); arm 2 blocked by damage taken"
    );
}
