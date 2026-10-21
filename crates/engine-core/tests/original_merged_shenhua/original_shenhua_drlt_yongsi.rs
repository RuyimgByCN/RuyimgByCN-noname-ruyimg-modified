//! Golden coverage for 庸肆/伪帝（shenhua 袁术，ABI 527-528）.

use content_schema::PackageCharacterEntry;
use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Identity, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DRLT_YONGSI: u32 = 4_168_032_997;
const DRLT_WEIDI: u32 = 2_524_927_368;
const DRLT_YONGSI1: u32 = 3_870_932_875;
const SOURCE: &str = include_str!("../../../../content/original/ported/shenhua/drlt_yongsi.skill");

fn character(id: u32, group: &str) -> PackageCharacterEntry {
    PackageCharacterEntry {
        id,
        name_key: format!("yongsi_test_{id}"),
        max_hp: 4,
        initial_hp: 0,
        skills: vec![],
        sex: "male".into(),
        group: group.into(),
        is_lord: false,
        clans: vec![],
        tags: vec![],
        sort_key: String::new(),
        portrait_asset: None,
        skin_assets: vec![],
    }
}

struct Setup {
    tx: Transaction,
    lord: PlayerId,
    qun2: PlayerId,
    qun3: PlayerId,
    other: PlayerId,
}

/// `groups[0]` is the lord's faction; the remaining three are the other seats.
fn setup_with_groups(groups: [&str; 4]) -> Setup {
    let mut package = content_compiler::compile_source_named("yongsi-test", SOURCE).unwrap();
    package.character_index = vec![
        character(9_218_010, groups[0]),
        character(9_218_011, groups[1]),
        character(9_218_012, groups[2]),
        character(9_218_013, groups[3]),
    ];
    package.manifest.logic_hash = package.compute_logic_hash();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 4, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let lord = seats
        .iter()
        .copied()
        .find(|player| tx.state().players.get(*player).unwrap().identity == Identity::Lord)
        .unwrap();
    let rest: Vec<_> = seats
        .iter()
        .copied()
        .filter(|player| *player != lord)
        .collect();
    tx.state_mut().players.get_mut(lord).unwrap().character_def = 9_218_010;
    tx.state_mut()
        .players
        .get_mut(rest[0])
        .unwrap()
        .character_def = 9_218_011;
    tx.state_mut()
        .players
        .get_mut(rest[1])
        .unwrap()
        .character_def = 9_218_012;
    tx.state_mut()
        .players
        .get_mut(rest[2])
        .unwrap()
        .character_def = 9_218_013;
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
    effects.push(EffectRequest::AddSkill {
        player: lord,
        skill_def: SkillDefId::new(DRLT_YONGSI),
    });
    effects.push(EffectRequest::AddSkill {
        player: lord,
        skill_def: SkillDefId::new(DRLT_WEIDI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = lord;
    Setup {
        tx,
        lord,
        qun2: rest[0],
        qun3: rest[1],
        other: rest[2],
    }
}

fn alloc_hand(tx: &mut Transaction, owner: PlayerId, count: u32, base: u32) -> Vec<CardId> {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let mut cards = Vec::new();
    for i in 0..count {
        let card = tx.state_mut().cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(96_200 + base + i),
            kind: CardKind::Sha,
            suit: Suit::Spade,
            rank: 5,
            weapon_range: 0,
            zone: hand,
            owner,
            face_up: false,
        });
        tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
        cards.push(card);
    }
    cards
}

fn fire_phase(tx: Transaction, current: PlayerId, tag: TurnPhaseTag) -> Transaction {
    let mut tx = tx;
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current,
            phase: tag,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Ok(_) => tx,
            Err(EffectError::ChoiceRequested) => tx,
            Err(e) => panic!("effect error: {e:?}"),
        },
        Err(EffectError::ChoiceRequested) => tx,
        Err(e) => panic!("dispatch error: {e:?}"),
    }
}

fn fire_play_end(tx: Transaction, current: PlayerId) -> Transaction {
    let mut tx = tx;
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current,
            phase: TurnPhaseTag::Play,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_hp_changed_players: 0,
            phase_targeted_players: 0,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Ok(_) => tx,
            Err(EffectError::ChoiceRequested) => tx,
            Err(e) => panic!("effect error: {e:?}"),
        },
        Err(EffectError::ChoiceRequested) => tx,
        Err(e) => panic!("dispatch error: {e:?}"),
    }
}

fn resolve_choice(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
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
        "choice must be accepted: {:?}",
        result.result
    );
    next
}

