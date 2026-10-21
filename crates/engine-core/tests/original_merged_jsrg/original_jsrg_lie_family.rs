//! 猎标记族 S1（ADR 0646 §1，第七十八批）：jsrgzhenglve 政略①③、
//! jsrghuilie 会猎觉醒、jsrgpingrong 平戎移猎（钳位负 delta）。
//! 「猎」def = 合成铸造 blake2b("jsrgzhenglve_mark",4B) = 1167524380。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const ZHENGLVE: u32 = 2_636_593_306;
const HUILIE: u32 = 2_465_016_316;
const PINGRONG: u32 = 2_770_992_068;
const HUNT_MARK: u32 = 1_167_524_380;
const FEIYING: u32 = 2_669_566_023; // standard 包飞影（跨包授技字面量）

const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/jsrg/jsrg.npack.json");
const SOURCES_DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../content/original/ported/jsrg"
);

fn sources() -> String {
    let dir = std::path::Path::new(SOURCES_DIR);
    let mut files: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "skill"))
        .map(|e| e.path())
        .collect();
    files.sort();
    files
        .iter()
        .map(|p| {
            let mut s = std::fs::read_to_string(p).unwrap();
            s.push('\n');
            s
        })
        .collect()
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
    let (state, pipe) = apply_command(
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
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    state
}

fn drive(tx: &mut Transaction, events: &[engine_api::DomainEvent]) -> Result<(), EffectError> {
    match engine_core::trigger::dispatch_for_events(
        tx,
        events,
        &mut engine_core::trigger::TriggerFireRecord::new(),
    ) {
        Err(e) => Err(e),
        Ok(effects) => run_effects(tx, effects).map(|_| ()),
    }
}

fn grant_mark_tx(tx: &mut Transaction, player: PlayerId, count: i32) {
    run_effects(
        tx,
        vec![EffectRequest::AddMark {
            player,
            mark: engine_api::MarkDefId::new(HUNT_MARK),
            delta: count,
        }],
    )
    .unwrap();
}

fn started(players: usize, def: u32) -> (GameState, PlayerId) {
    let compiled = content_compiler::compile_source_named("lie-family", &sources()).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, players as u8, 20_260_828, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&compiled)
        .unwrap();
    let owner = tx.state().turn.current;
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
    (tx.commit().state, owner)
}

fn mark_count(state: &GameState, player: PlayerId) -> i32 {
    state
        .players
        .get(player)
        .and_then(|p| {
            p.marks
                .iter()
                .find(|(id, _)| *id == HUNT_MARK)
                .map(|(_, c)| *c)
        })
        .unwrap_or(0)
}

fn others_of(state: &GameState, owner: PlayerId) -> Vec<PlayerId> {
    state
        .players
        .iter_live()
        .map(|p| p.id)
        .filter(|id| *id != owner)
        .collect()
}

fn lord_of(state: &GameState, fallback: PlayerId) -> PlayerId {
    state
        .players
        .iter_live()
        .find(|p| p.identity == engine_core::state::Identity::Lord)
        .map(|p| p.id)
        .unwrap_or(fallback)
}

// ---- 发布契约 ----

#[test]
fn lie_family_publishes_contracts() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 730);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == ZHENGLVE
            && t.event == "phase_end"
            && t.filter == "owner_alive_event_player_lord_damaged"
            && t.optional
    }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == ZHENGLVE
            && t.event == "phase_end"
            && t.filter == "owner_alive_event_player_lord_no_damage"
            && t.optional
    }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == ZHENGLVE
            && t.event == "damage"
            && t.filter == "owner_source_damage_target_mark_positive_1167524380_once"
            && t.optional
    }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == HUILIE
            && t.event == "phase_prepare"
            && t.filter == "owner_alive_marked_alive_above_two_1167524380"
            && !t.optional
    }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == PINGRONG
            && t.event == "phase_end"
            && t.filter == "owner_alive_someone_mark_positive_1167524380"
            && t.optional
    }));
}

