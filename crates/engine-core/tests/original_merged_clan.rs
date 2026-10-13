//! O2 全量合并批次 1（2026-10-14）：家族 clan 的 12 个独立测试文件逐字包裹为模块——#[test] 函数名不变、测试数量不变；测试全名获得确定性前缀（模块名=原文件名，子串过滤行为不变）；include_bytes/str 相对路径不受影响（同目录）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_clan_clan_family_batch {
//! 2026-08-15 conversion/clan-family batch: 保族 (clanbaozu) + 戒宣 (clanjiexuan).
//! clanbaozu uses the new parameterized filter `owner_alive_event_player_self_or_clan_<clan>`
//! (string filter — no ABI bump); clanjiexuan ships the dual view_as (red→shunshou /
//! black→guohe). Approximations are documented in the .skill headers.

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::query::{query, QueryContext};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const CLANBAOZU: u32 = 3_600_751_244;
const CLANJIEXUAN: u32 = 2_854_725_517;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/clan/clan.npack.json");

fn start() -> (GameState, PlayerId) {
    let mut engine = Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.load_package(&pkg).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_816,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    (engine.state().clone(), owner)
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
    engine_core::effect::run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

fn mark_two(state: &GameState, player: PlayerId) -> i32 {
    match query(
        state,
        &QueryRequest::MarkCount { player, mark: 2 },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(n) => n,
        other => panic!("unexpected mark query response: {other:?}"),
    }
}

#[test]
fn committed_package_publishes_clan_family_contract() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let baozu = pkg
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == CLANBAOZU)
        .expect("clanbaozu trigger");
    assert_eq!(baozu.event, "dying");
    assert_eq!(
        baozu.filter,
        "owner_alive_event_player_self_or_clan_颍川钟氏"
    );
    assert!(baozu.optional);
    let mut view_as: Vec<_> = pkg
        .view_as_index
        .iter()
        .filter(|view| view.skill_def_id == CLANJIEXUAN)
        .map(|view| (view.as_kind.as_str(), view.filter.as_str()))
        .collect();
    view_as.sort();
    assert_eq!(
        view_as,
        vec![("guohe", "black"), ("shunshou", "red")],
        "jiexuan dual view_as"
    );
}

#[test]
fn baozu_confirms_then_links_and_recovers_the_dying_owner() {
    let (state, owner) = start();
    let mut state = with_skill(state, owner, CLANBAOZU);
    // 濒死：hp<=0（过滤器 hp 门槛由濒死事件本身保证），未横置。
    state.players.get_mut(owner).unwrap().hp = 0;
    let mut tx = Transaction::begin(state);
    let result = dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::EnteredDying {
            player: owner,
            source: PlayerId::NONE,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(
        result.unwrap_err(),
        engine_core::effect::EffectError::ChoiceRequested
    );
    assert_eq!(tx.state().resolution.pending_observer, owner.raw());
    let mut state = tx.commit().state;

    // 确认 → 横置（MARK_CHAIN=2）+ 回复 1。
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (state, output) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: choice,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(mark_two(&state, owner), 1, "dying player linked");
    assert_eq!(state.players.get(owner).unwrap().hp, 1, "recovered 1");
    let _ = CardId::NONE;
}

#[test]
fn clanshize_first_response_in_own_turn_buffs_attack_range_then_damage_removes() {
    let CLANSHIZE: u32 = 3_304_769_267;
    let (state, owner) = start();
    let mut state = with_skill(state, owner, CLANSHIZE);
    let other = state
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let range_before = match query(
        &state,
        &QueryRequest::AttackRange { player: owner },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(n) => n,
        other => panic!("range query: {other:?}"),
    };
    // 本回合内其他角色响应 owner 的牌 → 可选确认。
    let mut tx = Transaction::begin(state);
    let result = dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::CardResponded {
            player: other,
            respond_to: owner,
            card: CardId::NONE,
            response_kind: CardKind::Shan.tag(),
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(
        result.unwrap_err(),
        engine_core::effect::EffectError::ChoiceRequested
    );
    let mut state = tx.commit().state;
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (state, output) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: choice,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    let range_buffed = match query(
        &state,
        &QueryRequest::AttackRange { player: owner },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(n) => n,
        other => panic!("range query: {other:?}"),
    };
    assert_eq!(range_buffed, range_before + 1, "attack range +1 after confirm");

    // 同回合第二次响应：storage 门槛拦下效果（确认仍弹，拒绝后范围不变）。
    let mut tx = Transaction::begin(state);
    let result = dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::CardResponded {
            player: other,
            respond_to: owner,
            card: CardId::NONE,
            response_kind: CardKind::Shan.tag(),
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(
        result.unwrap_err(),
        engine_core::effect::EffectError::ChoiceRequested
    );
    let mut state = tx.commit().state;
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (state, output) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::ResolveChoice {
                choice_id: choice,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    let range_still = match query(
        &state,
        &QueryRequest::AttackRange { player: owner },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(n) => n,
        other => panic!("range query: {other:?}"),
    };
    assert_eq!(range_still, range_before + 1, "once-per-turn gate holds");

    // 受伤 → 增益失去。
    let mut tx = Transaction::begin(state);
    engine_core::effect::run_effects(
        &mut tx,
        vec![EffectRequest::CommitDamage {
            damage_id: 0,
            source: other,
            target: owner,
            amount: 1,
            base_amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
            card: engine_api::CardId::NONE,
            card_kind: 0,
            card_flags: 0,
        }],
    )
    .unwrap();
    let state = tx.commit().state;
    let range_after = match query(
        &state,
        &QueryRequest::AttackRange { player: owner },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(n) => n,
        other => panic!("range query: {other:?}"),
    };
    assert_eq!(range_after, range_before, "buff lost after damage");
}

#[test]
fn muyin_permanently_raises_a_not_field_max_targets_hand_limit() {
    let CLANMUYIN: u32 = 2_666_909_205;
    let (state, owner) = start();
    let mut state = with_skill(state, owner, CLANMUYIN);
    let other = state
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    // 拉平双方上限基础，确保 owner 非全场最高（平局不算最高——上游 isMax 要求无严格更高者）。
    let owner_limit = match query(
        &state,
        &QueryRequest::HandLimit { player: owner },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(n) => n,
        o => panic!("{o:?}"),
    };
    let _ = other;

    // 准备阶段触发（owner 自己的回合开始）→ optional 确认。
    let mut tx = Transaction::begin(state);
    let result = dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(
        result.unwrap_err(),
        engine_core::effect::EffectError::ChoiceRequested
    );
    let mut state = tx.commit().state;
    // 确认 → SelectTargets（自己为合法候选）→ 选自己。
    let mut key = 10u64;
    let mut selects = vec![1u32];
    for _ in 0..4 {
        let choice = ChoiceId::new(state.resolution.pending_choice_id);
        let revision = state.revision;
        let (next, output) = apply_command(
            state,
            Command {
                actor: owner,
                expected_revision: ExpectedRevision(revision),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id: choice,
                    selections: selects.clone(),
                    cancelled: false,
                },
            },
        );
        assert!(
            matches!(output.result, CommandResult::Accepted { .. }),
            "resolve rejected: {:?}",
            output.result
        );
        state = next;
        key += 1;
        if !state.has_pending_choice() {
            break;
        }
        selects = vec![owner.raw()]; // 下一阶段：目标选择
    }
    assert!(!state.has_pending_choice(), "choice chain settled");
    let owner_limit_after = match query(
        &state,
        &QueryRequest::HandLimit { player: owner },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(n) => n,
        o => panic!("{o:?}"),
    };
    assert_eq!(owner_limit_after, owner_limit + 1, "persistent +1 hand limit");
    // 契约：包以 ABI 648 发布本原语。
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(pkg.manifest.rule_vm_abi_version, 649);
    assert!(pkg.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == CLANMUYIN && trigger.event == "phase_begin" && trigger.optional
    }));
}

#[test]
fn fenshi_extra_resolve_doubles_wuzhong_draw() {
    let CLANFENSHI: u32 = 2_827_730_630;
    let (state, owner) = start();
    let mut state = with_skill(state, owner, CLANFENSHI);
    // 注入一张无中生有（普通锦囊）到手牌。
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let wuzhong = state.cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: engine_api::CardDefId::new(120_101),
        kind: CardKind::TrickWuzhong,
        suit: engine_api::Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    state.zones.get_mut(hand).unwrap().cards.push(wuzhong);
    state.turn.current = owner;
    state.turn.phase = Phase::Play;
    let before = state.hand_count(owner);
    let revision = state.revision;
    let (mut state, output) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(20),
            kind: CommandKind::PlayCard {
                card: wuzhong,
                targets: vec![],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(
        matches!(output.result, CommandResult::Accepted { .. }),
        "play rejected: {:?}",
        output.result
    );
    // 排空选择链：纷势 optional 确认（selections=[1]）；wuxie 窗（cancel）。
    let mut key = 21u64;
    let mut confirmed = false;
    while state.has_pending_choice() {
        let kind = state.resolution.continuation_kind;
        let confirm_like = kind != engine_core::state::ContinuationKind::WuxieWindow;
        if confirm_like {
            confirmed = true;
        }
        let choice = ChoiceId::new(state.resolution.pending_choice_id);
        let revision = state.revision;
        let (next, output) = apply_command(
            state,
            Command {
                actor: owner,
                expected_revision: ExpectedRevision(revision),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id: choice,
                    selections: if confirm_like { vec![1] } else { vec![] },
                    cancelled: !confirm_like,
                },
            },
        );
        assert!(
            matches!(output.result, CommandResult::Accepted { .. }),
            "resolve {kind:?} rejected: {:?}",
            output.result
        );
        state = next;
        key += 1;
    }
    assert!(confirmed, "fenshi optional confirm appeared");
    // 无中生有：基础摸 2 + 纷势额外结算再摸 2；打出本身 -1 → 净 +3。
    assert_eq!(state.hand_count(owner), before + 3, "doubled wuzhong draw");
    assert!(state.extra_effect_count.is_empty(), "counter cleared");
}
}

