//! Golden coverage for 荆扬庞统「索舟」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, SkillDefId, Suit, RULE_VM_ABI_VERSION,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const JYSUOZHOU: u32 = 2_628_091_225;
const JY_PANGTONG: u32 = 3_283_376_492;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

#[test]
fn club_self_target_offers_once_and_draws_each_linked_player() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    assert_eq!(
        package
            .trigger_index
            .iter()
            .filter(|trigger| trigger.skill_def_id == JYSUOZHOU)
            .count(),
        2
    );
    assert!(package.character_index.iter().any(|character| {
        character.id == JY_PANGTONG
            && character.name_key == "jy_pangtong"
            && character.skills == [3_727_355_012, JYSUOZHOU, 2_185_491_588]
    }));

    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 2_026_073_102, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let linked = seats[1];
    let unlinked = seats[2];
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
        player: owner,
        skill_def: SkillDefId::new(JYSUOZHOU),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in [owner, linked] {
        tx.state_mut()
            .players
            .get_mut(player)
            .unwrap()
            .marks
            .push((engine_core::content::MARK_CHAIN, 1));
    }

    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(344_001),
        kind: CardKind::TrickTiesuo,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);

    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::CardUsed {
                player: owner,
                card,
                targets: vec![owner],
            }],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let committed = tx.commit();
    assert_eq!(
        committed.output.choice.as_ref().unwrap().kind,
        ChoiceKind::Confirm
    );
    let state = committed.state;
    let (state, result) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(owner), 2);
    assert_eq!(state.hand_count(linked), 1);
    assert_eq!(state.hand_count(unlinked), 0);
}
