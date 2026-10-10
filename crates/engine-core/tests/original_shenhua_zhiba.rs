//! Golden coverage for 制霸（shenhua 孙策，ABI 523）.

use content_schema::PackageCharacterEntry;
use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Identity, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const ZHIBA: u32 = 3_467_542_721;
const HUNZI: u32 = 4_151_424_761;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/zhiba.skill");

fn character(id: u32, group: &str) -> PackageCharacterEntry {
    PackageCharacterEntry {
        id,
        name_key: format!("zhiba_test_{id}"),
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
    actor: PlayerId,
}

/// Lord holds 制霸; a Wu-faction actor sits next in seat order. Both get one hand card.
fn setup(awakened: bool) -> Setup {
    // `awakened` = the lord already awakened (魂姿 removed). When false, the lord still
    // holds 魂姿 and therefore cannot refuse the comparison.
    let source = if !awakened {
        format!("{SOURCE}\nskill test_hunzi({HUNZI}) locked {{}}")
    } else {
        SOURCE.to_owned()
    };
    let mut package = content_compiler::compile_source_named("zhiba-test", &source).unwrap();
    package.character_index = vec![
        character(9_217_010, "wei"),
        character(9_217_011, "wu"),
        character(9_217_012, "shu"),
        character(9_217_013, "qun"),
    ];
    package.manifest.logic_hash = package.compute_logic_hash();

    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 4, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let lord = seats
        .iter()
        .copied()
        .find(|player| tx.state().players.get(*player).unwrap().identity == Identity::Lord)
        .unwrap();
    let actor = seats
        .iter()
        .copied()
        .find(|player| *player != lord)
        .unwrap();
    tx.state_mut().players.get_mut(lord).unwrap().character_def = 9_217_010;
    tx.state_mut().players.get_mut(actor).unwrap().character_def = 9_217_011;
    for seat in seats
        .iter()
        .filter(|player| **player != lord && **player != actor)
    {
        tx.state_mut().players.get_mut(*seat).unwrap().character_def = 9_217_012;
    }
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
        skill_def: SkillDefId::new(ZHIBA),
    });
    if !awakened {
        effects.push(EffectRequest::AddSkill {
            player: lord,
            skill_def: SkillDefId::new(HUNZI),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    // One hand card each for the comparison.
    for (owner, kind, suit, rank) in [
        (actor, CardKind::Sha, Suit::Spade, 5_u8),
        (lord, CardKind::Shan, Suit::Heart, 8_u8),
    ] {
        let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
        let card = tx.state_mut().cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(9_217_100 + rank as u32),
            kind,
            suit,
            rank,
            weapon_range: 0,
            zone: hand,
            owner,
            face_up: false,
        });
        tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    }
    tx.state_mut().turn.current = actor;
    tx.state_mut().turn.phase = Phase::Play;
    Setup { tx, lord, actor }
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
fn package_exposes_lord_compare_contract() {
    let compiled = content_compiler::compile_source_named("zhiba", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 523);
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == ZHIBA)
        .unwrap();
    assert_eq!(trigger.event, "phase_play");
    assert_eq!(
        trigger.filter,
        "owner_alive_other_wu_play_phase_can_compare_owner"
    );
    assert!(trigger.optional);
}

#[test]
fn awakened_lord_can_refuse_then_actor_compare_wins() {
    let setup = setup(true);
    let mut tx = Transaction::begin(setup.tx.commit().state);
    let tx = fire_phase(tx, setup.actor, TurnPhaseTag::Play);
    let mut state = tx.commit().state;
    // The Wu actor's optional confirm.
    assert!(state.has_pending_choice());
    state = resolve_choice(state, setup.actor, vec![1], 1);
    // Awakened lord's refuse choice.
    assert!(state.has_pending_choice());
    state = resolve_choice(state, setup.lord, vec![0], 2);
    // The actor's card outranks the lord's (8 > 5) so the actor wins.
    let actor_card = state.hand_cards(setup.actor)[0];
    let lord_card = state.hand_cards(setup.lord)[0];
    state.cards.get_mut(actor_card).unwrap().rank = 8;
    state.cards.get_mut(lord_card).unwrap().rank = 5;
    state = resolve_choice(state, setup.actor, vec![actor_card.raw()], 3);
    state = resolve_choice(state, setup.lord, vec![lord_card.raw()], 4);
    // Actor (8 > 5) wins: no settle choice; nobody gains.
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(setup.lord), 0);
    assert_eq!(state.hand_count(setup.actor), 0);
}

#[test]
fn unawakened_lord_wins_compare_and_takes_both_cards() {
    let setup = setup(false);
    let mut tx = Transaction::begin(setup.tx.commit().state);
    let tx = fire_phase(tx, setup.actor, TurnPhaseTag::Play);
    let mut state = tx.commit().state;
    // No refuse choice for the unawakened lord: the actor's confirm is the only prompt.
    state = resolve_choice(state, setup.actor, vec![1], 1);
    let actor_card = state.hand_cards(setup.actor)[0];
    let lord_card = state.hand_cards(setup.lord)[0];
    state = resolve_choice(state, setup.actor, vec![actor_card.raw()], 2);
    state = resolve_choice(state, setup.lord, vec![lord_card.raw()], 3);
    // Actor (5) did not win against the lord (8): the settle choice opens for the lord.
    assert!(state.has_pending_choice());
    state = resolve_choice(state, setup.lord, vec![1], 4);
    // The lord gained both comparison cards from the discard pile.
    assert_eq!(state.hand_count(setup.lord), 2);
    assert!(state.hand_cards(setup.lord).contains(&actor_card));
    assert!(state.hand_cards(setup.lord).contains(&lord_card));
    assert!(!state.has_pending_choice());
}
