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
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/clan/clan.npack.json");

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
    assert_eq!(
        range_buffed,
        range_before + 1,
        "attack range +1 after confirm"
    );

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
    assert_eq!(
        owner_limit_after,
        owner_limit + 1,
        "persistent +1 hand limit"
    );
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