mod original_clan_clanbaichu {
//! Golden coverage for clan 百出 (clanbaichu): locked clan skill that records each suit+category
//! combination of used ordinary tricks in keyed storage, and draws 1 when a combination repeats.
//!
//! Verifies the storage-map primitive (`storage_get`/`storage_set`) used to key the suit+kind
//! map and the `owner_source_ordinary_trick` filter. This is a verifiable subset of upstream
//! `clanbaichu` (the chooseButton name-recording and chooseDrawRecover recover-branch and the
//! dynamic 奇策 acquisition are not yet ported; see the .skill comment).

use engine_api::{
    CardDefId, CardId, Command, CommandKind, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const CLANBAICHU: u32 = 4_194_820_394;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/clan/clan.npack.json");

fn start() -> (Engine, PlayerId, PlayerId) {
    let mut engine = Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_812,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&pkg).unwrap();
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    (engine, owner, other)
}

fn inject_trick_hand(
    state: &mut engine_core::state::GameState,
    player: PlayerId,
    kind: CardKind,
    suit: Suit,
    def: u32,
) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: true,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn owner_with_skill(tx: &mut Transaction, owner: PlayerId) -> engine_api::SkillId {
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(CLANBAICHU),
    });
    run_effects(tx, effects).unwrap();
    tx.state()
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.def.raw() == CLANBAICHU)
        .unwrap()
        .id
}

fn used_card(tx: &mut Transaction, owner: PlayerId, card: CardId) {
    let effects = dispatch_for_events(
        tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card,
            targets: vec![],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("dispatch error: {e:?}"));
    run_effects(tx, effects).unwrap();
}

#[test]
fn committed_package_publishes_clanbaichu_trigger() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pkg.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == CLANBAICHU
            && trigger.event == "card_used"
            && trigger.filter == "owner_source_ordinary_trick"
    }));
}

#[test]
fn first_trick_use_records_and_second_repeat_draws() {
    let (engine, owner, _other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let skill = owner_with_skill(&mut tx, owner);

    // First ordinary-trick use (黑桃普通锦囊 TrickWuzhong): records the suit+kind key, NO draw.
    let trick1 = inject_trick_hand(tx.state_mut(), owner, CardKind::TrickWuzhong, Suit::Spade, 170_201);
    let hand_before_first = tx.state().hand_count(owner);
    used_card(&mut tx, owner, trick1);
    let any_recorded = !tx.state().skills.get(skill).unwrap().content_storage.is_empty();
    assert!(any_recorded, "first trick use records the suit+kind key");
    assert_eq!(
        tx.state().hand_count(owner),
        hand_before_first,
        "first use does NOT draw (key not yet recorded)"
    );

    // Second use of a trick with the SAME suit+kind: key already recorded → draws 1.
    let trick2 = inject_trick_hand(tx.state_mut(), owner, CardKind::TrickWuzhong, Suit::Spade, 170_202);
    let hand_before_second = tx.state().hand_count(owner);
    used_card(&mut tx, owner, trick2);
    assert_eq!(
        tx.state().hand_count(owner),
        hand_before_second + 1,
        "repeated suit+kind → draw 1"
    );
}

#[test]
fn different_suit_records_distinct_keys() {
    let (engine, owner, _other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let skill = owner_with_skill(&mut tx, owner);

    // Use a spade trick (records key A, no draw), then a heart trick (different key B, no draw).
    let trick1 = inject_trick_hand(tx.state_mut(), owner, CardKind::TrickWuzhong, Suit::Spade, 170_211);
    let hand_before = tx.state().hand_count(owner);
    used_card(&mut tx, owner, trick1);
    assert_eq!(tx.state().hand_count(owner), hand_before, "first use → no draw");

    let trick2 = inject_trick_hand(tx.state_mut(), owner, CardKind::TrickWuzhong, Suit::Heart, 170_212);
    let hand_before2 = tx.state().hand_count(owner);
    used_card(&mut tx, owner, trick2);
    assert_eq!(tx.state().hand_count(owner), hand_before2, "new suit+kind → no draw (first for that key)");

    // Two distinct keys recorded (spade vs heart same kind).
    let storage = &tx.state().skills.get(skill).unwrap().content_storage;
    assert_eq!(storage.len(), 2, "two distinct suit+kind keys recorded");
}
}

mod original_clan_clanjianbai {
//! Golden coverage for clan 坚白 (clanjianbai): locked clan skill that tracks each used card
//! category (kind tag) in keyed storage and draws 1 per use (a verifiable subset of upstream's
//! per-category-first-use + chooseControl + recast + gaintag-counter behavior).
//!
//! Verifies the storage-map primitive keyed by card kind and the `owner_source` card_used filter.
//! The full upstream semantics (per-category-first gating, suit keep/recast, gaintag accumulating
//! counters, round-end give+draw) need additional primitives not yet ported; see the .skill comment.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const CLANJIANBAI: u32 = 3_207_476_732;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/clan/clan.npack.json");

fn start() -> (Engine, PlayerId, PlayerId) {
    let mut engine = Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_813,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&pkg).unwrap();
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    (engine, owner, other)
}

fn inject_hand(
    state: &mut engine_core::state::GameState,
    player: PlayerId,
    kind: CardKind,
    def: u32,
) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: true,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn owner_with_skill(tx: &mut Transaction, owner: PlayerId) -> engine_api::SkillId {
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(CLANJIANBAI),
    });
    run_effects(tx, effects).unwrap();
    tx.state()
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.def.raw() == CLANJIANBAI)
        .unwrap()
        .id
}

fn used_card(tx: &mut Transaction, owner: PlayerId, card: CardId) {
    let effects = dispatch_for_events(
        tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card,
            targets: vec![],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("dispatch error: {e:?}"));
    run_effects(tx, effects).unwrap();
}

#[test]
fn committed_package_publishes_clanjianbai_trigger() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pkg.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == CLANJIANBAI
            && trigger.event == "card_used"
            && trigger.filter == "owner_source"
    }));
}

#[test]
fn card_use_records_kind_in_storage_and_first_per_kind_draws() {
    let (engine, owner, _other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let skill = owner_with_skill(&mut tx, owner);

    // First 杀 use (kind=1): records key=1, draws 1 (first per-category).
    let sha = inject_hand(tx.state_mut(), owner, CardKind::Sha, 170_301);
    let hand_before = tx.state().hand_count(owner);
    used_card(&mut tx, owner, sha);
    let storage = &tx.state().skills.get(skill).unwrap().content_storage;
    assert!(storage.contains_key(&(CardKind::Sha as u32)), "kind key recorded");
    assert_eq!(tx.state().hand_count(owner), hand_before + 1, "first per-kind → draw 1");

    // Second 杀 use (same kind=1): key already recorded → does NOT draw.
    let sha2 = inject_hand(tx.state_mut(), owner, CardKind::Sha, 170_303);
    let hand_before2 = tx.state().hand_count(owner);
    used_card(&mut tx, owner, sha2);
    assert_eq!(tx.state().hand_count(owner), hand_before2, "repeat same kind → no draw");

    // First 桃 use (kind=2): different key, first for this kind → draws 1.
    let tao = inject_hand(tx.state_mut(), owner, CardKind::Tao, 170_302);
    let hand_before3 = tx.state().hand_count(owner);
    used_card(&mut tx, owner, tao);
    let storage = &tx.state().skills.get(skill).unwrap().content_storage;
    assert_eq!(storage.len(), 2, "two distinct kind keys recorded");
    assert_eq!(tx.state().hand_count(owner), hand_before3 + 1, "first per new kind → draw 1");
}
}

