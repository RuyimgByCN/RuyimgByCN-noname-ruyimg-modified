//! 第一百六十二批：onlyOL「仁心」（ol_caochong olrenxin）近似重写。
//! 上游：其他角色进入濒死时，若你有装备牌（手牌或装备区），可弃一张装备牌并翻面，
//! 令其回复至 1 点体力。实现：dying + owner_alive_other_dying_has_equip_card（本批
//! 六点注册）→ turn_over(self) + recover_hp(event_player, 1 - hp(event_player))。
//! 近似：弃装备牌代价未建模（optional 确认保留选择权）。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const OLRENXIN: u32 = 4_255_762_868;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/onlyOL/onlyOL.npack.json");

fn started(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let state = engine.state().clone();
    let owner = state.turn.current;
    let dying = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    (state, owner, dying)
}

fn strip_and_arm(state: GameState, owner: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain(
            tx.state()
                .players
                .alive_by_seat()
                .into_iter()
                .map(|player| EffectRequest::DiscardFromHand {
                    player,
                    count: u32::MAX,
                }),
        )
        .chain([EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(OLRENXIN),
        }])
        .collect();
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.commit().state
}

fn inject_equip_hand(tx: &mut Transaction, owner: PlayerId, def: u32) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Weapon,
        suit: Suit::Spade,
        rank: 9,
        weapon_range: 1,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn push_to_dying(state: GameState, dying: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    // hp 直接归零后派发 EnteredDying 事件（olfuli 同款）；ChoiceRequested 属预期
    // 控制流（濒死求桃窗或仁心确认）。
    tx.state_mut().players.get_mut(dying).unwrap().hp = 0;
    let _ = engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::EnteredDying {
            player: dying,
            source: PlayerId::NONE,
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    );
    tx.commit().state
}

fn cancel_pending(state: GameState, actor: PlayerId, key: u64) -> GameState {
    let choice_id = engine_api::ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = engine_core::command::apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: Vec::new(),
                cancelled: true,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "cancel rejected: {:?}",
        result.result
    );
    state
}

fn confirm(state: GameState, owner: PlayerId, key: u64) -> GameState {
    let choice_id = engine_api::ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = engine_core::command::apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "confirm rejected: {:?}",
        result.result
    );
    state
}

fn hp_of(state: &GameState, player: PlayerId) -> i32 {
    state.players.get(player).unwrap().hp
}

#[test]
fn package_publishes_olrenxin_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == OLRENXIN
            && trigger.event == "dying"
            && trigger.filter == "owner_alive_other_dying_has_equip_card"
            && trigger.optional
    }));
    assert!(!package
        .modifier_index
        .iter()
        .any(|modifier| modifier.skill_def_id == OLRENXIN && modifier.target == "hand_limit"));
}

#[test]
fn olrenxin_rescues_dying_other_to_one_hp_at_turnover_cost() {
    let (state, owner, dying) = started(2_026_090_901);
    let state = strip_and_arm(state, owner);

    // owner 手牌含一张装备牌（过滤器门槛），濒死者被打入濒死。
    let mut tx = Transaction::begin(state);
    inject_equip_hand(&mut tx, owner, 98_801);
    let state = tx.commit().state;

    let mut state = push_to_dying(state, dying);
    assert!(hp_of(&state, dying) <= 0, "target must be dying");
    // 逐步跳过濒死求桃窗口，直到仁心确认出现（或窗口耗尽判失败）。
    for key in 2..12 {
        if state.resolution.continuation_kind
            == engine_core::state::ContinuationKind::OptionalTrigger
        {
            break;
        }
        if !state.has_pending_choice() {
            break;
        }
        state = cancel_pending(state, dying, key);
    }
    assert_eq!(
        state.resolution.continuation_kind,
        engine_core::state::ContinuationKind::OptionalTrigger,
        "other dying must open the renxin confirm"
    );
    let state = confirm(state, owner, 12);

    // 濒死者回复至 1 点体力（近似口径：无弃装备牌代价）。
    assert_eq!(
        hp_of(&state, dying),
        1,
        "dying player recovers to exactly 1"
    );
}

#[test]
fn olrenxin_skipped_without_any_equip_card() {
    let (state, owner, dying) = started(2_026_090_902);
    let state = strip_and_arm(state, owner);

    // owner 无装备牌（手牌与装备区皆空）：濒死事件不触发仁心。
    let mut state = push_to_dying(state, dying);
    assert!(hp_of(&state, dying) <= 0, "target must be dying");
    for key in 2..12 {
        if state.resolution.continuation_kind
            == engine_core::state::ContinuationKind::OptionalTrigger
        {
            break;
        }
        if !state.has_pending_choice() {
            break;
        }
        state = cancel_pending(state, dying, key);
    }
    assert_ne!(
        state.resolution.continuation_kind,
        engine_core::state::ContinuationKind::OptionalTrigger,
        "no equip card in hand or equip zone must gate renxin"
    );
}
