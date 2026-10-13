//! O2 全量合并第二批（2026-10-14）：家族 compare 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_compare_rewrite_batch {
//! Golden coverage for small legacy-JS rewrites that reuse the shared compare and choice paths.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

/// Compare-rewrite flows nest run_effects several levels deep with large frames;
/// run each on a dedicated thread with a bigger stack.
fn run_big_stack(body: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .stack_size(16 << 20)
        .spawn(body)
        .unwrap()
        .join()
        .unwrap();
}

const JYZHOUJI: u32 = 3_583_953_669;
const JYZHOUJI_RESULT: u32 = 4_259_159_799;
const HANNAN_WIN: u32 = 3_519_200_299;
const HANNAN_LOSE: u32 = 4_002_168_244;
const REJIANXIONG_OLD: u32 = 2_626_908_745;
const YUIKO_FENGLUN: u32 = 3_729_037_306;
const YUIKO_FENGLUN_BUFF: u32 = 2_522_572_214;
const SONGSHU: u32 = 3_706_775_979;
const SONGSHU_LOSE: u32 = 2_373_662_584;
const SHUIMENG: u32 = 2_224_403_544;
const SHUIMENG_WIN: u32 = 3_468_190_177;
const SHUIMENG_LOSE: u32 = 2_380_613_144;
const JYBOYAN_WIN: u32 = 2_251_726_731;
const JYBOYAN_LOSE: u32 = 2_151_014_347;
const JYBOYAN_EFFECT: u32 = 3_400_903_940;
const REXIANZHEN_WIN: u32 = 3_751_763_590;
const REXIANZHEN_LOSE: u32 = 4_037_147_006;
const REXIANZHEN_SHA: u32 = 3_222_374_511;
const SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/offline/jyzhouji.skill"),
    include_str!("../../../content/original/ported/offline/jyboyan.skill"),
    include_str!("../../../content/original/ported/mobile/hannan.skill"),
    include_str!("../../../content/original/ported/refresh/rejianxiong_old.skill"),
    include_str!("../../../content/original/ported/refresh/rexianzhen.skill"),
    include_str!("../../../content/original/ported/refresh/rexianzhen2.skill"),
    include_str!("../../../content/original/ported/refresh/rexianzhen3.skill"),
    include_str!("../../../content/original/ported/refresh/rexianzhen4.skill"),
    include_str!("../../../content/original/ported/key/yuiko_fenglun.skill"),
    include_str!("../../../content/original/ported/key/yuiko_fenglun2.skill"),
    include_str!("../../../content/original/ported/sp/shuimeng.skill"),
    include_str!("../../../content/original/ported/sp2/songshu.skill"),
);

fn setup() -> (GameState, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("rewrite-batch", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_731, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let seats = tx.state().players.alive_by_seat();
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
    run_effects(&mut tx, effects).unwrap();
    for player in &seats {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 100;
        participant.max_hp = 100;
    }
    (tx.commit().state, seats[0], seats[1])
}

fn dispatch_compare(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    winner: PlayerId,
) -> GameState {
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::HandCompareFinished {
            player: owner,
            target,
            player_card: CardId::NONE,
            target_card: CardId::NONE,
            winner,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

fn add_skill(state: GameState, player: PlayerId, skill_def: u32) -> GameState {
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(skill_def),
        }],
    )
    .unwrap();
    tx.commit().state
}

fn add_bound_skill(
    state: GameState,
    player: PlayerId,
    skill_def: u32,
    bound_target: PlayerId,
) -> GameState {
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddBoundSkill {
            player,
            skill_def: SkillDefId::new(skill_def),
            bound_target,
        }],
    )
    .unwrap();
    tx.commit().state
}

fn has_skill(state: &GameState, player: PlayerId, skill_def: u32) -> bool {
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|skill| skill.def.raw() == skill_def)
        })
}

#[test]
fn compiled_contracts_keep_dynamic_hp_limit_and_one_shot_helpers() {
    run_big_stack(|| {
        let package = content_compiler::compile_source_named("rewrite-batch", SOURCE).unwrap();
        let active = package
            .active_index
            .iter()
            .find(|active| active.skill_def_id == JYZHOUJI)
            .unwrap();
        assert_eq!(active.uses_per_turn_source.as_deref(), Some("owner_hp"));
        assert!(active.targets.as_ref().unwrap().can_compare);
        assert!(package.trigger_index.iter().any(|trigger| {
            trigger.skill_def_id == JYZHOUJI_RESULT
                && trigger.event == "hand_compare_finished"
                && trigger.filter == "owner_is_target_once"
        }));
        assert!(package.trigger_index.iter().any(|trigger| {
            trigger.skill_def_id == YUIKO_FENGLUN_BUFF
                && trigger.event == "phase_play_end"
                && trigger.filter == "owner_is_target"
        }));
        assert!(package
            .active_index
            .iter()
            .any(|active| active.skill_def_id == YUIKO_FENGLUN && active.uses_per_turn == Some(1)));
        assert!(package.trigger_index.iter().any(|trigger| {
            trigger.skill_def_id == SHUIMENG
                && trigger.event == "phase_play_end"
                && trigger.filter == "owner_is_target_can_compare_other"
                && trigger.optional
        }));
    });
}

