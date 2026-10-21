//! Golden for SP2「绝响」(dcjuexiang): on the owner's death the living damage
//! source discards all equipment and loses 1 HP; the owner then grants「残殒」
//! (dccanyun) to one other character.

use engine_api::{
    CardDefId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCJUEXIANG: u32 = 3_898_356_517;
const DCCANYUN: u32 = 3_523_035_260;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/sp2/sp2.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package())
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, first, second) = (players[0], players[1], players[2]);
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
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    // Keep seats distinct: owner kills first; second is the independent grant target.
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
}

fn equip_two_cards(tx: &mut Transaction, player: PlayerId) {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let equip_zone = tx.state().zones.find(ZoneKind::Equip, player).unwrap();
    let cards: Vec<_> = tx.state().hand_cards(player)[..2].to_vec();
    for (index, card) in cards.iter().enumerate() {
        let c = tx.state_mut().cards.get_mut(*card).unwrap();
        c.kind = if index == 0 {
            CardKind::Weapon
        } else {
            CardKind::Armor
        };
        c.weapon_range = if index == 0 { 2 } else { 0 };
    }
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards,
            to: equip_zone,
            from: Some(hand),
        }],
    )
    .unwrap();
}

fn has_skill(state: &GameState, player: PlayerId, def: u32) -> bool {
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .any(|skill| state.skills.get(*skill).map(|s| s.def.raw()) == Some(def))
}

#[test]
fn package_declares_dcjuexiang_death_trigger_with_source_alive_filter() {
    let package = package();
    let death = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCJUEXIANG)
        .expect("dcjuexiang death trigger");
    assert_eq!(death.event, "death");
    assert_eq!(death.filter, "owner_is_target_source_alive");
    assert!(!death.optional);
    assert!(!package
        .modifier_index
        .iter()
        .any(|m| m.skill_def_id == DCJUEXIANG));
}

#[test]
fn living_killer_discards_equipment_loses_hp_and_chosen_other_gains_canyun() {
    let (mut tx, owner, _first, second) = setup(20_261_101);
    add_skill(&mut tx, owner, DCJUEXIANG);
    let killer = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner && *player != second)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: killer,
            count: 2,
        }],
    )
    .unwrap();
    equip_two_cards(&mut tx, killer);
    let source_hp = tx.state().players.get(killer).unwrap().hp;

    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PlayerKilled {
            player: owner,
            killer,
            hand_equip_cards: vec![],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    let dispatched = run_effects(&mut tx, fired);
    assert!(matches!(dispatched, Err(EffectError::ChoiceRequested)));
    // Source equipment zone is emptied and source lost 1 HP before the grant choice.
    let equip_zone = tx.state().zones.find(ZoneKind::Equip, killer).unwrap();
    assert!(tx.state().zones.get(equip_zone).unwrap().cards.is_empty());
    assert_eq!(tx.state().players.get(killer).unwrap().hp, source_hp - 1);

    let committed = tx.commit();
    let revision = committed.state.revision;
    let choice_id = ChoiceId::new(committed.state.resolution.pending_choice_id);
    let (state, result) = engine_core::command::apply_command(
        committed.state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![second.raw()],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    assert!(has_skill(&state, second, DCCANYUN));
    assert!(!has_skill(&state, killer, DCCANYUN));
    assert!(!state.has_pending_choice());
}

#[test]
fn dead_killer_skips_effects_entirely() {
    let (mut tx, owner, _first, _second) = setup(20_261_102);
    add_skill(&mut tx, owner, DCJUEXIANG);
    // No real killer: PlayerId::NONE is never alive, so the source-alive filter
    // rejects the trigger and neither the discard nor the grant runs.
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PlayerKilled {
            player: owner,
            killer: PlayerId::NONE,
            hand_equip_cards: vec![],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(fired.is_empty());
}
