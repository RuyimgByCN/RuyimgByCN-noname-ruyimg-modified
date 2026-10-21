//! refresh/botu 博图（第八十五批·跨包复用第二消费者）：本回合四花色齐 →
//! 队列一个纯额外回合（queue_extra_turn check=0）。suit 历史由引擎 CardUsed
//! 路径自动记录（包 ABI≥232 opt-in）。

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const BOTU: u32 = 3_144_154_445;

const SOURCES_DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../content/original/ported/refresh"
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
    let compiled = content_compiler::compile_source_named("refresh-botu", &sources()).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 20_260_831, "identity-standard").unwrap();
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
            skill_def: SkillDefId::new(BOTU),
        }],
    )
    .unwrap();
    (tx.commit().state, owner)
}

fn inject_suit_card(tx: &mut Transaction, owner: PlayerId, suit: Suit, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(1),
        kind: engine_core::state::CardKind::Tao,
        suit,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn botu_publishes_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(include_bytes!(
        "../../../../content/original/ported/refresh/refresh.npack.json"
    ))
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 723);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == BOTU
            && t.event == "turn_ended"
            && t.filter == "owner_alive_turn_used_all_suits"
            && t.optional
    }));
}

#[test]
fn botu_queues_extra_turn_after_fourth_suit_turn_end() {
    let (state, holder) = started();
    let mut tx = Transaction::begin(state);

    // 四张不同花色的桃，逐一合成 CardUsed——引擎路径回填位掩码。
    let suits = [Suit::Spade, Suit::Heart, Suit::Club, Suit::Diamond];
    for (index, suit) in suits.iter().enumerate() {
        let card = inject_suit_card(&mut tx, holder, *suit, 5 + index as u8);
        drive(
            &mut tx,
            &[engine_api::DomainEvent::CardUsed {
                player: holder,
                card,
                targets: vec![],
            }],
        )
        .unwrap();

        if index < 3 {
            // 未齐四花色：回合结束不触发。
            let mut probe_tx = Transaction::begin(tx.commit().state);
            let cur = probe_tx.state().turn.current;
            let quiet = drive(
                &mut probe_tx,
                &[engine_api::DomainEvent::PhaseEnded {
                    current: cur,
                    phase: engine_api::TurnPhaseTag::End,
                    damage_sources: 0,
                    damaged_players: 0,
                    card_targeted_other: false,
                    current_dealt_damage_other: false,
                    phase_targeted_players: 0,
                    phase_hp_changed_players: 0,
                }],
            );
            quiet.unwrap();
            assert!(!probe_tx.state().has_pending_choice());
            tx = probe_tx;
        }
    }

    // 第四花色后回合结束：confirm → 入队纯额外回合。
    let cur = tx.state().turn.current;
    let opened = drive(
        &mut tx,
        &[engine_api::DomainEvent::PhaseEnded {
            current: cur,
            phase: engine_api::TurnPhaseTag::End,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_targeted_players: 0,
            phase_hp_changed_players: 0,
        }],
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    let tx = tx.commit();
    let final_state = resolve(tx.state, holder, vec![1], 801);
    assert_eq!(final_state.turn.extra_turn_queue.first(), Some(&holder));
    assert_eq!(
        final_state
            .turn
            .flag(engine_core::state::turn_flag::extra_turn_check_pending_by_player(holder)),
        0,
        "check=0：博图无尾随代价"
    );
}