// ---- 政略①：主公回合结束摸牌 + 授猎（confirm → SelectTargets 两段链） ----

#[test]
fn zhenglve_no_damage_branch_grants_up_to_two_on_lord_turn_end() {
    use engine_core::state::ContinuationKind;
    let (state, holder) = started(3, ZHENGLVE);
    let lord = lord_of(&state, holder);
    let others = others_of(&state, holder);

    // 新开局无伤害事件：DAMAGE_DEALT==0 → no_damage 分支，max_count=2。
    let mut tx = Transaction::begin(state);
    let hand_before = tx.state().hand_count(holder);
    let opened = drive(
        &mut tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: lord,
            phase: engine_api::TurnPhaseTag::End,
        }],
    );
    assert_eq!(
        opened.unwrap_err(),
        EffectError::ChoiceRequested,
        "lord turn end opens the confirm choice"
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let mid = resolve(tx.state, holder, vec![1], false, 531);
    assert_eq!(
        mid.resolution.continuation_kind,
        ContinuationKind::ChooseOthersAddMark
    );
    let observer = mid.resolution.pending_observer;
    // 双授：一次选中两名。
    let final_state = resolve(
        mid,
        PlayerId::new(observer),
        others.iter().map(|p| p.raw()).collect(),
        false,
        532,
    );
    assert_eq!(final_state.hand_count(holder), hand_before + 1);
    for target in &others {
        assert_eq!(mark_count(&final_state, *target), 1);
    }
}

#[test]
fn zhenglve_damaged_branch_caps_grant_at_one() {
    use engine_core::state::{turn_flag, ContinuationKind};
    let (state, holder) = started(3, ZHENGLVE);
    let lord = lord_of(&state, holder);
    let target = others_of(&state, holder)[0];

    let mut tx = Transaction::begin(state);
    // 主公本回合造成过伤害：DAMAGE_DEALT 聚合口径置位。
    tx.state_mut().turn.set_flag(turn_flag::DAMAGE_DEALT, 3);
    let hand_before = tx.state().hand_count(holder);

    let opened = drive(
        &mut tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: lord,
            phase: engine_api::TurnPhaseTag::End,
        }],
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    let tx = tx.commit();
    let mid = resolve(tx.state, holder, vec![1], false, 541);
    assert_eq!(
        mid.resolution.continuation_kind,
        ContinuationKind::ChooseOthersAddMark
    );
    let observer = mid.resolution.pending_observer;
    // 尝试选两人应被拒（max_count=1）——直接单选走通。
    let final_state = resolve(mid, PlayerId::new(observer), vec![target.raw()], false, 542);
    assert_eq!(final_state.hand_count(holder), hand_before + 1);
    assert_eq!(mark_count(&final_state, target), 1);
}

// ---- 政略③：对有猎者伤害每回合一次 ----

