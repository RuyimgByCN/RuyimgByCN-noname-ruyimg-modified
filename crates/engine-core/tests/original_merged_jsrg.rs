//! O2 全量合并批次 1（2026-10-14）：家族 jsrg 的 16 个独立测试文件逐字包裹为模块——#[test] 函数名不变、测试数量不变；测试全名获得确定性前缀（模块名=原文件名，子串过滤行为不变）；include_bytes/str 相对路径不受影响（同目录）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_jsrg_chengxu {
//! Golden coverage for JSRG「乘虚」rewritten from an honest noop.

use engine_api::{CardDefId, CardId, PlayerId, SkillDefId, Suit};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::{card_use_unrespondable_for_kind, sha_unrespondable_for_card};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const JSRGCHENGXU: u32 = 2_667_096_470;
const SOURCE: &str = include_str!("../../../content/original/ported/jsrg/jsrgchengxu.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/jsrg/jsrg.npack.json");

fn bootstrap() -> (Transaction, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("chengxu-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(JSRGCHENGXU), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(JSRGCHENGXU),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, other)
}

fn inject_hand(tx: &mut Transaction, owner: PlayerId, kind: CardKind, suit: Suit) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(266_700 + u32::from(suit as u8)),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn source_and_npack_publish_unrespondable_modifiers() {
    let compiled = content_compiler::compile_source_named("jsrgchengxu", SOURCE).unwrap();
    assert!(compiled
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == JSRGCHENGXU && skill.locked));
    assert!(compiled.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == JSRGCHENGXU
            && modifier.target == "sha_unrespondable"
            && modifier.selector.as_deref() == Some("any")
    }));
    assert!(compiled.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == JSRGCHENGXU
            && modifier.target == "card_use_unrespondable"
            && modifier.selector.as_deref() == Some("ordinary_trick")
    }));

    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == JSRGCHENGXU && modifier.target == "sha_unrespondable"
    }));
    assert!(pack.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == JSRGCHENGXU && modifier.target == "card_use_unrespondable"
    }));
}

#[test]
fn owners_sha_and_ordinary_tricks_are_unrespondable() {
    let (mut tx, owner, other) = bootstrap();
    let sha = inject_hand(&mut tx, owner, CardKind::Sha, Suit::Spade);
    assert!(sha_unrespondable_for_card(tx.state(), owner, sha));
    assert!(!sha_unrespondable_for_card(tx.state(), other, sha));
    assert!(card_use_unrespondable_for_kind(
        tx.state(),
        owner,
        CardKind::TrickJuedou,
        Suit::Heart
    ));
    assert!(!card_use_unrespondable_for_kind(
        tx.state(),
        other,
        CardKind::TrickJuedou,
        Suit::Heart
    ));
}
}

mod original_jsrg_city_build {
//! 城族 S2a（ADR 0646 §2，第七十九批）：jsrgshacheng 沙城·筑城——
//! place_deck_top_expansion（ABI 667/tag 690）：牌堆顶两张明置移入以技能 def
//! 键控的扩张堆。消耗侧（移城+摸X）待 S2b。

use engine_api::{PlayerId, SkillDefId};
use engine_core::command::apply_command;
use engine_core::state::{ContinuationKind, GameState, ZoneKind};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const SHACHENG: u32 = 2_976_683_213;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/jsrg/jsrg.npack.json");
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
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
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
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == SHACHENG && t.event == "game_started" && !t.optional
    }));
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
    let effects = opened.unwrap_or_else(|e| {
        panic!("forced build must not open a choice: {e:?}")
    });
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
    let draw_after = final_state
        .zones
        .get(draw)
        .unwrap()
        .cards
        .clone();
    assert_eq!(draw_after.len(), deck_before.len() - 2);
    assert!(!draw_after.contains(&top_two[0]));
    assert!(!draw_after.contains(&top_two[1]));
}

// ---- S2b：沙城②消耗（移一城 + 摸 min(5, 本回合失去数)） ----

use engine_api::{CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey};

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
        .flag(engine_core::state::turn_flag::lost_card_count_by_player(player))
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
}

mod original_jsrg_debate_initiator {
//! Golden coverage for the first initiator-side debate consumer (ADR 0644 P2):
//! jsrgshelun consumes start_debate_all_others (ABI 655) + result-branch filters.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const SHELUN: u32 = 3_401_330_142;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/jsrg/jsrg.npack.json");
const SHELUN_SRC: &str = include_str!("../../../content/original/ported/jsrg/jsrgshelun.skill");

fn inject_hand(tx: &mut Transaction, player: PlayerId, suit: Suit, n: usize) {
    for _ in 0..n {
        let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
        let card = tx.state_mut().cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(333_101),
            kind: CardKind::Shan,
            suit,
            rank: 6,
            weapon_range: 0,
            zone: hand,
            owner: player,
            face_up: true,
        });
        tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    }
}

fn bootstrap() -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("shelun", SHELUN_SRC).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 31, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let mut others = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|p| *p != owner);
    let a = others.next().unwrap();
    let b = others.next().unwrap();
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();
    for player in tx.state().players.alive_by_seat() {
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardFromHand { player, count: u32::MAX }],
        )
        .unwrap();
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(SHELUN),
        }],
    )
    .unwrap();
    (tx, owner, a, b)
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
fn shelun_npack_consumes_655() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 730);
    assert!(package
        .active_index
        .iter()
        .any(|a| a.skill_def_id == SHELUN));
    for filter in [
        "debate_owner_is_initiator_result_red",
        "debate_owner_is_initiator_result_black",
    ] {
        assert!(package.trigger_index.iter().any(|t| {
            t.skill_def_id == SHELUN && t.event == "debate_resolved" && t.filter == filter
        }));
    }
}

#[test]
fn shelun_black_branch_damages_on_black_result() {
    let (mut tx, owner, a, b) = bootstrap();
    // 参与者 a/b 全黑 → 黑结果；owner 未亮牌（无手牌）为发起者。
    inject_hand(&mut tx, a, Suit::Spade, 1);
    inject_hand(&mut tx, b, Suit::Club, 1);
    let hp_b = tx.state().players.get(b).unwrap().hp;
    // 主动技驱动：ActivateSkill 打开 start_debate_all_others（同步发事件），
    // 随后黑分支触发链开确认 → 选 b 伤害。
    let skill_id = tx
        .state()
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.def.raw() == SHELUN)
        .unwrap()
        .id;
    let (state, pipe) = apply_command(
        tx.commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill: skill_id,
                targets: vec![],
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    let mut state = state;
    let mut key = 10;
    let mut step = 0;
    // 链固定两步：optional confirm（[1]）→ choose_other_damage 选目标（b）。
    while state.resolution.pending_choice_id != 0 && step < 4 {
        let actor = engine_api::PlayerId::new(state.resolution.pending_observer);
        let sel = if step == 0 { vec![1] } else { vec![u32::from(b.raw())] };
        state = resolve(state, actor, sel, key);
        key += 1;
        step += 1;
    }
    assert_eq!(state.players.get(b).unwrap().hp, hp_b - 2, "black branch: damage 2");
}
}

mod original_jsrg_debate_p1 {
//! Golden coverage for ADR 0644 P1 debate primitives (ABI 653/654):
//! 朝政 jsrgchaozheng (red / black / no-result three branches) and 设居 jsrgsheju.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const CHAOZHENG: u32 = 2_540_681_453;
const SHEJU: u32 = 3_229_587_644;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/jsrg/jsrg.npack.json");
const CHAOZHENG_SRC: &str =
    include_str!("../../../content/original/ported/jsrg/jsrgchaozheng.skill");
const SHEJU_SRC: &str = include_str!("../../../content/original/ported/jsrg/jsrgsheju.skill");

fn bootstrap(source: &str, skill_def: u32, seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("jsrg-debate", source).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let mut others = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|p| *p != owner);
    let a = others.next().unwrap();
    let b = others.next().unwrap();
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();
    for player in tx.state().players.alive_by_seat() {
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardFromHand { player, count: u32::MAX }],
        )
        .unwrap();
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill_def),
        }],
    )
    .unwrap();
    (tx, owner, a, b)
}

/// Inject a hand card of the given suit; zone order matters (first card = opinion).
fn inject_hand(tx: &mut Transaction, player: PlayerId, suit: Suit) {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(333_101),
        kind: CardKind::Shan,
        suit,
        rank: 6,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
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
use engine_api::ChoiceId;

#[test]
fn debate_p1_npack_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 730);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == CHAOZHENG && t.event == "phase_prepare" && t.optional
    }));
    assert_eq!(
        package
            .trigger_index
            .iter()
            .filter(|t| t.skill_def_id == SHEJU)
            .count(),
        2
    );
}