mod original_clan_clanjianzhi {
//! Golden coverage for clan 谏直 (clanjianzhi): locked clan skill that, at the end of
//! the owner's turn, counts 惬诣 (clanqieyi) fires this turn via the cross-skill state
//! query (turn_fire_count of the 惬诣 instance), offers up to that many raw judgments
//! (top-pile reveals), assigns matching-suit judge cards to chosen players, and deals
//! 1 thunder damage when any judge card's suit was not used this play phase.
//!
//! Play-phase suit mask: Spade=1, Heart=2, Club=4, Diamond=8 (see turn.rs).

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ContinuationKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const CLANJIANZHI: u32 = 2_209_164_637;
/// 惬诣 — 谏直 counts its fires this turn.
const CLANQIEYI: u32 = 3_347_941_832;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/clan/clan.npack.json");

fn start() -> (Engine, PlayerId, PlayerId) {
    let mut engine = Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_813,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&pkg).unwrap();
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    (engine, owner, other)
}

fn strip_and_add_skills(
    tx: &mut Transaction,
    owner: PlayerId,
    defs: &[u32],
) -> Vec<engine_api::SkillId> {
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    let mut effects = effects;
    for def in defs {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(*def),
        });
    }
    run_effects(tx, effects).unwrap();
    tx.state()
        .skills
        .sorted_live()
        .iter()
        .filter(|skill| skill.owner == owner && defs.contains(&skill.def.raw()))
        .map(|skill| skill.id)
        .collect()
}

fn inject_hand(
    state: &mut engine_core::state::GameState,
    player: PlayerId,
    kind: CardKind,
    suit: Suit,
) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(170_100 + kind.tag() as u32),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: true,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn inject_pile_top(
    state: &mut engine_core::state::GameState,
    kind: CardKind,
    suit: Suit,
) -> CardId {
    let draw = state.zones.find(ZoneKind::DrawPile, PlayerId::NONE).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(170_100 + kind.tag() as u32),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: draw,
        owner: PlayerId::NONE,
        face_up: true,
    });
    state.zones.get_mut(draw).unwrap().cards.push(card);
    card
}

fn dispatch_phase_end(tx: &mut Transaction, owner: PlayerId) {
    let fired = dispatch_for_events(
        tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => {
            let _ = run_effects(tx, effects);
        }
        Err(engine_core::effect::EffectError::ChoiceRequested) => {}
        Err(e) => panic!("phase_end dispatch error: {e:?}"),
    }
}

fn resolve_choice(
    state: engine_core::state::GameState,
    actor: PlayerId,
    idempotency_key: IdempotencyKey,
    selections: Vec<u32>,
) -> (engine_core::state::GameState, CommandResult) {
    let choice_id = engine_api::ChoiceId::new(state.resolution.pending_choice_id);
    let (state, pipeline) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key,
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    );
    (state, pipeline.result)
}

#[test]
fn committed_package_publishes_clanjianzhi_trigger() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pkg.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == CLANJIANZHI
            && trigger.event == "phase_end"
            && trigger.filter == "owner_is_target"
    }));
}

#[test]
fn phase_end_without_qieyi_fires_is_noop() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    strip_and_add_skills(&mut tx, owner, &[CLANJIANZHI]);
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    dispatch_phase_end(&mut tx, owner);
    assert!(!tx.state().has_pending_choice());
    assert_eq!(tx.state().players.get(owner).unwrap().hp, owner_hp);
}

#[test]
fn matching_judge_cards_are_assigned_and_non_matching_deals_thunder() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let skills = strip_and_add_skills(&mut tx, owner, &[CLANJIANZHI, CLANQIEYI]);
    let qieyi = skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == CLANQIEYI)
        .unwrap();
    // 惬诣 fired twice this turn (the dispatcher normally increments this).
    tx.state_mut().skills.get_mut(qieyi).unwrap().turn_fire_count = 2;
    // This play phase the owner used a spade and a heart card.
    tx.state_mut().turn.remember_play_phase_card_suit_used(owner, Suit::Spade);
    tx.state_mut().turn.remember_play_phase_card_suit_used(owner, Suit::Heart);
    // Judge 1: spade (matching); judge 2: club (non-matching).
    let spade_judge = inject_pile_top(tx.state_mut(), CardKind::Sha, Suit::Spade);
    let club_judge = inject_pile_top(tx.state_mut(), CardKind::Shan, Suit::Club);
    let owner_hp = tx.state().players.get(owner).unwrap().hp;

    dispatch_phase_end(&mut tx, owner);
    assert!(
        tx.state().has_pending_choice(),
        "judgment count choice must open"
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::JianzhiCountChoice
    );

    // Choose 2 judgments.
    let (state, result) = resolve_choice(
        tx.clone().commit().state,
        owner,
        IdempotencyKey(51),
        vec![2],
    );
    assert!(matches!(result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::JianzhiAssignCard,
        "assignment loop opens with the matching spade card"
    );
    let options: Vec<_> = state
        .resolution
        .continuation_payload
        .get(4..)
        .unwrap_or(&[])
        .to_vec();
    assert_eq!(options, vec![spade_judge.raw()], "only the spade card matches");
    // The non-matching club judge card left the pile for the discard pile (judgments
    // remove their card from the pile; the engine has no limbo zone).
    assert!(
        !state
            .zones
            .get(
                state
                    .zones
                    .find(ZoneKind::DrawPile, PlayerId::NONE)
                    .unwrap()
            )
            .unwrap()
            .cards
            .contains(&club_judge)
    );
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(state.zones.get(discard).unwrap().cards.contains(&club_judge));

    // Assign the spade card to the other player: stage 0 selects the card, stage 1 the
    // recipient.
    let (state, result) = resolve_choice(
        state,
        owner,
        IdempotencyKey(52),
        vec![spade_judge.raw()],
    );
    assert!(matches!(result, CommandResult::Accepted { .. }));
    let (state, result) = resolve_choice(
        state,
        owner,
        IdempotencyKey(53),
        vec![other.raw()],
    );
    assert!(matches!(result, CommandResult::Accepted { .. }));
    assert!(
        state.hand_cards(other).contains(&spade_judge),
        "matching judge card given to the chosen player"
    );
    assert!(
        !state.hand_cards(owner).contains(&spade_judge),
        "owner no longer holds the judge card"
    );
    // The loop ended and the non-matching club judge dealt 1 thunder damage.
    assert_eq!(
        state.players.get(owner).unwrap().hp,
        owner_hp - 1,
        "non-matching judge card dealt thunder damage"
    );
}

#[test]
fn assignment_loop_covers_multiple_matching_cards() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let skills = strip_and_add_skills(&mut tx, owner, &[CLANJIANZHI, CLANQIEYI]);
    let qieyi = skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == CLANQIEYI)
        .unwrap();
    tx.state_mut().skills.get_mut(qieyi).unwrap().turn_fire_count = 2;
    tx.state_mut().turn.remember_play_phase_card_suit_used(owner, Suit::Spade);
    // Both judge cards are spades → both match.
    let spade_judge_1 = inject_pile_top(tx.state_mut(), CardKind::Sha, Suit::Spade);
    let spade_judge_2 = inject_pile_top(tx.state_mut(), CardKind::Shan, Suit::Spade);
    let owner_hp = tx.state().players.get(owner).unwrap().hp;

    dispatch_phase_end(&mut tx, owner);
    let (after_count, result) = resolve_choice(
        tx.clone().commit().state,
        owner,
        IdempotencyKey(61),
        vec![2],
    );
    assert!(matches!(result, CommandResult::Accepted { .. }));
    assert_eq!(
        after_count.resolution.continuation_kind,
        ContinuationKind::JianzhiAssignCard
    );
    // First assignment: stage 0 selects a card (first in the payload), stage 1 a target.
    let tx = Transaction::begin(
        resolve_choice(
            after_count,
            owner,
            IdempotencyKey(62),
            vec![spade_judge_1.raw()],
        )
        .0,
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::JianzhiAssignCard
    );
    let (state, result) = resolve_choice(
        tx.clone().commit().state,
        owner,
        IdempotencyKey(63),
        vec![owner.raw()],
    );
    assert!(matches!(result, CommandResult::Accepted { .. }));
    assert!(state.hand_cards(owner).contains(&spade_judge_1));
    // The loop re-opened for the remaining card.
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::JianzhiAssignCard
    );
    assert_eq!(
        state
            .resolution
            .continuation_payload
            .get(4..)
            .unwrap_or(&[])
            .to_vec(),
        vec![spade_judge_2.raw()]
    );
    // Assign the second card; all matched → no damage.
    let (state, result) = resolve_choice(
        state,
        owner,
        IdempotencyKey(64),
        vec![spade_judge_2.raw()],
    );
    assert!(matches!(result, CommandResult::Accepted { .. }));
    let (state, result) = resolve_choice(
        state,
        owner,
        IdempotencyKey(65),
        vec![other.raw()],
    );
    assert!(matches!(result, CommandResult::Accepted { .. }));
    assert!(state.hand_cards(other).contains(&spade_judge_2));
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp);
}
}