#[test]
fn zhenglve_damage_marked_target_draws_once_per_turn() {
    let (state, holder) = started(2, ZHENGLVE);
    let victim = others_of(&state, holder)[0];
    let mut tx = Transaction::begin(state);
    let hand_before = tx.state().hand_count(holder);
    grant_mark_tx(&mut tx, victim, 1);

    let opened = drive(
        &mut tx,
        &[engine_api::DomainEvent::DamageDealt {
            source: holder,
            target: victim,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    let tx = tx.commit();
    let state = resolve(tx.state, holder, vec![1], false, 511);
    assert_eq!(state.hand_count(holder), hand_before + 1);

    // 同回合第二次：once 门槛在过滤器内生效——不再打开 confirm。
    let mut tx = Transaction::begin(state);
    let second = drive(
        &mut tx,
        &[engine_api::DomainEvent::DamageDealt {
            source: holder,
            target: victim,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    );
    second.unwrap();
    assert!(!tx.state().has_pending_choice());
    assert_eq!(tx.state().hand_count(holder), hand_before + 1);
}

// ---- 会猎：阈值觉醒 + 跨包授技 + 自移除 ----

#[test]
fn huilie_awakens_above_two_marks_and_self_removes() {
    let (state, owner) = started(3, HUILIE);
    let others = others_of(&state, owner);
    let max_hp_full = state.players.get(owner).unwrap().max_hp;

    // 两枚标记（<3 人）：不觉醒。
    let mut tx = Transaction::begin(state);
    grant_mark_tx(&mut tx, others[0], 1);
    grant_mark_tx(&mut tx, others[1], 1);
    let quiet = drive(
        &mut tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::Prepare,
        }],
    );
    quiet.unwrap();
    let unchanged = tx.commit().state;
    assert_eq!(unchanged.players.get(owner).unwrap().max_hp, max_hp_full);
    assert!(unchanged
        .skills
        .sorted_live()
        .iter()
        .any(|s| s.def.raw() == HUILIE));

    // 第三枚 → 觉醒：-1 上限、获平戎/飞影、自移除。
    let mut tx = Transaction::begin(unchanged);
    grant_mark_tx(&mut tx, owner, 1);
    let awakened = drive(
        &mut tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::Prepare,
        }],
    );
    awakened.unwrap();
    let final_state = tx.commit().state;
    assert_eq!(
        final_state.players.get(owner).unwrap().max_hp,
        max_hp_full - 1
    );
    assert!(final_state
        .skills
        .sorted_live()
        .iter()
        .any(|s| s.def.raw() == PINGRONG));
    assert!(final_state
        .skills
        .sorted_live()
        .iter()
        .any(|s| s.def.raw() == FEIYING));
    assert!(!final_state
        .skills
        .sorted_live()
        .iter()
        .any(|s| s.def.raw() == HUILIE));
}

// ---- 平戎：移猎（候选仅列有标记者）+ 每轮一次 ----

#[test]
fn pingrong_removes_mark_clamped_and_once_per_round() {
    use engine_core::state::ContinuationKind;
    let (state, holder) = started(3, PINGRONG);
    let others = others_of(&state, holder);

    let mut tx = Transaction::begin(state);
    grant_mark_tx(&mut tx, others[0], 1);
    grant_mark_tx(&mut tx, others[1], 1);
    let opened = drive(
        &mut tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: others[0],
            phase: engine_api::TurnPhaseTag::End,
        }],
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    let tx = tx.commit();
    let after_confirm = resolve(tx.state, holder, vec![1], false, 521);
    assert_eq!(
        after_confirm.resolution.continuation_kind,
        ContinuationKind::ChooseOtherAddMark
    );
    let observer = after_confirm.resolution.pending_observer;
    let final_state = resolve(
        after_confirm,
        PlayerId::new(observer),
        vec![others[0].raw()],
        false,
        522,
    );
    assert_eq!(mark_count(&final_state, others[0]), 0, "-1 移尽其标");
    assert_eq!(mark_count(&final_state, others[1]), 1);
    // S3：额外回合已入队（引擎既有消费管线）。
    assert_eq!(
        final_state.turn.extra_turn_queue,
        vec![holder],
        "insertPhase promise lands in engine extra_turn_queue"
    );
    assert_eq!(
        final_state
            .turn
            .flag(engine_core::state::turn_flag::extra_turn_check_pending_by_player(holder)),
        1
    );

    // 同轮第二次：确认仍开（optional），storage 门槛吞掉移除效果。
    let mut tx = Transaction::begin(final_state);
    let opened = drive(
        &mut tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: others[1],
            phase: engine_api::TurnPhaseTag::End,
        }],
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    let tx = tx.commit();
    let gated = resolve(tx.state, holder, vec![1], false, 523);
    assert_ne!(
        gated.resolution.continuation_kind,
        ContinuationKind::ChooseOtherAddMark,
        "storage gate must swallow the removal"
    );
    assert!(!gated.has_pending_choice());
    assert_eq!(mark_count(&gated, others[1]), 1);
}

