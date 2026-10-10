//! 第一百六十批：sp2「辅袁」（辛评 fuyuan）零新原语重写。
//! 上游：回合外使用或打出/响应一张牌后，若当前回合角色手牌数少于你，其摸一张牌，
//! 否则你摸一张牌。实现：card_used/card_responded 双臂 + 既有过滤器
//! owner_source_outside_turn_card_used_or_responded → hand_count(current_phase_player())
//! 比较分支决定 draw_cards 目标（current_phase_player() 首用于效果参数位）。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const FUYUAN: u32 = 2_389_825_863;
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
    let current = state.turn.current;
    let owner = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != current)
        .unwrap();
    (state, current, owner)
}

fn strip_skills_and_hands(state: GameState) -> GameState {
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
        .collect();
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

fn add_fuyuan(state: GameState, owner: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    run_effects(&mut tx, vec![EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(FUYUAN),
    }])
    .unwrap();
    tx.commit().state
}

fn inject_hand(tx: &mut Transaction, owner: PlayerId, kind: CardKind, def: u32) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Diamond,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn play_sha(
    state: GameState,
    actor: PlayerId,
    target: PlayerId,
    key: u64,
) -> (GameState, PipelineResult) {
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = Phase::Play;
    let sha = inject_hand(&mut tx, actor, CardKind::Sha, 98_400 + key as u32);
    let tx = tx.commit();
    let revision = tx.state.revision;
    apply_command(
        tx.state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![target],
                card_targets: Vec::new(),
            },
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> GameState {
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

#[test]
fn package_publishes_fuyuan_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    for event in ["card_used", "card_responded"] {
        assert!(
            package.trigger_index.iter().any(|trigger| {
                trigger.skill_def_id == FUYUAN
                    && trigger.event == event
                    && trigger.filter == "owner_source_outside_turn_card_used_or_responded"
                    && trigger.optional
            }),
            "fuyuan must publish a {event} arm"
        );
    }
    assert!(!package
        .modifier_index
        .iter()
        .any(|modifier| modifier.skill_def_id == FUYUAN && modifier.target == "hand_limit"));
}

#[test]
fn fuyuan_outside_turn_response_fewer_hand_phase_player_draws() {
    let (state, current, owner) = started(2_026_090_601);
    let state = strip_skills_and_hands(state);
    let state = add_fuyuan(state, owner);

    // current 回合：其对 owner 出【杀】；owner 手牌 = 闪 + 填充（两张）。
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = current;
    let shan = inject_hand(&mut tx, owner, CardKind::Shan, 98_501);
    inject_hand(&mut tx, owner, CardKind::Tao, 98_502);
    let state = tx.commit().state;

    let (state, played) = play_sha(state, current, owner, 1);
    assert!(matches!(played.result, CommandResult::Accepted { .. }));
    // owner 响应窗：选【闪】。
    let shan_option = played
        .choice
        .as_ref()
        .expect("shan window opens")
        .options
        .iter()
        .find(|option| option.token == shan.raw())
        .expect("shan option listed");
    let state = resolve(state, owner, vec![shan_option.token], 2);

    // 回合外响应成立 → 辅袁可选确认。
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger,
        "outside-turn response must open the fuyuan confirm"
    );
    // owner 剩 1 手牌 > current 的 0 → 当前回合角色（current）摸一张。
    assert_eq!(hand_count(&state, owner), 1);
    assert_eq!(hand_count(&state, current), 0);
    let state = resolve(state, owner, vec![1], 3);
    assert_eq!(hand_count(&state, current), 1, "phase player with fewer hand draws");
    assert_eq!(hand_count(&state, owner), 1);
}

#[test]
fn fuyuan_outside_turn_response_equal_hand_self_draws() {
    let (state, current, owner) = started(2_026_090_602);
    let state = strip_skills_and_hands(state);
    let state = add_fuyuan(state, owner);

    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = current;
    let shan = inject_hand(&mut tx, owner, CardKind::Shan, 98_511);
    let state = tx.commit().state;

    let (state, played) = play_sha(state, current, owner, 1);
    assert!(matches!(played.result, CommandResult::Accepted { .. }));
    let shan_option = played
        .choice
        .as_ref()
        .expect("shan window opens")
        .options
        .iter()
        .find(|option| option.token == shan.raw())
        .expect("shan option listed");
    let state = resolve(state, owner, vec![shan_option.token], 2);

    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    // 响应后双方手牌均为 0（不小于）→ owner 自己摸一张。
    assert_eq!(hand_count(&state, owner), 0);
    let state = resolve(state, owner, vec![1], 3);
    assert_eq!(hand_count(&state, owner), 1, "equal hand falls to self draw");
    assert_eq!(hand_count(&state, current), 0);
}

#[test]
fn fuyuan_not_triggered_during_own_turn() {
    let (state, _current, owner) = started(2_026_090_603);
    let state = strip_skills_and_hands(state);
    let state = add_fuyuan(state, owner);

    // owner 自己回合内出【杀】：过滤器（回合外门槛）拦截，不弹确认。
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = owner;
    let state = tx.commit().state;
    let other = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let (state, played) = play_sha(state, owner, other, 1);
    assert!(matches!(played.result, CommandResult::Accepted { .. }));
    assert_ne!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger,
        "own-turn use must not open the fuyuan confirm"
    );
}
