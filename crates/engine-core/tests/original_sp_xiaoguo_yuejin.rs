//! Golden coverage for SP 乐进「骁果」rewritten from legacy JavaScript semantics.

use content_compiler::compile_source_named;
use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::{choose_command, Engine, Snapshot};
use rule_api::EffectRequest;

const XIAOGUO: u32 = 3_124_297_312;
const YUEJIN: u32 = 3_111_158_867;
const LIANYING: u32 = 21;
const XIAOJI: u32 = 22;
const SELF_PROTECT: u32 = 4_000_000_001;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/xiaoguo.skill");
const SP_PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");
const STANDARD_PACKAGE: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");
const XIAOGUO_LOGIC_HASH: [u8; 32] = [
    202, 246, 190, 46, 2, 115, 148, 61, 144, 149, 37, 189, 76, 146, 48, 6, 147, 0, 246, 171, 167,
    47, 165, 248, 157, 203, 236, 177, 184, 39, 163, 119,
];
const STANDARD_MODULE_LOGIC_HASH: [u8; 32] = [
    70, 220, 40, 61, 246, 193, 90, 74, 103, 36, 2, 104, 26, 2, 184, 113, 132, 28, 34, 241, 78, 86,
    25, 50, 39, 5, 150, 204, 19, 169, 163, 64,
];
const STANDARD_PACKAGE_LOGIC_HASH: [u8; 32] = [221,86,128,181,186,208,77,141,253,212,39,65,62,1,22,122,8,121,220,124,174,50,158,117,82,75,93,232,8,179,60,11];

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    cost: CardId,
    equipment: Option<CardId>,
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    rank: u8,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(98_500 + u32::from(rank)),
        kind,
        suit: Suit::Club,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(
    equipment_zone: Option<ZoneKind>,
    owner_extra_skill: Option<u32>,
    target_extra_skill: Option<u32>,
    protect_target_equipment: bool,
) -> Setup {
    let source = if protect_target_equipment {
        format!(
            "{SOURCE}\nskill self_protect({SELF_PROTECT}) {{\n  modifier card_discard_protected(equip) override 1;\n}}\n"
        )
    } else {
        SOURCE.to_owned()
    };
    let package = compile_source_named("sp-xiaoguo-test", &source).unwrap();
    let standard = content_schema::ContentPackage::from_json_bytes(STANDARD_PACKAGE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&standard).unwrap();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_730_01, "identity-standard").unwrap();
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
        skill_def: SkillDefId::new(XIAOGUO),
    });
    if let Some(skill) = owner_extra_skill {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill),
        });
    }
    if let Some(skill) = target_extra_skill {
        effects.push(EffectRequest::AddSkill {
            player: target,
            skill_def: SkillDefId::new(skill),
        });
    }
    if protect_target_equipment {
        effects.push(EffectRequest::AddSkill {
            player: target,
            skill_def: SkillDefId::new(SELF_PROTECT),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    let cost = inject(tx.state_mut(), owner, ZoneKind::Hand, CardKind::Other, 2);
    let equipment =
        equipment_zone.map(|zone| inject(tx.state_mut(), target, zone, CardKind::Armor, 3));
    tx.state_mut().turn.current = target;
    tx.state_mut().turn.phase = Phase::End;
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        target,
        cost,
        equipment,
    }
}

fn dispatch_end(state: GameState, target: PlayerId) -> (GameState, engine_api::ChoiceRequest) {
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: target,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert_eq!(
        run_effects(&mut tx, effects),
        Err(EffectError::ChoiceRequested)
    );
    let committed = tx.commit();
    (committed.state, committed.output.choice.unwrap())
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let command = Command {
        actor,
        expected_revision: ExpectedRevision(state.revision),
        idempotency_key: IdempotencyKey(key),
        kind: CommandKind::ResolveChoice {
            choice_id: ChoiceId::new(state.resolution.pending_choice_id),
            selections,
            cancelled,
        },
    };
    apply_command(state, command)
}

#[test]
fn package_and_module_freeze_sp_contract_without_touching_standard() {
    let compiled = compile_source_named("sp-xiaoguo", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 299);
    let trigger = compiled.trigger_index.first().unwrap();
    assert_eq!(trigger.event, "phase_end");
    assert_eq!(trigger.filter, "owner_alive_other_phase_end_hand_nonempty");
    assert!(!trigger.optional);
    let module = compiled
        .modules
        .iter()
        .find(|module| module.name == "xiaoguo_phase_end")
        .unwrap();
    assert_eq!(module.code, [83, 84, 90, 30, 1, 1]);
    assert_eq!(module.logic_hash, XIAOGUO_LOGIC_HASH);
    assert_eq!(rule_vm::bytecode::effect_tag::RESOLVE_SP_XIAOGUO, 286);
    assert_eq!(ContinuationKind::ResolveSpXiaoguo.tag(), 206);

    let sp = content_schema::ContentPackage::from_json_bytes(SP_PACKAGE).unwrap();
    assert!(sp.manifest.rule_vm_abi_version >= 647);
    let yuejin = sp
        .character_index
        .iter()
        .find(|character| character.id == YUEJIN)
        .unwrap();
    assert_eq!(yuejin.name_key, "yuejin");
    assert_eq!(yuejin.sex, "male");
    assert_eq!(yuejin.group, "wei");
    assert_eq!(yuejin.max_hp, 4);
    assert_eq!(yuejin.skills, [XIAOGUO]);

    let standard = content_schema::ContentPackage::from_json_bytes(STANDARD_PACKAGE).unwrap();
    assert_eq!(standard.manifest.rule_vm_abi_version, 261);
    assert_eq!(standard.manifest.logic_hash, STANDARD_PACKAGE_LOGIC_HASH);
    let std = standard
        .modules
        .iter()
        .find(|module| module.name == "stdxiaoguo_phase_end")
        .unwrap();
    assert_eq!(std.code, [83, 84, 80, 177, 1]);
    assert_eq!(std.logic_hash, STANDARD_MODULE_LOGIC_HASH);
}

#[test]
fn unrestricted_hand_cost_then_target_discards_equipment_or_takes_damage() {
    let setup = crate::setup(Some(ZoneKind::Hand), None, None, false);
    let hp = setup.state.players.get(setup.target).unwrap().hp;
    let equipment = setup.equipment.unwrap();
    let (state, cost) = dispatch_end(setup.state, setup.target);
    assert_eq!(cost.observer, setup.owner);
    assert_eq!(cost.kind, ChoiceKind::SelectCards);
    assert_eq!(
        cost.options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [setup.cost.raw()]
    );
    let (state, result) = resolve(state, setup.owner, vec![setup.cost.raw()], false, 1);
    let consequence = result.choice.unwrap();
    assert_eq!(consequence.observer, setup.target);
    assert!(consequence
        .options
        .iter()
        .any(|option| option.token == equipment.raw()));

    let (paid, result) = resolve(state.clone(), setup.target, vec![equipment.raw()], false, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(paid.players.get(setup.target).unwrap().hp, hp);
    assert_eq!(paid.hand_cards(setup.owner).len(), 1);
    assert!(!paid.hand_cards(setup.target).contains(&equipment));

    let (damaged, result) = resolve(state, setup.target, Vec::new(), true, 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(damaged.players.get(setup.target).unwrap().hp, hp - 1);
    assert!(damaged.hand_cards(setup.target).contains(&equipment));
    assert!(damaged.hand_cards(setup.owner).is_empty());
}

#[test]
fn owner_cancel_is_noop_and_missing_equipment_damages_without_another_choice() {
    let setup = setup(None, None, None, false);
    let hp = setup.state.players.get(setup.target).unwrap().hp;
    let (state, _) = dispatch_end(setup.state, setup.target);
    let (cancelled, result) = resolve(state.clone(), setup.owner, Vec::new(), true, 10);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert!(cancelled.hand_cards(setup.owner).contains(&setup.cost));
    assert_eq!(cancelled.players.get(setup.target).unwrap().hp, hp);

    let (damaged, result) = resolve(state, setup.owner, vec![setup.cost.raw()], false, 11);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(damaged.players.get(setup.target).unwrap().hp, hp - 1);
    assert!(damaged.hand_cards(setup.owner).is_empty());
}

#[test]
fn both_discard_followups_wait_for_nested_movement_triggers() {
    let setup = setup(Some(ZoneKind::Hand), Some(LIANYING), None, false);
    let equipment = setup.equipment.unwrap();
    let (state, _) = dispatch_end(setup.state, setup.target);
    let (state, result) = resolve(state, setup.owner, vec![setup.cost.raw()], false, 20);
    let lianying = result.choice.unwrap();
    assert_eq!(lianying.observer, setup.owner);
    assert!(state.hand_cards(setup.owner).is_empty());
    let (state, result) = resolve(state, setup.owner, vec![1], false, 21);
    let consequence = result.choice.unwrap();
    assert_eq!(consequence.observer, setup.target);
    assert_eq!(state.hand_cards(setup.owner).len(), 1);
    let (_, result) = resolve(state, setup.target, vec![equipment.raw()], false, 22);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));

    let setup = crate::setup(Some(ZoneKind::Equip), None, Some(XIAOJI), false);
    let equipment = setup.equipment.unwrap();
    let (state, _) = dispatch_end(setup.state, setup.target);
    let (state, result) = resolve(state, setup.owner, vec![setup.cost.raw()], false, 23);
    assert_eq!(result.choice.unwrap().observer, setup.target);
    let (state, result) = resolve(state, setup.target, vec![equipment.raw()], false, 24);
    let xiaoji = result.choice.unwrap();
    assert_eq!(xiaoji.observer, setup.target);
    assert!(state.hand_cards(setup.owner).is_empty());
    assert!(state.hand_cards(setup.target).is_empty());
    let (state, result) = resolve(state, setup.target, vec![1], false, 25);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_cards(setup.target).len(), 2);
    assert_eq!(state.hand_cards(setup.owner).len(), 1);
}

