//! 第一百九十批：ADR 0641 §8.2 S1-B 首铸——`discard_skill_pile` 原语
//! （tag 707 / ABI 684，纯同步零选择续体）+ 首件消费者 rebiaozhao3（表召·终段）。
//! 上游：准备阶段，若有「表」：全部「表」置入弃牌堆，令一名角色摸三张牌并回复
//! 1 点体力。近似债：目标「含自己」收窄为其他；draw3+recover1 并发收窄为二选一。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::Engine;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, Zone, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const REBIAOZHAO3: u32 = 3_401_911_930;
const REBIAOZHAO: u32 = 4_210_066_284;
const SOURCE: &str = include_str!("../../../content/original/ported/mobile/rebiaozhao3.skill");
const MOBILE: &[u8] =
    include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("rebiaozhao3-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let other = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(REBIAOZHAO3),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, other)
}

fn inject_hand_card(state: GameState, owner: PlayerId, def: u32) -> (GameState, CardId) {
    let mut tx = Transaction::begin(state);
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    (tx.commit().state, card)
}

/// 直接构造 (owner, REBIAOZHAO) 技能堆并移入 n 张牌。
fn fill_pile(state: GameState, owner: PlayerId, n: usize) -> GameState {
    let mut state = state;
    for i in 0..n {
        let (s2, card) = inject_hand_card(state, owner, 9_100 + i as u32);
        state = s2;
        let mut tx = Transaction::begin(state);
        let existing = tx
            .state()
            .skill_piles
            .iter()
            .find(|p| p.owner == owner && p.skill_def == REBIAOZHAO)
            .map(|p| p.zone);
        let zone = match existing {
            Some(zone) => zone,
            None => {
                let zone = tx.state_mut().zones.alloc(Zone {
                    id: engine_api::ZoneId::NONE,
                    kind: ZoneKind::Expansion,
                    owner,
                    cards: Vec::new(),
                });
                tx.state_mut().skill_piles.push(engine_core::state::SkillPileRef {
                    owner,
                    skill_def: REBIAOZHAO,
                    zone,
                    usable_as_hand: false,
                    wants_lost_count: false,
                });
                zone
            }
        };
        let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
        tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
        tx.state_mut()
            .zones
            .get_mut(hand)
            .unwrap()
            .cards
            .retain(|c| *c != card);
        if let Some(card_state) = tx.state_mut().cards.get_mut(card) {
            card_state.zone = zone;
        }
        state = tx.commit().state;
    }
    state
}

/// forced 触发模块 dispatch 期内联执行：体内 choose 使 dispatch 返回
/// Err(ChoiceRequested)（选择已进 tx）——见批 189 ADR 0641 §8.3 补记。
fn fire_prepare(state: GameState, owner: PlayerId) -> (GameState, Result<(), EffectError>) {
    let mut tx = Transaction::begin(state);
    let outcome = match dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    ) {
        Err(EffectError::ChoiceRequested) => Err(EffectError::ChoiceRequested),
        Ok(effects) => engine_core::effect::run_effects(&mut tx, effects).map(|_| ()),
        Err(other) => panic!("unexpected dispatch error: {other:?}"),
    };
    (tx.commit().state, outcome)
}

fn resolve(state: GameState, owner: PlayerId, pick: u32, key: u64) -> GameState {
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

fn pile_cards(state: &GameState, owner: PlayerId) -> usize {
    state.skill_pile_cards(owner, REBIAOZHAO).len()
}



#[test]
fn primitive_direct_discard_drains_pile() {
    let (state, owner, _other) = setup(20_260_914_03);
    let state = fill_pile(state, owner, 2);
    assert_eq!(pile_cards(&state, owner), 2);

    // 直接驱动原语（绕开触发面）。
    let instance = state
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.owner == owner && s.def.raw() == REBIAOZHAO3)
        .map(|s| s.id)
        .unwrap();
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardSkillPile {
            player: owner,
            skill: instance,
            pile_def: REBIAOZHAO,
        }],
    )
    .unwrap();
    let state = tx.commit().state;
    assert_eq!(pile_cards(&state, owner), 0, "direct primitive drains pile");
}

