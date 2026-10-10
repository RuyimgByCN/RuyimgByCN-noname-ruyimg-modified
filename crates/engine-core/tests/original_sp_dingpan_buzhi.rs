//! Golden coverage for SP 步骘「定叛」rewritten from legacy JavaScript semantics.

use content_compiler::compile_source_named;
use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Identity, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::{choose_command, Engine, Snapshot};
use rule_api::EffectRequest;

const DINGPAN: u32 = 3_739_676_314;
const HONGDE: u32 = 3_138_095_197;
const BUZHI: u32 = 4_110_519_851;
const PROTECT_EQUIP: u32 = 4_200_000_201;
const DIE_ON_MOVE: u32 = 4_200_000_202;
const DRAW_PROBE: u32 = 4_200_000_203;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/dingpan.skill");
const HONGDE_SOURCE: &str = include_str!("../../../content/original/ported/sp/hongde.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");
const MODULE_LOGIC_HASH: [u8; 32] = [
    93, 161, 222, 203, 230, 32, 163, 105,
    106, 44, 219, 70, 1, 166, 164, 166,
    246, 41, 189, 231, 46, 141, 42, 198,
    125, 110, 100, 76, 201, 124, 148, 222,
];
const PACKAGE_LOGIC_HASH: [u8; 32] = [104,11,251,180,248,79,187,16,89,76,180,14,141,177,202,117,119,248,212,56,232,136,9,34,34,192,63,77,154,118,188,178];
const HELPERS: &str = r#"
skill protect_equip(4200000201) locked {
  modifier card_discard_protected(equip) override 1;
}

skill die_on_move(4200000202) locked {
  trigger cards_moved owner_alive {
    effect lose_hp(self, 99);
  }
}

skill draw_probe(4200000203) {
  trigger cards_drawn owner_is_target optional {
    effect lose_hp(self, 1);
  }
}
"#;

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    other: PlayerId,
    other_rebel: PlayerId,
    skill: SkillId,
    equipment: Vec<CardId>,
}

fn inject_equipment(state: &mut GameState, player: PlayerId, rank: u8) -> CardId {
    let zone = state.zones.find(ZoneKind::Equip, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_300 + u32::from(rank)),
        kind: CardKind::Weapon,
        suit: Suit::Spade,
        rank,
        weapon_range: 2,
        zone,
        owner: player,
        face_up: true,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(
    self_target: bool,
    equipment_count: u8,
    owner_helpers: &[u32],
    target_helpers: &[u32],
) -> Setup {
    let package = compile_source_named(
        "sp-dingpan-test",
        &format!("{SOURCE}\n{HONGDE_SOURCE}\n{HELPERS}"),
    )
    .unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 5, 2_026_073_002, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[1];
    let target = if self_target { owner } else { players[2] };
    let other = players[0];
    let other_rebel = players[3];
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
        skill_def: SkillDefId::new(DINGPAN),
    });
    effects.extend(
        owner_helpers
            .iter()
            .map(|definition| EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(*definition),
            }),
    );
    effects.extend(
        target_helpers
            .iter()
            .filter(|definition| !self_target || !owner_helpers.contains(definition))
            .map(|definition| EffectRequest::AddSkill {
                player: target,
                skill_def: SkillDefId::new(*definition),
            }),
    );
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        tx.state_mut().players.get_mut(*player).unwrap().identity = Identity::Loyalist;
    }
    tx.state_mut().players.get_mut(other).unwrap().identity = Identity::Lord;
    tx.state_mut().players.get_mut(owner).unwrap().identity = Identity::Rebel;
    tx.state_mut()
        .players
        .get_mut(other_rebel)
        .unwrap()
        .identity = Identity::Rebel;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().revision = 1;
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == DINGPAN)
        .unwrap();
    let mut state = tx.commit().state;
    let equipment = (0..equipment_count)
        .map(|index| inject_equipment(&mut state, target, index + 1))
        .collect();
    Setup {
        state,
        owner,
        target,
        other,
        other_rebel,
        skill,
        equipment,
    }
}

