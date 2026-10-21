//! Golden for 四象费祎「调和」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const STDTIAOHE: u32 = 2_595_769_846;
const STDQIANSU: u32 = 2_964_345_042;
const STD_FEIYI: u32 = 2_878_317_274;
const PROTECT: u32 = 91_846;
const SOURCE: &str = concat!(
    include_str!("../../../../content/original/ported/sixiang/stdtiaohe.skill"),
    "\nskill protect(91846) locked { modifier card_discard_protected(equip) override 1; }\n"
);
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/sixiang/sixiang.npack.json");

fn setup() -> (GameState, [PlayerId; 3]) {
    let package = content_compiler::compile_source_named("stdtiaohe-test", SOURCE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(STDTIAOHE as u64),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: 20_260_718,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let players: [PlayerId; 3] = engine.state().players.alive_by_seat().try_into().unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(STDTIAOHE),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = players[0];
    tx.state_mut().turn.phase = Phase::Play;
    (tx.commit().state, players)
}

fn equip(state: &mut GameState, owner: PlayerId, kind: CardKind, def: u32) -> CardId {
    let zone = state.zones.find(ZoneKind::Equip, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone,
        owner,
        face_up: true,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn activate(
    state: GameState,
    owner: PlayerId,
    targets: Vec<PlayerId>,
    key: u64,
) -> (GameState, PipelineResult) {
    let skill = state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.def.raw() == STDTIAOHE)
        .unwrap()
        .id;
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets,
                card_targets: Vec::new(),
            },
        },
    )
}

fn resolve(
    state: GameState,
    owner: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

fn option_tokens(result: &PipelineResult) -> Vec<u32> {
    result
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .map(|option| option.token)
        .collect()
}

#[test]
fn committed_package_declares_typed_contract_and_complete_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == STDTIAOHE)
        .unwrap();
    assert_eq!(active.uses_per_turn, Some(1));
    let targets = active.targets.as_ref().unwrap();
    assert_eq!((targets.min, targets.max), (2, 2));
    assert!(targets.allow_self);
    assert!(targets.equipment_pair_with_armor);
    let feiyi = package
        .character_index
        .iter()
        .find(|character| character.id == STD_FEIYI)
        .unwrap();
    assert_eq!(feiyi.max_hp, 3);
    assert_eq!(feiyi.skills, vec![STDTIAOHE, STDQIANSU]);
}

#[test]
fn activation_rejects_duplicate_missing_equipment_and_pair_without_armor() {
    let (state, [owner, target0, target1]) = setup();
    let (_, duplicate) = activate(state.clone(), owner, vec![target0, target0], 1);
    assert!(matches!(duplicate.result, CommandResult::Rejected { .. }));
    let (_, empty) = activate(state.clone(), owner, vec![target0, target1], 2);
    assert!(matches!(empty.result, CommandResult::Rejected { .. }));

    let mut no_armor = state;
    equip(&mut no_armor, target0, CardKind::Weapon, 91_001);
    equip(&mut no_armor, target1, CardKind::HorseOff, 91_002);
    let (_, rejected) = activate(no_armor, owner, vec![target0, target1], 3);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
}

#[test]
fn armor_on_first_target_is_forced_then_second_equipment_is_discarded() {
    let (mut state, [owner, target0, target1]) = setup();
    let weapon0 = equip(&mut state, target0, CardKind::Weapon, 91_010);
    let armor0 = equip(&mut state, target0, CardKind::Armor, 91_011);
    let weapon1 = equip(&mut state, target1, CardKind::Weapon, 91_012);

    let (state, offered) = activate(state, owner, vec![target0, target1], 10);
    assert!(matches!(offered.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::DiscardEquipmentPairWithArmor
    );
    assert_eq!(option_tokens(&offered), vec![armor0.raw()]);
    let (_, cancelled) = resolve(state.clone(), owner, Vec::new(), true, 11);
    assert!(matches!(cancelled.result, CommandResult::Rejected { .. }));

    let (state, first) = resolve(state, owner, vec![armor0.raw()], false, 12);
    assert!(matches!(first.result, CommandResult::Accepted { .. }));
    assert_eq!(option_tokens(&first), vec![weapon1.raw()]);
    let (state, completed) = resolve(state, owner, vec![weapon1.raw()], false, 13);
    assert!(matches!(completed.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(armor0).unwrap().zone, discard);
    assert_eq!(state.cards.get(weapon1).unwrap().zone, discard);
    assert_ne!(state.cards.get(weapon0).unwrap().zone, discard);
    let moved = completed
        .events
        .iter()
        .filter(|event| matches!(event, engine_api::DomainEvent::CardsMoved { .. }))
        .count();
    assert_eq!(moved, 2);
    assert_eq!(
        state
            .skills
            .sorted_live()
            .into_iter()
            .find(|skill| skill.def.raw() == STDTIAOHE)
            .unwrap()
            .use_count,
        1,
        "successful activation consumes its once-per-turn use exactly once"
    );
}

#[test]
fn non_armor_first_selection_forces_armor_on_second_target() {
    let (mut state, [owner, target0, target1]) = setup();
    let weapon0 = equip(&mut state, target0, CardKind::Weapon, 91_020);
    let armor1 = equip(&mut state, target1, CardKind::Armor, 91_021);
    let weapon1 = equip(&mut state, target1, CardKind::Weapon, 91_022);
    let (state, offered) = activate(state, owner, vec![target0, target1], 20);
    assert_eq!(option_tokens(&offered), vec![weapon0.raw()]);
    let (state, first) = resolve(state, owner, vec![weapon0.raw()], false, 21);
    assert_eq!(option_tokens(&first), vec![armor1.raw()]);
    assert!(!option_tokens(&first).contains(&weapon1.raw()));
    let (state, completed) = resolve(state, owner, vec![armor1.raw()], false, 22);
    assert!(matches!(completed.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
}

#[test]
fn discard_protection_removes_equipment_from_the_pair_contract() {
    let (mut state, [owner, target0, target1]) = setup();
    equip(&mut state, target0, CardKind::Armor, 91_030);
    equip(&mut state, target1, CardKind::Weapon, 91_031);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: target0,
            skill_def: SkillDefId::new(PROTECT),
        }],
    )
    .unwrap();
    let state = tx.commit().state;
    let (_, rejected) = activate(state, owner, vec![target0, target1], 30);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
}