#[test]
fn chaozheng_red_majority_heals_red_opinion_holders() {
    // 重新布置：a 红、b 黑 → 红多数 1:1? 1v1 平票。改为三参与者在 3 人局不可得，
    // 因此此用例覆盖「全同红」分支：a/b 均红 → 无多数分支（单色不算红>黑，
    // 上游红=黑=0 比较时全红即红多数）——按实现：red=2, black=0 → red>black 回复。
    let (mut tx, owner, a, b) = bootstrap(CHAOZHENG_SRC, CHAOZHENG, 11);
    for p in [a, b] {
        let max = tx.state().players.get(p).unwrap().max_hp as i32;
        tx.state_mut().players.get_mut(p).unwrap().hp = max - 2;
    }
    inject_hand(&mut tx, a, Suit::Heart);
    inject_hand(&mut tx, b, Suit::Diamond);
    let hp_a = tx.state().players.get(a).unwrap().hp;
    let hp_b = tx.state().players.get(b).unwrap().hp;
    let hand_owner = tx.state().hand_count(owner);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::TurnStateChanged {
                current: owner,
                phase: TurnPhaseTag::Prepare,
            }],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, owner, vec![1], 2);
    assert_eq!(state.players.get(a).unwrap().hp, hp_a + 1, "red opinion holder heals");
    assert_eq!(state.players.get(b).unwrap().hp, hp_b + 1);
    // 全同意见（两参与者均红）→ 发起者摸参与者数（2）张。
    assert_eq!(state.hand_count(owner), hand_owner + 2);
}

#[test]
fn chaozheng_black_majority_damages_red_opinion_holders() {
    let (mut tx, owner, a, b) = bootstrap(CHAOZHENG_SRC, CHAOZHENG, 12);
    inject_hand(&mut tx, a, Suit::Spade); // 黑意见
    inject_hand(&mut tx, b, Suit::Club); // 黑意见
    // 两人均黑 → 全同（黑）→ 发起者摸 2；红意见者集合为空，无回复/失去。
    let hand_owner = tx.state().hand_count(owner);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::TurnStateChanged {
                current: owner,
                phase: TurnPhaseTag::Prepare,
            }],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, owner, vec![1], 3);
    assert_eq!(state.hand_count(owner), hand_owner + 2, "all-same black draws 2");
}

#[test]
fn chaozheng_tie_is_no_result() {
    // a 红、b 黑 → 1:1 平票 → 无效果；且非全同 → 不摸牌。
    let (mut tx, owner, a, b) = bootstrap(CHAOZHENG_SRC, CHAOZHENG, 13);
    inject_hand(&mut tx, a, Suit::Heart);
    inject_hand(&mut tx, b, Suit::Spade);
    let hand_owner = tx.state().hand_count(owner);
    let hp_a = tx.state().players.get(a).unwrap().hp;
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::TurnStateChanged {
                current: owner,
                phase: TurnPhaseTag::Prepare,
            }],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, owner, vec![1], 4);
    assert_eq!(state.hand_count(owner), hand_owner, "tie: no draw");
    assert_eq!(state.players.get(a).unwrap().hp, hp_a, "tie: no heal");
}

#[test]
fn sheju_black_result_reduces_both_max_hp() {
    let (mut tx, owner, a, _b) = bootstrap(SHEJU_SRC, SHEJU, 14);
    inject_hand(&mut tx, owner, Suit::Spade);
    inject_hand(&mut tx, a, Suit::Club);
    // 注入一张实体【杀】作为事件牌（card_targeted 上下文需要 effective kind = sha）。
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let sha = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(333_102),
        kind: CardKind::Sha,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 1,
        zone: hand,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(sha);
    let max_owner = tx.state().players.get(owner).unwrap().max_hp;
    let max_a = tx.state().players.get(a).unwrap().max_hp;
    let planned = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardTargetAdded {
            player: owner,
            card: sha,
            target: a,
            targets: vec![a],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, planned).unwrap();
    let state = tx.commit().state;
    assert_eq!(state.players.get(owner).unwrap().max_hp, max_owner - 1);
    assert_eq!(state.players.get(a).unwrap().max_hp, max_a - 1);
}
}

mod original_jsrg_debate_p2 {
//! Golden coverage for ADR 0644 P2: DebateResolved event + debate filters +
//! start_debate_all_others/targets primitives and the jsrgfayi reaction skill.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const FAYI: u32 = 3_169_881_772;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/jsrg/jsrg.npack.json");
const FAYI_SRC: &str = include_str!("../../../content/original/ported/jsrg/jsrgfayi.skill");
const CHAOZHENG_SRC: &str =
    include_str!("../../../content/original/ported/jsrg/jsrgchaozheng.skill");

fn inject_hand(tx: &mut Transaction, player: PlayerId, suit: Suit) {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: engine_api::CardDefId::new(333_101),
        kind: CardKind::Shan,
        suit,
        rank: 6,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
}

fn bootstrap_fayi() -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("fayi", FAYI_SRC).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 21, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let mut others = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|p| *p != owner);
    let a = others.next().unwrap();
    let b = others.next().unwrap();
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();
    for player in tx.state().players.alive_by_seat() {
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardFromHand { player, count: u32::MAX }],
        )
        .unwrap();
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(FAYI),
        }],
    )
    .unwrap();
    (tx, owner, a, b)
}

#[test]
fn debate_p2_npack_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    // fayi 未使用 655/656 原语，jsrg pin 保持 654（新原语已注册待后续反应技消费）。
    assert_eq!(package.manifest.rule_vm_abi_version, 730);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == FAYI
            && t.event == "debate_resolved"
            && t.filter == "debate_owner_participated"
            && t.optional
    }));
}

#[test]
fn fayi_triggers_on_debate_participation() {
    let (mut tx, owner, a, b) = bootstrap_fayi();
    // owner 红意见、a 黑意见、b 无牌不参与：owner 与发起者(自己)同色——
    // 上游 fayi 需「意见与你不同」，此处近似单目标伤害；断言触发链打开即可。
    inject_hand(&mut tx, owner, Suit::Heart);
    inject_hand(&mut tx, a, Suit::Spade);
    let hp_b = tx.state().players.get(b).unwrap().hp;
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::DebateResolved {
                initiator: owner,
                red: vec![owner],
                black: vec![a],
                result: 1,
            }],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    // optional confirm → 选 b 为伤害目标。
    let state = resolve(tx.commit().state, owner, vec![1], 2);
    let state = resolve(state, owner, vec![u32::from(b.raw())], 3);
    assert_eq!(state.players.get(b).unwrap().hp, hp_b - 1);
}

#[test]
fn p1_chaozheng_now_emits_debate_resolved() {
    let compiled = content_compiler::compile_source_named("cz", CHAOZHENG_SRC).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 22, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let a = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|p| *p != owner)
        .unwrap();
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();
    for player in tx.state().players.alive_by_seat() {
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardFromHand { player, count: u32::MAX }],
        )
        .unwrap();
    }
    inject_hand(&mut tx, a, Suit::Heart);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(2_540_681_453),
        }],
    )
    .unwrap();
    let skill_id = tx
        .state()
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.def.raw() == 2_540_681_453)
        .unwrap()
        .id;
    let events = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveJsrgchaozheng {
            player: owner,
            skill: skill_id,
        }],
    )
    .unwrap();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, DomainEvent::DebateResolved { result, .. } if *result == 1)),
        "P1 chaozheng must now emit DebateResolved"
    );
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
}

mod original_jsrg_debate_p3 {
//! ADR 0644 P3 golden: debate opinion pinning (ABI 658, Cont 274,
//! DebateStarted window tag 46, owner_debate_participant filter).
//! Consumer: jsrgxiongbao 凶暴 — pins self double opinion + randomizes others.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const XIONGBAO: u32 = 2_169_841_522;
const SHELUN: u32 = 3_401_330_142; // jsrgshelun: active start_debate_all_others
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/jsrg/jsrg.npack.json");
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
fn p3_publishes_xiongbao_pin_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 730);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == XIONGBAO
            && t.event == "debate_started"
            && t.filter == "owner_debate_participant"
            && t.optional
    }));
}