fn command(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn activate(setup: Setup, key: u64) -> (GameState, PipelineResult) {
    command(
        setup.state,
        setup.owner,
        CommandKind::ActivateSkill {
            skill: setup.skill,
            targets: vec![setup.target],
            card_targets: Vec::new(),
        },
        key,
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    command(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled,
        },
        key,
    )
}

fn pass_dying_choices(
    mut state: GameState,
    mut result: PipelineResult,
    mut key: u64,
) -> (GameState, Vec<DomainEvent>) {
    let mut events = result.events;
    while state.has_pending_choice() {
        assert_eq!(
            state.resolution.continuation_kind,
            engine_core::ContinuationKind::DyingSave
        );
        let actor = PlayerId::new(state.resolution.pending_observer);
        (state, result) = resolve(state, actor, vec![0], false, key);
        key += 1;
        events.extend(result.events);
    }
    (state, events)
}

#[test]
fn package_module_and_character_contract_are_stable() {
    let compiled = compile_source_named("sp-dingpan", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 302);
    let active = &compiled.active_index[0];
    assert_eq!(active.skill_def_id, DINGPAN);
    assert_eq!(
        active.uses_per_turn_source.as_deref(),
        Some("living_rebel_count")
    );
    assert!(active
        .targets
        .as_ref()
        .is_some_and(|targets| targets.allow_self && targets.has_equip));
    let module = &compiled.modules[0];
    assert_eq!(module.code, [83, 85, 90, 33, 1, 1]);
    assert_eq!(module.logic_hash, MODULE_LOGIC_HASH);
    assert_eq!(rule_vm::bytecode::effect_tag::RESOLVE_DINGPAN, 289);
    assert_eq!(
        engine_core::ContinuationKind::ChooseDingpanBranch.tag(),
        207
    );

    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 745);
    assert_eq!(package.manifest.logic_hash, PACKAGE_LOGIC_HASH);
    assert_eq!(package.manifest.logic_hash, package.compute_logic_hash());
    assert_eq!(package.skill_index.len(), 728);
    assert_eq!(package.character_index.len(), 217);
    assert_eq!(package.modules.len(), 295);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == BUZHI)
        .unwrap();
    assert_eq!(character.name_key, "buzhi");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "wu");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [HONGDE, DINGPAN]);
    assert!(!serde_json::to_string(&package).unwrap().contains(".js"));
}

#[test]
fn living_rebel_limit_is_dynamic_zero_forbids_and_extra_play_does_not_reset() {
    let fixture = setup(false, 1, &[], &[]);
    let owner = fixture.owner;
    let target = fixture.target;
    let other_rebel = fixture.other_rebel;
    let skill = fixture.skill;
    let equipment = fixture.equipment[0];
    let (state, result) = activate(fixture, 10);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let (state, _) = resolve(state, target, vec![0], false, 11);
    let (mut state, result) = resolve(state, owner, vec![equipment.raw()], false, 12);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.skills.get(skill).unwrap().use_count, 1);

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::Discard.to_u8(),
            },
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::Play.to_u8(),
            },
        ],
    )
    .unwrap();
    state = tx.commit().state;
    assert_eq!(state.skills.get(skill).unwrap().use_count, 1);
    inject_equipment(&mut state, target, 8);

    let allowed = Setup {
        state: state.clone(),
        owner,
        target,
        other: PlayerId::NONE,
        other_rebel,
        skill,
        equipment: Vec::new(),
    };
    let (_, result) = activate(allowed, 13);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));

    state.players.get_mut(other_rebel).unwrap().alive = false;
    let reduced = Setup {
        state,
        owner,
        target,
        other: PlayerId::NONE,
        other_rebel,
        skill,
        equipment: Vec::new(),
    };
    let (_, result) = activate(reduced, 14);
    assert!(matches!(result.result, CommandResult::Rejected { .. }));

    let mut zero = setup(false, 1, &[], &[]);
    for player in zero
        .state
        .players
        .iter_live()
        .map(|player| player.id)
        .collect::<Vec<_>>()
    {
        zero.state.players.get_mut(player).unwrap().identity = Identity::Loyalist;
    }
    let (_, result) = activate(zero, 15);
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
}