mod original_clan_clanqieyi {
//! Golden coverage for clan 惬诣 (clanqieyi): locked clan skill whose play-phase start
//! resets per-suit first-use storage and privately views the top two draw-pile cards;
//! whose card-used path (first use of a suit this play phase) reveals the top draw-pile
//! card and gains it when it shares color or kind with the used card, else opens the
//! put-one-card-on-top choice.
//!
//! The draw pile's top is its last element; a move into the draw zone lands on top.
//! Storage keys 10..13 hold the per-suit first-use flags (suit tag offset).

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ContinuationKind, ZoneKind};
use engine_core::command::apply_command;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const CLANQIEYI: u32 = 3_347_941_832;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/clan/clan.npack.json");

fn start() -> (Engine, PlayerId, PlayerId) {
    let mut engine = Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_812,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&pkg).unwrap();
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    (engine, owner, other)
}

fn strip_and_add_clanqieyi(tx: &mut Transaction, owner: PlayerId) -> engine_api::SkillId {
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    let mut effects = effects;
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(CLANQIEYI),
    });
    run_effects(tx, effects).unwrap();
    tx.state()
        .skills
        .sorted_live()
        .iter()
        .find(|skill| skill.def.raw() == CLANQIEYI)
        .unwrap()
        .id
}

fn inject_hand(
    state: &mut engine_core::state::GameState,
    player: PlayerId,
    kind: CardKind,
) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(170_100 + kind.tag() as u32),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: true,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn dispatch_phase_play(tx: &mut Transaction, owner: PlayerId) {
    let effects = dispatch_for_events(
        tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::Play,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(tx, effects).unwrap();
}

fn dispatch_card_used(tx: &mut Transaction, user: PlayerId, card: CardId) {
    let fired = dispatch_for_events(
        tx,
        &[DomainEvent::CardUsed {
            player: user,
            card,
            targets: vec![],
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => {
            let _ = run_effects(tx, effects);
        }
        Err(engine_core::effect::EffectError::ChoiceRequested) => {}
        Err(e) => panic!("card_used dispatch error: {e:?}"),
    }
}

#[test]
fn committed_package_publishes_clanqieyi_triggers() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let triggers: Vec<_> = pkg
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == CLANQIEYI)
        .collect();
    assert_eq!(triggers.len(), 2);
    assert!(triggers
        .iter()
        .any(|trigger| trigger.event == "phase_play" && trigger.filter == "owner_is_target"));
    assert!(triggers
        .iter()
        .any(|trigger| trigger.event == "card_used" && trigger.filter == "owner_source"));
}

#[test]
fn phase_start_resets_suit_gate_and_reveals_top_two() {
    let (engine, owner, _other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let skill = strip_and_add_clanqieyi(&mut tx, owner);
    // Simulate stale suit flags from a previous play phase.
    tx.state_mut().skills.storage_set(skill, 10, 1);
    tx.state_mut().skills.storage_set(skill, 12, 1);
    let draw = tx.state().zones.find(ZoneKind::DrawPile, PlayerId::NONE).unwrap();
    let top_two: Vec<_> = tx
        .state()
        .zones
        .get(draw)
        .unwrap()
        .cards
        .iter()
        .rev()
        .take(2)
        .copied()
        .collect();
    assert_eq!(top_two.len(), 2);

    dispatch_phase_play(&mut tx, owner);
    // Suit flags cleared.
    assert_eq!(tx.state().skills.storage_get(skill, 10), 0);
    assert_eq!(tx.state().skills.storage_get(skill, 12), 0);
    // Top two cards still on the pile (view only, no reorder).
    let draw = tx.state().zones.find(ZoneKind::DrawPile, PlayerId::NONE).unwrap();
    let after: Vec<_> = tx
        .state()
        .zones
        .get(draw)
        .unwrap()
        .cards
        .iter()
        .rev()
        .take(2)
        .copied()
        .collect();
    assert_eq!(after, top_two);
}

#[test]
fn first_suit_use_gains_same_kind_top_card() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let skill = strip_and_add_clanqieyi(&mut tx, owner);
    let owner_hand = tx.state().hand_cards(owner);
    let owner_count = owner_hand.len();
    dispatch_phase_play(&mut tx, owner);

    // Force the draw pile's top card to be a 杀: move the top card out, inject a 杀
    // directly on top, and remember the pile's second card for the used card.
    let draw = tx.state().zones.find(ZoneKind::DrawPile, PlayerId::NONE).unwrap();
    let top_sha = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(170_101),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: draw,
        owner: PlayerId::NONE,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(draw).unwrap().cards.push(top_sha);
    // Owner uses a 杀 (same kind) — the revealed top card must be gained.
    let used_sha = inject_hand(tx.state_mut(), owner, CardKind::Sha);
    dispatch_card_used(&mut tx, owner, used_sha);
    assert!(
        tx.state().hand_cards(owner).contains(&top_sha),
        "same-kind top card gained"
    );
    // Original hand + the injected used 杀 + the gained top 杀.
    assert_eq!(tx.state().hand_cards(owner).len(), owner_count + 2);
    // The suit gate is now closed for spade.
    assert_eq!(tx.state().skills.storage_get(skill, 10), 1);
    // The other player's use does not disturb the owner's gate.
    dispatch_card_used(&mut tx, other, used_sha);
    assert_eq!(tx.state().skills.storage_get(skill, 10), 1);
}

#[test]
fn non_matching_reveal_offers_put_card_on_top() {
    let (engine, owner, _other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let skill = strip_and_add_clanqieyi(&mut tx, owner);
    dispatch_phase_play(&mut tx, owner);

    // Top card: 桃 (heart, basic). Owner uses a spade 杀 → neither same color nor kind.
    let draw = tx.state().zones.find(ZoneKind::DrawPile, PlayerId::NONE).unwrap();
    let top_tao = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(170_102),
        kind: CardKind::Tao,
        suit: Suit::Heart,
        rank: 7,
        weapon_range: 0,
        zone: draw,
        owner: PlayerId::NONE,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(draw).unwrap().cards.push(top_tao);
    let used_sha = inject_hand(tx.state_mut(), owner, CardKind::Sha);
    let hand_card = inject_hand(tx.state_mut(), owner, CardKind::Shan);
    dispatch_card_used(&mut tx, owner, used_sha);
    // 桃 stays on the pile; the put-on-top choice opened over the owner's hand cards.
    assert!(!tx.state().hand_cards(owner).contains(&top_tao));
    assert!(
        tx.state().has_pending_choice(),
        "put-on-top choice must open"
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::QieyiPutTopChoice
    );

    // Stage 0: put the 闪 on top of the draw pile.
    let choice_id = engine_api::ChoiceId::new(tx.state().resolution.pending_choice_id);
    let (state, pipeline) = apply_command(
        tx.clone().commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(41),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![hand_card.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert!(!state.hand_cards(owner).contains(&hand_card));
    let draw = state.zones.find(ZoneKind::DrawPile, PlayerId::NONE).unwrap();
    let top = state.zones.get(draw).unwrap().cards.last().copied();
    assert_eq!(top, Some(hand_card), "selected card lands on top of the pile");
}
}

mod original_clan_clanqingjue {
//! Golden coverage for clan 清绝 (clanqingjue): locked clan skill that, after the owner takes
//! damage, draws 1 (a verifiable subset of upstream's first-HP-change → discard non-unique-suit
//! hand cards → chooseButton give/gain behavior).
//!
//! Verifies the damage trigger + draw effect. The full upstream semantics (first-HP-change
//! history gate, suit-uniqueness analysis, chooseToDiscard, chooseButton give/gain, hand-limit
//! exemption modifier, discard-phase exemption) need additional primitives not yet ported;
//! see the .skill comment.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::{damage_nature, EffectRequest};

const CLANQINGJUE: u32 = 3_860_800_298;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/clan/clan.npack.json");

fn start() -> (Engine, PlayerId, PlayerId) {
    let mut engine = Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_814,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&pkg).unwrap();
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    (engine, owner, other)
}

fn owner_with_skill(tx: &mut Transaction, owner: PlayerId) {
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(CLANQINGJUE),
    });
    run_effects(tx, effects).unwrap();
}

#[test]
fn committed_package_publishes_clanqingjue_trigger() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pkg.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == CLANQINGJUE
            && trigger.event == "damage"
            && trigger.filter == "owner_is_target"
    }));
}

#[test]
fn damage_to_owner_draws_one() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    owner_with_skill(&mut tx, owner);

    let hand_before = tx.state().hand_count(owner);
    // A Damage event with the owner as target fires clanqingjue → draw 1.
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source: other,
            target: owner,
            amount: 1,
            nature: damage_nature::NORMAL,
            is_lose_hp: false,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("dispatch error: {e:?}"));
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(
        tx.state().hand_count(owner),
        hand_before + 1,
        "damage to owner → draw 1"
    );
}

#[test]
fn damage_to_other_does_not_draw_for_owner() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    owner_with_skill(&mut tx, owner);

    let hand_before = tx.state().hand_count(owner);
    // Damage to the other player: owner is not the target → no draw.
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source: owner,
            target: other,
            amount: 1,
            nature: damage_nature::NORMAL,
            is_lose_hp: false,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(
        tx.state().hand_count(owner),
        hand_before,
        "damage to other → owner does not draw"
    );
}
}

