//! Golden coverage for MOBILE 胡车儿「盗戟」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, EquipSlot, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DAOJI: u32 = 4_263_252_675;
const HUCHEER: u32 = 3_031_184_045;
const GAIN_GUARD: u32 = 4_263_252_676;
const SOURCE: &str = include_str!("../../../content/original/ported/mobile/daoji.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
}

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    def: u32,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Club,
        rank: 7,
        weapon_range: if kind == CardKind::Weapon { 2 } else { 0 },
        zone,
        owner,
        face_up: zone_kind == ZoneKind::Equip,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(seed: u64, gain_guard: bool) -> Setup {
    let source = if gain_guard {
        format!(
            "{SOURCE}\nskill gain_guard({GAIN_GUARD}) locked {{\n  modifier card_gain_protected(equip) override 1;\n}}\n"
        )
    } else {
        SOURCE.into()
    };
    let package = content_compiler::compile_source_named("daoji-test", &source).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(DAOJI),
    });
    if gain_guard {
        effects.push(EffectRequest::AddSkill {
            player: target,
            skill_def: SkillDefId::new(GAIN_GUARD),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .find_map(|skill| {
            tx.state()
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == DAOJI)
                .then_some(*skill)
        })
        .unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        target,
        skill,
    }
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, CommandResult) {
    let (state, result) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    );
    (state, result.result)
}

fn activate(setup: &Setup, cost: CardId, key: u64) -> (GameState, CommandResult) {
    submit(
        setup.state.clone(),
        setup.owner,
        CommandKind::ActivateSkill {
            skill: setup.skill,
            targets: vec![setup.target],
            card_targets: vec![cost],
        },
        key,
    )
}

fn choose_equipment(
    state: GameState,
    owner: PlayerId,
    card: CardId,
    key: u64,
) -> (GameState, CommandResult) {
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        owner,
        CommandKind::ResolveChoice {
            choice_id: choice,
            selections: vec![card.raw()],
            cancelled: false,
        },
        key,
    )
}

#[test]
fn package_publishes_hucheer_and_daoji() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 327);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.id == HUCHEER)
            .unwrap()
            .skills,
        [DAOJI]
    );
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == DAOJI)
        .unwrap();
    assert!(active.targets.as_ref().unwrap().has_gainable_equip);
    assert_eq!(
        active.selection.as_ref().unwrap().filter.as_deref(),
        Some("card_nonbasic")
    );
}

#[test]
fn weapon_is_gained_equipped_then_damages_original_owner() {
    let setup = setup(2_026_073_201, false);
    let mut tx = Transaction::begin(setup.state);
    let cost = inject(
        &mut tx,
        setup.owner,
        ZoneKind::Hand,
        CardKind::Other,
        99_601,
    );
    let weapon = inject(
        &mut tx,
        setup.target,
        ZoneKind::Equip,
        CardKind::Weapon,
        99_602,
    );
    let hp = tx.state().players.get(setup.target).unwrap().hp;
    let setup = Setup {
        state: tx.commit().state,
        ..setup
    };
    let (state, result) = activate(&setup, cost, 1);
    assert!(matches!(result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ResolveDaoji
    );
    let (state, result) = choose_equipment(state, setup.owner, weapon, 2);
    assert!(matches!(result, CommandResult::Accepted { .. }));
    let equip = state.zones.find(ZoneKind::Equip, setup.owner).unwrap();
    assert_eq!(state.cards.get(weapon).unwrap().owner, setup.owner);
    assert_eq!(state.cards.get(weapon).unwrap().zone, equip);
    assert_eq!(state.players.get(setup.target).unwrap().hp, hp - 1);
}

#[test]
fn nonweapon_causes_no_damage_and_disabled_weapon_slot_still_takes_damage() {
    for (kind, disable_slot, expected_in_hand, expected_damage, seed) in [
        (CardKind::Armor, false, false, 0, 2_026_073_202),
        (CardKind::Weapon, true, true, 1, 2_026_073_203),
    ] {
        let setup = setup(seed, false);
        let mut tx = Transaction::begin(setup.state);
        if disable_slot {
            tx.state_mut()
                .players
                .get_mut(setup.owner)
                .unwrap()
                .disabled_equip_slots[EquipSlot::Weapon.index()] = 1;
        }
        let cost = inject(
            &mut tx,
            setup.owner,
            ZoneKind::Hand,
            CardKind::Other,
            99_610,
        );
        let equipment = inject(&mut tx, setup.target, ZoneKind::Equip, kind, 99_611);
        let hp = tx.state().players.get(setup.target).unwrap().hp;
        let setup = Setup {
            state: tx.commit().state,
            ..setup
        };
        let (state, result) = activate(&setup, cost, u64::from(kind.tag()));
        assert!(matches!(result, CommandResult::Accepted { .. }));
        let (state, result) =
            choose_equipment(state, setup.owner, equipment, u64::from(kind.tag()) + 10);
        assert!(matches!(result, CommandResult::Accepted { .. }));
        let hand = state.zones.find(ZoneKind::Hand, setup.owner).unwrap();
        assert_eq!(
            state.cards.get(equipment).unwrap().zone == hand,
            expected_in_hand
        );
        assert_eq!(
            state.players.get(setup.target).unwrap().hp,
            hp - expected_damage
        );
    }
}

#[test]
fn basic_cost_and_gain_protected_equipment_are_rejected() {
    {
        let setup = setup(2_026_073_204, false);
        let mut tx = Transaction::begin(setup.state);
        let basic = inject(&mut tx, setup.owner, ZoneKind::Hand, CardKind::Sha, 99_620);
        inject(
            &mut tx,
            setup.target,
            ZoneKind::Equip,
            CardKind::Armor,
            99_621,
        );
        let setup = Setup {
            state: tx.commit().state,
            ..setup
        };
        let (_, result) = activate(&setup, basic, 21);
        assert!(matches!(result, CommandResult::Rejected { .. }));
    }

    let setup = setup(2_026_073_205, true);
    let mut tx = Transaction::begin(setup.state);
    let cost = inject(
        &mut tx,
        setup.owner,
        ZoneKind::Hand,
        CardKind::Other,
        99_622,
    );
    inject(
        &mut tx,
        setup.target,
        ZoneKind::Equip,
        CardKind::Armor,
        99_623,
    );
    let setup = Setup {
        state: tx.commit().state,
        ..setup
    };
    let (_, result) = activate(&setup, cost, 22);
    assert!(matches!(result, CommandResult::Rejected { .. }));
}