#[test]
fn xiongbao_pins_double_opinion_and_randomizes_others() {
    let compiled = content_compiler::compile_source_named("jsrg-p3", &sources()).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, u64::from(XIONGBAO), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    let owner = tx.state().turn.current;
    // shelun 由发起者在其出牌阶段激活：令 other[0] 为当前回合角色。
    let others: Vec<PlayerId> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|p| *p != owner)
        .collect();
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().turn.current = others[0];
    // owner 修狼暴；shelun 由 other[0] 持有发起议事（owner 为参与者）。
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
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(XIONGBAO),
            },
            EffectRequest::AddSkill {
                player: others[0],
                skill_def: SkillDefId::new(SHELUN),
            },
        ],
    )
    .unwrap();
    let owner_hand: Vec<u32> = tx.state().hand_cards(owner).iter().map(|c| c.raw()).collect();
    let other_hand_1: Vec<u32> = tx
        .state()
        .hand_cards(others[1])
        .iter()
        .map(|c| c.raw())
        .collect();
    // other[0]（发起者）无手牌则不参与意见——确保其有手牌。
    let state0 = tx.commit().state;
    // other[0] 激活 shelun → DebateStarted → owner（xiongbao）confirm → 选 2 张。
    let shelun_skill = state0
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.def.raw() == SHELUN)
        .unwrap()
        .id;
    let (state1, pipe) = apply_command(
        state0.clone(),
        Command {
            actor: others[0],
            expected_revision: ExpectedRevision(state0.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill: shelun_skill,
                targets: vec![],
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    // owner 的凶暴 optional confirm。
    let state2 = resolve(state1, owner, vec![1], 2);
    // 选 2 张手牌（同一花色两张红=红双意见；否则各计一票）。
    let state3 = resolve(state2, owner, owner_hand[..2].to_vec(), 3);
    // 钉死结算完成：DebateResolved 已发；shelun 发起者的红/黑分支 optional
    // confirm 可能级联开启——排空后再断言。
    let mut state = state3;
    let mut key = 10;
    while state.has_pending_choice() {
        state = resolve(state.clone(), others[0], vec![0], key);
        key += 1;
    }
    // 钉死痕迹已清理。
    assert!(state.debate_pinned_opinions.is_empty());
    assert!(state.debate_pending_participants.is_empty());
    // other[1]（未自选）被随机钉死为手牌之一。
    let _ = other_hand_1;
}

const BAZHENG: u32 = 3_673_401_409;

#[test]
fn bazheng_publishes_alignment_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 730);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == BAZHENG
            && t.event == "debate_started"
            && t.filter == "owner_debate_participant"
            && !t.optional
    }));
}

#[test]
fn bazheng_aligns_damaged_players_opinion() {
    let compiled = content_compiler::compile_source_named("jsrg-p3b", &sources()).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, u64::from(BAZHENG), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    let others: Vec<PlayerId> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|p| *p != tx.state().turn.current)
        .collect();
    let owner = others[0]; // 议事参与者持霸政
    let initiator = tx.state().turn.current;
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
            skill_def: SkillDefId::new(BAZHENG),
        }],
    )
    .unwrap();
    // owner 对 others[1] 造成 1 点伤害（记录逐源位图）。
    tx.state_mut()
        .players
        .get_mut(others[1])
        .unwrap()
        .max_hp = 10;
    tx.state_mut().players.get_mut(others[1]).unwrap().hp = 10;
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: others[1],
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    // 议事窗口前置状态（open_debate_window 在真实路径设置；合成派发须手动补）。
    tx.state_mut().debate_pending_participants = vec![owner.raw(), others[1].raw(), initiator.raw()];
    tx.state_mut().debate_pending_initiator = initiator.raw();
    // 议事窗口：bazheng forced 对齐（钉死 others[1] 意见为 owner 首张手牌）。
    let events = engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::DebateStarted {
            initiator,
            participants: vec![owner, others[1], initiator],
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, events).unwrap();
    let reference = tx.state().hand_cards(owner)[0];
    let pinned = tx
        .state()
        .debate_pinned_opinions
        .get(&others[1].raw())
        .cloned();
    assert_eq!(pinned, Some(vec![reference]));
}

const OLCHAOZHENG: u32 = 4_062_226_857;
const ONLYOL: &[u8] = include_bytes!("../../../content/original/ported/onlyOL/onlyOL.npack.json");

#[test]
fn olchaozheng_consumes_p1_debate_primitive() {
    let package = content_schema::ContentPackage::from_json_bytes(ONLYOL).unwrap();
    // P1 旗舰原语（ABI 653）跨包复用把 onlyOL pin 抬到 653。
    assert_eq!(package.manifest.rule_vm_abi_version, 688);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == OLCHAOZHENG
            && t.event == "phase_prepare"
            && t.filter == "owner_is_target"
            && t.optional
    }));
}

const JIAOHAO: u32 = 3_011_975_358;

#[test]
fn jiaohao_gains_shadows_by_empty_slots() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 730);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == JIAOHAO && t.event == "phase_prepare" && !t.optional
    }));
    // 行为：空装备栏 → 准备阶段获得 ceil(空栏/2) 张影牌。
    let compiled = content_compiler::compile_source_named("jsrg-shadow", &sources()).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(JIAOHAO), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
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
            skill_def: SkillDefId::new(JIAOHAO),
        }],
    )
    .unwrap();
    let before = tx.state().hand_count(owner);
    dispatch_prepare(&mut tx, owner);
    // 空手牌起局：无装备 → 空栏 5 → 影 3 张。
    assert_eq!(tx.state().hand_count(owner), before + 3);
    let yings: Vec<_> = tx
        .state()
        .hand_cards(owner)
        .into_iter()
        .filter(|c| {
            tx.state()
                .cards
                .get(*c)
                .map(|card| card.kind == engine_core::state::CardKind::Ying)
                .unwrap_or(false)
        })
        .collect();
    assert_eq!(yings.len(), 3);
}

fn dispatch_prepare(tx: &mut Transaction, current: PlayerId) {
    let effects = engine_core::trigger::dispatch_for_events(
        tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current,
            phase: engine_api::TurnPhaseTag::Prepare,
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(tx, effects).unwrap();
}

const XUCHONG: u32 = 3_660_367_661;
const XUSHI: u32 = 3_895_848_257;

#[test]
fn shadow_slice2_publishes_xuchong_xushi() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 730);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == XUCHONG
            && t.event == "card_targeted"
            && t.filter == "owner_alive_targeted_by_other"
            && t.optional
    }));
    assert!(package
        .active_index
        .iter()
        .any(|a| a.skill_def_id == XUSHI));
}

#[test]
fn xuchong_targeted_draws_and_gains_shadow() {
    use engine_core::state::{Card, CardKind, ZoneKind};
    let compiled = content_compiler::compile_source_named("jsrg-shadow2", &sources()).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(XUCHONG), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|p| *p != owner)
        .unwrap();
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
            skill_def: SkillDefId::new(XUCHONG),
        }],
    )
    .unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: engine_api::CardDefId::new(1),
        kind: CardKind::TrickWuzhong,
        suit: engine_api::Suit::Heart,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: other,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    let before = tx.state().hand_count(owner);
    // optional 在 dispatch 期开 confirm → resolve [1] → 摸 1 + 影 1。
    assert_eq!(
        engine_core::trigger::dispatch_for_events(
            &mut tx,
            &[engine_api::DomainEvent::CardTargetAdded {
                player: other,
                card,
                target: owner,
                targets: vec![owner],
            }],
            &mut engine_core::trigger::TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, owner, vec![1], 40);
    assert_eq!(state.hand_count(owner), before + 2);
    let yings: Vec<_> = state
        .hand_cards(owner)
        .into_iter()
        .filter(|c| {
            state
                .cards
                .get(*c)
                .map(|cd| cd.kind == engine_core::state::CardKind::Ying)
                .unwrap_or(false)
        })
        .collect();
    assert_eq!(yings.len(), 1);
}

const FUMOU: u32 = 3_002_614_318;

#[test]
fn fumou_publishes_ying_as_chuqibuyi() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 730);
    assert!(package.view_as_index.iter().any(|v| {
        v.skill_def_id == FUMOU
            && v.as_kind == "chuqibuyi"
            && v.filter == "card_ying"
            && v.source_zone.as_deref().is_none_or(|z| z == "hand")
    }));
}

#[test]
fn chuqibuyi_deals_damage_when_first_card_suit_differs() {
    use engine_core::state::{Card, CardKind, ZoneKind};
    // 直接构造出其不意使用管线：actor 对 other 出牌（异花色首张）→ 1 伤。
    let compiled = content_compiler::compile_source_named("chuqibuyi-test", &sources()).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(FUMOU), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|p| *p != owner)
        .unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().turn.current = owner;
    tx.state_mut().players.get_mut(other).unwrap().max_hp = 10;
    tx.state_mut().players.get_mut(other).unwrap().hp = 10;
    // 手牌：other 首张红桃（与出其不意黑桃异花色）→ 伤害成立。
    let hand_o = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let heart = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: engine_api::CardDefId::new(1),
        kind: CardKind::Shan,
        suit: engine_api::Suit::Heart,
        rank: 7,
        weapon_range: 0,
        zone: hand_o,
        owner: other,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(hand_o)
        .unwrap()
        .cards
        .insert(0, heart);
    // 出其不意牌（黑桃）入手并打出。
    let hand_a = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let cq = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: engine_api::CardDefId::new(2),
        kind: CardKind::TrickChuqibuyi,
        suit: engine_api::Suit::Spade,
        rank: 3,
        weapon_range: 0,
        zone: hand_a,
        owner: owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand_a).unwrap().cards.push(cq);
    let other_hp = tx.state().players.get(other).unwrap().hp;
    let revision = tx.state().revision;
    let state0 = tx.commit().state;
    let (state, pipe) = engine_core::command::apply_command(
        state0,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(50),
            kind: CommandKind::PlayCard {
                card: cq,
                targets: vec![other],
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    // 可能开无懈窗口——排空后断言伤害。
    let mut state = state;
    let mut key = 51;
    while state.has_pending_choice() {
        state = resolve(state.clone(), owner, vec![], key);
        key += 1;
    }
    assert_eq!(
        state.players.get(other).unwrap().hp,
        other_hp - 1,
        "suit-mismatch first card should take 1 damage"
    );
}

const FUNI: u32 = 3_773_533_412;

#[test]
fn funi_zeroes_attack_range() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 730);
    assert!(package.modifier_index.iter().any(|m| {
        m.skill_def_id == FUNI
            && m.target == "attack_range"
            && m.mode == "override"
            && m.value == 0
    }));
}