mod original_clan_clanshenjun {
//! Golden coverage for clan 神君 (clanshenjun): locked clan skill that reveals
//! and gaintags same-kind hand cards when someone uses 杀/锦囊, then lets the
//! owner play one tagged card at phase end.
//!
//! Exercises the primitives added for the clan skill family's second cluster:
//!   - `owner_alive_hand_same_kind_event_card` trigger filter (同名牌),
//!   - `clanshenjun_reveal` effect (CardsRevealed + gaintag by skill def id),
//!   - `owner_alive_has_skill_tagged_cards` phase-end gate,
//!   - `clanshenjun_use` choice continuation (tagged card -> target -> play).
//! 同名牌 is approximated by effective_card_kind equality; the phase-end window
//! is the owner's own play-phase end (see the .skill comment).

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const CLANSHENJUN: u32 = 2_483_457_953;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/clan/clan.npack.json");

fn start() -> (engine_core::Engine, PlayerId, PlayerId) {
    let mut engine = Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_810,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&pkg).unwrap();
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    (engine, owner, other)
}

fn strip_and_add_clanshenjun(tx: &mut Transaction, owner: PlayerId) -> engine_api::SkillId {
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    let mut effects = effects;
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(CLANSHENJUN),
    });
    run_effects(tx, effects).unwrap();
    tx.state()
        .skills
        .sorted_live()
        .iter()
        .find(|skill| skill.def.raw() == CLANSHENJUN)
        .unwrap()
        .id
}

fn clear_hands(tx: &mut Transaction) {
    let seats: Vec<_> = tx.state().players.alive_by_seat();
    for player in seats {
        let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
        let cards: Vec<_> = tx.state().hand_cards(player);
        if !cards.is_empty() {
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .unwrap();
            run_effects(
                tx,
                vec![EffectRequest::MoveCards {
                    cards,
                    to: discard,
                    from: Some(hand),
                }],
            )
            .unwrap();
        }
    }
}

fn inject_hand(state: &mut engine_core::state::GameState, player: PlayerId, kind: CardKind) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(170_100 + kind.tag() as u32),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: true,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn dispatch_card_used(tx: &mut Transaction, user: PlayerId, card: CardId) {
    let fired = dispatch_for_events(
        tx,
        &[DomainEvent::CardUsed {
            player: user,
            card,
            targets: vec![],
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => {
            let _ = run_effects(tx, effects);
        }
        Err(engine_core::effect::EffectError::ChoiceRequested) => {}
        Err(e) => panic!("card_used dispatch error: {e:?}"),
    }
}

#[test]
fn committed_package_publishes_clanshenjun_triggers() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let triggers: Vec<_> = pkg
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == CLANSHENJUN)
        .collect();
    assert_eq!(triggers.len(), 2);
    assert!(triggers.iter().any(|trigger| {
        trigger.event == "card_used" && trigger.filter == "owner_alive_hand_same_kind_event_card"
    }));
    assert!(triggers.iter().any(|trigger| {
        trigger.event == "phase_end" && trigger.filter == "owner_alive_has_skill_tagged_cards"
    }));
}

#[test]
fn same_kind_use_reveals_and_tags_matching_hand_cards() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    strip_and_add_clanshenjun(&mut tx, owner);
    clear_hands(&mut tx);
    // The owner holds a 杀; the other player uses a 杀 → same-kind match.
    let owner_sha = inject_hand(tx.state_mut(), owner, CardKind::Sha);
    let owner_tao = inject_hand(tx.state_mut(), owner, CardKind::Tao);
    let used_sha = inject_hand(tx.state_mut(), other, CardKind::Sha);
    dispatch_card_used(&mut tx, other, used_sha);
    // The matching 杀 was tagged with the skill def; the 桃 was not.
    assert!(tx
        .state()
        .card_has_content_tag(owner_sha, CLANSHENJUN));
    assert!(!tx
        .state()
        .card_has_content_tag(owner_tao, CLANSHENJUN));
    // Both cards stay in hand (reveal, not discard).
    assert!(tx.state().hand_cards(owner).contains(&owner_sha));
    assert!(tx.state().hand_cards(owner).contains(&owner_tao));
}

#[test]
fn non_matching_kind_does_not_tag() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    strip_and_add_clanshenjun(&mut tx, owner);
    clear_hands(&mut tx);
    // Owner holds only 桃; the other uses a 杀 → no same-kind match.
    let owner_tao = inject_hand(tx.state_mut(), owner, CardKind::Tao);
    let used_sha = inject_hand(tx.state_mut(), other, CardKind::Sha);
    dispatch_card_used(&mut tx, other, used_sha);
    assert!(!tx
        .state()
        .card_has_content_tag(owner_tao, CLANSHENJUN));
}

#[test]
fn phase_end_use_plays_tagged_card_on_chosen_target() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let skill = strip_and_add_clanshenjun(&mut tx, owner);
    clear_hands(&mut tx);
    // Tag a 杀 via the reveal path, then end the owner's play phase.
    let owner_sha = inject_hand(tx.state_mut(), owner, CardKind::Sha);
    let used_sha = inject_hand(tx.state_mut(), other, CardKind::Sha);
    dispatch_card_used(&mut tx, other, used_sha);
    assert!(tx
        .state()
        .card_has_content_tag(owner_sha, CLANSHENJUN));
    let other_hp = tx.state().players.get(other).unwrap().hp;

    // 出牌阶段结束：神君 opens the tagged-card use choice.
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => {
            let _ = run_effects(&mut tx, effects);
        }
        Err(engine_core::effect::EffectError::ChoiceRequested) => {}
        Err(e) => panic!("phase_end dispatch error: {e:?}"),
    }
    assert!(
        tx.state().has_pending_choice(),
        "神君 use choice must open at phase end"
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        engine_core::state::ContinuationKind::ClanshenjunUseCard
    );

    // Stage 0: pick the tagged 杀.
    let choice_id = engine_api::ChoiceId::new(tx.state().resolution.pending_choice_id);
    let (state, picked) = apply_command(
        tx.clone().commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![owner_sha.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(picked.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        engine_core::state::ContinuationKind::ClanshenjunUseCard
    );

    // Stage 1: pick the target; the 杀 plays and deals 1.
    let stage1_choice = engine_api::ChoiceId::new(state.resolution.pending_choice_id);
    let (state, used) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: stage1_choice,
                selections: vec![other.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(used.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(other).unwrap().hp, other_hp - 1);
    assert!(!state.hand_cards(owner).contains(&owner_sha));
}
}

mod original_clan_clanxsyingxiang {
//! Golden coverage for clan 英襄 (clanxsyingxiang): locked clan skill derived from 清绝.
//! When a card carrying the clanxsyingxiang gaintag is consumed (leaves its holder) this turn
//! and a card is used, the owner draws. Exercises the gaintag_map foundation primitive:
//!   - `turn.lost_card_tags` tracking (cards' content tags recorded at loss),
//!   - `owner_alive_tagged_card_lost_this_turn` trigger filter.
//! Per-turn approximation of upstream's per-useEvent `hasHistory("lose", gaintag_map)`
//! correlation (the engine has no event-parent ids; see the .skill comment).

use engine_api::{
    CardDefId, CardId, Command, CommandKind, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const CLANXSYINGXIANG: u32 = 3_188_595_142;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/clan/clan.npack.json");

fn start() -> (Engine, PlayerId, PlayerId) {
    let mut engine = Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_810,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&pkg).unwrap();
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    (engine, owner, other)
}

fn inject_hand(state: &mut engine_core::state::GameState, player: PlayerId, def: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: true,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn owner_with_skill(tx: &mut Transaction, owner: PlayerId) -> engine_api::SkillId {
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(CLANXSYINGXIANG),
    });
    run_effects(tx, effects).unwrap();
    tx.state()
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.def.raw() == CLANXSYINGXIANG)
        .unwrap()
        .id
}

#[test]
fn committed_package_publishes_clanxsyingxiang_trigger() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pkg.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == CLANXSYINGXIANG
            && trigger.event == "card_used"
            && trigger.filter == "owner_alive_tagged_card_lost_this_turn"
    }));
}

#[test]
fn tagged_card_consumed_this_turn_fires_cross_player_draw() {
    // The owner holds clanxsyingxiang. A tagged card was consumed this turn. When ANOTHER player
    // uses a card targeting the owner, the skill fires draw_all_event_targets(1): the user AND
    // the owner (a target) each draw 1. Verifies the cross-player draw branch.
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    owner_with_skill(&mut tx, owner);

    // Record a tagged-card loss so the filter passes.
    tx.state_mut().turn.record_lost_card_tag(CLANXSYINGXIANG);
    assert!(tx.state().turn.lost_card_tag(CLANXSYINGXIANG));

    let used_card = inject_hand(tx.state_mut(), other, 170_001);
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: other,
            card: used_card,
            targets: vec![owner],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    // draw_all_event_targets(1) is emitted as one DrawAllEventTargets request whose player list
    // is the event player + the declared targets; run_effects expands it to per-player draws.
    let draw_all = fired.iter().find_map(|e| match e {
        EffectRequest::DrawAllEventTargets { players, count } => Some((players.clone(), *count)),
        _ => None,
    });
    let (mut players, count) = draw_all.expect("draw_all_event_targets(1) effect emitted");
    players.sort();
    let mut expected = vec![other, owner];
    expected.sort();
    assert_eq!(players, expected, "draws the event player and the owner (target)");
    assert_eq!(count, 1);
    let owner_before = tx.state().hand_count(owner);
    let other_before = tx.state().hand_count(other);
    run_effects(&mut tx, fired).unwrap();
    assert_eq!(tx.state().hand_count(owner), owner_before + 1);
    assert_eq!(tx.state().hand_count(other), other_before + 1);
}

