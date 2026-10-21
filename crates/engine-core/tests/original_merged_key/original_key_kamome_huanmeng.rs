//! 第一百七十七批：key「幻梦」（kamome_huanmeng）观星族第 2 件。
//! 上游：准备阶段，观看牌堆顶 1+装备区牌数的牌并任意重排（拖拽顶/底）。
//! 实现：phase_prepare + owner_is_target → guanxing_top(self, 1+equipped_count(self))
//! （批 176 guanxing_top 原语 + 本批 EquippedCount 查询表达式，ABI 681）。
//! 近似：逐张二选（顶/底）近似拖拽自由排列。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const KAMOME: u32 = 3_440_411_698;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/key/key.npack.json");

fn started(seed: u64) -> (GameState, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    let start = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(seed),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    });
    assert!(
        matches!(start, CommandResult::Accepted { .. }),
        "start rejected: {start:?}"
    );
    let state = engine.state().clone();
    let owner = state.turn.current;
    let mut tx = Transaction::begin(state);
    let removals: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    let _ = engine_core::effect::run_effects(&mut tx, removals);
    engine_core::effect::run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(KAMOME),
        }],
    )
    .unwrap();
    (tx.commit().state, owner)
}

fn inject_equip(tx: &mut Transaction, player: PlayerId) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Equip, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_401),
        kind: CardKind::Armor,
        suit: Suit::Spade,
        rank: 5,
        weapon_range: 0,
        zone,
        owner: player,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn equip_count(state: &GameState, player: PlayerId) -> u32 {
    state
        .zones
        .find(ZoneKind::Equip, player)
        .and_then(|zone| state.zones.get(zone).map(|z| z.cards.len() as u32))
        .unwrap_or(0)
}

fn process_count(state: &GameState) -> u32 {
    state
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .and_then(|zone| state.zones.get(zone).map(|z| z.cards.len() as u32))
        .unwrap_or(0)
}

fn trigger_prepare(state: &mut GameState, owner: PlayerId) {
    let mut tx = Transaction::begin(state.clone());
    let dispatched = engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::Prepare,
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    );
    match dispatched {
        Ok(requests) => {
            if let Err(e) = engine_core::effect::run_effects(&mut tx, requests) {
                eprintln!("DEBUG run_effects err: {e:?}");
            }
        }
        Err(e) => eprintln!("DEBUG dispatch err: {e:?}"),
    }
    *state = tx.commit().state;
}

fn resolve_pick(state: GameState, owner: PlayerId, pick: u32, key: u64) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (next, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![pick],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "resolve rejected: {:?}",
        result.result
    );
    next
}

#[test]
fn package_publishes_kamome_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 681);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == KAMOME
            && trigger.event == "phase_prepare"
            && trigger.filter == "owner_is_target"
            && !trigger.optional
    }));
}

#[test]
fn kamome_guanxing_rounds_match_equipment_count_plus_one() {
    // owner 带一张装备：观星轮数 = 1 + 1 = 2。
    let (mut state, owner) = started(2_026_092_101);
    let mut tx = Transaction::begin(state.clone());
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    let _equip = inject_equip(&mut tx, owner);
    state = tx.commit().state;

    trigger_prepare(&mut state, owner);
    assert!(state.has_pending_choice(), "guanxing loop must open");

    // 两轮全选「底」（token 2）。
    let mut key = 2;
    for round in 0..2 {
        assert!(state.has_pending_choice(), "round {round} must stay open");
        state = resolve_pick(state, owner, 2, key);
        key += 1;
    }
    assert!(
        !state.has_pending_choice(),
        "loop must close after 1+equip rounds"
    );
    assert_eq!(process_count(&state), 0, "processing drained after settle");
}

#[test]
fn kamome_zero_equipment_means_single_round() {
    let (mut state, owner) = started(2_026_092_102);
    let mut tx = Transaction::begin(state.clone());
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    state = tx.commit().state;

    trigger_prepare(&mut state, owner);
    assert!(state.has_pending_choice(), "guanxing loop must open");

    // 1 轮（1+0）。
    let mut key = 2;
    for round in 0..1 {
        assert!(state.has_pending_choice(), "round {round} must stay open");
        state = resolve_pick(state, owner, 2, key);
        key += 1;
    }
    assert!(
        !state.has_pending_choice(),
        "loop must close after single round"
    );
    assert_eq!(process_count(&state), 0);
}