#[test]
fn run_effects_applies_discard_then_choice_in_order() {
    let (state, owner, other) = setup(20_260_914_04);
    let state = fill_pile(state, owner, 1);
    let instance = state
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.owner == owner && s.def.raw() == REBIAOZHAO3)
        .map(|s| s.id)
        .unwrap();

    let mut tx = Transaction::begin(state);
    let outcome = run_effects(
        &mut tx,
        vec![
            EffectRequest::DiscardSkillPile {
                player: owner,
                skill: instance,
                pile_def: REBIAOZHAO,
            },
            EffectRequest::ChooseOtherDrawOrRecover {
                player: owner,
                skill: engine_api::SkillId::new(REBIAOZHAO3),
                draw_count: 3,
                recover_count: 1,
                times: 1,
            },
        ],
    );
    assert!(
        matches!(outcome, Err(EffectError::ChoiceRequested)),
        "choice opens after discard applied: {outcome:?}"
    );
    let state = tx.commit().state;
    assert_eq!(pile_cards(&state, owner), 0, "discard applied before choice");
    let _ = other;
}

/// 生产管线复测（已确认引擎真实缺陷，见 ADR 0641 §8.5）：
/// 触发模块 [choose_other_draw_or_recover, discard_skill_pile] 经 EndPhase
/// 命令流转，确认与目标选择正常，但 vm-pause 恢复后的弃堆**丢失**（堆保持
/// 非空）。批 193 vm 弹栈位型修复后本用例转正（生产管线全链通过），弃堆不再丢失。
#[test]
fn production_pipeline_endphase_reaches_prepare_and_drains_pile() {
    let source = format!(
        "skill probe_prod(9001004) {{\n  trigger phase_prepare owner_skill_pile_nonempty_{p} optional {{\n    effect choose_other_draw_or_recover(self, 3, 1, 1);\n    effect discard_skill_pile(self, self_skill, {p});\n  }}\n}}\n",
        p = REBIAOZHAO
    );
    let compiled = content_compiler::compile_source_named("rebiaozhao3-prod", &source).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&compiled).unwrap();
    let start = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(1),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_915_01,
            ruleset: "identity-standard".into(),
        },
    });
    assert!(matches!(start, CommandResult::Accepted { .. }));
    let state = engine.state().clone();
    let owner = state.turn.current;
    let other = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|p| *p != owner)
        .unwrap();

    let (state, _) = inject_hand_card(state, owner, 9_300);
    let mut tx = Transaction::begin(state);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(9001004),
    });
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;
    let state = fill_pile(state, owner, 2);
    assert_eq!(pile_cards(&state, owner), 2);

    let mut state = state;
    let mut key = 20_260_915_10u64;
    let mut drained = false;
    let mut last_choice: Option<engine_api::ChoiceRequest> = None;
    for _step in 0..40 {
        let pending = state.has_pending_choice();
        if pending {
            let lc = last_choice
                .clone()
                .expect("pending choice must be exposed by the opening command");
            let phase = state.turn.phase;
            let tokens: Vec<u32> = if phase == Phase::Discard {
                let n = usize::max(lc.min_count as usize, 1);
                lc.options.iter().take(n).map(|o| o.token).collect()
            } else if lc.options.len() == 2 && lc.options[0].token == 0 && lc.options[1].token == 1 {
                vec![1]
            } else if lc.options.iter().any(|o| o.token == other.raw()) && lc.min_count == 1 {
                vec![other.raw()]
            } else {
                vec![lc.options[0].token]
            };
            let choice_id = lc.choice_id;
            let revision = lc.revision;
            let actor = lc.observer;
            let (n2, result) = apply_command(
                state,
                Command {
                    actor,
                    expected_revision: ExpectedRevision(revision),
                    idempotency_key: IdempotencyKey(key),
                    kind: CommandKind::ResolveChoice {
                        choice_id,
                        selections: tokens,
                        cancelled: false,
                    },
                },
            );
            assert!(
                matches!(result.result, CommandResult::Accepted { .. }),
                "resolve rejected: {:?}",
                result.result
            );
            last_choice = result.choice.clone();
            state = n2;
            key += 1;
        } else if pile_cards(&state, owner) == 0 {
            drained = true;
            break;
        } else {
            let current = state.turn.current;
            let revision = state.revision;
            let (n2, result) = apply_command(
                state,
                Command {
                    actor: current,
                    expected_revision: ExpectedRevision(revision),
                    idempotency_key: IdempotencyKey(key),
                    kind: CommandKind::EndPhase,
                },
            );
            assert!(matches!(result.result, CommandResult::Accepted { .. }));
            last_choice = result.choice.clone();
            state = n2;
            key += 1;
        }
    }
    assert!(drained, "pile drained through production pipeline");
    assert_eq!(pile_cards(&state, owner), 0);
    assert!(!state.has_pending_choice());
}

#[test]
fn empty_pile_stays_silent() {
    let (state, owner, _other) = setup(20_260_914_02);
    let (state, outcome) = fire_prepare(state, owner);
    assert!(outcome.is_ok() && !state.has_pending_choice());
    assert_eq!(pile_cards(&state, owner), 0);
}