#[test]
fn headless_runner_retries_dynamic_skill_while_rebel_limit_remains() {
    let fixture = setup(false, 1, &[], &[]);
    let owner = fixture.owner;
    let target = fixture.target;
    let skill = fixture.skill;
    let equipment = fixture.equipment[0];
    let (state, _) = activate(fixture, 16);
    let (state, _) = resolve(state, target, vec![0], false, 17);
    let (mut state, _) = resolve(state, owner, vec![equipment.raw()], false, 18);
    inject_equipment(&mut state, target, 9);

    let mut engine = Engine::new();
    engine.restore_snapshot(&Snapshot::capture(&state)).unwrap();
    let command = choose_command(&engine);
    assert!(matches!(
        command.kind,
        CommandKind::ActivateSkill {
            skill: selected,
            ..
        } if selected == skill
    ));
    assert!(matches!(
        engine.submit(command),
        CommandResult::Accepted { .. }
    ));
}

#[test]
fn discard_branch_revalidates_protection_and_never_deals_damage() {
    let protected = setup(false, 1, &[], &[PROTECT_EQUIP]);
    let target = protected.target;
    let equipment = protected.equipment[0];
    let hp = protected.state.players.get(target).unwrap().hp;
    let (state, _) = activate(protected, 20);
    let (state, result) = resolve(state, target, vec![0], false, 21);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(
        state.cards.get(equipment).unwrap().zone,
        state.zones.find(ZoneKind::Equip, target).unwrap()
    );
    assert_eq!(state.players.get(target).unwrap().hp, hp);
    assert!(!result
        .events
        .iter()
        .any(|event| matches!(event, DomainEvent::DamageDealt { .. })));
}

#[test]
fn activation_and_discard_branch_reject_forged_inputs_then_discard_one_equip() {
    let (_, result) = activate(setup(false, 0, &[], &[]), 22);
    assert!(matches!(result.result, CommandResult::Rejected { .. }));

    let fixture = setup(false, 1, &[], &[]);
    let owner = fixture.owner;
    let target = fixture.target;
    let equipment = fixture.equipment[0];
    let (state, result) = activate(fixture, 23);
    assert_eq!(result.choice.as_ref().unwrap().kind, ChoiceKind::Custom);

    let (state, result) = resolve(state, target, Vec::new(), true, 24);
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        engine_core::ContinuationKind::ChooseDingpanBranch
    );
    let (state, result) = resolve(state, target, vec![2], false, 25);
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        engine_core::ContinuationKind::ChooseDingpanBranch
    );

    let (state, result) = resolve(state, target, vec![0], false, 26);
    let choice = result.choice.as_ref().unwrap();
    assert_eq!(choice.observer, owner);
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert_eq!(
        state.resolution.continuation_kind,
        engine_core::ContinuationKind::ChooseDiscardTargetCard
    );

    let (state, result) = resolve(state, owner, vec![u32::MAX], false, 27);
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        engine_core::ContinuationKind::ChooseDiscardTargetCard
    );
    let (state, result) = resolve(state, owner, vec![equipment.raw()], false, 28);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(equipment).unwrap().zone, discard);
    assert!(!result.events.iter().any(|event| {
        matches!(
            event,
            DomainEvent::DamageDealt {
                is_lose_hp: false,
                ..
            }
        )
    }));
}

#[test]
fn draw_trigger_choice_and_effect_finish_before_branch_choice() {
    let fixture = setup(false, 1, &[], &[DRAW_PROBE]);
    let target = fixture.target;
    let hp = fixture.state.players.get(target).unwrap().hp;
    let (state, result) = activate(fixture, 25);
    assert_eq!(result.choice.as_ref().unwrap().kind, ChoiceKind::Confirm);
    assert_ne!(
        state.resolution.continuation_kind,
        engine_core::ContinuationKind::ChooseDingpanBranch
    );
    assert_eq!(state.players.get(target).unwrap().hp, hp);

    let (state, result) = resolve(state, target, vec![1], false, 26);
    assert_eq!(result.choice.as_ref().unwrap().kind, ChoiceKind::Custom);
    assert_eq!(
        state.resolution.continuation_kind,
        engine_core::ContinuationKind::ChooseDingpanBranch
    );
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
}