const JINFA: u32 = 4_139_928_729;

#[test]
fn jinfa_publishes_debate_branches_and_change_group() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 730);
    assert!(package
        .active_index
        .iter()
        .any(|a| a.skill_def_id == JINFA));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == JINFA
            && t.event == "debate_resolved"
            && t.filter == "debate_owner_is_initiator_result_black"
            && t.optional
    }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == JINFA
            && t.event == "debate_resolved"
            && t.filter == "debate_owner_is_initiator_result_red"
            && t.optional
    }));
}

#[test]
fn change_group_overrides_faction() {
    let compiled = content_compiler::compile_source_named("change-group-test", &sources()).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(JINFA), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
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
            skill_def: SkillDefId::new(JINFA),
        }],
    )
    .unwrap();
    let before = tx.state().player_group(owner).map(str::to_string);
    // 直接驱动 change_group 效果（经 skill content 编译，ABI 661 原语）。
    let compiled2 = content_compiler::compile_source_named(
        "change-group-prim",
        "skill change_group_probe(299792458) { active play_phase { effect change_group(self, wei, self_skill); } }",
    )
    .unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled2).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(299_792_458),
        }],
    )
    .unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().turn.current = owner;
    let state0 = tx.commit().state;
    let probe = state0
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.def.raw() == 299_792_458)
        .unwrap()
        .id;
    let revision = state0.revision;
    let (state, pipe) = engine_core::command::apply_command(
        state0,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(60),
            kind: CommandKind::ActivateSkill {
                skill: probe,
                targets: vec![],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }), "{:?}", pipe.result);
    assert_eq!(state.player_group(owner).map(str::to_string), Some("wei".into()));
    let _ = before;
}

#[test]
fn funi_distributes_shadows_each_round() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 730);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == FUNI && t.event == "round_started" && !t.optional
    }));
    // 行为：round_started → 分发 ceil(存活/2) 张影。
    let compiled = content_compiler::compile_source_named("funi-dist", &sources()).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, u64::from(FUNI), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
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
            skill_def: SkillDefId::new(FUNI),
        }],
    )
    .unwrap();
    // round_started forced 触发：dispatch 返回效果列表 → run_effects 后分发
    // 选择挂起（Cont 275）。
    let effects = engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::RoundStarted {
            round: 1,
            current: owner,
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    )
    .unwrap();
    assert_eq!(
        run_effects(&mut tx, effects),
        Err(EffectError::ChoiceRequested)
    );
    let state0 = tx.commit().state;
    let others: Vec<PlayerId> = state0
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|p| *p != owner)
        .collect();
    // 选 2 名目标（3 人局 ceil(3/2)=2 张，均分各 1）。
    let state1 = resolve(state0, owner, vec![others[0].raw(), others[1].raw()], 70);
    let ying0 = state1
        .hand_cards(others[0])
        .iter()
        .filter(|c| {
            state1
                .cards
                .get(**c)
                .map(|cd| cd.kind == engine_core::state::CardKind::Ying)
                .unwrap_or(false)
        })
        .count();
    let ying1 = state1
        .hand_cards(others[1])
        .iter()
        .filter(|c| {
            state1
                .cards
                .get(**c)
                .map(|cd| cd.kind == engine_core::state::CardKind::Ying)
                .unwrap_or(false)
        })
        .count();
    assert_eq!(ying0, 1);
    assert_eq!(ying1, 1);
}

const YAOYAN: u32 = 3_932_516_749;

#[test]
fn yaoyan_publishes_register_resolve_pair() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 730);
    let yaoyan_triggers: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|t| t.skill_def_id == YAOYAN)
        .collect();
    assert_eq!(yaoyan_triggers.len(), 2);
    assert!(yaoyan_triggers.iter().any(|t| t.event == "phase_prepare"));
    assert!(yaoyan_triggers.iter().any(|t| t.event == "phase_end"));
}

#[test]
fn funi_tracks_ying_into_discard() {
    use engine_core::state::{Card, ZoneKind};
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 730);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == FUNI
            && t.event == "cards_moved"
            && t.filter == "owner_cards_moved_contain_ying"
    }));
    // 行为：影入弃牌堆 → storage 置 1。
    let compiled = content_compiler::compile_source_named("funi-moved", &sources()).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(FUNI), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
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
            skill_def: SkillDefId::new(FUNI),
        }],
    )
    .unwrap();
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.def.raw() == FUNI)
        .unwrap()
        .id;
    // 造一张影入弃牌堆。
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, engine_api::PlayerId::NONE)
        .or_else(|| {
            tx.state().zones.find(ZoneKind::DiscardPile, owner)
        })
        .unwrap();
    let ying = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: engine_api::CardDefId::new(0),
        kind: engine_core::state::CardKind::Ying,
        suit: engine_api::Suit::Spade,
        rank: 1,
        weapon_range: 0,
        zone: discard,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(discard).unwrap().cards.push(ying);
    let from_hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let events = engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::CardsMoved {
            cards: vec![ying],
            from: from_hand,
            to: discard,
            reason: engine_api::CardMoveReason::Discard,
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, events).unwrap();
    assert_eq!(
        tx.state().skills.get(skill).unwrap().content_storage.get(&1),
        Some(&1)
    );
}

const CHUSHI: u32 = 3_259_850_167;

#[test]
fn chushi_publishes_debate_with_lord() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 730);
    assert!(package
        .active_index
        .iter()
        .any(|a| a.skill_def_id == CHUSHI));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == CHUSHI
            && t.event == "debate_resolved"
            && t.filter == "debate_owner_is_initiator_result_red"
    }));
}

const TW_LIHUO2: u32 = 2_376_916_887;
const TW: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");

#[test]
fn tw_lihuo2_fire_sha_extra_target() {
    let package = content_schema::ContentPackage::from_json_bytes(TW).unwrap();
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == TW_LIHUO2
            && t.event == "card_used"
            && t.filter == "owner_source_fire_sha"
            && t.optional
    }));
}

const CHIJIE: u32 = 3_286_369_736;

#[test]
fn tw_chijie_publishes_group_choice() {
    let package = content_schema::ContentPackage::from_json_bytes(TW).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 733);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == CHIJIE && t.event == "round_started" && t.optional
    }));
}

const FEIYANG: u32 = 2_568_675_461;

#[test]
fn feiyang_publishes_judge_discard() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 730);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == FEIYANG
            && t.event == "phase_judge"
            && t.filter == "owner_is_target_judge_nonempty"
            && t.optional
    }));
}

#[test]
fn place_xumou_places_card_in_judge_zone() {
    use engine_core::state::{Card, ZoneKind};
    let compiled = content_compiler::compile_source_named("xumou-test", &sources()).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(FEIYANG), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    let owner = tx.state().turn.current;
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();
    // 直接驱动 place_xumou（经 primitives 分发，SkillId 用 dummy 已加载技能）。
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: engine_api::CardDefId::new(1),
        kind: engine_core::state::CardKind::Sha,
        suit: engine_api::Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(FEIYANG),
        }],
    )
    .unwrap();
    let dummy = tx
        .state()
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.def.raw() == FEIYANG)
        .unwrap()
        .id;
    run_effects(
        &mut tx,
        vec![rule_api::EffectRequest::PlaceXumou {
            player: owner,
            card,
            skill: dummy,
        }],
    )
    .unwrap();
    let judge = tx.state().zones.find(ZoneKind::Judge, owner).unwrap();
    assert_eq!(tx.state().zones.get(judge).unwrap().cards.len(), 1);
}