#[test]
fn no_tagged_card_lost_blocks_the_trigger() {
    // Symmetric negative case: without a tagged-card loss, the filter blocks the trigger.
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    owner_with_skill(&mut tx, owner);
    let used_card = inject_hand(tx.state_mut(), other, 170_001);
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: other,
            card: used_card,
            targets: vec![],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    let owner_draws = fired
        .iter()
        .filter(|e| matches!(e, EffectRequest::DrawCards { player, .. } | EffectRequest::DrawCardsFromSkill { player, .. } if *player == owner))
        .count();
    assert_eq!(owner_draws, 0, "no tagged card lost → no draw");
}

#[test]
fn gaintag_map_primitive_records_and_reports_lost_tags() {
    // State-layer verification of the gaintag_map foundation primitive (independent of the
    // runtime card_used dispatch path): recording a lost tag makes it queryable, and the
    // turn-scoped set clears on rollover.
    let (engine, owner, _other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let _ = owner;
    // Initially no tags are recorded as lost.
    assert!(!tx.state().turn.lost_card_tag(CLANXSYINGXIANG));
    // Recording the tag makes it present.
    tx.state_mut().turn.record_lost_card_tag(CLANXSYINGXIANG);
    assert!(tx.state().turn.lost_card_tag(CLANXSYINGXIANG));
    // Round/turn rollover clears the set.
    tx.state_mut().turn.clear_lost_card_tags();
    assert!(!tx.state().turn.lost_card_tag(CLANXSYINGXIANG));
}
}

mod original_clan_clanyuzhi {
//! Golden coverage for clan 迂志 (clanyuzhi): locked clan skill driven by the new
//! round_started / round_ended engine events (emitted at the seat-0 turn rollover).
//!
//! Round start: the owner chooses one hand card → reveal + gaintag + draw its name
//! length (tagging the draws) + record the round draw counts in per-skill keyed
//! storage. Round end: discard every gaintagged hand card; if this round's 迂志 draws
//! exceeded the previous round's (both positive) or the round's used-card count, the
//! owner loses 保族 or takes 1 thunder damage.
//!
//! Storage keys (see `clanyuzhi` primitives in pack_refresh.rs): 1 = drawn this
//! round, 2 = drawn last round, 3 = name-length mark.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ContinuationKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const CLANYUZHI: u32 = 2_443_866_463;
/// 保族 (clanbaozu) — the clan skill 迂志 can remove at round end.
const CLANBAOZU: u32 = 3_600_751_244;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/clan/clan.npack.json");

const YUZHI_KEY_DRAWN_ROUND: u32 = 1;
const YUZHI_KEY_DRAWN_LAST: u32 = 2;

fn start() -> (Engine, PlayerId, PlayerId) {
    let mut engine = Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_811,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&pkg).unwrap();
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    (engine, owner, other)
}

fn strip_and_add_clanyuzhi(tx: &mut Transaction, owner: PlayerId) -> engine_api::SkillId {
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    let mut effects = effects;
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(CLANYUZHI),
    });
    run_effects(tx, effects).unwrap();
    tx.state()
        .skills
        .sorted_live()
        .iter()
        .find(|skill| skill.def.raw() == CLANYUZHI)
        .unwrap()
        .id
}

fn add_clanbaozu(tx: &mut Transaction, owner: PlayerId) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(CLANBAOZU),
        }],
    )
    .unwrap();
}

fn inject_hand(
    state: &mut engine_core::state::GameState,
    player: PlayerId,
    kind: CardKind,
) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(170_100 + kind.tag() as u32),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: true,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn dispatch_round_event(
    tx: &mut Transaction,
    event: DomainEvent,
) -> Result<(), engine_core::effect::EffectError> {
    let effects = dispatch_for_events(tx, &[event], &mut TriggerFireRecord::new())?;
    match run_effects(tx, effects) {
        Ok(_) | Err(engine_core::effect::EffectError::ChoiceRequested) => Ok(()),
        Err(e) => Err(e),
    }
}

fn resolve_choice(
    state: engine_core::state::GameState,
    actor: PlayerId,
    idempotency_key: IdempotencyKey,
    selections: Vec<u32>,
) -> (engine_core::state::GameState, CommandResult) {
    let choice_id = engine_api::ChoiceId::new(state.resolution.pending_choice_id);
    let (state, pipeline) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key,
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    );
    (state, pipeline.result)
}

#[test]
fn committed_package_publishes_clanyuzhi_round_triggers() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let triggers: Vec<_> = pkg
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == CLANYUZHI)
        .collect();
    assert_eq!(triggers.len(), 2);
    assert!(triggers
        .iter()
        .any(|trigger| trigger.event == "round_started" && trigger.filter == "owner_alive"));
    assert!(triggers
        .iter()
        .any(|trigger| trigger.event == "round_ended" && trigger.filter == "owner_alive"));
}

#[test]
fn round_start_choice_reveals_and_draws_name_length_tagged() {
    let (engine, owner, _other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let skill = strip_and_add_clanyuzhi(&mut tx, owner);
    // A 决斗 (name length 2) in hand; the round-start choice must offer it.
    let juedou = inject_hand(tx.state_mut(), owner, CardKind::TrickJuedou);

    // Round rolls to round 2 with the owner first: the round_started event fires the
    // trigger, which opens the hand-card choice.
    dispatch_round_event(
        &mut tx,
        DomainEvent::RoundStarted {
            round: 2,
            current: owner,
        },
    )
    .unwrap();
    assert!(
        tx.state().has_pending_choice(),
        "迂志 reveal choice must open at round start"
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::ClanyuzhiRevealChoice
    );

    // Stage 0: pick the 决斗 → reveal + gaintag + draw 2 (tagged) + storage.
    let (state, result) = resolve_choice(
        tx.clone().commit().state,
        owner,
        IdempotencyKey(11),
        vec![juedou.raw()],
    );
    assert!(matches!(result, CommandResult::Accepted { .. }));
    assert!(
        state.card_has_content_tag(juedou, CLANYUZHI),
        "chosen card must be gaintagged"
    );
    let drawn: Vec<_> = state
        .hand_cards(owner)
        .into_iter()
        .filter(|card| *card != juedou && state.card_has_content_tag(*card, CLANYUZHI))
        .collect();
    assert_eq!(drawn.len(), 2, "name length 2 → exactly 2 tagged draws");
    let instance = state.skills.get(skill).unwrap();
    assert_eq!(
        instance.content_storage.get(&YUZHI_KEY_DRAWN_ROUND),
        Some(&2),
        "this round's 迂志 draw count recorded"
    );
    assert_eq!(
        instance.content_storage.get(&YUZHI_KEY_DRAWN_LAST),
        Some(&0),
        "previous round's count rotates in as 0"
    );
}

#[test]
fn round_end_discards_tagged_and_takes_thunder_damage_when_ambitious() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let skill = strip_and_add_clanyuzhi(&mut tx, owner);
    // 保族 present → the ambition check offers the choice (lose 保族 or damage).
    add_clanbaozu(&mut tx, owner);
    // A tagged card in hand (as if revealed at round start).
    let tagged = inject_hand(tx.state_mut(), owner, CardKind::Sha);
    tx.state_mut().add_content_card_tag(tagged, CLANYUZHI);
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    // This round drew 2, previous round drew 1 → ambitious (2 > 1 > 0).
    tx.state_mut()
        .skills
        .storage_set(skill, YUZHI_KEY_DRAWN_ROUND, 2);
    tx.state_mut()
        .skills
        .storage_set(skill, YUZHI_KEY_DRAWN_LAST, 1);

    dispatch_round_event(
        &mut tx,
        DomainEvent::RoundEnded {
            round: 1,
            current: other,
        },
    )
    .unwrap();
    // The tagged card was discarded before the ambition choice.
    assert!(
        !tx.state().hand_cards(owner).contains(&tagged),
        "tagged card discarded at round end"
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::ClanyuzhiEndChoice,
        "ambition choice opens when draws exceed previous round"
    );

    // Choice: 1 = take 1 thunder damage.
    let (state, result) = resolve_choice(
        tx.clone().commit().state,
        owner,
        IdempotencyKey(21),
        vec![1],
    );
    assert!(matches!(result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.players.get(owner).unwrap().hp,
        owner_hp - 1,
        "owner takes 1 thunder damage"
    );
}

#[test]
fn round_end_without_ambition_is_noop() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    strip_and_add_clanyuzhi(&mut tx, owner);
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    // No tagged cards, no draws recorded → nothing happens.
    dispatch_round_event(
        &mut tx,
        DomainEvent::RoundEnded {
            round: 1,
            current: other,
        },
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());
    assert_eq!(tx.state().players.get(owner).unwrap().hp, owner_hp);
}

