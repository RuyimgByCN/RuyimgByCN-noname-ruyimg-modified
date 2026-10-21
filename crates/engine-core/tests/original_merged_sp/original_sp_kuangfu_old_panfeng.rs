//! Golden coverage for 狂斧 and old 潘凤 rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, EquipSlot, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const KUANGFU: u32 = 2_689_128_355;
const SOURCE: &str = include_str!("../../../../content/original/ported/sp/kuangfu.skill");

fn inject_weapon(tx: &mut Transaction, owner: PlayerId, def: u32) -> CardId {
    let equip = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Weapon,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 3,
        zone: equip,
        owner,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(equip)
        .unwrap()
        .cards
        .push(card);
    card
}

fn setup(seed: u64, disable_weapon: bool) -> (GameState, PlayerId, PlayerId, CardId, CardId) {
    let package = content_compiler::compile_source_named("kuangfu-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 282);
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_source_sha_damage_target_alive_has_discardable_equip"
    );
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain([EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(KUANGFU),
        }])
        .collect();
    run_effects(&mut tx, effects).unwrap();
    if disable_weapon {
        tx.state_mut()
            .players
            .get_mut(owner)
            .unwrap()
            .disabled_equip_slots[EquipSlot::Weapon.index()] = 1;
    }
    let old_weapon = if disable_weapon {
        CardId::NONE
    } else {
        inject_weapon(&mut tx, owner, 98_101)
    };
    let target_weapon = inject_weapon(&mut tx, target, 98_102);
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def == SkillDefId::new(KUANGFU))
        .unwrap()
        .id;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveKuangfu {
                player: owner,
                target,
                card: CardId::NONE,
                stage: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    (tx.commit().state, owner, target, old_weapon, target_weapon)
}

fn choose(state: GameState, actor: PlayerId, selection: u32, key: u64) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
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
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{:?}",
        result.result
    );
    state
}

#[test]
fn compatible_equipment_moves_and_replaces_while_disabled_slot_forces_discard() {
    let (state, owner, _, old_weapon, target_weapon) = setup(2_026_072_901, false);
    assert_eq!(state.resolution.pending_observer, owner.raw());
    let state = choose(state, owner, target_weapon.raw(), 1);
    assert_eq!(state.resolution.pending_observer, owner.raw());
    let state = choose(state, owner, 1, 2);
    let owner_equip = state.zones.find(ZoneKind::Equip, owner).unwrap();
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(target_weapon).unwrap().zone, owner_equip);
    assert_eq!(state.cards.get(old_weapon).unwrap().zone, discard);

    let (state, owner, _, _, target_weapon) = setup(2_026_072_902, true);
    let state = choose(state, owner, target_weapon.raw(), 3);
    assert_eq!(state.resolution.pending_choice_id, 0);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(target_weapon).unwrap().zone, discard);
}