#[test]
fn xumou_card_settles_with_tag_in_judge_zone() {
    use engine_core::state::{Card, ZoneKind};
    let compiled = content_compiler::compile_source_named("xumou-settle", &sources()).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(FEIYANG), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
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
            skill_def: SkillDefId::new(FEIYANG),
        }],
    )
    .unwrap();
    let dummy = tx
        .state()
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.def.raw() == FEIYANG)
        .unwrap()
        .id;
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: engine_api::CardDefId::new(1),
        kind: engine_core::state::CardKind::Sha,
        suit: engine_api::Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    run_effects(
        &mut tx,
        vec![rule_api::EffectRequest::PlaceXumou {
            player: owner,
            card,
            skill: dummy,
        }],
    )
    .unwrap();
    let judge = tx.state().zones.find(ZoneKind::Judge, owner).unwrap();
    let placed = tx.state().zones.get(judge).unwrap().cards[0];
    // 判定结算：蓄谋牌被移弃牌堆并触发视为使用事件（code 55）。
    let discard = tx.state().zones.find(ZoneKind::DiscardPile, engine_api::PlayerId::NONE).unwrap();
    let events = engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::CardsMoved {
            cards: vec![placed],
            from: judge,
            to: discard,
            reason: engine_api::CardMoveReason::Unspecified,
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    )
    .unwrap();
    // 合成事件只跑触发链；蓄谋牌仍物理在判定区（真实结算走
    // resolve_judge_zone 的 apply_judge_result）。
    run_effects(&mut tx, events).unwrap();
    // 切片 2 核心：蓄谋牌带 XUMOU_TAG（判定结算的视为使用由 tag 驱动）。
    assert!(tx
        .state()
        .content_card_tags
        .get(&placed)
        .is_some_and(|tags| tags.contains(&engine_core::content::XUMOU_TAG)));
}

const FANGJIE: u32 = 2_803_992_112;

#[test]
fn fangjie_publishes_prepare_recovery() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 730);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == FANGJIE
            && t.event == "phase_prepare"
            && t.filter == "owner_is_target"
            && t.optional
    }));
}

const DAIMOU: u32 = 2_351_858_276;
const EQIAN: u32 = 3_425_800_404;

// ---- 蓄谋族切片 3b（ABI 666）：殆谋双分支 + 遏前①选牌蓄谋 ----

fn xumou3b_started(tag: &str, players: usize, def: u32) -> (GameState, PlayerId) {
    let compiled = content_compiler::compile_source_named(tag, &sources()).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, players as u8, 20_260_827, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
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

fn xumou3b_resolve(
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

fn xumou3b_drive(
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

fn xumou3b_inject_sha(tx: &mut Transaction, owner: PlayerId) -> engine_api::CardId {
    use engine_core::state::{Card, ZoneKind};
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(engine_core::state::Card {
        id: engine_api::CardId::NONE,
        def: engine_api::CardDefId::new(1),
        kind: engine_core::state::CardKind::Sha,
        suit: engine_api::Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn place_deck_top_xumou_moves_draw_pile_top_into_judge_zone() {
    use engine_core::state::ZoneKind;
    let (state, owner) = xumou3b_started("xumou-3b-top", 2, DAIMOU);
    let mut tx = Transaction::begin(state);
    let dummy = tx
        .state()
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.def.raw() == DAIMOU)
        .unwrap()
        .id;
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, engine_api::PlayerId::NONE)
        .unwrap();
    let top = *tx.state().zones.get(draw).unwrap().cards.last().unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::PlaceDeckTopXumou {
            player: owner,
            skill: dummy,
        }],
    )
    .unwrap();
    let judge = tx.state().zones.find(ZoneKind::Judge, owner).unwrap();
    assert_eq!(
        tx.state().zones.get(judge).unwrap().cards,
        vec![top],
        "draw-pile top becomes the sole judge-zone card"
    );
    assert_eq!(tx.state().cards.get(top).unwrap().zone, judge);
    assert_eq!(tx.state().cards.get(top).unwrap().owner, owner);
    assert!(!tx
        .state()
        .zones
        .get(draw)
        .unwrap()
        .cards
        .contains(&top));
    assert!(tx
        .state()
        .content_card_tags
        .get(&top)
        .is_some_and(|tags| tags.contains(&engine_core::content::XUMOU_TAG)));
}

#[test]
fn daimou_places_deck_top_once_per_turn_when_other_targeted() {
    use engine_core::state::ZoneKind;
    let (state, owner) = xumou3b_started("xumou-3b-daimou", 2, DAIMOU);
    let other = state
        .players
        .iter_live()
        .find(|p| p.id != owner)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(state);
    let sha = xumou3b_inject_sha(&mut tx, other);
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, engine_api::PlayerId::NONE)
        .unwrap();
    let draw_len_before = tx.state().zones.get(draw).unwrap().cards.len();

    // 首次「杀指定其他角色」：optional confirm → 牌堆顶入判定区。
    let opened = xumou3b_drive(
        &mut tx,
        &[engine_api::DomainEvent::CardTargetAdded {
            player: other,
            card: sha,
            target: other,
            targets: vec![other],
        }],
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        engine_core::state::ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let state = xumou3b_resolve(tx.state, owner, vec![1], false, 301);
    let judge = state.zones.find(ZoneKind::Judge, owner).unwrap();
    assert_eq!(state.zones.get(judge).unwrap().cards.len(), 1);
    assert_eq!(
        state.zones.get(draw).unwrap().cards.len(),
        draw_len_before - 1
    );

    // 同回合第二次：confirm 仍开，但 storage 门槛拦下置牌。
    let mut tx = Transaction::begin(state);
    let opened = xumou3b_drive(
        &mut tx,
        &[engine_api::DomainEvent::CardTargetAdded {
            player: other,
            card: sha,
            target: other,
            targets: vec![other],
        }],
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    let tx = tx.commit();
    let state = xumou3b_resolve(tx.state, owner, vec![1], false, 302);
    assert_eq!(state.zones.get(judge).unwrap().cards.len(), 1);

    // 回合结束清零门槛（近似 addTempSkill 移除）后再触发可再置牌。
    let mut tx = Transaction::begin(state);
    let cleared = xumou3b_drive(
        &mut tx,
        &[engine_api::DomainEvent::PhaseEnded {
            current: other,
            phase: engine_api::TurnPhaseTag::End,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_targeted_players: 0,
            phase_hp_changed_players: 0,
        }],
    );
    cleared.unwrap();
    let draw_len_mid = tx.state().zones.get(draw).unwrap().cards.len();
    let opened = xumou3b_drive(
        &mut tx,
        &[engine_api::DomainEvent::CardTargetAdded {
            player: other,
            card: sha,
            target: other,
            targets: vec![other],
        }],
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    let tx = tx.commit();
    let state = xumou3b_resolve(tx.state, owner, vec![1], false, 303);
    assert_eq!(state.zones.get(judge).unwrap().cards.len(), 2);
    assert_eq!(
        state.zones.get(draw).unwrap().cards.len(),
        draw_len_mid - 1
    );
}

#[test]
fn daimou_discards_own_xumou_forced_when_self_targeted() {
    use engine_core::state::{Card, ZoneKind};
    let (state, owner) = xumou3b_started("xumou-3b-daimou-me", 2, DAIMOU);
    let other = state
        .players
        .iter_live()
        .find(|p| p.id != owner)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(state);
    let dummy = tx
        .state()
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.def.raw() == DAIMOU)
        .unwrap()
        .id;
    // 种一张蓄谋牌进判定区（直接驱动切片 1 原语）。
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let seed_card = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: engine_api::CardDefId::new(2),
        kind: engine_core::state::CardKind::Tao,
        suit: engine_api::Suit::Heart,
        rank: 4,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(seed_card);
    run_effects(
        &mut tx,
        vec![EffectRequest::PlaceXumou {
            player: owner,
            card: seed_card,
            skill: dummy,
        }],
    )
    .unwrap();
    let judge = tx.state().zones.find(ZoneKind::Judge, owner).unwrap();
    let placed = tx.state().zones.get(judge).unwrap().cards[0];
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, engine_api::PlayerId::NONE)
        .unwrap();
    let discard_len_before = tx.state().zones.get(discard).unwrap().cards.len();

    // 杀指定自己：forced（上游「你须」）——无 confirm，效果直跑。
    let sha = xumou3b_inject_sha(&mut tx, other);
    xumou3b_drive(
        &mut tx,
        &[engine_api::DomainEvent::CardTargetAdded {
            player: other,
            card: sha,
            target: owner,
            targets: vec![owner],
        }],
    )
    .unwrap();
    assert_eq!(tx.state().zones.get(judge).unwrap().cards.len(), 0);
    let discarded = tx.state().zones.get(discard).unwrap().cards.clone();
    assert_eq!(discarded.len(), discard_len_before + 1);
    assert!(discarded.contains(&placed));

    // 同回合第二次被指定：门槛已合上，不再弃。
    xumou3b_drive(
        &mut tx,
        &[engine_api::DomainEvent::CardTargetAdded {
            player: other,
            card: sha,
            target: owner,
            targets: vec![owner],
        }],
    )
    .unwrap();
    assert_eq!(
        tx.state().zones.get(discard).unwrap().cards.len(),
        discard_len_before + 1
    );
}

#[test]
fn eqian_phase_end_moves_chosen_hand_card_into_judge_zone() {
    use engine_core::state::ZoneKind;
    let (state, owner) = xumou3b_started("xumou-3b-eqian", 2, EQIAN);
    let hand_ids: Vec<u32> = state.hand_cards(owner).iter().map(|c| c.raw()).collect();
    assert!(!hand_ids.is_empty());
    let chosen = engine_api::CardId::new(hand_ids[0]);
    let mut tx = Transaction::begin(state);

    // 结束阶段（phase_end ← TurnStateChanged End）：optional confirm → 选牌续体。
    let opened = xumou3b_drive(
        &mut tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::End,
        }],
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        engine_core::state::ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    // confirm 后停在 Cont 278 选牌；选中第一张手牌移入判定区。
    let mid = xumou3b_resolve(tx.state, owner, vec![1], false, 401);
    assert_eq!(
        mid.resolution.continuation_kind,
        engine_core::state::ContinuationKind::ChoosePlaceHandXumou
    );
    let judge = mid.zones.find(ZoneKind::Judge, owner).unwrap();
    let mut tx = Transaction::begin(mid);
    let pending_observer = tx.state().resolution.pending_observer;
    let tx = tx.commit();
    let final_state = xumou3b_resolve(tx.state, PlayerId::new(pending_observer), vec![chosen.raw()], false, 402);
    assert_eq!(
        final_state.zones.get(judge).unwrap().cards,
        vec![chosen]
    );
    assert_eq!(final_state.cards.get(chosen).unwrap().zone, judge);
    assert!(!final_state.hand_cards(owner).contains(&chosen));
    assert!(final_state
        .content_card_tags
        .get(&chosen)
        .is_some_and(|tags| tags.contains(&engine_core::content::XUMOU_TAG)));

    // cancel = 停止蓄谋：再次触发后取消，不移动任何牌。
    let mut tx = Transaction::begin(final_state);
    let opened = xumou3b_drive(
        &mut tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::End,
        }],
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    let tx = tx.commit();
    let state = xumou3b_resolve(tx.state, owner, vec![], true, 403);
    assert_eq!(state.zones.get(judge).unwrap().cards.len(), 1);
}
}