#[test]
fn round_end_without_baozu_deals_thunder_damage_directly() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let skill = strip_and_add_clanyuzhi(&mut tx, owner);
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    tx.state_mut()
        .skills
        .storage_set(skill, YUZHI_KEY_DRAWN_ROUND, 2);
    tx.state_mut()
        .skills
        .storage_set(skill, YUZHI_KEY_DRAWN_LAST, 1);

    dispatch_round_event(
        &mut tx,
        DomainEvent::RoundEnded {
            round: 1,
            current: other,
        },
    )
    .unwrap();
    // No 保族 → upstream takes the damage directly without a choice.
    assert!(!tx.state().has_pending_choice());
    assert_eq!(
        tx.state().players.get(owner).unwrap().hp,
        owner_hp - 1,
        "owner takes 1 thunder damage without a choice"
    );
}

#[test]
fn round_end_can_remove_clanbaozu_instead_of_damage() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let skill = strip_and_add_clanyuzhi(&mut tx, owner);
    add_clanbaozu(&mut tx, owner);
    assert!(tx
        .state()
        .skills
        .sorted_live()
        .iter()
        .any(|instance| instance.def.raw() == CLANBAOZU));
    tx.state_mut()
        .skills
        .storage_set(skill, YUZHI_KEY_DRAWN_ROUND, 2);
    tx.state_mut()
        .skills
        .storage_set(skill, YUZHI_KEY_DRAWN_LAST, 1);

    dispatch_round_event(
        &mut tx,
        DomainEvent::RoundEnded {
            round: 1,
            current: other,
        },
    )
    .unwrap();
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::ClanyuzhiEndChoice
    );

    // Choice: 0 = lose 保族; HP untouched.
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    let (state, result) = resolve_choice(
        tx.clone().commit().state,
        owner,
        IdempotencyKey(31),
        vec![0],
    );
    assert!(matches!(result, CommandResult::Accepted { .. }));
    assert!(
        !state
            .skills
            .sorted_live()
            .iter()
            .any(|instance| instance.def.raw() == CLANBAOZU),
        "保族 removed"
    );
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp);
}
}

mod original_clan_clanzhongliu {
//! Golden coverage for clan 中流 (clanzhongliu): locked clan skill that refreshes its own
//! use counts when its owner uses a card and no other member of 太原王氏 lost hand cards.
//!
//! This exercises the three Host primitives added to unlock the clan skill family:
//!   - character `clans` attribute (registry lookup, like `group`),
//!   - `refresh_skill` effect (single-skill per-turn counter reset),
//!   - `owner_is_event_player_no_<clan>_lost_hand_this_turn` trigger filter.
//! The card-loss check is a per-turn approximation of upstream's per-useEvent correlation
//! (the engine tracks per-turn card loss, not per-event-parent loss).

use engine_api::{
    CardDefId, CardId, Command, CommandKind, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const CLANZHONGLIU: u32 = 3_016_047_578;
const CLAN_WANGLING: u32 = 2_803_794_166;
/// Another 太原王氏 character used for the opposing seat so the clan-loss filter has a
/// same-clan player to observe.
const CLAN_WANGCHANG: u32 = 3_580_915_327;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/clan/clan.npack.json");

fn start() -> (engine_core::Engine, PlayerId, PlayerId) {
    let mut engine = Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_809,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&pkg).unwrap();
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    (engine, owner, other)
}

fn add_clanzhongliu(tx: &mut Transaction, owner: PlayerId) -> engine_api::SkillId {
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(CLANZHONGLIU),
    });
    run_effects(tx, effects).unwrap();
    tx.state()
        .skills
        .sorted_live()
        .iter()
        .find(|skill| skill.def.raw() == CLANZHONGLIU)
        .unwrap()
        .id
}

/// Assign two 太原王氏 clan characters (from the loaded clan package) to the seats, since the
/// default identity bootstrap draws characters from the standard pool which carries no clans.
fn assign_clan_seats(tx: &mut Transaction, owner: PlayerId, other: PlayerId) {
    tx.state_mut().players.get_mut(owner).unwrap().character_def = CLAN_WANGLING;
    tx.state_mut().players.get_mut(other).unwrap().character_def = CLAN_WANGCHANG;
}

fn inject_hand(state: &mut engine_core::state::GameState, player: PlayerId) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(170_001),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: true,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn dispatch_card_used(tx: &mut Transaction, user: PlayerId, card: CardId) {
    let effects = dispatch_for_events(
        tx,
        &[DomainEvent::CardUsed {
            player: user,
            card,
            targets: vec![],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(tx, effects).unwrap();
}

#[test]
fn committed_package_publishes_clanzhongliu_trigger_and_clan_roster() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    // The locked trigger is published with the clan-loss filter on card_used.
    assert!(pkg.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == CLANZHONGLIU
            && trigger.event == "card_used"
            && trigger
                .filter
                .ends_with("_lost_hand_this_turn")
            && trigger.filter.contains("太原王氏")
    }));
    // clan_wangling carries the 太原王氏 clan membership.
    let wangling = pkg
        .character_index
        .iter()
        .find(|character| character.id == CLAN_WANGLING)
        .unwrap();
    assert_eq!(wangling.clans, ["太原王氏"]);
}

#[test]
fn refresh_skill_clears_use_counts_when_no_clan_member_lost_hand() {
    let (engine, owner, other) = start();
    // Assign two 太原王氏 clan characters to the seats; the owner uses a card while no one has
    // lost hand cards this turn → clanzhongliu fires refresh_skill.
    let mut tx = Transaction::begin(engine.state().clone());
    assign_clan_seats(&mut tx, owner, other);
    let skill = add_clanzhongliu(&mut tx, owner);
    // Simulate prior activations so use_count is non-zero before the refresh.
    {
        let instance = tx.state_mut().skills.get_mut(skill).unwrap();
        instance.use_count = 2;
        instance.turn_fire_count = 1;
    }
    let card = inject_hand(tx.state_mut(), owner);
    assert_eq!(tx.state().skills.get(skill).unwrap().use_count, 2);
    dispatch_card_used(&mut tx, owner, card);
    // refresh_skill zeroed the per-turn counters.
    let refreshed = tx.state().skills.get(skill).unwrap();
    assert_eq!(refreshed.use_count, 0);
    assert_eq!(refreshed.turn_fire_count, 0);
}

#[test]
fn filter_blocks_when_a_clan_member_lost_hand_this_turn() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    assign_clan_seats(&mut tx, owner, other);
    let skill = add_clanzhongliu(&mut tx, owner);
    // Record that the other (clan-member) player lost a hand card this turn.
    let other_card = inject_hand(tx.state_mut(), other);
    tx.state_mut().turn.record_card_loss(other, other_card);
    assert!(tx.state().turn.card_loss_count(other) > 0);
    // Bump the owner's counts so we can detect that refresh did NOT fire.
    {
        let instance = tx.state_mut().skills.get_mut(skill).unwrap();
        instance.use_count = 2;
    }
    let card = inject_hand(tx.state_mut(), owner);
    dispatch_card_used(&mut tx, owner, card);
    // The filter rejected the trigger → counts unchanged (no refresh_skill).
    assert_eq!(tx.state().skills.get(skill).unwrap().use_count, 2);
}
}