#[test]
fn jyzhouji_draws_only_when_the_other_player_strictly_wins() {
    run_big_stack(|| {
        let (state, owner, target) = setup();
        let state = add_skill(state, owner, JYZHOUJI);
        let state = add_skill(state, owner, JYZHOUJI_RESULT);
        // 平局：无人摸牌。
        let state = dispatch_compare(state, owner, target, PlayerId::NONE);
        assert_eq!(state.hand_count(target), 0);
        // 目标严格获胜：其摸两张。
        let state = add_skill(state, owner, JYZHOUJI_RESULT);
        let state = dispatch_compare(state, owner, target, target);
        assert_eq!(state.hand_count(target), 2);
        assert_eq!(state.hand_count(owner), 0);
    });
}

#[test]
fn hannan_deals_damage_from_the_strict_winner_and_ignores_ties() {
    run_big_stack(|| {
        let (state, owner, target) = setup();
        // 发起者严格获胜：其（技能持有者）对目标造成 1 点伤害。
        let state = add_skill(state, owner, HANNAN_WIN);
        let state = dispatch_compare(state, owner, target, owner);
        assert_eq!(state.players.get(target).unwrap().hp, 99);
        assert_eq!(state.players.get(owner).unwrap().hp, 100);
        // 平局：无伤害。
        let state = add_skill(state, owner, HANNAN_LOSE);
        let state = dispatch_compare(state, owner, target, PlayerId::NONE);
        assert_eq!(state.players.get(target).unwrap().hp, 99);
        assert_eq!(state.players.get(owner).unwrap().hp, 100);
        // 目标严格获胜：目标对发起者造成 1 点伤害。
        let state = add_skill(state, owner, HANNAN_LOSE);
        let state = dispatch_compare(state, owner, target, target);
        assert_eq!(state.players.get(target).unwrap().hp, 99);
        assert_eq!(state.players.get(owner).unwrap().hp, 99);
    });
}

#[test]
fn songshu_non_win_draws_both_players_and_disables_only_the_parent_for_the_turn() {
    run_big_stack(|| {
        let (state, owner, target) = setup();
        let state = add_skill(state, owner, SONGSHU);
        let state = add_skill(state, owner, SONGSHU_LOSE);
        // 未严格获胜：双方各摸两张，结果技能移除，父技能仍保留（本阶段禁用）。
        let state = dispatch_compare(state, owner, target, PlayerId::NONE);
        assert_eq!(state.hand_count(owner), 2);
        assert_eq!(state.hand_count(target), 2);
        assert!(!has_skill(&state, owner, SONGSHU_LOSE));
        assert!(has_skill(&state, owner, SONGSHU));
    });
}

#[test]
fn shuimeng_uses_wuzhong_on_a_strict_win_and_guohe_otherwise() {
    run_big_stack(|| {
        let (state, owner, target) = setup();
        // 严格获胜：对己方使用无中生有（摸两张）。
        let state = add_skill(state, owner, SHUIMENG_WIN);
        let state = dispatch_compare(state, owner, target, owner);
        assert_eq!(state.hand_count(owner), 2);
        // 未获胜：对方对自己使用过河拆桥（弃置一张牌）。
        let state = add_skill(state, owner, SHUIMENG_LOSE);
        let mut tx = Transaction::begin(state);
        let mut fires = TriggerFireRecord::new();
        let effects = dispatch_for_events(
            &mut tx,
            &[DomainEvent::HandCompareFinished {
                player: owner,
                target,
                player_card: CardId::NONE,
                target_card: CardId::NONE,
                winner: target,
            }],
            &mut fires,
        )
        .unwrap();
        match run_effects(&mut tx, effects) {
            Ok(_) => {}
            Err(EffectError::ChoiceRequested) => {}
            Err(e) => panic!("guohe error: {e:?}"),
        }
        let state = tx.commit().state;
        // 过河拆桥的弃牌选择：弃置持有者一张手牌。
        assert!(state.has_pending_choice());
        let victim = state.hand_cards(owner)[0];
        let (state, result) = resolve_choice(state, target, victim.raw(), 50_101);
        assert!(matches!(result.result, CommandResult::Accepted { .. }));
        assert_eq!(state.hand_count(owner), 1);
    });
}