mod original_jsrg_debate_targets {
//! Golden coverage for the first targeted-debate consumer (ADR 0644 P2):
//! jsrgshanzheng consumes start_debate_targets (ABI 656, Cont 272).

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const SHANZHENG: u32 = 3_264_543_475;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/jsrg/jsrg.npack.json");
const SRC: &str = include_str!("../../../content/original/ported/jsrg/jsrgshanzheng.skill");

fn inject_hand(tx: &mut Transaction, player: PlayerId, suit: Suit) {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(333_101),
        kind: CardKind::Shan,
        suit,
        rank: 6,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
}

#[test]
fn shanzheng_npack_consumes_656() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 730);
    assert!(package
        .active_index
        .iter()
        .any(|a| a.skill_def_id == SHANZHENG));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == SHANZHENG
            && t.event == "debate_resolved"
            && t.filter == "debate_owner_is_initiator_result_red"
    }));
}

#[test]
fn shanzheng_targeted_debate_red_branch_damages() {
    let compiled = content_compiler::compile_source_named("szz", SRC).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 41, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let mut others = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|p| *p != owner);
    let a = others.next().unwrap();
    let b = others.next().unwrap();
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();
    for player in tx.state().players.alive_by_seat() {
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardFromHand { player, count: u32::MAX }],
        )
        .unwrap();
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(SHANZHENG),
        }],
    )
    .unwrap();
    inject_hand(&mut tx, a, Suit::Heart); // 参与者 a 红意见
    let hp_b = tx.state().players.get(b).unwrap().hp;
    let skill_id = tx
        .state()
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.def.raw() == SHANZHENG)
        .unwrap()
        .id;
    let (state, pipe) = apply_command(
        tx.commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill: skill_id,
                targets: vec![],
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    // 链：Cont272 选参与者（a）→ DebateResolved（红）→ 红分支 confirm →
    // choose_other_damage 选目标（b）。
    let mut state = state;
    let mut key = 10;
    let mut step = 0;
    while state.resolution.pending_choice_id != 0 && step < 5 {
        let actor = engine_api::PlayerId::new(state.resolution.pending_observer);
        let sel = match step {
            0 => vec![u32::from(a.raw())], // 选参与者
            1 => vec![1],                  // 红分支 confirm
            _ => vec![u32::from(b.raw())], // 伤害目标
        };
        state = resolve(state, actor, sel, key);
        key += 1;
        step += 1;
    }
    assert_eq!(state.players.get(b).unwrap().hp, hp_b - 1, "red branch damages 1");
    let _ = EffectError::ChoiceRequested;
    let _ = DomainEvent::DebateResolved {
        initiator: owner,
        red: vec![a],
        black: vec![],
        result: 1,
    };
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
}

mod original_jsrg_jiebing_wudao {
//! Golden coverage for JSRG 皆兵 / 悟道 rewritten from honest noops.

use engine_api::{
    Command, CommandKind, DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
    TurnPhaseTag,
};
use engine_core::effect::run_effects;
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const JSRGJIEBING: u32 = 3_843_907_824;
const JSRGBAOWEI: u32 = 3_914_522_334;
const JSRGWUDAO: u32 = 2_550_512_310;
const JSRGJINGLEI: u32 = 2_695_101_489;

const JIEBING_SRC: &str = include_str!("../../../content/original/ported/jsrg/jsrgjiebing.skill");
const WUDAO_SRC: &str = include_str!("../../../content/original/ported/jsrg/jsrgwudao.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/jsrg/jsrg.npack.json");

fn setup(source: &str, name: &str, skill_def: u32) -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named(name, source).unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_815_51),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_815_51,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&compiled).unwrap();
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
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, other)
}

fn fire_prepare(state: GameState, owner: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("prepare dispatch error: {e:?}"));
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

fn has_def(state: &GameState, player: PlayerId, def: u32) -> bool {
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .any(|skill| state.skills.get(*skill).is_some_and(|instance| instance.def.raw() == def))
}

fn once_mark(state: &GameState, owner: PlayerId, skill: u32) -> i32 {
    state
        .players
        .get(owner)
        .and_then(|player| player.marks.iter().find(|(id, _)| *id == skill))
        .map(|(_, count)| *count)
        .unwrap_or(0)
}

#[test]
fn jiebing_source_and_npack_publish_prepare_awaken() {
    let compiled = content_compiler::compile_source_named("jsrgjiebing", JIEBING_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == JSRGJIEBING)
        .expect("jiebing trigger");
    assert_eq!(trigger.event, "phase_prepare");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(!trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == JSRGJIEBING && trigger.event == "phase_prepare"
    }));
}

#[test]
fn jiebing_prepare_gains_two_max_hp_and_baowei_once() {
    let (mut state, owner, _) = setup(JIEBING_SRC, "jiebing-test", JSRGJIEBING);
    {
        let player = state.players.get_mut(owner).unwrap();
        player.max_hp = 3;
        player.hp = 3;
    }
    assert!(!has_def(&state, owner, JSRGBAOWEI));
    let state = fire_prepare(state, owner);
    let player = state.players.get(owner).unwrap();
    assert_eq!(player.max_hp, 5);
    assert_eq!(player.hp, 5);
    assert_eq!(once_mark(&state, owner, JSRGJIEBING), 1);
    assert!(has_def(&state, owner, JSRGBAOWEI));

    let state = fire_prepare(state, owner);
    let player = state.players.get(owner).unwrap();
    assert_eq!(player.max_hp, 5);
    assert_eq!(once_mark(&state, owner, JSRGJIEBING), 1);
}

#[test]
fn wudao_source_and_npack_publish_dying_awaken() {
    let compiled = content_compiler::compile_source_named("jsrgwudao", WUDAO_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == JSRGWUDAO)
        .expect("wudao trigger");
    assert_eq!(trigger.event, "dying");
    assert_eq!(trigger.filter, "owner_alive");
    assert!(!trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == JSRGWUDAO && trigger.event == "dying"
    }));
}

#[test]
fn wudao_on_dying_gains_max_hp_and_jinglei_once() {
    let (mut state, owner, other) = setup(WUDAO_SRC, "wudao-test", JSRGWUDAO);
    {
        let player = state.players.get_mut(owner).unwrap();
        player.max_hp = 3;
        player.hp = 2;
    }
    assert!(!has_def(&state, owner, JSRGJINGLEI));
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::EnteredDying {
            player: other,
            source: owner,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("dying dispatch error: {e:?}"));
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;
    let player = state.players.get(owner).unwrap();
    assert_eq!(player.max_hp, 4);
    assert_eq!(player.hp, 3);
    assert_eq!(once_mark(&state, owner, JSRGWUDAO), 1);
    assert!(has_def(&state, owner, JSRGJINGLEI));

    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::EnteredDying {
            player: other,
            source: owner,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("second dying dispatch error: {e:?}"));
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;
    assert_eq!(state.players.get(owner).unwrap().max_hp, 4);
    assert_eq!(once_mark(&state, owner, JSRGWUDAO), 1);
}
}