mod original_clan_draw_to {
//! Golden for CLAN「还阴」「清励」「八龙」的补牌语义。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const CLANBALONG: u32 = 2_722_267_819;
const CLANHUANYIN: u32 = 2_722_285_495;
const CLANQINGLI: u32 = 2_992_461_968;
const CHENLIUWUSHI: u32 = 3_882_274_053;
const CLANDANDAO: u32 = 2_693_545_350;
/// Concatenate every committed clan source in pack order (alphabetical),
/// mirroring `content-cli pack` input concatenation so the logic hash matches
/// the committed npack regardless of future roster growth.
fn clan_sources() -> String {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../content/original/ported/clan");
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

const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/clan/clan.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn add_skill(tx: &mut Transaction, owner: PlayerId, skill_def: u32) {
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
    run_effects(tx, effects).unwrap();
}

fn inject_hand(state: &mut GameState, player: PlayerId, kind: CardKind, def: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn committed_package_matches_source_and_draw_to_semantics() {
    // The dying pipeline nests run_effects several levels deep with large frames;
    // run on a dedicated thread with a bigger stack to stay clear of the default.
    std::thread::Builder::new()
        .stack_size(16 << 20)
        .spawn(|| run_clan_draw_to_semantics())
        .unwrap()
        .join()
        .unwrap();
}

fn run_clan_draw_to_semantics() {
    let committed = package();
    let mut compiled = content_compiler::compile_source_named("clan", &clan_sources()).unwrap();
    compiled.character_index = committed.character_index.clone();
    compiled.manifest.dependencies = committed.manifest.dependencies.clone();
    compiled.manifest.logic_hash = compiled.compute_logic_hash();
    assert_eq!(committed.manifest.logic_hash, compiled.manifest.logic_hash);
    for def in [
        CLANHUANYIN,
        CLANQINGLI,
        CHENLIUWUSHI,
        CLANBALONG,
        CLANDANDAO,
    ] {
        assert!(
            committed
                .skill_index
                .iter()
                .any(|skill| skill.skill_def_id == def),
            "clan pack must contain {def}"
        );
    }

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
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;

    let mut dying = Transaction::begin(engine.state().clone());
    add_skill(&mut dying, owner, CLANHUANYIN);
    run_effects(
        &mut dying,
        vec![
            EffectRequest::DiscardFromHand {
                player: owner,
                count: u32::MAX,
            },
            EffectRequest::EnterDying { player: owner },
        ],
    )
    .unwrap();
    assert_eq!(dying.state().hand_count(owner), 4);

    let mut phase_end = Transaction::begin(engine.state().clone());
    add_skill(&mut phase_end, owner, CLANQINGLI);
    run_effects(
        &mut phase_end,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        }],
    )
    .unwrap();
    phase_end.state_mut().players.get_mut(owner).unwrap().max_hp = 7;
    run_effects(
        &mut phase_end,
        vec![EffectRequest::SetTurnState {
            current: other,
            phase: engine_core::state::Phase::End.to_u8(),
        }],
    )
    .unwrap();
    assert_eq!(phase_end.state().hand_count(owner), 5);
}

#[test]
fn balong_only_uses_the_first_hp_change_and_requires_a_unique_trick_majority() {
    let committed = package();
    assert_eq!(committed.manifest.rule_vm_abi_version, 649);
    assert_eq!(
        committed
            .trigger_index
            .iter()
            .filter(|trigger| trigger.skill_def_id == CLANBALONG)
            .map(|trigger| trigger.event.as_str())
            .collect::<Vec<_>>(),
        ["damage", "hp_lost", "hp_recovered", "max_hp_changed"]
    );

    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(1),
        kind: CommandKind::StartGame {
            player_count: 3,
            rng_seed: 20_260_731,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&committed).unwrap();
    let owner = engine.state().turn.current;
    let source = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;

    let mut first = Transaction::begin(engine.state().clone());
    add_skill(&mut first, owner, CLANBALONG);
    run_effects(
        &mut first,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        }],
    )
    .unwrap();
    let trick = inject_hand(first.state_mut(), owner, CardKind::TrickWuzhong, 336_001);
    let events = run_effects(
        &mut first,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert!(events.iter().any(|event| matches!(
        event,
        DomainEvent::CardsRevealed { cards, to } if cards == &[trick] && to.is_empty()
    )));
    assert_eq!(first.state().hand_count(owner), 3);

    let second_events = run_effects(
        &mut first,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert!(!second_events
        .iter()
        .any(|event| matches!(event, DomainEvent::CardsRevealed { .. })));
    assert_eq!(first.state().hand_count(owner), 3);

    let mut tied = Transaction::begin(engine.state().clone());
    add_skill(&mut tied, owner, CLANBALONG);
    run_effects(
        &mut tied,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        }],
    )
    .unwrap();
    inject_hand(tied.state_mut(), owner, CardKind::TrickWuzhong, 336_002);
    inject_hand(tied.state_mut(), owner, CardKind::Sha, 336_003);
    let tied_events = run_effects(
        &mut tied,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert!(!tied_events
        .iter()
        .any(|event| matches!(event, DomainEvent::CardsRevealed { .. })));
    assert_eq!(tied.state().hand_count(owner), 2);
}
}

mod original_clan_name_length_batch {
//! Zero-new-primitive batch (2026-08-15): cardNameLength family.
//! clanjiejian 诫鉴 / clanhuanghan 惶汗 / clanxieshu 胁书 — semantics and
//! approximations documented in the .skill headers.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const CLANJIEJIAN: u32 = 3_544_602_984;
const CLANHUANGHAN: u32 = 4_234_060_719;
const CLANXIESHU: u32 = 2_196_874_944;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/clan/clan.npack.json");

fn start() -> (GameState, PlayerId, PlayerId) {
    let mut engine = Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.load_package(&pkg).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_815,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    (engine.state().clone(), owner, other)
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

fn inject_sha(state: &mut GameState, player: PlayerId) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(93_459),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: true,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn commit_sha_damage(
    state: GameState,
    source: PlayerId,
    target: PlayerId,
    card: CardId,
) -> (GameState, Result<Vec<engine_api::DomainEvent>, EffectError>) {
    let mut tx = Transaction::begin(state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::CommitDamage {
            damage_id: 0,
            source,
            target,
            amount: 1,
            base_amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
            card,
            card_kind: 1,
            card_flags: 0,
        }],
    );
    (tx.commit().state, result)
}

fn confirm(state: GameState, actor: PlayerId, key: u64) -> GameState {
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (state, output) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: choice,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn committed_package_publishes_name_length_family_triggers() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let jiejian = pkg
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == CLANJIEJIAN)
        .expect("clanjiejian trigger");
    assert_eq!(jiejian.event, "card_used");
    assert_eq!(jiejian.filter, "owner_is_source");
    assert!(jiejian.optional);
    let huanghan = pkg
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == CLANHUANGHAN)
        .expect("clanhuanghan trigger");
    assert_eq!(huanghan.event, "damage");
    assert_eq!(huanghan.filter, "owner_is_target");
    assert!(!huanghan.optional);
    let xieshu_events: Vec<_> = pkg
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == CLANXIESHU)
        .map(|trigger| (trigger.event.as_str(), trigger.filter.as_str(), trigger.optional))
        .collect();
    assert_eq!(
        xieshu_events,
        vec![
            ("damage", "owner_is_target", true),
            ("damage", "owner_is_source", true),
        ]
    );
}

#[test]
fn huanghan_draws_name_length_then_discards_lost_hp() {
    let (state, owner, other) = start();
    let state = with_skill(state, owner, CLANHUANGHAN);
    let mut state = state;
    let player = state.players.get_mut(owner).unwrap();
    player.max_hp = 10; // 拉高上限，避免连击伤害把测试武将推进濒死求桃选择
    player.hp = 8; // 已受伤 2 点
    let sha = inject_sha(&mut state, owner);
    let extra1 = inject_sha(&mut state, owner);
    let extra2 = inject_sha(&mut state, owner);
    let before = state.hand_count(owner);
    let (state, result) = commit_sha_damage(state, other, owner, sha);
    assert!(result.is_ok(), "forced trigger chain: {result:?}");
    // 「杀」名字数 1 → 摸 1；body 在伤害应用后求值（对齐上游 damageEnd 的
    // getDamagedHp）→ 已损失 3 → 弃 3：净 -2。
    assert_eq!(state.hand_count(owner), before - 2);
    assert!(state.zones.find(ZoneKind::Hand, owner).is_some());
    let _ = (extra1, extra2);
}

#[test]
fn xieshu_confirm_links_and_discards_then_blocks_second_use() {
    let (state, owner, other) = start();
    let state = with_skill(state, owner, CLANXIESHU);
    let mut state = state;
    let player = state.players.get_mut(owner).unwrap();
    player.max_hp = 10; // 同上：避开濒死
    player.hp = 9; // 已受伤 1 点
    let sha = inject_sha(&mut state, owner);
    let before = state.hand_count(owner);

    let (state, result) = commit_sha_damage(state, other, owner, sha);
    assert_eq!(result, Err(EffectError::ChoiceRequested), "optional confirm");
    let mut state = confirm(state, owner, 1);
    // 弃 1（名字数）+ 摸 2（伤害应用后已损失 2，对齐上游 damageEnd 语义）：净 +1，
    // 且横置已生效。
    assert_eq!(state.hand_count(owner), before + 1);

    // 已横置（mark 2）：optional 语义下确认仍会弹出（body 守卫在 confirm 之后），
    // 但效果被 mark_count 守卫拦下——拒绝后无弃牌/摸牌/再横置。
    let sha2 = inject_sha(&mut state, owner);
    let before_second = state.hand_count(owner);
    let (state, result) = commit_sha_damage(state, other, owner, sha2);
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (state, output) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(9),
            kind: CommandKind::ResolveChoice {
                choice_id: choice,
                selections: vec![],
                cancelled: true,
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), before_second, "guard blocks effects");
    assert_eq!(
        engine_core::query::query(
            &state,
            &QueryRequest::MarkCount { player: owner, mark: 2 },
            &mut engine_core::query::QueryContext::new(),
        ),
        QueryResponse::Int(1),
        "still exactly one link mark"
    );
}
}
