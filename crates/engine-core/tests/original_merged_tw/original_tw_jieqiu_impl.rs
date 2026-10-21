//! tw/twjieqiu 劫囚（第一百零二批·§7 装备栏编排首迁）：
//! resolve_twjieqiu 多路器 sub0（选目标禁全五槽+摸 k+BUFF 标）/ sub2（余禁用栏
//! → owner 入队纯额外回合）。BUFF 标 = blake2b("twjieqiu_buff") = 385592780。

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWJIEQIU: u32 = 4_201_797_709;
const TWJIEQIU_BUFF: u32 = 385_592_780;

const SOURCES_DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../content/original/ported/tw"
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

fn started() -> (GameState, PlayerId) {
    let compiled = content_compiler::compile_source_named("tw-jieqiu-impl", &sources()).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 20_260_837, "identity-standard").unwrap();
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
            skill_def: SkillDefId::new(TWJIEQIU),
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

fn buff_count(state: &GameState, player: PlayerId) -> i32 {
    state
        .players
        .get(player)
        .and_then(|p| {
            p.marks
                .iter()
                .find(|(id, _)| *id == TWJIEQIU_BUFF)
                .map(|(_, c)| *c)
        })
        .unwrap_or(0)
}

fn equip_card(tx: &mut Transaction, owner: PlayerId, slot_tag: u8) -> engine_api::CardId {
    let zone = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let (kind, def) = match slot_tag {
        1 => (engine_core::state::CardKind::Weapon, 11),
        2 => (engine_core::state::CardKind::Armor, 12),
        3 => (engine_core::state::CardKind::HorseDef, 13),
        4 => (engine_core::state::CardKind::HorseOff, 14),
        _ => (engine_core::state::CardKind::Treasure, 15),
    };
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(def),
        kind,
        suit: engine_api::Suit::Spade,
        rank: 1,
        weapon_range: 0,
        zone,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

#[test]
fn twjieqiu_publishes_contract() {
    let compiled =
        content_compiler::compile_source_named("tw-jieqiu-contract", &sources()).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 733);
    assert!(compiled
        .active_index
        .iter()
        .any(|a| a.skill_def_id == TWJIEQIU));
}

#[test]
fn twjieqiu_disables_all_slots_draws_and_marks_target() {
    let (state, holder) = started();
    let victim = others_of(&state, holder)[0];
    let mut tx = Transaction::begin(state);

    // 给 victim 两件装备（武器+防具马），记录禁用前手牌数。
    equip_card(&mut tx, victim, 1);
    equip_card(&mut tx, victim, 3);
    let victim_hand_before = tx.state().hand_count(victim);

    // 发动 sub0：选目标段。
    tx.state_mut().turn.phase = engine_core::state::Phase::Play;
    tx.state_mut().turn.current = holder;
    let sid = {
        let s = tx.state();
        s.skills
            .sorted_live()
            .iter()
            .find(|s| s.def.raw() == TWJIEQIU)
            .unwrap()
            .id
    };
    let opened = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveTwjieqiu {
            player: holder,
            target: engine_api::PlayerId::NONE,
            subaction: 0,
            skill: sid,
        }],
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::TwjieqiuSelect
    );
    let committed = tx.commit();
    let observer = committed.state.resolution.pending_observer;
    let final_state = resolve(
        committed.state,
        PlayerId::new(observer),
        vec![victim.raw()],
        1601,
    );

    // 全五槽禁用、BUFF 标记在场、摸 k=2 张。
    let victim_state = final_state.players.get(victim).unwrap();
    assert!(victim_state
        .disabled_equip_slots
        .iter()
        .all(|&count| count >= 1));
    assert_eq!(buff_count(&final_state, victim), 1);
    assert_eq!(final_state.hand_count(victim), victim_hand_before + 2);
}

#[test]
fn twjieqiu_sub2_enqueues_owner_only_when_slots_remain() {
    let (state, holder) = started();
    let victim = others_of(&state, holder)[0];
    let mut tx = Transaction::begin(state);
    let sid = tx
        .state()
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.def.raw() == TWJIEQIU)
        .unwrap()
        .id;

    // sub0 废除（无装备：k=0，仅打 BUFF 标）。
    tx.state_mut().turn.phase = engine_core::state::Phase::Play;
    tx.state_mut().turn.current = holder;
    let opened = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveTwjieqiu {
            player: holder,
            target: engine_api::PlayerId::NONE,
            subaction: 0,
            skill: sid,
        }],
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    let committed = tx.commit();
    let observer = committed.state.resolution.pending_observer;
    let bound = resolve(
        committed.state,
        PlayerId::new(observer),
        vec![victim.raw()],
        1611,
    );
    assert_eq!(buff_count(&bound, victim), 1);

    // sub2：目标仍有禁用栏 → owner 入队纯额外回合。
    let mut tx = Transaction::begin(bound);
    tx.state_mut().turn.current = victim;
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveTwjieqiu {
            player: holder,
            target: victim,
            subaction: 2,
            skill: sid,
        }],
    )
    .unwrap();
    let judged = tx.commit().state;
    assert_eq!(judged.turn.extra_turn_queue.first(), Some(&holder));
    assert_eq!(
        judged
            .turn
            .flag(engine_core::state::turn_flag::extra_turn_check_pending_by_player(holder)),
        0
    );
}

#[test]
fn twjieqiu_discard_restores_disabled_slots_incrementally() {
    let (state, holder) = started();
    let victim = others_of(&state, holder)[0];
    let mut tx = Transaction::begin(state);
    let sid = tx
        .state()
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.def.raw() == TWJIEQIU)
        .unwrap()
        .id;

    // sub0：禁全五槽+BUFF 标。
    tx.state_mut().turn.phase = engine_core::state::Phase::Play;
    tx.state_mut().turn.current = holder;
    let opened = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveTwjieqiu {
            player: holder,
            target: engine_api::PlayerId::NONE,
            subaction: 0,
            skill: sid,
        }],
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    let committed = tx.commit();
    let observer = committed.state.resolution.pending_observer;
    let bound = resolve(
        committed.state,
        PlayerId::new(observer),
        vec![victim.raw()],
        1701,
    );
    let disabled_total = |state: &GameState, player: PlayerId| -> i32 {
        state
            .players
            .get(player)
            .map(|p| p.disabled_equip_slots.iter().filter(|&&c| c > 0).count() as i32)
            .unwrap_or(0)
    };
    assert_eq!(disabled_total(&bound, victim), 5);

    // 弃 3 张 → 即时恢复 3 槽（剩 2）。
    let mut tx = Transaction::begin(bound);
    // 恢复钩子按「当前回合角色的弃牌」计数：victim 弃牌发生在 victim 回合。
    tx.state_mut().turn.current = victim;
    let cards = inject_cards(&mut tx, victim, 3);
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, engine_api::PlayerId::NONE)
        .unwrap();
    let moved = drive(
        &mut tx,
        &[engine_api::DomainEvent::CardsMoved {
            cards,
            from: engine_api::ZoneId::NONE,
            to: discard,
            reason: engine_api::CardMoveReason::Discard,
        }],
    );
    moved.unwrap();
    let after = tx.commit().state;
    assert_eq!(
        disabled_total(&after, victim),
        2,
        "3 discard restores 3 of 5 slots"
    );
}

fn inject_cards(tx: &mut Transaction, owner: PlayerId, count: u8) -> Vec<engine_api::CardId> {
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