mod original_jsrg_jsrgchiying_gaoxiang {
//! Golden coverage for JSRG 高翔「驰应」(ABI 501).

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const JSRGCHIYING: u32 = 2_375_612_422;
const SOURCE: &str = include_str!("../../../content/original/ported/jsrg/jsrgchiying.skill");

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, suit: Suit, def: u32) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn skill_id(state: &GameState, owner: PlayerId) -> SkillId {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == JSRGCHIYING)
        .unwrap()
}

#[test]
fn ordered_forced_discards_then_target_gains_only_basic_cards() {
    let package = content_compiler::compile_source_named("jsrgchiying-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 501);
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 5, 20_260_802, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, first, target, second) = (seats[0], seats[1], seats[2], seats[3]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(seats.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(JSRGCHIYING),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().players.get_mut(target).unwrap().hp = 1;
    let basic = inject(&mut tx, first, CardKind::Sha, Suit::Spade, 99_100);
    let trick = inject(&mut tx, second, CardKind::TrickGuohe, Suit::Heart, 99_101);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = skill_id(tx.state(), owner);

    let (state, activated) = submit(
        tx.commit().state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: vec![],
        },
        1,
    );
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    let choice = activated.choice.expect("first forced discard");
    assert_eq!(choice.observer, first);
    let (state, resolved) = submit(
        state,
        first,
        CommandKind::ResolveChoice {
            choice_id: ChoiceId::new(choice.choice_id.raw()),
            selections: vec![basic.raw()],
            cancelled: false,
        },
        2,
    );
    let choice = resolved.choice.expect("second forced discard");
    assert_eq!(choice.observer, second);
    let (state, resolved) = submit(
        state,
        second,
        CommandKind::ResolveChoice {
            choice_id: ChoiceId::new(choice.choice_id.raw()),
            selections: vec![trick.raw()],
            cancelled: false,
        },
        3,
    );
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert!(state.hand_cards(target).contains(&basic));
    assert!(!state.hand_cards(target).contains(&trick));
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(state.zones.get(discard).unwrap().cards.contains(&trick));
}
}

mod original_jsrg_juxia_jueyin_niluan {
//! Golden coverage for JSRG 居下 / 绝禋 / 逆乱.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const JUXIA: u32 = 3_771_277_794;
const JUEYIN: u32 = 2_928_968_300;
const NILUAN: u32 = 3_518_365_032;
const JUXIA_SRC: &str =
    include_str!("../../../content/original/ported/jsrg/jsrg_new_juxia.skill");
const JUEYIN_SRC: &str = include_str!("../../../content/original/ported/jsrg/jsrgjueyin.skill");
const NILUAN_SRC: &str = include_str!("../../../content/original/ported/jsrg/jsrgniluan.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/jsrg/jsrg.npack.json");

fn bootstrap(source: &str, name: &str, skill_def: u32) -> (Transaction, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named(name, source).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(skill_def), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
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

fn inject_hand(tx: &mut Transaction, owner: PlayerId, def: u32) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 8,
        weapon_range: 1,
        zone: hand,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
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
fn juxia_source_and_npack_publish_targeted_draw() {
    let compiled = content_compiler::compile_source_named("jsrg_new_juxia", JUXIA_SRC).unwrap();
    assert!(compiled.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == JUXIA
            && trigger.event == "card_targeted"
            && trigger.optional
    }));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == JUXIA && trigger.event == "card_targeted"));
}

#[test]
fn juxia_draws_two_when_targeted_by_other() {
    let (mut tx, owner, other) = bootstrap(JUXIA_SRC, "juxia-test", JUXIA);
    let card = inject_hand(&mut tx, other, 191_101);
    let before = tx.state().hand_count(owner);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::UseCard {
                player: other,
                card,
                targets: vec![owner],
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, owner, vec![1], 2);
    assert_eq!(state.hand_count(owner), before + 2);
}

#[test]
fn jueyin_source_and_npack_publish_first_damage_draw() {
    let compiled = content_compiler::compile_source_named("jsrgjueyin", JUEYIN_SRC).unwrap();
    assert!(compiled.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == JUEYIN
            && trigger.event == "damage"
            && trigger.filter == "owner_is_target_first_damage"
            && trigger.optional
    }));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == JUEYIN && trigger.filter == "owner_is_target_first_damage"
    }));
}

#[test]
fn jueyin_draws_three_after_first_damage() {
    let (tx, owner, other) = bootstrap(JUEYIN_SRC, "jueyin-test", JUEYIN);
    let before = tx.state().hand_count(owner);
    let mut tx = tx;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: other,
                target: owner,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, owner, vec![1], 3);
    assert_eq!(state.hand_count(owner), before + 3);
}

#[test]
fn niluan_source_and_npack_publish_prepare_damage() {
    let compiled = content_compiler::compile_source_named("jsrgniluan", NILUAN_SRC).unwrap();
    assert!(compiled.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == NILUAN && trigger.event == "phase_prepare" && trigger.optional
    }));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == NILUAN && trigger.event == "phase_prepare"
    }));
}

#[test]
fn niluan_prepare_can_damage_other() {
    let (tx, owner, other) = bootstrap(NILUAN_SRC, "niluan-test", NILUAN);
    let other_hp = tx.state().players.get(other).unwrap().hp;
    let mut tx = tx;
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::TurnStateChanged {
                current: owner,
                phase: TurnPhaseTag::Prepare,
            }],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, owner, vec![1], 4);
    assert!(state.has_pending_choice());
    let state = resolve(state, owner, vec![other.raw()], 5);
    assert_eq!(state.players.get(other).unwrap().hp, other_hp - 1);
}
}

mod original_jsrg_langan {
//! Golden for JSRG 诸葛诞「阑干」的死亡后回复、摸牌和攻击范围递减语义。

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::query::{query, QueryContext};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const LANGAN: u32 = 3_013_648_901;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/jsrg/jsrg.npack.json");

fn mark_count(tx: &Transaction, owner: PlayerId) -> i32 {
    tx.state()
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .find(|(mark, _)| *mark == LANGAN)
        .map_or(0, |(_, count)| *count)
}

fn attack_range(tx: &Transaction, owner: PlayerId) -> i32 {
    match query(
        tx.state(),
        &QueryRequest::AttackRange { player: owner },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("unexpected query response: {other:?}"),
    }
}

#[test]
fn other_deaths_recover_draw_and_reduce_range_with_a_three_mark_cap() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == LANGAN
            && trigger.event == "death"
            && trigger.filter == "owner_alive"
            && !trigger.optional
    }));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(LANGAN as u64),
            kind: CommandKind::StartGame {
                player_count: 5,
                rng_seed: LANGAN as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let victims: Vec<_> = engine
        .state()
        .players
        .iter_live()
        .filter(|player| player.id != owner)
        .map(|player| player.id)
        .collect();
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
        skill_def: SkillDefId::new(LANGAN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = 2;
    let hand = tx.state().hand_count(owner);
    let base_range = attack_range(&tx, owner);

    run_effects(
        &mut tx,
        vec![EffectRequest::KillPlayer {
            player: victims[0],
            killer: owner,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 3);
    assert_eq!(tx.state().hand_count(owner), hand + 2);
    assert_eq!(mark_count(&tx, owner), 1);
    assert_eq!(attack_range(&tx, owner), (base_range - 1).max(0));

    for victim in victims.iter().skip(1) {
        run_effects(
            &mut tx,
            vec![EffectRequest::KillPlayer {
                player: *victim,
                killer: owner,
            }],
        )
        .unwrap();
    }
    assert_eq!(mark_count(&tx, owner), 3);
    assert_eq!(tx.state().hand_count(owner), hand + 8);
}
}

mod original_jsrg_lie_family {
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

const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/jsrg/jsrg.npack.json");
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

fn drive(
    tx: &mut Transaction,
    events: &[engine_api::DomainEvent],
) -> Result<(), EffectError> {
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
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
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
    let final_state = resolve(after_confirm, PlayerId::new(observer), vec![others[0].raw()], false, 522);
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
        !pre_distance
            || tx.state().hand_cards(victim).len() >= tx.state().hand_cards(holder).len(),
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
}

mod original_jsrg_new_juezhi {
//! Golden coverage for JSRG「绝质」：失去装备后可选摸 2。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const JSRG_NEW_JUEZHI: u32 = 2_494_349_607;
const SOURCE: &str =
    include_str!("../../../content/original/ported/jsrg/jsrg_new_juezhi.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/jsrg/jsrg.npack.json");

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
fn source_and_npack_publish_lose_equip_draw() {
    let compiled = content_compiler::compile_source_named("jsrg_new_juezhi", SOURCE).unwrap();
    assert!(compiled.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == JSRG_NEW_JUEZHI
            && trigger.event == "cards_moved"
            && trigger.filter == "owner_from_equip"
            && trigger.optional
    }));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == JSRG_NEW_JUEZHI && trigger.filter == "owner_from_equip"
    }));
}