#[test]
fn jyboyan_restricts_the_loser_and_restricts_both_players_on_a_tie() {
    run_big_stack(|| {
        let (state, owner, target) = setup();
        // 发起者严格获胜：目标（输家）获得禁用手牌效果并摸两张。
        let state = add_skill(state, owner, JYBOYAN_WIN);
        let state = dispatch_compare(state, owner, target, owner);
        assert!(has_skill(&state, target, JYBOYAN_EFFECT));
        assert_eq!(state.hand_count(target), 2);
        assert_eq!(state.hand_count(owner), 0);
        // 平局：双方均获禁用手牌效果并各摸两张。
        let state = add_skill(state, owner, JYBOYAN_LOSE);
        let state = dispatch_compare(state, owner, target, PlayerId::NONE);
        assert!(has_skill(&state, owner, JYBOYAN_EFFECT));
        assert!(has_skill(&state, target, JYBOYAN_EFFECT));
        assert_eq!(state.hand_count(owner), 2);
        assert_eq!(state.hand_count(target), 4);
    });
}

#[test]
fn rexianzhen_keeps_sha_only_when_the_initiators_compare_card_is_sha() {
    run_big_stack(|| {
        let (state, owner, target) = setup();
        // 发起拼点牌为【杀】：保留【杀】效果技能加入。
        let state = add_skill(state, owner, REXIANZHEN_WIN);
        let state = dispatch_compare_with_card_kind(state, owner, target, owner, CardKind::Sha);
        assert!(has_skill(&state, owner, REXIANZHEN_SHA));
        // 发起拼点牌非【杀】：不加入（重新开局验证）。
        let (state, owner, target) = setup();
        let state = add_skill(state, owner, REXIANZHEN_WIN);
        let state = dispatch_compare_with_card_kind(state, owner, target, owner, CardKind::Tao);
        assert!(!has_skill(&state, owner, REXIANZHEN_SHA));
    });
}

fn inject_processing_kind(state: &mut GameState, kind: CardKind, def: u32) -> CardId {
    let zone = state
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone,
        owner: PlayerId::NONE,
        face_up: true,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn dispatch_compare_with_card_kind(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    winner: PlayerId,
    owner_kind: CardKind,
) -> GameState {
    let mut tx = Transaction::begin(state);
    let owner_card = inject_processing_kind(tx.state_mut(), owner_kind, 99_101);
    let target_card = inject_processing_kind(tx.state_mut(), CardKind::Shan, 99_102);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::HandCompareFinished {
            player: owner,
            target,
            player_card: owner_card,
            target_card,
            winner,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

fn inject_processing(state: &mut GameState) -> CardId {
    inject_processing_kind(state, CardKind::Sha, 99_001)
}

fn resolve_choice(
    state: GameState,
    actor: PlayerId,
    selection: u32,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![selection],
                cancelled: false,
            },
        },
    )
}

#[test]
#[test]
fn old_jianxiong_offers_draw_or_gain_only_for_a_processing_card() {
    run_big_stack(|| {
        let (state, owner, _target) = setup();
        let state = add_skill(state, owner, REJIANXIONG_OLD);
        // 伤害牌在处理区：提供摸一张或获得伤害牌的选择。
        let card = {
            let mut tx = Transaction::begin(state.clone());
            let card = inject_processing(tx.state_mut());
            match run_effects(
                &mut tx,
                vec![EffectRequest::CommitDamage {
                    damage_id: 0,
                    source: PlayerId::NONE,
                    target: owner,
                    amount: 1,
                    base_amount: 1,
                    is_lose_hp: false,
                    nature: 0,
                    card,
                    card_kind: 0,
                    card_flags: 0,
                }],
            ) {
                Ok(_) => {}
                Err(EffectError::ChoiceRequested) => {}
                Err(e) => panic!("damage error: {e:?}"),
            }
            let state = tx.commit().state;
            // 可选触发确认（接受），然后选择获得伤害牌。
            let (state, result) = resolve_choice(state, owner, 1, 50_001);
            assert!(matches!(result.result, CommandResult::Accepted { .. }));
            let (state, result) = resolve_choice(state, owner, 2, 50_002);
            assert!(matches!(result.result, CommandResult::Accepted { .. }));
            // 选择获得伤害牌：处理区的【杀】进入手牌。
            assert!(state.hand_cards(owner).contains(&card));
            state
        };
        // 伤害牌不在处理区：直接摸一张，无选择。
        let state = add_skill(state, owner, REJIANXIONG_OLD);
        let mut tx = Transaction::begin(state);
        let mut fires = TriggerFireRecord::new();
        let dispatched = dispatch_for_events(
            &mut tx,
            &[DomainEvent::DamageDealt {
                source: PlayerId::NONE,
                target: owner,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
            &mut fires,
        );
        match dispatched {
            Ok(effects) => match run_effects(&mut tx, effects) {
                Ok(_) => {}
                Err(EffectError::ChoiceRequested) => {}
                Err(e) => panic!("damage error: {e:?}"),
            },
            Err(EffectError::ChoiceRequested) => {}
            Err(e) => panic!("dispatch error: {e:?}"),
        }
        let state = tx.commit().state;
        let (state, result) = resolve_choice(state, owner, 1, 50_003);
        assert!(matches!(result.result, CommandResult::Accepted { .. }));
        // 伤害牌不在处理区：不提供摸牌/获得选择。
        assert!(!state.has_pending_choice());
    });
}
}
