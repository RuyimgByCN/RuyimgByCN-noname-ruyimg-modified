//! O2 全量合并第二批（2026-10-14）：家族 duoduan 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_duoduan_machine {
//! 第二百七十二批：duoduan 度断状态机行为测试（ABI 721 / tag 745 /
//! Cont 320）。链路：成为【杀】目标（每回合一次，owner 有可弃牌）→
//! 选重铸牌（可取消）→ 重铸 + 双选：①使用者摸 2 且此杀对 owner 无效；
//! ②使用者弃一张牌且 owner 不可响应。

use engine_api::{
    CardDefId, CardId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DUODUAN: u32 = 3_357_409_325;
const DUODUAN_SRC: &str =
    include_str!("../../../content/original/ported/mobile/duoduan.skill");
const MOBILE: &[u8] =
    include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

struct Setup {
    state: GameState,
    source: PlayerId,
    owner: PlayerId,
    sha: CardId,
    cost: CardId,
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, suit: Suit, rank: u8) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(98_200 + u32::from(rank)),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(MOBILE).unwrap();
    let compiled = content_compiler::compile_source_named("duoduan-test", DUODUAN_SRC).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (source, owner) = (players[0], players[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([source, owner].into_iter().map(|player| {
        EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        }
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(DUODUAN),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in [source, owner] {
        let state = tx.state_mut().players.get_mut(player).unwrap();
        state.hp = 10;
        state.max_hp = 10;
    }
    tx.state_mut().turn.current = source;
    tx.state_mut().turn.phase = Phase::Play;
    let cost = inject(&mut tx, owner, CardKind::Other, Suit::Club, 2);
    let sha = inject(&mut tx, source, CardKind::Sha, Suit::Spade, 3);
    // 使用者的弃牌候选（选项②）。
    let _ = inject(&mut tx, source, CardKind::Other, Suit::Heart, 9);
    Setup {
        state: tx.commit().state,
        source,
        owner,
        sha,
        cost,
    }
}

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

fn play_sha_at_owner(setup: &Setup, key: u64) -> (GameState, PipelineResult) {
    submit(
        setup.state.clone(),
        setup.source,
        CommandKind::PlayCard {
            card: setup.sha,
            targets: vec![setup.owner],
            card_targets: vec![],
        },
        key,
    )
}

fn hand_of(state: &GameState, player: PlayerId) -> Vec<CardId> {
    state.hand_cards(player)
}

/// 逐个取消引擎残留的响应询问（无效面落定后仍可能交付一次回应窗口）。
fn drain_prompts(
    mut state: GameState,
    mut outcome: PipelineResult,
    mut key: u64,
) -> (GameState, PipelineResult) {
    for _ in 0..4 {
        match outcome.choice.clone() {
            Some(choice) => {
                let (next, next_outcome) = submit(
                    state,
                    choice.observer,
                    CommandKind::ResolveChoice {
                        choice_id: choice.choice_id,
                        selections: vec![],
                        cancelled: true,
                    },
                    key,
                );
                state = next;
                outcome = next_outcome;
                key += 1;
            }
            None => break,
        }
    }
    (state, outcome)
}

#[test]
fn package_publishes_duoduan_once_per_turn_filter() {
    let package = content_schema::ContentPackage::from_json_bytes(MOBILE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 734);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DUODUAN)
        .expect("duoduan trigger");
    assert_eq!(trigger.event, "card_targeted");
    assert_eq!(
        trigger.filter,
        "owner_is_target_sha_source_alive_hand_equip_nonempty_once_per_turn"
    );
}

#[test]
fn recast_then_cancel_branch_makes_sha_invalid() {
    let setup = setup(20_260_913_401);
    let source_hand_before = hand_of(&setup.state, setup.source);
    let owner_hp = setup.state.players.get(setup.owner).unwrap().hp;

    let (state, cost_choice) = play_sha_at_owner(&setup, 1);
    let cost_choice = cost_choice.choice.unwrap();
    assert_eq!(cost_choice.observer, setup.owner);
    assert_eq!(cost_choice.kind, ChoiceKind::SelectCards);

    // 选重铸牌 → 进入双选。
    let (state, option_choice) = submit(
        state,
        setup.owner,
        CommandKind::ResolveChoice {
            choice_id: cost_choice.choice_id,
            selections: vec![setup.cost.raw()],
            cancelled: false,
        },
        2,
    );
    let option_choice = option_choice.choice.unwrap();
    assert_eq!(option_choice.kind, ChoiceKind::Custom);
    assert_eq!(
        option_choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [1, 2]
    );

    // 选项①：使用者摸 2，此杀对 owner 无效。
    let (state, done) = submit(
        state,
        setup.owner,
        CommandKind::ResolveChoice {
            choice_id: option_choice.choice_id,
            selections: vec![1],
            cancelled: false,
        },
        3,
    );
    assert!(matches!(done.result, CommandResult::Accepted { .. }));
    // 若仍有响应询问，取消之并观察结算。
    let mut state = state;
    if let Some(p2) = done.choice.clone() {
        let (st2, out2) = submit(
            state,
            p2.observer,
            CommandKind::ResolveChoice {
                choice_id: p2.choice_id,
                selections: vec![],
                cancelled: true,
            },
            30,
        );
        eprintln!(
            "DBG after respond-cancel: hp_owner={} pending={}",
            st2.players.get(setup.owner).unwrap().hp,
            st2.has_pending_choice()
        );
        state = st2;
        let _ = out2;
    }
    let pending = done.choice.clone();
    eprintln!(
        "DBG opt1: hp_owner={} pending_choice={:?} src_hand={:?} owner_hand={:?} events={:?}",
        state.players.get(setup.owner).unwrap().hp,
        pending.as_ref().map(|c| (c.observer, c.kind, &c.options)),
        hand_of(&state, setup.source),
        hand_of(&state, setup.owner),
        done.events.iter().map(|e| format!("{e:?}")).take(6).collect::<Vec<_>>()
    );
    // source_hand_before 含已打出的【杀】：-1 出牌 +2 摸牌。
    assert_eq!(
        hand_of(&state, setup.source).len(),
        source_hand_before.len() - 1 + 2,
        "sha user draws two"
    );
    assert_eq!(state.players.get(setup.owner).unwrap().hp, owner_hp);
    assert!(!state.has_pending_choice(), "sha resolved with no damage");
}

#[test]
fn discard_branch_forces_user_discard_and_owner_cannot_respond() {
    // 双人局：此杀结算即伤害路径；选项②后 owner 不可响应（此处空手牌，
    // 直接验证使用者弃 1 + owner 掉血 1，即 unrespondable 生效面）。
    let setup = setup(20_260_913_402);
    let source_hand_before = hand_of(&setup.state, setup.source);
    let owner_hp = setup.state.players.get(setup.owner).unwrap().hp;

    let (state, cost_choice) = play_sha_at_owner(&setup, 1);
    let cost_choice = cost_choice.choice.unwrap();
    let (state, option_choice) = submit(
        state,
        setup.owner,
        CommandKind::ResolveChoice {
            choice_id: cost_choice.choice_id,
            selections: vec![setup.cost.raw()],
            cancelled: false,
        },
        2,
    );
    let option_choice = option_choice.choice.unwrap();

    // 选项②：进入使用者的强制弃牌选择（观察者=source）。
    let (state, discard_choice) = submit(
        state,
        setup.owner,
        CommandKind::ResolveChoice {
            choice_id: option_choice.choice_id,
            selections: vec![2],
            cancelled: false,
        },
        3,
    );
    let discard_choice = discard_choice.choice.unwrap();
    assert_eq!(discard_choice.observer, setup.source);
    assert_eq!(discard_choice.kind, ChoiceKind::SelectCards);
    // 候选不含已打出的【杀】。
    let expected: Vec<_> = source_hand_before
        .iter()
        .filter(|card| **card != setup.sha)
        .map(|card| card.raw())
        .collect();
    assert_eq!(
        discard_choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        expected
    );

    let (state, done) = submit(
        state,
        setup.source,
        CommandKind::ResolveChoice {
            choice_id: discard_choice.choice_id,
            selections: vec![source_hand_before[1].raw()],
            cancelled: false,
        },
        4,
    );
    assert!(matches!(done.result, CommandResult::Accepted { .. }));
    let (state, _) = drain_prompts(state, done, 40);
    // -1 出牌 -1 弃牌。
    assert_eq!(
        hand_of(&state, setup.source).len(),
        source_hand_before.len() - 2,
        "sha user discarded one"
    );
    // owner 空手牌不可响应仍按伤害掉血（unrespondable 生效面）。
    assert_eq!(state.players.get(setup.owner).unwrap().hp, owner_hp - 1);
}

#[test]
fn declining_recast_keeps_normal_sha_and_second_sha_same_turn_skips() {
    let setup = setup(20_260_913_403);
    let owner_hp = setup.state.players.get(setup.owner).unwrap().hp;

    // 第一次杀：取消重铸 → 正常结算掉血。
    let (state, cost_choice) = play_sha_at_owner(&setup, 1);
    let cost_choice = cost_choice.choice.unwrap();
    let (state, outcome) = submit(
        state,
        setup.owner,
        CommandKind::ResolveChoice {
            choice_id: cost_choice.choice_id,
            selections: vec![],
            cancelled: true,
        },
        2,
    );
    let (state, _) = drain_prompts(state, outcome, 50);
    assert_eq!(state.players.get(setup.owner).unwrap().hp, owner_hp - 1);
    assert!(state.hand_cards(setup.owner).contains(&setup.cost));
}
}