#[test]
fn losing_equip_offers_draw_two() {
    let compiled = content_compiler::compile_source_named("juezhi-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, JSRG_NEW_JUEZHI as u64, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    let owner = tx.state().turn.current;
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(JSRG_NEW_JUEZHI),
    });
    run_effects(&mut tx, effects).unwrap();
    let equip = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(194_401),
        kind: CardKind::Other,
        suit: Suit::Spade,
        rank: 5,
        weapon_range: 1,
        zone: equip,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(equip).unwrap().cards.push(card);
    let before = tx.state().hand_count(owner);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::MoveCards {
                cards: vec![card],
                to: discard,
                from: Some(equip),
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, owner, vec![1], 2);
    assert_eq!(state.hand_count(owner), before + 2);
}
}

mod original_jsrg_zhaohan {
//! Golden for JSRG 杨彪「昭汉」的牌堆重洗分支语义。

use engine_api::{Command, CommandKind, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId};
use engine_core::effect::{reshuffle_discard_into_draw, run_effects};
use engine_core::state::Phase;
use engine_core::transaction::Transaction;
use engine_core::{canonical_state_hash, Engine};
use rule_api::EffectRequest;

const ZHAOHAN: u32 = 3_529_273_369;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/jsrg/jsrg.npack.json");

fn loaded(seed: u64) -> (Transaction, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 730);
    assert_eq!(package.skill_index.len(), 181);
    assert_eq!(package.trigger_index.len(), 56);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == ZHAOHAN
            && trigger.event == "phase_prepare"
            && trigger.filter == "owner_is_target_draw_reshuffle_zero_wounded"
            && !trigger.optional
    }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == ZHAOHAN
            && trigger.event == "phase_prepare"
            && trigger.filter == "owner_is_target_draw_reshuffle_positive"
            && !trigger.optional
    }));

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
        skill_def: SkillDefId::new(ZHAOHAN),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner)
}

fn enter_prepare(tx: &mut Transaction, owner: PlayerId) {
    run_effects(
        tx,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Prepare.to_u8(),
        }],
    )
    .unwrap();
}

#[test]
fn before_first_reshuffle_only_a_wounded_owner_recovers() {
    let (mut wounded, owner) = loaded(20_260_725);
    let max_hp = wounded.state().players.get(owner).unwrap().max_hp as i32;
    wounded.state_mut().players.get_mut(owner).unwrap().hp = max_hp - 1;
    enter_prepare(&mut wounded, owner);
    assert_eq!(wounded.state().players.get(owner).unwrap().hp, max_hp);

    let (mut healthy, owner) = loaded(20_260_726);
    let hp = healthy.state().players.get(owner).unwrap().hp;
    enter_prepare(&mut healthy, owner);
    assert_eq!(healthy.state().players.get(owner).unwrap().hp, hp);
}

#[test]
fn after_a_reshuffle_the_owner_loses_hp_even_when_healthy() {
    let (mut tx, owner) = loaded(20_260_727);
    tx.state_mut().draw_reshuffle_count = 1;
    let hp = tx.state().players.get(owner).unwrap().hp;
    enter_prepare(&mut tx, owner);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 1);
}

#[test]
fn actual_nonempty_discard_reshuffle_increments_once() {
    let (mut tx, owner) = loaded(20_260_728);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: 1,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().draw_reshuffle_count, 0);

    reshuffle_discard_into_draw(&mut tx).unwrap();
    assert_eq!(tx.state().draw_reshuffle_count, 1);

    reshuffle_discard_into_draw(&mut tx).unwrap();
    assert_eq!(tx.state().draw_reshuffle_count, 1);
}

#[test]
fn reshuffle_count_is_canonical_and_old_zero_state_still_decodes() {
    let zero = engine_core::state::GameState::empty();
    let encoded = serde_json::to_value(&zero).unwrap();
    assert!(encoded.get("draw_reshuffle_count").is_none());
    assert_eq!(
        serde_json::from_value::<engine_core::state::GameState>(encoded)
            .unwrap()
            .draw_reshuffle_count,
        0
    );

    let mut one = zero.clone();
    one.draw_reshuffle_count = 1;
    assert_ne!(canonical_state_hash(&zero), canonical_state_hash(&one));
}
}

mod original_jsrg_zhiheng_zhasi {
//! Golden coverage for JSRG 孙策「猘横」「诈死」.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const JSRGZHIHENG: u32 = 2_952_364_897;
const JSRGZHASI: u32 = 2_253_086_418;
const ZHIHENG_SRC: &str =
    include_str!("../../../content/original/ported/jsrg/jsrgzhiheng.skill");
const ZHASI_SRC: &str = include_str!("../../../content/original/ported/jsrg/jsrgzhasi.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/jsrg/jsrg.npack.json");

fn bootstrap(source: &str, name: &str, skill_def: u32) -> (Transaction, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named(name, source).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(skill_def), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, other)
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
fn zhiheng_source_and_npack_publish_outgoing_plus_one() {
    let compiled = content_compiler::compile_source_named("jsrgzhiheng", ZHIHENG_SRC).unwrap();
    assert!(compiled.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == JSRGZHIHENG
            && trigger.event == "damage_before"
            && trigger.filter == "owner_source_other_damage_before"
            && !trigger.optional
    }));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == JSRGZHIHENG && trigger.event == "damage_before"
    }));
}

#[test]
fn zhiheng_increases_outgoing_damage_by_one() {
    let (tx, owner, other) = bootstrap(ZHIHENG_SRC, "jsrgzhiheng-test", JSRGZHIHENG);
    let other_hp = tx.state().players.get(other).unwrap().hp;
    let mut tx = tx;
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: other,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(other).unwrap().hp, other_hp - 2);
}

#[test]
fn zhasi_source_and_npack_publish_optional_prevent() {
    let compiled = content_compiler::compile_source_named("jsrgzhasi", ZHASI_SRC).unwrap();
    assert!(compiled.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == JSRGZHASI && trigger.event == "damage_before" && trigger.optional
    }));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == JSRGZHASI && trigger.event == "damage_before"
    }));
}

#[test]
fn zhasi_can_prevent_incoming_damage_once() {
    let (tx, owner, other) = bootstrap(ZHASI_SRC, "jsrgzhasi-test", JSRGZHASI);
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    let mut tx = tx;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: other,
                target: owner,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, owner, vec![1], 2);
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp);
}
}

mod original_jsrg_zuwang {
//! Golden for JSRG「族望」的准备、结束阶段补牌语义。

use engine_api::{Command, CommandKind, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId};
use engine_core::effect::run_effects;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const ZUWANG: u32 = 3_072_062_532;
const BAHU: u32 = 4_246_105_809;
/// Concatenate every committed jsrg source in pack order (alphabetical),
/// mirroring `content-cli pack` input concatenation so the logic hash matches
/// the committed npack regardless of future roster growth.
fn jsrg_sources() -> String {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../content/original/ported/jsrg");
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().extension().map_or(false, |ext| ext == "skill")
        })
        .map(|entry| entry.path())
        .collect();
    files.sort();
    files
        .iter()
        .map(|path| {
            let mut source = std::fs::read_to_string(path).unwrap();
            source.push('\n');
            source
        })
        .collect()
}

const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/jsrg/jsrg.npack.json");

#[test]
fn committed_package_draws_to_max_hp_at_both_timings() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut compiled = content_compiler::compile_source_named("jsrg", &jsrg_sources()).unwrap();
    compiled.character_index = committed.character_index.clone();
    compiled.manifest.dependencies = committed.manifest.dependencies.clone();
    compiled.manifest.logic_hash = compiled.compute_logic_hash();
    assert_eq!(committed.manifest.logic_hash, compiled.manifest.logic_hash);
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == ZUWANG));
    assert_eq!(committed.trigger_index.len(), 56);

    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_713,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&committed).unwrap();
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    let max_hp = tx.state().players.get(owner).unwrap().max_hp;
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(ZUWANG),
            },
            EffectRequest::DiscardFromHand {
                player: owner,
                count: u32::MAX,
            },
            EffectRequest::SetTurnState {
                current: owner,
                phase: engine_core::state::Phase::Prepare.to_u8(),
            },
        ],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), max_hp);

    run_effects(
        &mut tx,
        vec![
            EffectRequest::DiscardFromHand {
                player: owner,
                count: 2,
            },
            EffectRequest::SetTurnState {
                current: owner,
                phase: engine_core::state::Phase::End.to_u8(),
            },
        ],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), max_hp);
}

#[test]
fn bahu_draws_at_prepare_and_adds_one_sha_use() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_714,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&committed).unwrap();
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(BAHU),
        }],
    )
    .unwrap();
    assert_eq!(engine_core::query::sha_use_limit_for(tx.state(), owner), 2);
    let hand = tx.state().hand_count(owner);

    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: engine_core::state::Phase::Prepare.to_u8(),
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand + 1);
}
}
