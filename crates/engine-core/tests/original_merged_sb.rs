//! O2 全量合并批次 1（2026-10-14）：家族 sb 的 3 个独立测试文件逐字包裹为模块——#[test] 函数名不变、测试数量不变；测试全名获得确定性前缀（模块名=原文件名，子串过滤行为不变）；include_bytes/str 相对路径不受影响（同目录）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_sb_batch1 {
//! Golden coverage for sb（界限突破）包批次一：制衡/闭月 + 全包契约。

use engine_api::{
    Command, CommandKind, DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const SBZHIHENG: u32 = 4_242_420_001;
const SBBIYUE: u32 = 4_152_380_059;
const SBYANGWEI: u32 = 3_601_751_055;
const SOURCE_BIYUE: &str = include_str!("../../../content/original/ported/sb/sbbiyue.skill");

fn hand_count(state: &GameState, player: PlayerId) -> u32 {
    state
        .zones
        .find(ZoneKind::Hand, player)
        .map(|zone| {
            state
                .zones
                .get(zone)
                .map(|z| z.cards.len() as u32)
                .unwrap_or(0)
        })
        .unwrap_or(0)
}

fn setup(source: &str, defs: &[u32]) -> (Transaction, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("sb-batch1", source).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_803, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, other) = (seats[0], seats[1]);
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
    for def in defs {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(*def),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, other)
}

#[test]
fn package_exposes_sb_contract() {
    let committed = content_schema::ContentPackage::from_json_bytes(include_bytes!(
        "../../../content/original/ported/sb/sb.npack.json"
    ))
    .unwrap();
    assert_eq!(committed.character_index.len(), 55);
    for def in [SBZHIHENG, SBBIYUE, SBYANGWEI] {
        assert!(
            committed
                .skill_index
                .iter()
                .any(|skill| skill.skill_def_id == def),
            "skill {def} must exist"
        );
    }
    assert!(committed.character_index.iter().any(
        |character| character.name_key == "sb_sunquan" && character.skills.contains(&SBZHIHENG)
    ));
    assert!(committed.character_index.iter().any(
        |character| character.name_key == "sb_diaochan" && character.skills.contains(&SBBIYUE)
    ));
    assert!(committed.character_index.iter().any(|character| {
        character.name_key == "sb_huaxiong"
            && character.skills == [2_159_889_222, SBYANGWEI]
    }));
}

#[test]
fn sbzhiheng_discards_and_draws_equal() {
    let source = include_str!("../../../content/original/ported/sb/sbzhiheng.skill");
    let (mut tx, owner, _other) = setup(source, &[SBZHIHENG]);
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    // 给两张手牌
    for (suit, rank) in [
        (engine_api::Suit::Spade, 5u8),
        (engine_api::Suit::Heart, 6u8),
    ] {
        let card = tx.state_mut().cards.alloc(engine_core::state::Card {
            id: engine_api::CardId::NONE,
            def: engine_api::CardDefId::new(1),
            kind: engine_core::state::CardKind::Sha,
            suit,
            rank,
            weapon_range: 0,
            zone: hand,
            owner,
            face_up: false,
        });
        tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    }
    let before = hand_count(&tx.state(), owner);
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .find(|skill| {
            tx.state()
                .skills
                .get(**skill)
                .is_some_and(|instance| instance.def.raw() == SBZHIHENG)
        })
        .copied()
        .unwrap();
    let state = tx.commit().state;
    let cards: Vec<_> = state
        .zones
        .find(ZoneKind::Hand, owner)
        .map(|zone| state.zones.get(zone).unwrap().cards.clone())
        .unwrap();
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0x2410),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![],
                card_targets: cards.clone(),
            },
        },
    );
    assert!(
        matches!(result.result, engine_api::CommandResult::Accepted { .. }),
        "sbzhiheng must activate: {result:?}"
    );
    // 弃 2 摸 2（或更多）：手牌数与激活前一致或更多。
    assert!(
        hand_count(&state, owner) >= before,
        "sbzhiheng draws at least as many as discarded"
    );
    // 弃置牌进入弃牌堆。
    assert!(
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .is_some_and(|zone| {
                let z = state.zones.get(zone).unwrap();
                cards.iter().all(|card| z.cards.contains(card))
            }),
        "discarded cards go to the discard pile"
    );
}