#[test]
fn package_exposes_yongsi_weidi_contract() {
    let compiled = content_compiler::compile_source_named("yongsi", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 528);
    let draw_trigger = compiled
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == DRLT_YONGSI && t.event == "phase_draw")
        .unwrap();
    assert_eq!(draw_trigger.filter, "owner_is_target_draw_not_replaced");
    let end_trigger = compiled
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == DRLT_YONGSI && t.event == "phase_play_end")
        .unwrap();
    assert_eq!(end_trigger.filter, "owner_is_target");
    let weidi = compiled
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == DRLT_WEIDI)
        .unwrap();
    assert_eq!(weidi.event, "phase_discard");
    assert_eq!(
        weidi.filter,
        "owner_is_target_discard_phase_owe_limit_other_qun"
    );
}

#[test]
fn yongsi_replaces_draw_with_living_group_count() {
    let setup = setup_with_groups(["qun", "wei", "shu", "wu"]);
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.phase = Phase::Draw;
    let tx = fire_phase(tx, setup.lord, TurnPhaseTag::Draw);
    let state = tx.commit().state;
    assert_eq!(
        state.turn.draw_phase_fixed_count(),
        Some(4),
        "four factions alive"
    );
}

#[test]
fn yongsi_play_end_without_damage_draws_to_hp() {
    let setup = setup_with_groups(["qun", "qun", "qun", "wei"]);
    let mut tx = Transaction::begin(setup.tx.commit().state);
    alloc_hand(&mut tx, setup.lord, 2, 0);
    tx.state_mut().players.get_mut(setup.lord).unwrap().hp = 4;
    tx.state_mut().turn.phase = Phase::Play;
    let tx = fire_play_end(tx, setup.lord);
    let state = tx.commit().state;
    assert_eq!(state.hand_count(setup.lord), 4, "drew up to HP");
}

#[test]
fn yongsi_play_end_with_damage_grants_discard_state() {
    let setup = setup_with_groups(["qun", "qun", "qun", "wei"]);
    let mut tx = Transaction::begin(setup.tx.commit().state);
    alloc_hand(&mut tx, setup.lord, 2, 0);
    tx.state_mut().players.get_mut(setup.lord).unwrap().hp = 4;
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: setup.lord,
            target: setup.other,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let tx = fire_play_end(tx, setup.lord);
    let state = tx.commit().state;
    assert!(
        state
            .players
            .get(setup.lord)
            .unwrap()
            .skills
            .iter()
            .any(|skill| {
                state
                    .skills
                    .get(*skill)
                    .is_some_and(|instance| instance.def == SkillDefId::new(DRLT_YONGSI1))
            }),
        "the discard-phase state skill was granted after dealing damage"
    );
    assert_eq!(
        state.hand_count(setup.lord),
        2,
        "no draw when damage was dealt"
    );
}

#[test]
fn weidi_gives_cards_to_equal_qun_players() {
    let setup = setup_with_groups(["qun", "qun", "qun", "wei"]);
    let mut tx = Transaction::begin(setup.tx.commit().state);
    // Six hand cards vs hand limit 4: needs two, exactly two Qun others exist.
    alloc_hand(&mut tx, setup.lord, 6, 10);
    tx.state_mut().players.get_mut(setup.lord).unwrap().hp = 4;
    tx.state_mut().turn.phase = Phase::Discard;
    let tx = fire_phase(tx, setup.lord, TurnPhaseTag::Discard);
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice(), "optional confirm pending");
    state = resolve_choice(state, setup.lord, vec![1], 1);
    // Card choice: up to two discardable hand cards.
    assert!(state.has_pending_choice(), "card choice pending");
    let cards = state.hand_cards(setup.lord);
    assert!(cards.len() >= 6);
    let c1 = cards[0];
    let c2 = cards[1];
    let hand_before = state.hand_count(setup.lord);
    state = resolve_choice(state, setup.lord, vec![c1.raw(), c2.raw()], 2);
    // Target choice: two distinct Qun others.
    assert!(state.has_pending_choice(), "target choice pending");
    state = resolve_choice(
        state,
        setup.lord,
        vec![setup.qun2.raw(), setup.qun3.raw()],
        3,
    );

    assert_eq!(
        state.hand_count(setup.lord),
        hand_before - 2,
        "two cards were given away"
    );
    assert!(state.hand_cards(setup.qun2).contains(&c1));
    assert!(state.hand_cards(setup.qun3).contains(&c2));
    assert!(!state.has_pending_choice());
}