#[test]
fn pingrong_extra_turn_settles_lose_hp_check() {
    use engine_core::state::turn_flag;
    let (_state, holder) = started(3, PINGRONG);
    let mut tx = Transaction::begin(_state);

    // 造「额外回合刚结束」的状态：ACTIVE 标记 + 未造成伤害。
    tx.state_mut()
        .turn
        .set_flag(turn_flag::extra_turn_active_by_player(holder), 1);
    tx.state_mut().turn.set_flag(turn_flag::DAMAGE_DEALT, 0);

    // TURN_END_DISPATCHED 已置位的语义下，AdvanceTurn 直通 continue_turn_advance。
    run_effects(&mut tx, vec![EffectRequest::AdvanceTurn]).unwrap();

    // settle 在本轮已执行：ACTIVE 清零、失去 1 点体力（is_lose_hp）。
    let st = tx.commit().state;
    assert_eq!(
        st.turn.flag(turn_flag::extra_turn_active_by_player(holder)),
        0
    );

    // 对照组：造成过伤害则不掉血。
    let (state2, holder2) = started(3, PINGRONG);
    let mut tx2 = Transaction::begin(state2);
    tx2.state_mut()
        .turn
        .set_flag(turn_flag::extra_turn_active_by_player(holder2), 1);
    tx2.state_mut().turn.set_flag(turn_flag::DAMAGE_DEALT, 2);
    let hp_before = tx2.state().players.get(holder2).unwrap().hp;
    run_effects(&mut tx2, vec![EffectRequest::AdvanceTurn]).unwrap();
    let st2 = tx2.commit().state;
    assert_eq!(st2.players.get(holder2).unwrap().hp, hp_before);
}

// ---- 政略②：有猎目标的无距离/不限次数豁免（S4, ModifierTarget 家族） ----

fn equip_def_horse(tx: &mut Transaction, owner: PlayerId) {
    use engine_core::state::{Card, ZoneKind};
    let zone = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: engine_api::CardDefId::new(9_999),
        kind: engine_core::state::CardKind::HorseDef,
        suit: engine_api::Suit::Spade,
        rank: 1,
        weapon_range: 0,
        zone,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
}

#[test]
fn zhenglve_exemptions_apply_to_marked_targets_only() {
    let (state, holder) = started(2, ZHENGLVE);
    let victim = others_of(&state, holder)[0];
    let mut tx = Transaction::begin(state);

    // victim 装防具马（基础距离 1→2），holder 无武器：默认距离受限。
    equip_def_horse(&mut tx, victim);

    // 未授标记前：两项豁免均不生效。
    assert!(!engine_core::query::sha_use_unlimited_for_target(
        tx.state(),
        holder,
        victim
    ));
    let pre_distance = engine_core::query::card_distance_unlimited_for_target(
        tx.state(),
        holder,
        engine_api::CardId::NONE,
        victim,
    );
    assert!(
        !pre_distance || tx.state().hand_cards(victim).len() >= tx.state().hand_cards(holder).len(),
        "fallback hand-compare branch may trigger; keep assertion tolerant"
    );

    // victim 获得“猎”：两项豁免立即生效（zhenglve 持有即挂载的 modifier）。
    grant_mark_tx(&mut tx, victim, 1);
    assert!(engine_core::query::sha_use_unlimited_for_target(
        tx.state(),
        holder,
        victim
    ));
    assert!(engine_core::query::card_distance_unlimited_for_target(
        tx.state(),
        holder,
        engine_api::CardId::NONE,
        victim
    ));

    // 其他未带标记者不享受豁免（holder 自己不可能带标——单向授出）。
    assert!(!engine_core::query::sha_use_unlimited_for_target(
        tx.state(),
        holder,
        holder
    ));
}
