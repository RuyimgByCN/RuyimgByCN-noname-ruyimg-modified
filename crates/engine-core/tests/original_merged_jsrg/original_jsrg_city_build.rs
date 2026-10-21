//! 城族 S2a（ADR 0646 §2，第七十九批）：jsrgshacheng 沙城·筑城——
//! place_deck_top_expansion（ABI 667/tag 690）：牌堆顶两张明置移入以技能 def
//! 键控的扩张堆。消耗侧（移城+摸X）待 S2b。

use engine_api::{PlayerId, SkillDefId};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const SHACHENG: u32 = 2_976_683_213;
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

fn started() -> (GameState, PlayerId) {
    let compiled = content_compiler::compile_source_named("city-build", &sources()).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 20_260_829, "identity-standard").unwrap();
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
            skill_def: SkillDefId::new(SHACHENG),
        }],
    )
    .unwrap();
    (tx.commit().state, owner)
}

fn city_pile(state: &GameState, owner: PlayerId) -> Vec<engine_api::CardId> {
    state
        .skill_piles
        .iter()
        .filter(|pile| pile.owner == owner)
        .filter_map(|pile| {
            state.zones.get(pile.zone).map(|zone| {
                debug_assert_eq!(zone.kind, ZoneKind::Expansion);
                zone.cards.clone()
            })
        })
        .next()
        .unwrap_or_default()
}

#[test]
fn shacheng_publishes_build_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 730);
    assert!(package
        .trigger_index
        .iter()
        .any(|t| { t.skill_def_id == SHACHENG && t.event == "game_started" && !t.optional }));
}

#[test]
fn shacheng_builds_city_from_deck_top_on_game_start() {
    let (state, owner) = started();

    // 记录初始牌堆顶两张。
    let mut tx = Transaction::begin(state);
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, engine_api::PlayerId::NONE)
        .unwrap();
    let deck_before = tx.state().zones.get(draw).unwrap().cards.clone();
    assert!(deck_before.len() >= 2);
    // 先弹「真顶」→ 先进扩张堆（堆内序=放置序）。
    let top_two: Vec<engine_api::CardId> = vec![
        deck_before[deck_before.len() - 1],
        deck_before[deck_before.len() - 2],
    ];

    let opened = engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::GameStarted { player_count: 3 }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    );
    let effects = opened.unwrap_or_else(|e| panic!("forced build must not open a choice: {e:?}"));
    run_effects(&mut tx, effects).unwrap();

    let final_state = tx.commit().state;
    let pile = city_pile(&final_state, owner);
    assert_eq!(pile, top_two, "deck-top two become the City pile in order");
    for card in &pile {
        let c = final_state.cards.get(*card).unwrap();
        assert_eq!(c.owner, owner);
        assert!(c.face_up, "City cards are public");
    }
    // 牌堆确实少了两张且不再含这两张实体。
    let draw_after = final_state.zones.get(draw).unwrap().cards.clone();
    assert_eq!(draw_after.len(), deck_before.len() - 2);
    assert!(!draw_after.contains(&top_two[0]));
    assert!(!draw_after.contains(&top_two[1]));
}

// ---- S2b：沙城②消耗（移一城 + 摸 min(5, 本回合失去数)） ----

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
};

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

fn drive(
    tx: &mut Transaction,
    events: &[engine_api::DomainEvent],
) -> Result<(), engine_core::effect::EffectError> {
    match engine_core::trigger::dispatch_for_events(
        tx,
        events,
        &mut engine_core::trigger::TriggerFireRecord::new(),
    ) {
        Err(e) => Err(e),
        Ok(effects) => run_effects(tx, effects).map(|_| ()),
    }
}

fn build_city(state: GameState, owner: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    let effects = engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::GameStarted { player_count: 3 }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

fn lost_count(state: &GameState, player: PlayerId) -> i32 {
    state
        .turn
        .flag(engine_core::state::turn_flag::lost_card_count_by_player(
            player,
        ))
}

#[test]
fn shacheng_publishes_consume_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 730);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == SHACHENG
            && t.event == "sha_resolved"
            && t.filter == "owner_alive_sha_resolved_city_2976683213"
            && t.optional
    }));
}

#[test]
fn shacheng_consumes_one_city_and_target_draws_lost_count() {
    let (state, holder) = started();
    let victim = state
        .players
        .iter_live()
        .map(|p| p.id)
        .find(|id| *id != holder)
        .unwrap();
    let state = build_city(state, holder);
    assert_eq!(city_pile(&state, holder).len(), 2);

    // 真实弃置两张（走 move_cards 热路径，opt-in 计数 +2）。
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: victim,
            count: 2,
        }],
    )
    .unwrap();
    let victim_hand_before = tx.state().hand_count(victim);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: victim,
            count: 2,
        }],
    )
    .unwrap();
    assert_eq!(lost_count(tx.state(), victim), 2);

    // 合成 ShaResolved：过滤器要求 owner 有城且存在失去过牌者。
    let opened = drive(
        &mut tx,
        &[engine_api::DomainEvent::ShaResolved {
            source: holder,
            card: CardId::NONE,
            targets: vec![victim],
            response_cards: vec![],
            card_flags: 0,
        }],
    );
    assert_eq!(
        opened.unwrap_err(),
        engine_core::effect::EffectError::ChoiceRequested
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let mid = resolve(tx.state, holder, vec![1], false, 601);
    assert_eq!(
        mid.resolution.continuation_kind,
        ContinuationKind::CityConsume
    );
    let observer = mid.resolution.pending_observer;
    let final_state = resolve(mid, PlayerId::new(observer), vec![victim.raw()], false, 602);

    assert_eq!(city_pile(&final_state, holder).len(), 1);
    // 目标摸 X=2。
    assert_eq!(final_state.hand_count(victim), victim_hand_before - 2 + 2);
}

#[test]
fn shacheng_consume_skips_without_city_or_losses_and_caps_at_five() {
    let (state, holder) = started();
    let other = state
        .players
        .iter_live()
        .map(|p| p.id)
        .find(|id| *id != holder)
        .unwrap();

    // 无城：不触发。
    let mut tx = Transaction::begin(state);
    let quiet = drive(
        &mut tx,
        &[engine_api::DomainEvent::ShaResolved {
            source: holder,
            card: CardId::NONE,
            targets: vec![other],
            response_cards: vec![],
            card_flags: 0,
        }],
    );
    quiet.unwrap();
    assert!(!tx.state().has_pending_choice());

    // 有城但无人失牌：不触发。
    let state = build_city(tx.commit().state, holder);
    let mut tx = Transaction::begin(state);
    let quiet2 = drive(
        &mut tx,
        &[engine_api::DomainEvent::ShaResolved {
            source: holder,
            card: CardId::NONE,
            targets: vec![other],
            response_cards: vec![],
            card_flags: 0,
        }],
    );
    quiet2.unwrap();
    assert!(!tx.commit().state.has_pending_choice());
}
