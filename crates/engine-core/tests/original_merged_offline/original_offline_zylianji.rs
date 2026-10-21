//! offline/zylianji 连计（第一百一十九批·§6 S2）：
//! phase_play_end（本阶段用过牌，类别≥1）→ 令一名角色摸 1 → 跳自己剩余阶段 →
//! 仅 Play 的裁剪额外回合（skip_remaining_phases + queue_extra_phases）。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const ZYLIANJI: u32 = 4_062_778_093;
const TWJIEQIU: u32 = 4_201_797_709;

const SOURCES_DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../content/original/ported/offline"
);

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

fn drive(tx: &mut Transaction, events: &[engine_api::DomainEvent]) -> Result<(), EffectError> {
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
    let compiled = content_compiler::compile_source_named("offline-zylianji", &sources()).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 20_260_839, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&compiled)
        .unwrap();
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
            skill_def: SkillDefId::new(ZYLIANJI),
        }],
    )
    .unwrap();
    (tx.commit().state, owner)
}

fn others_of(state: &GameState, owner: PlayerId) -> Vec<PlayerId> {
    state
        .players
        .iter_live()
        .map(|p| p.id)
        .filter(|id| *id != owner)
        .collect()
}

fn inject_cards(tx: &mut Transaction, owner: PlayerId, count: u8) -> Vec<engine_api::CardId> {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    (0..count)
        .map(|index| {
            let card = tx.state_mut().cards.alloc(Card {
                id: engine_api::CardId::NONE,
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

fn discard_move(cards: Vec<engine_api::CardId>, to: engine_api::ZoneId) -> engine_api::DomainEvent {
    engine_api::DomainEvent::CardsMoved {
        cards,
        from: engine_api::ZoneId::NONE,
        to,
        reason: engine_api::CardMoveReason::Discard,
    }
}

fn end_of_play_event(current: PlayerId) -> engine_api::DomainEvent {
    engine_api::DomainEvent::PhaseEnded {
        current,
        phase: engine_api::TurnPhaseTag::Play,
        damage_sources: 0,
        damaged_players: 0,
        card_targeted_other: false,
        current_dealt_damage_other: false,
        phase_targeted_players: 0,
        phase_hp_changed_players: 0,
    }
}

#[test]
fn zylianji_publishes_contract() {
    let compiled =
        content_compiler::compile_source_named("offline-zylianji-contract", &sources()).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 691);
    assert!(compiled.trigger_index.iter().any(|t| {
        t.skill_def_id == ZYLIANJI
            && t.event == "phase_play_end"
            && t.filter == "owner_alive_play_end_used_categories_positive"
            && t.optional
    }));
}

#[test]
fn zylianji_grants_draw_skips_remaining_and_cropped_turn() {
    let (state, holder) = started();
    let victim = others_of(&state, holder)[0];
    let mut tx = Transaction::begin(state);
    let sid = tx
        .state()
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.def.raw() == ZYLIANJI)
        .unwrap()
        .id;

    // 出牌阶段（Play）用了 2 张牌 → 类别数 ≥1。
    tx.state_mut().turn.phase = engine_core::state::Phase::Play;
    tx.state_mut().turn.current = holder;
    let cards = inject_cards(&mut tx, holder, 2);
    let opened = drive(
        &mut tx,
        &[engine_api::DomainEvent::CardUsed {
            player: holder,
            card: cards[0],
            targets: vec![],
        }],
    );
    opened.unwrap();

    // 出牌阶段结束：confirm → 摸1给任一角色 → 跳自己剩余 → 仅 Play 裁剪回合。
    let cur = tx.state().turn.current;
    let opened = drive(&mut tx, &[end_of_play_event(cur)]);
    assert_eq!(
        opened.unwrap_err(),
        EffectError::ChoiceRequested,
        "used cards this phase opens the confirm"
    );
    let committed = tx.commit();
    // confirm 后先开「选一名角色摸 1」的续体（choose_other_draw_or_recover）。
    let mid = resolve(committed.state, holder, vec![1], 1801);
    assert_eq!(
        mid.resolution.continuation_kind,
        engine_core::state::ContinuationKind::ChooseOtherDrawOrRecover
    );
    let observer = mid.resolution.pending_observer;
    let mid = resolve(mid, PlayerId::new(observer), vec![victim.raw()], 1802);
    assert!(!mid.has_pending_choice());

    // 跳过的 SKIP 标志已置位（Draw/Discard），Play 未跳。
    assert!(mid.turn.skip_draw());
    assert!(!mid.turn.skip_play());
    assert!(mid.turn.skip_discard());
    // 裁剪回合入队：仅 Play。
    assert_eq!(mid.turn.extra_phase_queue.len(), 1);
    assert_eq!(mid.turn.extra_phase_queue[0].player, holder);
    assert_eq!(mid.turn.extra_phase_queue[0].phases, vec![3u8]);
}
