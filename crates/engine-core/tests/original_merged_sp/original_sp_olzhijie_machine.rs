//! Golden for SP 直谏 olzhijie (ABI 738, tag 759, Cont 329):
//! 锁定技——一名角色的结束阶段，若你本回合使用/打出过牌、且本回合被选
//! 择为牌的目标、且射程内有角色：选射程内一名角色对其造成 1 点伤害。
//! 真管道驱动（dispatch_for_events PhaseEnded）。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const OLZHIJIE: u32 = 3_419_173_958;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/sp/sp.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn started(seed: u64) -> (GameState, Vec<PlayerId>) {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package())
        .unwrap();
    let players = tx.state().players.alive_by_seat();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    (tx.commit().state, players)
}

fn grant(state: &mut GameState, player: PlayerId, def: u32) {
    let mut tx = Transaction::begin(state.clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
    *state = tx.commit().state;
}

fn inject_hand(state: &mut GameState, owner: PlayerId, tag: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(95_000 + tag),
        kind: CardKind::Tao,
        suit: engine_api::Suit::Club,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = engine_api::ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

fn end_phase(state: &mut GameState, current: PlayerId) -> Result<Vec<EffectRequest>, EffectError> {
    let mut tx = Transaction::begin(state.clone());
    // 上游 phaseJieshuBegin = 结束阶段开始 = TurnStateChanged(End)。
    let effects = dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    )?;
    // open_choice 打开时以 Err(ChoiceRequested) 短路——正常信号。
    let dispatch_result = run_effects(&mut tx, effects);
    *state = tx.commit().state;
    match dispatch_result {
        Ok(_) | Err(EffectError::ChoiceRequested) => Ok(Vec::new()),
        Err(err) => Err(err),
    }
}

#[test]
fn package_publishes_olzhijie_contract() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 745);
    let entry = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == OLZHIJIE)
        .expect("olzhijie trigger");
    assert_eq!(entry.event.as_str(), "phase_end");
    assert_eq!(
        entry.filter.as_str(),
        "owner_used_and_was_targeted_this_turn_in_range"
    );
    assert!(!entry.optional);
}

#[test]
fn end_phase_with_all_conditions_met_deals_one_damage_in_range() {
    let (mut state, players) = started(2_026_094_001);
    let owner = players[0];
    let target = players[1];
    grant(&mut state, owner, OLZHIJIE);
    // 三条件置备：本回合用过牌（flag 直置）、被指定为目标（事件记录）、
    // 射程内有角色（二人局相邻默认在射程内）。
    {
        let mut tx = Transaction::begin(state.clone());
        let key = engine_core::state::turn_flag::card_action_count_by_player(owner);
        tx.state_mut().turn.set_flag(key, 1);
        state = tx.commit().state;
    }
    // 经真实 CardUsed 事件产生被指定目标记录（track 门控开启）。
    let card = inject_hand(&mut state, target, 1);
    let mut tx = Transaction::begin(state);
    dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::CardUsed {
            player: target,
            card,
            targets: vec![owner],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    let mut state = tx.commit().state;
    assert!(
        state.turn.card_targeted_count(owner) > 0,
        "被指定目标应记录"
    );

    // 结束阶段触发 → 强制选目标（射程内）。
    end_phase(&mut state, target).unwrap();
    assert!(state.has_pending_choice(), "三条件齐备应打开选目标窗口");
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ResolveOlzhijieStage
    );

    let (state, result) = resolve(state, owner, vec![target.raw()], false, 7);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, 2);
    assert!(!state.has_pending_choice());
}

#[test]
fn end_phase_without_conditions_is_quiet() {
    let (mut state, players) = started(2_026_094_002);
    let owner = players[0];
    let other = players[1];
    grant(&mut state, owner, OLZHIJIE);
    // 无用牌、无被指定目标 → 静默。
    let effects = end_phase(&mut state, other).unwrap();
    assert!(effects.is_empty());
    let other_hp = state.players.get(other).unwrap().hp;
    assert_eq!(other_hp, 3);
}