#[test]
fn sbbiyue_draws_one_at_end_phase() {
    let source = include_str!("../../../content/original/ported/sb/sbbiyue.skill");
    let (mut tx, owner, _other) = setup(source, &[SBBIYUE]);
    let before = hand_count(&tx.state(), owner);
    let state = tx.commit().state;
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = Phase::End;
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    );
    let mut state = match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Ok(_) => tx.commit().state,
            Err(EffectError::ChoiceRequested) => tx.commit().state,
            Err(e) => panic!("run error: {e:?}"),
        },
        Err(EffectError::ChoiceRequested) => tx.commit().state,
        Err(e) => panic!("dispatch error: {e:?}"),
    };
    if state.has_pending_choice() {
        let pending = state.resolution.pending_choice_id;
        let observer = PlayerId::new(state.resolution.pending_observer);
        let (state2, result) = apply_command(
            state,
            Command {
                actor: observer,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(0x2411),
                kind: CommandKind::ResolveChoice {
                    choice_id: engine_api::ChoiceId::new(pending),
                    selections: vec![1],
                    cancelled: false,
                },
            },
        );
        assert!(
            matches!(result.result, engine_api::CommandResult::Accepted { .. }),
            "sbbiyue confirm: {result:?}"
        );
        state = state2;
    }
    assert_eq!(hand_count(&state, owner), before + 1, "sbbiyue draws one");
    let _ = SOURCE_BIYUE;
}
}

mod original_sb_sbbiyue {
//! Golden for SB Diaochan's distinct-damaged-player end-phase draw count.

use engine_api::{Command, CommandKind, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId};
use engine_core::effect::run_effects;
use engine_core::state::{turn_flag, Phase};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const SBBIYUE: u32 = 4_152_380_059;
const SOURCE: &str = include_str!("../../../content/original/ported/sb/sbbiyue.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sb/sb.npack.json");

fn prepared() -> (Transaction, Vec<PlayerId>) {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let compiled = content_compiler::compile_source_named("sb", SOURCE).unwrap();
    // 全包逻辑哈希含全部界武将技能；此处校验包内登记与编译契约一致。
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == SBBIYUE));
    assert!(compiled
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == SBBIYUE));

    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 5,
            rng_seed: 20_260_714_379,
            ruleset: "identity-standard".into(),
        },
    });
    let players: Vec<_> = engine
        .state()
        .players
        .iter_live()
        .map(|player| player.id)
        .collect();
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
        skill_def: SkillDefId::new(SBBIYUE),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        let state = tx.state_mut().players.get_mut(*player).unwrap();
        state.hp = 10;
        state.max_hp = 10;
    }
    (tx, players)
}

#[test]
fn sbbiyue_counts_distinct_damaged_players_and_caps_draw_at_four() {
    for (damaged, expected_draw) in [(vec![], 1), (vec![1, 1, 2], 3), (vec![0, 1, 2, 3, 4], 4)] {
        let (mut tx, players) = prepared();
        let owner = tx.state().turn.current;
        for index in damaged {
            run_effects(
                &mut tx,
                vec![EffectRequest::DealDamage {
                    source: PlayerId::NONE,
                    target: players[index],
                    amount: 1,
                    is_lose_hp: false,
                    nature: 0,
                }],
            )
            .unwrap();
        }
        let hand = tx.state().hand_count(owner);
        run_effects(
            &mut tx,
            vec![EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::End.to_u8(),
            }],
        )
        .unwrap();
        assert_eq!(tx.state().hand_count(owner), hand + expected_draw);
    }
}

#[test]
fn damaged_player_history_resets_when_the_turn_owner_changes() {
    let (mut tx, players) = prepared();
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: players[1],
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().turn.damaged_player_count(), 1);

    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: players[1],
            phase: Phase::Prepare.to_u8(),
        }],
    )
    .unwrap();
    assert_eq!(tx.state().turn.flag(turn_flag::DAMAGED_PLAYERS), 0);
}
}

mod original_sb_sbjiang {
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
}
