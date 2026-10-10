//! diy/nsjihui 集计（第九十五批·queue_extra_turn 第五消费者）：单次弃牌≥3 张
//! 入弃牌堆 → 每回合一次入队纯额外回合（check=0，storage 自锁，own turn 结束清零）。

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const NSJIHUI: u32 = 3_934_300_121;

const SOURCES_DIR: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/../../content/original/ported/diy");

fn sources() -> String {
    let dir = std::path::Path::new(SOURCES_DIR);
    let mut files: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "skill"))
        .map(|e| e.path())
        .collect();
    files.sort();
    files
        .iter()
        .map(|p| {
            let mut s = std::fs::read_to_string(p).unwrap();
            s.push('\n');
            s
        })
        .collect()
}

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let (state, pipe) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    state
}

fn drive(
    tx: &mut Transaction,
    events: &[engine_api::DomainEvent],
) -> Result<(), EffectError> {
    match engine_core::trigger::dispatch_for_events(
        tx,
        events,
        &mut engine_core::trigger::TriggerFireRecord::new(),
    ) {
        Err(e) => Err(e),
        Ok(effects) => run_effects(tx, effects).map(|_| ()),
    }
}

fn started() -> (GameState, PlayerId) {
    let compiled = content_compiler::compile_source_named("diy-nsjihui", &sources()).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 20_260_835, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    let owner = tx.state().turn.current;
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(NSJIHUI),
        }],
    )
    .unwrap();
    (tx.commit().state, owner)
}


fn discard_zone_of(tx: &Transaction) -> engine_api::ZoneId {
    tx.state()
        .zones
        .find(ZoneKind::DiscardPile, engine_api::PlayerId::NONE)
        .unwrap()
}

fn inject_cards(tx: &mut Transaction, owner: PlayerId, count: u8) -> Vec<CardId> {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    (0..count)
        .map(|index| {
            let card = tx.state_mut().cards.alloc(Card {
                id: CardId::NONE,
                def: engine_api::CardDefId::new(1 + index as u32),
                kind: engine_core::state::CardKind::Tao,
                suit: engine_api::Suit::Heart,
                rank: 2 + index,
                weapon_range: 0,
                zone: hand,
                owner,
                face_up: true,
            });
            tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
            card
        })
        .collect()
}

fn discard_move_event(cards: Vec<CardId>, to: engine_api::ZoneId) -> engine_api::DomainEvent {
    engine_api::DomainEvent::CardsMoved {
        cards,
        from: engine_api::ZoneId::NONE,
        to,
        reason: engine_api::CardMoveReason::Discard,
    }
}

#[test]
fn nsjihui_publishes_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(include_bytes!(
        "../../../content/original/ported/diy/diy.npack.json"
    ))
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 684);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == NSJIHUI
            && t.event == "cards_moved"
            && t.filter == "cards_moved_discard_at_least_three"
            && t.optional
    }));
}

#[test]
fn nsjihui_queues_once_per_turn_with_storage_latch() {
    use engine_core::state::turn_flag;
    let (state, holder) = started();
    let mut tx = Transaction::begin(state);
    let discard = discard_zone_of(&tx);

    // 单次弃置 4 张入弃牌堆。
    let cards = inject_cards(&mut tx, holder, 4);
    let cur = tx.state().turn.current;
    let opened = drive(&mut tx, &[discard_move_event(cards, discard)]);
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let mid = resolve(tx.state, holder, vec![1], 1301);
    assert!(!mid.has_pending_choice());
    assert_eq!(mid.turn.extra_turn_queue.first(), Some(&holder));
    assert_eq!(
        mid.turn
            .flag(turn_flag::extra_turn_check_pending_by_player(holder)),
        0
    );
    let sid = mid
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.def.raw() == NSJIHUI)
        .unwrap()
        .id;
    assert_eq!(mid.skills.storage_get(sid, 1), 1);

    // 同回合第二次 ≥3 弃置：confirm 照开、body 静默——队列不增长。
    let mut tx = Transaction::begin(mid);
    let cards2 = inject_cards(&mut tx, holder, 4);
    let opened = drive(&mut tx, &[discard_move_event(cards2, discard)]);
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    let tx = tx.commit();
    let gated = resolve(tx.state, holder, vec![1], 1302);
    let entries = gated
        .turn
        .extra_turn_queue
        .iter()
        .filter(|c| **c == holder)
        .count();
    assert_eq!(entries, 1, "latched body must not re-queue");

    // 自身回合结束：自锁清零。
    let mut tx = Transaction::begin(gated);
    let cleared = drive(
        &mut tx,
        &[engine_api::DomainEvent::PhaseEnded {
            current: holder,
            phase: engine_api::TurnPhaseTag::End,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_targeted_players: 0,
            phase_hp_changed_players: 0,
        }],
    );
    cleared.unwrap();
    let cleared_state = tx.commit().state;
    let sid = cleared_state
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.def.raw() == NSJIHUI)
        .unwrap()
        .id;
    assert_eq!(cleared_state.skills.storage_get(sid, 1), 0);
}

#[test]
fn nsjihui_ignores_moves_below_three_cards() {
    let (state, holder) = started();
    let mut tx = Transaction::begin(state);
    let discard = discard_zone_of(&tx);
    let cards = inject_cards(&mut tx, holder, 2);
    let quiet = drive(&mut tx, &[discard_move_event(cards, discard)]);
    quiet.unwrap();
    assert!(!tx.state().has_pending_choice());
}