#[test]
fn other_player_discard_protection_does_not_block_target_self_discard() {
    let setup = setup(Some(ZoneKind::Equip), None, None, true);
    let hp = setup.state.players.get(setup.target).unwrap().hp;
    let equipment = setup.equipment.unwrap();
    let (state, _) = dispatch_end(setup.state, setup.target);
    let (state, result) = resolve(state, setup.owner, vec![setup.cost.raw()], false, 30);
    let consequence = result.choice.unwrap();
    assert_eq!(consequence.observer, setup.target);
    assert!(consequence
        .options
        .iter()
        .any(|option| option.token == equipment.raw()));
    let (state, result) = resolve(state, setup.target, vec![equipment.raw()], false, 31);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(setup.target).unwrap().hp, hp);
    let equip = state.zones.find(ZoneKind::Equip, setup.target).unwrap();
    assert!(!state.zones.get(equip).unwrap().cards.contains(&equipment));
    assert_eq!(state.hand_cards(setup.owner).len(), 1);
}

#[test]
fn headless_runner_cancels_both_optional_sp_xiaoguo_choices_legally() {
    let setup = setup(Some(ZoneKind::Hand), None, None, false);
    let owner = setup.owner;
    let target = setup.target;
    let hp = setup.state.players.get(target).unwrap().hp;
    let (state, cost_choice) = dispatch_end(setup.state, target);
    let mut snapshot = Snapshot::capture(&state);
    snapshot.pending_choice = Some(cost_choice);
    let mut engine = Engine::new();
    engine.restore_snapshot(&snapshot).unwrap();
    let command = choose_command(&engine);
    assert!(matches!(
        &command.kind,
        CommandKind::ResolveChoice {
            selections,
            cancelled: true,
            ..
        } if selections.is_empty()
    ));
    assert!(matches!(
        engine.submit(command),
        CommandResult::Accepted { .. }
    ));
    assert!(engine.state().hand_cards(owner).contains(&setup.cost));
    assert_eq!(engine.state().players.get(target).unwrap().hp, hp);

    let setup = crate::setup(Some(ZoneKind::Hand), None, None, false);
    let owner = setup.owner;
    let target = setup.target;
    let equipment = setup.equipment.unwrap();
    let hp = setup.state.players.get(target).unwrap().hp;
    let (state, _) = dispatch_end(setup.state, target);
    let (state, result) = resolve(state, owner, vec![setup.cost.raw()], false, 40);
    let consequence_choice = result.choice.unwrap();
    let mut snapshot = Snapshot::capture(&state);
    snapshot.pending_choice = Some(consequence_choice);
    let mut engine = Engine::new();
    engine.restore_snapshot(&snapshot).unwrap();
    let command = choose_command(&engine);
    assert!(matches!(
        &command.kind,
        CommandKind::ResolveChoice {
            selections,
            cancelled: true,
            ..
        } if selections.is_empty()
    ));
    assert!(matches!(
        engine.submit(command),
        CommandResult::Accepted { .. }
    ));
    assert_eq!(engine.state().players.get(target).unwrap().hp, hp - 1);
    assert!(engine.state().hand_cards(target).contains(&equipment));
}
