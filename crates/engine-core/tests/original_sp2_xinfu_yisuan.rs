//! 第一百六十一批：sp2「翼算」（李傕 xinfu_yisuan）重写——新过滤器 + 既有原语。
//! 上游：出牌阶段限一次，你使用普通锦囊牌结算结束后，可以减 1 点体力上限，
//! 然后获得此牌对应牌堆里的实体牌。实现：card_resolved +
//! owner_source_trick_resolved_own_play_phase_once_per_turn（本批六点注册）→
//! change_max_hp(self, 0 - 1) + gain_event_cards(self)。
//! 近似：gain_event_cards 只取已进弃牌堆者（filterInD 边界）。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const YISUAN: u32 = 4_087_417_398;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp2/sp2.npack.json");

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
    let other = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    (state, owner, other)
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
            skill_def: SkillDefId::new(YISUAN),
        }])
        .collect();
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.commit().state
}

fn play_card(
    state: GameState,
    actor: PlayerId,
    kind: CardKind,
    targets: Vec<PlayerId>,
    key: u64,
) -> (GameState, PipelineResult) {
    let mut tx = Transaction::begin(state);
    let zone = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(98_700 + key as u32),
        kind,
        suit: Suit::Spade,
        rank: 8,
        weapon_range: 0,
        zone,
        owner: actor,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    let tx = tx.commit();
    let revision = tx.state.revision;
    apply_command(
        tx.state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card,
                targets,
                card_targets: Vec::new(),
            },
        },
    )
}

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let choice_id = engine_api::ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "resolve rejected: {:?}",
        result.result
    );
    state
}

fn hand_count(state: &GameState, player: PlayerId) -> u32 {
    state
        .zones
        .find(ZoneKind::Hand, player)
        .and_then(|zone| state.zones.get(zone).map(|z| z.cards.len() as u32))
        .unwrap_or(0)
}

fn max_hp_of(state: &GameState, player: PlayerId) -> i32 {
    state.players.get(player).unwrap().max_hp as i32
}

#[test]
fn package_publishes_yisuan_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == YISUAN
            && trigger.event == "card_resolved"
            && trigger.filter == "owner_source_trick_resolved_own_play_phase_once_per_turn"
            && trigger.optional
    }));
    assert!(!package
        .modifier_index
        .iter()
        .any(|modifier| modifier.skill_def_id == YISUAN && modifier.target == "hand_limit"));
}

#[test]
fn yisuan_trick_resolution_gains_cards_at_max_hp_cost() {
    let (state, owner, other) = started(2_026_090_801);
    let state = strip_and_arm(state, owner);
    let max_hp = max_hp_of(&state, owner);

    // 出无中生有（普通锦囊，无目标）——结算后实体牌进弃牌堆。
    let (state, played) = play_card(state, owner, CardKind::TrickWuzhong, Vec::new(), 1);
    assert!(matches!(played.result, CommandResult::Accepted { .. }), "play rejected: {:?}", played.result);

    // 无中生有先结算摸 2（含可能的抽牌选择），然后翼算确认弹出。
    let mut state = state;
    for key in 2..8 {
        if state.resolution.continuation_kind
            == engine_core::state::ContinuationKind::OptionalTrigger
        {
            break;
        }
        if !state.has_pending_choice() {
            break;
        }
        state = resolve(state, owner, vec![], key);
    }
    assert_eq!(
        state.resolution.continuation_kind,
        engine_core::state::ContinuationKind::OptionalTrigger,
        "trick resolution must open the yisuan confirm"
    );
    let hand_before = hand_count(&state, owner);
    let state = resolve(state, owner, vec![1], 8);

    // 减 1 体力上限 + 获得弃牌堆中的此【无中生有】实体牌。
    assert_eq!(max_hp_of(&state, owner), max_hp - 1, "loses one max hp");
    assert_eq!(hand_count(&state, owner), hand_before + 1, "gains the resolved trick card");
}

#[test]
fn yisuan_second_trick_same_turn_does_not_refire() {
    let (state, owner, other) = started(2_026_090_802);
    let state = strip_and_arm(state, owner);
    let max_hp = max_hp_of(&state, owner);

    // 第一张锦囊：确认发动。
    let (state, played) = play_card(state, owner, CardKind::TrickWuzhong, Vec::new(), 1);
    assert!(matches!(played.result, CommandResult::Accepted { .. }));
    let mut state = state;
    for key in 2..8 {
        if state.resolution.continuation_kind
            == engine_core::state::ContinuationKind::OptionalTrigger
        {
            break;
        }
        if !state.has_pending_choice() {
            break;
        }
        state = resolve(state, owner, vec![], key);
    }
    let state = resolve(state, owner, vec![1], 8);
    assert_eq!(max_hp_of(&state, owner), max_hp - 1);

    // 第二张锦囊：每回合限一次，不再弹确认。
    let (state, played) = play_card(state, owner, CardKind::TrickWuzhong, Vec::new(), 11);
    assert!(matches!(played.result, CommandResult::Accepted { .. }));
    let mut state = state;
    for key in 12..16 {
        if !state.has_pending_choice() {
            break;
        }
        state = resolve(state, owner, vec![], key);
    }
    assert_ne!(
        state.resolution.continuation_kind,
        engine_core::state::ContinuationKind::OptionalTrigger,
        "second trick in the same play phase must not refire yisuan"
    );
    assert_eq!(max_hp_of(&state, owner), max_hp - 1, "max hp unchanged by the second use");
}
