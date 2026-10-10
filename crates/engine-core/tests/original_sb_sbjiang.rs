//! 第一百五十七批：sb「激昂」（界孙策 sbjiang）主臂零新原语重写。
//! 上游：使用【决斗】或红色【杀】指定目标后，或成为【决斗】或红色【杀】的目标后，
//! 可以摸一张牌。实现：card_used + 既有过滤器 owner_source_or_targeted_by_juedou_or_red_sha
//! （与 shenhua jiang 经典激昂同面）→ draw_cards(self, 1)。
//! 近似：group 子技 sbjiang_add / sbjiang_qiben 未建模（登记债）。

use engine_api::{
    CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const SBJIANG: u32 = 4_242_421_054;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sb/sb.npack.json");

fn started(seed: u64) -> (GameState, PlayerId, Vec<PlayerId>) {
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
    let others: Vec<_> = state
        .players
        .iter_live()
        .filter(|player| player.id != owner)
        .map(|player| player.id)
        .collect();
    (state, owner, others)
}

fn strip_skills_and_hands(state: GameState) -> GameState {
    let mut tx = Transaction::begin(state);
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
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

fn add_skill(state: GameState, owner: PlayerId, def: u32) -> GameState {
    let mut tx = Transaction::begin(state);
    run_effects(&mut tx, vec![EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    }])
    .unwrap();
    tx.commit().state
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

/// Inject a card of the given kind/suit into `actor`'s hand and play it at `targets`.
fn play_card(
    state: GameState,
    actor: PlayerId,
    kind: CardKind,
    suit: engine_api::Suit,
    targets: Vec<PlayerId>,
    key: u64,
) -> GameState {
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().turn.current = actor;
    let zone = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(92_100),
        kind,
        suit,
        rank: 9,
        weapon_range: 0,
        zone,
        owner: actor,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    let tx = tx.commit();
    let (state, played) = apply_command(
        tx.state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card,
                targets,
                card_targets: Vec::new(),
            },
        },
    );
    assert!(
        matches!(played.result, CommandResult::Accepted { .. }),
        "play rejected: {:?}",
        played.result
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

#[test]
fn package_publishes_sbjiang_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == SBJIANG
            && trigger.event == "card_used"
            && trigger.filter == "owner_source_or_targeted_by_juedou_or_red_sha"
            && trigger.optional
    }));
    assert!(!package
        .modifier_index
        .iter()
        .any(|modifier| modifier.skill_def_id == SBJIANG && modifier.target == "hand_limit"));
}

#[test]
fn sbjiang_draws_when_owner_uses_red_sha() {
    let (state, owner, others) = started(2_026_090_501);
    let state = strip_skills_and_hands(state);
    let state = add_skill(state, owner, SBJIANG);

    // 出红【杀】指定目标：主臂触发，确认后摸一张。
    let state = play_card(state, owner, CardKind::Sha, engine_api::Suit::Diamond, vec![others[0]], 1);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger,
        "owner using red sha must open the sbjiang confirm"
    );
    let state = resolve(state, owner, vec![1], 2);

    // 手牌从 0（打出后）经激昂摸 1。
    assert_eq!(hand_count(&state, owner), 1, "sbjiang draws one after confirm");
}

#[test]
fn sbjiang_draws_when_owner_targeted_by_red_sha() {
    let (state, owner, others) = started(2_026_090_502);
    let state = strip_skills_and_hands(state);
    let state = add_skill(state, owner, SBJIANG);

    // 他人对 owner 出红【杀】：owner 为目标侧触发，确认后摸一张。
    let state = play_card(state, others[1], CardKind::Sha, engine_api::Suit::Diamond, vec![owner], 1);
    // 目标侧可能先进入闪响应窗；将杀结算推进到激昂可选确认（逐步 resolve）。
    let mut state = state;
    for key in 2..8 {
        if state.resolution.continuation_kind == ContinuationKind::OptionalTrigger {
            break;
        }
        if !state.has_pending_choice() {
            break;
        }
        state = resolve(state, owner, vec![], key);
    }
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger,
        "being targeted by red sha must open the sbjiang confirm"
    );
    let state = resolve(state, owner, vec![1], 8);
    assert_eq!(hand_count(&state, owner), 1, "sbjiang draws one after confirm");
}

#[test]
fn sbjiang_not_triggered_by_black_sha() {
    let (state, owner, others) = started(2_026_090_503);
    let state = strip_skills_and_hands(state);
    let state = add_skill(state, owner, SBJIANG);

    // 黑【杀】不过滤：无可选触发。
    let state = play_card(state, owner, CardKind::Sha, engine_api::Suit::Spade, vec![others[0]], 1);
    assert_ne!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger,
        "black sha must not open the sbjiang confirm"
    );
    assert_eq!(hand_count(&state, owner), 0);
}