#[test]
fn gain_branch_moves_one_batch_hongde_finishes_before_damage_and_empty_still_hurts() {
    let fixture = setup(true, 2, &[HONGDE], &[]);
    let owner = fixture.owner;
    let other = fixture.other;
    let equipment = fixture.equipment.clone();
    let hp = fixture.state.players.get(owner).unwrap().hp;
    let (state, _) = activate(fixture, 30);
    let (state, result) = resolve(state, owner, vec![1], false, 31);
    assert_eq!(result.choice.as_ref().unwrap().kind, ChoiceKind::Confirm);
    assert_eq!(state.players.get(owner).unwrap().hp, hp);
    assert!(equipment.iter().all(|card| {
        state.cards.get(*card).unwrap().zone == state.zones.find(ZoneKind::Hand, owner).unwrap()
    }));
    let moves: Vec<_> = result
        .events
        .iter()
        .filter_map(|event| match event {
            DomainEvent::CardsMoved { cards, .. } if cards == &equipment => Some(cards.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(moves, [equipment]);

    let (state, result) = resolve(state, owner, vec![1], false, 32);
    assert_eq!(
        result.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectTargets
    );
    assert_eq!(state.players.get(owner).unwrap().hp, hp);
    let (state, result) = resolve(state, owner, vec![other.raw()], false, 33);
    let drawn = result
        .events
        .iter()
        .position(
            |event| matches!(event, DomainEvent::CardsDrawn { player, .. } if *player == other),
        )
        .unwrap();
    let damaged = result
        .events
        .iter()
        .position(
            |event| matches!(event, DomainEvent::DamageDealt { target, .. } if *target == owner),
        )
        .unwrap();
    assert!(drawn < damaged);
    assert_eq!(state.players.get(owner).unwrap().hp, hp - 1);

    let empty = setup(false, 1, &[], &[]);
    let target = empty.target;
    let hp = empty.state.players.get(target).unwrap().hp;
    let equip = empty.equipment[0];
    let (mut state, _) = activate(empty, 34);
    let hand = state.zones.find(ZoneKind::Hand, target).unwrap();
    let equip_zone = state.zones.find(ZoneKind::Equip, target).unwrap();
    state.zones.get_mut(equip_zone).unwrap().cards.clear();
    state.zones.get_mut(hand).unwrap().cards.push(equip);
    let card = state.cards.get_mut(equip).unwrap();
    card.zone = hand;
    card.owner = target;
    card.face_up = false;
    let (state, result) = resolve(state, target, vec![1], false, 35);
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
    assert!(result.events.iter().any(|event| {
        matches!(event, DomainEvent::DamageDealt { target: damaged, .. } if *damaged == target)
    }));
}

#[test]
fn movement_death_makes_damage_source_free_and_dead_target_skips_damage() {
    let source_dies = setup(false, 1, &[DIE_ON_MOVE], &[]);
    let owner = source_dies.owner;
    let target = source_dies.target;
    let target_hp = source_dies.state.players.get(target).unwrap().hp;
    let (state, _) = activate(source_dies, 40);
    let (state, result) = resolve(state, target, vec![1], false, 41);
    let (state, events) = pass_dying_choices(state, result, 42);
    assert!(!state.players.get(owner).unwrap().alive);
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 1);
    assert!(events.iter().any(|event| {
        matches!(
            event,
            DomainEvent::DamageDealt { source, target: damaged, .. }
                if source.is_none() && *damaged == target
        )
    }));

    let target_dies = setup(false, 1, &[], &[DIE_ON_MOVE]);
    let target = target_dies.target;
    let (state, _) = activate(target_dies, 50);
    let (state, result) = resolve(state, target, vec![1], false, 51);
    let (state, events) = pass_dying_choices(state, result, 52);
    assert!(!state.players.get(target).unwrap().alive);
    assert!(!events.iter().any(|event| {
        matches!(
            event,
            DomainEvent::DamageDealt {
                target: damaged,
                is_lose_hp: false,
                ..
            } if *damaged == target
        )
    }));
}
